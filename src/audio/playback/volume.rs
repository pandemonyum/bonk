//! Il volume di una voce viva o di un deck della musica (progetto § 3.4), senza tipi di
//! `bevy_audio`: la catena dei guadagni e la soglia per riscrivere un sink. La riproduzione
//! (`playback.rs`, `voices.rs`) e il direttore della musica (`music`) lo applicano.

use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;

use crate::audio::catalog::{AudioCatalog, CueBus};
use crate::audio::focus::FocusFade;
use crate::audio::settings::{AudioSettings, Bus};

/// Il volume di un sink si riscrive solo se cambia più di così (lineare).
pub const VOLUME_EPSILON: f32 = 0.001;

/// Volume lineare sul bus `bus`: generale × cursore del bus × trim del bus × guadagno ×
/// dissolvenza del fuoco. Un bus a zero o il muto (anche `--mute`) danno 0.
pub fn bus_volume(settings: &AudioSettings, bus: Bus, trim: f32, gain: f32, focus: f32) -> f32 {
    settings.bus_gain(bus) * trim * gain * focus
}

/// Volume lineare di una voce: generale × bus (cursori) × trim del bus × guadagno della voce ×
/// dissolvenza del fuoco. Un bus a zero o il muto danno 0.
pub fn voice_volume(
    settings: &AudioSettings,
    bus: CueBus,
    trim: f32,
    gain: f32,
    focus: f32,
) -> f32 {
    bus_volume(settings, bus.bus(), trim, gain, focus)
}

/// Il sink va riscritto: il volume cambia più di [`VOLUME_EPSILON`], oppure si vuole il silenzio
/// e non c'è ancora (un bus a 0 è silenzio vero, non quasi).
pub fn needs_volume(current: f32, wanted: f32) -> bool {
    (current - wanted).abs() > VOLUME_EPSILON || (wanted == 0.0 && current != 0.0)
}

/// Impostazioni, catalogo e fuoco: quanto serve per il volume di una voce.
#[derive(SystemParam)]
pub(crate) struct Mix<'w> {
    settings: Res<'w, AudioSettings>,
    pub(crate) catalog: Res<'w, AudioCatalog>,
    fade: Res<'w, FocusFade>,
}

impl Mix<'_> {
    pub(super) fn volume(&self, bus: CueBus, gain: f32) -> f32 {
        voice_volume(
            &self.settings,
            bus,
            self.catalog.trim(bus),
            gain,
            self.fade.level,
        )
    }

    /// Livello della dissolvenza del fuoco della finestra, 0–1.
    pub(crate) fn focus(&self) -> f32 {
        self.fade.level
    }

    /// Volume di un deck della musica: generale × cursore Musica × `trims.music` × `gain` (il
    /// guadagno della traccia per il livello del deck) × fuoco della finestra.
    pub(crate) fn music(&self, gain: f32) -> f32 {
        bus_volume(
            &self.settings,
            Bus::Music,
            self.catalog.trims.music,
            gain,
            self.fade.level,
        )
    }
}
