//! Stati dell'applicazione e plugin della simulazione (senza rendering).

use bevy::prelude::*;

use crate::{gameplay::GameplayPlugin, world::WorldPlugin};

/// Stato di primo livello. `MainMenu`, `GameOver` e `Victory` arrivano con la P0.6.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Una run in corso. Le entità della run hanno `DespawnOnExit(AppState::InRun)`:
    /// uscire e rientrare in questo stato equivale a un restart pulito.
    #[default]
    InRun,
}

/// Fase interna della run. La simulazione avanza solo in `Playing`.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InRun)]
pub enum RunPhase {
    #[default]
    Playing,
    Paused,
}

/// Seed della run: mondo e spawn derivano da qui, quindi stesso seed = stessa partita.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub u64);

/// Stati, simulazione di gioco e mondo. Non richiede finestra né GPU.
pub struct SimulationPlugin {
    pub seed: u64,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<RunPhase>()
            .insert_resource(RunSeed(self.seed))
            .add_plugins((GameplayPlugin, WorldPlugin))
            // In pausa si ferma il tempo virtuale: il passo fisso non avanza e
            // l'interpolazione del rendering resta ferma.
            .add_systems(OnEnter(RunPhase::Paused), pause_time)
            .add_systems(OnExit(RunPhase::Paused), resume_time);
    }
}

fn pause_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
