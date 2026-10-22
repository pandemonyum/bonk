//! Caricamento dei dati nel client: i file di `game-data/` arrivano dall'AssetServer come
//! testo, poi `GameConfig::parse` li valida tutti insieme. Con la ricarica a caldo attiva
//! (`--watch`, default nelle build di sviluppo) ogni salvataggio ricarica i dati e, se validi,
//! fa ripartire la run.

use std::io;

use bevy::asset::io::{AssetSourceBuilder, Reader};
use bevy::asset::{AssetLoadFailedEvent, AssetLoader, LoadContext};
use bevy::prelude::*;

use super::{DATA_FILES, DataSources, DataStatus, GameConfig};
use crate::app::AppState;

/// Sorgente degli asset per i dati: `data://player.ron` → `game-data/player.ron`
/// (sul disco accanto ad `assets/`, sul web nella cartella `game-data/` del sito).
pub const DATA_SOURCE: &str = "data";
const DATA_DIR: &str = "game-data";

/// Testo grezzo di un file di dati.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct DataText(pub String);

#[derive(Default, TypePath)]
pub struct DataTextLoader;

impl AssetLoader for DataTextLoader {
    type Asset = DataText;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<DataText, io::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        String::from_utf8(bytes)
            .map(DataText)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Da chiamare prima di `DefaultPlugins`: l'AssetServer legge le sorgenti alla creazione.
pub fn register_data_source(app: &mut App) {
    register_data_source_at(app, DATA_DIR);
}

/// Come [`register_data_source`] ma da una cartella qualsiasi (test).
pub fn register_data_source_at(app: &mut App, dir: &str) {
    app.register_asset_source(DATA_SOURCE, AssetSourceBuilder::platform_default(dir, None));
}

#[derive(Resource)]
struct DataHandles([Handle<DataText>; DATA_FILES.len()]);

pub struct DataLoaderPlugin;

impl Plugin for DataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DataText>()
            .init_asset_loader::<DataTextLoader>()
            .add_systems(Startup, start_loading)
            .add_systems(Update, update_config);
    }
}

fn start_loading(mut commands: Commands, assets: Res<AssetServer>) {
    let handles = DATA_FILES.map(|file| assets.load(format!("{DATA_SOURCE}://{file}")));
    commands.insert_resource(DataHandles(handles));
}

/// Testi dei file se sono tutti caricati, nell'ordine di `DATA_FILES`.
fn loaded_texts<'a>(
    handles: &DataHandles,
    texts: &'a Assets<DataText>,
) -> Option<[&'a str; DATA_FILES.len()]> {
    let mut out = [""; DATA_FILES.len()];
    for (slot, handle) in out.iter_mut().zip(&handles.0) {
        *slot = texts.get(handle)?.0.as_str();
    }
    Some(out)
}

/// Primo caricamento e ricariche. Si (ri)valida quando un file arriva o cambia:
/// - dati validi al primo caricamento → `GameConfig` e la run può partire;
/// - dati validi e diversi durante il gioco → nuova configurazione, run riavviata;
/// - dati non validi o file illeggibile (es. non UTF-8) → errori a schermo, anche durante la
///   run; l'eventuale configurazione precedente resta in uso e salvando il file corretto si
///   riparte da qui (anche se l'errore era al primo avvio).
///
/// Bevy segnala con `Added` (non `Modified`) la prima versione riuscita di un file il cui
/// caricamento era fallito: per questo contano entrambi gli eventi.
fn update_config(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<DataText>>,
    mut failures: MessageReader<AssetLoadFailedEvent<DataText>>,
    handles: Res<DataHandles>,
    texts: Res<Assets<DataText>>,
    current: Option<Res<GameConfig>>,
    mut status: ResMut<DataStatus>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let ours = |id: AssetId<DataText>| handles.0.iter().any(|h| h.id() == id);
    let unreadable: Vec<String> = failures
        .read()
        .filter(|failure| ours(failure.id))
        .map(|failure| {
            format!(
                "{}: impossibile leggere il file ({})",
                failure.path, failure.error
            )
        })
        .collect();
    if !unreadable.is_empty() {
        set_error(&mut status, unreadable);
        return;
    }
    let changed = events.read().any(|event| match event {
        AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::LoadedWithDependencies { id } => ours(*id),
        _ => false,
    });
    let first_load = current.is_none() && *status == DataStatus::Loading;
    if !changed && !first_load {
        return;
    }
    let Some(texts) = loaded_texts(&handles, &texts) else {
        return; // ancora in caricamento
    };
    match GameConfig::parse(&DataSources::from_files(texts)) {
        Ok(config) => {
            *status = DataStatus::Ready;
            match current {
                None => {
                    info!("dati di gioco caricati da {DATA_DIR}/");
                    commands.insert_resource(config);
                }
                Some(current) if *current != config => {
                    info!("dati di gioco ricaricati: la run riparte con i nuovi valori");
                    commands.insert_resource(config);
                    if *state.get() == AppState::InRun {
                        next.set(AppState::Loading);
                    }
                }
                Some(_) => {}
            }
        }
        Err(errors) => set_error(&mut status, errors),
    }
}

