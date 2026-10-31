//! Criterio § 12.2: stessa seed, 2 minuti di run headless con l'autoplay e il player
//! invulnerabile, senza l'audio e con tutto l'audio del gioco (`GameAudioPlugin`: nucleo,
//! impostazioni, fuoco della finestra e riproduzione, che crea e toglie entità di voce nello
//! stesso mondo della simulazione) più un ascolto che chiede suoni a ogni fotogramma: identiche
//! a ogni secondo. Una terza run, uguale alla seconda con in più un ascolto che sposta il player
//! di un ulp, prova che lo stesso confronto vede la scrittura più piccola. Le tre run vanno in
//! parallelo; quella senza audio è una sola, condivisa dai due test.

use std::sync::OnceLock;
use std::time::Instant;

use bevy::audio::AudioSource;
use bevy::prelude::*;
use pixel_monster_survivor::app::RunPhase;
use pixel_monster_survivor::audio::{
    AudioCatalog, AudioLibrary, AudioManager, AudioSet, AudioStats, CueQueue, CueRequest,
    GameAudioPlugin, PlaybackStats, PlaybackVoice,
};
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::headless_app;

use super::snapshot::{Snapshot, divergence, snapshot};
use crate::common::{count, god};

/// Seed e durata del confronto (i 7200 passi del § 12.2), un controllo al secondo.
const SEED: u64 = 7;
const STEPS: usize = 60 * 120;
const CHECK_EVERY: usize = 60;
/// Il passo in cui la terza run sposta il player: l'ultimo del secondo 60, subito prima del
/// controllo (un ulp tra due controlli la simulazione potrebbe riassorbirlo arrotondando). La
/// terza run si ferma lì.
const NUDGE_STEP: usize = 60 * 60;

/// Catalogo di prova: quattro cue che passano da tutte le regole del mixer (priorità fino a
/// `Critical`, `Restart` e `Drop`, cooldown, coalescenza con boost, cull fuori vista, varianti,
/// ripiego) con pochi posti e due partenze per fotogramma. Scattano budget del fotogramma,
/// limite d'istanze, `Restart`, cooldown, cull e margine, anche fermando voci per farci stare
/// quelle più alte (`headroom_stolen`); il furto dei posti (`stolen`) no: il margine (1,5) si
/// riempie prima dei 6 posti, e quel furto lo provano i test di `src/audio/tests/mixer.rs`. I
/// file non si leggono: la libreria punta ad asset già pronti, mai decodificati (niente
/// `AudioPlugin`, niente dispositivo).
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

/// Ascolto di prova per nome, come fa l'interfaccia: legge il player e chiede suoni (un ripiego
/// a ogni fotogramma, gli altri a intervalli, ogni tanto un nome che non esiste).
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
/// nella sua posizione; ogni 50 fotogrammi invece uno solo, fuori vista, che il mixer scarta
/// (`culled` sale qualunque cosa facciano i nemici).
fn ask_by_id(
    mut frame: Local<u32>,
    mut queue: ResMut<CueQueue>,
    catalog: Res<AudioCatalog>,
    enemies: Query<&Position, With<Enemy>>,
    player: Query<&Position, With<Player>>,
) {
    *frame += 1;
    let cue = catalog.exact("enemy.hit").expect("cue di prova");
    let mut push = |at: Vec2| {
        queue.push(CueRequest {
            cue,
            count: 1,
            at: Some(at),
        })
    };
    if frame.is_multiple_of(50) {
        if let Ok(player) = player.single() {
            push(player.0 + Vec2::splat(10_000.0));
        }
    } else {
        enemies.iter().for_each(|p| push(p.0));
    }
}

