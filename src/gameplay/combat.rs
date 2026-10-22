//! Combattimento: attacchi automatici, proiettili, danni, morte, feedback di colpo.
//!
//! Tutto in `FixedUpdate`; nessuna dipendenza dalla grafica. Le collisioni proiettile→nemico
//! sono O(proiettili × nemici): va bene per le decine di entità della P0.2, la griglia
//! spaziale arriva con lo spawn director (P0.3).

use bevy::prelude::*;

use super::SimSet;
use super::enemies::Enemy;
use super::kinematics::{Position, body};
use crate::app::AppState;
use crate::world::ArenaBounds;

/// Raggio di collisione dei proiettili. Con velocità fino a 400 px/s (6,7 px per passo) e
/// hurtbox da 7 px un proiettile non può attraversare un nemico tra due passi.
pub const PROJECTILE_RADIUS: f32 = 3.0;
/// Durata della tinta di colpo.
pub const HIT_FLASH_SECS: f32 = 0.12;
/// Margine di vita del proiettile oltre la gittata (raggiunge nemici in movimento).
const LIFETIME_MARGIN: f32 = 1.25;
/// Tolleranza sul cooldown: sottrarre 1/60 s a ogni passo accumula errore di arrotondamento,
/// e senza tolleranza un intervallo di 1,1 s potrebbe durare 67 passi invece di 66.
const COOLDOWN_EPSILON: f32 = 1.0e-4;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Cerchio colpibile, centrato `center_offset` px sopra i piedi (`Position`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hurtbox {
    pub radius: f32,
    pub center_offset: f32,
}

impl Hurtbox {
    pub fn center(&self, feet: Vec2) -> Vec2 {
        feet + Vec2::Y * self.center_offset
    }
}

/// Colpito da poco: la presentazione lo tinge, la simulazione lo fa scadere.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct HitFlash(pub f32);

/// Attacco a proiettili mirati sul nemico più vicino entro `range`.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct AutoAttack {
    pub damage: f32,
    /// Secondi tra due attacchi.
    pub interval: f32,
    /// Secondi al prossimo attacco; a 0 l'attacco è pronto e parte appena c'è un bersaglio.
    pub cooldown: f32,
    pub range: f32,
    pub projectile_speed: f32,
    pub projectiles: u32,
    /// Angolo in radianti tra due proiettili della stessa raffica.
    pub spread: f32,
    /// Nemici extra che un proiettile attraversa (0 = si ferma al primo).
    pub pierce: u32,
    /// Altezza da cui parte il colpo rispetto ai piedi di chi attacca.
    pub muzzle_height: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Projectile {
    pub velocity: Vec2,
    pub damage: f32,
    pub radius: f32,
    pub remaining_secs: f32,
    pub hits_left: u32,
    /// Nemici già colpiti: un proiettile perforante non colpisce due volte lo stesso.
    pub already_hit: Vec<Entity>,
}

/// Statistiche della run per HUD e schermata risultati.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct RunStats {
    pub defeated: u32,
    pub projectiles_fired: u32,
    /// Colpi andati a segno (un proiettile perforante può contarne più d'uno).
    pub hits: u32,
}

/// Il bersaglio più vicino a `origin` entro `range`. A parità di distanza vince il primo.
pub fn nearest_target(
    origin: Vec2,
    candidates: impl IntoIterator<Item = (Entity, Vec2)>,
    range: f32,
) -> Option<(Entity, Vec2)> {
    let mut best: Option<(Entity, Vec2, f32)> = None;
    for (entity, at) in candidates {
        let d2 = origin.distance_squared(at);
        if d2 <= range * range && best.is_none_or(|(_, _, best_d2)| d2 < best_d2) {
            best = Some((entity, at, d2));
        }
    }
    best.map(|(entity, at, _)| (entity, at))
}

/// Direzioni di una raffica di `count` proiettili a ventaglio attorno a `aim`.
pub fn spread_directions(aim: Vec2, count: u32, spread: f32) -> Vec<Vec2> {
    let middle = (count.max(1) - 1) as f32 / 2.0;
    (0..count.max(1))
        .map(|i| Vec2::from_angle((i as f32 - middle) * spread).rotate(aim))
        .collect()
}

pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    a.distance_squared(b) <= (ra + rb) * (ra + rb)
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunStats>()
            .add_systems(OnEnter(AppState::InRun), reset_stats)
            .add_systems(FixedUpdate, move_projectiles.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (
                    fire_auto_attacks,
                    projectile_hits,
                    resolve_deaths,
                    expire_projectiles,
                    fade_hit_flash,
                )
                    .chain()
                    .in_set(SimSet::Combat),
            );
    }
}

fn reset_stats(mut stats: ResMut<RunStats>) {
    *stats = RunStats::default();
}

fn move_projectiles(time: Res<Time>, mut projectiles: Query<(&Projectile, &mut Position)>) {
    let dt = time.delta_secs();
    for (projectile, mut position) in &mut projectiles {
        position.0 += projectile.velocity * dt;
    }
}

