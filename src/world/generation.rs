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
/// Sali delle varianti solo grafiche dell'erba (`visual_tile_at`).
const DECOR_SALT: u64 = 0xDEC0_0001;
const FLOWER_SALT: u64 = 0xF10E_0001;
/// Lato in tile delle macchie di fiori, soglia del rumore oltre cui l'erba può fiorire e quota
/// di tile fioriti dentro una macchia. Con 0,72 e 0,4 le macchie coprivano mezzo schermo
/// (screenshot del gioco, seed 20): ora sono ciuffi sparsi.
const FLOWER_CELL: i32 = 6;
const FLOWER_THRESHOLD: f32 = 0.76;
const FLOWER_SHARE: f32 = 0.3;

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

/// Tile da disegnare in `(x, y)`: il materiale di `tile_at` con le varianti solo grafiche, cioè
/// bordi morbidi dove terra o siepe confinano con l'erba, erba variata e fiori a macchie.
/// Ostacoli e gameplay leggono `tile_at`: queste varianti non cambiano la mappa logica.
pub fn visual_tile_at(seed: u64, x: u32, y: u32) -> u16 {
    match tile_at(seed, x, y) {
        DIRT => edge_variant(seed, x, y, DIRT, DIRT_EDGES),
        HEDGE => edge_variant(seed, x, y, HEDGE, HEDGE_EDGES),
        grass => grass_variant(seed, x, y, grass),
    }
}

/// Bit `EDGE_*` dei lati il cui vicino non è `material`. Fuori dall'arena conta come lo stesso
/// materiale: la siepe esterna non ha bordo.
fn edge_mask(seed: u64, x: u32, y: u32, material: u16) -> u16 {
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
            Some((nx, ny)) if tile_at(seed, nx, ny) != material => mask | bit,
            _ => mask,
        }
    })
}

fn edge_variant(seed: u64, x: u32, y: u32, material: u16, first_edge: u16) -> u16 {
    match edge_mask(seed, x, y, material) {
        0 => material,
        mask => first_edge + mask - 1,
    }
}

fn grass_variant(seed: u64, x: u32, y: u32, grass: u16) -> u16 {
    let (ix, iy) = (x as i32, y as i32);
    let roll = unit_f32(hash2(seed ^ DECOR_SALT, ix, iy));
    if value_noise(seed ^ FLOWER_SALT, ix, iy, FLOWER_CELL) > FLOWER_THRESHOLD
        && roll < FLOWER_SHARE
    {
        // tre colori in parti uguali
        let color = (roll / FLOWER_SHARE * 3.0) as usize;
        return [FLOWERS_WHITE, FLOWERS_PINK, FLOWERS_YELLOW][color.min(2)];
    }
    match grass {
        GRASS_A if roll < 0.30 => GRASS_C,
        GRASS_A if roll < 0.42 => GRASS_CLOVER,
        GRASS_A if roll < 0.50 => GRASS_PEBBLES,
        GRASS_B if roll < 0.14 => GRASS_TALL,
        FLOWERS_WHITE | FLOWERS_PINK if roll < 0.3 => FLOWERS_YELLOW,
        _ => grass,
    }
}

/// Tile di un chunk in ordine y-up, riga per riga: il formato di `TilemapChunkTileData`.
pub fn chunk_tiles(seed: u64, chunk: UVec2) -> Vec<u16> {
    let origin = chunk * CHUNK_TILES;
    (0..CHUNK_TILES)
        .flat_map(|y| (0..CHUNK_TILES).map(move |x| (x, y)))
        .map(|(x, y)| visual_tile_at(seed, origin.x + x, origin.y + y))
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

    /// Materiale logico di un tile grafico: le varianti non devono mai cambiarlo.
    fn material_of(visual: u16) -> u16 {
        match visual {
            DIRT => DIRT,
            HEDGE => HEDGE,
            v if (DIRT_EDGES..DIRT_EDGES + 15).contains(&v) => DIRT,
            v if (HEDGE_EDGES..HEDGE_EDGES + 15).contains(&v) => HEDGE,
            _ => GRASS_A,
        }
    }

    #[test]
    fn visual_tiles_keep_the_logic_material_and_valid_indices() {
        for y in 0..ARENA_TILES {
            for x in (0..ARENA_TILES).step_by(3) {
                let visual = visual_tile_at(4, x, y);
                assert!(u32::from(visual) < TILE_COUNT, "indice {visual} in {x},{y}");
                let logic = match tile_at(4, x, y) {
                    t @ (DIRT | HEDGE) => t,
                    _ => GRASS_A,
                };
                assert_eq!(material_of(visual), logic, "tile {x},{y}");
            }
        }
    }

    /// Ogni tile di terra mostra l'erba proprio sui lati dove il vicino non è terra (nord = y+1,
    /// la riga in alto dell'immagine del tile).
    #[test]
    fn dirt_edges_face_the_grass_around_them() {
        let mut edges = 0;
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                if tile_at(6, x, y) != DIRT {
                    continue;
                }
                let grass = |nx: u32, ny: u32| tile_at(6, nx, ny) != DIRT;
                let expected = [
                    (grass(x, y + 1), EDGE_N),
                    (grass(x + 1, y), EDGE_E),
                    (grass(x, y - 1), EDGE_S),
                    (grass(x - 1, y), EDGE_W),
                ]
                .into_iter()
                .filter(|(open, _)| *open)
                .fold(0, |mask, (_, bit)| mask | bit);
                let visual = visual_tile_at(6, x, y);
                if expected == 0 {
                    assert_eq!(visual, DIRT);
                } else {
                    assert_eq!(visual, DIRT_EDGES + expected - 1, "tile {x},{y}");
                    edges += 1;
                }
            }
        }
        assert!(edges > 0, "nessun bordo: test vacuo");
    }

    /// La siepe ha il bordo solo verso l'arena: la fila interna a nord guarda a sud, quella a
    /// ovest guarda a est; la siepe esterna resta piena.
    #[test]
    fn hedge_edges_face_the_arena_only() {
        let (inner_low, inner_high) = (BORDER_TILES - 1, ARENA_TILES - BORDER_TILES);
        let mid = ARENA_TILES / 2;
        let is_hedge = |t: u16| material_of(t) == HEDGE;
        assert_eq!(visual_tile_at(1, mid, inner_high), HEDGE_EDGES + EDGE_S - 1);
        assert_eq!(visual_tile_at(1, inner_low, mid), HEDGE_EDGES + EDGE_E - 1);
        assert_eq!(visual_tile_at(1, mid, inner_low), HEDGE_EDGES + EDGE_N - 1);
        assert_eq!(visual_tile_at(1, inner_high, mid), HEDGE_EDGES + EDGE_W - 1);
        for (x, y) in [(mid, 0), (0, mid), (mid, ARENA_TILES - 1), (0, 0)] {
            assert_eq!(visual_tile_at(1, x, y), HEDGE);
        }
        assert!(is_hedge(visual_tile_at(1, inner_low, inner_low)));
    }

    #[test]
    fn decorative_variants_appear_on_the_grass() {
        let mut seen = [false; TILE_COUNT as usize];
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                seen[visual_tile_at(2, x, y) as usize] = true;
            }
        }
        for tile in [
            GRASS_C,
            GRASS_CLOVER,
            FLOWERS_YELLOW,
            GRASS_PEBBLES,
            GRASS_TALL,
        ] {
            assert!(seen[tile as usize], "manca la variante {tile}");
        }
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
