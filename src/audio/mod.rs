//! Audio del client (D-069, D-070; progetto in `docs/design/AUDIO_SYSTEM.md`).
//!
//! Solo in `client_app`: la simulazione non conosce questo modulo. L'audio osserva i fatti di
//! gioco, non scrive nulla nel mondo della simulazione e ha un caso proprio (`mixer`). Tre livelli:
//! ascolto (fatti di gioco → [`CueRequest`] nella [`CueQueue`], AUDIO-5), politica ([`Mixer`],
//! pura e testata senza scheda audio) e riproduzione (AUDIO-2, l'unico punto che userà
//! `bevy_audio`). In AUDIO-1 c'è la politica: le decisioni finiscono in [`VoiceCommands`], che
//! ancora nessuno esegue.
//!
//! Tempo dell'audio: `Time<Real>`, così pausa, draft e `--time-scale` non cambiano i cooldown.

pub mod catalog;
pub mod cues;
pub mod mixer;
pub mod settings;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use catalog::{AudioCatalog, CueBus, CueId, Priority};
pub use cues::{AudioManager, CueQueue, CueRequest};
pub use mixer::{Decision, Mixer, MixerStats, VoiceId};
pub use settings::{AudioSettings, Bus};

use crate::gameplay::kinematics::Position;
use crate::gameplay::player::Player;
use crate::world::view::view_rect;

/// Fasi dell'audio in `Update`, in quest'ordine: ascolto dei fatti di gioco, decisioni del
/// mixer, musica, ambienti, riproduzione.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioSet {
    Listen,
    Decide,
    Music,
    Ambience,
    Apply,
}

/// Il mixer come risorsa (il modulo `mixer` resta senza tipi di Bevy).
#[derive(Resource, Debug)]
pub struct AudioMixer(pub Mixer);

/// Decisioni dell'ultimo fotogramma per la riproduzione; si svuota a ogni `AudioSet::Decide`.
#[derive(Resource, Debug, Default)]
pub struct VoiceCommands(pub Vec<Decision>);

/// Diagnostica dell'audio (overlay F3 in AUDIO-5).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct AudioStats {
    pub mixer: MixerStats,
    /// Voci vive per il mixer.
    pub live_voices: usize,
    /// Somma dei guadagni delle voci vive nel mix (trim compreso).
    pub mix_gain: f32,
    /// Richieste con un nome che il catalogo non conosce.
    pub unknown_cues: u64,
}

/// Catalogo, coda, mixer e statistiche: niente finestra, niente scheda audio, va anche con
/// `MinimalPlugins`. Un `AudioCatalog` inserito prima del plugin prende il posto di quello
/// incorporato (test).
pub struct AudioCorePlugin;

impl Plugin for AudioCorePlugin {
    fn build(&self, app: &mut App) {
        let catalog = app
            .world()
            .get_resource::<AudioCatalog>()
            .cloned()
            .unwrap_or_else(AudioCatalog::embedded);
        let mixer = Mixer::new(&catalog);
        app.configure_sets(
            Update,
            (
                AudioSet::Listen,
                AudioSet::Decide,
                AudioSet::Music,
                AudioSet::Ambience,
                AudioSet::Apply,
            )
                .chain(),
        )
        .insert_resource(catalog)
        .insert_resource(AudioMixer(mixer))
        .init_resource::<CueQueue>()
        .init_resource::<VoiceCommands>()
        .init_resource::<AudioStats>()
        .add_systems(Update, decide.in_set(AudioSet::Decide));
    }
}

/// L'audio del gioco: il nucleo più le impostazioni del giocatore. In AUDIO-2 si aggiungono la
/// riproduzione e le impostazioni dal profilo.
pub struct GameAudioPlugin {
    /// Muto della sessione (`--mute`): `AudioSettings::session_muted`, mai salvato nel profilo.
    pub muted: bool,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioCorePlugin)
            .insert_resource(AudioSettings {
                session_muted: self.muted,
                ..default()
            });
    }
}

/// Svuota la coda e chiede al mixer cosa avviare e fermare. La vista per il cull è quella più
/// grande attorno al player (in menu non c'è: nessun cull).
fn decide(
    time: Res<Time<Real>>,
    mut queue: ResMut<CueQueue>,
    mut mixer: ResMut<AudioMixer>,
    mut commands: ResMut<VoiceCommands>,
    mut stats: ResMut<AudioStats>,
    player: Query<&Position, With<Player>>,
) {
    commands.0.clear();
    let requests = queue.take();
    let view = player.single().ok().map(|at| view_rect(at.0));
    let decisions = mixer.0.decide(time.elapsed_secs_f64(), &requests, view);
    commands.0.extend(decisions);
    *stats = AudioStats {
        mixer: mixer.0.stats(),
        live_voices: mixer.0.voices().len(),
        mix_gain: mixer.0.voices().mix_gain(&[]),
        unknown_cues: queue.unknown_requests(),
    };
}
