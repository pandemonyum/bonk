//! Ostacoli dell'arena (rocce, alberi): generazione pura dal seed e griglia spaziale statica.
//!
//! Garanzia di attraversabilità: due ostacoli non stanno mai in tile adiacenti (anche in
//! diagonale), quindi i centri distano almeno 2 tile (32 px) e tra due impronte resta un
//! varco di almeno 32 − 2·7 = 18 px, più largo del corpo più grande (12 px). Niente vicoli
//! ciechi, niente percorsi da calcolare.

use bevy::prelude::*;

use super::generation::tile_at;
use super::tiles::DIRT;
use super::{ARENA_HALF_EXTENT, ARENA_TILES, BORDER_TILES, TILE_SIZE};
use crate::rng::{hash2, unit_f32};

const OBSTACLE_SALT: u64 = 0x0B57_AC1E;
const KIND_SALT: u64 = 0x7EE5_0001;
/// Probabilità che un tile candidato ospiti un ostacolo (prima dello scarto per vicinanza).
pub const OBSTACLE_DENSITY: f32 = 0.012;
/// Raggio della zona libera attorno al punto di partenza del player (8 tile).
pub const SAFE_RADIUS: f32 = 128.0;
/// Lato delle celle della griglia spaziale.
const CELL_SIZE: f32 = 32.0;
const GRID_CELLS: usize = (ARENA_TILES * TILE_SIZE) as usize / CELL_SIZE as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObstacleKind {
    Rock,
    Tree,
}

impl ObstacleKind {
    /// Impronta a terra: (raggio, altezza del centro sopra la base dello sprite).
    pub const fn footprint(self) -> (f32, f32) {
        match self {
            Self::Rock => (7.0, 3.0),
            Self::Tree => (4.0, 2.0),
        }
    }
}

/// Raggio dell'impronta più grande, per le query sulla griglia.
pub const MAX_FOOTPRINT_RADIUS: f32 = 7.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleSpec {
    pub kind: ObstacleKind,
    /// Tile dell'arena (origine in basso a sinistra).
    pub tile: UVec2,
    /// Base dello sprite in pixel di mondo (ancora in basso al centro).
    pub base: Vec2,
}

impl ObstacleSpec {
    pub fn footprint_center(&self) -> Vec2 {
        self.base + Vec2::Y * self.kind.footprint().1
    }

    pub fn footprint_radius(&self) -> f32 {
        self.kind.footprint().0
    }
}

/// Centro in pixel di mondo del tile `(x, y)`.
pub fn tile_center(tile: UVec2) -> Vec2 {
    tile.as_vec2() * TILE_SIZE as f32 + Vec2::splat(TILE_SIZE as f32 / 2.0 - ARENA_HALF_EXTENT)
}

/// Ostacoli dell'arena per `seed`, in ordine di scansione (y poi x): deterministico.
pub fn generate_obstacles(seed: u64) -> Vec<ObstacleSpec> {
    let size = ARENA_TILES as usize;
    let mut occupied = vec![false; size * size];
    let mut obstacles = Vec::new();
    // Un tile di margine dalla siepe: nessun ostacolo a ridosso del bordo.
    let (first, last) = (BORDER_TILES + 1, ARENA_TILES - BORDER_TILES - 1);
    for y in first..last {
        for x in first..last {
            let (ix, iy) = (x as i32, y as i32);
            if unit_f32(hash2(seed ^ OBSTACLE_SALT, ix, iy)) >= OBSTACLE_DENSITY {
                continue;
            }
            let tile = UVec2::new(x, y);
            let center = tile_center(tile);
            if center.length() < SAFE_RADIUS {
                continue;
            }
            let near_other = (y - 1..=y + 1)
                .any(|ny| (x - 1..=x + 1).any(|nx| occupied[ny as usize * size + nx as usize]));
            if near_other {
                continue;
            }
            occupied[y as usize * size + x as usize] = true;
            // Sulla terra solo rocce; sull'erba 60% alberi.
            let tree_roll = unit_f32(hash2(seed ^ KIND_SALT, ix, iy));
            let kind = if tile_at(seed, x, y) != DIRT && tree_roll < 0.6 {
                ObstacleKind::Tree
            } else {
                ObstacleKind::Rock
            };
            obstacles.push(ObstacleSpec {
                kind,
                tile,
                base: center - Vec2::Y * (TILE_SIZE as f32 / 2.0 - 4.0),
            });
        }
    }
    obstacles
}

/// Entità visibile di un ostacolo (la presentazione aggancia lo sprite). La collisione usa
/// `ObstacleField`, non queste entità.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Obstacle {
    pub kind: ObstacleKind,
}

/// Ostacoli della run con griglia spaziale per trovare in O(1) quelli vicini a un punto.
#[derive(Resource, Debug, Clone)]
pub struct ObstacleField {
    pub obstacles: Vec<ObstacleSpec>,
    cells: Vec<Vec<u32>>,
}

