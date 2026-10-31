//! AUDIO-1: l'audio è separato dalla simulazione (D-069, `docs/design/AUDIO_SYSTEM.md` § 3.12 e
//! § 12, punti 1–3). La run headless non ha nulla dell'audio; con `AudioCorePlugin` che lavora
//! a ogni fotogramma la simulazione resta identica bit per bit; i moduli della simulazione non
//! importano l'audio e fuori da `src/audio/` nessuno crea voci (`AudioPlayer`).
//!
//! Oltre le ~400 righe di `AGENTS.md` per un motivo: i tre criteri stanno nel solo file assegnato
//! ad AUDIO-1 e ognuno ha accanto il proprio controllo contro i test vacui.

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::audio::{AudioPlayer, AudioSource, Pitch, PlaybackSettings};
use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase};
use pixel_monster_survivor::audio::{
    AudioCatalog, AudioCorePlugin, AudioManager, AudioMixer, AudioSet, AudioSettings, AudioStats,
    CueQueue, CueRequest, GameAudioPlugin, VoiceCommands,
};
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::combat::RunStats;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::gameplay::progression::Progression;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::gameplay::treasure::Treasury;
use pixel_monster_survivor::headless_app;

use crate::common::{clock, enemy_positions, gems, god, player_position, run_steps};

// --- 1. Headless muto ---

/// Tutti i pezzi dell'audio che un'app può avere: le risorse del client, gli asset e le voci di
/// `bevy_audio` (anche quelle del generatore `Pitch`).
const AUDIO_PIECES: [&str; 11] = [
    "AudioSettings",
    "CueQueue",
    "AudioMixer",
    "VoiceCommands",
    "AudioStats",
    "AudioCatalog",
    "Assets<AudioSource>",
    "Assets<Pitch>",
    "AudioPlayer<AudioSource>",
    "AudioPlayer<Pitch>",
    "PlaybackSettings",
];

fn any<C: Component>(world: &mut World) -> bool {
    world
        .query_filtered::<(), With<C>>()
        .iter(world)
        .next()
        .is_some()
}

/// I pezzi di [`AUDIO_PIECES`] presenti in `app`.
fn audio_footprint(app: &mut App) -> Vec<&'static str> {
    let world = app.world_mut();
    let present = [
        world.contains_resource::<AudioSettings>(),
        world.contains_resource::<CueQueue>(),
        world.contains_resource::<AudioMixer>(),
        world.contains_resource::<VoiceCommands>(),
        world.contains_resource::<AudioStats>(),
        world.contains_resource::<AudioCatalog>(),
        world.contains_resource::<Assets<AudioSource>>(),
        world.contains_resource::<Assets<Pitch>>(),
        any::<AudioPlayer>(world),
        any::<AudioPlayer<Pitch>>(world),
        any::<PlaybackSettings>(world),
    ];
    AUDIO_PIECES
        .into_iter()
        .zip(present)
        .filter_map(|(name, present)| present.then_some(name))
        .collect()
}

/// Criterio § 12.1: dopo 10 s di gioco simulato `headless_app` non ha risorse, asset né voci
/// dell'audio. Prima il controllo vede tutto in un'app con l'audio (non passa a vuoto).
#[test]
fn the_headless_run_has_nothing_of_the_audio() {
    let mut client = App::new();
    client
        .add_plugins(MinimalPlugins)
        .add_plugins(GameAudioPlugin { muted: false })
        .init_resource::<Assets<AudioSource>>()
        .init_resource::<Assets<Pitch>>();
    let world = client.world_mut();
    world.spawn(AudioPlayer::<AudioSource>(default()));
    world.spawn(AudioPlayer::<Pitch>(default()));
    client.update();
    assert_eq!(audio_footprint(&mut client), AUDIO_PIECES);

    let mut app = headless_app(3);
    run_steps(&mut app, 600);
    assert!(clock(&app) > 9.9, "run ferma a {} s", clock(&app));
    assert!(!enemy_positions(&mut app).is_empty(), "nessun nemico");
    assert_eq!(audio_footprint(&mut app), Vec::<&str>::new());
}

