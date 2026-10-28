//! Le meccaniche dei talenti (`talents::TalentKit`) nella catena del combattimento. Esplosioni,
//! coni e colpi periodici diventano `AreaHit`, risolti tutti insieme qui contro la griglia dei
//! nemici; la bruciatura è un danno al secondo sul nemico; la cura per colpito sta sull'onda.
//! Tutto conta come danno e sconfitti del compagno che l'ha fatto; nessun effetto secondario
//! ne attiva un altro (un'area non esplode e non brucia, salvo la bruciatura del cono).

use bevy::prelude::*;

use super::attacks::{AutoAttack, nearest_target};
use super::chain::ChainAttack;
use super::combat::{HIT_FLASH_SECS, Health, HitFlash, Hurtbox, RunStats, Source, credit_damage};
use super::enemies::Enemy;
use super::kinematics::Position;
use super::orbit::OrbitAttack;
use super::pulse::PulseAttack;
use super::spatial::EnemyGrid;
use super::talents::{Burn, HealPerHit, TalentKit, TalentState};
use crate::app::AppState;

/// Secondi a schermo del segno di un colpo ad area.
pub const BLAST_S: f32 = 0.25;

/// Forma di un colpo ad area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Circle {
        center: Vec2,
        radius: f32,
    },
    /// Spicchio da `origin` lungo `direction` (normalizzata), fino a `range`, largo
    /// `half_angle` radianti per lato.
    Cone {
        origin: Vec2,
        direction: Vec2,
        range: f32,
        half_angle: f32,
    },
}

impl Shape {
    /// Il cerchio colpibile (centro, raggio) è dentro la forma.
    pub fn reaches(&self, at: Vec2, radius: f32) -> bool {
        match *self {
            Shape::Circle { center, radius: r } => at.distance(center) <= r + radius,
            Shape::Cone {
                origin,
                direction,
                range,
                half_angle,
            } => {
                let offset = at - origin;
                let distance = offset.length();
                if distance > range + radius {
                    return false;
                }
                if distance <= radius {
                    return true; // addosso a chi soffia
                }
                // il bordo del cerchio entra nello spicchio: margine angolare del raggio
                let margin = (radius / distance).clamp(-1.0, 1.0).asin();
                offset.angle_to(direction).abs() <= half_angle + margin
            }
        }
    }

    /// Centro e raggio da cui cercare i candidati nella griglia.
    fn bounds(&self) -> (Vec2, f32) {
        match *self {
            Shape::Circle { center, radius } => (center, radius),
            Shape::Cone { origin, range, .. } => (origin, range),
        }
    }
}

/// Un colpo ad area di un talento, da risolvere in questo passo.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct AreaHit {
    pub shape: Shape,
    pub damage: f32,
    pub source: Option<Source>,
    /// Nemico già colpito dal colpo diretto (l'esplosione ferisce gli altri).
    pub exclude: Option<Entity>,
    pub burn: Option<Burn>,
}

/// Il segno di un colpo ad area, per la presentazione.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Blast {
    pub shape: Shape,
    pub source: Option<Source>,
    pub remaining: f32,
}

/// Il proiettile esplode sul colpito.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Explosive(pub super::talents::Explode);

/// Chi è colpito da questo proiettile (o da queste foglie) brucia.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Burns(pub Burn);

/// L'onda cura il player per ogni nemico colpito, fino a `cap` in tutto.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct HealsOnHit {
    pub heal: HealPerHit,
    pub given: f32,
}

/// Nemico che brucia: `dps` al secondo per `remaining` secondi, accreditati a `source`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Burning {
    pub dps: f32,
    pub remaining: f32,
    pub source: Option<Source>,
}

impl Burning {
    /// La bruciatura di un colpo da `hit` danni: si rinnova, non si somma (vale la più forte).
    pub fn from_hit(burn: Burn, hit: f32, source: Option<Source>, old: Option<&Burning>) -> Self {
        let dps = hit * burn.dps_pct / 100.0;
        Self {
            dps: old.map_or(dps, |o| o.dps.max(dps)),
            remaining: burn.duration,
            source,
        }
    }
}

pub struct TalentEffectsPlugin;

impl Plugin for TalentEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AreaHit>();
    }
}

/// Danno dell'attacco base di un compagno (già con carte e talenti).
fn attack_damage(
    shot: Option<&AutoAttack>,
    pulse: Option<&PulseAttack>,
    orbit: Option<&OrbitAttack>,
    chain: Option<&ChainAttack>,
) -> f32 {
    shot.map(|a| a.damage)
        .or(pulse.map(|a| a.damage))
        .or(orbit.map(|a| a.damage))
        .or(chain.map(|a| a.damage))
        .unwrap_or(0.0)
}

