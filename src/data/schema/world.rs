//! Biomi (`stages.ron: biomes`, D-068): l'aspetto della mappa in tre livelli, cioè terreno,
//! decorazioni calpestabili e ostacoli con collisione. Uno stage sceglie il bioma per id; quanti
//! ostacoli e quanto è larga la zona libera restano dello stage (`ObstacleParams`).
//!
//! I tile si nominano come nel manifest degli asset: gli indici li ricava `world::biome`
//! dall'ordine dei tileset, e un test confronta i nomi con `assets/manifest.json`.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BiomeData {
    pub id: String,
    pub terrain: TerrainData,
    /// Senza: nessun livello di decorazioni (la prateria ha solo le varianti del terreno).
    #[serde(default)]
    pub decorations: Option<DecorationsData>,
    /// Tipi di ostacolo; il tiro sceglie fra quelli con peso sul terreno del tile.
    pub obstacles: Vec<ObstacleKindData>,
}

/// Terreno: tileset opaco, materiale di base, macchie (radure), muro di bordo e varianti
/// solo grafiche. Per la logica (ostacoli, decorazioni) contano solo base, macchie e bordo.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainData {
    /// Tileset a colonna di tile 16×16 opachi, relativo ad `assets/`.
    pub tileset: String,
    /// Tile del tileset dall'alto, prima dei bordi.
    pub tiles: Vec<String>,
    /// Materiali con i bordi morbidi: in coda a `tiles`, 15 tile ciascuno (`<nome>_n`,
    /// `<nome>_e`, `<nome>_ne`, ... nell'ordine della maschera N=1, E=2, S=4, O=8).
    #[serde(default)]
    pub edges: Vec<String>,
    /// Terreno di base: (tile, peso), tirato tile per tile.
    pub ground: Vec<(String, f64)>,
    /// Macchie di un secondo materiale dove il rumore supera la soglia.
    #[serde(default)]
    pub patch: Option<PatchData>,
    /// Muro che chiude l'arena (`world::BORDER_TILES` di spessore).
    pub border: String,
    /// Varianti dei tile pieni: (tile, variante, soglia). Vale la prima il cui tiro è sotto la
    /// soglia: per lo stesso tile le soglie vanno in ordine crescente.
    #[serde(default)]
    pub variants: Vec<(String, String, f32)>,
    /// Tile raccolti a macchie sul terreno di base (i fiori del prato).
    #[serde(default)]
    pub clusters: Option<ClusterData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchData {
    pub tile: String,
    /// Lato in tile delle celle del rumore: più grande, macchie più larghe.
    pub cell: u32,
    /// Soglia del rumore (0..1): più alta, macchie più rare.
    pub threshold: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterData {
    /// Uno a caso fra questi, in parti uguali.
    pub tiles: Vec<String>,
    pub cell: u32,
    pub threshold: f32,
    /// Quota dei tile di una macchia che ne prendono uno.
    pub share: f32,
}

/// Decorazioni calpestabili: un tileset con trasparenza cotto sopra il terreno, mai sotto un
/// ostacolo né sul muro. Non cambiano la mappa logica.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecorationsData {
    /// Tileset a colonna di tile 16×16 con alpha 0 o 255, relativo ad `assets/`.
    pub tileset: String,
    pub tiles: Vec<String>,
    /// In ordine: su ogni tile vince la prima regola che lo decora.
    pub rules: Vec<DecorRuleData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecorRuleData {
    /// Uno a caso fra questi, in parti uguali.
    pub tiles: Vec<String>,
    pub on: Surface,
    /// Quota dei tile adatti (dentro le macchie, con `cluster`) che la ricevono.
    pub density: f32,
    /// A macchie: (lato in tile delle celle del rumore, soglia). Senza: sparse.
    #[serde(default)]
    pub cluster: Option<(u32, f32)>,
}

/// Dove sta una decorazione.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Surface {
    Ground,
    Patch,
    /// Base e macchie (mai il muro).
    Any,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleKindData {
    pub id: String,
    /// Sprite con l'ancora in basso al centro, relativo ad `assets/`.
    pub sprite: String,
    /// Raggio dell'impronta di collisione, un cerchio centrato nel tile (4 px sopra la base
    /// dello sprite): la parte dello sprite che tocca terra.
    pub footprint_radius: f32,
    /// Peso fra i tipi sul terreno di base (0 = mai).
    pub ground: f64,
    /// Peso fra i tipi sulle macchie (0 = mai).
    #[serde(default)]
    pub patch: f64,
}
