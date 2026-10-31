//! Catalogo dei suoni (`assets/audio/catalog.ron`, progetto § 7): limiti delle voci, trim dei
//! bus e i cue con file, priorità, cooldown, istanze, coalescenza e cull. I valori sono del
//! proprietario degli asset audio (D-070), lo schema è qui. Incorporato nel binario come
//! `assets/ui/home_ambient.ron`: nessun caricamento, e un catalogo non valido nel gioco dà un
//! errore nel log e un catalogo vuoto (mai un panic); il test `the_real_audio_catalog_is_valid`
//! lo fa fallire prima.
//!
//! `deny_unknown_fields` come in `game-data/`: un refuso nel nome di un campo è un errore. Mappe
//! ordinate (`BTreeMap`, D-047): i cue si numerano in ordine di nome ([`CueId`]).

use std::collections::BTreeMap;

use bevy::log::error;
use bevy::prelude::Resource;
use serde::Deserialize;

use super::settings::Bus;

mod validate;

/// Versione dello schema del catalogo supportata da questo codice.
pub const CATALOG_VERSION: u32 = 1;

/// Testo del catalogo incorporato.
pub const CATALOG_TEXT: &str = include_str!("../../assets/audio/catalog.ron");

/// Variazione di altezza predefinita di un cue (progetto § 3.6), come `PlaybackSettings::speed`.
pub const DEFAULT_PITCH: (f32, f32) = (0.95, 1.05);
/// Variazione di guadagno predefinita di un cue, ± dB (progetto § 3.6).
pub const DEFAULT_GAIN_JITTER_DB: f32 = 1.5;
/// Durata massima stimata predefinita di un cue, in secondi: serve a ripulire le voci senza sink.
pub const DEFAULT_LENGTH_S: f32 = 2.0;

/// Il file `catalog.ron` così com'è scritto.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogFile {
    pub version: u32,
    pub voices: VoiceLimits,
    pub trims: Trims,
    #[serde(default)]
    pub cues: BTreeMap<String, CueData>,
    /// Ripieghi: un nome senza cue proprio (es. `"shot.tide"`) cerca qui sé stesso e poi i suoi
    /// prefissi fino al punto (`"shot"`), e suona il cue indicato.
    #[serde(default)]
    pub fallbacks: BTreeMap<String, String>,
}

/// Limiti delle voci effetto vive insieme (progetto § 3.5), per il desktop e per il web.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceLimits {
    /// Voci vive insieme al massimo.
    pub max: u32,
    pub max_web: u32,
    /// Posti tenuti liberi per la priorità `Critical`.
    pub reserved_critical: u32,
    /// Voci nuove per fotogramma al massimo; le altre del fotogramma si scartano.
    pub starts_per_frame: u32,
    pub starts_per_frame_web: u32,
    /// Tetto lineare della somma dei guadagni delle voci vive (protezione dalla saturazione).
    pub sfx_headroom: f32,
}

/// Guadagno di ogni bus nel mix, sotto i cursori del giocatore.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trims {
    pub music: f32,
    pub sfx: f32,
    pub ambience: f32,
    /// Suoni dell'interfaccia: seguono il cursore "Effetti" con un trim proprio.
    pub ui: f32,
}

/// Bus di un cue: gli effetti di gioco o l'interfaccia (lo stesso cursore "Effetti", trim
/// diversi). I cue sono one-shot in WAV (nessuna decodifica); musica, stinger e ambienti avranno
/// le loro sezioni del catalogo (AUDIO-3, AUDIO-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum CueBus {
    Sfx,
    Ui,
}

impl CueBus {
    /// Il cursore del giocatore che regola il cue: per tutti e due quello degli effetti.
    pub fn bus(self) -> Bus {
        Bus::Sfx
    }
}

/// Estensione dei file dei cue.
pub const CUE_EXTENSION: &str = "wav";

/// Priorità di un cue: a posti finiti una richiesta ruba la voce di priorità più bassa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Cosa fare quando un cue ha già `max_instances` voci vive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum OnLimit {
    /// La richiesta nuova si scarta.
    #[default]
    Drop,
    /// La voce più vecchia dello stesso cue si ferma e la nuova parte.
    Restart,
}

/// Richieste dello stesso cue nello stesso fotogramma: sempre una sola voce.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Coalesce {
    /// Una voce al guadagno del cue.
    #[default]
    Single,
    /// Una voce più forte di 1,5 dB per ogni raddoppio della quantità, fino a `max_db`.
    Boost { max_db: f32 },
}

/// Quando una richiesta con una posizione suona.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum Cull {
    #[default]
    Always,
    /// Solo dentro la vista più grande (640×360 attorno alla camera, `world::view`).
    View,
}

