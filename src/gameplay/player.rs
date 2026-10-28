//! Evocatore controllato dal giocatore: movimento a 8 direzioni dentro l'arena.

use bevy::prelude::*;

use super::SimSet;
use super::collision::{Footprint, resolve_against_obstacles};
use super::combat::Health;
use super::damage::{Invulnerability, Toughness};
use super::kinematics::{Position, body};
use super::progression::PickupRadius;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::world::ArenaBounds;
use crate::world::obstacles::ObstacleField;

/// Punto di partenza: il centro dell'arena (sempre libero da ostacoli).
pub const PLAYER_SPAWN: Vec2 = Vec2::ZERO;

#[derive(Component, Debug, Clone, Copy)]
pub struct Player {
    pub speed: f32,
}

/// Direzione desiderata (lunghezza 0 o 1), scritta da tastiera, autoplay o test.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct MoveIntent(pub Vec2);

/// Direzione normalizzata dai quattro tasti: 8 direzioni, i tasti opposti si annullano.
pub fn direction_from_keys(up: bool, down: bool, left: bool, right: bool) -> Vec2 {
    let x = f32::from(u8::from(right)) - f32::from(u8::from(left));
    let y = f32::from(u8::from(up)) - f32::from(u8::from(down));
    Vec2::new(x, y).normalize_or_zero()
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveIntent>()
            .add_systems(OnEnter(AppState::InRun), spawn_player)
            .add_systems(FixedUpdate, move_player.in_set(SimSet::Player));
    }
}

fn spawn_player(mut commands: Commands, config: Res<GameConfig>) {
    let data = &config.player;
    commands.spawn((
        Name::new("Evocatore"),
        Player { speed: data.speed },
        Health::full(data.max_hp),
        Invulnerability {
            remaining: 0.0,
            duration: data.invulnerability_s,
        },
        Toughness::default(),
        Footprint(data.footprint_radius),
        PickupRadius(data.pickup_radius),
        body(PLAYER_SPAWN),
        DespawnOnExit(AppState::InRun),
    ));
}

/// Movimento, poi respinta dagli ostacoli (si scivola lungo il bordo), poi limiti dell'arena.
/// Gli ostacoli distano due tile dalla siepe e la loro zona di respinta sta dentro i limiti
/// (test `obstacle_exclusion_zones_stay_inside_the_walkable_area`): il clamp finale non può
/// riportare il player dentro un'impronta.
fn move_player(
    time: Res<Time>,
    intent: Res<MoveIntent>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
    mut players: Query<(&Player, &Footprint, &mut Position)>,
) {
    let direction = intent.clamp_length_max(1.0);
    for (player, footprint, mut position) in &mut players {
        let mut next = position.0 + direction * player.speed * time.delta_secs();
        if let Some(field) = &field {
            next = resolve_against_obstacles(next, footprint.0, field);
        }
        position.0 = arena.clamp(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_directions_are_unit_length() {
        for (up, down, left, right) in [
            (true, false, false, false),
            (true, false, false, true),
            (false, false, false, true),
            (false, true, false, true),
            (false, true, false, false),
            (false, true, true, false),
            (false, false, true, false),
            (true, false, true, false),
        ] {
            let d = direction_from_keys(up, down, left, right);
            assert!((d.length() - 1.0).abs() < 1e-6, "{d:?}");
        }
        let diagonal = direction_from_keys(true, false, false, true);
        assert!((diagonal.x - diagonal.y).abs() < 1e-6 && diagonal.x > 0.0);
    }

    #[test]
    fn no_keys_or_opposite_keys_mean_no_movement() {
        assert_eq!(direction_from_keys(false, false, false, false), Vec2::ZERO);
        assert_eq!(direction_from_keys(true, true, false, false), Vec2::ZERO);
        assert_eq!(direction_from_keys(true, true, true, true), Vec2::ZERO);
    }
}
