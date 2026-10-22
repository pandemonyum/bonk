//! Grafica di onde e foglie. Le foglie sono sprite centrati come i proiettili; l'onda è un
//! anello di gocce figlie dell'entità `Pulse`, spostate dal centro al raggio del fronte (con
//! l'interpolazione tra due passi e lo snap al pixel). Niente scale o rotazioni degli sprite.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Z_PROJECTILES;
use super::sprites::{DepthBase, SyncTransforms, data_sprite};
use crate::data::{DataId, GameConfig};
use crate::gameplay::orbit::Leaf;
use crate::gameplay::pulse::Pulse;

/// Distanza tra due gocce sul fronte dell'onda al raggio finale.
const DROPLET_SPACING: f32 = 9.0;

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
    fn rings_have_a_readable_number_of_droplets() {
        assert_eq!(droplet_count(5.0), 8);
        assert_eq!(droplet_count(56.0), 39);
        assert_eq!(droplet_count(500.0), 40);
    }
}
