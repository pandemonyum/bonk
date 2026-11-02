//! Il direttore della musica in un'app con `MinimalPlugins`, `StatesPlugin` e `AssetPlugin`, con
//! gli stati veri del gioco e senza l'`AudioPlugin` di Bevy: il sink lo fa [`FakeSink`], dato ai
//! deck in `PostUpdate` come farebbe Bevy quando il file è caricato (spento: caricamento lento).
//! Il caricamento vero delle tracce e l'app senza audio in `tests/music_load.rs`.

use std::time::Duration;

use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowFocused;

use super::playback::FakeSink;
use super::{cue_data, file_with};
use crate::app::{AppState, RunPhase};
use crate::audio::catalog::*;
use crate::audio::music::{MAX_DECKS, MusicStats, Slot, deck_volumes, notice_decks};
use crate::audio::playback::PlaybackStep;
use crate::audio::settings::step_gain;
use crate::audio::*;
use crate::data::GameConfig;
use crate::presentation::home::MenuScreen;

const TRIM: f32 = 0.8;
const GAIN: f32 = 0.8;
const FADE: f32 = 1.5;

/// Il dispositivo finto: con `on` ogni deck senza sink ne riceve uno.
#[derive(Resource)]
struct Device {
    on: bool,
}

fn fake_device(
    mut commands: Commands,
    device: Res<Device>,
    decks: Query<(Entity, &PlaybackSettings), (With<MusicDeck>, Without<FakeSink>)>,
) {
    if !device.on {
        return;
    }
    for (entity, settings) in &decks {
        commands.entity(entity).insert(FakeSink {
            volume: settings.volume.to_linear(),
            writes: 0,
        });
    }
}

/// Catalogo di prova con le tracce `tracks` (la partita del prato se c'è `run_meadow`).
fn catalog(tracks: &[&str], focus: f32) -> AudioCatalog {
    let mut file = file_with(vec![("shot.ember", cue_data())]);
    file.trims.music = TRIM;
    for name in tracks {
        file.music.tracks.insert(
            name.to_string(),
            TrackData {
                file: format!("audio/music/{name}.ogg"),
                gain: GAIN,
            },
        );
    }
    if tracks.contains(&"run_meadow") {
        file.music
            .run_by_stage
            .insert("meadow".into(), "run_meadow".into());
    }
    file.music.fades = MusicFades {
        to_run: FADE,
        to_home: FADE,
        focus,
    };
    AudioCatalog::compile(file).unwrap()
}

/// Gli stati del gioco e lo stage dei dati veri (il primo, il prato).
pub(super) fn add_states(app: &mut App) {
    app.add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_sub_state::<RunPhase>()
        .add_sub_state::<MenuScreen>()
        .insert_resource(GameConfig::embedded());
}

/// L'app a 60 FPS di tempo reale, con un asset pronto per ogni traccia e il dispositivo spento.
fn app(catalog: AudioCatalog) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<AudioSource>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(Device { on: false })
        .insert_resource(AudioLibrary::from_handles(Vec::new()));
    add_states(&mut app);
    let bytes: &[u8] = b"non decodificato: niente AudioPlugin";
    let handles: Vec<(TrackId, Handle<AudioSource>)> = (0..catalog.music.tracks().len())
        .map(|i| {
            let source = AudioSource {
                bytes: bytes.into(),
            };
            let handle = app
                .world_mut()
                .resource_mut::<Assets<AudioSource>>()
                .add(source);
            (TrackId(i as u16), handle)
        })
        .collect();
    app.insert_resource(catalog)
        .add_plugins(GameAudioPlugin { muted: false })
        .insert_resource(MusicLibrary::from_handles(handles))
        .add_systems(
            Update,
            (
                notice_decks::<FakeSink>.in_set(PlaybackStep::Notice),
                deck_volumes::<FakeSink>.in_set(PlaybackStep::Volumes),
            ),
        )
        .add_systems(PostUpdate, fake_device);
    app.update();
    app
}

pub(super) fn go(app: &mut App, state: AppState) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(state);
}

pub(super) fn frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

/// I deck vivi, in ordine di posto: (posto, traccia, livello, volume, suona).
pub(super) fn decks(app: &mut App) -> Vec<(Slot, String, f32, f32, bool)> {
    let world = app.world_mut();
    let names: Vec<String> = world
        .resource::<AudioCatalog>()
        .music
        .tracks()
        .iter()
        .map(|t| t.name.clone())
        .collect();
    let mut query = world.query::<&MusicDeck>();
    let mut live: Vec<_> = query
        .iter(world)
        .map(|d| {
            let name = names[d.track.index()].clone();
            (d.slot, name, d.level, d.volume, d.sounding)
        })
        .collect();
    live.sort_by_key(|deck| deck.0);
    live
}

