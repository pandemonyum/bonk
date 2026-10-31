//! Criterio § 12.3, sul testo dei sorgenti. Due regole, ognuna con i suoi casi contro i falsi
//! negativi e i falsi positivi e una prova sul codice vero:
//! 1. i moduli della simulazione ([`SIMULATION`]) non nominano l'audio del gioco né quello di
//!    Bevy: niente `crate::audio`, `bevy::audio`, `bevy_audio` (commenti compresi), niente
//!    modulo `audio` e niente tipi audio della prelude di Bevy ([`PRELUDE_AUDIO`]) nel codice,
//!    perché `use bevy::prelude::*` li porta dentro senza scrivere `bevy::audio`;
//! 2. `AudioPlayer` (crea le voci) sta solo nel codice di `src/audio/playback.rs` e dei test
//!    sotto `src/audio/tests/`.
//!
//! `Volume` non è vietato: non è nella prelude (si importa solo da `bevy::audio` o `bevy_audio`,
//! che la regola 1 vede già) e un tipo `Volume` della simulazione sarebbe un falso allarme. Oggi
//! né `Volume` né `Pitch` compaiono nei moduli della simulazione, quindi vietare `Pitch` (che la
//! prelude porta) non dà falsi allarmi.

use std::path::{Path, PathBuf};

/// Moduli della simulazione (§ 3.1, principio 1).
const SIMULATION: [&str; 5] = [
    "src/app.rs",
    "src/gameplay",
    "src/world",
    "src/data",
    "src/profile/unlocks.rs",
];
/// Nel testo dei moduli della simulazione, commenti compresi.
const FORBIDDEN_TEXT: [&str; 3] = ["crate::audio", "bevy::audio", "bevy_audio"];
/// I tipi che `bevy::prelude::*` riesporta da `bevy_audio` (`bevy_internal-0.20.0/src/
/// prelude.rs`, `bevy_audio-0.20.0/src/lib.rs`), tranne `AudioPlayer`, che ha la regola 2.
const PRELUDE_AUDIO: [&str; 9] = [
    "AudioSink",
    "AudioSinkPlayback",
    "AudioSource",
    "Decodable",
    "GlobalVolume",
    "Pitch",
    "PlaybackSettings",
    "SpatialAudioSink",
    "SpatialListener",
];
/// Il solo file che crea voci, e la cartella dei suoi test.
const PLAYBACK: &str = "src/audio/playback.rs";
const AUDIO_TESTS: &str = "src/audio/tests";

/// I file `.rs` sotto `path` (o `path` stesso), in ordine.
fn rust_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let mut files = Vec::new();
    let entries = std::fs::read_dir(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    for entry in entries {
        let path = entry.expect("voce della cartella").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// La riga senza il commento di riga (`//`, `///`, `//!`).
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// `word` come identificatore intero: `audio` prende `use crate::{app, audio}` e `super::audio`,
/// non `audio_cue` né `AudioCue`; `Pitch` prende `Handle<Pitch>`, non `PitchShift`.
fn names(code: &str, word: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(word).any(|(i, found)| {
        !code[..i].chars().next_back().is_some_and(ident)
            && !code[i + found.len()..].chars().next().is_some_and(ident)
    })
}

/// Regola 1: le violazioni di una riga di un modulo della simulazione.
fn simulation_line_problems(line: &str) -> Vec<String> {
    let code = code_of(line);
    let mut problems: Vec<String> = FORBIDDEN_TEXT
        .iter()
        .filter(|text| line.contains(*text))
        .map(|text| format!("contiene `{text}`"))
        .collect();
    if problems.is_empty() && names(code, "audio") {
        problems.push("nomina il modulo `audio` nel codice".into());
    }
    problems.extend(
        PRELUDE_AUDIO
            .iter()
            .filter(|name| names(code, name))
            .map(|name| format!("nomina `{name}`, tipo audio della prelude di Bevy")),
    );
    problems
}

/// Regola 2: la riga nomina `AudioPlayer` nel codice.
fn creates_voices(line: &str) -> bool {
    code_of(line).contains("AudioPlayer")
}

/// Regola 2: il file (relativo alla radice del crate) può creare voci.
fn may_create_voices(file: &Path) -> bool {
    file == Path::new(PLAYBACK) || file.starts_with(AUDIO_TESTS)
}

/// Violazioni come `file:riga: problema: testo della riga`, per i file in cui `applies`.
fn scan(
    root: &Path,
    files: &[PathBuf],
    applies: impl Fn(&Path) -> bool,
    problems: impl Fn(&str) -> Vec<String>,
) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let relative = file.strip_prefix(root).unwrap_or(file);
        if !applies(relative) {
            continue;
        }
        let text = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("{file:?}: {e}"));
        let name = relative.to_string_lossy().replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            for problem in problems(line) {
                found.push(format!("{name}:{}: {problem}: {}", i + 1, line.trim()));
            }
        }
    }
    found
}

fn voice_problems(line: &str) -> Vec<String> {
    if creates_voices(line) {
        vec!["crea voci fuori da src/audio/playback.rs (`AudioPlayer`)".into()]
    } else {
        Vec::new()
    }
}