/// Un cue: un suono che il gioco chiede per nome, con le sue varianti e le sue regole.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueData {
    pub bus: CueBus,
    /// Varianti, relative ad `assets/` (es. `audio/sfx/ui/tick.wav`).
    pub files: Vec<String>,
    /// Guadagno lineare del cue, in (0, 1].
    pub gain: f32,
    /// Variazione casuale del guadagno, ± dB.
    #[serde(default = "default_gain_jitter_db")]
    pub gain_jitter_db: f32,
    /// Intervallo della velocità di riproduzione (altezza e durata), min e max.
    #[serde(default = "default_pitch")]
    pub pitch: (f32, f32),
    #[serde(default)]
    pub priority: Priority,
    /// Intervallo medio tra due partenze del cue, in millisecondi di tempo reale (cooldown "a
    /// credito", vedi `mixer`): mai meno di un fotogramma, mai meno di `cooldown_ms` meno un
    /// fotogramma.
    #[serde(default)]
    pub cooldown_ms: u32,
    /// Voci vive del cue insieme al massimo.
    #[serde(default = "one")]
    pub max_instances: u32,
    #[serde(default)]
    pub on_limit: OnLimit,
    #[serde(default)]
    pub coalesce: Coalesce,
    #[serde(default)]
    pub cull: Cull,
    /// Durata massima stimata in secondi (con il pitch più basso): una voce che non ha mai
    /// suonato (nessun sink: file non pronto, nessun dispositivo) si toglie dopo questa durata
    /// più mezzo secondo. Una voce che suona finisce solo quando la riproduzione lo dice.
    #[serde(default = "default_length_s")]
    pub length_s: f32,
}

fn default_gain_jitter_db() -> f32 {
    DEFAULT_GAIN_JITTER_DB
}

fn default_pitch() -> (f32, f32) {
    DEFAULT_PITCH
}

fn one() -> u32 {
    1
}

fn default_length_s() -> f32 {
    DEFAULT_LENGTH_S
}

/// Indice compatto di un cue: la sua posizione nel catalogo in ordine di nome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CueId(pub u16);

impl CueId {
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// Un cue compilato: nome e dati.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub name: String,
    pub data: CueData,
}

/// Catalogo validato e indicizzato.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct AudioCatalog {
    pub voices: VoiceLimits,
    pub trims: Trims,
    /// In ordine di nome: la posizione è il [`CueId`].
    cues: Vec<Cue>,
    /// Ripieghi già risolti in id.
    fallbacks: BTreeMap<String, CueId>,
}

impl AudioCatalog {
    /// Parsing e validazione; in caso di errori li restituisce tutti, ognuno con il campo.
    pub fn parse(text: &str) -> Result<Self, Vec<String>> {
        let file: CatalogFile =
            ron::from_str(text).map_err(|e| vec![format!("catalog.ron: {e}")])?;
        Self::compile(file)
    }

    /// Validazione e indici di un catalogo già letto.
    pub fn compile(file: CatalogFile) -> Result<Self, Vec<String>> {
        let errors = validate::validate(&file);
        if !errors.is_empty() {
            return Err(errors);
        }
        let cues: Vec<Cue> = file
            .cues
            .into_iter()
            .map(|(name, data)| Cue { name, data })
            .collect();
        let mut catalog = Self {
            voices: file.voices,
            trims: file.trims,
            cues,
            fallbacks: BTreeMap::new(),
        };
        for (from, to) in file.fallbacks {
            // la validazione garantisce che il cue esista
            if let Some(id) = catalog.exact(&to) {
                catalog.fallbacks.insert(from, id);
            }
        }
        Ok(catalog)
    }

    /// Il catalogo incorporato. Se non è valido (lo impedisce un test): errore nel log e
    /// catalogo vuoto, il gioco resta muto ma parte.
    pub fn embedded() -> Self {
        match Self::parse(CATALOG_TEXT) {
            Ok(catalog) => catalog,
            Err(errors) => {
                error!(
                    "assets/audio/catalog.ron non valido, audio spento:\n{}",
                    errors.join("\n")
                );
                Self::empty()
            }
        }
    }

    /// Nessun cue: nessuna richiesta può partire, quindi i limiti non contano (valori minimi).
    pub fn empty() -> Self {
        Self {
            voices: VoiceLimits {
                max: 1,
                max_web: 1,
                reserved_critical: 0,
                starts_per_frame: 1,
                starts_per_frame_web: 1,
                sfx_headroom: 1.0,
            },
            trims: Trims {
                music: 1.0,
                sfx: 1.0,
                ambience: 1.0,
                ui: 1.0,
            },
            cues: Vec::new(),
            fallbacks: BTreeMap::new(),
        }
    }

    /// Id del cue `name`, oppure del suo ripiego: prima `name` in `fallbacks`, poi i suoi
    /// prefissi fino a ogni punto (`"shot.tide"` → `"shot"`).
    pub fn id(&self, name: &str) -> Option<CueId> {
        if let Some(id) = self.exact(name) {
            return Some(id);
        }
        let mut candidate = name;
        loop {
            if let Some(&id) = self.fallbacks.get(candidate) {
                return Some(id);
            }
            candidate = &candidate[..candidate.rfind('.')?];
        }
    }

    /// Id del cue che si chiama esattamente `name`, senza ripieghi.
    pub fn exact(&self, name: &str) -> Option<CueId> {
        let index = self
            .cues
            .binary_search_by(|cue| cue.name.as_str().cmp(name))
            .ok()?;
        // la validazione limita i cue a u16::MAX + 1
        u16::try_from(index).ok().map(CueId)
    }

    pub fn cue(&self, id: CueId) -> Option<&Cue> {
        self.cues.get(id.index())
    }

    /// Tutti i cue, nell'ordine dei loro id.
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    /// Trim del bus di un cue.
    pub fn trim(&self, bus: CueBus) -> f32 {
        match bus {
            CueBus::Sfx => self.trims.sfx,
            CueBus::Ui => self.trims.ui,
        }
    }
}
