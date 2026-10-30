//! Indici del tileset `assets/tiles/meadow_tileset.png`.
//! Devono restare allineati a `TILE_NAMES` in scripts/art_tileset.py.

pub const GRASS_A: u16 = 0;
pub const GRASS_B: u16 = 1;
pub const GRASS_TUFT: u16 = 2;
pub const FLOWERS_WHITE: u16 = 3;
pub const FLOWERS_PINK: u16 = 4;
pub const DIRT: u16 = 5;
pub const HEDGE: u16 = 6;
// Varianti solo grafiche (art pass v1): per la logica restano erba, terra o siepe.
pub const GRASS_C: u16 = 7;
pub const GRASS_CLOVER: u16 = 8;
pub const FLOWERS_YELLOW: u16 = 9;
pub const GRASS_PEBBLES: u16 = 10;
pub const GRASS_TALL: u16 = 11;
/// Primo dei 15 bordi terra/erba: indice `DIRT_EDGES + maschera - 1`, con la maschera (1..=15)
/// fatta dei bit `EDGE_*` dei lati che confinano con un altro materiale.
pub const DIRT_EDGES: u16 = 12;
/// Primo dei 15 bordi siepe/erba, stessa maschera.
pub const HEDGE_EDGES: u16 = 27;
/// Bit dei lati di un tile di bordo; nord è y + 1 (l'arena è y-up, la riga 0 dell'immagine del
/// tile è il lato nord).
pub const EDGE_N: u16 = 1;
pub const EDGE_E: u16 = 2;
pub const EDGE_S: u16 = 4;
pub const EDGE_W: u16 = 8;
/// Numero di tile impilati verticalmente nel tileset.
pub const TILE_COUNT: u32 = 42;

pub const TILESET_PATH: &str = "tiles/meadow_tileset.png";