#[test]
fn the_simulation_rule_catches_every_way_of_naming_the_audio() {
    let bad = [
        "use crate::audio::AudioManager;",
        "use crate::audio;",
        "use crate::{app, audio};",
        "    audio::{AudioSet, CueQueue},", // seconda riga di un `use crate::{` su più righe
        "use super::audio::CueQueue;",
        "use pixel_monster_survivor::audio::AudioSet;",
        "use bevy::audio::AudioPlayer;",
        "use bevy::audio::Volume;",
        "bevy_audio::AudioSink",
        "// usa bevy::audio",
        // i tipi della prelude, senza `bevy::audio` nella riga
        "    handle: Handle<AudioSource>,",
        "    sources: Res<Assets<AudioSource>>,",
        "fn quiet(sinks: Query<&AudioSink>) {}",
        "fn fade<S: AudioSinkPlayback>(sink: &mut S) {}",
        "commands.spawn((player, PlaybackSettings::DESPAWN));",
        "    PlaybackSettings,", // riga interna di un `use bevy::audio::{` su più righe
        "fn louder(mut volume: ResMut<GlobalVolume>) {}",
        "commands.spawn(SpatialListener::new(4.0));",
        "fn moving(sinks: Query<&SpatialAudioSink>) {}",
        "let tone = pitches.add(Pitch::new(440.0, duration));",
        "impl Decodable for Noise {}",
        "let scale: bevy::prelude::AudioSource = todo!();",
    ];
    for line in bad {
        assert!(
            !simulation_line_problems(line).is_empty(),
            "non vista: {line}"
        );
    }
    let good = [
        "// l'audio osserva la simulazione senza scriverla",
        "// qui niente PlaybackSettings né AudioSource: stanno nel client",
        "let audio_cue = 1;",
        "struct AudioCue;",
        "fn radiofrequency() {}",
        "let pitch = 1.0;",
        "struct PitchShift;",
        "fn audio_source_count() -> usize { 0 }",
        "struct MyAudioSource;",
        "let playback_settings = 3;",
        "struct Volume(f32);", // un tipo della simulazione: `Volume` non è vietato
        "let volume = 0.5;",
    ];
    for line in good {
        assert!(
            simulation_line_problems(line).is_empty(),
            "falso allarme: {line}: {:?}",
            simulation_line_problems(line)
        );
    }
}

#[test]
fn the_voice_rule_allows_only_playback_and_its_tests() {
    assert!(creates_voices("commands.spawn(AudioPlayer::new(handle));"));
    assert!(creates_voices(
        "world.spawn(AudioPlayer::<Pitch>(default()));"
    ));
    assert!(creates_voices(
        "use bevy::audio::{AudioPlayer, PlaybackSettings};"
    ));
    assert!(creates_voices("type Voice = AudioPlayer;"));
    assert!(!creates_voices("// solo playback.rs crea AudioPlayer"));
    assert!(!creates_voices("let audio_player = 1;"));

    for allowed in [
        "src/audio/playback.rs",
        "src/audio/tests/playback.rs",
        "src/audio/tests/more/voices.rs",
    ] {
        assert!(may_create_voices(Path::new(allowed)), "vietato: {allowed}");
    }
    for denied in [
        "src/audio/mod.rs",
        "src/audio/focus.rs",
        "src/audio/tests.rs",
        "src/audio/testsuite.rs",
        "src/audio/playback/extra.rs",
        "src/audio/playback.rs.old/x.rs",
        "src/presentation/options_menu.rs",
        "src/gameplay/player.rs",
        "src/lib.rs",
    ] {
        assert!(!may_create_voices(Path::new(denied)), "ammesso: {denied}");
    }
}

/// Le due regole su tutti i sorgenti; prima, applicate a `playback.rs`, devono vedere l'audio
/// che c'è davvero (il controllo legge i file giusti e non passa a vuoto).
#[test]
fn the_simulation_sources_do_not_import_the_audio() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let playback = [root.join(PLAYBACK)];
    let all = |_: &Path| true;
    let seen = scan(root, &playback, all, simulation_line_problems);
    assert!(seen.iter().any(|p| p.contains("contiene `bevy::audio`")));
    assert!(seen.iter().any(|p| p.contains("tipo audio della prelude")));
    let voices = scan(root, &playback, all, voice_problems);
    assert!(!voices.is_empty(), "playback.rs senza AudioPlayer?");

    let mut simulation = Vec::new();
    for part in SIMULATION {
        let files = rust_files(&root.join(part));
        assert!(!files.is_empty(), "{part}: nessun file .rs");
        simulation.extend(files);
    }
    let mut problems = scan(root, &simulation, all, simulation_line_problems);
    let sources = rust_files(&root.join("src"));
    assert!(sources.len() > simulation.len(), "src/ letto male");
    let checked = sources
        .iter()
        .filter(|file| !may_create_voices(file.strip_prefix(root).unwrap_or(file)))
        .count();
    problems.extend(scan(
        root,
        &sources,
        |file| !may_create_voices(file),
        voice_problems,
    ));
    println!(
        "confine: {} file della simulazione, {checked} file di src/ senza voci su {}; in \
         playback.rs {} righe che la regola 1 rifiuterebbe e {} con AudioPlayer",
        simulation.len(),
        sources.len(),
        seen.len(),
        voices.len()
    );
    assert!(
        problems.is_empty(),
        "audio fuori posto (D-069):\n{}",
        problems.join("\n")
    );
}
