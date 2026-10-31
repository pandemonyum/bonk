//! Schema e validazione del catalogo (progetto § 7): ogni errore con un test, come
//! `src/data/validate/tests.rs`.

use super::{cue_data, file_with};
use crate::audio::catalog::*;

fn errors_after(change: impl FnOnce(&mut CatalogFile)) -> Vec<String> {
    let mut file = file_with(vec![("shot.ember", cue_data())]);
    change(&mut file);
    AudioCatalog::compile(file).err().unwrap_or_default()
}

fn assert_error(change: impl FnOnce(&mut CatalogFile), expected: &str) {
    let errors = errors_after(change);
    assert!(
        errors.iter().any(|e| e.contains(expected)),
        "atteso '{expected}' in {errors:?}"
    );
}

fn shot(file: &mut CatalogFile) -> &mut CueData {
    file.cues.get_mut("shot.ember").expect("cue di prova")
}

#[test]
fn the_real_audio_catalog_is_valid() {
    let catalog = match AudioCatalog::parse(CATALOG_TEXT) {
        Ok(catalog) => catalog,
        Err(errors) => panic!("assets/audio/catalog.ron:\n{}", errors.join("\n")),
    };
    assert_eq!(AudioCatalog::embedded(), catalog);
}

#[test]
fn the_test_catalog_has_no_errors() {
    assert_eq!(errors_after(|_| {}), Vec::<String>::new());
}

#[test]
fn version_and_voice_limits_are_checked() {
    assert_error(
        |f| f.version = 2,
        "catalog.ron: version 2 non supportata (attesa 1)",
    );
    assert_error(
        |f| f.voices.reserved_critical = 24,
        "catalog.ron: voices.reserved_critical = 24: deve essere minore di voices.max (24)",
    );
    assert_error(
        |f| f.voices.reserved_critical = 16,
        "voices.reserved_critical = 16: deve essere minore di voices.max_web (16)",
    );
    assert_error(
        |f| f.voices.starts_per_frame = 0,
        "voices.starts_per_frame = 0: deve essere >= 1",
    );
    assert_error(
        |f| f.voices.starts_per_frame_web = 0,
        "voices.starts_per_frame_web = 0: deve essere >= 1",
    );
    assert_error(
        |f| f.voices.sfx_headroom = 0.0,
        "voices.sfx_headroom = 0: deve essere > 0",
    );
    assert_error(
        |f| f.voices.sfx_headroom = f32::NAN,
        "voices.sfx_headroom = NaN",
    );
    assert_error(|f| f.trims.ui = 1.5, "trims.ui = 1.5: deve stare in (0, 1]");
    assert_error(
        |f| f.trims.music = 0.0,
        "trims.music = 0: deve stare in (0, 1]",
    );
}

#[test]
fn cue_numbers_out_of_range_are_reported_with_the_field() {
    assert_error(
        |f| shot(f).gain = 1.5,
        "catalog.ron: cues.\"shot.ember\".gain = 1.5: deve stare in (0, 1]",
    );
    assert_error(
        |f| shot(f).gain = 0.0,
        "cues.\"shot.ember\".gain = 0: deve stare in (0, 1]",
    );
    for (jitter, shown) in [(-1.0, "-1"), (6.5, "6.5"), (f32::NAN, "NaN")] {
        assert_error(
            |f| shot(f).gain_jitter_db = jitter,
            &format!("cues.\"shot.ember\".gain_jitter_db = {shown}: deve stare in [0, 6]"),
        );
    }
    assert_eq!(
        errors_after(|f| shot(f).gain_jitter_db = 6.0),
        Vec::<String>::new()
    );
    for pitch in [(0.4, 1.0), (1.0, 2.5), (1.1, 1.0), (f32::NAN, 1.0)] {
        assert_error(|f| shot(f).pitch = pitch, "cues.\"shot.ember\".pitch = (");
    }
    assert_error(
        |f| shot(f).pitch = (1.1, 1.0),
        "pitch = (1.1, 1): deve stare in [0.5, 2] con min <= max",
    );
    assert_error(
        |f| shot(f).max_instances = 0,
        "cues.\"shot.ember\".max_instances = 0: deve essere >= 1",
    );
    for max_db in [0.0, -1.0, 20.0] {
        assert_error(
            |f| shot(f).coalesce = Coalesce::Boost { max_db },
            &format!("cues.\"shot.ember\".coalesce.max_db = {max_db}: deve stare in (0, 12]"),
        );
    }
    assert_error(
        |f| shot(f).length_s = 0.0,
        "cues.\"shot.ember\".length_s = 0: deve essere > 0",
    );
}

