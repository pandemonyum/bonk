//! Ogni file del catalogo vero si decodifica (D-069, rischio noto): `AudioSource::decoder` fa
//! `.build().unwrap()` (`bevy_audio-0.20.0/src/audio_source.rs:101`), quindi un file corrotto o di
//! un formato non abilitato manderebbe in panic il gioco alla prima riproduzione. Qui ogni file
//! citato da `assets/audio/catalog.ron` si legge da `assets/`, si decodifica tutto e si misura.

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
