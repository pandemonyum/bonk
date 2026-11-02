//! La sezione `music` del catalogo (progetto § 7): facoltativa, con i suoi errori leggibili, e
//! quella vera coerente con i dati del gioco.

use super::{cue_data, file_with};
use crate::audio::catalog::music::{DEFAULT_FADE_S, DEFAULT_FOCUS_FADE_S, HOME_TRACK};
use crate::audio::catalog::*;

/// Un catalogo di prova con due tracce e la partita del prato.
fn music_file() -> CatalogFile {
    let mut file = file_with(vec![("shot.ember", cue_data())]);
    file.music = MusicFile {
        tracks: [
            ("home", "audio/music/home.ogg", 0.8),
            ("run_meadow", "audio/music/run_meadow.ogg", 0.75),
        ]
        .into_iter()
        .map(|(name, file, gain)| {
            (
                name.to_string(),
                TrackData {
                    file: file.to_string(),
                    gain,
                },
            )
        })
        .collect(),
        run_by_stage: [("meadow".to_string(), "run_meadow".to_string())].into(),
        run_default: None,
        fades: MusicFades::default(),
    };
    file
}

fn errors_after(change: impl FnOnce(&mut MusicFile)) -> Vec<String> {
    let mut file = music_file();
    change(&mut file.music);
    AudioCatalog::compile(file).err().unwrap_or_default()
}

fn assert_error(change: impl FnOnce(&mut MusicFile), expected: &str) {
    let errors = errors_after(change);
    assert!(
        errors.iter().any(|e| e.contains(expected)),
        "atteso '{expected}' in {errors:?}"
    );
}

fn track<'a>(music: &'a mut MusicFile, name: &str) -> &'a mut TrackData {
    music.tracks.get_mut(name).expect("traccia di prova")
}

#[test]
fn a_catalog_without_music_is_valid_and_silent_with_the_default_fades() {
    let catalog = AudioCatalog::compile(file_with(vec![])).unwrap();
    assert!(catalog.music.tracks().is_empty());
    assert_eq!(catalog.music.home(), None);
    assert_eq!(catalog.music.run(Some("meadow")), None);
    let fades = catalog.music.fades;
    assert_eq!(
        (fades.to_run, fades.to_home, fades.focus),
        (DEFAULT_FADE_S, DEFAULT_FADE_S, DEFAULT_FOCUS_FADE_S)
    );
    assert_eq!(AudioCatalog::empty().music, MusicCatalog::default());
    // scritto come il file, senza la sezione: come i cataloghi di prima di AUDIO-3
    let text = r#"(version: 1,
        voices: (max: 8, max_web: 8, reserved_critical: 1, starts_per_frame: 2,
                 starts_per_frame_web: 2, sfx_headroom: 2.0),
        trims: (music: 1.0, sfx: 1.0, ambience: 1.0, ui: 1.0))"#;
    assert_eq!(
        AudioCatalog::parse(text).unwrap().music,
        MusicCatalog::default()
    );
}

#[test]
fn tracks_are_numbered_by_name_and_the_run_falls_back_to_the_default() {
    let mut file = music_file();
    file.music.run_default = Some("home".into());
    let catalog = AudioCatalog::compile(file).unwrap();
    let music = &catalog.music;
    let names: Vec<&str> = music.tracks().iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["home", "run_meadow"]);
    assert_eq!(music.id("home"), Some(TrackId(0)));
    assert_eq!(music.id("run_meadow"), Some(TrackId(1)));
    assert_eq!(music.id("boss"), None);
    assert_eq!(music.home(), Some(TrackId(0)));
    assert_eq!(HOME_TRACK, "home");
    let home = music.track(TrackId(0)).unwrap();
    assert_eq!(
        (home.file.as_str(), home.gain),
        ("audio/music/home.ogg", 0.8)
    );
    assert_eq!(music.run(Some("meadow")), music.id("run_meadow"));
    assert_eq!(music.run(Some("forest")), music.id("home"), "ripiego");
    assert_eq!(music.run(None), music.id("home"), "senza stage: ripiego");
    assert_eq!(music.name(TrackId(1)), "run_meadow");
    assert_eq!(music.name(TrackId(9)), "?");
    assert_eq!(music.run_stages().collect::<Vec<_>>(), ["meadow"]);
    // senza ripiego uno stage senza traccia è silenzio
    let catalog = AudioCatalog::compile(music_file()).unwrap();
    assert_eq!(catalog.music.run(Some("forest")), None);
}

#[test]
fn the_test_music_has_no_errors() {
    assert_eq!(errors_after(|_| {}), Vec::<String>::new());
}

