//! Sezione `music` del catalogo (progetto § 3.7 e § 7): le tracce in loop del bus Music (file Ogg
//! Vorbis relativi ad `assets/`, guadagno), la traccia della partita per stage con un ripiego e le
//! durate delle dissolvenze, compresa quella del fuoco della finestra (§ 3.11), che vale per tutto
//! l'audio. Facoltativa: un catalogo senza `music` non ha tracce, e ogni momento del gioco è
//! silenzio con le dissolvenze predefinite.
//!
//! Le tracce si numerano in ordine di nome ([`TrackId`], come i cue). La home suona la traccia
//! [`HOME_TRACK`] se c'è; orda, boss, risultati e stinger arriveranno con le loro tracce
//! (AUDIO-3b).

use std::collections::BTreeMap;

use serde::Deserialize;

/// Nome della traccia del menu, in ogni sua schermata.
pub const HOME_TRACK: &str = "home";
/// Estensione dei file delle tracce: Ogg Vorbis, che pesa meno da scaricare (D-069).
pub const MUSIC_EXTENSION: &str = "ogg";
/// Dissolvenza predefinita verso la partita e verso il menu, in secondi (§ 3.7).
pub const DEFAULT_FADE_S: f32 = 1.5;
/// Dissolvenza predefinita del fuoco della finestra, in secondi (§ 3.11).
pub const DEFAULT_FOCUS_FADE_S: f32 = 0.25;

/// La sezione `music` così com'è scritta.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MusicFile {
    #[serde(default)]
    pub tracks: BTreeMap<String, TrackData>,
    /// Traccia della partita per id di stage (`game-data/stages.ron`).
    #[serde(default)]
    pub run_by_stage: BTreeMap<String, String>,
    /// Traccia della partita per gli stage che non sono in `run_by_stage`.
    #[serde(default)]
    pub run_default: Option<String>,
    #[serde(default)]
    pub fades: MusicFades,
}

/// Una traccia in loop.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackData {
    /// Relativo ad `assets/` (es. `audio/music/home.ogg`).
    pub file: String,
    /// Guadagno lineare della traccia, in (0, 1].
    pub gain: f32,
}

/// Durate delle dissolvenze, in secondi di tempo reale (≥ 0; 0 è un cambio secco).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MusicFades {
    /// Verso la partita e i suoi momenti, anche quando il momento è silenzio.
    #[serde(default = "default_fade_s")]
    pub to_run: f32,
    /// Verso il menu.
    #[serde(default = "default_fade_s")]
    pub to_home: f32,
    /// Finestra non attiva: tutto l'audio scende a zero e risale in questo tempo (`focus`).
    #[serde(default = "default_focus_fade_s")]
    pub focus: f32,
}

impl Default for MusicFades {
    fn default() -> Self {
        Self {
            to_run: DEFAULT_FADE_S,
            to_home: DEFAULT_FADE_S,
            focus: DEFAULT_FOCUS_FADE_S,
        }
    }
}

fn default_fade_s() -> f32 {
    DEFAULT_FADE_S
}

fn default_focus_fade_s() -> f32 {
    DEFAULT_FOCUS_FADE_S
}

/// Indice compatto di una traccia: la sua posizione in ordine di nome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackId(pub u16);

impl TrackId {
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// Una traccia compilata: nome e dati.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub name: String,
    pub file: String,
    pub gain: f32,
}

/// La sezione `music` validata, con i riferimenti già risolti in id.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MusicCatalog {
    /// In ordine di nome: la posizione è il [`TrackId`].
    tracks: Vec<Track>,
    run_by_stage: BTreeMap<String, TrackId>,
    run_default: Option<TrackId>,
    pub fades: MusicFades,
}

impl MusicCatalog {
    /// Indici di una sezione già validata (i riferimenti a tracce che non esistono si perdono, ma
    /// la validazione li ha già rifiutati).
    pub(super) fn compile(file: MusicFile) -> Self {
        let tracks: Vec<Track> = file
            .tracks
            .into_iter()
            .map(|(name, data)| Track {
                name,
                file: data.file,
                gain: data.gain,
            })
            .collect();
        let mut music = Self {
            tracks,
            run_by_stage: BTreeMap::new(),
            run_default: None,
            fades: file.fades,
        };
        for (stage, track) in file.run_by_stage {
            if let Some(id) = music.id(&track) {
                music.run_by_stage.insert(stage, id);
            }
        }
        music.run_default = file.run_default.and_then(|track| music.id(&track));
        music
    }

    /// Id della traccia che si chiama `name`.
    pub fn id(&self, name: &str) -> Option<TrackId> {
        let index = self
            .tracks
            .binary_search_by(|track| track.name.as_str().cmp(name))
            .ok()?;
        // la validazione limita le tracce a u16::MAX + 1
        u16::try_from(index).ok().map(TrackId)
    }

    pub fn track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(id.index())
    }

    /// Tutte le tracce, nell'ordine dei loro id.
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Nome di una traccia per il log (`?` se l'id non c'è).
    pub fn name(&self, id: TrackId) -> &str {
        self.track(id).map_or("?", |track| track.name.as_str())
    }

    /// La traccia del menu, se il catalogo ce l'ha.
    pub fn home(&self) -> Option<TrackId> {
        self.id(HOME_TRACK)
    }

    /// La traccia della partita nello stage `stage`: quella dello stage, altrimenti il ripiego.
    pub fn run(&self, stage: Option<&str>) -> Option<TrackId> {
        stage
            .and_then(|stage| self.run_by_stage.get(stage).copied())
            .or(self.run_default)
    }

    /// Gli stage con una traccia propria, in ordine.
    pub fn run_stages(&self) -> impl Iterator<Item = &str> {
        self.run_by_stage.keys().map(String::as_str)
    }
}
