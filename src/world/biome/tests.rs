//! I biomi dei dati e gli asset della pipeline (`assets/manifest.json`) devono coincidere: ogni
//! bioma carica i suoi tileset, con gli stessi nomi nello stesso ordine, e i suoi sprite.

use super::*;
use crate::world::TILE_SIZE;

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn manifest() -> String {
    std::fs::read_to_string(format!("{ASSETS}/manifest.json")).expect("assets/manifest.json")
}

/// Testo della voce del manifest con questo `path` (il manifest lo scrive `json.dumps` di
/// `scripts/build-assets.py`: `path` è sempre la prima chiave di una voce).
fn entry<'a>(manifest: &'a str, path: &str) -> &'a str {
    let key = format!("\"path\": \"{path}\"");
    let start = manifest
        .find(&key)
        .unwrap_or_else(|| panic!("{path} non è nel manifest: rigenera con build-assets.py"));
    let rest = &manifest[start + key.len()..];
    &rest[..rest.find("\"path\":").unwrap_or(rest.len())]
}

/// Valore lista di `field` nella voce: stringhe o numeri, senza virgolette.
fn list(entry: &str, field: &str) -> Vec<String> {
    let key = format!("\"{field}\": [");
    let start = entry.find(&key).unwrap_or_else(|| panic!("manca {field}")) + key.len();
    let body = &entry[start..start + entry[start..].find(']').unwrap()];
    body.split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn size(entry: &str) -> (u32, u32) {
    let numbers: Vec<u32> = list(entry, "size")
        .iter()
        .map(|n| n.parse().unwrap())
        .collect();
    (numbers[0], numbers[1])
}

fn assert_tileset(manifest: &str, path: &str, names: &[String]) {
    let entry = entry(manifest, path);
    assert_eq!(list(entry, "tiles"), names, "nomi dei tile di {path}");
    assert_eq!(
        size(entry),
        (TILE_SIZE, TILE_SIZE * names.len() as u32),
        "{path}"
    );
    assert!(
        std::path::Path::new(&format!("{ASSETS}/{path}")).is_file(),
        "{path}"
    );
}

#[test]
fn every_biome_loads_its_own_assets_as_built_by_the_pipeline() {
    let config = GameConfig::embedded();
    let manifest = manifest();
    for data in &config.biomes {
        let biome = Biome::compile(data).unwrap();
        let names = Biome::terrain_names(data);
        assert_eq!(biome.terrain.tile_count as usize, names.len());
        assert_tileset(&manifest, &biome.terrain.tileset, &names);
        if let (Some(decorations), Some(data)) = (&biome.decorations, &data.decorations) {
            assert_eq!(decorations.tile_count as usize, data.tiles.len());
            assert_tileset(&manifest, &decorations.tileset, &data.tiles);
        }
        for kind in &biome.obstacles {
            let (w, h) = size(entry(&manifest, &kind.sprite));
            // ancora in basso al centro e larghezza pari (pixel-perfect); l'impronta ci sta
            assert!(
                w % 2 == 0 && w as f32 >= 2.0 * kind.radius,
                "{}: {w}×{h}",
                kind.id
            );
            assert!(h as f32 >= super::super::obstacles::FOOTPRINT_LIFT + kind.radius);
            let file = format!("{ASSETS}/{}", kind.sprite);
            assert!(std::path::Path::new(&file).is_file(), "{file}");
        }
    }
    // prateria e bosco caricano ciascuno il proprio terreno (biomi futuri possono condividerlo)
    let tileset = |id: &str| config.find_biome(id).unwrap().terrain.tileset.clone();
    assert_ne!(tileset("meadow"), tileset("forest"));
}

#[test]
fn edge_names_follow_the_art_pipeline() {
    assert_eq!(edge_name("dirt", EDGE_N), "dirt_n");
    assert_eq!(edge_name("dirt", EDGE_N | EDGE_E), "dirt_ne");
    assert_eq!(edge_name("hedge", EDGE_S | EDGE_W), "hedge_sw");
    assert_eq!(edge_name("thicket", 15), "thicket_nesw");
}

#[test]
fn weights_become_the_same_thresholds_as_the_old_constants() {
    let meadow = Biome::compile(GameConfig::embedded().find_biome("meadow").unwrap()).unwrap();
    let thresholds: Vec<f32> = meadow.terrain.ground.iter().map(|(_, t)| *t).collect();
    assert_eq!(thresholds, [0.42, 0.84, 0.93, 0.97, 1.0]);
    // sulla terra solo rocce, sull'erba alberi sotto 0,6
    let tree = meadow
        .obstacles
        .iter()
        .position(|k| k.id == "tree")
        .unwrap() as u16;
    let rock = meadow
        .obstacles
        .iter()
        .position(|k| k.id == "rock")
        .unwrap() as u16;
    assert_eq!(meadow.obstacle_kind(Floor::Ground, 0.599_999), Some(tree));
    assert_eq!(meadow.obstacle_kind(Floor::Ground, 0.6), Some(rock));
    assert_eq!(meadow.obstacle_kind(Floor::Patch, 0.0), Some(rock));
    assert_eq!(meadow.obstacle_kind(Floor::Border, 0.0), None);
}
