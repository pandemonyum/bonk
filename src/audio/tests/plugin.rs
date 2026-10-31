//! `AudioManager`, `AudioCorePlugin` e `GameAudioPlugin` in un'app con `MinimalPlugins`: niente
//! finestra, niente scheda audio.

use bevy::prelude::*;

use super::{cue_data, file_with};
use crate::audio::catalog::{AudioCatalog, CueBus, CueData};
use crate::audio::*;

#[derive(Resource, Default)]
struct Asked {
    ran: bool,
    queued: Vec<bool>,
}

/// Un'app con `MinimalPlugins` e un sistema che chiede i suoni `names` al primo fotogramma.
fn app_asking(names: &'static [&'static str]) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).init_resource::<Asked>();
    let ask = move |mut audio: AudioManager, mut asked: ResMut<Asked>| {
        if asked.ran {
            return;
        }
        asked.ran = true;
        asked.queued = names.iter().map(|name| audio.play(name)).collect();
    };
    app.add_systems(Update, ask.in_set(AudioSet::Listen));
    app
}

fn test_catalog() -> AudioCatalog {
    let mut file = file_with(vec![
        (
            "ui.tick",
            CueData {
                bus: CueBus::Ui,
                files: vec!["audio/sfx/ui/tick.wav".into()],
                ..cue_data()
            },
        ),
        ("shot.ember", cue_data()),
    ]);
    file.fallbacks.insert("shot".into(), "shot.ember".into());
    AudioCatalog::compile(file).unwrap()
}

fn starts(app: &App) -> Vec<CueId> {
    app.world()
        .resource::<VoiceCommands>()
        .0
        .iter()
        .filter_map(|d| match d {
            Decision::Start { cue, .. } => Some(*cue),
            Decision::Stop { .. } => None,
        })
        .collect()
}

#[test]
fn the_audio_manager_without_audio_does_nothing_and_does_not_panic() {
    let mut app = app_asking(&["ui.tick", "nope"]);
    app.update();
    app.update();
    let asked = app.world().resource::<Asked>();
    assert!(asked.ran);
    assert_eq!(asked.queued, [false, false]);
    assert!(app.world().get_resource::<CueQueue>().is_none());
}

#[test]
fn the_core_plugin_turns_requests_into_decisions_once_per_frame() {
    let mut app = app_asking(&["ui.tick", "shot.tide", "nope", "nope"]);
    app.insert_resource(test_catalog())
        .add_plugins(AudioCorePlugin);
    app.update();
    assert_eq!(
        app.world().resource::<Asked>().queued,
        [true, true, false, false]
    );
    let catalog = app.world().resource::<AudioCatalog>();
    let expected = vec![
        catalog.exact("shot.ember").unwrap(),
        catalog.exact("ui.tick").unwrap(),
    ];
    assert_eq!(starts(&app), expected, "ripiego shot.tide → shot.ember");
    let queue = app.world().resource::<CueQueue>();
    assert!(queue.is_empty(), "la coda si svuota a ogni fotogramma");
    assert_eq!(queue.unknown().collect::<Vec<_>>(), ["nope"]);
    let stats = *app.world().resource::<AudioStats>();
    assert_eq!(stats.mixer.started, 2);
    assert_eq!(stats.live_voices, 2);
    assert_eq!(stats.unknown_cues, 2);
    // fotogramma dopo: nessuna richiesta, le decisioni vecchie non restano
    app.update();
    assert!(app.world().resource::<VoiceCommands>().0.is_empty());
    assert_eq!(app.world().resource::<AudioStats>().live_voices, 2);
}

#[test]
fn a_request_of_zero_events_queues_nothing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(test_catalog())
        .add_plugins(AudioCorePlugin)
        .init_resource::<Asked>();
    let ask = |mut audio: AudioManager, mut asked: ResMut<Asked>| {
        let id = audio.id("ui.tick").expect("cue di prova");
        asked.queued = vec![
            audio.request("ui.tick", 0, None),
            audio.request_id(id, 0, None),
        ];
    };
    // prima del mixer, che svuota la coda
    let look = |queue: Res<CueQueue>, mut asked: ResMut<Asked>| asked.ran = queue.is_empty();
    app.add_systems(Update, (ask, look).chain().in_set(AudioSet::Listen));
    app.update();
    let asked = app.world().resource::<Asked>();
    assert_eq!(asked.queued, [false, false]);
    assert!(asked.ran, "coda non vuota");
    assert!(starts(&app).is_empty());
    assert_eq!(app.world().resource::<AudioStats>().mixer.requests, 0);
}

#[test]
fn the_game_plugin_uses_the_embedded_catalog_and_the_mute_option() {
    let mut app = app_asking(&["ui.tick"]);
    app.add_plugins(GameAudioPlugin { muted: true });
    app.update();
    assert_eq!(
        *app.world().resource::<AudioCatalog>(),
        AudioCatalog::embedded()
    );
    let settings = app.world().resource::<AudioSettings>();
    // `--mute` vale per la sessione: la scelta del giocatore (nel profilo) resta quella
    assert!(settings.session_muted && settings.is_muted() && !settings.muted);
    assert_eq!(
        AudioSettings {
            session_muted: false,
            ..settings.clone()
        },
        AudioSettings::default()
    );
    let mut loud = App::new();
    loud.add_plugins(MinimalPlugins)
        .add_plugins(GameAudioPlugin { muted: false });
    assert!(!loud.world().resource::<AudioSettings>().is_muted());
}