/// Colpo periodico: ogni `every` secondi sul nemico vivo più vicino entro `range` dal
/// compagno; senza bersaglio resta armato.
#[allow(clippy::type_complexity)]
pub(super) fn fire_strikes(
    time: Res<Time>,
    grid: Res<EnemyGrid>,
    mut companions: Query<(
        &Position,
        &TalentKit,
        &mut TalentState,
        Option<&Source>,
        Option<&AutoAttack>,
        Option<&PulseAttack>,
        Option<&OrbitAttack>,
        Option<&ChainAttack>,
    )>,
    enemies: Query<(&Position, &Hurtbox, &Health), With<Enemy>>,
    mut hits: MessageWriter<AreaHit>,
) {
    let dt = time.delta_secs();
    for (at, kit, mut state, source, shot, pulse, orbit, chain) in &mut companions {
        let Some(strike) = kit.strike else {
            continue;
        };
        state.strike_charge = (state.strike_charge + dt).min(strike.every);
        if state.strike_charge + 1e-4 < strike.every {
            continue;
        }
        let alive = grid.candidates(at.0, strike.range).filter_map(|entry| {
            let (feet, hurtbox, health) = enemies.get(entry.entity).ok()?;
            (!health.is_dead()).then(|| (entry.entity, hurtbox.center(feet.0)))
        });
        let Some((_, target)) = nearest_target(at.0, alive, strike.range) else {
            continue;
        };
        state.strike_charge = 0.0;
        hits.write(AreaHit {
            shape: Shape::Circle {
                center: target,
                radius: strike.radius,
            },
            damage: attack_damage(shot, pulse, orbit, chain) * strike.damage_pct / 100.0,
            source: source.copied(),
            exclude: None,
            burn: None,
        });
    }
}

/// Tutti i colpi ad area del passo: ogni nemico vivo dentro la forma prende il danno (una volta
/// per colpo), il colpo lascia il suo segno.
pub(super) fn resolve_area_hits(
    mut commands: Commands,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut area_hits: MessageReader<AreaHit>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health, Option<&Burning>), With<Enemy>>,
) {
    for hit in area_hits.read() {
        let (center, reach) = hit.shape.bounds();
        let candidates: Vec<Entity> = grid.candidates(center, reach).map(|e| e.entity).collect();
        for enemy in candidates {
            if hit.exclude == Some(enemy) {
                continue;
            }
            let Ok((feet, hurtbox, mut health, burning)) = enemies.get_mut(enemy) else {
                continue;
            };
            if health.is_dead() || !hit.shape.reaches(hurtbox.center(feet.0), hurtbox.radius) {
                continue;
            }
            super::combat::strike_enemy(&mut health, hit.damage, hit.source, &mut stats);
            if let Some(burn) = hit.burn {
                let next = Burning::from_hit(burn, hit.damage, hit.source, burning);
                commands.entity(enemy).try_insert(next);
            }
            commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
        }
        commands.spawn((
            Name::new("Colpo ad area"),
            Blast {
                shape: hit.shape,
                source: hit.source,
                remaining: BLAST_S,
            },
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// La bruciatura toglie HP ogni passo e si spegne da sola.
pub(super) fn tick_burns(
    mut commands: Commands,
    time: Res<Time>,
    mut stats: ResMut<RunStats>,
    mut burning: Query<(Entity, &mut Burning, &mut Health), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (entity, mut burn, mut health) in &mut burning {
        let step = dt.min(burn.remaining.max(0.0));
        if !health.is_dead() && step > 0.0 {
            credit_damage(&mut health, burn.dps * step, burn.source, &mut stats);
        }
        burn.remaining -= dt;
        if burn.remaining <= 0.0 {
            commands.entity(entity).try_remove::<Burning>();
        }
    }
}

/// I segni dei colpi ad area restano un attimo.
pub(super) fn fade_blasts(
    mut commands: Commands,
    time: Res<Time>,
    mut blasts: Query<(Entity, &mut Blast)>,
) {
    for (entity, mut blast) in &mut blasts {
        blast.remaining -= time.delta_secs();
        if blast.remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circles_reach_what_touches_them() {
        let s = Shape::Circle {
            center: Vec2::ZERO,
            radius: 24.0,
        };
        assert!(s.reaches(Vec2::new(30.0, 0.0), 6.0));
        assert!(!s.reaches(Vec2::new(31.0, 0.0), 6.0));
    }

    #[test]
    fn cones_reach_ahead_within_their_angle_and_range() {
        let s = Shape::Cone {
            origin: Vec2::ZERO,
            direction: Vec2::X,
            range: 72.0,
            half_angle: 25f32.to_radians(),
        };
        assert!(s.reaches(Vec2::new(60.0, 0.0), 4.0));
        assert!(s.reaches(Vec2::new(60.0, 25.0), 4.0), "dentro i 25°");
        assert!(!s.reaches(Vec2::new(40.0, 40.0), 4.0), "45° è fuori");
        assert!(!s.reaches(Vec2::new(-30.0, 0.0), 4.0), "dietro");
        assert!(!s.reaches(Vec2::new(80.0, 0.0), 4.0), "oltre la portata");
        assert!(s.reaches(Vec2::new(-2.0, 0.0), 4.0), "addosso");
    }

    #[test]
    fn a_burn_refreshes_and_keeps_the_strongest() {
        let burn = Burn {
            dps_pct: 30.0,
            duration: 3.0,
        };
        let first = Burning::from_hit(burn, 20.0, None, None);
        assert_eq!((first.dps, first.remaining), (6.0, 3.0));
        let mut old = first;
        old.remaining = 0.5;
        let weaker = Burning::from_hit(burn, 10.0, None, Some(&old));
        assert_eq!((weaker.dps, weaker.remaining), (6.0, 3.0));
    }
}
