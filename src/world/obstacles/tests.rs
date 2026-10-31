use std::collections::VecDeque;

use super::*;
use crate::data::GameConfig;
use crate::world::ArenaBounds;

/// Ogni stage di `stages.ron` come stage giocato: id, bioma compilato e parametri.
fn stages() -> Vec<(String, Biome, ObstacleParams)> {
    let config = GameConfig::embedded();
    (config.stages.iter())
        .map(|stage| {
            let played = config.clone().with_stage(&stage.id).unwrap();
            (stage.id.clone(), Biome::of_stage(&played), stage.obstacles)
        })
        .collect()
}

fn meadow() -> (Biome, ObstacleParams) {
    let config = GameConfig::embedded().with_stage("meadow").unwrap();
    (Biome::of_stage(&config), config.stage.obstacles)
}

/// D-068: la prateria ha gli stessi ostacoli di prima, nello stesso ordine. Le impronte sono
/// state calcolate prima della modifica, con i tipi di allora (roccia 0, albero 1).
#[test]
fn the_meadow_obstacles_are_identical_to_before_the_biomes() {
    let (biome, params) = meadow();
    let old_kind = |kind: u16| match biome.obstacles[kind as usize].id.as_str() {
        "rock" => 0,
        "tree" => 1,
        other => panic!("tipo nuovo nella prateria: {other}"),
    };
    for (seed, count, expected) in [
        (1, 704, 0x2dc0_c277_82e8_7172),
        (7, 734, 0xcbcc_f37b_562f_fd37),
        (1_592_598_566, 705, 0xcc90_16f3_fff5_4ee6),
    ] {
        let obstacles = generate_obstacles(seed, &biome, &params);
        assert_eq!(obstacles.len(), count, "seed {seed}");
        let print = obstacles
            .iter()
            .flat_map(|o| {
                [
                    u64::from(o.tile.x) << 32 | u64::from(o.tile.y),
                    old_kind(o.kind),
                    u64::from(o.base.x.to_bits()) << 32 | u64::from(o.base.y.to_bits()),
                ]
            })
            .fold(0xCBF2_9CE4_8422_2325_u64, |acc, v| {
                (acc ^ v).wrapping_mul(0x100_0000_01B3)
            });
        assert_eq!(print, expected, "seed {seed}");
        let radius = |id: &str| biome.obstacles.iter().find(|k| k.id == id).unwrap().radius;
        assert_eq!((radius("rock"), radius("tree")), (7.0, 4.0));
    }
}

#[test]
fn generation_is_deterministic_seed_dependent_and_uses_every_kind() {
    for (id, biome, params) in stages() {
        let a = generate_obstacles(11, &biome, &params);
        assert_eq!(a, generate_obstacles(11, &biome, &params), "{id}");
        assert_ne!(a, generate_obstacles(12, &biome, &params), "{id}");
        // densità 1–2% su ~61 mila tile candidati, meno gli scarti per vicinanza
        assert!((300..1500).contains(&a.len()), "{id}: {} ostacoli", a.len());
        for (kind, rule) in biome.obstacles.iter().enumerate() {
            let used = a.iter().any(|o| usize::from(o.kind) == kind);
            assert!(used, "{id}: nessun ostacolo di tipo {}", rule.id);
            let mut radii = a.iter().filter(|o| usize::from(o.kind) == kind);
            assert!(radii.all(|o| o.radius == rule.radius));
        }
    }
}

#[test]
fn parameters_and_weights_change_the_field() {
    let config = GameConfig::embedded();
    let (biome, params) = meadow();
    let none = ObstacleParams {
        density: 0.0,
        ..params
    };
    assert!(generate_obstacles(1, &biome, &none).is_empty());
    // bioma con le sole rocce: niente alberi
    let mut data = config.find_biome("meadow").unwrap().clone();
    for kind in &mut data.obstacles {
        if kind.id != "rock" {
            kind.ground = 0.0;
            kind.patch = 0.0;
        }
    }
    let rocks = Biome::compile(&data).unwrap();
    let field = generate_obstacles(1, &rocks, &params);
    let rock = rocks.obstacles.iter().position(|k| k.id == "rock").unwrap() as u16;
    assert!(!field.is_empty() && field.iter().all(|o| o.kind == rock));
    // un tipo senza peso sulle macchie non ci finisce mai
    let mut data = config.find_biome("meadow").unwrap().clone();
    for kind in &mut data.obstacles {
        kind.patch = 0.0;
    }
    let open_patches = Biome::compile(&data).unwrap();
    for o in generate_obstacles(1, &open_patches, &params) {
        let floor =
            crate::world::generation::floor_at(&open_patches.terrain, 1, o.tile.x, o.tile.y);
        assert_eq!(floor, crate::world::biome::Floor::Ground, "{:?}", o.tile);
    }
}

