//! Grafica di onde, foglie e fulmini. Le foglie sono sprite centrati come i proiettili; l'onda
//! è un anello di gocce figlie dell'entità `Pulse`, spostate dal centro al raggio del fronte (con
//! l'interpolazione tra due passi e lo snap al pixel); il fulmine è una fila di scintille a
//! zigzag lungo i suoi segmenti, figlie del `ChainBolt`. Niente scale o rotazioni degli sprite.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Z_PROJECTILES;
use super::sprites::{DepthBase, SyncTransforms, data_sprite};
use crate::data::{DataId, GameConfig};
use crate::gameplay::chain::ChainBolt;
use crate::gameplay::orbit::Leaf;
use crate::gameplay::pulse::Pulse;

/// Distanza tra due gocce sul fronte dell'onda al raggio finale.
const DROPLET_SPACING: f32 = 9.0;
/// Distanza tra due scintille del fulmine e scarto a zigzag dalla linea retta (px).
const SPARK_SPACING: f32 = 4.0;
const ZIGZAG: [f32; 4] = [0.0, 2.0, 0.0, -2.0];

/// Una goccia del fronte, nella direzione `direction` dal centro dell'onda.
#[derive(Component, Debug, Clone, Copy)]
struct Droplet {
    direction: Vec2,
}

pub struct AttackArtPlugin;

impl Plugin for AttackArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_leaf_sprite)
            .add_observer(attach_pulse_ring)
            .add_observer(attach_bolt_sparks)
            .add_systems(
                PostUpdate,
                place_droplets
                    .after(SyncTransforms)
                    .before(TransformSystems::Propagate),
            );
    }
}

/// Gocce sull'anello: abbastanza fitte da leggere un cerchio, non tante da coprire l'orda.
pub fn droplet_count(max_radius: f32) -> u32 {
    ((TAU * max_radius / DROPLET_SPACING).round() as u32).clamp(8, 40)
}

fn attach_leaf_sprite(
    add: On<Add<Leaf>>,
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

fn attach_pulse_ring(
    add: On<Add<Pulse>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    pulses: Query<(&DataId, &Pulse)>,
) {
    let Ok((id, pulse)) = pulses.get(add.entity) else {
        return;
    };
    let Some(path) = data_sprite("attacco", Some(id), |id| {
        config.attack(id).map(|a| a.sprite().to_string())
    }) else {
        return;
    };
    let image = assets.load(path);
    let count = droplet_count(pulse.max_radius);
    commands
        .entity(add.entity)
        .insert((
            Transform::default(),
            Visibility::default(),
            DepthBase(Z_PROJECTILES),
        ))
        .with_children(|ring| {
            for i in 0..count {
                ring.spawn((
                    Droplet {
                        direction: Vec2::from_angle(TAU * i as f32 / count as f32),
                    },
                    Sprite::from_image(image.clone()),
                    Transform::default(),
                ));
            }
        });
}

/// Punti delle scintille lungo i segmenti di un fulmine, a zigzag e allineati al pixel.
pub fn bolt_sparks(points: &[Vec2]) -> Vec<Vec2> {
    let mut sparks = Vec::new();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let length = a.distance(b);
        let steps = (length / SPARK_SPACING).ceil().max(1.0) as usize;
        let side = (b - a).perp().normalize_or_zero();
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let offset = if i == 0 {
                0.0
            } else {
                ZIGZAG[i % ZIGZAG.len()]
            };
            sparks.push((a.lerp(b, t) + side * offset).round());
        }
    }
    if let Some(last) = points.last() {
        sparks.push(last.round());
    }
    sparks
}

fn attach_bolt_sparks(
    add: On<Add<ChainBolt>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    bolts: Query<&ChainBolt>,
) {
    let Ok(bolt) = bolts.get(add.entity) else {
        return;
    };
    let Some(path) = config.attack(&bolt.id).map(|a| a.sprite().to_string()) else {
        return;
    };
    let image = assets.load(path);
    let sparks = bolt_sparks(&bolt.points);
    commands
        .entity(add.entity)
        .insert((
            Transform::from_xyz(0.0, 0.0, Z_PROJECTILES + 1.0),
            Visibility::default(),
        ))
        .with_children(|line| {
            for at in sparks {
                line.spawn((
                    Sprite::from_image(image.clone()),
                    Transform::from_translation(at.extend(0.0)),
                ));
            }
        });
}

/// Le gocce stanno sul fronte, interpolato tra due passi; il centro è già allineato al pixel.
fn place_droplets(
    fixed: Res<Time<Fixed>>,
    pulses: Query<(&Pulse, &Children)>,
    mut droplets: Query<(&Droplet, &mut Transform)>,
) {
    let alpha = fixed.overstep_fraction();
    for (pulse, children) in &pulses {
        let radius = pulse.previous_radius + (pulse.radius - pulse.previous_radius) * alpha;
        for child in children.iter() {
            if let Ok((droplet, mut transform)) = droplets.get_mut(child) {
                transform.translation = (droplet.direction * radius).round().extend(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bolts_are_dotted_zigzags_from_end_to_end() {
        let sparks = bolt_sparks(&[Vec2::ZERO, Vec2::new(20.0, 0.0), Vec2::new(20.0, 12.0)]);
        assert_eq!(sparks.first(), Some(&Vec2::ZERO));
        assert_eq!(sparks.last(), Some(&Vec2::new(20.0, 12.0)));
        // 20 px a passi di 4: 5 scintille, poi 12 px: 3, più l'ultima
        assert_eq!(sparks.len(), 5 + 3 + 1);
        assert!(sparks.iter().all(|p| *p == p.round()), "al pixel");
        assert!(sparks.iter().any(|p| p.y != 0.0 && p.x < 20.0), "zigzag");
    }

    #[test]
    fn rings_have_a_readable_number_of_droplets() {
        assert_eq!(droplet_count(5.0), 8);
        assert_eq!(droplet_count(56.0), 39);
        assert_eq!(droplet_count(500.0), 40);
    }
}