#[test]
fn track_files_are_ogg_files_relative_to_assets_with_a_unit_gain() {
    assert_error(
        |m| track(m, "home").file = "audio/music/home.wav".into(),
        "catalog.ron: music.tracks.\"home\".file = \"audio/music/home.wav\": la musica vuole \
         file .ogg (Ogg Vorbis)",
    );
    for bad in ["/audio/home.ogg", "audio/Home.ogg", "audio/home.mp3", ""] {
        assert_error(
            |m| track(m, "home").file = bad.into(),
            &format!("music.tracks.\"home\".file = \"{bad}\": serve un percorso relativo"),
        );
    }
    for (gain, shown) in [(0.0, "0"), (1.5, "1.5"), (f32::NAN, "NaN")] {
        assert_error(
            |m| track(m, "home").gain = gain,
            &format!("music.tracks.\"home\".gain = {shown}: deve stare in (0, 1]"),
        );
    }
    assert_eq!(
        errors_after(|m| track(m, "home").gain = 1.0),
        Vec::<String>::new()
    );
    assert_error(
        |m| {
            let data = m.tracks.remove("home").unwrap();
            m.tracks.insert("Home".into(), data);
        },
        "music.tracks.\"Home\": nome non valido",
    );
}

#[test]
fn unknown_tracks_and_bad_stage_ids_are_reported() {
    assert_error(
        |m| {
            m.run_by_stage.insert("forest".into(), "run_forest".into());
        },
        "catalog.ron: music.run_by_stage.\"forest\" = \"run_forest\": la traccia non esiste in \
         music.tracks",
    );
    assert_error(
        |m| m.run_default = Some("run_any".into()),
        "music.run_default = \"run_any\": la traccia non esiste in music.tracks",
    );
    assert_error(
        |m| {
            m.run_by_stage.insert("Forest".into(), "run_meadow".into());
        },
        "music.run_by_stage.\"Forest\": id di stage non valido",
    );
}

#[test]
fn fades_are_finite_and_not_negative() {
    for (value, shown) in [(-0.5, "-0.5"), (f32::NAN, "NaN"), (f32::INFINITY, "inf")] {
        assert_error(
            |m| m.fades.to_run = value,
            &format!("music.fades.to_run = {shown}: deve essere >= 0 (secondi)"),
        );
        assert_error(
            |m| m.fades.to_home = value,
            &format!("music.fades.to_home = {shown}"),
        );
        assert_error(
            |m| m.fades.focus = value,
            &format!("music.fades.focus = {shown}"),
        );
    }
    // zero è un cambio secco, ammesso
    let zero = errors_after(|m| {
        m.fades = MusicFades {
            to_run: 0.0,
            to_home: 0.0,
            focus: 0.0,
        }
    });
    assert_eq!(zero, Vec::<String>::new());
}

#[test]
fn unknown_fields_of_the_music_are_errors_and_missing_fades_take_the_default() {
    let text = |music: &str| {
        format!(
            "(version: 1, voices: (max: 8, max_web: 8, reserved_critical: 1, \
             starts_per_frame: 2, starts_per_frame_web: 2, sfx_headroom: 2.0), \
             trims: (music: 1.0, sfx: 1.0, ambience: 1.0, ui: 1.0), music: {music})"
        )
    };
    let partial = AudioCatalog::parse(&text(
        "(tracks: { \"home\": (file: \"audio/music/home.ogg\", gain: 0.5) }, \
         run_default: Some(\"home\"), fades: (to_run: 0.6))",
    ))
    .unwrap();
    let fades = partial.music.fades;
    assert_eq!(
        (fades.to_run, fades.to_home, fades.focus),
        (0.6, DEFAULT_FADE_S, DEFAULT_FOCUS_FADE_S)
    );
    assert_eq!(partial.music.run(Some("meadow")), partial.music.home());
    for (typo, field) in [
        ("(trakcs: {})", "trakcs"),
        ("(fades: (to_hom: 1.0))", "to_hom"),
        (
            "(tracks: { \"home\": (file: \"audio/music/home.ogg\", gain: 0.5, loop: true) })",
            "loop",
        ),
    ] {
        let errors = AudioCatalog::parse(&text(typo)).unwrap_err();
        assert!(
            errors[0].starts_with("catalog.ron: ") && errors[0].contains(field),
            "{errors:?}"
        );
    }
}

/// La sezione vera: la home c'è, è un Ogg del catalogo, e ogni stage nominato esiste nei dati.
#[test]
fn the_real_music_names_real_stages_and_has_the_home_track() {
    use crate::data::GameConfig;
    let catalog = AudioCatalog::parse(CATALOG_TEXT).unwrap();
    let home = catalog
        .music
        .home()
        .expect("il catalogo vero ha la traccia home");
    let track = catalog.music.track(home).unwrap();
    assert!(track.file.ends_with(".ogg"), "{}", track.file);
    let config = GameConfig::embedded();
    for stage in catalog.music.run_stages() {
        assert!(
            config.stages.iter().any(|s| s.id == stage),
            "music.run_by_stage: lo stage \"{stage}\" non è in game-data/stages.ron"
        );
    }
}