/// Un ascolto che scrive nella simulazione, quello che l'audio non deve mai fare: al passo
/// [`NUDGE_STEP`], dopo il passo fisso, sposta il player di un ulp in x.
fn nudge_player(mut frame: Local<usize>, mut player: Query<&mut Position, With<Player>>) {
    *frame += 1;
    if *frame == NUDGE_STEP {
        for mut at in &mut player {
            at.0.x = at.0.x.next_up();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Run {
    /// La simulazione da sola.
    Plain,
    /// Con tutto l'audio al lavoro.
    Heard,
    /// Come `Heard`, con [`nudge_player`]; si ferma a [`NUDGE_STEP`].
    Nudged,
}

struct Outcome {
    snapshots: Vec<Snapshot>,
    audio: Option<AudioStats>,
    playback: Option<PlaybackStats>,
    /// Entità di voce vive alla fine.
    voices: usize,
    seconds: f32,
}

/// Il catalogo di prova, la libreria con un asset pronto per ogni file, `GameAudioPlugin` e i
/// due ascolti.
fn add_the_audio(app: &mut App) {
    let catalog = AudioCatalog::parse(TEST_CATALOG).expect("catalogo di prova valido");
    let mut sources = Assets::<AudioSource>::default();
    let bytes: &[u8] = b"non decodificato: niente AudioPlugin";
    let handles = catalog
        .cues()
        .iter()
        .map(|cue| {
            let files = cue.data.files.iter();
            files
                .map(|_| {
                    sources.add(AudioSource {
                        bytes: bytes.into(),
                    })
                })
                .collect()
        })
        .collect();
    app.insert_resource(catalog)
        .insert_resource(sources)
        .insert_resource(AudioLibrary::from_handles(handles))
        .add_plugins(GameAudioPlugin { muted: false })
        .add_systems(Update, (ask_by_name, ask_by_id).in_set(AudioSet::Listen));
}

fn record(run: Run) -> Outcome {
    let started = Instant::now();
    let mut app = headless_app(SEED);
    if run != Run::Plain {
        add_the_audio(&mut app);
    }
    if run == Run::Nudged {
        app.add_systems(Update, nudge_player.in_set(AudioSet::Listen));
    }
    god(&mut app);
    let steps = if run == Run::Nudged {
        NUDGE_STEP
    } else {
        STEPS
    };
    let mut snapshots = Vec::with_capacity(steps / CHECK_EVERY);
    for step in 0..steps {
        app.insert_resource(MoveIntent(autoplay_direction(step as f32 / 60.0)));
        app.update();
        if (step + 1) % CHECK_EVERY == 0 {
            snapshots.push(snapshot(&mut app));
        }
    }
    assert_eq!(snapshots.len(), steps / CHECK_EVERY);
    Outcome {
        snapshots,
        audio: app.world().get_resource::<AudioStats>().copied(),
        playback: app.world().get_resource::<PlaybackStats>().cloned(),
        voices: count::<PlaybackVoice>(&mut app),
        seconds: started.elapsed().as_secs_f32(),
    }
}

/// La run senza audio, una sola per i due test che la confrontano.
fn plain() -> &'static Outcome {
    static PLAIN: OnceLock<Outcome> = OnceLock::new();
    PLAIN.get_or_init(|| record(Run::Plain))
}

/// Il confronto non passa a vuoto: un player spostato di un ulp lo fa fallire nel secondo dello
/// spostamento, e prima le run sono identiche (anche questa ha tutto l'audio).
#[test]
fn the_comparison_sees_the_player_moved_by_one_ulp() {
    let nudged = record(Run::Nudged);
    let plain = plain();
    let second = NUDGE_STEP / CHECK_EVERY;
    let found = divergence(&plain.snapshots, &nudged.snapshots);
    println!(
        "trasparenza, prova: {} s con l'audio e il player spostato in {:.2} s; {found:?}",
        NUDGE_STEP / 60,
        nudged.seconds
    );
    let (at, difference) = found.expect("il confronto non vede il player spostato di un ulp");
    assert_eq!(at, second, "{difference}");
    assert!(difference.starts_with("player (bit)"), "{difference}");
}

/// Il confronto vero: tutto l'audio al lavoro per 2 minuti non cambia nulla.
#[test]
fn the_simulation_is_identical_with_and_without_the_audio() {
    let started = Instant::now();
    let (plain, heard) = std::thread::scope(|scope| {
        let heard = scope.spawn(|| record(Run::Heard));
        let plain = plain();
        (plain, heard.join().expect("run con l'audio"))
    });
    let elapsed = started.elapsed().as_secs_f32();
    assert_eq!(heard.snapshots.len(), plain.snapshots.len());
    // se l'audio scrivesse nella simulazione qui le run divergerebbero (lo prova il test sopra)
    if let Some((second, difference)) = divergence(&plain.snapshots, &heard.snapshots) {
        panic!("la simulazione cambia con l'audio a {second} s: {difference}");
    }

    let end = plain.snapshots.last().expect("istantanee");
    let audio = heard.audio.expect("AudioStats nella run con l'audio");
    let playback = heard.playback.clone().expect("PlaybackStats");
    let mixer = audio.mixer;
    println!(
        "trasparenza: {} s di gioco ({STEPS} passi) senza e con l'audio in parallelo in \
         {elapsed:.2} s (run {:.2} s e {:.2} s); livello {}, sconfitti {}, nemici {}, gemme {}; \
         {} voci vive; audio {audio:?}; riproduzione {playback:?}",
        STEPS / 60,
        plain.seconds,
        heard.seconds,
        end.progression.as_ref().map_or(0, |p| p.level),
        end.stats.as_ref().map_or(0, |s| s.defeated),
        end.enemies.len(),
        end.gems.len(),
        heard.voices,
    );
    // il confronto prova qualcosa solo se la run è viva...
    assert_eq!(end.run_phase, Some(RunPhase::Playing));
    let seconds = (STEPS / 60) as f32;
    assert!(end.clock.is_some_and(|c| c.elapsed > seconds - 1.0));
    assert!(end.stats.as_ref().is_some_and(|s| s.defeated > 0));
    assert!(end.progression.as_ref().is_some_and(|p| p.level > 1));
    assert!(!end.enemies.is_empty());
    // ...e se l'audio ha lavorato davvero: almeno una richiesta per passo, ogni regola del mixer
    // che il catalogo di prova tocca, i nomi sconosciuti; poi la riproduzione nel mondo della
    // simulazione: ogni avvio un'entità, ogni fermata (scadenza, `Restart`, furto) un despawn
    // (senza `AudioPlugin` le entità non spariscono da sole), un'entità per voce viva
    let steps = STEPS as u64;
    assert!(
        mixer.requests >= steps,
        "mixer: {} richieste in {steps} passi",
        mixer.requests
    );
    for (what, count) in [
        ("started", mixer.started),
        ("culled", mixer.culled),
        ("cooldown", mixer.cooldown),
        ("restarted", mixer.restarted),
        ("headroom_stolen", mixer.headroom_stolen),
        ("unknown_cues", audio.unknown_cues),
        ("riproduzione: stopped", playback.stopped),
    ] {
        assert!(count > 0, "{what} = 0: l'audio non ha lavorato come atteso");
    }
    assert_eq!(playback.not_ready, 0, "file di prova non pronti");
    assert_eq!(playback.spawned, mixer.started, "avvii senza entità");
    let stops = mixer.expired + mixer.restarted + mixer.stolen + mixer.headroom_stolen;
    assert_eq!(playback.stopped, stops, "fermate senza despawn");
    assert_eq!(heard.voices, audio.live_voices, "voci vive senza entità");
}
