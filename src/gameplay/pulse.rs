//! Onda in area (pattern `Pulse`, Gocci): a ogni intervallo, se un nemico è nel raggio, parte
//! un'onda dal player che si allarga fino al raggio finale. Il fronte colpisce una sola volta
//! ogni nemico che raggiunge e lo rallenta per un po'. La carica è quella dei proiettili
//! ([`attack_tick`]): senza bersagli l'onda resta armata e parte appena qualcuno entra.

use bevy::prelude::*;

use super::attacks::{CHARGE_EPSILON, STEP, attack_tick};
use super::combat::{HIT_FLASH_SECS, Health, HitFlash, Hurtbox, RunStats};
use super::enemies::Enemy;
use super::kinematics::{Position, body};
use super::player::Player;
use super::spatial::EnemyGrid;
use crate::app::AppState;
use crate::data::DataId;
use crate::data::schema::{AttackData, PulseData};

/// Attacco a onda di un compagno.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct PulseAttack {
    /// Id in `attacks.ron`, passato alle onde (la presentazione ne ricava lo sprite).
    pub id: String,
    pub damage: f32,
    pub interval: f32,
    /// Secondi accumulati dall'ultima onda (vedi [`attack_tick`]).
    pub charge: f32,
    /// Raggio finale dell'onda.
    pub radius: f32,
    /// px/s del fronte.
    pub speed: f32,
    /// Frazione di velocità che resta ai nemici colpiti.
    pub slow: f32,
    pub slow_secs: f32,
    pub center_height: f32,
}

impl PulseAttack {
    /// Attacco armato con i valori di `attacks.ron`.
    pub fn from_data(data: &AttackData, p: &PulseData) -> Self {
        Self {
            id: data.id.clone(),
            damage: data.damage,
            interval: data.interval_s,
            charge: data.interval_s - STEP,
            radius: data.range,
            speed: p.speed,
            slow: p.slow,
            slow_secs: p.slow_s,
            center_height: p.center_height,
        }
    }
}

/// Onda in corso: il centro (`Position`) resta dov'è partita, il fronte si allarga.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Pulse {
    /// Raggio del fronte adesso e al passo prima (per interpolare il disegno).
    pub radius: f32,
    pub previous_radius: f32,
    pub max_radius: f32,
    pub speed: f32,
    pub damage: f32,
    pub slow: f32,
    pub slow_secs: f32,
    /// Nemici già colpiti da quest'onda.
    pub already_hit: Vec<Entity>,
}

/// Nemico rallentato: la sua velocità di inseguimento è moltiplicata per `factor`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Slowed {
    pub factor: f32,
    pub remaining: f32,
}

/// Il fronte di raggio `front` attorno a `center` tocca un cerchio di raggio `target_radius`
/// centrato in `target`: l'onda è un disco pieno, chi sta dentro è già stato raggiunto.
pub fn front_reaches(center: Vec2, front: f32, target: Vec2, target_radius: f32) -> bool {
    center.distance(target) <= front + target_radius
}

/// Fattore di velocità per un nemico rallentato più volte: vale il rallentamento più forte e
/// la durata più lunga, non si sommano.
pub fn combine_slow(current: Option<Slowed>, new: Slowed) -> Slowed {
    match current {
        None => new,
        Some(old) => Slowed {
            factor: old.factor.min(new.factor),
            remaining: old.remaining.max(new.remaining),
        },
    }
}

