//! Politica delle voci (progetto § 3.5 e § 3.6), pura: nessun tipo di `bevy_audio`, l'orologio
//! arriva come argomento (secondi di tempo reale). Ogni fotogramma [`Mixer::decide`] riceve le
//! richieste e restituisce quali voci avviare e quali fermare; la riproduzione (AUDIO-2) esegue
//! le decisioni, dice con [`Mixer::playing`] quando una voce suona davvero e con
//! [`Mixer::finished`] quando finisce.
//!
//! Ordine delle regole: scadenze → coalescenza (una voce per cue nel fotogramma, le richieste
//! fuori vista dei cue `cull: View` non entrano) → priorità decrescente, poi id → per ciascuna,
//! finché il budget del fotogramma lo permette (le `Critical` non lo consumano): cooldown →
//! istanze → posti → margine → variante, velocità e guadagno (`start`). Una richiesta scartata
//! non cambia nulla: le voci da fermare si fermano solo se la nuova parte davvero.

use std::collections::BTreeMap;

use bevy::math::Rect;

use super::catalog::{AudioCatalog, CueId, Priority, VoiceLimits};
use super::cues::CueRequest;
use crate::rng::Pcg32;

mod start;
mod voices;
pub use start::next_allowed;
pub use voices::{VariantPicker, Voice, VoiceId, VoiceTable};

/// Seed fissa del caso dell'audio: non dipende dalla seed della run e i test sono ripetibili.
pub const AUDIO_SEED: u64 = 0xA0D1_0000_5EED;
/// Stream di `Pcg32` dell'audio, lontano da quelli della simulazione (0–4: spawn 1, draft 2,
/// orde 3, forzieri 4). Il generatore è comunque un'istanza a parte.
pub const AUDIO_STREAM: u64 = 0xA0D1;
/// Secondi oltre la durata stimata dopo cui una voce che non ha mai suonato si toglie.
pub const EXPIRY_GRACE_S: f64 = 0.5;

/// Istante in cui una voce avviata a `started` che non ha mai suonato scade: la sua durata
/// stimata `length_s` più [`EXPIRY_GRACE_S`]. La stessa regola per il mixer e per la
/// riproduzione, che così riconosce le fermate per scadenza.
pub fn expiry_time(started: f64, length_s: f32) -> f64 {
    started + (f64::from(length_s) + EXPIRY_GRACE_S)
}
/// Boost di coalescenza per ogni raddoppio della quantità, in dB (fino a `max_db` del cue).
pub const BOOST_DB_PER_DOUBLING: f32 = 1.5;

/// Cosa la riproduzione deve fare.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decision {
    /// Avviare la variante `variant` del cue al guadagno `gain` (cue × variazione × boost, senza
    /// trim né cursori) e alla velocità `speed`.
    Start {
        voice: VoiceId,
        cue: CueId,
        variant: usize,
        gain: f32,
        speed: f32,
    },
    /// Fermare una voce: furto (per un posto o per il margine), `Restart`, oppure scadenza di
    /// una voce che non ha mai suonato.
    Stop { voice: VoiceId },
}

/// Contatori diagnostici (overlay F3 in AUDIO-5). Le richieste scartate si contano una volta per
/// cue e fotogramma, come le voci che sarebbero partite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MixerStats {
    /// Richieste ricevute (prima della coalescenza).
    pub requests: u64,
    pub started: u64,
    /// Scartate perché tutte fuori vista (`cull: View`).
    pub culled: u64,
    pub cooldown: u64,
    /// Scartate per `max_instances` con `on_limit: Drop`.
    pub instances: u64,
    /// Scartate per mancanza di posti, senza una voce da rubare.
    pub seats: u64,
    /// Non `Critical` scartate per il margine, senza abbastanza voci più basse da fermare.
    pub headroom: u64,
    /// Voci partite senza il boost per stare nel margine.
    pub boost_cut: u64,
    /// Non `Critical` scartate oltre `starts_per_frame`.
    pub frame_budget: u64,
    /// Voci fermate per un posto (al più una per avvio).
    pub stolen: u64,
    /// Voci fermate per far stare una voce più alta nel margine.
    pub headroom_stolen: u64,
    /// Voci fermate da `on_limit: Restart`.
    pub restarted: u64,
    /// Voci mai suonate tolte a `length_s + 0,5 s`.
    pub expired: u64,
    pub finished: u64,
    /// Richieste con un id che il catalogo non ha.
    pub unknown: u64,
}

/// Limiti in uso: quelli del desktop o del web.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub max_voices: usize,
    pub reserved_critical: usize,
    pub starts_per_frame: usize,
    pub sfx_headroom: f32,
}

impl Limits {
    pub fn native(voices: &VoiceLimits) -> Self {
        Self::pick(voices, voices.max, voices.starts_per_frame)
    }

    pub fn web(voices: &VoiceLimits) -> Self {
        Self::pick(voices, voices.max_web, voices.starts_per_frame_web)
    }

