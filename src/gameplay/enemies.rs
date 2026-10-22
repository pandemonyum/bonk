//! Nemici S0: un'ondata iniziale che insegue il player e si separa per non sovrapporsi.
//! Lo spawn director con archetipi da `game-data/` arriva con la P0.3.

use bevy::prelude::*;

use super::SimSet;
use super::kinematics::{Position, body};
use super::player::{PLAYER_SPAWN, Player};
use crate::app::{AppState, RunSeed};
use crate::rng::Pcg32;
use crate::world::ArenaBounds;

pub const ENEMY_SPEED: f32 = 40.0;
pub const INITIAL_ENEMIES: usize = 10;
/// Anello di spawn attorno al player: fuori dai 135 px di mezza altezza dello schermo.
pub const SPAWN_RING: (f32, f32) = (170.0, 240.0);
/// Distanza a cui il nemico smette di avanzare (contatto; i danni arrivano con la P0.3).
pub const CONTACT_DISTANCE: f32 = 10.0;
/// Distanza minima tra due nemici.
pub const SEPARATION_RADIUS: f32 = 14.0;
/// Stream RNG dedicato agli spawn (vedi `Pcg32::new`).
const SPAWN_STREAM: u64 = 1;

#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy {
    pub speed: f32,
}

/// Punto casuale nell'anello `[r_min, r_max)` attorno a `center`.
pub fn ring_point(rng: &mut Pcg32, center: Vec2, r_min: f32, r_max: f32) -> Vec2 {
    let angle = rng.range_f32(0.0, std::f32::consts::TAU);
    let radius = rng.range_f32(r_min, r_max);
    center + Vec2::from_angle(angle) * radius
}

/// Un passo di inseguimento: avanza verso `target` senza superare la distanza di contatto.
pub fn chase_step(from: Vec2, target: Vec2, speed: f32, dt: f32, stop_distance: f32) -> Vec2 {
    let to_target = target - from;
    let distance = to_target.length();
    if distance <= stop_distance {
        return from;
    }
    let step = (speed * dt).min(distance - stop_distance);
    from + to_target / distance * step
}

/// Spostamento da applicare ad `a` (e opposto a `b`) per portarli a `min_distance`.
pub fn separation_offset(a: Vec2, b: Vec2, min_distance: f32) -> Vec2 {
    let delta = a - b;
    let distance = delta.length();
    if distance >= min_distance {
        return Vec2::ZERO;
    }
    // Sovrapposizione perfetta: direzione fissa per restare deterministici.
    let direction = if distance > f32::EPSILON {
        delta / distance
    } else {
        Vec2::X
    };
    direction * (min_distance - distance) * 0.5
}

/// Riporta `point` sul bordo del cerchio `(center, radius)` se ci è entrato.
pub fn push_outside(point: Vec2, center: Vec2, radius: f32) -> Vec2 {
    let delta = point - center;
    let distance = delta.length();
    if distance >= radius {
        return point;
    }
    let direction = if distance > f32::EPSILON {
        delta / distance
    } else {
        Vec2::X
    };
    center + direction * radius
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_initial_wave)
            .add_systems(FixedUpdate, chase_player.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                // La respinta dal player viene per ultima: contro un muro vale più non entrare
                // nel player che restare dentro i limiti (si sfora di al massimo
                // CONTACT_DISTANCE: lo sprite può sovrapporsi un poco alla siepe).
                (separate_enemies, keep_enemies_in_arena, keep_out_of_player)
                    .chain()
                    .in_set(SimSet::Resolve),
            );
    }
}

