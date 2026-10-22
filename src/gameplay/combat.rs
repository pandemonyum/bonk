//! Combattimento: proiettili in volo, colpi sui nemici, morte, feedback di colpo, statistiche.
//! Chi spara e quando è in `attacks`.
//!
//! Tutto in `FixedUpdate`; nessuna dipendenza dalla grafica. I colpi interrogano la griglia
//! spaziale (`spatial::EnemyGrid`): solo i nemici vicini, non tutti.

use bevy::prelude::*;

use super::SimSet;
use super::attacks::fire_auto_attacks;
use super::enemies::Enemy;
use super::kinematics::Position;
use super::progression::drop_xp_gems;
use super::spatial::EnemyGrid;
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
                    drop_xp_gems,
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
