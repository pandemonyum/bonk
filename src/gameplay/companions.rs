//! Compagni: seguono l'evocatore in formazione e attaccano da soli con il pattern del loro
//! attacco (proiettile, onda, foglie). Il primo è il compagno iniziale dello stage; gli altri
//! arrivano con le carte `JoinCompanion`, nei posti di `player.ron` (`companion_slots`),
//! nell'ordine in cui vengono scelti. Valori da `companions.ron` e `attacks.ron`.

use bevy::prelude::*;

use super::SimSet;
use super::kinematics::{Position, body};
use super::player::{MoveIntent, PLAYER_SPAWN, Player};
use super::upgrades::{Modifiers, UpgradeStacks};
use crate::app::AppState;
use crate::data::schema::{AttackPattern, CompanionData, Effect, FollowData};
use crate::data::{DataId, GameConfig};
use crate::world::ArenaBounds;

/// Posti al massimo nella squadra (validato nei dati): oltre, la formazione si affolla.
pub const MAX_COMPANION_SLOTS: usize = 6;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Companion {
    /// Ultima direzione di marcia del player: il compagno sta dalla parte opposta.
    pub trail: Vec2,
    pub follow: FollowData,
    /// Posto nella squadra (indice in `player.ron: companion_slots`).
    pub slot: usize,
    /// Rotazione del posto rispetto a "dritto dietro al player", in radianti.
    pub slot_angle: f32,
}

/// Un passo di inseguimento morbido verso `target`, con velocità massima.
pub fn follow_step(from: Vec2, target: Vec2, max_speed: f32, gain: f32, dt: f32) -> Vec2 {
    let wanted = (target - from) * (gain * dt).min(1.0);
    from + wanted.clamp_length_max(max_speed * dt)
}

/// Dove sta un compagno: a `distance` dietro al player rispetto alla direzione di marcia
/// `trail`, spostato dell'angolo del suo posto (positivo = alla sinistra di chi avanza).
pub fn formation_target(player: Vec2, trail: Vec2, slot_angle: f32, distance: f32) -> Vec2 {
    player - Vec2::from_angle(-slot_angle).rotate(trail) * distance
}

pub struct CompanionsPlugin;

impl Plugin for CompanionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_starting_companion)
            .add_systems(FixedUpdate, follow_player.in_set(SimSet::Movement));
    }
}

/// Crea il compagno `data` nel posto `slot`, con l'attacco del suo pattern già modificato
/// dagli upgrade presi finora. `trail` è la direzione di marcia corrente della squadra: il
/// compagno nasce già al suo posto dietro al player, non davanti.
pub fn spawn_companion(
    commands: &mut Commands,
    config: &GameConfig,
    data: &CompanionData,
    slot: usize,
    player: Vec2,
    trail: Vec2,
    modifiers: &Modifiers,
) {
    let attack = config.companion_attack(data);
    let slot_angle = config.player.companion_slots[slot].angle_deg.to_radians();
    let mut entity = commands.spawn((
        Name::new(data.display_name.clone()),
        DataId(data.id.clone()),
        Companion {
            trail,
            follow: data.follow,
            slot,
            slot_angle,
        },
        body(formation_target(
            player,
            trail,
            slot_angle,
            data.follow.distance,
        )),
        DespawnOnExit(AppState::InRun),
    ));
    match &attack.pattern {
        AttackPattern::Projectile(p) => entity.insert(modifiers.projectile(attack, p)),
        AttackPattern::Pulse(p) => entity.insert(modifiers.pulse(attack, p)),
        AttackPattern::Orbit(o) => entity.insert(modifiers.orbit(attack, o)),
    };
}

fn spawn_starting_companion(mut commands: Commands, config: Res<GameConfig>) {
    let data = config.starting_companion();
    // a inizio run nessun upgrade: valori base
    spawn_companion(
        &mut commands,
        &config,
        data,
        0,
        PLAYER_SPAWN,
        Vec2::X,
        &Modifiers::default(),
    );
}

/// Le carte `JoinCompanion` prese aggiungono il loro compagno, se non c'è già, nel primo posto
/// libero e con gli upgrade correnti (`Modifiers`, già ricalcolati in questo fotogramma). Gira
/// quando cambiano gli stack, dopo `upgrades::refresh_modifiers`.
pub(super) fn join_companions(
    mut commands: Commands,
    config: Res<GameConfig>,
    stacks: Res<UpgradeStacks>,
    modifiers: Res<Modifiers>,
    player: Single<&Position, With<Player>>,
    present: Query<(&DataId, &Companion)>,
) {
    let mut party: Vec<&str> = present.iter().map(|(id, _)| id.0.as_str()).collect();
    // tutti i compagni seguono la stessa direzione di marcia
    let trail = present.iter().next().map_or(Vec2::X, |(_, c)| c.trail);
    for (i, upgrade) in config.upgrades.iter().enumerate() {
        let Effect::JoinCompanion(id) = &upgrade.effect else {
            continue;
        };
        if stacks.of(i) == 0 || party.contains(&id.as_str()) {
            continue;
        }
        let slot = party.len();
        let Some(data) = config.companion(id) else {
            continue; // riferimento validato: solo dopo una ricarica dei dati a metà fotogramma
        };
        if slot >= config.player.companion_slots.len() {
            warn!("nessun posto libero per {}", data.display_name);
            break;
        }
        spawn_companion(
            &mut commands,
            &config,
            data,
            slot,
            player.0,
            trail,
            &modifiers,
        );
        info!(
            "{} si unisce alla squadra (posto {})",
            data.display_name,
            slot + 1
        );
        party.push(id);
    }
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
        let target = formation_target(
            player.0,
            companion.trail,
            companion.slot_angle,
            follow.distance,
        );
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

    #[test]
    fn formation_slots_sit_behind_the_player_at_their_angle() {
        let player = Vec2::new(100.0, 0.0);
        let east = Vec2::X;
        assert_eq!(
            formation_target(player, east, 0.0, 20.0),
            Vec2::new(80.0, 0.0)
        );
        // +90°: a sinistra di chi guarda verso est, cioè a nord
        let left = formation_target(player, east, 90f32.to_radians(), 20.0);
        assert!((left - Vec2::new(100.0, 20.0)).length() < 1e-4, "{left:?}");
        // posti diversi, punti diversi alla stessa distanza dal player
        let a = formation_target(player, east, 55f32.to_radians(), 22.0);
        let b = formation_target(player, east, -55f32.to_radians(), 22.0);
        assert!((a.distance(player) - 22.0).abs() < 1e-4 && a.distance(b) > 30.0);
    }
}
