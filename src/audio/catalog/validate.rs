//! Regole del catalogo audio oltre alla sintassi. Ogni errore nomina il campo:
//! `catalog.ron: cues."shot.ember".gain = 1.5: deve stare in (0, 1]`.

use super::music::{MUSIC_EXTENSION, MusicFile};
use super::{CATALOG_VERSION, CUE_EXTENSION, CatalogFile, Coalesce, CueData};

/// Regola dei nomi di cue e ripieghi, per i messaggi.
const NAME_RULE: &str =
    "nome non valido (minuscole, cifre e _ in parole separate da un punto, es. \"shot.ember\")";

/// Pitch ammesso (velocità di riproduzione): oltre un'ottava in su o in giù non è più lo stesso
/// suono.
const PITCH_RANGE: (f32, f32) = (0.5, 2.0);
/// Variazione di guadagno massima di un cue, ± dB.
const MAX_GAIN_JITTER_DB: f32 = 6.0;
/// Boost di coalescenza massimo, dB (×4 in ampiezza).
const MAX_BOOST_DB: f32 = 12.0;

/// Tutti gli errori del catalogo; vuoto se è valido.
pub(super) fn validate(file: &CatalogFile) -> Vec<String> {
    let mut errors = Vec::new();
    let mut push = |message: String| errors.push(format!("catalog.ron: {message}"));
    if file.version != CATALOG_VERSION {
        push(format!(
            "version {} non supportata (attesa {CATALOG_VERSION})",
            file.version
        ));
    }

    let voices = &file.voices;
    for (field, max) in [("max", voices.max), ("max_web", voices.max_web)] {
        if voices.reserved_critical >= max {
            push(format!(
                "voices.reserved_critical = {}: deve essere minore di voices.{field} ({max})",
                voices.reserved_critical
            ));
        }
    }
    for (field, value) in [
        ("starts_per_frame", voices.starts_per_frame),
        ("starts_per_frame_web", voices.starts_per_frame_web),
    ] {
        if value == 0 {
            push(format!("voices.{field} = 0: deve essere >= 1"));
        }
    }
    if !(voices.sfx_headroom.is_finite() && voices.sfx_headroom > 0.0) {
        push(format!(
            "voices.sfx_headroom = {}: deve essere > 0",
            voices.sfx_headroom
        ));
    }

    let trims = &file.trims;
    for (field, value) in [
        ("music", trims.music),
        ("sfx", trims.sfx),
        ("ambience", trims.ambience),
        ("ui", trims.ui),
    ] {
        if !unit_gain(value) {
            push(format!("trims.{field} = {value}: deve stare in (0, 1]"));
        }
    }

    if file.cues.len() > usize::from(u16::MAX) + 1 {
        push(format!(
            "cues: {} cue, al massimo {}",
            file.cues.len(),
            usize::from(u16::MAX) + 1
        ));
    }
    for (name, cue) in &file.cues {
        for message in cue_errors(name, cue) {
            push(message);
        }
    }

    for (from, to) in &file.fallbacks {
        if !valid_cue_name(from) {
            push(format!("fallbacks.\"{from}\": {NAME_RULE}"));
        }
        if !file.cues.contains_key(to) {
            push(format!(
                "fallbacks.\"{from}\" = \"{to}\": il cue non esiste"
            ));
        }
        if file.cues.contains_key(from) {
            push(format!(
                "fallbacks.\"{from}\": è già il nome di un cue, il ripiego non si userebbe mai"
            ));
        }
    }
    for message in music_errors(&file.music) {
        push(message);
    }
    errors
}

/// La sezione `music`: tracce Ogg con un guadagno in (0, 1], riferimenti a tracce che esistono,
/// durate finite e ≥ 0.
fn music_errors(music: &MusicFile) -> Vec<String> {
    let mut errors = Vec::new();
    if music.tracks.len() > usize::from(u16::MAX) + 1 {
        errors.push(format!(
            "music.tracks: {} tracce, al massimo {}",
            music.tracks.len(),
            usize::from(u16::MAX) + 1
        ));
    }
    for (name, track) in &music.tracks {
        let at = format!("music.tracks.\"{name}\"");
        if !valid_cue_name(name) {
            errors.push(format!("{at}: {NAME_RULE}"));
        }
        if !unit_gain(track.gain) {
            errors.push(format!("{at}.gain = {}: deve stare in (0, 1]", track.gain));
        }
        let file = &track.file;
        if !valid_file_name(file) {
            errors.push(format!(
                "{at}.file = \"{file}\": serve un percorso relativo ad assets/ con minuscole, \
                 cifre, _ e / (es. \"audio/music/home.ogg\")"
            ));
        } else if !file.ends_with(&format!(".{MUSIC_EXTENSION}")) {
            errors.push(format!(
                "{at}.file = \"{file}\": la musica vuole file .{MUSIC_EXTENSION} (Ogg Vorbis)"
            ));
        }
    }
    for (stage, track) in &music.run_by_stage {
        if !valid_cue_name(stage) {
            errors.push(format!(
                "music.run_by_stage.\"{stage}\": id di stage non valido (minuscole, cifre e _, \
                 come in game-data/stages.ron)"
            ));
        }
        if !music.tracks.contains_key(track) {
            errors.push(format!(
                "music.run_by_stage.\"{stage}\" = \"{track}\": la traccia non esiste in \
                 music.tracks"
            ));
        }
    }
    if let Some(track) = &music.run_default
        && !music.tracks.contains_key(track)
    {
        errors.push(format!(
            "music.run_default = \"{track}\": la traccia non esiste in music.tracks"
        ));
    }
    let fades = &music.fades;
    for (field, value) in [
        ("to_run", fades.to_run),
        ("to_home", fades.to_home),
        ("focus", fades.focus),
    ] {
        if !(value.is_finite() && value >= 0.0) {
            errors.push(format!(
                "music.fades.{field} = {value}: deve essere >= 0 (secondi)"
            ));
        }
    }
    errors
}

