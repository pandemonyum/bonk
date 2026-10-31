//! Le regole di un avvio (progetto § 3.5), nell'ordine: cooldown a credito → istanze → posti (al
//! più un furto) → margine (prima via il boost, poi furti) → variante, velocità e guadagno.
//!
//! **Cooldown a credito.** Un cue riparte quando `now >= next_allowed`. Un cooldown rigido
//! (`now - ultimo >= cooldown`) si arrotonda al fotogramma successivo e il ritardo si somma a ogni
//! avvio: con 35 ms dava 15, 20 e 22 avvii al secondo a 30, 60 e 144 FPS. Qui il ritardo dovuto al
//! fotogramma (al più la durata dell'ultimo fotogramma) si recupera all'avvio dopo, così il
//! cooldown è un intervallo **medio** e gli avvii al secondo non dipendono dal frame rate.
//! L'intervallo minimo tra due avvii è `cooldown - durata del fotogramma`, e mai meno di un
//! fotogramma (una voce per cue per fotogramma): con cooldown più brevi di un fotogramma a
//! 30 FPS (< 34 ms) il limite vero è un avvio per fotogramma.

use super::voices::VoiceId;
use super::{Decision, Mixer, Voice, boost_db, db_to_gain};
use crate::audio::catalog::{Coalesce, CueData, CueId, Cull, OnLimit, Priority};

/// Regole di un cue, copiate dal catalogo.
#[derive(Debug, Clone, Copy)]
pub(super) struct Rules {
    pub(super) priority: Priority,
    cooldown_s: f64,
    max_instances: usize,
    on_limit: OnLimit,
    boost_max_db: Option<f32>,
    pub(super) cull_view: bool,
    gain: f32,
    trim: f32,
    jitter_db: f32,
    pitch: (f32, f32),
    expiry_s: f64,
    variants: usize,
}

impl Rules {
    pub(super) fn of(data: &CueData, trim: f32) -> Self {
        Self {
            priority: data.priority,
            cooldown_s: f64::from(data.cooldown_ms) / 1000.0,
            max_instances: data.max_instances as usize,
            on_limit: data.on_limit,
            boost_max_db: match data.coalesce {
                Coalesce::Single => None,
                Coalesce::Boost { max_db } => Some(max_db),
            },
            cull_view: data.cull == Cull::View,
            gain: data.gain,
            trim,
            jitter_db: data.gain_jitter_db,
            pitch: data.pitch,
            expiry_s: f64::from(data.length_s) + super::EXPIRY_GRACE_S,
            variants: data.files.len(),
        }
    }
}

/// Primo istante in cui un cue può ripartire dopo un avvio a `now`: al primo avvio
/// (`previous` = `None`) `now + cooldown`, poi `max(previous + cooldown, now + cooldown − lag)`,
/// con `lag` = durata dell'ultimo fotogramma limitata a `[0, cooldown]`.
pub fn next_allowed(previous: Option<f64>, now: f64, cooldown: f64, frame_dt: f64) -> f64 {
    let lag = frame_dt.clamp(0.0, cooldown);
    match previous {
        None => now + cooldown,
        Some(next) => (next + cooldown).max(now + cooldown - lag),
    }
}

impl Mixer {
    /// Le regole per una richiesta coalescente; `true` se la voce parte.
    pub(super) fn try_start(
        &mut self,
        now: f64,
        cue: CueId,
        count: u32,
        out: &mut Vec<Decision>,
    ) -> bool {
        let index = cue.index();
        let rules = self.rules[index];
        let critical = rules.priority == Priority::Critical;
        if self.next_allowed[index].is_some_and(|next| now < next) {
            self.stats.cooldown += 1;
            return false;
        }

        // istanze: con `Restart` si fermano le più vecchie dello stesso cue
        let mut leaving: Vec<VoiceId> = Vec::new();
        let live = self.voices.of_cue(cue).count();
        if live >= rules.max_instances {
            if rules.on_limit == OnLimit::Drop {
                self.stats.instances += 1;
                return false;
            }
            let excess = live + 1 - rules.max_instances;
            leaving.extend(self.voices.of_cue(cue).take(excess).map(|v| v.id));
        }
        let restarted = leaving.len();

        // posti: le non `Critical` contro `max - reserved_critical`, tutte contro `max`; a posti
        // finiti si ruba una voce sola, la più bassa (e più vecchia) di priorità minore
        let total = self.voices.len() - restarted;
        let non_critical = self.voices.non_critical() - if critical { 0 } else { restarted };
        let limits = self.limits;
        let full = |total: usize, non_critical: usize| {
            total >= limits.max_voices || (!critical && non_critical >= limits.non_critical_seats())
        };
        let mut seat_stolen = false;
        if full(total, non_critical) {
            let victim = self
                .voices
                .steal_order(rules.priority, &leaving)
                .first()
                .copied();
            match victim {
                // la vittima ha priorità minore, quindi non è mai `Critical`
                Some(victim) if !full(total - 1, non_critical.saturating_sub(1)) => {
                    leaving.push(victim.id);
                    seat_stolen = true;
                }
                _ => {
                    self.stats.seats += 1;
                    return false;
                }
            }
        }

        // margine (§ 12.9): una non `Critical` oltre il tetto perde il boost, poi ferma voci di
        // priorità minore finché ci sta, altrimenti si scarta. Le `Critical` partono sempre.
        let jitter = db_to_gain(self.rng.range_f32(-rules.jitter_db, rules.jitter_db));
        let base = rules.gain * jitter;
        let mut boost = rules
            .boost_max_db
            .map_or(1.0, |max_db| db_to_gain(boost_db(count, max_db)));
        let mut boost_cut = false;
        let mut margin_stolen = 0;
        if !critical {
            let headroom = limits.sfx_headroom;
            let fits = |leaving: &[VoiceId], gain: f32, voices: &super::VoiceTable| {
                voices.mix_gain(leaving) + gain * rules.trim <= headroom
            };
            if !fits(&leaving, base * boost, &self.voices) {
                boost_cut = boost > 1.0;
                boost = 1.0;
                let mut victims = self
                    .voices
                    .steal_order(rules.priority, &leaving)
                    .into_iter();
                while !fits(&leaving, base, &self.voices) {
                    let Some(victim) = victims.next() else {
                        self.stats.headroom += 1;
                        return false;
                    };
                    leaving.push(victim.id);
                    margin_stolen += 1;
                }
            }
        }

        // la voce parte: prima si fermano quelle che lasciano il posto
        for &voice in &leaving {
            self.voices.remove(voice);
            out.push(Decision::Stop { voice });
        }
        self.stats.restarted += restarted as u64;
        self.stats.stolen += u64::from(seat_stolen);
        self.stats.headroom_stolen += margin_stolen;
        self.stats.boost_cut += u64::from(boost_cut);
        let variant = self.pickers[index].pick(rules.variants, &mut self.rng);
        let speed = self.rng.range_f32(rules.pitch.0, rules.pitch.1);
        let gain = base * boost;
        let voice = VoiceId(self.next_voice);
        self.next_voice += 1;
        self.voices.push(Voice {
            id: voice,
            cue,
            priority: rules.priority,
            started: now,
            expires: now + rules.expiry_s,
            playing: false,
            mix_gain: gain * rules.trim,
        });
        self.next_allowed[index] = Some(next_allowed(
            self.next_allowed[index],
            now,
            rules.cooldown_s,
            self.frame_dt,
        ));
        self.stats.started += 1;
        out.push(Decision::Start {
            voice,
            cue,
            variant,
            gain,
            speed,
        });
        true
    }
}
