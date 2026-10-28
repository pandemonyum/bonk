//! Orde della run (`stages.ron: run.hordes`): a istanti fissi una massa di nemici tutta insieme.

use serde::Deserialize;

/// Le orde di una run. Senza orde (`[]`) la run ha solo le ondate continue.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HordePlan {
    /// Secondi di preavviso: l'annuncio compare prima che nascano i nemici.
    pub warning_s: f32,
    /// Secondi in cui nascono tutti i nemici di un'orda, distribuiti nel tempo.
    pub duration_s: f32,
    /// Nemici vivi al massimo durante un'orda, contando anche quelli delle ondate: oltre,
    /// l'orda aspetta che ne muoia qualcuno.
    pub max_alive: u32,
    pub list: Vec<HordeData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HordeData {
    /// Secondo della run a cui iniziano a nascere i nemici (l'annuncio viene `warning_s` prima).
    pub at_s: f32,
    pub groups: Vec<HordeGroup>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HordeGroup {
    /// Id in `enemies.ron`.
    pub enemy: String,
    pub count: u32,
}

impl HordeData {
    pub fn total(&self) -> u32 {
        self.groups.iter().map(|g| g.count).sum()
    }
}