fn cue_errors(name: &str, cue: &CueData) -> Vec<String> {
    let mut errors = Vec::new();
    let at = format!("cues.\"{name}\"");
    if !valid_cue_name(name) {
        errors.push(format!("{at}: {NAME_RULE}"));
    }
    if !unit_gain(cue.gain) {
        errors.push(format!("{at}.gain = {}: deve stare in (0, 1]", cue.gain));
    }
    let jitter = cue.gain_jitter_db;
    if !(jitter.is_finite() && (0.0..=MAX_GAIN_JITTER_DB).contains(&jitter)) {
        errors.push(format!(
            "{at}.gain_jitter_db = {jitter}: deve stare in [0, {MAX_GAIN_JITTER_DB}]"
        ));
    }
    let (low, high) = cue.pitch;
    let in_range = |p: f32| p.is_finite() && (PITCH_RANGE.0..=PITCH_RANGE.1).contains(&p);
    if !(in_range(low) && in_range(high) && low <= high) {
        errors.push(format!(
            "{at}.pitch = ({low}, {high}): deve stare in [{}, {}] con min <= max",
            PITCH_RANGE.0, PITCH_RANGE.1
        ));
    }
    if cue.max_instances == 0 {
        errors.push(format!("{at}.max_instances = 0: deve essere >= 1"));
    }
    if let Coalesce::Boost { max_db } = cue.coalesce
        && !(max_db.is_finite() && max_db > 0.0 && max_db <= MAX_BOOST_DB)
    {
        errors.push(format!(
            "{at}.coalesce.max_db = {max_db}: deve stare in (0, {MAX_BOOST_DB}]"
        ));
    }
    if !(cue.length_s.is_finite() && cue.length_s > 0.0) {
        errors.push(format!("{at}.length_s = {}: deve essere > 0", cue.length_s));
    }
    if cue.files.is_empty() {
        errors.push(format!("{at}.files: serve almeno un file"));
    }
    for (i, file) in cue.files.iter().enumerate() {
        if !valid_file_name(file) {
            errors.push(format!(
                "{at}.files[{i}] = \"{file}\": serve un percorso relativo ad assets/ con \
                 minuscole, cifre, _ e / (es. \"audio/sfx/ui/tick.wav\")"
            ));
        } else if !file.ends_with(&format!(".{CUE_EXTENSION}")) {
            errors.push(format!(
                "{at}.files[{i}] = \"{file}\": i cue vogliono file .{CUE_EXTENSION}"
            ));
        }
    }
    errors
}

/// `shot.ember`, `enemy.die.small`: parole di minuscole, cifre e `_` separate da un punto.
fn valid_cue_name(name: &str) -> bool {
    name.split('.').all(|word| {
        !word.is_empty()
            && word
                .chars()
                .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
    })
}

/// Guadagno lineare in (0, 1].
fn unit_gain(value: f32) -> bool {
    value.is_finite() && value > 0.0 && value <= 1.0
}

/// `^[a-z0-9_/]+\.(wav|ogg)$`, relativo (niente `/` iniziale o finale, niente cartelle vuote). I
/// cue vogliono `.wav`, la musica `.ogg`.
fn valid_file_name(path: &str) -> bool {
    let Some((stem, extension)) = path.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "wav" | "ogg")
        && !stem.is_empty()
        && stem
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '/'))
        && stem.split('/').all(|part| !part.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{valid_cue_name, valid_file_name};

    #[test]
    fn cue_names_are_dotted_words() {
        for good in ["shot.ember", "enemy.die.small", "ui", "a_1.b2"] {
            assert!(valid_cue_name(good), "{good}");
        }
        for bad in [
            "",
            ".shot",
            "shot.",
            "shot..ember",
            "Shot",
            "shot ember",
            "shot-ember",
        ] {
            assert!(!valid_cue_name(bad), "{bad}");
        }
    }

    #[test]
    fn file_names_follow_the_asset_pattern() {
        for good in ["audio/sfx/ui/tick.wav", "audio/music/home.ogg", "a_1.wav"] {
            assert!(valid_file_name(good), "{good}");
        }
        for bad in [
            "",
            ".wav",
            "audio/sfx/Tick.wav",
            "audio/sfx/tick.mp3",
            "/audio/tick.wav",
            "audio//tick.wav",
            "audio/sfx/",
            "../tick.wav",
            "audio/sfx/tick.wav.wav",
            "audio\\sfx\\tick.wav",
            "audio/sfx/tick one.wav",
        ] {
            assert!(!valid_file_name(bad), "{bad}");
        }
    }
}
