//! Aggancia gli sprite alle entità di gioco e ne sincronizza la posizione a schermo.
//! Gli sprite di attori e proiettili vengono dai percorsi in `game-data/` (via `DataId`);
//! quelli degli ostacoli sono parte del mondo e restano qui.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::{Z_ACTORS, Z_PICKUPS, Z_PROJECTILES};
use crate::data::{DataId, GameConfig};
use crate::gameplay::combat::{HitFlash, Projectile};
use crate::gameplay::companions::Companion;
use crate::gameplay::enemies::Enemy;
use crate::gameplay::kinematics::{Position, PreviousPosition};
use crate::gameplay::player::Player;
use crate::gameplay::progression::XpGem;
use crate::gameplay::pulse::Slowed;
use crate::world::obstacles::{Obstacle, ObstacleKind};

/// Profondità per pixel di y: chi sta più in basso è disegnato davanti.
const Y_SORT: f32 = 1.0e-4;
/// Tinta di un nemico appena colpito (lo sprite viene moltiplicato per questo colore).
const HIT_TINT: Color = Color::srgb(1.0, 0.45, 0.45);
/// Tinta di un nemico rallentato dall'onda: azzurrata, il colpo ha la precedenza.
const SLOW_TINT: Color = Color::srgb(0.6, 0.85, 1.0);

#[derive(Resource)]
pub struct SpriteAssets {
    pub rock: Handle<Image>,
    pub tree: Handle<Image>,
}

/// Livello di profondità di base di uno sprite (default: `Z_ACTORS`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DepthBase(pub f32);

/// Set in `PostUpdate` che scrive i `Transform` degli attori.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncTransforms;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world().resource::<AssetServer>();
        let sprites = SpriteAssets {
            rock: assets.load("sprites/rock.png"),
            tree: assets.load("sprites/tree.png"),
        };
        app.insert_resource(sprites)
            .add_observer(attach_player_sprite)
            .add_observer(attach_companion_sprite)
            .add_observer(attach_enemy_sprite)
            .add_observer(attach_projectile_sprite)
            .add_observer(attach_gem_sprite)
            .add_observer(attach_obstacle_sprite)
            .add_systems(Update, tint_hit_enemies)
            .add_systems(
                PostUpdate,
                interpolate_transforms
                    .in_set(SyncTransforms)
                    .before(TransformSystems::Propagate),
            );
    }
}

/// Posizione a schermo: interpolata tra due passi fissi e arrotondata al pixel.
/// Gli sprite degli attori hanno l'ancora in basso al centro e larghezza pari, quindi i bordi
/// cadono esattamente sulla griglia della texture 480×270.
pub fn screen_translation(previous: Vec2, current: Vec2, alpha: f32, depth: f32) -> Vec3 {
    let at = previous.lerp(current, alpha).round();
    at.extend(depth - at.y * Y_SORT)
}

/// Sprite di un attore con i piedi sulla `Position`.
fn actor_sprite(image: Handle<Image>) -> (Sprite, Anchor) {
    (Sprite::from_image(image), Anchor::BOTTOM_CENTER)
}

/// Percorso dello sprite di un'entità dai dati; `None` (con avviso) se l'id non esiste più,
/// per esempio dopo una ricarica che l'ha rinominato.
pub(super) fn data_sprite(
    kind: &str,
    id: Option<&DataId>,
    lookup: impl FnOnce(&str) -> Option<String>,
) -> Option<String> {
    let id = id?;
    let path = lookup(&id.0);
    if path.is_none() {
        warn!("{kind} \"{}\" assente dai dati: nessuno sprite", id.0);
    }
    path
}

fn attach_player_sprite(
    add: On<Add<Player>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
) {
    let image = assets.load(config.player.sprite.clone());
    commands.entity(add.entity).insert(actor_sprite(image));
}

fn attach_companion_sprite(
    add: On<Add<Companion>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    ids: Query<&DataId>,
) {
    let path = data_sprite("compagno", ids.get(add.entity).ok(), |id| {
        config.companion(id).map(|c| c.sprite.clone())
    });
    if let Some(path) = path {
        commands
            .entity(add.entity)
            .insert(actor_sprite(assets.load(path)));
    }
}

