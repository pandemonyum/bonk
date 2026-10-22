//! Combattimento: attacchi automatici, proiettili, danni ai nemici, morte, feedback di colpo.
//!
//! Tutto in `FixedUpdate`; nessuna dipendenza dalla grafica. Mira e colpi interrogano la
//! griglia spaziale (`spatial::EnemyGrid`): solo i nemici vicini, non tutti.

use bevy::prelude::*;

use super::enemies::Enemy;
use super::kinematics::{Position, body};
use super::spatial::EnemyGrid;
use super::{SIM_HZ, SimSet};
use crate::app::AppState;
use crate::data::DataId;
use crate::data::schema::AttackData;
use crate::world::inside_arena;

/// Durata della tinta di colpo.
pub const HIT_FLASH_SECS: f32 = 0.12;
/// Tolleranza sulla carica: sommare 1/60 s a ogni passo accumula errore di arrotondamento,
/// e senza tolleranza un intervallo di 1,1 s potrebbe durare 67 passi invece di 66.
const CHARGE_EPSILON: f32 = 1.0e-4;
/// Durata di un passo fisso.
const STEP: f32 = 1.0 / SIM_HZ as f32;

/// Un passo di un attacco automatico. `charge` è il tempo accumulato dall'ultimo colpo;
/// restituisce `(nuova carica, spara)`.
/// - Il tempo oltre l'intervallo resta nella carica: la cadenza media è esattamente
///   `interval` anche quando non è un multiplo del passo (0,935 s = 56,1 passi).
/// - Senza bersaglio l'attacco resta *armato* (`interval - dt`): spara al primo passo in cui
///   un nemico entra nel raggio, senza accumulare tempo di attesa e quindi senza raffiche.
pub fn attack_tick(charge: f32, interval: f32, dt: f32, has_target: bool) -> (f32, bool) {
    let charge = charge + dt;
    if charge + CHARGE_EPSILON < interval {
        (charge, false)
    } else if has_target {
        ((charge - interval).min(interval), true)
    } else {
        (interval - dt, false)
    }
}

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
    /// Id in `attacks.ron`, passato ai proiettili (la presentazione ne ricava lo sprite).
    pub id: String,
    pub damage: f32,
    /// Secondi tra due attacchi.
    pub interval: f32,
    /// Secondi accumulati dall'ultimo attacco (vedi [`attack_tick`]).
    pub charge: f32,
    pub range: f32,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    /// Secondi di volo prima che un proiettile mancato sparisca.
    pub projectile_lifetime: f32,
    pub projectiles: u32,
    /// Angolo in radianti tra due proiettili della stessa raffica.
    pub spread: f32,
    /// Nemici extra che un proiettile attraversa (0 = si ferma al primo).
    pub pierce: u32,
    /// Altezza da cui parte il colpo rispetto ai piedi di chi attacca.
    pub muzzle_height: f32,
}

impl AutoAttack {
    /// Attacco armato (spara al primo passo con un bersaglio) con i valori di `attacks.ron`.
    pub fn from_data(data: &AttackData) -> Self {
        Self {
            id: data.id.clone(),
            damage: data.damage,
            interval: data.interval_s,
            charge: data.interval_s - STEP,
            range: data.range,
            projectile_speed: data.projectile_speed,
            projectile_radius: data.projectile_radius,
            projectile_lifetime: data.projectile_lifetime_s,
            projectiles: data.projectiles,
            spread: data.spread_deg.to_radians(),
            pierce: data.pierce,
            muzzle_height: data.muzzle_height,
        }
    }
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
    /// Danni subiti dal player.
    pub damage_taken: f32,
}

