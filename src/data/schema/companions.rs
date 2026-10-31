//! Schema di `companions.ron`: i compagni mostriciattolo, con il loro attacco, il foglio animato,
//! l'albero dei talenti, il testo della selezione e la regola che li rende compagni iniziali.

use serde::Deserialize;

use super::{SheetFrame, TalentBranch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Element {
    Fire,
    Water,
    Nature,
    Electric,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowData {
    pub distance: f32,
    pub max_speed: f32,
    pub gain: f32,
}

/// Quando un compagno si può scegliere come compagno iniziale (D-066). Le condizioni future
/// (missioni, obiettivi) saranno varianti nuove, valutate solo in `profile::unlocks`: la UI legge
/// lo stato che ne esce, mai questa regola.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Unlock {
    /// Disponibile anche su un profilo nuovo.
    Start,
    /// Non ancora ottenibile: disponibile solo se il profilo lo ha tra gli `unlocked`.
    Locked,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionData {
    pub id: String,
    pub display_name: String,
    /// Due o tre frasi per la schermata di selezione.
    pub description: String,
    pub element: Element,
    /// Compagno iniziale fin dall'inizio o da sbloccare.
    pub unlock: Unlock,
    pub sprite: String,
    pub attack: String,
    pub follow: FollowData,
    /// Fotogramma del foglio animato in `sprite` (4 colonne; righe: fermo, cammino, attacco).
    /// Assente: immagine fissa.
    #[serde(default)]
    pub frame: Option<SheetFrame>,
    /// Due rami di talenti, sbloccati con i livelli del compagno tra una run e l'altra.
    #[serde(default)]
    pub talents: Vec<TalentBranch>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionsFile {
    pub version: u32,
    pub companions: Vec<CompanionData>,
}
