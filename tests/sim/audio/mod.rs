//! L'audio è separato dalla simulazione (D-069, `docs/design/AUDIO_SYSTEM.md` § 3.12 e § 12,
//! punti 1–3; AUDIO-1, resi più severi in AUDIO-2). Un criterio per file, ognuno con accanto il
//! proprio controllo contro i test vacui:
//! - [`headless`]: la run headless non ha risorse, asset né voci dell'audio (§ 12.1);
//! - [`transparency`]: con tutto l'audio del gioco al lavoro, riproduzione compresa, la
//!   simulazione resta identica bit per bit, e lo stesso confronto vede un player spostato di un
//!   ulp (§ 12.2);
//! - [`boundary`]: i moduli della simulazione non nominano l'audio né i tipi audio della prelude
//!   di Bevy, e solo `src/audio/playback.rs` crea voci (`AudioPlayer`) (§ 12.3).

mod boundary;
mod headless;
mod snapshot;
mod transparency;
