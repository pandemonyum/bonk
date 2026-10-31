//! Generazione pura (seed + bioma → tile): nessuna dipendenza da Bevy render, testabile headless.
//! Le stesse funzioni servono ogni bioma (D-068); sali e rumore sono quelli della prateria, così
//! la prateria resta identica a prima.

use bevy::math::{UVec2, Vec2};

use super::biome::{Biome, EDGE_E, EDGE_N, EDGE_S, EDGE_W, Floor, Terrain};
use super::{ARENA_HALF_EXTENT, ARENA_TILES, BORDER_TILES, CHUNK_TILES, TILE_SIZE};
use crate::rng::{hash2, unit_f32};

/// Sale del rumore delle macchie (le radure della prateria).
const PATCH_SALT: u64 = 0xC1EA_0001;
/// Sale del tiro delle varianti solo grafiche del terreno.
const VARIANT_SALT: u64 = 0xDEC0_0001;
/// Sale del rumore dei gruppi di tile (i fiori della prateria).
const CLUSTER_SALT: u64 = 0xF10E_0001;
/// Sale del livello delle decorazioni; ogni regola ne sposta il valore di `RULE_STRIDE`.
const DECOR_SALT: u64 = 0xDEC0_A7E5;
const RULE_STRIDE: u64 = 0x9E37_79B9_7F4A_7C15;

/// Terreno logico in `(x, y)`, con `(0, 0)` in basso a sinistra: muro sui bordi, macchie dove
/// il rumore supera la soglia, base altrove.
pub fn floor_at(terrain: &Terrain, seed: u64, x: u32, y: u32) -> Floor {
    let last = ARENA_TILES - BORDER_TILES;
    if x < BORDER_TILES || y < BORDER_TILES || x >= last || y >= last {
        return Floor::Border;
    }
    match &terrain.patch {
        Some(patch)
            if value_noise(seed ^ PATCH_SALT, x as i32, y as i32, patch.cell) > patch.threshold =>
        {
            Floor::Patch
        }
        _ => Floor::Ground,
    }
}

/// Tile logico in `(x, y)`: il materiale (muro, macchia o un tile di base), senza varianti. Il
/// gameplay usa `floor_at`; questo serve ai test del terreno.
#[cfg(test)]
pub fn tile_at(terrain: &Terrain, seed: u64, x: u32, y: u32) -> u16 {
    match (floor_at(terrain, seed, x, y), &terrain.patch) {
        (Floor::Border, _) => terrain.border.tile,
        (Floor::Patch, Some(patch)) => patch.material.tile,
        _ => terrain.ground_tile(unit_f32(hash2(seed, x as i32, y as i32))),
    }
}

/// Tile da disegnare in `(x, y)`: il materiale di `tile_at` con le varianti solo grafiche, cioè
/// bordi morbidi dove macchia o muro confinano con altro, varianti dei tile pieni e gruppi a
/// macchie. Ostacoli e gameplay leggono `floor_at`: queste varianti non cambiano la mappa logica.
pub fn visual_tile_at(terrain: &Terrain, seed: u64, x: u32, y: u32) -> u16 {
    let (ix, iy) = (x as i32, y as i32);
    let roll = unit_f32(hash2(seed ^ VARIANT_SALT, ix, iy));
    let floor = floor_at(terrain, seed, x, y);
    let material = match (floor, &terrain.patch) {
        (Floor::Border, _) => terrain.border,
        (Floor::Patch, Some(patch)) => patch.material,
        _ => return ground_variant(terrain, seed, x, y, roll),
    };
    match edge_mask(terrain, seed, x, y, floor) {
        0 => terrain.variant(material.tile, roll),
        mask => material.edge(mask),
    }
}

