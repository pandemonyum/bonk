//! Rendering dell'arena (solo client): il terreno in pezzi da 32×32 tile (512 px), "cotti" sulla
//! CPU in un'immagine normale dal tileset e mostrati come sprite, uno per pezzo e solo attorno
//! alla camera (fino a 6 pronti, gli altri tolti). Uguale su desktop e web: il `TilemapChunk` di
//! Bevy legge gli indici da una texture `Rgba16Uint` che WebGL2 rifiuta ("texSubImage2D:
//! invalid format") e nel browser l'erba non si vedeva.
//!
//! Lo stesso codice per ogni bioma (D-068): i tileset vengono dal `Biome` della run e le
//! decorazioni calpestabili si cuociono sopra il terreno, tranne sotto gli ostacoli.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::biome::Biome;
use super::generation::{decor_at, visual_tile_at};
use super::obstacles::ObstacleField;
use super::view::view_rect;
use super::{ARENA_HALF_EXTENT, ARENA_TILES, TILE_SIZE, WorldSetup};
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

/// I tileset del bioma (colonne di tile 16×16, il terreno opaco e le decorazioni con alpha) e
/// gli sprite dei suoi ostacoli, tenuti caricati per tutta la run: con F5 non spariscono.
#[derive(Resource)]
struct GroundArt {
    terrain: Handle<Image>,
    decorations: Option<Handle<Image>>,
    _obstacles: Vec<Handle<Image>>,
}

/// Pixel di una colonna di tile: pronti, ancora in arrivo, o inutilizzabili (file mancante o
/// misura diversa dai dati).
enum Column {
    Ready(Vec<u8>),
    Loading,
    Broken(String),
}

/// Sprite di un pezzo di terreno (indice del pezzo).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct GroundPiece(UVec2);

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), load_ground_art.after(WorldSetup))
            .add_systems(
                Update,
                stream_ground
                    .run_if(in_state(AppState::InRun))
                    .run_if(resource_exists::<GroundArt>),
            );
    }
}

fn load_ground_art(mut commands: Commands, assets: Res<AssetServer>, biome: Res<Biome>) {
    commands.insert_resource(GroundArt {
        terrain: assets.load(biome.terrain.tileset.clone()),
        decorations: (biome.decorations.as_ref()).map(|d| assets.load(d.tileset.clone())),
        _obstacles: (biome.obstacles.iter())
            .map(|kind| assets.load(kind.sprite.clone()))
            .collect(),
    });
}

/// Pixel RGBA di un tileset a colonna di `count` tile 16×16.
fn column_pixels(
    images: &Assets<Image>,
    assets: &AssetServer,
    handle: &Handle<Image>,
    count: u32,
) -> Column {
    let path = || handle.path().map_or(String::new(), |p| p.to_string());
    let Some(image) = images.get(handle) else {
        return if assets.load_state(handle).is_failed() {
            Column::Broken(format!("{} non si carica", path()))
        } else {
            Column::Loading
        };
    };
    let expected = (TILE_SIZE, TILE_SIZE * count);
    match image.data.clone() {
        Some(pixels) if (image.width(), image.height()) == expected => Column::Ready(pixels),
        _ => Column::Broken(format!(
            "{} è {}×{}, i dati del bioma chiedono {}×{} ({count} tile)",
            path(),
            image.width(),
            image.height(),
            expected.0,
            expected.1
        )),
    }
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

/// Cosa serve per cuocere un pezzo: il bioma, i pixel RGBA delle colonne di tile (16 px di
/// larghezza, una tile ogni 16 righe) e gli ostacoli, che coprono le decorazioni.
pub struct PieceArt<'a> {
    pub biome: &'a Biome,
    pub terrain: &'a [u8],
    pub decorations: Option<&'a [u8]>,
    pub obstacles: Option<&'a ObstacleField>,
}