fn players(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query::<&AudioPlayer>().iter(world).count()
}

fn stats(app: &App) -> MusicStats {
    app.world().resource::<MusicDirector>().stats
}

/// Il volume di un deck con i predefiniti (generale 8, musica 7) e il fuoco pieno.
fn full_volume(level: f32) -> f32 {
    step_gain(8) * step_gain(7) * TRIM * GAIN * level
}

#[test]
fn the_home_waits_for_its_sink_fades_in_and_plays_in_every_menu_screen() {
    let mut app = app(catalog(&["home"], 0.25));
    frames(&mut app, 5);
    assert!(decks(&mut app).is_empty(), "Loading: nessuna musica");
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 60);
    // un deck a zero in loop, fermo finché non nasce il sink (caricamento lento)
    let live = decks(&mut app);
    assert_eq!(live.len(), 1);
    assert_eq!(
        (live[0].0, live[0].1.as_str(), live[0].2),
        (Slot::A, "home", 0.0)
    );
    let world = app.world_mut();
    let settings = *world.query::<&PlaybackSettings>().single(world).unwrap();
    assert!(matches!(settings.mode, PlaybackMode::Loop));
    assert_eq!(settings.volume.to_linear(), 0.0);
    app.world_mut().resource_mut::<Device>().on = true;
    frames(&mut app, 2);
    assert!(decks(&mut app)[0].4, "il sink è nato");
    frames(&mut app, 100);
    let live = decks(&mut app);
    assert_eq!(live[0].2, 1.0);
    assert!((live[0].3 - full_volume(1.0)).abs() < 1e-6, "{}", live[0].3);
    let world = app.world_mut();
    let sink = world.query::<&FakeSink>().single(world).unwrap();
    assert!((sink.volume - full_volume(1.0)).abs() < 1e-6);
    // selezione, opzioni e ritorno alla pagina iniziale: nessun cambio
    for screen in [MenuScreen::Select, MenuScreen::Options, MenuScreen::Home] {
        app.world_mut()
            .resource_mut::<NextState<MenuScreen>>()
            .set(screen);
        frames(&mut app, 10);
        assert_eq!(decks(&mut app).len(), 1);
        assert_eq!(decks(&mut app)[0].2, 1.0);
    }
    let stats = stats(&app);
    assert_eq!((stats.changes, stats.spawned, stats.sounding), (1, 1, 1));
}

#[test]
fn the_run_fades_to_silence_and_the_menu_comes_back_from_the_same_deck() {
    let mut app = app(catalog(&["home"], 0.25));
    app.world_mut().resource_mut::<Device>().on = true;
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 120);
    // menu → partita: silenzio in 1,5 s, poi il deck si toglie
    go(&mut app, AppState::InRun);
    frames(&mut app, 45);
    let half = decks(&mut app)[0].2;
    assert!(half > 0.5 && half < 0.8, "{half}");
    frames(&mut app, 50);
    assert!(decks(&mut app).is_empty(), "{:?}", decks(&mut app));
    assert_eq!(players(&mut app), 0, "nessun deck residuo");
    // F5: Loading e di nuovo la partita, niente cambia
    go(&mut app, AppState::Loading);
    frames(&mut app, 3);
    go(&mut app, AppState::InRun);
    frames(&mut app, 3);
    assert!(decks(&mut app).is_empty());
    // di nuovo nel menu, poi in partita e subito indietro (passando da Loading): stesso deck
    go(&mut app, AppState::Loading);
    frames(&mut app, 1);
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 120);
    go(&mut app, AppState::InRun);
    frames(&mut app, 30);
    let before = decks(&mut app).remove(0);
    go(&mut app, AppState::Loading);
    frames(&mut app, 1);
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 2);
    let after = decks(&mut app);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].0, before.0, "un deck nuovo invece del rientro");
    // dal livello in cui era (un fotogramma giù in Loading, due su), non da zero
    assert!(
        (after[0].2 - before.2).abs() < 0.05 && after[0].2 > 0.5,
        "{before:?} → {after:?}"
    );
    frames(&mut app, 120);
    assert_eq!(decks(&mut app)[0].2, 1.0);
    let stats = stats(&app);
    assert_eq!((stats.spawned, stats.despawned, stats.reentries), (2, 1, 1));
}

