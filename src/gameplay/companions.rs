//! Compagni: seguono l'evocatore da dietro e attaccano da soli. P0.2: un compagno di fuoco
//! (nome di lavorazione "Brasa") con proiettile mirato. Slot e archetipi da dati: P0.0/P0.5.

use bevy::prelude::*;

use super::SimSet;
use super::combat::AutoAttack;
use super::kinematics::{Position, body};
use super::player::{MoveIntent, PLAYER_SPAWN, PLAYER_SPEED, Player};
use crate::app::AppState;
use crate::world::ArenaBounds;

/// Distanza dietro al player in cui il compagno si ferma.
pub const FOLLOW_DISTANCE: f32 = 18.0;
/// Più veloce del player, così recupera dopo un cambio di direzione.
pub const FOLLOW_MAX_SPEED: f32 = PLAYER_SPEED * 1.6;
/// Frazione della distanza dal punto di arrivo recuperata ogni secondo (avvicinamento morbido).
pub const FOLLOW_GAIN: f32 = 6.0;

/// Attacco base del compagno di fuoco (valori dell'handoff §7: attacco 12, intervallo 1,1 s).
pub fn ember_attack() -> AutoAttack {
    AutoAttack {
        damage: 12.0,
        interval: 1.1,
        cooldown: 0.0,
        range: 150.0,
        projectile_speed: 220.0,
        projectiles: 1,
        spread: 0.2,
        pierce: 0,
        muzzle_height: 8.0,
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Companion {
    /// Ultima direzione di marcia del player: il compagno sta dalla parte opposta.
    pub trail: Vec2,
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

fn spawn_starting_companion(mut commands: Commands) {
    let trail = Vec2::X;
    commands.spawn((
        Name::new("Brasa"),
        Companion { trail },
        ember_attack(),
        body(PLAYER_SPAWN - trail * FOLLOW_DISTANCE),
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
        let target = player.0 - companion.trail * FOLLOW_DISTANCE;
        let next = follow_step(position.0, target, FOLLOW_MAX_SPEED, FOLLOW_GAIN, dt);
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
            let next = follow_step(at, target, FOLLOW_MAX_SPEED, FOLLOW_GAIN, 1.0 / 60.0);
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
