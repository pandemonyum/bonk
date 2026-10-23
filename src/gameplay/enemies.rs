//! Nemici: inseguimento, separazione, ostacoli, limiti. Gli archetipi vengono da
//! `game-data/enemies.ron`; chi nasce, quando e dove lo decide lo spawn director (`spawning`).

use bevy::prelude::*;

use super::SimSet;
use super::collision::{Footprint, push_outside, resolve_against_obstacles};
use super::combat::{Health, Hurtbox};
use super::kinematics::{Position, body};
use super::player::Player;
use super::pulse::Slowed;
use super::spatial::EnemyGrid;
use super::threats::{Charger, Slammer};
use crate::app::AppState;
use crate::data::DataId;
use crate::data::schema::EnemyData;
use crate::world::ArenaBounds;
use crate::world::obstacles::ObstacleField;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub speed: f32,
    /// Distanza dal player a cui smette di avanzare e a cui infligge `contact_damage`.
    pub contact_distance: f32,
    /// Distanza minima da un altro nemico.
    pub separation_radius: f32,
    pub contact_damage: f32,
    /// XP della gemma lasciata alla morte (0 = nessuna gemma).
    pub xp: u32,
}

/// Il boss della run: quando non c'è più (dopo essere arrivato) la run è vinta.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boss;

/// Moltiplicatori di difficoltà al momento della nascita.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    pub hp: f32,
    pub damage: f32,
}

impl Scale {
    pub const BASE: Self = Self {
        hp: 1.0,
        damage: 1.0,
    };

    /// Al secondo `t`: `1 + crescita × minuti`, per HP e danno.
    pub fn at(hp_growth_per_min: f32, damage_growth_per_min: f32, t: f32) -> Self {
        let minutes = t.max(0.0) / 60.0;
        Self {
            hp: 1.0 + hp_growth_per_min * minutes,
            damage: 1.0 + damage_growth_per_min * minutes,
        }
    }
}

/// Crea un nemico dell'archetipo `data` con HP e danno moltiplicati da `scale`, più la carica
/// o il colpo ad area se i dati li prevedono.
pub fn spawn_enemy(
    commands: &mut Commands,
    data: &EnemyData,
    at: Vec2,
    serial: u32,
    scale: Scale,
) -> Entity {
    let mut entity = commands.spawn(enemy_bundle(data, at, serial));
    if scale != Scale::BASE {
        entity.insert((
            Enemy {
                contact_damage: data.contact_damage * scale.damage,
                ..enemy_stats(data)
            },
            Health::full(data.max_hp * scale.hp),
        ));
    }
    if let Some(charge) = data.charge {
        entity.insert(Charger::new(charge));
    }
    if let Some(slam) = data.slam {
        entity.insert(Slammer::new(slam));
    }
    entity.id()
}

fn enemy_stats(data: &EnemyData) -> Enemy {
    Enemy {
        speed: data.speed,
        contact_distance: data.contact_distance,
        separation_radius: data.separation_radius,
        contact_damage: data.contact_damage,
        xp: data.xp,
    }
}

/// Correzioni dei nemici in `SimSet::Resolve`, in catena.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnemyResolve;

/// Componenti di un nemico dell'archetipo `data` con i piedi in `at`.
pub fn enemy_bundle(data: &EnemyData, at: Vec2, serial: u32) -> impl Bundle {
    (
        Name::new(format!("{} {serial}", data.display_name)),
        DataId(data.id.clone()),
        enemy_stats(data),
        Health::full(data.max_hp),
        Hurtbox {
            radius: data.hurtbox.radius,
            center_offset: data.hurtbox.center_offset,
        },
        Footprint(data.footprint_radius),
        body(at),
        DespawnOnExit(AppState::InRun),
    )
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

/// Spostamento da applicare ad `a` per allontanarlo da `b` fino a `min_distance` (metà della
/// sovrapposizione: l'altra metà la riceve `b`). Se i due punti coincidono si usa `tie`,
/// che deve essere opposto per i due membri della coppia.
pub fn separation_offset(a: Vec2, b: Vec2, min_distance: f32, tie: Vec2) -> Vec2 {
    let delta = a - b;
    let distance = delta.length();
    if distance >= min_distance {
        return Vec2::ZERO;
    }
    let direction = if distance > f32::EPSILON {
        delta / distance
    } else {
        tie
    };
    direction * (min_distance - distance) * 0.5
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, chase_player.in_set(SimSet::Movement))
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
                    .in_set(EnemyResolve)
                    .in_set(SimSet::Resolve),
            );
    }
}

/// Inseguimento; un nemico rallentato dall'onda va a `speed × factor`. Chi sta caricando
/// (preavviso o scatto) lo muove `threats`.
pub(super) fn chase_player(
    time: Res<Time>,
    player: Single<&Position, With<Player>>,
    mut enemies: Query<(&Enemy, Option<&Slowed>, Option<&Charger>, &mut Position), Without<Player>>,
) {
    let target = player.0;
    let dt = time.delta_secs();
    for (enemy, slowed, charger, mut position) in &mut enemies {
        if charger.is_some_and(|c| !c.is_chasing()) {
            continue;
        }
        let speed = enemy.speed * slowed.map_or(1.0, |s| s.factor);
        position.0 = chase_step(position.0, target, speed, dt, enemy.contact_distance);
    }
}

/// Separazione con la griglia: ogni nemico guarda solo i vicini e tutte le spinte sono
/// calcolate sulle posizioni della griglia (fotografia prima della separazione), quindi il
/// risultato non dipende dall'ordine di visita. Tra due archetipi vale il raggio più grande.
fn separate_enemies(grid: Res<EnemyGrid>, mut enemies: Query<(Entity, &Enemy, &mut Position)>) {
    for (entity, enemy, mut position) in &mut enemies {
        let own = position.0;
        let mut push = Vec2::ZERO;
        for other in grid.candidates(own, enemy.separation_radius) {
            if other.entity == entity {
                continue;
            }
            let radius = enemy.separation_radius.max(other.separation_radius);
            let tie = if entity < other.entity {
                Vec2::X
            } else {
                Vec2::NEG_X
            };
            push += separation_offset(own, other.position, radius, tie);
        }
        if push != Vec2::ZERO {
            position.0 += push;
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
        let push_a = separation_offset(a, b, 14.0, Vec2::X);
        let push_b = separation_offset(b, a, 14.0, Vec2::NEG_X);
        assert_eq!(push_a, -push_b);
        assert!(((a + push_a).distance(b + push_b) - 14.0).abs() < 1e-4);
        assert_eq!(
            separation_offset(Vec2::ZERO, Vec2::new(20.0, 0.0), 14.0, Vec2::X),
            Vec2::ZERO
        );
    }

    #[test]
    fn coincident_enemies_are_pushed_apart_not_together() {
        let a = separation_offset(Vec2::ONE, Vec2::ONE, 14.0, Vec2::X);
        let b = separation_offset(Vec2::ONE, Vec2::ONE, 14.0, Vec2::NEG_X);
        assert_eq!(a, Vec2::new(7.0, 0.0));
        assert_eq!(b, -a);
    }
}