#[test]
fn ten_trips_to_the_menu_keep_one_loop_per_track_and_at_most_two_decks() {
    let mut app = app(catalog(&["home", "run_meadow"], 0.25));
    app.world_mut().resource_mut::<Device>().on = true;
    // fotogrammi nel menu e in partita a ogni giro: rapidi, a metà dissolvenza, completi
    let stays = [1, 2, 3, 120, 5, 45, 1, 91, 30, 2];
    for (trip, &stay) in stays.iter().enumerate() {
        for state in [AppState::MainMenu, AppState::InRun] {
            if trip % 3 == 0 {
                go(&mut app, AppState::Loading);
                app.update();
            }
            go(&mut app, state);
            for _ in 0..stay {
                app.update();
                let live = decks(&mut app);
                assert!(live.len() <= MAX_DECKS, "giro {trip}: {live:?}");
                let mut names: Vec<&String> = live.iter().map(|d| &d.1).collect();
                names.dedup();
                assert_eq!(names.len(), live.len(), "loop doppio: {live:?}");
                assert!(players(&mut app) <= MAX_DECKS);
                assert_eq!(
                    app.world().resource::<MusicDirector>().decks.len(),
                    live.len()
                );
            }
        }
    }
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 300);
    let live = decks(&mut app);
    assert_eq!(live.len(), 1, "{live:?}");
    assert_eq!((live[0].1.as_str(), live[0].2), ("home", 1.0));
    assert_eq!(players(&mut app), 1);
    let stats = stats(&app);
    assert!(stats.reentries > 0, "{stats:?}");
    assert_eq!(stats.spawned - stats.despawned, 1, "{stats:?}");
}

#[test]
fn a_slider_change_reaches_the_live_decks_in_the_same_frame() {
    let mut app = app(catalog(&["home", "run_meadow"], 0.25));
    app.world_mut().resource_mut::<Device>().on = true;
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 120);
    go(&mut app, AppState::InRun);
    frames(&mut app, 40);
    let live = decks(&mut app);
    assert_eq!(live.len(), 2, "a metà dissolvenza: {live:?}");
    let sinks = |app: &mut App| {
        let world = app.world_mut();
        let mut query = world.query::<(&MusicDeck, &FakeSink)>();
        let mut found: Vec<_> = query
            .iter(world)
            .map(|(d, s)| (d.slot, d.level, s.volume))
            .collect();
        found.sort_by_key(|f| f.0);
        found
    };
    let mut settings = app.world().resource::<AudioSettings>().clone();
    settings.music = 3;
    settings.master = 10;
    app.insert_resource(settings);
    app.update();
    for (slot, level, volume) in sinks(&mut app) {
        let wanted = step_gain(10) * step_gain(3) * TRIM * GAIN * level;
        assert!(
            (volume - wanted).abs() < 1e-6,
            "{slot:?}: {volume} invece di {wanted}"
        );
    }
    // muto della sessione (--mute) e cursore a zero: silenzio vero
    app.world_mut()
        .resource_mut::<AudioSettings>()
        .session_muted = true;
    app.update();
    assert!(sinks(&mut app).iter().all(|s| s.2 == 0.0));
    app.world_mut()
        .resource_mut::<AudioSettings>()
        .session_muted = false;
    app.world_mut().resource_mut::<AudioSettings>().music = 0;
    app.update();
    assert!(sinks(&mut app).iter().all(|s| s.2 == 0.0));
}

#[test]
fn the_focus_fade_takes_its_time_from_the_catalog() {
    let mut app = app(catalog(&["home"], 0.5));
    app.world_mut().resource_mut::<Device>().on = true;
    go(&mut app, AppState::MainMenu);
    frames(&mut app, 120);
    app.world_mut().write_message(WindowFocused {
        window: Entity::PLACEHOLDER,
        focused: false,
    });
    app.update();
    let level = app.world().resource::<FocusFade>().level;
    assert!((level - (1.0 - 1.0 / 60.0 / 0.5)).abs() < 1e-5, "{level}");
    assert!((decks(&mut app)[0].3 - full_volume(1.0) * level).abs() < 1e-6);
    frames(&mut app, 31);
    assert_eq!(app.world().resource::<FocusFade>().level, 0.0);
    assert_eq!(decks(&mut app)[0].3, 0.0);
}