    /// Quelli della piattaforma per cui è compilato il gioco.
    pub fn for_target(voices: &VoiceLimits) -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::web(voices)
        } else {
            Self::native(voices)
        }
    }

    fn pick(voices: &VoiceLimits, max: u32, starts: u32) -> Self {
        Self {
            max_voices: max as usize,
            reserved_critical: voices.reserved_critical as usize,
            starts_per_frame: starts as usize,
            sfx_headroom: voices.sfx_headroom,
        }
    }

    /// Posti delle voci non `Critical`: i riservati restano alle `Critical`.
    pub fn non_critical_seats(&self) -> usize {
        self.max_voices.saturating_sub(self.reserved_critical)
    }
}

/// Guadagno lineare di `db` decibel.
pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// Boost in dB di `count` richieste coalescenti: 1,5 dB per raddoppio, al massimo `max_db`.
pub fn boost_db(count: u32, max_db: f32) -> f32 {
    (BOOST_DB_PER_DOUBLING * (count.max(1) as f32).log2()).clamp(0.0, max_db.max(0.0))
}

/// Il mixer: voci vive, cooldown e varianti di ogni cue, caso proprio.
#[derive(Debug, Clone)]
pub struct Mixer {
    limits: Limits,
    rules: Vec<start::Rules>,
    voices: VoiceTable,
    /// Primo istante in cui ogni cue può ripartire (cooldown a credito, [`next_allowed`]).
    next_allowed: Vec<Option<f64>>,
    pickers: Vec<VariantPicker>,
    rng: Pcg32,
    next_voice: u64,
    /// Istante dell'ultimo `decide` e durata dell'ultimo fotogramma.
    last_decide: Option<f64>,
    frame_dt: f64,
    stats: MixerStats,
}

impl Mixer {
    /// Con i limiti della piattaforma e la seed dell'audio.
    pub fn new(catalog: &AudioCatalog) -> Self {
        Self::with(catalog, Limits::for_target(&catalog.voices), AUDIO_SEED)
    }

    pub fn with(catalog: &AudioCatalog, limits: Limits, seed: u64) -> Self {
        let rules: Vec<start::Rules> = catalog
            .cues()
            .iter()
            .map(|cue| start::Rules::of(&cue.data, catalog.trim(cue.data.bus)))
            .collect();
        Self {
            limits,
            next_allowed: vec![None; rules.len()],
            pickers: vec![VariantPicker::default(); rules.len()],
            rules,
            voices: VoiceTable::default(),
            rng: Pcg32::new(seed, AUDIO_STREAM),
            next_voice: 0,
            last_decide: None,
            frame_dt: 0.0,
            stats: MixerStats::default(),
        }
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    pub fn voices(&self) -> &VoiceTable {
        &self.voices
    }

    pub fn stats(&self) -> MixerStats {
        self.stats
    }

    /// La riproduzione dice che la voce suona (il sink esiste): da qui non scade più. `false` se
    /// la voce non c'è.
    pub fn playing(&mut self, voice: VoiceId) -> bool {
        match self.voices.get_mut(voice) {
            Some(voice) => {
                voice.playing = true;
                true
            }
            None => false,
        }
    }

    /// La riproduzione dice che la voce è finita (sink vuoto o fermato). `false` se non c'era.
    pub fn finished(&mut self, voice: VoiceId) -> bool {
        let found = self.voices.remove(voice).is_some();
        if found {
            self.stats.finished += 1;
        }
        found
    }

    /// Le decisioni del fotogramma per le richieste `requests` all'istante `now_s` (tempo reale)
    /// con la vista `view` (`None`: nessun cull).
    pub fn decide(
        &mut self,
        now_s: f64,
        requests: &[CueRequest],
        view: Option<Rect>,
    ) -> Vec<Decision> {
        self.frame_dt = self.last_decide.map_or(0.0, |last| (now_s - last).max(0.0));
        self.last_decide = Some(now_s);
        let mut out = Vec::new();
        for voice in self.voices.expire(now_s) {
            self.stats.expired += 1;
            out.push(Decision::Stop { voice: voice.id });
        }

        // coalescenza: quantità visibili per cue, in ordine di id
        let mut groups: BTreeMap<CueId, u32> = BTreeMap::new();
        for request in requests {
            let Some(rules) = self.rules.get(request.cue.index()) else {
                self.stats.unknown += 1;
                continue;
            };
            if request.count == 0 {
                continue;
            }
            self.stats.requests += 1;
            let visible = !rules.cull_view
                || match (view, request.at) {
                    (Some(view), Some(at)) => view.contains(at),
                    _ => true,
                };
            let count = groups.entry(request.cue).or_insert(0);
            if visible {
                *count = count.saturating_add(request.count);
            }
        }
        let mut pending: Vec<(Priority, CueId, u32)> = Vec::with_capacity(groups.len());
        for (cue, count) in groups {
            if count == 0 {
                self.stats.culled += 1;
            } else {
                pending.push((self.rules[cue.index()].priority, cue, count));
            }
        }
        // priorità decrescente, poi id crescente (stabile: `groups` è già in ordine di id)
        pending.sort_by_key(|&(priority, _, _)| std::cmp::Reverse(priority));

        // le `Critical` partono sempre: non consumano il budget del fotogramma
        let mut starts = 0;
        for (priority, cue, count) in pending {
            let critical = priority == Priority::Critical;
            if !critical && starts >= self.limits.starts_per_frame {
                self.stats.frame_budget += 1;
            } else if self.try_start(now_s, cue, count, &mut out) && !critical {
                starts += 1;
            }
        }
        out
    }
}
