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

/// D-068: lo stage scelto (`--stage` → `RunSetup::stage`) diventa lo stage giocato al primo
/// caricamento e resta dopo una ricarica a caldo.
#[test]
fn the_chosen_stage_is_played_and_survives_a_reload() {
    let dir = DataDir::new("stage");
    let mut app = app(&dir);
    let embedded = GameConfig::embedded();
    let chosen = embedded.stages.last().unwrap().id.clone();
    assert_ne!(
        chosen, embedded.stage.id,
        "serve uno stage diverso dal primo"
    );
    app.insert_resource(RunSetup {
        stage: Some(chosen.clone()),
        ..default()
    });
    update_until(&mut app, has_config);
    assert_eq!(app.world().resource::<GameConfig>().stage.id, chosen);
    let current = (embedded.clone().with_stage(&chosen).unwrap())
        .stage
        .spawns
        .initial_burst;
    let wanted = current + 3;
    let more = EMBEDDED.stages.replace(
        &format!("initial_burst: {current},"),
        &format!("initial_burst: {wanted},"),
    );
    dir.write("stages.ron", more);
    reload(&mut app, "stages.ron");
    update_until(&mut app, |w| {
        w.resource::<GameConfig>().stage.spawns.initial_burst == wanted
    });
    assert_eq!(app.world().resource::<GameConfig>().stage.id, chosen);
}

#[test]
fn a_chosen_stage_that_does_not_exist_is_a_data_error() {
    let dir = DataDir::new("unknown-stage");
    let mut app = app(&dir);
    app.insert_resource(RunSetup {
        stage: Some("volcano".into()),
        ..default()
    });
    update_until(&mut app, error_contains("\"volcano\" non esiste"));
    assert!(!has_config(app.world()));
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

/// Rilievo del code review P0.6: salvando i dati dal menu, il menu si ricostruisce
/// (passa da `Loading`) invece di restare con le carte vecchie.
#[test]
fn a_valid_change_in_the_menu_reloads_too() {
    let dir = DataDir::new("reload-menu");
    let mut app = app(&dir);
    update_until(&mut app, has_config);
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::MainMenu);
    app.update();
    let current = GameConfig::embedded().stage.spawns.initial_burst;
    let wanted = current + 2;
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
    app.update();
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Loading
    );
}
