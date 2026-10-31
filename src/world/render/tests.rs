use super::*;
use crate::data::GameConfig;
use crate::world::obstacles::generate_obstacles;

fn biome(stage: &str) -> (Biome, GameConfig) {
    let config = GameConfig::embedded().with_stage(stage).unwrap();
    (Biome::of_stage(&config), config)
}

/// Colonna finta di `count` tile: la tile i ha tutti i pixel al valore `base + i` e alpha 255.
fn column(count: u32, base: u8) -> Vec<u8> {
    let tile = (TILE_SIZE * TILE_SIZE) as usize;
    (0..count as u8)
        .flat_map(|i| std::iter::repeat_n([base + i, base + i, base + i, 255], tile))
        .flatten()
        .collect()
}

#[test]
fn pieces_tile_the_arena_and_follow_the_view() {
    assert_eq!(PIECES * PIECE_PX, ARENA_TILES * TILE_SIZE);
    // il primo pezzo in basso a sinistra, l'ultimo in alto a destra
    let side = PIECE_PX as f32;
    assert_eq!(
        piece_center(UVec2::ZERO),
        Vec2::splat(side / 2.0 - ARENA_HALF_EXTENT)
    );
    assert_eq!(
        piece_center(UVec2::splat(PIECES - 1)),
        Vec2::splat(ARENA_HALF_EXTENT - side / 2.0)
    );
    // al centro dell'arena la vista più grande (640×360 più il margine) tocca 3×2 pezzi
    let near = pieces_near(view_rect(Vec2::ZERO));
    assert_eq!(near.len(), 6, "{near:?}");
    // in un angolo, solo pezzi dentro l'arena
    let corner = pieces_near(view_rect(Vec2::splat(-ARENA_HALF_EXTENT)));
    assert!(corner.iter().all(|p| p.x < PIECES && p.y < PIECES));
    assert!(corner.contains(&UVec2::ZERO));
}

#[test]
fn a_baked_piece_puts_each_tile_in_its_place() {
    let (biome, _) = biome("meadow");
    let terrain = column(biome.terrain.tile_count, 0);
    let art = PieceArt {
        biome: &biome,
        terrain: &terrain,
        decorations: None,
        obstacles: None,
    };
    let seed = 7;
    let piece = UVec2::new(3, 5);
    let pixels = bake_piece(seed, piece, &art);
    let side = PIECE_PX as usize;
    let at = |px: usize, py: usize| pixels[(py * side + px) * RGBA];
    let origin = piece * PIECE_TILES;
    let t = &biome.terrain;
    // in alto a sinistra la tile più alta del pezzo, in basso a sinistra la più bassa
    assert_eq!(
        at(0, 0) as u16,
        visual_tile_at(t, seed, origin.x, origin.y + PIECE_TILES - 1)
    );
    assert_eq!(
        at(0, side - 1) as u16,
        visual_tile_at(t, seed, origin.x, origin.y)
    );
    assert_eq!(
        at(side - 1, side - 1) as u16,
        visual_tile_at(t, seed, origin.x + PIECE_TILES - 1, origin.y)
    );
}

/// Le decorazioni coprono il terreno solo dove i loro pixel sono opachi, mai sotto un
/// ostacolo; i tile senza decorazione restano terreno.
#[test]
fn decorations_are_baked_over_the_ground_except_under_obstacles() {
    let (biome, config) = biome("forest");
    let decor = biome
        .decorations
        .as_ref()
        .expect("il bosco ha le decorazioni");
    let terrain = column(biome.terrain.tile_count, 0);
    // decorazioni finte: pixel pari opachi a 200 + i, dispari trasparenti
    let tile = (TILE_SIZE * TILE_SIZE) as usize;
    let decorations: Vec<u8> = (0..decor.tile_count as u8)
        .flat_map(|i| (0..tile).map(move |p| [200 + i, 0, 0, if p % 2 == 0 { 255 } else { 0 }]))
        .flatten()
        .collect();
    let seed = 3;
    let field = ObstacleField::new(generate_obstacles(seed, &biome, &config.stage.obstacles));
    let art = PieceArt {
        biome: &biome,
        terrain: &terrain,
        decorations: Some(&decorations),
        obstacles: Some(&field),
    };
    let side = PIECE_PX as usize;
    let (mut decorated, mut covered) = (0, 0);
    // una fila intera di pezzi: abbastanza ostacoli per trovarne su una decorazione
    for piece in (0..PIECES).map(|x| UVec2::new(x, 2)) {
        let pixels = bake_piece(seed, piece, &art);
        let origin = piece * PIECE_TILES;
        for ty in 0..PIECE_TILES {
            for tx in 0..PIECE_TILES {
                let (x, y) = (origin.x + tx, origin.y + ty);
                // primi due pixel della riga in alto del tile: uno opaco e uno trasparente
                let top = (PIECE_TILES - 1 - ty) as usize * TILE_SIZE as usize;
                let left = tx as usize * TILE_SIZE as usize;
                let px = |dx: usize| pixels[(top * side + left + dx) * RGBA];
                let ground = visual_tile_at(&biome.terrain, seed, x, y) as u8;
                assert_eq!(px(1), ground, "pixel trasparente in {x},{y}");
                match decor_at(&biome, seed, x, y) {
                    Some(d) if !field.occupies(x, y) => {
                        assert_eq!(px(0), 200 + d as u8, "{x},{y}");
                        decorated += 1;
                    }
                    Some(_) => {
                        assert_eq!(px(0), ground, "decorazione sotto un ostacolo in {x},{y}");
                        covered += 1;
                    }
                    None => assert_eq!(px(0), ground),
                }
            }
        }
    }
    assert!(decorated > 0, "nessuna decorazione: test vacuo");
    assert!(
        covered > 0,
        "nessuna decorazione sotto un ostacolo: test vacuo"
    );
}
