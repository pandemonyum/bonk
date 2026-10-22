//! Generazione pura (seed → tile): nessuna dipendenza da Bevy render, testabile headless.

use bevy::math::{UVec2, Vec2};

use super::tiles::*;
use super::{ARENA_HALF_EXTENT, ARENA_TILES, BORDER_TILES, CHUNK_TILES, TILE_SIZE};
use crate::rng::{hash2, unit_f32};

/// Sale per separare il rumore delle radure dal rumore delle varianti d'erba.
const CLEARING_SALT: u64 = 0xC1EA_0001;
/// Lato in tile delle celle del rumore delle radure (forme grandi e morbide).
const CLEARING_CELL: i32 = 10;
/// Soglia oltre cui il terreno diventa radura di terra. Con 0.74 le radure coprivano il 15%
/// dell'arena (misurato dal test sotto); 0.80 le riduce a macchie.
const CLEARING_THRESHOLD: f32 = 0.80;

/// Tile alla coordinata `(x, y)` dell'arena, con `(0, 0)` in basso a sinistra.
pub fn tile_at(seed: u64, x: u32, y: u32) -> u16 {
    let last = ARENA_TILES - BORDER_TILES;
    if x < BORDER_TILES || y < BORDER_TILES || x >= last || y >= last {
        return HEDGE;
    }
    let (ix, iy) = (x as i32, y as i32);
    if value_noise(seed ^ CLEARING_SALT, ix, iy, CLEARING_CELL) > CLEARING_THRESHOLD {
        return DIRT;
    }
    match unit_f32(hash2(seed, ix, iy)) {
        r if r < 0.42 => GRASS_A,
        r if r < 0.84 => GRASS_B,
        r if r < 0.93 => GRASS_TUFT,
        r if r < 0.97 => FLOWERS_WHITE,
        _ => FLOWERS_PINK,
    }
}

/// Tile di un chunk in ordine y-up, riga per riga: il formato di `TilemapChunkTileData`.
pub fn chunk_tiles(seed: u64, chunk: UVec2) -> Vec<u16> {
    let origin = chunk * CHUNK_TILES;
    (0..CHUNK_TILES)
        .flat_map(|y| (0..CHUNK_TILES).map(move |x| (x, y)))
        .map(|(x, y)| tile_at(seed, origin.x + x, origin.y + y))
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
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_map_different_seed_different_map() {
        let a = chunk_tiles(1, UVec2::new(1, 2));
        assert_eq!(a, chunk_tiles(1, UVec2::new(1, 2)));
        assert_ne!(a, chunk_tiles(2, UVec2::new(1, 2)));
        assert_eq!(a.len(), (CHUNK_TILES * CHUNK_TILES) as usize);
    }

    #[test]
    fn arena_is_closed_by_hedge_and_indices_are_valid() {
        for i in 0..ARENA_TILES {
            for (x, y) in [(i, 0), (i, ARENA_TILES - 1), (0, i), (ARENA_TILES - 1, i)] {
                assert_eq!(tile_at(7, x, y), HEDGE);
            }
        }
        for y in (0..ARENA_TILES).step_by(3) {
            for x in (0..ARENA_TILES).step_by(3) {
                assert!(u32::from(tile_at(7, x, y)) < TILE_COUNT);
            }
        }
    }

    #[test]
    fn interior_has_no_hedge_and_some_variety() {
        let mut counts = [0u32; TILE_COUNT as usize];
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                counts[tile_at(3, x, y) as usize] += 1;
            }
        }
        assert_eq!(counts[HEDGE as usize], 0);
        for tile in [
            GRASS_A,
            GRASS_B,
            GRASS_TUFT,
            FLOWERS_WHITE,
            FLOWERS_PINK,
            DIRT,
        ] {
            assert!(counts[tile as usize] > 0, "manca il tile {tile}");
        }
        // L'erba domina: le radure sono macchie, non metà mappa.
        let grass = counts[GRASS_A as usize] + counts[GRASS_B as usize];
        assert!(
            counts[DIRT as usize] * 5 < grass,
            "distribuzione {counts:?}"
        );
    }

    #[test]
    fn chunks_tile_the_arena_symmetrically() {
        use crate::world::ARENA_CHUNKS;
        assert_eq!(chunk_center(UVec2::ZERO), Vec2::splat(-1536.0));
        assert_eq!(
            chunk_center(UVec2::splat(ARENA_CHUNKS - 1)),
            Vec2::splat(1536.0)
        );
    }
}