#[test]
fn cue_files_are_wav_files_relative_to_assets() {
    assert_error(
        |f| shot(f).files.clear(),
        "cues.\"shot.ember\".files: serve almeno un file",
    );
    assert_error(
        |f| shot(f).files = vec!["audio/sfx/Shot.wav".into()],
        "cues.\"shot.ember\".files[0] = \"audio/sfx/Shot.wav\": serve un percorso relativo",
    );
    assert_error(
        |f| shot(f).files = vec!["audio/a.wav".into(), "/audio/b.wav".into()],
        "files[1] = \"/audio/b.wav\"",
    );
    for bus in [CueBus::Sfx, CueBus::Ui] {
        assert_error(
            |f| {
                let cue = shot(f);
                cue.bus = bus;
                cue.files = vec!["audio/sfx/a.ogg".into()];
            },
            "cues.\"shot.ember\".files[0] = \"audio/sfx/a.ogg\": i cue vogliono file .wav",
        );
    }
    let ui = errors_after(|f| {
        let cue = shot(f);
        cue.bus = CueBus::Ui;
        cue.files = vec!["audio/sfx/ui/tick.wav".into()];
    });
    assert_eq!(ui, Vec::<String>::new());
}

#[test]
fn cues_play_only_on_the_effects_and_interface_buses() {
    // musica, stinger e ambienti avranno le loro sezioni (AUDIO-3, AUDIO-4)
    for bus in ["Music", "Ambience"] {
        let text = CATALOG_TEXT.replace(
            "cues: {}",
            &format!("cues: {{ \"x\": (bus: {bus}, files: [\"audio/x.ogg\"], gain: 0.5) }}"),
        );
        let errors = AudioCatalog::parse(&text).unwrap_err();
        assert!(
            errors[0].starts_with("catalog.ron: ") && errors[0].contains(bus),
            "{errors:?}"
        );
    }
}

#[test]
fn cue_names_and_fallbacks_are_checked() {
    assert_error(
        |f| {
            f.cues.insert("Shot.Ember".into(), cue_data());
        },
        "cues.\"Shot.Ember\": nome non valido",
    );
    for name in ["shot..ember", ".shot", "shot.", "."] {
        assert_error(
            |f| {
                f.cues.insert(name.into(), cue_data());
            },
            &format!("cues.\"{name}\": nome non valido"),
        );
    }
    for key in ["Shot", ".shot", "shot.", "shot..tide", ""] {
        assert_error(
            |f| {
                f.fallbacks.insert(key.into(), "shot.ember".into());
            },
            &format!("catalog.ron: fallbacks.\"{key}\": nome non valido"),
        );
    }
    assert_error(
        |f| {
            f.fallbacks.insert("shot".into(), "shot.tide".into());
        },
        "catalog.ron: fallbacks.\"shot\" = \"shot.tide\": il cue non esiste",
    );
    assert_error(
        |f| {
            f.fallbacks.insert("shot.ember".into(), "shot.ember".into());
        },
        "fallbacks.\"shot.ember\": è già il nome di un cue",
    );
}

#[test]
fn every_error_is_reported_not_only_the_first() {
    let errors = errors_after(|f| {
        f.version = 9;
        shot(f).gain = 2.0;
        shot(f).max_instances = 0;
    });
    assert_eq!(errors.len(), 3, "{errors:?}");
}

#[test]
fn an_unknown_field_inside_boost_is_an_error() {
    let cue = |coalesce: &str| {
        CATALOG_TEXT.replace(
            "cues: {}",
            &format!(
                "cues: {{ \"x\": (bus: Sfx, files: [\"audio/x.wav\"], gain: 0.5, \
                 coalesce: {coalesce}) }}"
            ),
        )
    };
    assert!(AudioCatalog::parse(&cue("Boost(max_db: 3.0)")).is_ok());
    let errors = AudioCatalog::parse(&cue("Boost(max_db: 3.0, max_dbb: 6.0)")).unwrap_err();
    assert!(
        errors[0].starts_with("catalog.ron: ") && errors[0].contains("max_dbb"),
        "{errors:?}"
    );
}

