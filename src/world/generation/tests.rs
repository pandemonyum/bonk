use super::*;
use crate::data::GameConfig;
use crate::world::ARENA_CHUNKS;
use crate::world::biome::EDGE_TILES;

/// Tutti i biomi di `stages.ron`, compilati.
fn biomes() -> Vec<Biome> {
    let config = GameConfig::embedded();
    assert!(config.biomes.len() >= 2, "servono almeno prateria e bosco");
    config
        .biomes
        .iter()
        .map(|b| Biome::compile(b).unwrap())
        .collect()
}

fn biome(id: &str) -> Biome {
    biomes().into_iter().find(|b| b.id == id).unwrap()
}

/// FNV-1a a 64 bit: impronta stabile di una sequenza di valori.
fn fingerprint(values: impl IntoIterator<Item = u64>) -> u64 {
    values.into_iter().fold(0xCBF2_9CE4_8422_2325, |acc, v| {
        (acc ^ v).wrapping_mul(0x100_0000_01B3)
    })
}

/// D-068: la prateria disegnata dal generatore comune è quella di prima, tile per tile. Le
/// impronte sono state calcolate con `visual_tile_at` della prateria prima della modifica.
#[test]
fn the_meadow_is_identical_to_before_the_biomes() {
    let meadow = biome("meadow");
    for (seed, expected) in [
        (1, 0xeeae_6eac_a954_37fc),
        (7, 0xb5f4_1a60_fe69_a6fa),
        (1_592_598_566, 0xda3d_30e2_74bf_1e69),
    ] {
        let tiles = (0..ARENA_TILES).flat_map(|y| (0..ARENA_TILES).map(move |x| (x, y)));
        let print =
            fingerprint(tiles.map(|(x, y)| u64::from(visual_tile_at(&meadow.terrain, seed, x, y))));
        assert_eq!(print, expected, "seed {seed}");
    }
    assert!(meadow.decorations.is_none());
}

#[test]
fn same_seed_same_map_different_seed_different_map() {
    for biome in biomes() {
        let t = &biome.terrain;
        let a = chunk_tiles(t, 1, UVec2::new(1, 2));
        assert_eq!(a, chunk_tiles(t, 1, UVec2::new(1, 2)), "{}", biome.id);
        assert_ne!(a, chunk_tiles(t, 2, UVec2::new(1, 2)), "{}", biome.id);
        assert_eq!(a.len(), (CHUNK_TILES * CHUNK_TILES) as usize);
        let decor = |seed| -> Vec<Option<u16>> {
            (0..64)
                .map(|i| decor_at(&biome, seed, 40 + i, 90))
                .collect()
        };
        assert_eq!(decor(1), decor(1), "{}", biome.id);
    }
}

#[test]
fn the_arena_is_closed_by_the_border_and_indices_are_valid() {
    for biome in biomes() {
        let t = &biome.terrain;
        for i in 0..ARENA_TILES {
            for (x, y) in [(i, 0), (i, ARENA_TILES - 1), (0, i), (ARENA_TILES - 1, i)] {
                assert_eq!(floor_at(t, 7, x, y), Floor::Border);
                assert_eq!(tile_at(t, 7, x, y), t.border.tile);
            }
        }
        for y in (0..ARENA_TILES).step_by(3) {
            for x in 0..ARENA_TILES {
                let visual = visual_tile_at(t, 7, x, y);
                assert!(u32::from(visual) < t.tile_count, "{} {x},{y}", biome.id);
            }
        }
    }
}

/// Dentro il muro c'è ogni tile di base e la base domina: le macchie restano macchie.
#[test]
fn the_ground_dominates_and_uses_every_base_tile() {
    for biome in biomes() {
        let t = &biome.terrain;
        let (mut ground, mut patch) = (0, 0);
        let mut seen = vec![false; t.tile_count as usize];
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                match floor_at(t, 3, x, y) {
                    Floor::Ground => ground += 1,
                    Floor::Patch => patch += 1,
                    Floor::Border => panic!("muro dentro l'arena in {x},{y}"),
                }
                seen[tile_at(t, 3, x, y) as usize] = true;
            }
        }
        for (tile, _) in &t.ground {
            assert!(seen[*tile as usize], "{}: manca il tile {tile}", biome.id);
        }
        if t.patch.is_some() {
            assert!(patch > 0, "{}: nessuna macchia", biome.id);
            assert!(
                patch * 4 < ground,
                "{}: macchie {patch}, base {ground}",
                biome.id
            );
        }
    }
}

/// Terreno logico di un tile grafico: le varianti non devono mai cambiarlo.
fn floor_of(t: &Terrain, visual: u16) -> Floor {
    let is = |m: crate::world::biome::Material| {
        visual == m.tile
            || m.edges
                .is_some_and(|e| (e..e + EDGE_TILES).contains(&visual))
    };
    let variant_of = |m: crate::world::biome::Material| {
        t.variants
            .iter()
            .any(|v| v.from == m.tile && v.to == visual)
    };
    if is(t.border) || variant_of(t.border) {
        Floor::Border
    } else if t
        .patch
        .as_ref()
        .is_some_and(|p| is(p.material) || variant_of(p.material))
    {
        Floor::Patch
    } else {
        Floor::Ground
    }
}

