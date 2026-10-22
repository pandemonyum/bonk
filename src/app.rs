//! Stati dell'applicazione e plugin della simulazione (senza rendering).

use bevy::prelude::*;

use crate::data::{DataStatus, GameConfig};
use crate::{gameplay::GameplayPlugin, world::WorldPlugin};

/// Stato di primo livello. `MainMenu`, `GameOver` e `Victory` arrivano con la P0.6.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// In attesa di una `GameConfig` valida. Anche il restart passa da qui.
    #[default]
    Loading,
    /// Una run in corso. Le entità della run hanno `DespawnOnExit(AppState::InRun)`:
    /// uscire e rientrare in questo stato equivale a un restart pulito.
    InRun,
}

/// Fase interna della run. La simulazione avanza solo in `Playing`.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InRun)]
pub enum RunPhase {
    #[default]
    Playing,
    Paused,
    /// Il player è a 0 HP: la run è ferma finché non si ricomincia (schermata risultati
    /// completa e vittoria con la P0.6).
    Defeated,
}

/// Seed della run: mondo e spawn derivano da qui, quindi stesso seed = stessa partita.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub u64);

/// Stati, simulazione di gioco e mondo. Non richiede finestra né GPU. La `GameConfig` la
/// fornisce chi costruisce l'app: il loader dei dati nel client, i file incorporati in headless.
pub struct SimulationPlugin {
    pub seed: u64,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<RunPhase>()
            .init_resource::<DataStatus>()
            .insert_resource(RunSeed(self.seed))
            .add_plugins((GameplayPlugin, WorldPlugin))
            .add_systems(
                Update,
                start_run_when_ready.run_if(in_state(AppState::Loading)),
            )
            // In pausa si ferma il tempo virtuale: il passo fisso non avanza e
            // l'interpolazione del rendering resta ferma.
            .add_systems(OnEnter(RunPhase::Paused), pause_time)
            .add_systems(OnExit(RunPhase::Paused), resume_time);
    }
}

/// Fa ripartire la run da capo con la configurazione corrente (tasto F5, ricarica dei dati).
pub fn restart_run(next: &mut NextState<AppState>) {
    next.set(AppState::Loading);
}

fn start_run_when_ready(config: Option<Res<GameConfig>>, mut next: ResMut<NextState<AppState>>) {
    if config.is_some() {
        next.set(AppState::InRun);
    }
}

fn pause_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
