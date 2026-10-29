//! Rendering dell'arena (solo client): il terreno in pezzi da 32×32 tile (512 px), "cotti" sulla
//! CPU in un'immagine normale dal tileset e mostrati come sprite, uno per pezzo e solo attorno
//! alla camera (2–4 a schermo, gli altri tolti). Uguale su desktop e web: il `TilemapChunk` di
//! Bevy legge gli indici da una texture `Rgba16Uint` che WebGL2 rifiuta ("texSubImage2D:
//! invalid format") e nel browser l'erba non si vedeva.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::generation::visual_tile_at;
use super::tiles::{TILE_COUNT, TILESET_PATH};
use super::view::view_rect;
use super::{ARENA_HALF_EXTENT, ARENA_TILES, TILE_SIZE};
use crate::app::{AppState, RunSeed};
use crate::gameplay::kinematics::Position;
use crate::gameplay::player::Player;
use crate::presentation::Z_GROUND;

/// Lato di un pezzo in tile e in pixel, pezzi per lato dell'arena.
pub const PIECE_TILES: u32 = 32;
pub const PIECE_PX: u32 = PIECE_TILES * TILE_SIZE;
pub const PIECES: u32 = ARENA_TILES / PIECE_TILES;
/// Un pezzo si prepara quando la vista gli arriva a questa distanza (px), prima che si veda.
const MARGIN: f32 = 192.0;
/// Pezzi cotti al massimo per fotogramma (all'ingresso nella run ne servono 2–4).
const BAKES_PER_FRAME: usize = 2;
/// Byte per pixel (RGBA a 8 bit).
const RGBA: usize = 4;

/// Il tileset: una colonna di `TILE_COUNT` tile 16×16.
#[derive(Resource)]
struct Tileset(Handle<Image>);

/// Sprite di un pezzo di terreno (indice del pezzo).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct GroundPiece(UVec2);

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), load_tileset)
            .add_systems(
                Update,
                stream_ground
                    .run_if(in_state(AppState::InRun))
                    .run_if(resource_exists::<Tileset>),
            );
    }
}

fn load_tileset(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Tileset(assets.load(TILESET_PATH)));
}

/// Centro di un pezzo in coordinate di mondo (y in alto, il pezzo (0, 0) in basso a sinistra).
pub fn piece_center(piece: UVec2) -> Vec2 {
    let side = PIECE_PX as f32;
    piece.as_vec2() * side + Vec2::splat(side / 2.0 - ARENA_HALF_EXTENT)
}

/// I pezzi che toccano la vista `view` allargata di `MARGIN`.
pub fn pieces_near(view: Rect) -> Vec<UVec2> {
    let side = PIECE_PX as f32;
    let area = view.inflate(MARGIN);
    let to_piece = |v: f32| ((v + ARENA_HALF_EXTENT) / side).floor();
    let clamp = |v: f32| v.clamp(0.0, (PIECES - 1) as f32) as u32;
    let (x0, x1) = (clamp(to_piece(area.min.x)), clamp(to_piece(area.max.x)));
    let (y0, y1) = (clamp(to_piece(area.min.y)), clamp(to_piece(area.max.y)));
    (y0..=y1)
        .flat_map(|y| (x0..=x1).map(move |x| UVec2::new(x, y)))
        .collect()
}

/// I pixel RGBA di un pezzo (riga 0 in alto), dal tileset in colonna `tileset` (16 px di
/// larghezza, una tile ogni 16 righe). Le tile vanno dal basso verso l'alto come le y del mondo.
pub fn bake_piece(seed: u64, piece: UVec2, tileset: &[u8]) -> Vec<u8> {
    let tile = TILE_SIZE as usize;
    let side = PIECE_PX as usize;
    let tile_row = tile * RGBA;
    let mut out = vec![0u8; side * side * RGBA];
    let origin = piece * PIECE_TILES;
    for ty in 0..PIECE_TILES {
        // la riga di tile più bassa del pezzo sta in fondo all'immagine
        let top = (PIECE_TILES - 1 - ty) as usize * tile;
        for tx in 0..PIECE_TILES {
            let index = visual_tile_at(seed, origin.x + tx, origin.y + ty) as usize;
            let left = tx as usize * tile_row;
            for row in 0..tile {
                let src = (index * tile + row) * tile_row;
                let dst = (top + row) * side * RGBA + left;
                if let Some(pixels) = tileset.get(src..src + tile_row) {
                    out[dst..dst + tile_row].copy_from_slice(pixels);
                }
            }
        }
    }
    out
}