/// La catena del combattimento in `SimSet::Combat` (spari → colpi → morti → scadenze).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSteps;

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
                    .in_set(CombatSteps)
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
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut shooters: Query<(&Position, &mut AutoAttack)>,
    enemies: Query<(&Position, &Hurtbox, &Health), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (position, mut attack) in &mut shooters {
        if attack.charge + dt + CHARGE_EPSILON < attack.interval {
            attack.charge += dt; // in ricarica: niente ricerca del bersaglio
            continue;
        }
        let origin = position.0 + Vec2::Y * attack.muzzle_height;
        let alive = grid.candidates(origin, attack.range).filter_map(|entry| {
            let (feet, hurtbox, health) = enemies.get(entry.entity).ok()?;
            (!health.is_dead()).then(|| (entry.entity, hurtbox.center(feet.0)))
        });
        let target = nearest_target(origin, alive, attack.range);
        let (charge, fire) = attack_tick(attack.charge, attack.interval, dt, target.is_some());
        attack.charge = charge;
        let (true, Some((_, target))) = (fire, target) else {
            continue; // armato: spara appena un nemico entra nel raggio
        };
        let aim = (target - origin).normalize_or(Vec2::X);
        for direction in spread_directions(aim, attack.projectiles, attack.spread) {
            commands.spawn((
                Name::new("Proiettile"),
                DataId(attack.id.clone()),
                Projectile {
                    velocity: direction * attack.projectile_speed,
                    damage: attack.damage,
                    radius: attack.projectile_radius,
                    remaining_secs: attack.projectile_lifetime,
                    // saturating: `pierce` enorme (es. u32::MAX = "infinito") non va in overflow
                    hits_left: attack.pierce.saturating_add(1),
                    already_hit: Vec::new(),
                },
                body(origin),
                DespawnOnExit(AppState::InRun),
            ));
            stats.projectiles_fired += 1;
        }
    }
}

fn projectile_hits(
    mut commands: Commands,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut projectiles: Query<(Entity, &Position, &mut Projectile)>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health), With<Enemy>>,
) {
    for (projectile_entity, at, mut projectile) in &mut projectiles {
        for entry in grid.candidates(at.0, projectile.radius) {
            if projectile.hits_left == 0 {
                break;
            }
            let enemy = entry.entity;
            let Ok((feet, hurtbox, mut health)) = enemies.get_mut(enemy) else {
                continue;
            };
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
    mut projectiles: Query<(Entity, &Position, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, position, mut projectile) in &mut projectiles {
        if projectile.hits_left == 0 {
            continue; // già rimosso da projectile_hits
        }
        projectile.remaining_secs -= dt;
        if projectile.remaining_secs <= 0.0 || !inside_arena(position.0) {
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

    /// Passi in cui l'attacco spara, simulando `steps` passi con il bersaglio deciso da `target`.
    fn shots(interval: f32, steps: u32, target: impl Fn(u32) -> bool) -> Vec<u32> {
        let mut charge = interval - STEP; // armato, come AutoAttack::from_data
        let mut fired = Vec::new();
        for step in 1..=steps {
            let (next, fire) = attack_tick(charge, interval, STEP, target(step));
            charge = next;
            if fire {
                fired.push(step);
            }
        }
        fired
    }

    #[test]
    fn cadence_is_exact_for_whole_steps() {
        // 1,1 s = 66 passi: primo colpo subito, poi ogni 66.
        let fired = shots(1.1, 660, |_| true);
        assert_eq!(fired.len(), 10);
        assert!(fired.windows(2).all(|w| w[1] - w[0] == 66), "{fired:?}");
        assert_eq!(fired[0], 1);
    }

    #[test]
    fn cadence_averages_exactly_for_fractional_steps() {
        // 0,935 s = 56,1 passi: senza conservare il resto diventerebbero 57 (0,95 s).
        let fired = shots(0.935, 20_000, |_| true);
        let span = (fired[fired.len() - 1] - fired[0]) as f32;
        let average = span / (fired.len() - 1) as f32;
        assert!((average - 56.1).abs() < 0.05, "media {average} passi");
    }

    #[test]
    fn waiting_without_a_target_does_not_store_shots() {
        // nessun bersaglio per 500 passi, poi sempre: un colpo subito, poi cadenza normale
        let fired = shots(1.1, 700, |step| step > 500);
        assert_eq!(fired[0], 501);
        assert_eq!(fired[1], 501 + 66);
    }
}