impl ObstacleField {
    pub fn new(obstacles: Vec<ObstacleSpec>) -> Self {
        let mut cells = vec![Vec::new(); GRID_CELLS * GRID_CELLS];
        for (index, obstacle) in obstacles.iter().enumerate() {
            let (cx, cy) = cell_of(obstacle.footprint_center());
            cells[cy * GRID_CELLS + cx].push(index as u32);
        }
        Self { obstacles, cells }
    }

    /// Ostacoli la cui impronta può toccare un cerchio di raggio `reach` attorno a `point`.
    pub fn near(&self, point: Vec2, reach: f32) -> impl Iterator<Item = &ObstacleSpec> {
        let margin = Vec2::splat(reach + MAX_FOOTPRINT_RADIUS);
        let (x0, y0) = cell_of(point - margin);
        let (x1, y1) = cell_of(point + margin);
        (y0..=y1)
            .flat_map(move |cy| (x0..=x1).map(move |cx| cy * GRID_CELLS + cx))
            .flat_map(|cell| self.cells[cell].iter())
            .map(|&index| &self.obstacles[index as usize])
    }
}

fn cell_of(point: Vec2) -> (usize, usize) {
    let local = ((point + Vec2::splat(ARENA_HALF_EXTENT)) / CELL_SIZE).floor();
    let clamp = |v: f32| (v.max(0.0) as usize).min(GRID_CELLS - 1);
    (clamp(local.x), clamp(local.y))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn generation_is_deterministic_and_seed_dependent() {
        let a = generate_obstacles(11);
        assert_eq!(a, generate_obstacles(11));
        assert_ne!(a, generate_obstacles(12));
        // densità 1,2% su ~62 mila tile interni, meno gli scarti per vicinanza
        assert!((400..900).contains(&a.len()), "{} ostacoli", a.len());
        assert!(a.iter().any(|o| o.kind == ObstacleKind::Tree));
        assert!(a.iter().any(|o| o.kind == ObstacleKind::Rock));
    }

    #[test]
    fn start_area_is_free() {
        for seed in 0..20 {
            for o in generate_obstacles(seed) {
                assert!(
                    o.footprint_center().length() >= SAFE_RADIUS - 16.0,
                    "seed {seed}"
                );
            }
        }
    }

    #[test]
    fn gaps_between_obstacles_fit_the_largest_body() {
        let largest_body_diameter = 12.0;
        let obstacles = generate_obstacles(3);
        let field = ObstacleField::new(obstacles.clone());
        for a in &obstacles {
            for b in field.near(a.footprint_center(), 40.0) {
                if a == b {
                    continue;
                }
                let gap = a.footprint_center().distance(b.footprint_center())
                    - a.footprint_radius()
                    - b.footprint_radius();
                assert!(gap >= largest_body_diameter, "varco di {gap} px");
            }
        }
    }

    #[test]
    fn every_free_tile_is_reachable_from_the_start() {
        let size = ARENA_TILES as usize;
        let mut blocked = vec![false; size * size];
        for o in generate_obstacles(5) {
            blocked[o.tile.y as usize * size + o.tile.x as usize] = true;
        }
        let inside = |x: usize, y: usize| {
            let (lo, hi) = (BORDER_TILES as usize, size - BORDER_TILES as usize);
            (lo..hi).contains(&x) && (lo..hi).contains(&y)
        };
        let start = (size / 2, size / 2);
        let mut seen = vec![false; size * size];
        let mut queue = VecDeque::from([start]);
        seen[start.1 * size + start.0] = true;
        let mut reached = 0;
        while let Some((x, y)) = queue.pop_front() {
            reached += 1;
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                let i = ny * size + nx;
                if inside(nx, ny) && !blocked[i] && !seen[i] {
                    seen[i] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        let interior = (size - 2 * BORDER_TILES as usize).pow(2);
        let free = interior - blocked.iter().filter(|b| **b).count();
        assert_eq!(reached, free, "tile liberi irraggiungibili");
    }

    #[test]
    fn grid_query_finds_exactly_the_nearby_obstacles() {
        let obstacles = generate_obstacles(9);
        let field = ObstacleField::new(obstacles.clone());
        for probe in [
            Vec2::new(300.0, -200.0),
            Vec2::new(-1900.0, 1900.0),
            Vec2::ZERO,
        ] {
            let reach = 20.0;
            let mut brute: Vec<_> = obstacles
                .iter()
                .filter(|o| o.footprint_center().distance(probe) <= reach + o.footprint_radius())
                .collect();
            let mut grid: Vec<_> = field
                .near(probe, reach)
                .filter(|o| o.footprint_center().distance(probe) <= reach + o.footprint_radius())
                .collect();
            brute.sort_by_key(|o| (o.tile.y, o.tile.x));
            grid.sort_by_key(|o| (o.tile.y, o.tile.x));
            assert_eq!(brute, grid);
        }
    }
}
