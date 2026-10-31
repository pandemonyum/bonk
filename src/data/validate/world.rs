//! Regole dei biomi e degli stage che li usano (`stages.ron`, D-068). La struttura dei biomi
//! (nomi dei tile, soglie, impronte) la controlla `world::biome::Biome::compile`, l'unico che
//! la interpreta; qui si aggiungono riferimenti e percorsi degli asset.

use super::{GameConfig, Report};
use crate::world::biome::Biome;

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    r.unique_ids("stages.ron: stages", config.stages.iter().map(|s| &s.id));
    r.unique_ids("stages.ron: biomes", config.biomes.iter().map(|b| &b.id));
    let stage = &config.stage;
    r.reference(
        &format!("stages.ron: {}.biome", stage.id),
        &stage.biome,
        config.find_biome(&stage.biome).is_some(),
        "stages.ron: biomes",
    );
    for biome in &config.biomes {
        let at = |field: &str| format!("stages.ron: biomes.{}.{field}", biome.id);
        r.sprite(&at("terrain.tileset"), &biome.terrain.tileset);
        if let Some(decorations) = &biome.decorations {
            r.sprite(&at("decorations.tileset"), &decorations.tileset);
        }
        for kind in &biome.obstacles {
            r.sprite(&at(&format!("obstacles.{}.sprite", kind.id)), &kind.sprite);
        }
        if let Err(errors) = Biome::compile(biome) {
            r.0.extend(errors);
        }
    }
}

#[cfg(test)]
mod tests;