fn spawn_initial_wave(mut commands: Commands, seed: Res<RunSeed>, arena: Res<ArenaBounds>) {
    let mut rng = Pcg32::new(seed.0, SPAWN_STREAM);
    for i in 0..INITIAL_ENEMIES {
        let at = ring_point(&mut rng, PLAYER_SPAWN, SPAWN_RING.0, SPAWN_RING.1);
        commands.spawn((
            Name::new(format!("Gelatina {i}")),
            Enemy { speed: ENEMY_SPEED },
            body(arena.clamp(at)),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

fn chase_player(
    time: Res<Time>,
    player: Single<&Position, With<Player>>,
    mut enemies: Query<(&Enemy, &mut Position), Without<Player>>,
) {
    let target = player.0;
    let dt = time.delta_secs();
    for (enemy, mut position) in &mut enemies {
        position.0 = chase_step(position.0, target, enemy.speed, dt, CONTACT_DISTANCE);
    }
}

/// O(n²): accettabile per le 10 entità della S0. Con lo spawn director (P0.3) va sostituito
/// da una griglia spaziale (vedi docs/BACKLOG.md).
fn separate_enemies(mut enemies: Query<&mut Position, With<Enemy>>) {
    let mut bodies: Vec<Mut<Position>> = enemies.iter_mut().collect();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let push = separation_offset(bodies[i].0, bodies[j].0, SEPARATION_RADIUS);
            if push != Vec2::ZERO {
                bodies[i].0 += push;
                bodies[j].0 -= push;
            }
        }
    }
}

/// La separazione tra nemici ammassati e il clamp ai bordi possono spingerne alcuni dentro il
/// player: li si respinge sul raggio di contatto (test `enemies_chase_the_player_and_do_not_stack`
/// e `enemies_never_enter_the_player_against_a_wall`).
fn keep_out_of_player(
    player: Single<&Position, With<Player>>,
    mut enemies: Query<&mut Position, (With<Enemy>, Without<Player>)>,
) {
    let center = player.0;
    for mut position in &mut enemies {
        position.0 = push_outside(position.0, center, CONTACT_DISTANCE);
    }
}

fn keep_enemies_in_arena(arena: Res<ArenaBounds>, mut enemies: Query<&mut Position, With<Enemy>>) {
    for mut position in &mut enemies {
        position.0 = arena.clamp(position.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chase_moves_closer_by_speed_times_dt() {
        let next = chase_step(Vec2::new(100.0, 0.0), Vec2::ZERO, 40.0, 0.5, 10.0);
        assert!((next - Vec2::new(80.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn chase_never_overshoots_contact_distance() {
        let next = chase_step(Vec2::new(12.0, 0.0), Vec2::ZERO, 40.0, 1.0, 10.0);
        assert!((next.length() - 10.0).abs() < 1e-4);
        let still = chase_step(Vec2::new(5.0, 0.0), Vec2::ZERO, 40.0, 1.0, 10.0);
        assert_eq!(still, Vec2::new(5.0, 0.0));
    }

    #[test]
    fn separation_is_symmetric_and_restores_min_distance() {
        let (a, b) = (Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0));
        let push = separation_offset(a, b, 14.0);
        let (a2, b2) = (a + push, b - push);
        assert!((a2.distance(b2) - 14.0).abs() < 1e-4);
        assert_eq!(
            separation_offset(Vec2::ZERO, Vec2::new(20.0, 0.0), 14.0),
            Vec2::ZERO
        );
        assert_ne!(separation_offset(Vec2::ONE, Vec2::ONE, 14.0), Vec2::ZERO);
    }

    #[test]
    fn push_outside_only_moves_points_inside_the_circle() {
        let pushed = push_outside(Vec2::new(3.0, 4.0), Vec2::ZERO, 10.0);
        assert!((pushed - Vec2::new(6.0, 8.0)).length() < 1e-4);
        assert_eq!(
            push_outside(Vec2::new(30.0, 0.0), Vec2::ZERO, 10.0),
            Vec2::new(30.0, 0.0)
        );
        assert_eq!(
            push_outside(Vec2::ONE, Vec2::ONE, 10.0),
            Vec2::new(11.0, 1.0)
        );
    }

    #[test]
    fn ring_points_stay_in_ring_and_are_deterministic() {
        let mut rng = Pcg32::new(99, SPAWN_STREAM);
        let mut again = Pcg32::new(99, SPAWN_STREAM);
        for _ in 0..500 {
            let p = ring_point(&mut rng, Vec2::new(10.0, -5.0), 170.0, 240.0);
            let d = p.distance(Vec2::new(10.0, -5.0));
            assert!((170.0 - 1e-3..240.0 + 1e-3).contains(&d), "{d}");
            assert_eq!(
                p,
                ring_point(&mut again, Vec2::new(10.0, -5.0), 170.0, 240.0)
            );
        }
    }
}
