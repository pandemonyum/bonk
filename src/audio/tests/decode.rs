//! Ogni file del catalogo vero si decodifica (D-069, rischio noto): `AudioSource::decoder` fa
//! `.build().unwrap()` (`bevy_audio-0.20.0/src/audio_source.rs:101`), quindi un file corrotto o di
//! un formato non abilitato manderebbe in panic il gioco alla prima riproduzione. Qui ogni file
//! citato da `assets/audio/catalog.ron` si legge da `assets/`, si decodifica tutto e si misura;
//! le tracce della musica anche nel loop, ripetute come le ripete Bevy.

use std::path::Path;

use bevy::audio::{AudioSource, Decodable, Source};

use crate::audio::catalog::{AudioCatalog, CATALOG_TEXT};
use crate::audio::mixer::db_to_gain;

/// Picco massimo ammesso di un file: −1 dBFS (spazio per il mix e per le varianti di guadagno).
const PEAK_DBFS: f32 = -1.0;
/// Un passo di quantizzazione a 16 bit: il picco scritto a −1 dBFS può arrotondare in su.
const LSB_16: f32 = 1.0 / 32768.0;

/// Durata in secondi e picco lineare di un file, dopo averlo decodificato tutto.
fn measure(bytes: Vec<u8>) -> (f64, f32) {
    let source = AudioSource {
        bytes: bytes.into(),
    };
    let mut decoder = source.decoder();
    let channels = u64::from(decoder.channels().get());
    let rate = f64::from(decoder.sample_rate().get());
    let mut samples: u64 = 0;
    let mut peak: f32 = 0.0;
    for sample in decoder.by_ref() {
        samples += 1;
        peak = peak.max(sample.abs());
    }
    let frames = samples / channels.max(1);
    (frames as f64 / rate, peak)
}

