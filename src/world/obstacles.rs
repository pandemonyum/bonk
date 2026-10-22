//! Ostacoli dell'arena (rocce, alberi): generazione pura dal seed e griglia spaziale statica.
//!
//! Garanzie strutturali (nel codice, non nei dati):
//! - due ostacoli non stanno mai in tile adiacenti (anche in diagonale): i centri distano
//!   almeno 2 tile e tra due impronte resta un varco di almeno 32 − 2·7 = 18 px, più largo di
//!   qualunque corpo ammesso dalla validazione dei dati. Niente vicoli ciechi, niente pathfinding;
//! - due tile di margine dalla siepe: la zona di respinta di ogni ostacolo sta dentro i limiti
//!   dei piedi, quindi il clamp ai bordi non riporta mai un corpo dentro un ostacolo.
//!
//! Densità, zona libera e quota di alberi sono parametri dello stage (`stages.ron`).

use bevy::prelude::*;

use super::generation::tile_at;
use super::tiles::DIRT;
use super::{ARENA_HALF_EXTENT, ARENA_TILES, BORDER_TILES, TILE_SIZE};
use crate::data::schema::ObstacleParams;
use crate::rng::{hash2, unit_f32};

const OBSTACLE_SALT: u64 = 0x0B57_AC1E;
const KIND_SALT: u64 = 0x7EE5_0001;
/// Tile liberi tra la siepe e il primo ostacolo.
const EDGE_MARGIN_TILES: u32 = 2;
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
/// Raggio dell'impronta più piccola (tronco d'albero): limita la velocità dei corpi, perché
/// una sola respinta per passo non deve far "saltare" un corpo oltre il centro dell'ostacolo.
pub const MIN_FOOTPRINT_RADIUS: f32 = 4.0;
/// Varco minimo garantito tra due impronte: centri ad almeno 2 tile, meno due impronte
/// massime. I dati validano che nessun corpo sia più largo (`data::validate`).
pub const MIN_OBSTACLE_GAP: f32 = 2.0 * TILE_SIZE as f32 - 2.0 * MAX_FOOTPRINT_RADIUS;

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
pub fn generate_obstacles(seed: u64, params: &ObstacleParams) -> Vec<ObstacleSpec> {
    let size = ARENA_TILES as usize;
    let mut occupied = vec![false; size * size];
    let mut obstacles = Vec::new();
    let first = BORDER_TILES + EDGE_MARGIN_TILES;
    let last = ARENA_TILES - BORDER_TILES - EDGE_MARGIN_TILES;
    for y in first..last {
        for x in first..last {
            let (ix, iy) = (x as i32, y as i32);
            if unit_f32(hash2(seed ^ OBSTACLE_SALT, ix, iy)) >= params.density {
                continue;
            }
            let tile = UVec2::new(x, y);
            let center = tile_center(tile);
            if center.length() < params.safe_radius {
                continue;
            }
            let near_other = (y - 1..=y + 1)
                .any(|ny| (x - 1..=x + 1).any(|nx| occupied[ny as usize * size + nx as usize]));
            if near_other {
                continue;
            }
            occupied[y as usize * size + x as usize] = true;
            // Sulla terra solo rocce; sull'erba alberi con probabilità `tree_chance`.
            let tree_roll = unit_f32(hash2(seed ^ KIND_SALT, ix, iy));
            let kind = if tile_at(seed, x, y) != DIRT && tree_roll < params.tree_chance {
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
    use crate::data::GameConfig;
    use crate::world::ArenaBounds;

    /// Parametri dello stage giocato (`stages.ron`).
    fn params() -> ObstacleParams {
        GameConfig::embedded().stage.obstacles
    }

    fn generate(seed: u64) -> Vec<ObstacleSpec> {
        generate_obstacles(seed, &params())
    }

    #[test]
    fn generation_is_deterministic_and_seed_dependent() {
        let a = generate(11);
        assert_eq!(a, generate(11));
        assert_ne!(a, generate(12));
        // densità 1,2% su ~61 mila tile candidati, meno gli scarti per vicinanza
        assert!((400..900).contains(&a.len()), "{} ostacoli", a.len());
        assert!(a.iter().any(|o| o.kind == ObstacleKind::Tree));
        assert!(a.iter().any(|o| o.kind == ObstacleKind::Rock));
    }

    #[test]
    fn parameters_change_the_field() {
        let none = ObstacleParams {
            density: 0.0,
            ..params()
        };
        assert!(generate_obstacles(1, &none).is_empty());
        let rocks_only = ObstacleParams {
            tree_chance: 0.0,
            ..params()
        };
        let field = generate_obstacles(1, &rocks_only);
        assert!(!field.is_empty() && field.iter().all(|o| o.kind == ObstacleKind::Rock));
    }

    #[test]
    fn start_area_is_free() {
        let safe = params().safe_radius;
        for seed in 0..20 {
            for o in generate(seed) {
                assert!(o.footprint_center().length() >= safe - 16.0, "seed {seed}");
            }
        }
    }

    #[test]
    fn footprint_constants_match_the_kinds() {
        let radii = [ObstacleKind::Rock, ObstacleKind::Tree].map(|k| k.footprint().0);
        assert_eq!(MAX_FOOTPRINT_RADIUS, radii.into_iter().fold(0.0, f32::max));
        assert_eq!(
            MIN_FOOTPRINT_RADIUS,
            radii.into_iter().fold(f32::MAX, f32::min)
        );
    }

    /// Con due tile di margine dalla siepe, la zona da cui un ostacolo respinge il corpo più
    /// largo ammesso sta tutta dentro i limiti dei piedi: il clamp non può riportarvi un corpo.
    #[test]
    fn obstacle_exclusion_zones_stay_inside_the_walkable_area() {
        let arena = ArenaBounds::default();
        let widest_body = MIN_OBSTACLE_GAP / 2.0;
        for seed in 0..10 {
            for o in generate(seed) {
                let reach = Vec2::splat(o.footprint_radius() + widest_body);
                let (lo, hi) = (o.footprint_center() - reach, o.footprint_center() + reach);
                assert!(
                    arena.contains(lo) && arena.contains(hi),
                    "seed {seed}, tile {:?}",
                    o.tile
                );
            }
        }
    }

    #[test]
    fn gaps_between_obstacles_are_never_below_the_guaranteed_minimum() {
        assert_eq!(MIN_OBSTACLE_GAP, 18.0);
        let obstacles = generate(3);
        let field = ObstacleField::new(obstacles.clone());
        for a in &obstacles {
            for b in field.near(a.footprint_center(), 40.0) {
                if a == b {
                    continue;
                }
                let gap = a.footprint_center().distance(b.footprint_center())
                    - a.footprint_radius()
                    - b.footprint_radius();
                assert!(gap >= MIN_OBSTACLE_GAP - 1e-3, "varco di {gap} px");
            }
        }
    }

    #[test]
    fn every_free_tile_is_reachable_from_the_start() {
        let size = ARENA_TILES as usize;
        let mut blocked = vec![false; size * size];
        for o in generate(5) {
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

    /// Sonde vicino a ostacoli veri: quelli sulle righe e colonne estreme (bordi della griglia)
    /// e uno centrale, con uno scarto che fa cadere la sonda in un'altra cella.
    #[test]
    fn grid_query_finds_exactly_the_nearby_obstacles() {
        let obstacles = generate(9);
        let field = ObstacleField::new(obstacles.clone());
        let pick =
            |key: fn(&ObstacleSpec) -> i64| *obstacles.iter().min_by_key(|o| key(o)).unwrap();
        let probes = [
            pick(|o| i64::from(o.tile.x)),  // colonna più a ovest
            pick(|o| -i64::from(o.tile.x)), // colonna più a est
            pick(|o| i64::from(o.tile.y)),  // riga più a sud
            pick(|o| -i64::from(o.tile.y)), // riga più a nord
            obstacles[obstacles.len() / 2],
        ];
        let reach = 20.0;
        for obstacle in probes {
            let probe = obstacle.footprint_center() + Vec2::new(13.0, -11.0);
            let close = |o: &&ObstacleSpec| {
                o.footprint_center().distance(probe) <= reach + o.footprint_radius()
            };
            let mut brute: Vec<_> = obstacles.iter().filter(close).collect();
            let mut grid: Vec<_> = field.near(probe, reach).filter(close).collect();
            assert!(!brute.is_empty(), "sonda senza ostacoli vicini: test vacuo");
            brute.sort_by_key(|o| (o.tile.y, o.tile.x));
            grid.sort_by_key(|o| (o.tile.y, o.tile.x));
            assert_eq!(brute, grid);
        }
    }
}
