//! Nemici: ondata iniziale, inseguimento, separazione, rimpiazzo di chi viene sconfitto.
//! Valori da `game-data/enemies.ron` e `stages.ron`. Lo spawn director con più archetipi e
//! curve per minuto arriva con la P0.3.

use bevy::prelude::*;

use super::SimSet;
use super::collision::{Footprint, push_outside, resolve_against_obstacles};
use super::combat::{Health, Hurtbox};
use super::kinematics::{Position, body};
use super::player::{PLAYER_SPAWN, Player};
use crate::app::{AppState, RunSeed};
use crate::data::schema::EnemyData;
use crate::data::{DataId, GameConfig};
use crate::rng::Pcg32;
use crate::world::ArenaBounds;
use crate::world::obstacles::ObstacleField;

/// Stream RNG dedicato agli spawn (vedi `Pcg32::new`).
const SPAWN_STREAM: u64 = 1;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub speed: f32,
    /// Distanza dal player a cui smette di avanzare (i danni da contatto arrivano in P0.3).
    pub contact_distance: f32,
    /// Distanza minima da un altro nemico.
    pub separation_radius: f32,
}

/// Generatore degli spawn della run: unico per ondata iniziale e rimpiazzi, così la
/// sequenza dipende solo dal seed e dall'ordine degli eventi.
#[derive(Resource, Debug, Clone)]
pub struct SpawnRng {
    rng: Pcg32,
    spawned: u32,
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

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_initial_wave)
            .add_systems(FixedUpdate, chase_player.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                // La respinta dal player viene per ultima: contro un muro vale più non entrare
                // nel player che restare dentro i limiti (si sfora di al massimo
                // `contact_distance`: lo sprite può sovrapporsi un poco alla siepe o a un
                // ostacolo).
                (
                    separate_enemies,
                    avoid_obstacles,
                    keep_enemies_in_arena,
                    keep_out_of_player,
                )
                    .chain()
                    .in_set(SimSet::Resolve),
            )
            .add_systems(FixedUpdate, maintain_population.in_set(SimSet::Spawn));
    }
}

fn spawn_enemy(commands: &mut Commands, spawns: &mut SpawnRng, data: &EnemyData, at: Vec2) {
    commands.spawn((
        Name::new(format!("{} {}", data.display_name, spawns.spawned)),
        DataId(data.id.clone()),
        Enemy {
            speed: data.speed,
            contact_distance: data.contact_distance,
            separation_radius: data.separation_radius,
        },
        Health::full(data.max_hp),
        Hurtbox {
            radius: data.hurtbox.radius,
            center_offset: data.hurtbox.center_offset,
        },
        Footprint(data.footprint_radius),
        body(at),
        DespawnOnExit(AppState::InRun),
    ));
    spawns.spawned += 1;
}

fn spawn_initial_wave(
    mut commands: Commands,
    config: Res<GameConfig>,
    seed: Res<RunSeed>,
    arena: Res<ArenaBounds>,
) {
    let mut spawns = SpawnRng {
        rng: Pcg32::new(seed.0, SPAWN_STREAM),
        spawned: 0,
    };
    let (enemy, (r_min, r_max)) = (config.stage_enemy(), config.stage.initial_ring);
    for _ in 0..config.stage.initial_enemies {
        let at = ring_point(&mut spawns.rng, PLAYER_SPAWN, r_min, r_max);
        spawn_enemy(&mut commands, &mut spawns, enemy, arena.clamp(at));
    }
    commands.insert_resource(spawns);
}

/// Rimpiazza i nemici sconfitti fuori dallo schermo. Vicino ai bordi dell'arena il clamp può
/// portare un rimpiazzo in vista: le zone di spawn vere arrivano con la P0.3.
fn maintain_population(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut spawns: ResMut<SpawnRng>,
    arena: Res<ArenaBounds>,
    player: Single<&Position, With<Player>>,
    enemies: Query<(), With<Enemy>>,
) {
    let cap = config.stage.population_cap as usize;
    let missing = cap.saturating_sub(enemies.iter().count());
    let (enemy, (r_min, r_max)) = (config.stage_enemy(), config.stage.respawn_ring);
    for _ in 0..missing {
        let at = ring_point(&mut spawns.rng, player.0, r_min, r_max);
        spawn_enemy(&mut commands, &mut spawns, enemy, arena.clamp(at));
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
        position.0 = chase_step(position.0, target, enemy.speed, dt, enemy.contact_distance);
    }
}

/// O(n²): accettabile per le decine di nemici della P0. Con lo spawn director (P0.3) va
/// sostituito da una griglia spaziale (vedi docs/BACKLOG.md). Tra due archetipi vale il
/// raggio di separazione più grande.
fn separate_enemies(mut enemies: Query<(&Enemy, &mut Position)>) {
    let mut bodies: Vec<(f32, Mut<Position>)> = enemies
        .iter_mut()
        .map(|(enemy, position)| (enemy.separation_radius, position))
        .collect();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let radius = bodies[i].0.max(bodies[j].0);
            let push = separation_offset(bodies[i].1.0, bodies[j].1.0, radius);
            if push != Vec2::ZERO {
                bodies[i].1.0 += push;
                bodies[j].1.0 -= push;
            }
        }
    }
}

/// Scivolano attorno a rocce e alberi invece di attraversarli.
fn avoid_obstacles(
    field: Option<Res<ObstacleField>>,
    mut enemies: Query<(&Footprint, &mut Position), With<Enemy>>,
) {
    let Some(field) = field else { return };
    for (footprint, mut position) in &mut enemies {
        position.0 = resolve_against_obstacles(position.0, footprint.0, &field);
    }
}

/// La separazione tra nemici ammassati e il clamp ai bordi possono spingerne alcuni dentro il
/// player: li si respinge sul raggio di contatto (test `enemies_chase_the_player_and_do_not_stack`
/// e `enemies_never_enter_the_player_against_a_wall`).
fn keep_out_of_player(
    player: Single<&Position, With<Player>>,
    mut enemies: Query<(&Enemy, &mut Position), Without<Player>>,
) {
    let center = player.0;
    for (enemy, mut position) in &mut enemies {
        position.0 = push_outside(position.0, center, enemy.contact_distance);
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