/// Nella catena del combattimento, dopo i proiettili: le onde pronte partono.
pub(super) fn fire_pulses(
    mut commands: Commands,
    time: Res<Time>,
    grid: Res<EnemyGrid>,
    player: Single<&Position, With<Player>>,
    mut attacks: Query<&mut PulseAttack>,
    enemies: Query<(&Position, &Hurtbox, &Health), (With<Enemy>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    for mut attack in &mut attacks {
        if attack.charge + dt + CHARGE_EPSILON < attack.interval {
            attack.charge += dt; // in ricarica: niente ricerca dei bersagli
            continue;
        }
        let center = player.0 + Vec2::Y * attack.center_height;
        let in_reach = grid.candidates(center, attack.radius).any(|entry| {
            enemies
                .get(entry.entity)
                .is_ok_and(|(feet, hurtbox, health)| {
                    !health.is_dead()
                        && front_reaches(
                            center,
                            attack.radius,
                            hurtbox.center(feet.0),
                            hurtbox.radius,
                        )
                })
        });
        let (charge, fire) = attack_tick(attack.charge, attack.interval, dt, in_reach);
        attack.charge = charge;
        if !fire {
            continue;
        }
        commands.spawn((
            Name::new("Onda"),
            DataId(attack.id.clone()),
            Pulse {
                radius: 0.0,
                previous_radius: 0.0,
                max_radius: attack.radius,
                speed: attack.speed,
                damage: attack.damage,
                slow: attack.slow,
                slow_secs: attack.slow_secs,
                already_hit: Vec::new(),
            },
            body(center),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// In `SimSet::Movement`: il fronte si allarga.
pub(super) fn expand_pulses(time: Res<Time>, mut pulses: Query<&mut Pulse>) {
    let dt = time.delta_secs();
    for mut pulse in &mut pulses {
        pulse.previous_radius = pulse.radius;
        pulse.radius = (pulse.radius + pulse.speed * dt).min(pulse.max_radius);
    }
}

/// Il fronte colpisce chi raggiunge (una volta per onda) e lo rallenta; l'onda arrivata al
/// raggio finale sparisce. I rallentamenti dello stesso passo si combinano prima di essere
/// scritti, così due onde insieme non si sovrascrivono, e si scrivono nell'ordine dei colpi:
/// l'ordine degli inserimenti decide l'ordine dei nemici nelle query, e con una `HashMap`
/// (ordine casuale a ogni istanza) la stessa seed dava run diverse.
pub(super) fn pulse_hits(
    mut commands: Commands,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut pulses: Query<(Entity, &Position, &mut Pulse)>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health, Option<&Slowed>), With<Enemy>>,
) {
    let mut slows: Vec<(Entity, Slowed)> = Vec::new();
    for (entity, center, mut pulse) in &mut pulses {
        let front = pulse.radius;
        for entry in grid.candidates(center.0, front) {
            let enemy = entry.entity;
            if pulse.already_hit.contains(&enemy) {
                continue;
            }
            let Ok((feet, hurtbox, mut health, slowed)) = enemies.get_mut(enemy) else {
                continue;
            };
            if health.is_dead()
                || !front_reaches(center.0, front, hurtbox.center(feet.0), hurtbox.radius)
            {
                continue;
            }
            health.current -= pulse.damage;
            pulse.already_hit.push(enemy);
            stats.hits += 1;
            let slow = Slowed {
                factor: pulse.slow,
                remaining: pulse.slow_secs,
            };
            match slows.iter_mut().find(|(e, _)| *e == enemy) {
                Some((_, pending)) => *pending = combine_slow(Some(*pending), slow),
                None => slows.push((enemy, combine_slow(slowed.copied(), slow))),
            }
            commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
        }
        if pulse.radius >= pulse.max_radius {
            commands.entity(entity).try_despawn();
        }
    }
    for (enemy, slow) in slows {
        commands.entity(enemy).try_insert(slow);
    }
}

/// Fine del rallentamento.
pub(super) fn wear_off_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut slowed: Query<(Entity, &mut Slowed)>,
) {
    let dt = time.delta_secs();
    for (entity, mut slow) in &mut slowed {
        slow.remaining -= dt;
        if slow.remaining <= 0.0 {
            commands.entity(entity).try_remove::<Slowed>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_front_reaches_whoever_it_touches_or_contains() {
        let center = Vec2::ZERO;
        assert!(front_reaches(center, 20.0, Vec2::new(25.0, 0.0), 5.0));
        assert!(!front_reaches(center, 20.0, Vec2::new(25.1, 0.0), 5.0));
        // dentro il disco: raggiunto anche se lontano dal bordo
        assert!(front_reaches(center, 20.0, Vec2::new(2.0, 0.0), 1.0));
    }

    #[test]
    fn slows_do_not_stack() {
        let weak = Slowed {
            factor: 0.8,
            remaining: 3.0,
        };
        let strong = Slowed {
            factor: 0.5,
            remaining: 1.0,
        };
        assert_eq!(combine_slow(None, weak), weak);
        assert_eq!(
            combine_slow(Some(weak), strong),
            Slowed {
                factor: 0.5,
                remaining: 3.0
            }
        );
    }
}