fn stream_ground(
    mut commands: Commands,
    seed: Res<RunSeed>,
    tileset: Res<Tileset>,
    mut images: ResMut<Assets<Image>>,
    player: Query<&Position, With<Player>>,
    pieces: Query<(Entity, &GroundPiece)>,
) {
    let Ok(me) = player.single() else {
        return;
    };
    let wanted = pieces_near(view_rect(me.0));
    for (entity, piece) in &pieces {
        if !wanted.contains(&piece.0) {
            commands.entity(entity).despawn();
        }
    }
    let present: Vec<UVec2> = pieces.iter().map(|(_, p)| p.0).collect();
    let missing: Vec<UVec2> = wanted
        .into_iter()
        .filter(|p| !present.contains(p))
        .take(BAKES_PER_FRAME)
        .collect();
    if missing.is_empty() {
        return;
    }
    let Some(pixels) = images
        .get(&tileset.0)
        .filter(|image| image.width() == TILE_SIZE && image.height() == TILE_SIZE * TILE_COUNT)
        .and_then(|image| image.data.clone())
    else {
        return; // tileset non ancora caricato
    };
    for piece in missing {
        let image = Image::new(
            Extent3d {
                width: PIECE_PX,
                height: PIECE_PX,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bake_piece(seed.0, piece, &pixels),
            TextureFormat::Rgba8UnormSrgb,
            // sulla GPU e basta: i pixel sulla CPU non servono più
            RenderAssetUsages::RENDER_WORLD,
        );
        commands.spawn((
            Name::new(format!("Terreno {},{}", piece.x, piece.y)),
            GroundPiece(piece),
            Sprite::from_image(images.add(image)),
            Transform::from_translation(piece_center(piece).extend(Z_GROUND)),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // al centro dell'arena la vista tocca i quattro pezzi attorno all'incrocio
        let near = pieces_near(view_rect(Vec2::ZERO));
        assert_eq!(near.len(), 4, "{near:?}");
        // in un angolo, solo pezzi dentro l'arena
        let corner = pieces_near(view_rect(Vec2::splat(-ARENA_HALF_EXTENT)));
        assert!(corner.iter().all(|p| p.x < PIECES && p.y < PIECES));
        assert!(corner.contains(&UVec2::ZERO));
    }

    #[test]
    fn a_baked_piece_puts_each_tile_in_its_place() {
        // tileset finto: la tile i ha tutti i pixel al valore i
        let tile = (TILE_SIZE * TILE_SIZE) as usize * RGBA;
        let tileset: Vec<u8> = (0..TILE_COUNT as usize)
            .flat_map(|i| std::iter::repeat_n(i as u8, tile))
            .collect();
        let seed = 7;
        let piece = UVec2::new(3, 5);
        let pixels = bake_piece(seed, piece, &tileset);
        let side = PIECE_PX as usize;
        let at = |px: usize, py: usize| pixels[(py * side + px) * RGBA];
        let origin = piece * PIECE_TILES;
        // in alto a sinistra la tile più alta del pezzo, in basso a sinistra la più bassa
        assert_eq!(
            at(0, 0) as u16,
            visual_tile_at(seed, origin.x, origin.y + PIECE_TILES - 1)
        );
        assert_eq!(
            at(0, side - 1) as u16,
            visual_tile_at(seed, origin.x, origin.y)
        );
        assert_eq!(
            at(side - 1, side - 1) as u16,
            visual_tile_at(seed, origin.x + PIECE_TILES - 1, origin.y)
        );
    }
}
