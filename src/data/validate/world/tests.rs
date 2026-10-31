//! Validazione dei biomi: un test per ogni campo. Nomi e id vengono dai dati veri.

use super::super::tests::{assert_error, embedded};
use crate::data::GameConfig;
use crate::data::schema::{BiomeData, DecorRuleData, PatchData, Surface};

/// Un bioma senza decorazioni (la prateria) e uno con (il bosco), dai dati veri.
fn ids() -> (String, String) {
    let c = embedded();
    let plain = c.biomes.iter().find(|b| b.decorations.is_none()).unwrap();
    let decorated = c.biomes.iter().find(|b| b.decorations.is_some()).unwrap();
    (plain.id.clone(), decorated.id.clone())
}

fn biome<'a>(c: &'a mut GameConfig, id: &str) -> &'a mut BiomeData {
    c.biomes.iter_mut().find(|b| b.id == id).unwrap()
}

fn at(id: &str, field: &str) -> String {
    format!("stages.ron: biomes.{id}.{field}")
}

#[test]
fn stages_name_an_existing_biome_and_ids_are_unique() {
    let c = embedded();
    assert_error(
        |c| c.stage.biome = "lava".into(),
        &format!(
            "stages.ron: {}.biome = \"lava\": non esiste in stages.ron: biomes",
            c.stage.id
        ),
    );
    assert_error(
        |c| c.biomes[1].id = c.biomes[0].id.clone(),
        &format!("stages.ron: biomes: id \"{}\" duplicato", c.biomes[0].id),
    );
    assert_error(
        |c| c.stages[1].id = c.stages[0].id.clone(),
        &format!("stages.ron: stages: id \"{}\" duplicato", c.stages[0].id),
    );
}

#[test]
fn tile_names_must_exist_and_materials_stay_distinct() {
    let (plain, decorated) = ids();
    let c = embedded();
    let border = c.find_biome(&plain).unwrap().terrain.border.clone();
    let first = c.find_biome(&decorated).unwrap().terrain.tiles[0].clone();
    assert_error(
        |c| biome(c, &plain).terrain.ground[0].0 = "nessuno".into(),
        &format!(
            "{}: \"nessuno\" non è un tile del tileset",
            at(&plain, "terrain.ground[0]")
        ),
    );
    assert_error(
        |c| biome(c, &plain).terrain.ground[0].0 = border.clone(),
        &format!(
            "{}: \"{border}\" è già muro o macchia",
            at(&plain, "terrain.ground")
        ),
    );
    assert_error(
        |c| biome(c, &decorated).terrain.edges.push("nessuno".into()),
        &format!("{}: \"nessuno\"", at(&decorated, "terrain.edges")),
    );
    assert_error(
        |c| biome(c, &decorated).terrain.tiles.push(first.clone()),
        &format!("{}: \"{first}\" duplicato", at(&decorated, "terrain.tiles")),
    );
}

#[test]
fn weights_and_noise_are_checked() {
    let (plain, _) = ids();
    assert_error(
        |c| {
            let patch = biome(c, &plain).terrain.patch.clone().unwrap();
            biome(c, &plain).terrain.patch = Some(PatchData {
                threshold: 1.5,
                ..patch
            })
        },
        &format!(
            "{}: soglia tra 0 e 1 (esclusa), trovato 1.5",
            at(&plain, "terrain.patch.threshold")
        ),
    );
    // con soglia 1 nessuna macchia: le regole sulle macchie non varrebbero mai
    assert_error(
        |c| biome(c, &plain).terrain.patch.as_mut().unwrap().threshold = 1.0,
        &format!(
            "{}: soglia tra 0 e 1 (esclusa), trovato 1",
            at(&plain, "terrain.patch.threshold")
        ),
    );
    assert_error(
        |c| biome(c, &plain).terrain.patch.as_mut().unwrap().cell = 0,
        &format!(
            "{}: lato della cella tra 1 e 64",
            at(&plain, "terrain.patch.cell")
        ),
    );
    assert_error(
        |c| biome(c, &plain).terrain.ground[1].1 = -1.0,
        &format!(
            "{}: i pesi devono essere >= 0",
            at(&plain, "terrain.ground")
        ),
    );
    assert_error(
        |c| {
            for (_, weight) in &mut biome(c, &plain).terrain.ground {
                *weight = 0.0;
            }
        },
        &format!("{}: serve almeno un peso > 0", at(&plain, "terrain.ground")),
    );
}