#[test]
fn syntax_errors_and_unknown_fields_name_the_file() {
    let errors = AudioCatalog::parse("(version: 1,").unwrap_err();
    assert!(errors[0].starts_with("catalog.ron: "), "{errors:?}");
    let typo = CATALOG_TEXT.replace("sfx_headroom", "sfx_headrom");
    let errors = AudioCatalog::parse(&typo).unwrap_err();
    assert!(
        errors[0].starts_with("catalog.ron: ") && errors[0].contains("sfx_headrom"),
        "{errors:?}"
    );
}

#[test]
fn optional_cue_fields_take_the_defaults_of_the_project() {
    let text = r#"(
        version: 1,
        voices: (max: 8, max_web: 8, reserved_critical: 1, starts_per_frame: 2,
                 starts_per_frame_web: 2, sfx_headroom: 2.0),
        trims: (music: 1.0, sfx: 1.0, ambience: 1.0, ui: 1.0),
        cues: {
            "ui.tick": (bus: Ui, files: ["audio/sfx/ui/tick.wav"], gain: 0.5),
            "shot.ember": (bus: Sfx, files: ["audio/sfx/a.wav"], gain: 0.4,
                           coalesce: Boost(max_db: 3.0), cull: View, on_limit: Restart),
        },
    )"#;
    let catalog = AudioCatalog::parse(text).unwrap();
    let tick = &catalog.cue(catalog.exact("ui.tick").unwrap()).unwrap().data;
    assert_eq!(tick.gain_jitter_db, DEFAULT_GAIN_JITTER_DB);
    assert_eq!(tick.pitch, DEFAULT_PITCH);
    assert_eq!(tick.length_s, DEFAULT_LENGTH_S);
    assert_eq!(tick.priority, Priority::Normal);
    assert_eq!(tick.cooldown_ms, 0);
    assert_eq!(tick.max_instances, 1);
    assert_eq!(tick.on_limit, OnLimit::Drop);
    assert_eq!(tick.coalesce, Coalesce::Single);
    assert_eq!(tick.cull, Cull::Always);
    let shot = &catalog
        .cue(catalog.exact("shot.ember").unwrap())
        .unwrap()
        .data;
    assert_eq!(shot.coalesce, Coalesce::Boost { max_db: 3.0 });
    assert_eq!((shot.cull, shot.on_limit), (Cull::View, OnLimit::Restart));
}

#[test]
fn cues_are_numbered_by_name_and_fallbacks_resolve_by_prefix() {
    let mut file = file_with(vec![
        ("shot.ember", cue_data()),
        ("enemy.die.small", cue_data()),
        ("chest.open", cue_data()),
    ]);
    file.fallbacks.insert("shot".into(), "shot.ember".into());
    file.fallbacks
        .insert("enemy.die".into(), "enemy.die.small".into());
    let catalog = AudioCatalog::compile(file).unwrap();
    let names: Vec<&str> = catalog.cues().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["chest.open", "enemy.die.small", "shot.ember"]);
    for (index, name) in names.iter().enumerate() {
        assert_eq!(catalog.exact(name), Some(CueId(index as u16)), "{name}");
        assert_eq!(catalog.id(name), catalog.exact(name));
    }
    let ember = catalog.exact("shot.ember");
    assert_eq!(catalog.id("shot.tide"), ember);
    assert_eq!(catalog.id("shot.tide.big"), ember);
    assert_eq!(catalog.id("shot"), ember);
    assert_eq!(
        catalog.id("enemy.die.boss_king"),
        catalog.exact("enemy.die.small")
    );
    assert_eq!(catalog.exact("shot.tide"), None);
    assert_eq!(catalog.id("pulse.tide"), None);
    assert_eq!(catalog.id(""), None);
    assert_eq!(catalog.cue(CueId(3)), None);
    assert_eq!(AudioCatalog::empty().id("shot.ember"), None);
}

#[test]
fn interface_cues_follow_the_effects_slider_with_their_own_trim() {
    use crate::audio::settings::Bus;
    assert_eq!(CueBus::Ui.bus(), Bus::Sfx);
    assert_eq!(CueBus::Sfx.bus(), Bus::Sfx);
    let catalog = AudioCatalog::parse(CATALOG_TEXT).unwrap();
    assert_eq!(catalog.trim(CueBus::Ui), catalog.trims.ui);
    assert_eq!(catalog.trim(CueBus::Sfx), catalog.trims.sfx);
}