// --- 2. Trasparenza ---

/// Seed e durata del confronto, un controllo al secondo. 2 minuti invece dei 3 del § 12.2: 3
/// minuti costavano 11 s in `cargo test` (dev), 2 minuti restano sotto i 10 s.
const SEED: u64 = 7;
const STEPS: usize = 60 * 120;
const CHECK_EVERY: usize = 60;

/// Catalogo di prova: quattro cue che passano da tutte le regole del mixer (priorità fino a
/// `Critical`, `Restart` e `Drop`, cooldown, coalescenza con boost, cull fuori vista, varianti,
/// ripiego) con pochi posti e due partenze per fotogramma, così scattano anche furto, budget e
/// margine. I file non servono: in AUDIO-1 nessuno li carica.
const TEST_CATALOG: &str = r#"(
    version: 1,
    voices: (max: 6, max_web: 6, reserved_critical: 1, starts_per_frame: 2,
             starts_per_frame_web: 2, sfx_headroom: 1.5),
    trims: (music: 0.8, sfx: 0.8, ambience: 0.7, ui: 0.6),
    cues: {
        "enemy.hit": (bus: Sfx, files: ["audio/sfx/test/hit_1.wav", "audio/sfx/test/hit_2.wav"],
                      gain: 0.3, priority: Low, max_instances: 3, coalesce: Boost(max_db: 6.0),
                      cull: View, length_s: 0.2),
        "level.up": (bus: Sfx, files: ["audio/sfx/test/level.wav"], gain: 0.8, priority: High,
                     on_limit: Restart),
        "shot.ember": (bus: Sfx, files: ["audio/sfx/test/shot_1.wav", "audio/sfx/test/shot_2.wav",
                       "audio/sfx/test/shot_3.wav"], gain: 0.45, cooldown_ms: 50,
                       max_instances: 2, on_limit: Restart, cull: View, length_s: 0.5),
        "ui.tick": (bus: Ui, files: ["audio/sfx/test/tick.wav"], gain: 0.5, priority: Critical,
                    length_s: 0.1),
    },
    fallbacks: { "shot": "shot.ember" },
)"#;

/// Ascolto di prova per nome, come farà l'interfaccia: legge il player e chiede suoni (un
/// ripiego a ogni fotogramma, gli altri a intervalli, ogni tanto un nome che non esiste).
fn ask_by_name(
    mut audio: AudioManager,
    mut frame: Local<u32>,
    player: Query<&Position, With<Player>>,
) {
    *frame += 1;
    let at = player.single().ok().map(|p| p.0);
    audio.request("shot.tide", 3, at);
    if frame.is_multiple_of(10) {
        audio.play("ui.tick");
    }
    if frame.is_multiple_of(30) {
        audio.play("level.up");
    }
    if frame.is_multiple_of(100) {
        audio.play("nope");
    }
}

/// Ascolto di prova per id, come faranno gli osservatori di AUDIO-5: un colpo per ogni nemico
/// nella sua posizione e uno sempre fuori vista.
fn ask_by_id(
    mut queue: ResMut<CueQueue>,
    catalog: Res<AudioCatalog>,
    enemies: Query<&Position, With<Enemy>>,
    player: Query<&Position, With<Player>>,
) {
    let cue = catalog.exact("enemy.hit").expect("cue di prova");
    let far = player.single().ok().map(|p| p.0 + Vec2::splat(10_000.0));
    for at in enemies.iter().map(|p| p.0).chain(far) {
        let at = Some(at);
        queue.push(CueRequest { cue, count: 1, at });
    }
}

