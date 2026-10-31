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
    /// Menu iniziale: pagina iniziale, selezione del compagno e avvio della run (solo client).
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

/// Avvia una run con il compagno `companion`, senza controlli: il menu passa da
/// [`try_start_run`]. F5 e Invio dai risultati ripartiranno poi direttamente, senza ripassare dal
/// menu.
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

/// Perché il menu non ha avviato la run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartRefused {
    /// L'id non c'è nei dati (una ricarica può averlo tolto).
    Unknown(String),
    /// Il compagno c'è ma non è ancora un compagno iniziale (D-066).
    Locked(String),
}

impl std::fmt::Display for StartRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(id) => write!(f, "il compagno \"{id}\" non esiste nei dati"),
            Self::Locked(id) => write!(f, "il compagno \"{id}\" è ancora da sbloccare"),
        }
    }
}

/// Avvia la run scelta nel menu solo con un compagno iniziale disponibile per il profilo
/// (`profile::unlocks`), anche se l'interfaccia lasciasse passare un compagno bloccato: in quel
/// caso nulla cambia e si resta nel menu.
pub fn try_start_run(
    config: &GameConfig,
    profile: &crate::profile::Profile,
    setup: &mut RunSetup,
    after: &mut AfterLoading,
    next: &mut NextState<AppState>,
    companion: &str,
) -> Result<(), StartRefused> {
    use crate::profile::{UnlockStatus, unlock_status};
    match unlock_status(config, profile, companion) {
        None => Err(StartRefused::Unknown(companion.to_string())),
        Some(UnlockStatus::Locked) => Err(StartRefused::Locked(companion.to_string())),
        Some(UnlockStatus::Available) => {
            start_run(setup, after, next, companion);
            Ok(())
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    /// Il controllo sta nella logica dell'avvio, non nel pulsante: un compagno bloccato o
    /// sconosciuto non tocca né la scelta né lo stato.
    #[test]
    fn only_an_available_companion_starts_a_run() {
        let config = GameConfig::embedded();
        let profile = Profile::default();
        let locked = config
            .companions
            .iter()
            .find(|c| !crate::profile::is_unlocked(&config, &profile, &c.id))
            .expect("un compagno da sbloccare")
            .id
            .clone();
        let mut setup = RunSetup::default();
        let mut after = AfterLoading(AppState::MainMenu);
        let mut next = NextState::<AppState>::default();
        for (id, refused) in [
            (locked.clone(), StartRefused::Locked(locked.clone())),
            (
                "nessuno".to_string(),
                StartRefused::Unknown("nessuno".into()),
            ),
        ] {
            let result = try_start_run(&config, &profile, &mut setup, &mut after, &mut next, &id);
            assert_eq!(result, Err(refused));
            assert_eq!(setup, RunSetup::default());
            assert_eq!(after, AfterLoading(AppState::MainMenu));
            assert!(matches!(next, NextState::Unchanged), "{next:?}");
        }
        let starter = config.stage.starting_companion.clone();
        let result = try_start_run(
            &config, &profile, &mut setup, &mut after, &mut next, &starter,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(setup.companion.as_deref(), Some(starter.as_str()));
        assert_eq!(after, AfterLoading(AppState::InRun));
        assert!(
            matches!(next, NextState::Pending(AppState::InRun)),
            "{next:?}"
        );
    }
}
