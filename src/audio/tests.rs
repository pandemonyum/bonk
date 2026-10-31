//! Test dell'audio: catalogo (`tests/catalog.rs`), decodifica dei file del catalogo vero
//! (`tests/decode.rs`), mixer (`tests/mixer.rs`, `tests/margin.rs` e `tests/mixer_time.rs`),
//! plugin, `AudioManager` e impostazioni dal profilo in un'app con `MinimalPlugins`
//! (`tests/plugin.rs`), riproduzione senza dispositivo (`tests/playback.rs`, `tests/playback_time.rs`). Qui gli aiuti
//! comuni: i cataloghi di prova si costruiscono nel codice, il catalogo vero si controlla per la
//! validità e per i suoi file.

use std::collections::BTreeMap;

use super::catalog::*;

mod catalog;
mod decode;
mod margin;
mod mixer;
mod mixer_time;
mod playback;
mod playback_time;
mod plugin;

/// Un cue di prova senza caso (niente variazioni di guadagno e pitch).
pub(super) fn cue_data() -> CueData {
    CueData {
        bus: CueBus::Sfx,
        files: vec!["audio/sfx/test/a.wav".into()],
        gain: 0.5,
        gain_jitter_db: 0.0,
        pitch: (1.0, 1.0),
        priority: Priority::Normal,
        cooldown_ms: 0,
        max_instances: 8,
        on_limit: OnLimit::Drop,
        coalesce: Coalesce::Single,
        cull: Cull::Always,
        length_s: 1.0,
    }
}

pub(super) fn file_with(cues: Vec<(&str, CueData)>) -> CatalogFile {
    CatalogFile {
        version: CATALOG_VERSION,
        voices: VoiceLimits {
            max: 24,
            max_web: 16,
            reserved_critical: 3,
            starts_per_frame: 6,
            starts_per_frame_web: 4,
            sfx_headroom: 3.0,
        },
        trims: Trims {
            music: 1.0,
            sfx: 1.0,
            ambience: 1.0,
            ui: 1.0,
        },
        cues: cues
            .into_iter()
            .map(|(name, cue)| (name.to_string(), cue))
            .collect(),
        fallbacks: BTreeMap::new(),
    }
}

pub(super) fn catalog_with(cues: Vec<(&str, CueData)>) -> AudioCatalog {
    AudioCatalog::compile(file_with(cues)).expect("catalogo di prova valido")
}