fn set_error(status: &mut DataStatus, errors: Vec<String>) {
    let new = DataStatus::Error(errors);
    if *status != new {
        if let DataStatus::Error(list) = &new {
            for error in list {
                error!("dati di gioco: {error}");
            }
        }
        *status = new;
    }
}

/// Caricamento vero dall'AssetServer, senza GPU, da una cartella temporanea con copie dei file.
/// Le ricariche sono forzate con `AssetServer::reload` invece che dal watcher (deterministico).
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::data::EMBEDDED;

    /// Cartella temporanea con tutti i file di dati, rimossa a fine test.
    struct DataDir(PathBuf);

    impl DataDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("pms-data-{}-{name}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            for (file, text) in DATA_FILES.iter().zip(EMBEDDED.files()) {
                std::fs::write(dir.join(file), text).unwrap();
            }
            Self(dir)
        }

        fn write(&self, file: &str, bytes: impl AsRef<[u8]>) {
            std::fs::write(self.0.join(file), bytes).unwrap();
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for DataDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn app(dir: &DataDir) -> App {
        let mut app = App::new();
        register_data_source_at(&mut app, dir.path().to_str().unwrap());
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                watch_for_changes_override: Some(false),
                ..default()
            },
            StatesPlugin,
        ))
        .init_state::<AppState>()
        .init_resource::<DataStatus>()
        .add_plugins(DataLoaderPlugin);
        app
    }

    /// Aggiorna l'app finché `done` è vero (al massimo ~5 s: il caricamento è asincrono).
    fn update_until(app: &mut App, done: impl Fn(&World) -> bool) {
        for _ in 0..1000 {
            app.update();
            if done(app.world()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!(
            "condizione non raggiunta; stato dati: {:?}",
            app.world().resource::<DataStatus>()
        );
    }

    fn reload(app: &mut App, file: &str) {
        app.world()
            .resource::<AssetServer>()
            .reload(format!("{DATA_SOURCE}://{file}"));
    }

    fn has_config(world: &World) -> bool {
        world.contains_resource::<GameConfig>()
    }

    fn error_contains(text: &'static str) -> impl Fn(&World) -> bool {
        move |world| {
            matches!(world.resource::<DataStatus>(), DataStatus::Error(errors)
                if errors.iter().any(|e| e.contains(text)))
        }
    }

    #[test]
    fn valid_files_produce_the_same_config_as_the_embedded_copy() {
        let dir = DataDir::new("valid");
        let mut app = app(&dir);
        update_until(&mut app, has_config);
        assert_eq!(*app.world().resource::<DataStatus>(), DataStatus::Ready);
        assert_eq!(
            *app.world().resource::<GameConfig>(),
            GameConfig::embedded()
        );
    }

    #[test]
    fn invalid_values_show_an_error_and_no_config() {
        let dir = DataDir::new("invalid");
        dir.write(
            "enemies.ron",
            EMBEDDED.enemies.replace("speed: 40.0", "speed: -3.0"),
        );
        let mut app = app(&dir);
        update_until(&mut app, error_contains("blob.speed"));
        assert!(!has_config(app.world()));
    }

    #[test]
    fn fixing_an_unreadable_file_after_a_failed_first_load_starts_the_game() {
        let dir = DataDir::new("unreadable");
        // "più" salvato in Latin-1: non è UTF-8 valido, il caricamento fallisce
        dir.write("stages.ron", b"// pi\xf9\n(version: 1)");
        let mut app = app(&dir);
        update_until(&mut app, error_contains("impossibile leggere"));
        dir.write("stages.ron", EMBEDDED.stages);
        reload(&mut app, "stages.ron");
        update_until(&mut app, has_config); // arriva come AssetEvent::Added
        assert_eq!(*app.world().resource::<DataStatus>(), DataStatus::Ready);
    }

    #[test]
    fn a_valid_change_during_the_run_reloads_and_restarts() {
        let dir = DataDir::new("reload");
        let mut app = app(&dir);
        update_until(&mut app, has_config);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InRun);
        app.update();
        // Valore nuovo ricavato da quello attuale: il test non dipende dai numeri in stages.ron.
        let current = GameConfig::embedded().stage.spawns.initial_burst;
        let wanted = current + 3;
        let more = EMBEDDED.stages.replace(
            &format!("initial_burst: {current},"),
            &format!("initial_burst: {wanted},"),
        );
        assert_ne!(more, EMBEDDED.stages, "la modifica deve cambiare il file");
        dir.write("stages.ron", more);
        reload(&mut app, "stages.ron");
        update_until(&mut app, |w| {
            w.resource::<GameConfig>().stage.spawns.initial_burst == wanted
        });
        app.update(); // applica la transizione chiesta da update_config
        assert_eq!(
            *app.world().resource::<State<AppState>>().get(),
            AppState::Loading
        );
    }

    #[test]
    fn a_broken_save_during_the_run_keeps_the_old_config_and_shows_the_error() {
        let dir = DataDir::new("broken-save");
        let mut app = app(&dir);
        update_until(&mut app, has_config);
        let before = app.world().resource::<GameConfig>().clone();
        dir.write("stages.ron", b"// pi\xf9\n(version: 1)");
        reload(&mut app, "stages.ron");
        update_until(&mut app, error_contains("impossibile leggere"));
        assert_eq!(*app.world().resource::<GameConfig>(), before);
    }
}
