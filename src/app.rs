//! Stati dell'applicazione e plugin della simulazione (senza rendering).

use bevy::prelude::*;

use crate::data::schema::CompanionData;
use crate::data::{DataStatus, GameConfig};
use crate::{gameplay::GameplayPlugin, world::WorldPlugin};

/// Stato di primo livello.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// In attesa di una `GameConfig` valida. Anche il restart passa da qui.
    #[default]
    Loading,
    /// Menu iniziale: scelta del compagno e avvio della run (solo client).
    MainMenu,
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
    /// Draft delle carte dopo un livello nuovo: tempo fermo finché non si sceglie.
    LevelUp,
    /// Il player è a 0 HP: schermata dei risultati, poi si ricomincia o si torna al menu.
    Defeated,
    /// Boss sconfitto: schermata dei risultati come per la sconfitta.
    Victory,
}

/// Seed della run: mondo e spawn derivano da qui, quindi stesso seed = stessa partita.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub u64);

/// Scelte fatte nel menu per le prossime run. Senza scelta (headless, autoplay) vale il
/// compagno iniziale dello stage.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct RunSetup {
    pub companion: Option<String>,
}

impl RunSetup {
    /// Il compagno iniziale della prossima run: quello scelto se esiste ancora nei dati (una
    /// ricarica può averlo tolto), altrimenti quello dello stage.
    pub fn starter<'a>(&self, config: &'a GameConfig) -> &'a CompanionData {
        self.companion
            .as_deref()
            .and_then(|id| config.companion(id))
            .unwrap_or_else(|| config.starting_companion())
    }
}

/// Dove va l'app quando la configurazione è pronta: il menu all'avvio del client e con
/// "torna al menu", direttamente una run in headless, in autoplay e con il restart.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AfterLoading(pub AppState);

impl Default for AfterLoading {
    fn default() -> Self {
        Self(AppState::InRun)
    }
}

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
            .init_resource::<RunSetup>()
            .init_resource::<AfterLoading>()
            .insert_resource(RunSeed(self.seed))
            .add_plugins((GameplayPlugin, WorldPlugin, crate::profile::ProfilePlugin))
            .add_systems(
                Update,
                start_run_when_ready.run_if(in_state(AppState::Loading)),
            )
            // Fuori da `Playing` si ferma il tempo virtuale: il passo fisso non avanza e
            // l'interpolazione del rendering resta ferma (senza, gli sprite tremerebbero di un
            // pixel tra le ultime due posizioni dietro la schermata).
            .add_systems(OnEnter(RunPhase::Paused), pause_time)
            .add_systems(OnExit(RunPhase::Paused), resume_time)
            .add_systems(OnEnter(RunPhase::LevelUp), pause_time)
            .add_systems(OnExit(RunPhase::LevelUp), resume_time)
            .add_systems(OnEnter(RunPhase::Defeated), pause_time)
            .add_systems(OnExit(RunPhase::Defeated), resume_time)
            .add_systems(OnEnter(RunPhase::Victory), pause_time)
            .add_systems(OnExit(RunPhase::Victory), resume_time);
    }
}

/// Fa ripartire la run da capo con la configurazione corrente (F5, Invio dai risultati,
/// ricarica dei dati).
pub fn restart_run(next: &mut NextState<AppState>) {
    next.set(AppState::Loading);
}

/// Avvia una run con il compagno `companion` (scelta del menu). F5 e Invio dai risultati
/// ripartiranno poi direttamente, senza ripassare dal menu.
pub fn start_run(
    setup: &mut RunSetup,
    after: &mut AfterLoading,
    next: &mut NextState<AppState>,
    companion: &str,
) {
    setup.companion = Some(companion.to_string());
    after.0 = AppState::InRun;
    next.set(AppState::InRun);
}

/// Chiude la run e torna al menu.
pub fn back_to_menu(next: &mut NextState<AppState>, after: &mut AfterLoading) {
    after.0 = AppState::MainMenu;
    next.set(AppState::Loading);
}

fn start_run_when_ready(
    config: Option<Res<GameConfig>>,
    after: Res<AfterLoading>,
    mut next: ResMut<NextState<AppState>>,
) {
    if config.is_some() {
        next.set(after.0);
    }
}

fn pause_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
