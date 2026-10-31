//! La riproduzione in un'app con `MinimalPlugins` e `AssetPlugin`, con `Assets<AudioSource>` ma
//! senza l'`AudioPlugin` di Bevy: niente dispositivo e niente sink veri. Il sink lo fa
//! [`FakeSink`], inserito dalle prove come lo inserirebbe Bevy alla nascita.

use std::time::Duration;

use bevy::audio::{
    AudioLoader, AudioPlayer, AudioSinkPlayback, AudioSource, PlaybackMode, PlaybackSettings,
    SeekError, Volume,
};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowFocused;

use super::{cue_data, file_with};
use crate::audio::catalog::{AudioCatalog, CueBus, CueData, OnLimit};
use crate::audio::playback::{PlaybackStep, apply_volumes, needs_volume, notice_sinks};
use crate::audio::settings::step_gain;
use crate::audio::*;

/// Un sink finto: tiene il volume e conta le scritture.
#[derive(Component, Debug)]
pub(super) struct FakeSink {
    pub(super) volume: f32,
    pub(super) writes: u32,
}

impl AudioSinkPlayback for FakeSink {
    fn volume(&self) -> Volume {
        Volume::Linear(self.volume)
    }
    fn set_volume(&mut self, volume: Volume) {
        self.volume = volume.to_linear();
        self.writes += 1;
    }
    fn speed(&self) -> f32 {
        1.0
    }
    fn set_speed(&self, _: f32) {}
    fn play(&self) {}
    fn position(&self) -> Duration {
        Duration::ZERO
    }
    fn try_seek(&self, _: Duration) -> Result<(), SeekError> {
        Ok(())
    }
    fn pause(&self) {}
    fn is_paused(&self) -> bool {
        false
    }
    fn stop(&self) {}
    fn empty(&self) -> bool {
        false
    }
    fn is_muted(&self) -> bool {
        false
    }
    fn mute(&mut self) {}
    fn unmute(&mut self) {}
}

const TRIM_UI: f32 = 0.6;
const GAIN: f32 = 0.5;

/// Un cue dell'interfaccia senza caso, una voce sola che riparte (`Restart`).
fn tick_catalog() -> AudioCatalog {
    let mut file = file_with(vec![(
        "ui.tick",
        CueData {
            bus: CueBus::Ui,
            files: vec!["audio/sfx/ui/tick.wav".into()],
            gain: GAIN,
            max_instances: 1,
            on_limit: OnLimit::Restart,
            ..cue_data()
        },
    )]);
    file.trims.ui = TRIM_UI;
    AudioCatalog::compile(file).unwrap()
}

/// L'app delle prove a 60 FPS di tempo reale; con `ready` il file è già negli asset.
pub(super) fn app(ready: bool) -> (App, Handle<AudioSource>) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<AudioSource>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(tick_catalog())
        .add_plugins(GameAudioPlugin { muted: false })
        .add_systems(
            Update,
            (
                notice_sinks::<FakeSink>.in_set(PlaybackStep::Notice),
                apply_volumes::<FakeSink>.in_set(PlaybackStep::Volumes),
            ),
        );
    let handle = if ready {
        let bytes: &[u8] = b"non decodificato: niente AudioPlugin";
        app.world_mut()
            .resource_mut::<Assets<AudioSource>>()
            .add(AudioSource {
                bytes: bytes.into(),
            })
    } else {
        Handle::default()
    };
    app.insert_resource(AudioLibrary::from_handles(vec![vec![handle.clone()]]));
    (app, handle)
}

pub(super) fn ask(app: &mut App) {
    let cue = app
        .world()
        .resource::<AudioCatalog>()
        .exact("ui.tick")
        .unwrap();
    app.world_mut().resource_mut::<CueQueue>().push(CueRequest {
        cue,
        count: 1,
        at: None,
    });
}

pub(super) fn voices(app: &mut App) -> Vec<(Entity, PlaybackVoice)> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &PlaybackVoice)>();
    query.iter(world).map(|(e, v)| (e, *v)).collect()
}

pub(super) fn mixer_voices(app: &App) -> usize {
    app.world().resource::<AudioMixer>().0.voices().len()
}

/// Il volume atteso con i predefiniti (generale 8, effetti 8) e il fuoco pieno.
fn expected(settings: &AudioSettings) -> f32 {
    settings.bus_gain(Bus::Sfx) * TRIM_UI * GAIN
}