#[test]
fn every_file_of_the_catalog_decodes_fits_its_length_and_keeps_its_headroom() {
    let catalog = AudioCatalog::parse(CATALOG_TEXT).expect("catalogo valido");
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let limit = db_to_gain(PEAK_DBFS) + LSB_16;
    let mut checked = 0;
    let mut problems = Vec::new();
    for cue in catalog.cues() {
        let slowest = cue.data.pitch.0.min(cue.data.pitch.1);
        let longest = f64::from(cue.data.length_s) * f64::from(slowest);
        for file in &cue.data.files {
            let bytes = std::fs::read(assets.join(file))
                .unwrap_or_else(|e| panic!("{}: {file}: {e}", cue.name));
            let measured = std::panic::catch_unwind(|| measure(bytes));
            let Ok((seconds, peak)) = measured else {
                problems.push(format!("{file}: la decodifica va in panic"));
                continue;
            };
            checked += 1;
            println!(
                "{}: {file}: {:.1} ms (al più {:.1} ms), picco {:.2} dBFS",
                cue.name,
                seconds * 1000.0,
                longest * 1000.0,
                20.0 * peak.max(1e-9).log10()
            );
            if seconds <= 0.0 {
                problems.push(format!("{file}: nessun campione"));
            }
            if seconds > longest {
                problems.push(format!(
                    "{file}: {seconds:.4} s, oltre length_s × pitch minimo = {longest:.4} s \
                     ({}: una voce che non suona scadrebbe prima della fine)",
                    cue.name
                ));
            }
            if peak > limit {
                problems.push(format!("{file}: picco {peak:.4}, oltre {PEAK_DBFS} dBFS"));
            }
        }
    }
    let files: usize = catalog.cues().iter().map(|c| c.data.files.len()).sum();
    println!(
        "decodifica: {checked} file controllati su {files}, {} cue",
        catalog.cues().len()
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert_eq!(checked, files);
}

/// Il controllo non passa a vuoto: un file che non è audio va in panic nel decoder di Bevy (il
/// test sopra lo trasforma in un errore leggibile).
#[test]
fn a_file_that_is_not_audio_is_caught() {
    let caught = std::panic::catch_unwind(|| measure(b"RIFF niente di audio".to_vec()));
    assert!(caught.is_err());
}

/// La musica è Vorbis stereo a 44,1 kHz (progetto § 5.1).
const MUSIC_CHANNELS: usize = 2;
const MUSIC_RATE: u32 = 44_100;
/// Il gradino al giro si confronta con questo percentile delle differenze fra campioni vicini,
/// per canale (come `scripts/audio_inbox.py`).
const JUMP_PERCENTILE: f64 = 99.9;
/// Campioni per blocco al più nel `Buffered` di rodio (`rodio-0.22.2/src/source/buffered.rs`).
const RODIO_SPAN: usize = 32_768;
/// Finestra attorno al punto di giro per i silenzi: un secondo per lato.
const SEAM_FRAMES: usize = 44_100;

/// Tutta la traccia: canali, frequenza, campioni interleaved e blocchi del `Buffered` di rodio,
/// letti come fa `buffered::extract` (al più `current_span_len` campioni per volta, fino a
/// [`RODIO_SPAN`]): sono i blocchi che il loop di Bevy tiene in memoria.
fn decode_spans(source: &AudioSource) -> (usize, u32, Vec<f32>, usize) {
    let mut decoder = source.decoder();
    let channels = usize::from(decoder.channels().get());
    let rate = decoder.sample_rate().get();
    let mut samples = Vec::new();
    let mut spans = 0;
    loop {
        let span = decoder.current_span_len();
        if span == Some(0) {
            break;
        }
        let before = samples.len();
        let take = span.unwrap_or(RODIO_SPAN).min(RODIO_SPAN);
        samples.extend(decoder.by_ref().take(take));
        if samples.len() == before {
            break;
        }
        spans += 1;
    }
    (channels, rate, samples, spans)
}

/// Il campo intero `field` della voce di `path` in `assets/manifest.json` (le voci sono oggetti
/// piatti che cominciano con `"path"`: si legge fino alla prima `}`).
fn manifest_number(manifest: &str, path: &str, field: &str) -> Option<u64> {
    let start = manifest.find(&format!("\"path\": \"{path}\""))?;
    let entry = &manifest[start..start + manifest[start..].find('}')?];
    let key = format!("\"{field}\":");
    let at = entry.find(&key)? + key.len();
    let digits: String = entry[at..]
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Percentile a rango più vicino (come `percentile_top` di `scripts/audio_inbox.py`).
fn percentile(mut values: Vec<f32>, percentile: f64) -> f32 {
    let rank = ((percentile / 100.0) * values.len() as f64).ceil() as usize;
    let index = rank.clamp(1, values.len()) - 1;
    *values.select_nth_unstable_by(index, f32::total_cmp).1
}

/// La serie più lunga di frame tutti a zero.
fn longest_silence(samples: &[f32], channels: usize) -> usize {
    let (mut longest, mut run) = (0, 0);
    for frame in samples.chunks(channels) {
        run = if frame.iter().all(|&s| s == 0.0) {
            run + 1
        } else {
            0
        };
        longest = longest.max(run);
    }
    longest
}

/// Ogni traccia si decodifica senza panic, è stereo a 44,1 kHz, lunga come dice il manifest (±1
/// campione) e con il picco sotto −1 dBFS; ripetuta come la ripete Bevy con `PlaybackMode::Loop`
/// (`decoder.repeat_infinite()`, `bevy_audio-0.20.0/src/audio_output.rs:235`) il giro torna
/// all'inizio senza vuoto (nessun campione perso o aggiunto, nessuna serie di zeri che il file
/// non ha) e senza gradino (salto al giro entro il 99,9° percentile dei salti del file).
#[test]
fn every_music_track_decodes_and_loops_in_bevy_without_a_gap_or_a_step() {
    let started = std::time::Instant::now();
    let catalog = AudioCatalog::parse(CATALOG_TEXT).expect("catalogo valido");
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let manifest = std::fs::read_to_string(assets.join("manifest.json")).expect("manifest");
    let limit = db_to_gain(PEAK_DBFS) + LSB_16;
    let mut problems = Vec::new();
    for track in catalog.music.tracks() {
        let file = &track.file;
        let bytes = std::fs::read(assets.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        let source = AudioSource {
            bytes: bytes.into(),
        };
        let decoding = std::time::Instant::now();
        let Ok((channels, rate, samples, spans)) =
            std::panic::catch_unwind(|| decode_spans(&source))
        else {
            problems.push(format!("{file}: la decodifica va in panic"));
            continue;
        };
        let decoded_in = decoding.elapsed();
        let frames = samples.len() / channels.max(1);
        let expected = manifest_number(&manifest, file, "samples");
        let peak = samples.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
        println!(
            "{}: {file}: {channels} canali, {rate} Hz, {frames} campioni (manifest {expected:?}), \
             picco {:.2} dBFS, decodificata in {:.2} s; loop in memoria: {} byte di f32 in \
             {spans} blocchi",
            track.name,
            20.0 * peak.max(1e-9).log10(),
            decoded_in.as_secs_f64(),
            samples.len() * std::mem::size_of::<f32>()
        );
        if channels != MUSIC_CHANNELS || rate != MUSIC_RATE {
            problems.push(format!(
                "{file}: {channels} canali a {rate} Hz, attesi {MUSIC_CHANNELS} a {MUSIC_RATE}"
            ));
            continue;
        }
        match expected {
            Some(n) if frames.abs_diff(n as usize) <= 1 => {}
            _ => problems.push(format!(
                "{file}: {frames} campioni, il manifest ne dice {expected:?} (±1)"
            )),
        }
        if peak > limit {
            problems.push(format!("{file}: picco {peak:.4}, oltre {PEAK_DBFS} dBFS"));
        }

        // il loop di Bevy: il primo giro è il file, poi si riparte dal primo campione
        let lap = samples.len();
        let after = SEAM_FRAMES.min(frames) * channels;
        let mut repeated = source.decoder().repeat_infinite();
        let mut first_lap = repeated.by_ref().take(lap).zip(&samples);
        if let Some(i) = first_lap.position(|(a, &b)| a != b) {
            problems.push(format!(
                "{file}: il primo giro del loop cambia al campione {i}"
            ));
        }
        let second: Vec<f32> = repeated.take(after).collect();
        if second[..] != samples[..after] {
            problems.push(format!(
                "{file}: dopo il giro il loop non riparte dall'inizio (campioni persi o aggiunti)"
            ));
        }
        let mut seam = samples[lap - after..].to_vec();
        seam.extend(&second);
        let mut circle = samples[lap - after..].to_vec();
        circle.extend(&samples[..after]);
        let (gap, own) = (
            longest_silence(&seam, channels),
            longest_silence(&circle, channels),
        );
        if gap > own {
            problems.push(format!(
                "{file}: {gap} frame di zeri al giro, il file ne ha {own}"
            ));
        }
        for c in 0..channels {
            let channel: Vec<f32> = samples[c..].iter().step_by(channels).copied().collect();
            let steps: Vec<f32> = channel.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
            let threshold = percentile(steps, JUMP_PERCENTILE);
            let jump = (second[c] - samples[lap - channels + c]).abs();
            println!(
                "{}: canale {c}: salto al giro {jump:.5}, {JUMP_PERCENTILE}° percentile dei \
                 salti {threshold:.5}; zeri al giro {gap} (nel file attorno al giro {own})",
                track.name
            );
            if jump > threshold {
                problems.push(format!(
                    "{file}: canale {c}, gradino al giro {jump:.5} oltre il {JUMP_PERCENTILE}° \
                     percentile {threshold:.5}"
                ));
            }
        }
    }
    println!(
        "musica: {} tracce in {:.2} s",
        catalog.music.tracks().len(),
        started.elapsed().as_secs_f64()
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_manifest_reader_and_the_loop_measures_work() {
    let manifest = r#"[{"path": "audio/a.ogg", "samples": 12, "x": 1},
        {"path": "audio/b.ogg", "channels": 2, "samples": 34}]"#;
    assert_eq!(
        manifest_number(manifest, "audio/a.ogg", "samples"),
        Some(12)
    );
    assert_eq!(
        manifest_number(manifest, "audio/b.ogg", "samples"),
        Some(34)
    );
    assert_eq!(manifest_number(manifest, "audio/a.ogg", "channels"), None);
    assert_eq!(manifest_number(manifest, "audio/c.ogg", "samples"), None);
    assert_eq!(percentile(vec![5.0, 1.0, 3.0, 2.0, 4.0], 50.0), 3.0);
    assert_eq!(percentile(vec![5.0, 1.0, 3.0, 2.0, 4.0], 99.9), 5.0);
    let frames = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
    assert_eq!(longest_silence(&frames, 2), 2);
}