fn attach_enemy_sprite(
    add: On<Add<Enemy>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    ids: Query<&DataId>,
) {
    let path = data_sprite("nemico", ids.get(add.entity).ok(), |id| {
        config.enemy(id).map(|e| e.sprite.clone())
    });
    if let Some(path) = path {
        commands
            .entity(add.entity)
            .insert(actor_sprite(assets.load(path)));
    }
}

/// I proiettili sono centrati sulla `Position` (lato pari) e disegnati sopra gli attori.
fn attach_projectile_sprite(
    add: On<Add<Projectile>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    ids: Query<&DataId>,
) {
    let path = data_sprite("attacco", ids.get(add.entity).ok(), |id| {
        config.attack(id).map(|a| a.sprite().to_string())
    });
    if let Some(path) = path {
        commands.entity(add.entity).insert((
            Sprite::from_image(assets.load(path)),
            DepthBase(Z_PROJECTILES),
        ));
    }
}

/// Le gemme stanno a terra: sotto gli attori, che ci camminano sopra.
fn attach_gem_sprite(
    add: On<Add<XpGem>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
) {
    let image = assets.load(config.progression.gem.sprite.clone());
    commands
        .entity(add.entity)
        .insert((actor_sprite(image), DepthBase(Z_PICKUPS)));
}

/// Gli ostacoli condividono il y-sort degli attori: chi passa dietro un albero ne è coperto.
fn attach_obstacle_sprite(
    add: On<Add<Obstacle>>,
    mut commands: Commands,
    art: Res<SpriteAssets>,
    obstacles: Query<&Obstacle>,
) {
    let Ok(obstacle) = obstacles.get(add.entity) else {
        return;
    };
    let image = match obstacle.kind {
        ObstacleKind::Rock => &art.rock,
        ObstacleKind::Tree => &art.tree,
    };
    commands
        .entity(add.entity)
        .insert(actor_sprite(image.clone()));
}

/// Tinta di colpo su nemici e player (stesso segnale: "questo ha appena preso danno"), tinta
/// fredda sui nemici rallentati.
fn tint_hit_enemies(
    mut enemies: Query<(&mut Sprite, Has<HitFlash>, Has<Slowed>), Or<(With<Enemy>, With<Player>)>>,
) {
    for (mut sprite, flashing, slowed) in &mut enemies {
        let wanted = if flashing {
            HIT_TINT
        } else if slowed {
            SLOW_TINT
        } else {
            Color::WHITE
        };
        if sprite.color != wanted {
            sprite.color = wanted;
        }
    }
}

fn interpolate_transforms(
    fixed: Res<Time<Fixed>>,
    mut actors: Query<(
        &Position,
        &PreviousPosition,
        Option<&DepthBase>,
        &mut Transform,
    )>,
) {
    let alpha = fixed.overstep_fraction();
    for (current, previous, depth, mut transform) in &mut actors {
        let depth = depth.map_or(Z_ACTORS, |d| d.0);
        transform.translation = screen_translation(previous.0, current.0, alpha, depth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_is_pixel_snapped_and_interpolated() {
        let t = screen_translation(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 0.26, Z_ACTORS);
        assert_eq!(t.truncate(), Vec2::new(3.0, 0.0));
        let t = screen_translation(Vec2::new(1.4, -2.6), Vec2::new(1.4, -2.6), 0.9, Z_ACTORS);
        assert_eq!(t.truncate(), Vec2::new(1.0, -3.0));
    }

    #[test]
    fn lower_actors_are_drawn_in_front_and_projectiles_on_top() {
        let at = |y: f32, depth| screen_translation(Vec2::Y * y, Vec2::Y * y, 0.0, depth);
        assert!(at(-100.0, Z_ACTORS).z > at(100.0, Z_ACTORS).z);
        // anche il proiettile più in alto dell'arena sta sopra l'attore più in basso
        assert!(at(2048.0, Z_PROJECTILES).z > at(-2048.0, Z_ACTORS).z);
    }
}