/// Varianti e gruppi: soglie valide e in ordine, mai un tile che sembra un altro materiale.
#[test]
fn variants_and_clusters_are_checked() {
    let (plain, _) = ids();
    let c = embedded();
    let terrain = &c.find_biome(&plain).unwrap().terrain;
    let border = terrain.border.clone();
    let patch = terrain.patch.as_ref().unwrap().tile.clone();
    let base = terrain.ground[0].0.clone();
    let (from, to, below) = terrain.variants[0].clone();
    assert_error(
        |c| biome(c, &plain).terrain.variants[0] = (patch.clone(), base.clone(), 0.1),
        &format!("\"{patch}\" non può sembrare il terreno di base \"{base}\""),
    );
    assert_error(
        |c| biome(c, &plain).terrain.variants[0].2 = 1.5,
        &format!(
            "{}: deve essere tra 0 e 1",
            at(&plain, "terrain.variants[0]")
        ),
    );
    assert_error(
        |c| {
            let t = &mut biome(c, &plain).terrain;
            t.variants.push((from.clone(), to.clone(), below / 2.0));
        },
        "le soglie dello stesso tile vanno in ordine crescente",
    );
    assert_error(
        |c| biome(c, &plain).terrain.variants[0].1 = border.clone(),
        &format!("\"{border}\" è un muro, una macchia o un bordo"),
    );
    assert_error(
        |c| biome(c, &plain).terrain.clusters.as_mut().unwrap().tiles[0] = border.clone(),
        &format!(
            "{}: \"{border}\" è un muro",
            at(&plain, "terrain.clusters.tiles")
        ),
    );
    assert_error(
        |c| {
            biome(c, &plain)
                .terrain
                .clusters
                .as_mut()
                .unwrap()
                .tiles
                .clear()
        },
        &format!(
            "{}: serve almeno un tile",
            at(&plain, "terrain.clusters.tiles")
        ),
    );
    assert_error(
        |c| biome(c, &plain).terrain.clusters.as_mut().unwrap().cell = 99,
        &at(&plain, "terrain.clusters.cell"),
    );
    assert_error(
        |c| {
            biome(c, &plain)
                .terrain
                .clusters
                .as_mut()
                .unwrap()
                .threshold = 1.0
        },
        &format!(
            "{}: soglia tra 0 e 1",
            at(&plain, "terrain.clusters.threshold")
        ),
    );
    assert_error(
        |c| biome(c, &plain).terrain.clusters.as_mut().unwrap().share = 0.0,
        &format!("{}: deve essere > 0", at(&plain, "terrain.clusters.share")),
    );
}

#[test]
fn decorations_are_checked() {
    let (_, decorated) = ids();
    let c = embedded();
    let first = c
        .find_biome(&decorated)
        .unwrap()
        .decorations
        .as_ref()
        .unwrap()
        .tiles[0]
        .clone();
    fn rules<'a>(c: &'a mut GameConfig, id: &str) -> &'a mut Vec<DecorRuleData> {
        &mut biome(c, id).decorations.as_mut().unwrap().rules
    }
    assert_error(
        |c| (biome(c, &decorated).decorations.as_mut().unwrap().tiles).push(first.clone()),
        &format!(
            "{}: \"{first}\" duplicato",
            at(&decorated, "decorations.tiles")
        ),
    );
    assert_error(
        |c| biome(c, &decorated).decorations.as_mut().unwrap().tileset = "decor.gif".into(),
        &format!("{} = \"decor.gif\"", at(&decorated, "decorations.tileset")),
    );
    assert_error(
        |c| rules(c, &decorated)[0].density = 0.0,
        &format!(
            "{}: deve essere > 0",
            at(&decorated, "decorations.rules[0].density")
        ),
    );
    assert_error(
        |c| rules(c, &decorated)[0].cluster = Some((4, 1.0)),
        &format!(
            "{}: soglia tra 0 e 1",
            at(&decorated, "decorations.rules[0].cluster")
        ),
    );
    assert_error(
        |c| rules(c, &decorated)[0].tiles = vec!["nessuno".into()],
        &format!(
            "{}: \"nessuno\"",
            at(&decorated, "decorations.rules[0].tiles")
        ),
    );
    assert_error(
        |c| {
            biome(c, &decorated).terrain.patch = None;
            rules(c, &decorated).push(DecorRuleData {
                tiles: vec![first.clone()],
                on: Surface::Patch,
                density: 0.1,
                cluster: None,
            });
        },
        "il terreno non ha macchie",
    );
}

#[test]
fn obstacle_kinds_keep_the_structural_guarantees() {
    let (plain, decorated) = ids();
    let c = embedded();
    let kind = c.find_biome(&plain).unwrap().obstacles[0].id.clone();
    let other = c.find_biome(&decorated).unwrap().obstacles[0].id.clone();
    assert_error(
        |c| biome(c, &plain).obstacles[0].footprint_radius = 9.0,
        &format!(
            "{}: tra 4 e 7 px",
            at(&plain, &format!("obstacles.{kind}.footprint_radius"))
        ),
    );
    assert_error(
        |c| biome(c, &decorated).obstacles[0].footprint_radius = 2.0,
        &format!("obstacles.{other}.footprint_radius: tra 4 e 7 px"),
    );
    assert_error(
        |c| {
            for kind in &mut biome(c, &decorated).obstacles {
                kind.ground = 0.0;
            }
        },
        &format!(
            "{}: serve un tipo con peso sul terreno di base",
            at(&decorated, "obstacles")
        ),
    );
    assert_error(
        |c| biome(c, &decorated).obstacles[1].patch = -1.0,
        &format!(
            "{}: i pesi devono essere >= 0",
            at(&decorated, "obstacles.patch")
        ),
    );
    assert_error(
        |c| {
            let kinds = &mut biome(c, &decorated).obstacles;
            kinds[1].id = kinds[0].id.clone();
        },
        &format!("{}: \"{other}\" duplicato", at(&decorated, "obstacles")),
    );
    assert_error(
        |c| biome(c, &decorated).obstacles[1].sprite = "sprites/x.gif".into(),
        "serve un percorso .png relativo ad assets/",
    );
    assert_error(
        |c| biome(c, &decorated).terrain.tileset = "../t.png".into(),
        &at(&decorated, "terrain.tileset"),
    );
    // radure senza ostacoli: tutti i pesi sulle macchie a 0 vanno bene
    let mut open = embedded();
    for kind in &mut biome(&mut open, &decorated).obstacles {
        kind.patch = 0.0;
    }
    assert!(super::super::validate(&open).is_empty());
}