/// Bit `EDGE_*` dei lati il cui vicino non è `floor`. Fuori dall'arena conta come lo stesso
/// terreno: il muro esterno non ha bordo.
fn edge_mask(terrain: &Terrain, seed: u64, x: u32, y: u32, floor: Floor) -> u16 {
    let sides = [
        (0, 1, EDGE_N),
        (1, 0, EDGE_E),
        (0, -1, EDGE_S),
        (-1, 0, EDGE_W),
    ];
    sides.into_iter().fold(0, |mask, (dx, dy, bit)| {
        let neighbor = x
            .checked_add_signed(dx)
            .zip(y.checked_add_signed(dy))
            .filter(|&(nx, ny)| nx < ARENA_TILES && ny < ARENA_TILES);
        match neighbor {
            Some((nx, ny)) if floor_at(terrain, seed, nx, ny) != floor => mask | bit,
            _ => mask,
        }
    })
}

fn ground_variant(terrain: &Terrain, seed: u64, x: u32, y: u32, roll: f32) -> u16 {
    let (ix, iy) = (x as i32, y as i32);
    if let Some(clusters) = &terrain.clusters
        && value_noise(seed ^ CLUSTER_SALT, ix, iy, clusters.cell) > clusters.threshold
        && roll < clusters.share
    {
        // in parti uguali
        let count = clusters.tiles.len();
        let pick = (roll / clusters.share * count as f32) as usize;
        return clusters.tiles[pick.min(count - 1)];
    }
    let base = terrain.ground_tile(unit_f32(hash2(seed, ix, iy)));
    terrain.variant(base, roll)
}

/// Decorazione calpestabile in `(x, y)` (indice nel tileset delle decorazioni), se c'è: la
/// prima regola del bioma che decora il tile. Mai sul muro; gli ostacoli le coprono nel render.
pub fn decor_at(biome: &Biome, seed: u64, x: u32, y: u32) -> Option<u16> {
    let decorations = biome.decorations.as_ref()?;
    let floor = floor_at(&biome.terrain, seed, x, y);
    let (ix, iy) = (x as i32, y as i32);
    decorations
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| floor.accepts(rule.on))
        .find_map(|(i, rule)| {
            let salt = seed ^ DECOR_SALT.wrapping_add(RULE_STRIDE.wrapping_mul(i as u64));
            if let Some((cell, threshold)) = rule.cluster
                && value_noise(salt ^ CLUSTER_SALT, ix, iy, cell) <= threshold
            {
                return None;
            }
            let roll = unit_f32(hash2(salt, ix, iy));
            (roll < rule.density).then(|| {
                let count = rule.tiles.len();
                let pick = (roll / rule.density * count as f32) as usize;
                rule.tiles[pick.min(count - 1)]
            })
        })
}

/// Tile di un chunk in ordine y-up, riga per riga.
pub fn chunk_tiles(terrain: &Terrain, seed: u64, chunk: UVec2) -> Vec<u16> {
    let origin = chunk * CHUNK_TILES;
    (0..CHUNK_TILES)
        .flat_map(|y| (0..CHUNK_TILES).map(move |x| (x, y)))
        .map(|(x, y)| visual_tile_at(terrain, seed, origin.x + x, origin.y + y))
        .collect()
}

/// Centro di un chunk in pixel di mondo (l'arena è centrata nell'origine).
pub fn chunk_center(chunk: UVec2) -> Vec2 {
    let chunk_px = (CHUNK_TILES * TILE_SIZE) as f32;
    chunk.as_vec2() * chunk_px + Vec2::splat(chunk_px / 2.0 - ARENA_HALF_EXTENT)
}

/// Value noise bilineare con smoothstep su una griglia di lato `cell`.
fn value_noise(seed: u64, x: i32, y: i32, cell: i32) -> f32 {
    let (cx, cy) = (x.div_euclid(cell), y.div_euclid(cell));
    let fx = smoothstep(x.rem_euclid(cell) as f32 / cell as f32);
    let fy = smoothstep(y.rem_euclid(cell) as f32 / cell as f32);
    let corner = |dx: i32, dy: i32| unit_f32(hash2(seed, cx + dx, cy + dy));
    let bottom = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx;
    let top = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx;
    bottom + (top - bottom) * fy
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests;
