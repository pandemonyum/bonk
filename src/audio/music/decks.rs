//! I due deck della musica come li vede il direttore, senza Bevy (progetto § 3.7): quale traccia
//! ha ogni deck, a che punto è la sua dissolvenza e se il suo sink è nato. [`Decks::advance`] fa un
//! passo di tempo reale verso la traccia voluta e dice cosa creare e cosa togliere.
//!
//! Dissolvenza equal-power: ogni deck ha una fase in [0, 1] e il livello è sen(fase·π/2). Sul deck
//! della traccia voluta la fase sale, sugli altri scende, alla stessa velocità (1 / durata): in
//! entrata fase = t e livello sen(t·π/2), in uscita fase = 1 − t e livello cos(t·π/2), quindi la
//! somma delle potenze resta 1. Un deck che torna voluto mentre esce rientra dalla fase in cui è,
//! senza ripartire da capo.
//!
//! Regole:
//! - al più [`MAX_DECKS`] deck, mai due con la stessa traccia (nessun loop doppio);
//! - la traccia voluta senza deck ne prende uno libero, a livello zero; se sono occupati tutti e
//!   due, aspetta che uno arrivi a zero;
//! - finché il sink del deck voluto non è nato (caricamento lento) la dissolvenza è ferma: non
//!   salta, e il deck uscente resta al suo livello;
//! - un deck che non è voluto e arriva a zero (o non ha mai suonato) si toglie.

use std::f32::consts::FRAC_PI_2;

use crate::audio::catalog::TrackId;
use crate::audio::focus::fade_toward;

/// Deck vivi insieme al massimo: la traccia che entra e quella che esce (D-071: sul web due flussi
/// Vorbis al massimo).
pub const MAX_DECKS: usize = 2;

/// Livello in entrata dopo la frazione `t` della dissolvenza: sen(t·π/2).
pub fn fade_in(t: f32) -> f32 {
    (t.clamp(0.0, 1.0) * FRAC_PI_2).sin()
}

/// Livello in uscita dopo la frazione `t` della dissolvenza: cos(t·π/2), scritto come
/// sen((1 − t)·π/2) perché a `t = 1` sia zero esatto.
pub fn fade_out(t: f32) -> f32 {
    fade_in(1.0 - t.clamp(0.0, 1.0))
}

/// Uno dei due posti per un deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    A,
    B,
}

impl Slot {
    pub const ALL: [Slot; MAX_DECKS] = [Slot::A, Slot::B];

    pub fn index(self) -> usize {
        match self {
            Slot::A => 0,
            Slot::B => 1,
        }
    }

    /// Per il log.
    pub fn name(self) -> &'static str {
        match self {
            Slot::A => "A",
            Slot::B => "B",
        }
    }
}

/// Un deck vivo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deck {
    pub slot: Slot,
    pub track: TrackId,
    /// Fase della dissolvenza in [0, 1]: il livello è [`fade_in`] della fase.
    pub phase: f32,
    /// Il sink è nato: il deck suona (anche a volume zero).
    pub sounding: bool,
}

impl Deck {
    /// Livello della dissolvenza, 0–1, che moltiplica il volume del deck.
    pub fn level(&self) -> f32 {
        fade_in(self.phase)
    }
}

/// Cosa è successo in un passo, per la riproduzione e per il log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckEvent {
    /// Un deck libero prende la traccia voluta, a livello zero: va creato e aspetta il sink.
    Spawn { slot: Slot, track: TrackId },
    /// La traccia voluta è arrivata a pieno livello.
    Full { slot: Slot, track: TrackId },
    /// Un deck non voluto è arrivato a zero (o non ha mai suonato): va tolto.
    Despawn { slot: Slot, track: TrackId },
}

/// I deck vivi, al più [`MAX_DECKS`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Decks {
    decks: Vec<Deck>,
}

impl Decks {
    pub fn iter(&self) -> impl Iterator<Item = &Deck> {
        self.decks.iter()
    }

    pub fn len(&self) -> usize {
        self.decks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.decks.is_empty()
    }

    pub fn get(&self, slot: Slot) -> Option<&Deck> {
        self.decks.iter().find(|deck| deck.slot == slot)
    }

    /// Il deck che ha la traccia `track`.
    pub fn find(&self, track: TrackId) -> Option<&Deck> {
        self.decks.iter().find(|deck| deck.track == track)
    }

    /// Il sink del deck `slot` è nato.
    pub fn set_sounding(&mut self, slot: Slot) {
        if let Some(deck) = self.decks.iter_mut().find(|deck| deck.slot == slot) {
            deck.sounding = true;
        }
    }

    /// Il deck `slot` non c'è più (entità tolta da fuori, file che non si carica): esce subito.
    pub fn forget(&mut self, slot: Slot) -> Option<Deck> {
        let index = self.decks.iter().position(|deck| deck.slot == slot)?;
        Some(self.decks.remove(index))
    }

    /// La dissolvenza aspetta: la traccia voluta ha un deck ma il suo sink non è ancora nato.
    pub fn waiting(&self, target: Option<TrackId>) -> bool {
        target
            .and_then(|track| self.find(track))
            .is_some_and(|deck| !deck.sounding)
    }

    /// Un passo di `dt` secondi verso `target` (`None`: silenzio), con la dissolvenza di `fade_s`
    /// secondi (0: subito).
    pub fn advance(&mut self, target: Option<TrackId>, fade_s: f32, dt: f32) -> Vec<DeckEvent> {
        let mut events = Vec::new();
        self.drop_silent(target, &mut events);
        self.take_free(target, &mut events);
        if !self.waiting(target) {
            for deck in &mut self.decks {
                let wanted = Some(deck.track) == target;
                let before = deck.phase;
                let goal = if wanted { 1.0 } else { 0.0 };
                deck.phase = fade_toward(deck.phase, goal, dt, fade_s);
                if wanted && before < 1.0 && deck.phase >= 1.0 {
                    events.push(DeckEvent::Full {
                        slot: deck.slot,
                        track: deck.track,
                    });
                }
            }
        }
        // un posto liberato adesso si riprende subito
        self.drop_silent(target, &mut events);
        self.take_free(target, &mut events);
        events
    }

    /// Toglie i deck non voluti a zero.
    fn drop_silent(&mut self, target: Option<TrackId>, events: &mut Vec<DeckEvent>) {
        self.decks.retain(|deck| {
            let keep = Some(deck.track) == target || deck.phase > 0.0;
            if !keep {
                events.push(DeckEvent::Despawn {
                    slot: deck.slot,
                    track: deck.track,
                });
            }
            keep
        });
    }

    /// La traccia voluta senza deck prende il primo posto libero, se c'è.
    fn take_free(&mut self, target: Option<TrackId>, events: &mut Vec<DeckEvent>) {
        let Some(track) = target else {
            return;
        };
        if self.find(track).is_some() {
            return;
        }
        let Some(slot) = Slot::ALL.into_iter().find(|&slot| self.get(slot).is_none()) else {
            return;
        };
        self.decks.push(Deck {
            slot,
            track,
            phase: 0.0,
            sounding: false,
        });
        events.push(DeckEvent::Spawn { slot, track });
    }
}
