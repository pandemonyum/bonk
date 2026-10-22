//! Compagni: seguono l'evocatore da dietro e attaccano da soli. Valori da
//! `game-data/companions.ron` e `attacks.ron`; secondo e terzo slot con la P0.5.

use bevy::prelude::*;

use super::SimSet;
use super::attacks::AutoAttack;
use super::kinematics::{Position, body};
use super::player::{MoveIntent, PLAYER_SPAWN, Player};
use crate::app::AppState;
use crate::data::schema::FollowData;
use crate::data::{DataId, GameConfig};
use crate::world::ArenaBounds;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Companion {
    /// Ultima direzione di marcia del player: il compagno sta dalla parte opposta.
    pub trail: Vec2,
    pub follow: FollowData,
}

/// Un passo di inseguimento morbido verso `target`, con velocità massima.
pub fn follow_step(from: Vec2, target: Vec2, max_speed: f32, gain: f32, dt: f32) -> Vec2 {
    let wanted = (target - from) * (gain * dt).min(1.0);
    from + wanted.clamp_length_max(max_speed * dt)
}

pub struct CompanionsPlugin;

impl Plugin for CompanionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_starting_companion)
            .add_systems(FixedUpdate, follow_player.in_set(SimSet::Movement));
    }
}

fn spawn_starting_companion(mut commands: Commands, config: Res<GameConfig>) {
    let data = config.starting_companion();
    let trail = Vec2::X;
    commands.spawn((
        Name::new(data.display_name.clone()),
        DataId(data.id.clone()),
        Companion {
            trail,
            follow: data.follow,
        },
        AutoAttack::from_data(config.companion_attack(data)),
        body(PLAYER_SPAWN - trail * data.follow.distance),
        DespawnOnExit(AppState::InRun),
    ));
}

fn follow_player(
    time: Res<Time>,
    intent: Res<MoveIntent>,
    arena: Res<ArenaBounds>,
    player: Single<&Position, With<Player>>,
    mut companions: Query<(&mut Companion, &mut Position), Without<Player>>,
) {
    let dt = time.delta_secs();
    for (mut companion, mut position) in &mut companions {
        if let Some(direction) = intent.try_normalize() {
            companion.trail = direction;
        }
        let follow = companion.follow;
        let target = player.0 - companion.trail * follow.distance;
        let next = follow_step(position.0, target, follow.max_speed, follow.gain, dt);
        position.0 = arena.clamp(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_converges_without_overshooting() {
        let target = Vec2::new(30.0, 0.0);
        let mut at = Vec2::ZERO;
        for _ in 0..120 {
            let next = follow_step(at, target, 120.0, 6.0, 1.0 / 60.0);
            assert!(next.x >= at.x && next.x <= target.x);
            at = next;
        }
        assert!(at.distance(target) < 0.1, "{at:?}");
    }

    #[test]
    fn follow_respects_max_speed() {
        let next = follow_step(Vec2::ZERO, Vec2::new(1000.0, 0.0), 120.0, 6.0, 0.5);
        assert!((next.x - 60.0).abs() < 1e-4);
    }
}
