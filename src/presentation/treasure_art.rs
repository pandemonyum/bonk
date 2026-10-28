//! Monete e forzieri a schermo: la moneta gira (4 fotogrammi), il forziere chiuso luccica (riga
//! 0 del foglio) e quando si apre resta per un attimo dove stava, si apre (riga 1), poi sbiadisce.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::Z_PICKUPS;
use super::enemy_anim::{COLUMNS, cycle_frame, once_frame};
use super::sprites::DepthBase;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::gameplay::treasure::{Chest, ChestOpened, Coin};

/// Fotogrammi della moneta e del luccichio al secondo.
const COIN_FPS: f32 = 8.0;
const GLINT_FPS: f32 = 4.0;
/// Il forziere aperto: apertura, attesa con la luce, dissolvenza (secondi).
const OPEN_S: f32 = 0.5;
const HOLD_S: f32 = 0.4;
const FADE_S: f32 = 0.4;
/// Lato del fotogramma della moneta.
const COIN_SIDE: u32 = 10;

#[derive(Resource)]
struct TreasureSheets {
    coin: Handle<Image>,
    coin_layout: Handle<TextureAtlasLayout>,
    chest: Handle<Image>,
    chest_layout: Handle<TextureAtlasLayout>,
}

/// Animazione in corso di una moneta o di un forziere chiuso.
#[derive(Component, Debug, Default)]
struct Spin(f32);

/// Forziere che si sta aprendo dove stava quello chiuso.
#[derive(Component, Debug, Default)]
pub struct OpeningChest {
    pub clock: f32,
}

/// Trasparenza del forziere aperto dopo `t` secondi.
pub fn opening_alpha(t: f32) -> f32 {
    (1.0 - (t - OPEN_S - HOLD_S) / FADE_S).clamp(0.0, 1.0)
}

pub struct TreasureArtPlugin;

impl Plugin for TreasureArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), load_sheets)
            .add_systems(
                Update,
                (attach_sprites, spin, show_openings, open_chests)
                    .chain()
                    .run_if(in_state(AppState::InRun))
                    .run_if(resource_exists::<TreasureSheets>),
            );
    }
}

fn load_sheets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    sheets: Option<Res<TreasureSheets>>,
) {
    if sheets.is_some() {
        return;
    }
    let chests = &config.stage.chests;
    let coin_layout =
        TextureAtlasLayout::from_grid(UVec2::splat(COIN_SIDE), COLUMNS, 1, None, None);
    let chest_layout = TextureAtlasLayout::from_grid(
        UVec2::new(chests.frame_w, chests.frame_h),
        COLUMNS,
        2,
        None,
        None,
    );
    commands.insert_resource(TreasureSheets {
        coin: assets.load(config.progression.coin.sprite.clone()),
        coin_layout: layouts.add(coin_layout),
        chest: assets.load(chests.sprite.clone()),
        chest_layout: layouts.add(chest_layout),
    });
}

fn atlas_sprite(image: &Handle<Image>, layout: &Handle<TextureAtlasLayout>) -> Sprite {
    Sprite::from_atlas_image(
        image.clone(),
        TextureAtlas {
            layout: layout.clone(),
            index: 0,
        },
    )
}

/// Monete e forzieri nuovi prendono lo sprite al primo fotogramma (i fogli si caricano entrando
/// nella run, quando i primi forzieri possono già esserci). Le monete stanno a terra come le
/// gemme; il forziere è alto come un attore e si ordina con loro per y.
fn attach_sprites(
    mut commands: Commands,
    sheets: Res<TreasureSheets>,
    coins: Query<Entity, (With<Coin>, Without<Sprite>)>,
    chests: Query<Entity, (With<Chest>, Without<Sprite>)>,
) {
    for entity in &coins {
        commands.entity(entity).try_insert((
            atlas_sprite(&sheets.coin, &sheets.coin_layout),
            Anchor::BOTTOM_CENTER,
            DepthBase(Z_PICKUPS),
            Spin::default(),
        ));
    }
    for entity in &chests {
        commands.entity(entity).try_insert((
            atlas_sprite(&sheets.chest, &sheets.chest_layout),
            Anchor::BOTTOM_CENTER,
            Spin::default(),
        ));
    }
}

fn spin(
    time: Res<Time>,
    mut spinning: Query<(&mut Spin, &mut Sprite, Has<Chest>), Or<(With<Coin>, With<Chest>)>>,
) {
    for (mut spin, mut sprite, chest) in &mut spinning {
        spin.0 += time.delta_secs();
        let fps = if chest { GLINT_FPS } else { COIN_FPS };
        let index = cycle_frame(spin.0, fps) as usize;
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}

/// Il forziere aperto prende il posto di quello chiuso, alla stessa profondità.
fn show_openings(
    mut commands: Commands,
    mut opened: MessageReader<ChestOpened>,
    sheets: Res<TreasureSheets>,
) {
    for open in opened.read() {
        let at = open.at.round();
        commands.spawn((
            Name::new("Forziere aperto"),
            OpeningChest::default(),
            atlas_sprite(&sheets.chest, &sheets.chest_layout),
            Anchor::BOTTOM_CENTER,
            Transform::from_translation(super::sprites::screen_translation(
                at,
                at,
                0.0,
                super::Z_ACTORS,
            )),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// Il tempo dell'apertura è reale: la run è in pausa sul draft delle reliquie mentre si apre.
fn open_chests(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut chests: Query<(Entity, &mut OpeningChest, &mut Sprite)>,
) {
    for (entity, mut chest, mut sprite) in &mut chests {
        chest.clock += time.delta_secs();
        if chest.clock >= OPEN_S + HOLD_S + FADE_S {
            commands.entity(entity).despawn();
            continue;
        }
        let index = (COLUMNS + once_frame(chest.clock, OPEN_S)) as usize;
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        sprite.color = Color::WHITE.with_alpha(opening_alpha(chest.clock));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_open_chest_waits_then_fades() {
        assert_eq!(opening_alpha(0.0), 1.0);
        assert_eq!(opening_alpha(OPEN_S + HOLD_S), 1.0);
        assert!((opening_alpha(OPEN_S + HOLD_S + FADE_S / 2.0) - 0.5).abs() < 1e-5);
        assert!(opening_alpha(OPEN_S + HOLD_S + FADE_S) < 1e-5);
    }
}