/// I pixel RGBA di un pezzo (riga 0 in alto): il terreno e sopra le decorazioni, copiate senza
/// fusione (hanno alpha 0 o 255). Le tile vanno dal basso verso l'alto come le y del mondo.
pub fn bake_piece(seed: u64, piece: UVec2, art: &PieceArt) -> Vec<u8> {
    let tile = TILE_SIZE as usize;
    let side = PIECE_PX as usize;
    let tile_row = tile * RGBA;
    let mut out = vec![0u8; side * side * RGBA];
    let origin = piece * PIECE_TILES;
    for ty in 0..PIECE_TILES {
        // la riga di tile più bassa del pezzo sta in fondo all'immagine
        let top = (PIECE_TILES - 1 - ty) as usize * tile;
        for tx in 0..PIECE_TILES {
            let (x, y) = (origin.x + tx, origin.y + ty);
            let index = visual_tile_at(&art.biome.terrain, seed, x, y) as usize;
            let left = tx as usize * tile_row;
            for row in 0..tile {
                let src = (index * tile + row) * tile_row;
                let dst = (top + row) * side * RGBA + left;
                if let Some(pixels) = art.terrain.get(src..src + tile_row) {
                    out[dst..dst + tile_row].copy_from_slice(pixels);
                }
            }
            let covered = art.obstacles.is_some_and(|field| field.occupies(x, y));
            let decorations = art.decorations.filter(|_| !covered);
            let Some((pixels, index)) = decorations.zip(decor_at(art.biome, seed, x, y)) else {
                continue;
            };
            for row in 0..tile {
                let src = (index as usize * tile + row) * tile_row;
                let dst = (top + row) * side * RGBA + left;
                let Some(source) = pixels.get(src..src + tile_row) else {
                    continue;
                };
                let (target, _) = out[dst..dst + tile_row].as_chunks_mut::<RGBA>();
                let (source, _) = source.as_chunks::<RGBA>();
                for (to, from) in target.iter_mut().zip(source) {
                    if from[3] != 0 {
                        *to = *from;
                    }
                }
            }
        }
    }
    out
}

/// Prepara i pezzi attorno alla vista. Senza tileset del terreno non si cuoce nulla; senza
/// decorazioni (file rotto o di misura diversa dai dati) si cuoce il terreno senza, con un
/// avviso: meglio un bosco spoglio di un fondo nero.
#[allow(clippy::too_many_arguments)]
fn stream_ground(
    mut commands: Commands,
    seed: Res<RunSeed>,
    biome: Res<Biome>,
    art: Res<GroundArt>,
    field: Option<Res<ObstacleField>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut warned: Local<Vec<String>>,
    player: Query<&Position, With<Player>>,
    pieces: Query<(Entity, &GroundPiece)>,
) {
    let mut warn_once = |problem: String| {
        if !warned.contains(&problem) {
            warn!("terreno del bioma {}: {problem}", biome.id);
            warned.push(problem);
        }
    };
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
    let terrain = match column_pixels(&images, &assets, &art.terrain, biome.terrain.tile_count) {
        Column::Ready(pixels) => pixels,
        Column::Loading => return,
        Column::Broken(problem) => return warn_once(problem),
    };
    let decorations = match (&art.decorations, &biome.decorations) {
        (Some(handle), Some(d)) => match column_pixels(&images, &assets, handle, d.tile_count) {
            Column::Ready(pixels) => Some(pixels),
            // le decorazioni arrivano insieme al terreno
            Column::Loading => return,
            Column::Broken(problem) => {
                warn_once(format!("{problem}: niente decorazioni"));
                None
            }
        },
        _ => None,
    };
    let piece_art = PieceArt {
        biome: &biome,
        terrain: &terrain,
        decorations: decorations.as_deref(),
        obstacles: field.as_deref(),
    };
    for piece in missing {
        let image = Image::new(
            Extent3d {
                width: PIECE_PX,
                height: PIECE_PX,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bake_piece(seed.0, piece, &piece_art),
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
mod tests;
