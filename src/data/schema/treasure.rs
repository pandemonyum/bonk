//! Monete e forzieri: le monete cadono dai nemici (`enemies.ron: coins`) e si raccolgono come le
//! gemme (`progression.ron: coin`); i forzieri stanno lontani sulla mappa (`stages.ron: chests`)
//! e con abbastanza monete si aprono: si sceglie una reliquia (`upgrades.ron`, rarità `Relic`).

use serde::Deserialize;

/// Monete lasciate da un nemico sconfitto: `count` monete con probabilità `chance`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CoinDrop {
    pub chance: f32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoinData {
    pub sprite: String,
    /// Monete a terra al massimo: oltre, il valore di una nuova va alla più vicina.
    pub max_on_field: u32,
    /// Distanza a cui cadono dalla gemma (px), sparse attorno al punto della morte.
    pub scatter: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChestPlan {
    /// Foglio del forziere: riga 0 chiuso (4 fotogrammi), riga 1 apertura (4 fotogrammi).
    pub sprite: String,
    pub frame_w: u32,
    pub frame_h: u32,
    /// Forzieri sulla mappa nello stesso momento.
    pub on_map: u32,
    /// Fascia (min, max) di distanza dal player in cui nasce un forziere, fuori dalla vista.
    pub distance: (f32, f32),
    /// Fascia di riserva se nella prima non c'è posto.
    pub fallback_distance: (f32, f32),
    /// Distanza minima dal bordo dell'arena e dagli altri forzieri.
    pub min_from_edge: f32,
    pub min_apart: f32,
    /// Oltre questa distanza dal player il forziere rinasce nella fascia.
    pub respawn_beyond: f32,
    /// Distanza dei piedi del player dal forziere per aprirlo.
    pub open_radius: f32,
    /// Prezzo: `base + per_open × forzieri già aperti`.
    pub price_base: u32,
    pub price_per_open: u32,
}

impl ChestPlan {
    /// Prezzo del prossimo forziere dopo `opened` aperti.
    pub fn price(&self, opened: u32) -> u32 {
        self.price_base
            .saturating_add(self.price_per_open.saturating_mul(opened))
    }
}
