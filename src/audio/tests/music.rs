//! La traccia voluta in ogni combinazione di stato (progetto § 3.7), senza Bevy. Curve e deck in
//! `tests/music_decks.rs`.

use super::{cue_data, file_with};
use crate::app::{AppState, RunPhase};
use crate::audio::catalog::*;
use crate::audio::music::*;
use crate::presentation::home::MenuScreen;

const PHASES: [RunPhase; 5] = [
    RunPhase::Playing,
    RunPhase::Paused,
    RunPhase::LevelUp,
    RunPhase::Defeated,
    RunPhase::Victory,
];
const SCREENS: [Option<MenuScreen>; 4] = [
    None,
    Some(MenuScreen::Home),
    Some(MenuScreen::Select),
    Some(MenuScreen::Options),
];
const STAGES: [Option<&str>; 4] = [None, Some("meadow"), Some("forest"), Some("nowhere")];
/// Durate diverse, per vedere quale dissolvenza si sceglie.
const TO_RUN: f32 = 1.5;
const TO_HOME: f32 = 2.0;

/// Musica di prova: le tracce `tracks`, la partita del prato e, se c'è, il ripiego.
fn music_with(tracks: &[&str], run_default: Option<&str>) -> MusicCatalog {
    let mut file = file_with(vec![("shot.ember", cue_data())]);
    for name in tracks {
        file.music.tracks.insert(
            name.to_string(),
            TrackData {
                file: format!("audio/music/{name}.ogg"),
                gain: 0.5,
            },
        );
    }
    if tracks.contains(&"run_meadow") {
        file.music
            .run_by_stage
            .insert("meadow".into(), "run_meadow".into());
    }
    file.music.run_default = run_default.map(String::from);
    file.music.fades = MusicFades {
        to_run: TO_RUN,
        to_home: TO_HOME,
        focus: 0.25,
    };
    AudioCatalog::compile(file).unwrap().music
}

/// Ogni combinazione di stato, schermata del menu, fase e stage (anche quelle che il gioco non
/// produce, come una fase fuori dalla partita).
fn every_context() -> Vec<MusicContext<'static>> {
    let mut all = Vec::new();
    for app in [AppState::Loading, AppState::MainMenu, AppState::InRun] {
        for menu in SCREENS {
            for phase in PHASES.map(Some).into_iter().chain([None]) {
                for stage in STAGES {
                    all.push(MusicContext {
                        app,
                        menu,
                        phase,
                        stage,
                    });
                }
            }
        }
    }
    all
}

/// La traccia attesa, scritta come la tabella del § 3.7.
fn expected(context: &MusicContext, music: &MusicCatalog) -> Option<Wanted> {
    let run_track = match context.stage {
        Some("meadow") => music.id("run_meadow").or(music.run(None)),
        _ => music.run(None),
    };
    match (context.app, context.phase) {
        (AppState::Loading, _) => None,
        (AppState::MainMenu, _) => Some(Wanted {
            track: music.id("home"),
            fade_s: TO_HOME,
        }),
        (AppState::InRun, Some(RunPhase::Victory | RunPhase::Defeated)) => Some(Wanted {
            track: None,
            fade_s: TO_RUN,
        }),
        (AppState::InRun, _) => Some(Wanted {
            track: run_track,
            fade_s: TO_RUN,
        }),
    }
}

#[test]
fn the_wanted_track_follows_the_table_in_every_state() {
    let catalogs = [
        music_with(&["home", "run_meadow"], None),
        music_with(&["home", "run_meadow", "run_any"], Some("run_any")),
        music_with(&["home"], None),
        music_with(&["run_meadow"], None),
        music_with(&[], None),
    ];
    let contexts = every_context();
    assert_eq!(contexts.len(), 3 * 4 * 6 * 4);
    for music in &catalogs {
        for context in &contexts {
            assert_eq!(
                wanted(context, music),
                expected(context, music),
                "{context:?} con {:?}",
                music.tracks()
            );
        }
    }
}

#[test]
fn the_menu_plays_home_the_run_its_stage_and_the_results_are_silent() {
    let music = music_with(&["home", "run_meadow", "run_any"], Some("run_any"));
    let at = |app, menu, phase, stage| {
        wanted(
            &MusicContext {
                app,
                menu,
                phase,
                stage,
            },
            &music,
        )
        .map(|wanted| {
            (
                wanted.track.map(|t| music.name(t).to_string()),
                wanted.fade_s,
            )
        })
    };
    let home = Some((Some("home".to_string()), TO_HOME));
    for screen in SCREENS {
        assert_eq!(
            at(AppState::MainMenu, screen, None, None),
            home,
            "{screen:?}"
        );
    }
    let meadow = Some((Some("run_meadow".to_string()), TO_RUN));
    let any = Some((Some("run_any".to_string()), TO_RUN));
    for phase in [RunPhase::Playing, RunPhase::Paused, RunPhase::LevelUp] {
        // pausa e draft non cambiano traccia
        assert_eq!(
            at(AppState::InRun, None, Some(phase), Some("meadow")),
            meadow
        );
        assert_eq!(at(AppState::InRun, None, Some(phase), Some("forest")), any);
    }
    for phase in [RunPhase::Victory, RunPhase::Defeated] {
        let silence = Some((None, TO_RUN));
        assert_eq!(
            at(AppState::InRun, None, Some(phase), Some("meadow")),
            silence
        );
    }
    // il restart (F5) e la ricarica passano da Loading: nessun cambio
    assert_eq!(at(AppState::Loading, None, None, Some("meadow")), None);
}

#[test]
fn the_real_catalog_plays_home_in_the_menu_and_silence_in_the_run() {
    let music = AudioCatalog::parse(CATALOG_TEXT).unwrap().music;
    let home = music.home().expect("traccia home");
    let menu = MusicContext {
        app: AppState::MainMenu,
        menu: Some(MenuScreen::Home),
        phase: None,
        stage: None,
    };
    let wanted_home = wanted(&menu, &music).unwrap();
    assert_eq!(wanted_home.track, Some(home));
    assert_eq!(wanted_home.fade_s, music.fades.to_home);
    for stage in ["meadow", "forest"] {
        for phase in PHASES {
            let run = MusicContext {
                app: AppState::InRun,
                menu: None,
                phase: Some(phase),
                stage: Some(stage),
            };
            let silence = wanted(&run, &music).unwrap();
            assert_eq!(silence.track, None, "{stage} {phase:?}");
            assert_eq!(silence.fade_s, music.fades.to_run);
        }
    }
}