/// Stato della simulazione da confrontare; le posizioni in bit, ordinate come in `movement.rs`.
#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    app_state: Option<AppState>,
    run_phase: Option<RunPhase>,
    clock: Option<RunClock>,
    stats: Option<RunStats>,
    progression: Option<Progression>,
    treasury: Option<Treasury>,
    player: [u32; 2],
    enemies: Vec<[u32; 2]>,
    gems: Vec<([u32; 2], u32)>,
}

fn bits(p: Vec2) -> [u32; 2] {
    [p.x.to_bits(), p.y.to_bits()]
}

fn by_position(a: &Vec2, b: &Vec2) -> std::cmp::Ordering {
    a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y))
}

fn snapshot(app: &mut App) -> Snapshot {
    fn cloned<T: Resource + Clone>(app: &App) -> Option<T> {
        app.world().get_resource::<T>().cloned()
    }
    fn state<S: States>(app: &App) -> Option<S> {
        app.world()
            .get_resource::<State<S>>()
            .map(|s| s.get().clone())
    }
    let mut enemies = enemy_positions(app);
    enemies.sort_by(by_position);
    let mut gems = gems(app);
    gems.sort_by(|a, b| by_position(&a.0, &b.0).then(a.1.cmp(&b.1)));
    Snapshot {
        app_state: state(app),
        run_phase: state(app),
        clock: cloned(app),
        stats: cloned(app),
        progression: cloned(app),
        treasury: cloned(app),
        player: bits(player_position(app)),
        enemies: enemies.into_iter().map(bits).collect(),
        gems: gems.into_iter().map(|(p, v)| (bits(p), v)).collect(),
    }
}

/// Il primo campo diverso, con i due valori (le liste lunghe dal primo elemento diverso).
fn first_difference(a: &Snapshot, b: &Snapshot) -> Option<String> {
    fn field<T: PartialEq + Debug>(name: &str, a: &T, b: &T) -> Option<String> {
        (a != b).then(|| format!("{name}: senza audio {a:?}, con l'audio {b:?}"))
    }
    fn list<T: PartialEq + Debug>(name: &str, a: &[T], b: &[T]) -> Option<String> {
        let at = a.iter().zip(b).position(|(x, y)| x != y);
        match at {
            Some(i) => Some(format!(
                "{name}[{i}]: senza audio {:?}, con l'audio {:?}",
                a[i], b[i]
            )),
            None => (a.len() != b.len())
                .then(|| format!("{name}: {} senza audio, {} con l'audio", a.len(), b.len())),
        }
    }
    field("AppState", &a.app_state, &b.app_state)
        .or_else(|| field("RunPhase", &a.run_phase, &b.run_phase))
        .or_else(|| field("RunClock", &a.clock, &b.clock))
        .or_else(|| field("RunStats", &a.stats, &b.stats))
        .or_else(|| field("Progression", &a.progression, &b.progression))
        .or_else(|| field("Treasury", &a.treasury, &b.treasury))
        .or_else(|| field("player (bit)", &a.player, &b.player))
        .or_else(|| list("nemici (bit)", &a.enemies, &b.enemies))
        .or_else(|| list("gemme (bit, valore)", &a.gems, &b.gems))
}

/// Una run headless con l'autoplay e il player invulnerabile, con o senza l'audio al lavoro:
/// l'istantanea di ogni secondo e le statistiche dell'audio.
fn record(audio: bool) -> (Vec<Snapshot>, Option<AudioStats>) {
    let mut app = headless_app(SEED);
    if audio {
        app.insert_resource(AudioCatalog::parse(TEST_CATALOG).expect("catalogo di prova valido"))
            .add_plugins(AudioCorePlugin)
            .add_systems(Update, (ask_by_name, ask_by_id).in_set(AudioSet::Listen));
    }
    god(&mut app);
    let mut snapshots = Vec::with_capacity(STEPS / CHECK_EVERY);
    for step in 0..STEPS {
        app.insert_resource(MoveIntent(autoplay_direction(step as f32 / 60.0)));
        app.update();
        if (step + 1) % CHECK_EVERY == 0 {
            snapshots.push(snapshot(&mut app));
        }
    }
    (snapshots, app.world().get_resource::<AudioStats>().copied())
}