#[test]
fn a_start_spawns_a_voice_at_the_right_volume_and_a_stop_removes_it() {
    let (mut app, handle) = app(true);
    ask(&mut app);
    app.update();
    let live = voices(&mut app);
    assert_eq!(live.len(), 1);
    let (entity, voice) = live[0];
    let volume = step_gain(8) * step_gain(8) * TRIM_UI * GAIN;
    assert!((voice.volume - volume).abs() < 1e-6, "{}", voice.volume);
    assert_eq!(
        (voice.bus, voice.gain, voice.sounding),
        (CueBus::Ui, GAIN, false)
    );
    let world = app.world();
    assert_eq!(world.get::<AudioPlayer>(entity).unwrap().0, handle);
    let settings = world.get::<PlaybackSettings>(entity).unwrap();
    assert!(matches!(settings.mode, PlaybackMode::Despawn));
    assert!(matches!(settings.volume, Volume::Linear(v) if (v - volume).abs() < 1e-6));
    assert_eq!(settings.speed, 1.0);
    assert_eq!(mixer_voices(&app), 1);
    assert_eq!(app.world().resource::<PlaybackStats>().spawned, 1);

    // `Restart`: il mixer ferma la voce vecchia e ne avvia una nuova
    ask(&mut app);
    app.update();
    let live = voices(&mut app);
    assert_eq!(live.len(), 1);
    assert_ne!(live[0].1.voice, voice.voice);
    assert!(
        app.world().get_entity(entity).is_err(),
        "la voce fermata resta"
    );
    assert_eq!(mixer_voices(&app), 1);
    let stats = app.world().resource::<PlaybackStats>();
    assert_eq!(
        (stats.spawned, stats.stopped, stats.silent_expired),
        (2, 1, 0)
    );
    let mixer = app.world().resource::<AudioMixer>().0.stats();
    assert_eq!(mixer.finished, 0, "la voce fermata era già fuori dal mixer");

    // Bevy toglie l'entità a fine suono: il mixer libera la voce
    app.world_mut().entity_mut(live[0].0).despawn();
    assert_eq!(mixer_voices(&app), 0);
    assert_eq!(app.world().resource::<AudioMixer>().0.stats().finished, 1);
}

#[test]
fn a_file_that_is_not_ready_is_dropped_and_frees_the_voice_at_once() {
    let (mut app, _) = app(false);
    ask(&mut app);
    app.update();
    assert!(voices(&mut app).is_empty());
    assert_eq!(mixer_voices(&app), 0);
    assert_eq!(app.world().resource::<PlaybackStats>().not_ready, 1);
}

#[test]
fn the_sink_marks_the_voice_as_playing_and_follows_the_settings() {
    let (mut app, _) = app(true);
    ask(&mut app);
    app.update();
    let (entity, voice) = voices(&mut app)[0];
    // come fa Bevy: il sink nasce con il volume di `PlaybackSettings`
    app.world_mut().entity_mut(entity).insert(FakeSink {
        volume: voice.volume,
        writes: 0,
    });
    app.update();
    let voice = voices(&mut app)[0].1;
    assert!(voice.sounding);
    assert!(
        app.world()
            .resource::<AudioMixer>()
            .0
            .voices()
            .get(voice.voice)
            .unwrap()
            .playing
    );
    let stats = app.world().resource::<PlaybackStats>();
    assert_eq!(stats.sounding, 1);
    assert_eq!(stats.heard.len(), 1);
    let sink = |app: &App| {
        let sink = app.world().get::<FakeSink>(entity).unwrap();
        (sink.volume, sink.writes)
    };
    assert_eq!(sink(&app).1, 0, "volume già giusto: nessuna scrittura");

    // un cursore cambia il volume calcolato e quello del sink
    app.world_mut().resource_mut::<AudioSettings>().master = 4;
    app.update();
    let wanted = expected(app.world().resource::<AudioSettings>());
    assert!((voices(&mut app)[0].1.volume - wanted).abs() < 1e-6);
    assert_eq!(sink(&app), (wanted, 1));
    // sotto la soglia il sink non si riscrive
    app.world_mut().get_mut::<FakeSink>(entity).unwrap().volume = wanted + 0.0005;
    app.update();
    assert_eq!(sink(&app).1, 1);
    // un bus a zero è silenzio vero
    app.world_mut().resource_mut::<AudioSettings>().sfx = 0;
    app.update();
    assert_eq!(voices(&mut app)[0].1.volume, 0.0);
    assert_eq!(sink(&app), (0.0, 2));
    // il muto della sessione anche, senza toccare le tacche
    app.world_mut().resource_mut::<AudioSettings>().sfx = 8;
    app.update();
    assert!(sink(&app).0 > 0.0);
    app.world_mut()
        .resource_mut::<AudioSettings>()
        .session_muted = true;
    app.update();
    assert_eq!(sink(&app).0, 0.0);
    assert_eq!(app.world().resource::<AudioSettings>().sfx, 8);
}

