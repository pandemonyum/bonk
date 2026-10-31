//! Ostacoli dell'arena: generazione pura dal seed e griglia spaziale statica. I tipi (sprite,
//! impronta, dove vanno) li dà il bioma dello stage (D-068), le regole sotto valgono per tutti.
//!
//! Garanzie strutturali (nel codice, non nei dati):
//! - due ostacoli non stanno mai in tile adiacenti (anche in diagonale): i centri distano
//!   almeno 2 tile e tra due impronte resta un varco di almeno 32 − 2·7 = 18 px, più largo di
//!   qualunque corpo ammesso dalla validazione dei dati. Niente vicoli ciechi, niente pathfinding;
//! - due tile di margine dalla siepe: la zona di respinta di ogni ostacolo sta dentro i limiti
//!   dei piedi, quindi il clamp ai bordi non riporta mai un corpo dentro un ostacolo.
//!
//! - ogni impronta è un cerchio centrato nel tile, con raggio tra `MIN_FOOTPRINT_RADIUS` e
//!   `MAX_FOOTPRINT_RADIUS` (validato nei dati).
//!
//! Densità e zona libera sono parametri dello stage, i tipi e i loro pesi del bioma
//! (`stages.ron`).

use bevy::prelude::*;

use super::biome::Biome;
use super::generation::floor_at;
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

/// Altezza del centro dell'impronta sopra la base dello sprite: il centro del tile (art pass
/// v1: la base sta 1-2 px sopra il fondo dello sprite, con l'ombra cotta).
pub const FOOTPRINT_LIFT: f32 = 4.0;
/// Raggio massimo di un'impronta (la roccia), per le query sulla griglia e per i varchi.
pub const MAX_FOOTPRINT_RADIUS: f32 = 7.0;
/// Raggio minimo di un'impronta (il tronco d'albero): limita la velocità dei corpi, perché
/// una sola respinta per passo non deve far "saltare" un corpo oltre il centro dell'ostacolo.
pub const MIN_FOOTPRINT_RADIUS: f32 = 4.0;
/// Varco minimo garantito tra due impronte: centri ad almeno 2 tile, meno due impronte
/// massime. I dati validano che nessun corpo sia più largo (`data::validate`).
pub const MIN_OBSTACLE_GAP: f32 = 2.0 * TILE_SIZE as f32 - 2.0 * MAX_FOOTPRINT_RADIUS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleSpec {
    /// Indice del tipo negli ostacoli del bioma.
    pub kind: u16,
    /// Tile dell'arena (origine in basso a sinistra).
    pub tile: UVec2,
    /// Base dello sprite in pixel di mondo (ancora in basso al centro).
    pub base: Vec2,
    pub radius: f32,
}

impl ObstacleSpec {
    pub fn footprint_center(&self) -> Vec2 {
        self.base + Vec2::Y * FOOTPRINT_LIFT
    }

    pub fn footprint_radius(&self) -> f32 {
        self.radius
    }
}

/// Centro in pixel di mondo del tile `(x, y)`.
pub fn tile_center(tile: UVec2) -> Vec2 {
    tile.as_vec2() * TILE_SIZE as f32 + Vec2::splat(TILE_SIZE as f32 / 2.0 - ARENA_HALF_EXTENT)
}

/// Ostacoli dell'arena per `seed`, in ordine di scansione (y poi x): deterministico.
pub fn generate_obstacles(seed: u64, biome: &Biome, params: &ObstacleParams) -> Vec<ObstacleSpec> {
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
            // Il tipo dipende dal terreno: il bioma dà i pesi su base e macchie (la prateria:
            // sulla terra solo rocce, sull'erba alberi e rocce).
            let roll = unit_f32(hash2(seed ^ KIND_SALT, ix, iy));
            let floor = floor_at(&biome.terrain, seed, x, y);
            let Some(kind) = biome.obstacle_kind(floor, roll) else {
                continue;
            };
            occupied[y as usize * size + x as usize] = true;
            obstacles.push(ObstacleSpec {
                kind,
                tile,
                base: center - Vec2::Y * (TILE_SIZE as f32 / 2.0 - FOOTPRINT_LIFT),
                radius: biome.obstacles[kind as usize].radius,
            });
        }
    }
    obstacles
}

/// Entità visibile di un ostacolo (la presentazione aggancia lo sprite). La collisione usa
/// `ObstacleField`, non queste entità.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Obstacle {
    /// Indice del tipo negli ostacoli del bioma (`Biome::obstacles`).
    pub kind: u16,
}

/// Ostacoli della run con griglia spaziale per trovare in O(1) quelli vicini a un punto.
#[derive(Resource, Debug, Clone)]
pub struct ObstacleField {
    pub obstacles: Vec<ObstacleSpec>,
    cells: Vec<Vec<u32>>,
    /// Tile occupati da un ostacolo, riga per riga.
    tiles: Vec<bool>,
}

impl ObstacleField {
    pub fn new(obstacles: Vec<ObstacleSpec>) -> Self {
        let mut cells = vec![Vec::new(); GRID_CELLS * GRID_CELLS];
        let side = ARENA_TILES as usize;
        let mut tiles = vec![false; side * side];
        for (index, obstacle) in obstacles.iter().enumerate() {
            let (cx, cy) = cell_of(obstacle.footprint_center());
            cells[cy * GRID_CELLS + cx].push(index as u32);
            tiles[obstacle.tile.y as usize * side + obstacle.tile.x as usize] = true;
        }
        Self {
            obstacles,
            cells,
            tiles,
        }
    }

    /// C'è un ostacolo nel tile `(x, y)` (le decorazioni non vanno sotto gli ostacoli).
    pub fn occupies(&self, x: u32, y: u32) -> bool {
        let side = ARENA_TILES as usize;
        (x as usize) < side && self.tiles.get(y as usize * side + x as usize) == Some(&true)
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
mod tests;