fn fire_auto_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut stats: ResMut<RunStats>,
    mut shooters: Query<(&Position, &mut AutoAttack)>,
    enemies: Query<(Entity, &Position, &Hurtbox, &Health), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (position, mut attack) in &mut shooters {
        attack.cooldown = (attack.cooldown - dt).max(0.0);
        if attack.cooldown > COOLDOWN_EPSILON {
            continue;
        }
        let origin = position.0 + Vec2::Y * attack.muzzle_height;
        let alive = enemies
            .iter()
            .filter(|(_, _, _, health)| !health.is_dead())
            .map(|(entity, feet, hurtbox, _)| (entity, hurtbox.center(feet.0)));
        let Some((_, target)) = nearest_target(origin, alive, attack.range) else {
            continue; // resta pronto: spara appena un nemico entra nel raggio
        };
        let aim = (target - origin).normalize_or(Vec2::X);
        for direction in spread_directions(aim, attack.projectiles, attack.spread) {
            commands.spawn((
                Name::new("Proiettile"),
                Projectile {
                    velocity: direction * attack.projectile_speed,
                    damage: attack.damage,
                    radius: PROJECTILE_RADIUS,
                    remaining_secs: attack.range / attack.projectile_speed * LIFETIME_MARGIN,
                    hits_left: attack.pierce + 1,
                    already_hit: Vec::new(),
                },
                body(origin),
                DespawnOnExit(AppState::InRun),
            ));
            stats.projectiles_fired += 1;
        }
        attack.cooldown = attack.interval;
    }
}

fn projectile_hits(
    mut commands: Commands,
    mut stats: ResMut<RunStats>,
    mut projectiles: Query<(Entity, &Position, &mut Projectile)>,
    mut enemies: Query<(Entity, &Position, &Hurtbox, &mut Health), With<Enemy>>,
) {
    for (projectile_entity, at, mut projectile) in &mut projectiles {
        for (enemy, feet, hurtbox, mut health) in &mut enemies {
            if projectile.hits_left == 0 {
                break;
            }
            if health.is_dead() || projectile.already_hit.contains(&enemy) {
                continue;
            }
            if circles_overlap(
                at.0,
                projectile.radius,
                hurtbox.center(feet.0),
                hurtbox.radius,
            ) {
                health.current -= projectile.damage;
                commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
                projectile.already_hit.push(enemy);
                projectile.hits_left -= 1;
                stats.hits += 1;
            }
        }
        if projectile.hits_left == 0 {
            commands.entity(projectile_entity).try_despawn();
        }
    }
}

/// Un nemico a 0 HP sparisce e viene contato una sola volta: il despawn è applicato prima
/// del passo successivo e i colpi ignorano chi è già a 0 HP.
fn resolve_deaths(
    mut commands: Commands,
    mut stats: ResMut<RunStats>,
    enemies: Query<(Entity, &Health), With<Enemy>>,
) {
    for (entity, health) in &enemies {
        if health.is_dead() {
            commands.entity(entity).try_despawn();
            stats.defeated += 1;
        }
    }
}

fn expire_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    arena: Res<ArenaBounds>,
    mut projectiles: Query<(Entity, &Position, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, position, mut projectile) in &mut projectiles {
        if projectile.hits_left == 0 {
            continue; // già rimosso da projectile_hits
        }
        projectile.remaining_secs -= dt;
        if projectile.remaining_secs <= 0.0 || !arena.contains(position.0) {
            commands.entity(entity).try_despawn();
        }
    }
}

fn fade_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut HitFlash)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash) in &mut flashes {
        flash.0 -= dt;
        if flash.0 <= 0.0 {
            commands.entity(entity).try_remove::<HitFlash>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(index: u32) -> Entity {
        Entity::from_raw_u32(index).unwrap()
    }

    #[test]
    fn nearest_target_picks_the_closest_in_range() {
        let candidates = [
            (e(1), Vec2::new(50.0, 0.0)),
            (e(2), Vec2::new(0.0, 20.0)),
            (e(3), Vec2::new(-200.0, 0.0)),
        ];
        assert_eq!(
            nearest_target(Vec2::ZERO, candidates, 150.0),
            Some((e(2), Vec2::new(0.0, 20.0)))
        );
        assert_eq!(nearest_target(Vec2::ZERO, candidates, 10.0), None);
        assert_eq!(nearest_target(Vec2::ZERO, [], 150.0), None);
    }

    #[test]
    fn nearest_target_ties_go_to_the_first() {
        let candidates = [(e(7), Vec2::X * 10.0), (e(8), Vec2::Y * 10.0)];
        assert_eq!(
            nearest_target(Vec2::ZERO, candidates, 50.0).unwrap().0,
            e(7)
        );
    }

    #[test]
    fn spread_is_symmetric_around_the_aim() {
        let one = spread_directions(Vec2::X, 1, 0.3);
        assert_eq!(one.len(), 1);
        assert!((one[0] - Vec2::X).length() < 1e-6);
        let three = spread_directions(Vec2::Y, 3, 0.3);
        assert_eq!(three.len(), 3);
        assert!((three[1] - Vec2::Y).length() < 1e-6);
        assert!((three[0].angle_to(Vec2::Y) + three[2].angle_to(Vec2::Y)).abs() < 1e-5);
        assert!(three.iter().all(|d| (d.length() - 1.0).abs() < 1e-5));
        // count 0 = trattato come 1: un attacco spara sempre almeno un proiettile
        assert_eq!(spread_directions(Vec2::X, 0, 0.3).len(), 1);
    }

    #[test]
    fn overlap_includes_touching_circles() {
        assert!(circles_overlap(Vec2::ZERO, 3.0, Vec2::new(10.0, 0.0), 7.0));
        assert!(!circles_overlap(Vec2::ZERO, 3.0, Vec2::new(10.1, 0.0), 7.0));
    }

    #[test]
    fn health_dies_at_zero_or_below() {
        let mut h = Health::full(30.0);
        assert!(!h.is_dead());
        h.current -= 30.0;
        assert!(h.is_dead());
    }
}
