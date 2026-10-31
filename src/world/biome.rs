//! Bioma compilato (D-068): i nomi dei tile di `stages.ron` diventano indici dei tileset e i pesi
//! soglie del tiro. Un solo generatore (`generation`, `obstacles`) e un solo renderer (`render`)
//! servono tutti i biomi: un bioma nuovo è un blocco di dati con i suoi asset, non codice.
//!
//! `compile` è anche la validazione dei biomi: `data::validate` ne riporta gli errori.

use bevy::prelude::*;

use super::obstacles::{MAX_FOOTPRINT_RADIUS, MIN_FOOTPRINT_RADIUS};
use crate::data::GameConfig;
use crate::data::schema::{BiomeData, Surface};

/// Bit dei lati di un tile di bordo; nord è y + 1 (l'arena è y-up, la riga 0 dell'immagine del
/// tile è il lato nord). Il bit è acceso se il vicino da quel lato è di un altro materiale.
pub const EDGE_N: u16 = 1;
pub const EDGE_E: u16 = 2;
pub const EDGE_S: u16 = 4;
pub const EDGE_W: u16 = 8;
/// Bordi di un materiale: uno per maschera da 1 a 15, in coda al tileset.
pub const EDGE_TILES: u16 = 15;

/// Nome del bordo di `material` per `mask`, come `edge_name` in `scripts/art_tileset.py`.
pub fn edge_name(material: &str, mask: u16) -> String {
    let sides = [(EDGE_N, 'n'), (EDGE_E, 'e'), (EDGE_S, 's'), (EDGE_W, 'w')];
    let suffix: String = sides
        .iter()
        .filter(|(bit, _)| mask & bit != 0)
        .map(|(_, side)| *side)
        .collect();
    format!("{material}_{suffix}")
}

/// Terreno logico di un tile: decide dove vanno ostacoli e decorazioni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Floor {
    Ground,
    Patch,
    Border,
}

impl Floor {
    pub fn accepts(self, surface: Surface) -> bool {
        matches!(
            (surface, self),
            (Surface::Any, Floor::Ground | Floor::Patch)
                | (Surface::Ground, Floor::Ground)
                | (Surface::Patch, Floor::Patch)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    pub tile: u16,
    /// Primo dei 15 bordi, se il materiale li ha.
    pub edges: Option<u16>,
}

impl Material {
    /// Il bordo per la maschera dei lati che confinano con un altro materiale; il tile pieno
    /// con maschera 0 o senza bordi.
    pub fn edge(self, mask: u16) -> u16 {
        match self.edges {
            Some(first) if mask != 0 => first + mask - 1,
            _ => self.tile,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub material: Material,
    pub cell: i32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Variant {
    pub from: u16,
    pub to: u16,
    pub below: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clusters {
    pub tiles: Vec<u16>,
    pub cell: i32,
    pub threshold: f32,
    pub share: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    pub tileset: String,
    pub tile_count: u32,
    /// Tile di base con la soglia cumulata del tiro (l'ultimo prende anche il resto).
    pub ground: Vec<(u16, f32)>,
    pub patch: Option<Patch>,
    pub border: Material,
    pub variants: Vec<Variant>,
    pub clusters: Option<Clusters>,
}

impl Terrain {
    /// Tile di base per il tiro `roll` in [0, 1).
    pub fn ground_tile(&self, roll: f32) -> u16 {
        let (tile, _) = self
            .ground
            .iter()
            .find(|(_, below)| roll < *below)
            .or(self.ground.last())
            .expect("almeno un tile di base: validato");
        *tile
    }

    /// Variante grafica di un tile pieno per il tiro `roll`, o il tile stesso.
    pub fn variant(&self, tile: u16, roll: f32) -> u16 {
        self.variants
            .iter()
            .find(|v| v.from == tile && roll < v.below)
            .map_or(tile, |v| v.to)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecorRule {
    pub tiles: Vec<u16>,
    pub on: Surface,
    pub density: f32,
    pub cluster: Option<(i32, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decorations {
    pub tileset: String,
    pub tile_count: u32,
    pub rules: Vec<DecorRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObstacleKind {
    pub id: String,
    pub sprite: String,
    pub radius: f32,
    /// Soglie cumulate del tiro sul terreno di base e sulle macchie.
    ground: f32,
    patch: f32,
}

/// Il bioma dello stage giocato, pronto per generare e disegnare la mappa.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Biome {
    pub id: String,
    pub terrain: Terrain,
    pub decorations: Option<Decorations>,
    pub obstacles: Vec<ObstacleKind>,
}

impl Biome {
    /// Bioma dello stage giocato (i dati sono già validati).
    pub fn of_stage(config: &GameConfig) -> Self {
        Self::compile(config.biome())
            .unwrap_or_else(|errors| panic!("bioma non validato: {}", errors.join("; ")))
    }

    /// Tipo di ostacolo (indice in `obstacles`) per il tiro `roll` su `floor`; `None` se lì
    /// non ne va nessuno.
    pub fn obstacle_kind(&self, floor: Floor, roll: f32) -> Option<u16> {
        self.obstacles
            .iter()
            .position(|kind| match floor {
                Floor::Ground => roll < kind.ground,
                Floor::Patch => roll < kind.patch,
                Floor::Border => false,
            })
            .map(|index| index as u16)
    }

    /// Nomi dei tile del terreno nell'ordine del tileset, bordi compresi.
    pub fn terrain_names(data: &BiomeData) -> Vec<String> {
        let terrain = &data.terrain;
        let edges = terrain
            .edges
            .iter()
            .flat_map(|material| (1..=EDGE_TILES).map(move |mask| edge_name(material, mask)));
        terrain.tiles.iter().cloned().chain(edges).collect()
    }
}

mod compile;

#[cfg(test)]
mod tests;
