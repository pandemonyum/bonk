//! Il volume di una voce viva (progetto § 3.4), senza tipi di `bevy_audio`: la catena dei
//! guadagni e la soglia per riscrivere un sink. La riproduzione (`playback.rs`) lo applica.

use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;

use crate::audio::catalog::{AudioCatalog, CueBus};
use crate::audio::focus::FocusFade;
use crate::audio::settings::AudioSettings;

/// Il volume di un sink si riscrive solo se cambia più di così (lineare).
pub const VOLUME_EPSILON: f32 = 0.001;

/// Volume lineare di una voce: generale × bus (cursori) × trim del bus × guadagno della voce ×
/// dissolvenza del fuoco. Un bus a zero o il muto danno 0.
pub fn voice_volume(
    settings: &AudioSettings,
    bus: CueBus,
    trim: f32,
    gain: f32,
    focus: f32,
) -> f32 {
    settings.bus_gain(bus.bus()) * trim * gain * focus
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
    pub(super) catalog: Res<'w, AudioCatalog>,
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
}
