//! Foglie orbitanti (pattern `Orbit`, Frond): girano sempre attorno al player, equidistanti.
//! Una foglia colpisce chi tocca; lo stesso nemico non può essere colpito di nuovo dalle foglie
//! dello stesso compagno prima di `rehit_interval` secondi.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::combat::{
    HIT_FLASH_SECS, Health, HitFlash, Hurtbox, RunStats, Source, circles_overlap, strike_enemy,
};
use super::enemies::Enemy;
use super::kinematics::{Position, body};
use super::player::Player;
use super::spatial::EnemyGrid;
use crate::app::AppState;
use crate::data::DataId;
use crate::data::schema::{AttackData, OrbitData};

/// Attacco a foglie orbitanti di un compagno.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct OrbitAttack {
    /// Id in `attacks.ron`, passato alle foglie (la presentazione ne ricava lo sprite).
    pub id: String,
    pub damage: f32,
    /// Secondi prima che le foglie colpiscano di nuovo lo stesso nemico.
    pub rehit_interval: f32,
    /// Raggio dell'orbita.
    pub radius: f32,
    pub leaves: u32,
    /// Radianti al secondo.
    pub angular_speed: f32,
    pub leaf_radius: f32,
    pub center_height: f32,
    /// Angolo della prima foglia.
    pub angle: f32,
    /// Nemici colpiti da poco, con i secondi che mancano al prossimo colpo possibile.
    pub recent: Vec<(Entity, f32)>,
}

impl OrbitAttack {
    pub fn from_data(data: &AttackData, o: &OrbitData) -> Self {
        Self {
            id: data.id.clone(),
            damage: data.damage,
            rehit_interval: data.interval_s,
            radius: data.range,
            leaves: o.count,
            angular_speed: o.angular_speed_deg.to_radians(),
            leaf_radius: o.radius,
            center_height: o.center_height,
            angle: 0.0,
            recent: Vec::new(),
        }
    }

    pub fn center(&self, player_feet: Vec2) -> Vec2 {
        player_feet + Vec2::Y * self.center_height
    }
}

/// Una foglia: `index` da 0 a `leaves - 1` dell'attacco del compagno `owner`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leaf {
    pub owner: Entity,
    pub index: u32,
}

/// Posizione della foglia `index` su `count`, equidistanti a partire da `angle`.
pub fn leaf_position(center: Vec2, radius: f32, angle: f32, index: u32, count: u32) -> Vec2 {
    let a = angle + TAU * index as f32 / count.max(1) as f32;
    center + Vec2::from_angle(a) * radius
}

/// In `SimSet::Movement`, prima di [`move_leaves`]: tante foglie quante ne dice l'attacco
/// (cambiano con gli upgrade), nate già al loro posto.
pub(super) fn sync_leaves(
    mut commands: Commands,
    player: Single<&Position, With<Player>>,
    owners: Query<(Entity, &OrbitAttack)>,
    leaves: Query<(Entity, &Leaf)>,
) {
    for (entity, leaf) in &leaves {
        let keep = owners
            .get(leaf.owner)
            .is_ok_and(|(_, attack)| leaf.index < attack.leaves);
        if !keep {
            commands.entity(entity).try_despawn();
        }
    }
    for (owner, attack) in &owners {
        let center = attack.center(player.0);
        for index in 0..attack.leaves {
            let exists = leaves
                .iter()
                .any(|(_, leaf)| leaf.owner == owner && leaf.index == index);
            if exists {
                continue;
            }
            let at = leaf_position(center, attack.radius, attack.angle, index, attack.leaves);
            commands.spawn((
                Name::new("Foglia"),
                DataId(attack.id.clone()),
                Leaf { owner, index },
                body(at),
                DespawnOnExit(AppState::InRun),
            ));
        }
    }
}

/// In `SimSet::Movement`, dopo il player: l'orbita gira e segue il player.
pub(super) fn move_leaves(
    time: Res<Time>,
    player: Single<&Position, (With<Player>, Without<Leaf>)>,
    mut owners: Query<&mut OrbitAttack>,
    mut leaves: Query<(&Leaf, &mut Position)>,
) {
    let dt = time.delta_secs();
    for mut attack in &mut owners {
        attack.angle = (attack.angle + attack.angular_speed * dt).rem_euclid(TAU);
    }
    for (leaf, mut position) in &mut leaves {
        let Ok(attack) = owners.get(leaf.owner) else {
            continue;
        };
        let center = attack.center(player.0);
        position.0 = leaf_position(
            center,
            attack.radius,
            attack.angle,
            leaf.index,
            attack.leaves,
        );
    }
}

/// Nella catena del combattimento: le foglie colpiscono chi toccano, una volta ogni
/// `rehit_interval` per nemico.
pub(super) fn leaf_hits(
    mut commands: Commands,
    time: Res<Time>,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut owners: Query<(&mut OrbitAttack, Option<&Source>)>,
    leaves: Query<(&Leaf, &Position)>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health), (With<Enemy>, Without<Leaf>)>,
) {
    let dt = time.delta_secs();
    // l'attesa si arrotonda al passo più vicino (0,54 s = 32 passi, non 33): nessuno sbilancio
    // sistematico per eccesso con gli upgrade di cadenza
    for (mut attack, _) in &mut owners {
        attack.recent.retain_mut(|(_, wait)| {
            *wait -= dt;
            *wait > 0.5 * dt
        });
    }
    for (leaf, at) in &leaves {
        let Ok((mut attack, source)) = owners.get_mut(leaf.owner) else {
            continue;
        };
        let source = source.copied();
        for entry in grid.candidates(at.0, attack.leaf_radius) {
            let enemy = entry.entity;
            if attack.recent.iter().any(|(e, _)| *e == enemy) {
                continue;
            }
            let Ok((feet, hurtbox, mut health)) = enemies.get_mut(enemy) else {
                continue;
            };
            if health.is_dead()
                || !circles_overlap(
                    at.0,
                    attack.leaf_radius,
                    hurtbox.center(feet.0),
                    hurtbox.radius,
                )
            {
                continue;
            }
            strike_enemy(&mut health, attack.damage, source, &mut stats);
            let wait = attack.rehit_interval;
            attack.recent.push((enemy, wait));
            commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_are_evenly_spaced_on_the_orbit() {
        let center = Vec2::new(10.0, 5.0);
        let points: Vec<Vec2> = (0..3)
            .map(|i| leaf_position(center, 30.0, 0.4, i, 3))
            .collect();
        for p in &points {
            assert!((p.distance(center) - 30.0).abs() < 1e-4);
        }
        let side = points[0].distance(points[1]);
        assert!((points[1].distance(points[2]) - side).abs() < 1e-3);
        assert!((points[2].distance(points[0]) - side).abs() < 1e-3);
        // lato di un triangolo equilatero inscritto: r × √3
        assert!((side - 30.0 * 3f32.sqrt()).abs() < 1e-3);
    }
}