fn focus(app: &mut App, focused: bool) {
    app.world_mut().write_message(WindowFocused {
        window: Entity::PLACEHOLDER,
        focused,
    });
}

fn level(app: &App) -> f32 {
    app.world().resource::<FocusFade>().level
}

#[test]
fn without_focus_the_voices_fade_out_and_come_back_without_touching_the_settings() {
    let (mut app, _) = app(true);
    ask(&mut app);
    app.update();
    let full = voices(&mut app)[0].1.volume;
    assert!(full > 0.0);
    let before = app.world().resource::<AudioSettings>().clone();
    focus(&mut app, false);
    app.update();
    let first = level(&app);
    assert!(first < 1.0 && first > 0.8, "{first}");
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(level(&app), 0.0);
    assert_eq!(voices(&mut app)[0].1.volume, 0.0);
    assert_eq!(*app.world().resource::<AudioSettings>(), before);
    focus(&mut app, true);
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(level(&app), 1.0);
    assert!((voices(&mut app)[0].1.volume - full).abs() < 1e-6);

    // con "Audio fuori dalla finestra" il volume resta pieno
    app.world_mut()
        .resource_mut::<AudioSettings>()
        .play_unfocused = true;
    focus(&mut app, false);
    for _ in 0..20 {
        app.update();
    }
    assert!(!app.world().resource::<FocusFade>().focused);
    assert_eq!(level(&app), 1.0);
}

#[test]
fn the_sink_volume_is_rewritten_only_when_it_changes() {
    assert!(!needs_volume(0.5, 0.5));
    assert!(!needs_volume(0.5, 0.5009));
    assert!(needs_volume(0.5, 0.502));
    assert!(needs_volume(0.0005, 0.0), "lo zero è silenzio vero");
    assert!(!needs_volume(0.0, 0.0));
}

#[test]
fn without_audio_assets_nothing_loads_and_nothing_panics() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .add_plugins(GameAudioPlugin { muted: false });
    let wav = app
        .world()
        .resource::<AudioCatalog>()
        .exact("ui.tick")
        .expect("il catalogo vero ha ui.tick");
    for _ in 0..3 {
        app.world_mut().resource_mut::<CueQueue>().push(CueRequest {
            cue: wav,
            count: 1,
            at: None,
        });
        app.update();
    }
    let world = app.world_mut();
    assert!(!world.contains_resource::<Assets<AudioSource>>());
    assert!(!world.contains_resource::<AudioLibrary>());
    assert_eq!(world.query::<&AudioPlayer>().iter(world).count(), 0);
    assert_eq!(world.query::<&PlaybackVoice>().iter(world).count(), 0);
    // il mixer decide lo stesso: le voci scadranno da sole
    assert!(app.world().resource::<AudioStats>().mixer.started >= 1);
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

/// Il percorso vero: la libreria carica i file del catalogo dall'`AssetServer` (con il loader
/// di Bevy, senza l'uscita audio) e un `Start` parte appena il file è caricato.
#[test]
fn the_library_loads_the_catalog_files_and_plays_them_once_loaded() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<AudioSource>()
        .init_asset_loader::<AudioLoader>()
        .add_plugins(GameAudioPlugin { muted: false });
    app.update();
    let catalog = app.world().resource::<AudioCatalog>().clone();
    let files: usize = catalog.cues().iter().map(|c| c.data.files.len()).sum();
    assert_eq!(app.world().resource::<AudioLibrary>().files(), files);
    let tick = catalog
        .exact("ui.tick")
        .expect("il catalogo vero ha ui.tick");
    let handle = app
        .world()
        .resource::<AudioLibrary>()
        .handle(tick, 0)
        .cloned()
        .unwrap();
    update_until(&mut app, |app| {
        app.world()
            .resource::<Assets<AudioSource>>()
            .contains(handle.id())
    });
    app.world_mut().resource_mut::<CueQueue>().push(CueRequest {
        cue: tick,
        count: 1,
        at: None,
    });
    app.update();
    let live = voices(&mut app);
    assert_eq!(live.len(), 1);
    assert_eq!(app.world().get::<AudioPlayer>(live[0].0).unwrap().0, handle);
}