#[test]
fn start_area_is_free() {
    for (id, biome, params) in stages() {
        for seed in 0..20 {
            for o in generate_obstacles(seed, &biome, &params) {
                let distance = o.footprint_center().length();
                assert!(distance >= params.safe_radius - 16.0, "{id}, seed {seed}");
            }
        }
    }
}

#[test]
fn footprints_stay_within_the_structural_limits() {
    for (id, biome, _) in stages() {
        for kind in &biome.obstacles {
            let radius = kind.radius;
            assert!(
                (MIN_FOOTPRINT_RADIUS..=MAX_FOOTPRINT_RADIUS).contains(&radius),
                "{id}: {} con raggio {radius}",
                kind.id
            );
        }
    }
}

/// Con due tile di margine dal muro, la zona da cui un ostacolo respinge il corpo più largo
/// ammesso sta tutta dentro i limiti dei piedi: il clamp non può riportarvi un corpo.
#[test]
fn obstacle_exclusion_zones_stay_inside_the_walkable_area() {
    let arena = ArenaBounds::default();
    let widest_body = MIN_OBSTACLE_GAP / 2.0;
    for (id, biome, params) in stages() {
        for seed in 0..10 {
            for o in generate_obstacles(seed, &biome, &params) {
                let reach = Vec2::splat(o.footprint_radius() + widest_body);
                let (lo, hi) = (o.footprint_center() - reach, o.footprint_center() + reach);
                assert!(
                    arena.contains(lo) && arena.contains(hi),
                    "{id}, seed {seed}, tile {:?}",
                    o.tile
                );
            }
        }
    }
}

#[test]
fn gaps_between_obstacles_are_never_below_the_guaranteed_minimum() {
    assert_eq!(MIN_OBSTACLE_GAP, 18.0);
    for (id, biome, params) in stages() {
        let obstacles = generate_obstacles(3, &biome, &params);
        let field = ObstacleField::new(obstacles.clone());
        for a in &obstacles {
            for b in field.near(a.footprint_center(), 40.0) {
                if a == b {
                    continue;
                }
                let gap = a.footprint_center().distance(b.footprint_center())
                    - a.footprint_radius()
                    - b.footprint_radius();
                assert!(gap >= MIN_OBSTACLE_GAP - 1e-3, "{id}: varco di {gap} px");
            }
        }
    }
}

#[test]
fn every_free_tile_is_reachable_from_the_start() {
    let size = ARENA_TILES as usize;
    for (id, biome, params) in stages() {
        let mut blocked = vec![false; size * size];
        for o in generate_obstacles(5, &biome, &params) {
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
        assert_eq!(reached, free, "{id}: tile liberi irraggiungibili");
    }
}

#[test]
fn the_field_knows_which_tiles_are_occupied() {
    for (id, biome, params) in stages() {
        let obstacles = generate_obstacles(4, &biome, &params);
        let field = ObstacleField::new(obstacles.clone());
        let mut occupied = 0;
        for y in 0..ARENA_TILES {
            for x in 0..ARENA_TILES {
                occupied += usize::from(field.occupies(x, y));
            }
        }
        assert_eq!(occupied, obstacles.len(), "{id}");
        assert!(obstacles.iter().all(|o| field.occupies(o.tile.x, o.tile.y)));
        assert!(!field.occupies(ARENA_TILES, 0) && !field.occupies(0, ARENA_TILES));
    }
}

/// Sonde vicino a ostacoli veri: quelli sulle righe e colonne estreme (bordi della griglia)
/// e uno centrale, con uno scarto che fa cadere la sonda in un'altra cella.
#[test]
fn grid_query_finds_exactly_the_nearby_obstacles() {
    for (id, biome, params) in stages() {
        let obstacles = generate_obstacles(9, &biome, &params);
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
            assert!(
                !brute.is_empty(),
                "{id}: sonda senza ostacoli vicini, test vacuo"
            );
            brute.sort_by_key(|o| (o.tile.y, o.tile.x));
            grid.sort_by_key(|o| (o.tile.y, o.tile.x));
            assert_eq!(brute, grid, "{id}");
        }
    }
}
