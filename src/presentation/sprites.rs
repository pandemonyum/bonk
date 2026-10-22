//! Aggancia gli sprite alle entità di gioco e ne sincronizza la posizione a schermo.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::Z_ACTORS;
use crate::gameplay::enemies::Enemy;
use crate::gameplay::kinematics::{Position, PreviousPosition};
use crate::gameplay::player::Player;

/// Profondità per pixel di y: chi sta più in basso è disegnato davanti.
const Y_SORT: f32 = 1.0e-4;

#[derive(Resource)]
pub struct SpriteAssets {
    pub summoner: Handle<Image>,
    pub blob_enemy: Handle<Image>,
}

/// Set in `PostUpdate` che scrive i `Transform` degli attori.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncTransforms;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world().resource::<AssetServer>();
        let sprites = SpriteAssets {
            summoner: assets.load("sprites/summoner.png"),
            blob_enemy: assets.load("sprites/blob_enemy.png"),
        };
        app.insert_resource(sprites)
            .add_observer(attach_player_sprite)
            .add_observer(attach_enemy_sprite)
            .add_systems(
                PostUpdate,
                interpolate_transforms
                    .in_set(SyncTransforms)
                    .before(TransformSystems::Propagate),
            );
    }
}

/// Posizione a schermo: interpolata tra due passi fissi e arrotondata al pixel.
/// Gli sprite hanno l'ancora in basso al centro e larghezza pari, quindi i bordi cadono
/// esattamente sulla griglia della texture 480×270.
pub fn screen_translation(previous: Vec2, current: Vec2, alpha: f32) -> Vec3 {
    let at = previous.lerp(current, alpha).round();
    at.extend(Z_ACTORS - at.y * Y_SORT)
}

fn attach_player_sprite(add: On<Add<Player>>, mut commands: Commands, art: Res<SpriteAssets>) {
    commands.entity(add.entity).insert((
        Sprite::from_image(art.summoner.clone()),
        Anchor::BOTTOM_CENTER,
    ));
}

fn attach_enemy_sprite(add: On<Add<Enemy>>, mut commands: Commands, art: Res<SpriteAssets>) {
    commands.entity(add.entity).insert((
        Sprite::from_image(art.blob_enemy.clone()),
        Anchor::BOTTOM_CENTER,
    ));
}

fn interpolate_transforms(
    fixed: Res<Time<Fixed>>,
    mut actors: Query<(&Position, &PreviousPosition, &mut Transform)>,
) {
    let alpha = fixed.overstep_fraction();
    for (current, previous, mut transform) in &mut actors {
        transform.translation = screen_translation(previous.0, current.0, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_is_pixel_snapped_and_interpolated() {
        let t = screen_translation(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 0.26);
        assert_eq!(t.truncate(), Vec2::new(3.0, 0.0));
        let t = screen_translation(Vec2::new(1.4, -2.6), Vec2::new(1.4, -2.6), 0.9);
        assert_eq!(t.truncate(), Vec2::new(1.0, -3.0));
    }

    #[test]
    fn lower_actors_are_drawn_in_front() {
        let low = screen_translation(Vec2::new(0.0, -100.0), Vec2::new(0.0, -100.0), 0.0);
        let high = screen_translation(Vec2::new(0.0, 100.0), Vec2::new(0.0, 100.0), 0.0);
        assert!(low.z > high.z);
    }
}