/// Criterio § 12.2: stessa seed, una run senza audio e una con `AudioCorePlugin` e un ascolto
/// che chiede suoni a ogni fotogramma, identiche a ogni secondo. Le due run vanno in parallelo.
#[test]
fn the_simulation_is_identical_with_and_without_the_audio() {
    let started = Instant::now();
    let ((plain, _), (heard, audio)) = std::thread::scope(|scope| {
        let heard = scope.spawn(|| record(true));
        let plain = record(false);
        (plain, heard.join().expect("run con l'audio"))
    });
    let elapsed = started.elapsed().as_secs_f32();
    assert_eq!(plain.len(), STEPS / CHECK_EVERY);
    assert_eq!(heard.len(), plain.len());
    for (second, (a, b)) in plain.iter().zip(&heard).enumerate() {
        // se l'audio scrivesse nella simulazione le due run qui divergerebbero e il test fallirebbe
        if let Some(difference) = first_difference(a, b) {
            panic!(
                "la simulazione cambia con l'audio a {} s: {difference}",
                second + 1
            );
        }
    }

    let end = plain.last().expect("istantanee");
    let audio = audio.expect("AudioStats nella run con l'audio");
    println!(
        "trasparenza: {} s di gioco ({STEPS} passi) x 2 app in parallelo in {elapsed:.2} s; \
         livello {}, sconfitti {}, nemici {}, gemme {}; audio {audio:?}",
        STEPS / 60,
        end.progression.as_ref().map_or(0, |p| p.level),
        end.stats.as_ref().map_or(0, |s| s.defeated),
        end.enemies.len(),
        end.gems.len(),
    );
    // il confronto prova qualcosa solo se la run è viva e il mixer ha lavorato davvero
    assert_eq!(end.run_phase, Some(RunPhase::Playing));
    let seconds = (STEPS / 60) as f32;
    assert!(end.clock.is_some_and(|c| c.elapsed > seconds - 1.0));
    assert!(end.stats.as_ref().is_some_and(|s| s.defeated > 0));
    assert!(end.progression.as_ref().is_some_and(|p| p.level > 1));
    assert!(!end.enemies.is_empty());
    // soglie che non dipendono dalla politica del mixer (la provano i test in `src/audio/`)
    let mixer = audio.mixer;
    let steps = STEPS as u64;
    for (what, count, at_least) in [
        ("richieste", mixer.requests, steps),
        ("voci avviate (1 al s)", mixer.started, steps / 60),
        ("restarted", mixer.restarted, 1),
        ("unknown_cues", audio.unknown_cues, 1),
    ] {
        assert!(
            count >= at_least,
            "mixer: {what} = {count}, atteso >= {at_least}"
        );
    }
    assert!(audio.live_voices > 0);
}

// --- 3. Confine nei sorgenti ---

/// Moduli della simulazione (§ 3.1, principio 1): né l'audio del gioco né quello di Bevy.
const SIMULATION: [&str; 5] = [
    "src/app.rs",
    "src/gameplay",
    "src/world",
    "src/data",
    "src/profile/unlocks.rs",
];
/// Nel testo dei moduli della simulazione, commenti compresi (§ 12.3).
const FORBIDDEN_TEXT: [&str; 3] = ["crate::audio", "bevy::audio", "bevy_audio"];

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

/// `audio` come identificatore intero: prende anche `use crate::{app, audio}` e `super::audio`,
/// non `audio_cue` né `AudioCue`.
fn names_audio(code: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices("audio").any(|(i, word)| {
        !code[..i].chars().next_back().is_some_and(ident)
            && !code[i + word.len()..].chars().next().is_some_and(ident)
    })
}

