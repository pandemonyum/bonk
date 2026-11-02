//! Il caricamento delle tracce con l'`AssetServer` vero e il loader di Bevy (senza uscita audio),
//! e l'app senza `Assets<AudioSource>` (`--no-audio`): nessun `load`, nessun panic.

use std::time::Duration;

use bevy::audio::{AudioLoader, AudioSource};
use bevy::prelude::*;

use super::file_with;
use super::music_app::{add_states, decks, frames, go};
use crate::app::AppState;
use crate::audio::catalog::*;
use crate::audio::*;

#[test]
fn without_audio_assets_the_music_loads_nothing_and_does_not_panic() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    add_states(&mut app);
    app.add_plugins(GameAudioPlugin { muted: false });
    app.update();
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 10);
    assert!(decks(&mut app).is_empty());
    assert!(app.world().resource::<MusicLibrary>().is_empty());
    assert!(!app.world().contains_resource::<Assets<AudioSource>>());
}

/// Aggiorna finché `done` è vero (al massimo ~5 s: il caricamento è asincrono).
fn update_until(app: &mut App, done: impl Fn(&mut App) -> bool) {
    for _ in 0..1000 {
        app.update();
        if done(app) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("condizione non raggiunta");
}

/// Un'app con il loader vero di Bevy (senza uscita audio) e il catalogo `catalog`.
fn loading_app(catalog: AudioCatalog) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<AudioSource>()
        .init_asset_loader::<AudioLoader>()
        .insert_resource(AudioLibrary::from_handles(Vec::new()))
        .insert_resource(catalog);
    add_states(&mut app);
    app.add_plugins(GameAudioPlugin { muted: false });
    app.update();
    go(&mut app, AppState::MainMenu);
    app
}

#[test]
fn the_real_home_track_loads_once_and_a_missing_file_is_silence() {
    let mut app = loading_app(AudioCatalog::embedded());
    update_until(&mut app, |app| {
        let world = app.world();
        let home = world.resource::<AudioCatalog>().music.home().unwrap();
        world
            .resource::<MusicLibrary>()
            .handle(home)
            .is_some_and(|handle| {
                world
                    .resource::<Assets<AudioSource>>()
                    .contains(handle.id())
            })
    });
    assert_eq!(app.world().resource::<MusicLibrary>().len(), 1);
    assert_eq!(decks(&mut app).len(), 1);
    // un file che non c'è: il deck si toglie e resta il silenzio, senza ritentare
    let mut file = file_with(vec![]);
    file.music.tracks.insert(
        "home".into(),
        TrackData {
            file: "audio/music/missing_for_the_test.ogg".into(),
            gain: 0.5,
        },
    );
    let mut app = loading_app(AudioCatalog::compile(file).unwrap());
    update_until(&mut app, |app| {
        let director = app.world().resource::<MusicDirector>();
        director.stats.despawned == 1 && director.decks.is_empty()
    });
    let changes = app.world().resource::<MusicDirector>().stats.changes;
    frames(&mut app, 10);
    assert!(decks(&mut app).is_empty());
    let director = app.world().resource::<MusicDirector>();
    assert_eq!(director.stats.spawned, 1);
    assert_eq!(director.target.unwrap().track, None);
    assert_eq!(
        director.stats.changes, changes,
        "un cambio a ogni fotogramma"
    );
}
