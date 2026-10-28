//! Combattimento: proiettili in volo, colpi sui nemici, morte, feedback di colpo, statistiche.
//! Chi spara e quando è in `attacks`.
//!
//! Tutto in `FixedUpdate`; nessuna dipendenza dalla grafica. I colpi interrogano la griglia
//! spaziale (`spatial::EnemyGrid`): solo i nemici vicini, non tutti.

use bevy::prelude::*;

use super::SimSet;
use super::attacks::fire_auto_attacks;
use super::chain::{fade_bolts, fire_chains};
use super::enemies::Enemy;
use super::kinematics::Position;
use super::orbit::{leaf_hits, move_leaves, sync_leaves};
use super::progression::drop_xp_gems;
use super::pulse::{expand_pulses, fire_pulses, pulse_hits, wear_off_slow};
use super::spatial::EnemyGrid;
use super::talent_effects::{
    AreaHit, Burning, Burns, Explosive, Shape, fade_blasts, fire_strikes, resolve_area_hits,
    tick_burns,
};
use super::treasure::drop_coins;
use crate::app::AppState;
use crate::world::inside_arena;

/// Durata della tinta di colpo.
pub const HIT_FLASH_SECS: f32 = 0.12;

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

/// Chi fa un colpo: indice del compagno in `companions.ron`. Sta sul compagno e su quello che
/// lancia (proiettili, onde), così il riepilogo sa di chi sono danni, sconfitti e cure.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source(pub usize);

/// Il contributo di un compagno alla run.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Tally {
    /// Danni andati a segno, senza l'eccesso oltre gli HP rimasti al nemico.
    pub damage: f32,
    /// Nemici finiti da un suo colpo.
    pub kills: u32,
    /// HP restituiti al player.
    pub healing: f32,
}

/// Statistiche della run per HUD e schermata risultati.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct RunStats {
    pub defeated: u32,
    pub projectiles_fired: u32,
    /// Colpi andati a segno di tutti gli attacchi (un proiettile perforante o un'onda possono
    /// contarne più d'uno).
    pub hits: u32,
    /// Colpi andati a segno dei soli proiettili, da confrontare con `projectiles_fired`.
    pub projectile_hits: u32,
    /// Danni subiti dal player.
    pub damage_taken: f32,
    /// Contributo di ogni compagno, nell'ordine di `companions.ron` (si allunga al primo colpo).
    pub by_companion: Vec<Tally>,
    /// HP recuperati dal player con carte e rigenerazione (le cure dei compagni sono nel loro
    /// `Tally`).
    pub healing: f32,
}

impl RunStats {
    pub fn tally(&self, source: Source) -> Tally {
        self.by_companion.get(source.0).copied().unwrap_or_default()
    }

    pub fn tally_mut(&mut self, source: Source) -> &mut Tally {
        if self.by_companion.len() <= source.0 {
            self.by_companion.resize(source.0 + 1, Tally::default());
        }
        &mut self.by_companion[source.0]
    }
}

/// Un colpo a un nemico vivo: toglie `damage` HP, conta il colpo e lo accredita a `source`
/// (danno vero, e il nemico se è il colpo che lo finisce). Senza `source` (prove) conta solo.
pub fn strike_enemy(
    health: &mut Health,
    damage: f32,
    source: Option<Source>,
    stats: &mut RunStats,
) {
    stats.hits += 1;
    credit_damage(health, damage, source, stats);
}

/// Danno senza un colpo (la bruciatura): toglie HP e lo accredita a `source` come
/// [`strike_enemy`], ma non conta come colpo andato a segno.
pub fn credit_damage(
    health: &mut Health,
    damage: f32,
    source: Option<Source>,
    stats: &mut RunStats,
) {
    let dealt = damage.clamp(0.0, health.current.max(0.0));
    health.current -= damage;
    if let Some(source) = source {
        let tally = stats.tally_mut(source);
        tally.damage += dealt;
        tally.kills += u32::from(health.is_dead() && dealt > 0.0);
    }
}

/// Una cura al player: aggiunge fino agli HP massimi e restituisce quanto ha curato davvero.
pub fn heal(health: &mut Health, amount: f32) -> f32 {
    if health.is_dead() || amount <= 0.0 {
        return 0.0;
    }
    let before = health.current;
    health.current = (health.current + amount).min(health.max);
    (health.current - before).max(0.0)
}

/// La catena del combattimento in `SimSet::Combat` (spari e onde → colpi di proiettili, onde
/// e foglie → gemme e monete → morti → scadenze).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSteps;

pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    a.distance_squared(b) <= (ra + rb) * (ra + rb)
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunStats>()
            .add_systems(OnEnter(AppState::InRun), reset_stats)
            .add_systems(
                FixedUpdate,
                (
                    move_projectiles,
                    expand_pulses,
                    (sync_leaves, move_leaves).chain(),
                )
                    .in_set(SimSet::Movement),
            )
            .add_systems(
                FixedUpdate,
                (
                    fire_auto_attacks,
                    fire_pulses,
                    fire_chains,
                    fire_strikes,
                    projectile_hits,
                    pulse_hits,
                    leaf_hits,
                    resolve_area_hits,
                    tick_burns,
                    drop_xp_gems,
                    drop_coins,
                    resolve_deaths,
                    expire_projectiles,
                    fade_hit_flash,
                    wear_off_slow,
                    fade_bolts,
                    fade_blasts,
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

/// Un proiettile colpisce chi tocca (una volta per nemico, fino alla perforazione); con i
/// talenti esplode sul colpito (`AreaHit` sugli altri) e lo fa bruciare.
#[allow(clippy::type_complexity)]
fn projectile_hits(
    mut commands: Commands,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut projectiles: Query<(
        Entity,
        &Position,
        &mut Projectile,
        Option<&Source>,
        Option<&Explosive>,
        Option<&Burns>,
    )>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health, Option<&Burning>), With<Enemy>>,
    mut area_hits: MessageWriter<AreaHit>,
) {
    for (projectile_entity, at, mut projectile, source, explosive, burns) in &mut projectiles {
        for entry in grid.candidates(at.0, projectile.radius) {
            if projectile.hits_left == 0 {
                break;
            }
            let enemy = entry.entity;
            let Ok((feet, hurtbox, mut health, burning)) = enemies.get_mut(enemy) else {
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
                strike_enemy(&mut health, projectile.damage, source.copied(), &mut stats);
                commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
                if let Some(Burns(burn)) = burns.copied() {
                    let next = Burning::from_hit(burn, projectile.damage, source.copied(), burning);
                    commands.entity(enemy).try_insert(next);
                }
                if let Some(Explosive(blast)) = explosive.copied() {
                    area_hits.write(AreaHit {
                        shape: Shape::Circle {
                            center: hurtbox.center(feet.0),
                            radius: blast.radius,
                        },
                        damage: projectile.damage * blast.damage_pct / 100.0,
                        source: source.copied(),
                        exclude: Some(enemy),
                        burn: None,
                    });
                }
                projectile.already_hit.push(enemy);
                projectile.hits_left -= 1;
                stats.projectile_hits += 1;
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

    #[test]
    fn overlap_includes_touching_circles() {
        assert!(circles_overlap(Vec2::ZERO, 3.0, Vec2::new(10.0, 0.0), 7.0));
        assert!(!circles_overlap(Vec2::ZERO, 3.0, Vec2::new(10.1, 0.0), 7.0));
    }

    #[test]
    fn a_strike_credits_real_damage_and_the_final_blow() {
        let mut stats = RunStats::default();
        let mut health = Health::full(10.0);
        strike_enemy(&mut health, 6.0, Some(Source(2)), &mut stats);
        assert_eq!(stats.tally(Source(2)).damage, 6.0);
        assert_eq!(stats.tally(Source(2)).kills, 0);
        // il secondo colpo ne toglie 6 ma ne restavano 4: valgono 4, e finisce il nemico
        strike_enemy(&mut health, 6.0, Some(Source(2)), &mut stats);
        assert_eq!(
            stats.tally(Source(2)),
            Tally {
                damage: 10.0,
                kills: 1,
                healing: 0.0
            }
        );
        assert_eq!(stats.hits, 2);
        // senza fonte conta il colpo e basta; altri compagni restano a zero
        strike_enemy(&mut Health::full(5.0), 1.0, None, &mut stats);
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.tally(Source(0)), Tally::default());
    }

    #[test]
    fn healing_stops_at_max_hp() {
        let mut health = Health {
            current: 90.0,
            max: 100.0,
        };
        assert_eq!(heal(&mut health, 30.0), 10.0);
        assert_eq!(health.current, 100.0);
        assert_eq!(heal(&mut health, 5.0), 0.0);
        health.current = 0.0;
        assert_eq!(heal(&mut health, 5.0), 0.0, "da morto niente cure");
    }

    #[test]
    fn health_dies_at_zero_or_below() {
        let mut h = Health::full(30.0);
        assert!(!h.is_dead());
        h.current -= 30.0;
        assert!(h.is_dead());
    }
}
