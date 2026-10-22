//! Rendering dell'arena (solo client): un `TilemapChunk` per chunk, una draw call ciascuno.

use bevy::image::{ImageArrayLayout, ImageLoaderSettings};
use bevy::prelude::*;
use bevy::sprite_render::{TileData, TilemapChunk, TilemapChunkTileData};

use super::generation::{chunk_center, chunk_tiles};
use super::tiles::{TILE_COUNT, TILESET_PATH};
use super::{ARENA_CHUNKS, CHUNK_TILES, TILE_SIZE};
use crate::app::{AppState, RunSeed};
use crate::presentation::Z_GROUND;

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_arena_chunks);
    }
}

fn spawn_arena_chunks(mut commands: Commands, assets: Res<AssetServer>, seed: Res<RunSeed>) {
    // Il tileset è una colonna di tile 16×16: la carichiamo come array texture.
    let tileset: Handle<Image> = assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.array_layout = Some(ImageArrayLayout::RowCount { rows: TILE_COUNT });
        })
        .load(TILESET_PATH);

    for cy in 0..ARENA_CHUNKS {
        for cx in 0..ARENA_CHUNKS {
            let chunk = UVec2::new(cx, cy);
            let tiles = chunk_tiles(seed.0, chunk)
                .into_iter()
                .map(|index| Some(TileData::from_tileset_index(index)))
                .collect();
            commands.spawn((
                Name::new(format!("Chunk {cx},{cy}")),
                TilemapChunk {
                    chunk_size: UVec2::splat(CHUNK_TILES),
                    tile_display_size: UVec2::splat(TILE_SIZE),
                    tileset: tileset.clone(),
                    ..default()
                },
                TilemapChunkTileData(tiles),
                Transform::from_translation(chunk_center(chunk).extend(Z_GROUND)),
                DespawnOnExit(AppState::InRun),
            ));
        }
    }
}
