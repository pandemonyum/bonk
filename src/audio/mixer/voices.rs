//! Voci vive e scelta delle varianti, per il mixer.

use crate::audio::catalog::{CueId, Priority};
use crate::rng::Pcg32;

/// Identificativo di una voce, crescente: un id più piccolo è una voce più vecchia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoiceId(pub u64);

/// Una voce viva per il mixer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    pub id: VoiceId,
    pub cue: CueId,
    pub priority: Priority,
    pub started: f64,
    /// Finché la voce non suona (nessun sink), oltre questo istante si toglie da sola.
    pub expires: f64,
    /// La riproduzione ha detto che suona ([`super::Mixer::playing`]): non scade più, finisce
    /// solo con [`super::Mixer::finished`], un furto o un `Restart`.
    pub playing: bool,
    /// Guadagno nel mix (con il trim del bus): conta per il margine.
    pub mix_gain: f32,
}

/// Le voci vive, dalla più vecchia alla più nuova.
#[derive(Debug, Clone, Default)]
pub struct VoiceTable {
    voices: Vec<Voice>,
}

impl VoiceTable {
    pub fn len(&self) -> usize {
        self.voices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.voices.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Voice> {
        self.voices.iter()
    }

    pub fn get(&self, id: VoiceId) -> Option<&Voice> {
        self.voices.iter().find(|v| v.id == id)
    }

    /// Voci vive del cue, dalla più vecchia.
    pub fn of_cue(&self, cue: CueId) -> impl Iterator<Item = &Voice> {
        self.voices.iter().filter(move |v| v.cue == cue)
    }

    /// Voci vive non `Critical`.
    pub fn non_critical(&self) -> usize {
        self.voices
            .iter()
            .filter(|v| v.priority != Priority::Critical)
            .count()
    }

    /// Somma dei guadagni nel mix, senza le voci `except`.
    pub fn mix_gain(&self, except: &[VoiceId]) -> f32 {
        self.voices
            .iter()
            .filter(|v| !except.contains(&v.id))
            .map(|v| v.mix_gain)
            .sum()
    }

    /// Somma dei guadagni nel mix delle sole voci non `Critical`.
    pub fn non_critical_mix_gain(&self) -> f32 {
        self.voices
            .iter()
            .filter(|v| v.priority != Priority::Critical)
            .map(|v| v.mix_gain)
            .sum()
    }

    pub(super) fn push(&mut self, voice: Voice) {
        self.voices.push(voice);
    }

    pub(super) fn remove(&mut self, id: VoiceId) -> Option<Voice> {
        let index = self.voices.iter().position(|v| v.id == id)?;
        Some(self.voices.remove(index))
    }

    pub(super) fn get_mut(&mut self, id: VoiceId) -> Option<&mut Voice> {
        self.voices.iter_mut().find(|v| v.id == id)
    }

    /// Toglie le voci che non hanno mai suonato e sono scadute a `now`; le restituisce.
    pub(super) fn expire(&mut self, now: f64) -> Vec<Voice> {
        let (expired, alive) = std::mem::take(&mut self.voices)
            .into_iter()
            .partition(|v| !v.playing && now >= v.expires);
        self.voices = alive;
        expired
    }

    /// Voci rubabili da una richiesta di priorità `priority`, nell'ordine del furto: solo di
    /// priorità strettamente più bassa, prima la più bassa e a parità la più vecchia.
    pub(super) fn steal_order(&self, priority: Priority, except: &[VoiceId]) -> Vec<Voice> {
        let mut candidates: Vec<Voice> = self
            .voices
            .iter()
            .filter(|v| v.priority < priority && !except.contains(&v.id))
            .copied()
            .collect();
        // gli id crescono con il tempo: a parità di priorità il più piccolo è il più vecchio
        candidates.sort_by_key(|v| (v.priority, v.id));
        candidates
    }
}

/// Sceglie la variante di un cue a caso, mai la stessa due volte di fila (con una sola, quella).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VariantPicker {
    last: Option<usize>,
}

impl VariantPicker {
    pub fn pick(&mut self, variants: usize, rng: &mut Pcg32) -> usize {
        let choice = match (variants, self.last) {
            (0 | 1, _) => 0,
            (n, Some(last)) if last < n => {
                // una delle altre n − 1, saltando l'ultima
                let other = below(rng, n - 1);
                if other >= last { other + 1 } else { other }
            }
            (n, _) => below(rng, n),
        };
        self.last = Some(choice);
        choice
    }
}

/// Intero uniforme in `[0, n)` (n piccolo: lo scarto del modulo è trascurabile).
fn below(rng: &mut Pcg32, n: usize) -> usize {
    (rng.next_u32() as usize) % n.max(1)
}