#[test]
fn visual_tiles_keep_the_logic_floor() {
    for biome in biomes() {
        let t = &biome.terrain;
        for y in 0..ARENA_TILES {
            for x in (0..ARENA_TILES).step_by(3) {
                let visual = visual_tile_at(t, 4, x, y);
                assert_eq!(
                    floor_of(t, visual),
                    floor_at(t, 4, x, y),
                    "{} {x},{y}",
                    biome.id
                );
            }
        }
    }
}

/// Ogni tile di macchia mostra il bordo proprio sui lati dove il vicino non è macchia (nord =
/// y+1, la riga in alto dell'immagine del tile).
#[test]
fn patch_edges_face_the_floor_around_them() {
    for biome in biomes() {
        let t = &biome.terrain;
        let Some(edges) = t.patch.as_ref().and_then(|p| p.material.edges) else {
            continue;
        };
        let mut seen = 0;
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                if floor_at(t, 6, x, y) != Floor::Patch {
                    continue;
                }
                let open = |nx: u32, ny: u32| floor_at(t, 6, nx, ny) != Floor::Patch;
                let expected = [
                    (open(x, y + 1), EDGE_N),
                    (open(x + 1, y), EDGE_E),
                    (open(x, y - 1), EDGE_S),
                    (open(x - 1, y), EDGE_W),
                ]
                .into_iter()
                .filter(|(open, _)| *open)
                .fold(0, |mask, (_, bit)| mask | bit);
                if expected != 0 {
                    assert_eq!(visual_tile_at(t, 6, x, y), edges + expected - 1, "{x},{y}");
                    seen += 1;
                }
            }
        }
        assert!(seen > 0, "{}: nessun bordo, test vacuo", biome.id);
    }
}

/// Il muro ha il bordo solo verso l'arena: la fila interna a nord guarda a sud, quella a ovest
/// guarda a est; il muro esterno resta pieno.
#[test]
fn border_edges_face_the_arena_only() {
    for biome in biomes() {
        let t = &biome.terrain;
        let Some(edges) = t.border.edges else {
            continue;
        };
        let (low, high) = (BORDER_TILES - 1, ARENA_TILES - BORDER_TILES);
        let mid = ARENA_TILES / 2;
        assert_eq!(visual_tile_at(t, 1, mid, high), edges + EDGE_S - 1);
        assert_eq!(visual_tile_at(t, 1, low, mid), edges + EDGE_E - 1);
        assert_eq!(visual_tile_at(t, 1, mid, low), edges + EDGE_N - 1);
        assert_eq!(visual_tile_at(t, 1, high, mid), edges + EDGE_W - 1);
        for (x, y) in [(mid, 0), (0, mid), (mid, ARENA_TILES - 1), (0, 0)] {
            assert_eq!(floor_of(t, visual_tile_at(t, 1, x, y)), Floor::Border);
            assert!(!(edges..edges + EDGE_TILES).contains(&visual_tile_at(t, 1, x, y)));
        }
    }
}

#[test]
fn every_variant_and_cluster_tile_appears() {
    for biome in biomes() {
        let t = &biome.terrain;
        let mut seen = vec![false; t.tile_count as usize];
        for y in BORDER_TILES..ARENA_TILES - BORDER_TILES {
            for x in BORDER_TILES..ARENA_TILES - BORDER_TILES {
                seen[visual_tile_at(t, 2, x, y) as usize] = true;
            }
        }
        let clustered = t.clusters.iter().flat_map(|c| c.tiles.iter());
        for tile in t.variants.iter().map(|v| &v.to).chain(clustered) {
            assert!(
                seen[*tile as usize],
                "{}: manca la variante {tile}",
                biome.id
            );
        }
    }
}

/// Le decorazioni stanno dentro il muro, sul terreno giusto, e ogni regola ne mette qualcuna.
#[test]
fn decorations_follow_their_rules() {
    for biome in biomes() {
        let Some(decorations) = &biome.decorations else {
            assert_eq!(decor_at(&biome, 1, 100, 100), None);
            continue;
        };
        let mut seen = vec![false; decorations.tile_count as usize];
        let mut decorated = 0;
        for y in 0..ARENA_TILES {
            for x in 0..ARENA_TILES {
                let Some(tile) = decor_at(&biome, 8, x, y) else {
                    continue;
                };
                assert!(u32::from(tile) < decorations.tile_count);
                assert_ne!(floor_at(&biome.terrain, 8, x, y), Floor::Border, "{x},{y}");
                seen[tile as usize] = true;
                decorated += 1;
            }
        }
        for rule in &decorations.rules {
            assert!(rule.tiles.iter().any(|t| seen[*t as usize]), "{rule:?}");
        }
        // sparse: il terreno si vede ancora
        let area = (ARENA_TILES - 2 * BORDER_TILES).pow(2);
        assert!(
            decorated * 5 < area,
            "{}: {decorated} decorazioni",
            biome.id
        );
    }
}

#[test]
fn chunks_tile_the_arena_symmetrically() {
    assert_eq!(chunk_center(UVec2::ZERO), Vec2::splat(-1536.0));
    assert_eq!(
        chunk_center(UVec2::splat(ARENA_CHUNKS - 1)),
        Vec2::splat(1536.0)
    );
}
