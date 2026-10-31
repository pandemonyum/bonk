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

/// Un profilo salvato prima dell'audio si legge con i predefiniti (come
/// `profiles_saved_before_the_option_still_load` delle Opzioni).
#[test]
fn profiles_saved_before_the_audio_still_load() {
    use crate::profile::ProfileData;
    let old: ProfileData =
        ron::from_str("(version: 1, companions: [], fullscreen: true, still_home: true)").unwrap();
    assert_eq!(old.audio, AudioSettings::default());
    assert!(old.fullscreen && old.still_home);
    // un profilo con l'audio a metà: i campi mancanti hanno il loro default
    let partial: ProfileData = ron::from_str("(version: 1, audio: (music: 2))").unwrap();
    assert_eq!(partial.audio.music, 2);
    assert_eq!(partial.audio.master, AudioSettings::default().master);
    // e il testo salvato si rilegge uguale
    let text = ron::to_string(&partial).unwrap();
    assert_eq!(ron::from_str::<ProfileData>(&text).unwrap(), partial);
}

/// All'avvio le impostazioni vengono dal profilo, e `--mute` resta (mai nel profilo).
#[test]
fn at_startup_the_settings_come_from_the_profile_and_keep_the_session_mute() {
    use crate::profile::Profile;
    let mut profile = Profile::default();
    profile.data.audio.music = 3;
    profile.data.audio.play_unfocused = true;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameAudioPlugin { muted: true })
        .insert_resource(profile.clone());
    app.update();
    let settings = app.world().resource::<AudioSettings>().clone();
    assert_eq!(settings.music, 3);
    assert!(settings.play_unfocused && settings.session_muted && settings.is_muted());
    assert_eq!(
        *app.world().resource::<Profile>(),
        profile,
        "il profilo non cambia"
    );
    assert!(!app.world().resource::<Profile>().data.audio.session_muted);
}

/// Le Opzioni salvano il profilo a ogni tacca: il file si scrive accanto e poi si rinomina,
/// quindi un salvataggio interrotto non lo lascia mai troncato. Solo in una cartella temporanea
/// dei test, mai nel profilo vero.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_save_replaces_the_profile_whole_and_a_broken_one_leaves_it_intact() {
    use crate::profile::Profile;
    let dir = std::env::temp_dir().join(format!("pms-profile-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("profile.ron");
    let temporary = path.with_extension("ron.tmp");
    let mut profile = Profile {
        data: Default::default(),
        path: Some(path.clone()),
    };
    profile.data.audio.music = 2;
    profile.save();
    assert_eq!(Profile::open(path.clone()).data, profile.data);
    // un altro salvataggio sostituisce il file che c'è (su Windows anche)
    profile.data.audio.music = 9;
    profile.save();
    assert_eq!(Profile::open(path.clone()).data.audio.music, 9);
    assert!(!temporary.exists(), "il file temporaneo non resta");
    // un salvataggio che si rompe a metà (il file accanto non si può scrivere): resta quello di
    // prima, intero
    std::fs::create_dir_all(&temporary).unwrap();
    profile.data.audio.music = 4;
    profile.save();
    let kept = Profile::open(path.clone());
    assert_eq!(kept.data.audio.music, 9, "profilo troncato o sovrascritto");
    assert!(
        !path.with_extension("ron.bak").exists(),
        "il profilo era leggibile"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