/// Le violazioni di una riga di un modulo della simulazione.
fn simulation_line_problems(line: &str) -> Vec<String> {
    let mut problems: Vec<String> = FORBIDDEN_TEXT
        .iter()
        .filter(|text| line.contains(*text))
        .map(|text| format!("contiene `{text}`"))
        .collect();
    if problems.is_empty() && names_audio(code_of(line)) {
        problems.push("nomina il modulo `audio` nel codice".into());
    }
    problems
}

/// Fuori da `src/audio/` nessun codice nomina `AudioPlayer` (le voci le crea solo l'audio).
fn creates_voices(line: &str) -> bool {
    code_of(line).contains("AudioPlayer")
}

/// Violazioni come `file:riga: problema: testo della riga`.
fn scan(root: &Path, files: &[PathBuf], problems: impl Fn(&str) -> Vec<String>) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("{file:?}: {e}"));
        let name = file.strip_prefix(root).unwrap_or(file);
        let name = name.to_string_lossy().replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            for problem in problems(line) {
                found.push(format!("{name}:{}: {problem}: {}", i + 1, line.trim()));
            }
        }
    }
    found
}

/// Le regole riconoscono i modi di importare l'audio e lasciano stare i nomi che lo contengono
/// soltanto (il controllo sui sorgenti non passa a vuoto).
#[test]
fn the_boundary_rules_catch_every_way_of_naming_the_audio() {
    let bad = [
        "use crate::audio::AudioManager;",
        "use crate::audio;",
        "use crate::{app, audio};",
        "    audio::{AudioSet, CueQueue},", // seconda riga di un `use crate::{` su più righe
        "use super::audio::CueQueue;",
        "use pixel_monster_survivor::audio::AudioSet;",
        "use bevy::audio::AudioPlayer;",
        "bevy_audio::AudioSink",
        "// usa bevy::audio",
    ];
    for line in bad {
        assert!(
            !simulation_line_problems(line).is_empty(),
            "non vista: {line}"
        );
    }
    let good = [
        "// l'audio osserva la simulazione senza scriverla",
        "let audio_cue = 1;",
        "struct AudioCue;",
        "fn radiofrequency() {}",
    ];
    for line in good {
        assert!(
            simulation_line_problems(line).is_empty(),
            "falso allarme: {line}"
        );
    }
    assert!(creates_voices("commands.spawn(AudioPlayer::new(handle));"));
    assert!(creates_voices(
        "use bevy::audio::{AudioPlayer, PlaybackSettings};"
    ));
    assert!(!creates_voices("// solo src/audio/ crea AudioPlayer"));
}

/// Criterio § 12.3: i moduli della simulazione non importano l'audio; `AudioPlayer` non compare
/// nel codice di `src/` fuori da `src/audio/` (in AUDIO-2 lo userà il solo `playback.rs`).
#[test]
fn the_simulation_sources_do_not_import_the_audio() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut simulation = Vec::new();
    for part in SIMULATION {
        let files = rust_files(&root.join(part));
        assert!(!files.is_empty(), "{part}: nessun file .rs");
        simulation.extend(files);
    }
    let mut problems = scan(root, &simulation, simulation_line_problems);

    let audio_dir = root.join("src/audio");
    let outside: Vec<PathBuf> = rust_files(&root.join("src"))
        .into_iter()
        .filter(|file| !file.starts_with(&audio_dir))
        .collect();
    assert!(outside.len() > simulation.len(), "src/ letto male");
    problems.extend(scan(root, &outside, |line| {
        if creates_voices(line) {
            vec!["crea voci fuori da src/audio/ (`AudioPlayer`)".into()]
        } else {
            Vec::new()
        }
    }));
    println!(
        "confine: {} file della simulazione, {} file di src/ fuori da src/audio/",
        simulation.len(),
        outside.len()
    );
    assert!(
        problems.is_empty(),
        "audio fuori posto (D-069):\n{}",
        problems.join("\n")
    );
}
