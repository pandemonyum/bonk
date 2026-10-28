//! Fulmine a catena (pattern `Chain`, Tuonny): a ogni intervallo, se un nemico è entro `range`
//! da chi attacca, il fulmine lo colpisce e salta al nemico vivo più vicino non ancora colpito
//! entro `jump_range` dall'ultimo, fino a `jumps` salti, con il danno moltiplicato per
//! `falloff` a ogni salto. È istantaneo (niente proiettile in volo) e lascia per un attimo un
//! `ChainBolt` con i punti toccati, che la presentazione disegna. Ogni nemico colpito prende una
//! scossa che lo rallenta per un attimo (come l'onda, più breve). La carica è quella dei
//! proiettili ([`attack_tick`]): senza bersagli il fulmine resta armato.

use bevy::prelude::*;

use super::attacks::{CHARGE_EPSILON, STEP, attack_tick, nearest_target};
use super::combat::{HIT_FLASH_SECS, Health, HitFlash, Hurtbox, RunStats, Source, strike_enemy};
use super::enemies::Enemy;
use super::kinematics::Position;
use super::pulse::{Slowed, combine_slow};
use super::spatial::EnemyGrid;
use crate::app::AppState;
use crate::data::schema::{AttackData, ChainData};

/// Secondi in cui il fulmine resta a schermo.
pub const BOLT_S: f32 = 0.15;

/// Attacco a catena di un compagno.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChainAttack {
    /// Id in `attacks.ron`, passato ai fulmini (la presentazione ne ricava lo sprite).
    pub id: String,
    pub damage: f32,
    pub interval: f32,
    /// Secondi accumulati dall'ultimo fulmine (vedi [`attack_tick`]).
    pub charge: f32,
    pub range: f32,
    pub jumps: u32,
    pub jump_range: f32,
    pub falloff: f32,
    pub muzzle_height: f32,
    pub slow: f32,
    pub slow_s: f32,
}

impl ChainAttack {
    /// Attacco armato (colpisce al primo passo con un bersaglio) con i valori di `attacks.ron`.
    pub fn from_data(data: &AttackData, c: &ChainData) -> Self {
        Self {
            id: data.id.clone(),
            damage: data.damage,
            interval: data.interval_s,
            charge: data.interval_s - STEP,
            range: data.range,
            jumps: c.jumps,
            jump_range: c.jump_range,
            falloff: c.falloff,
            muzzle_height: c.muzzle_height,
            slow: c.slow,
            slow_s: c.slow_s,
        }
    }
}

/// Un fulmine appena scoccato: da chi attacca al primo bersaglio, poi di salto in salto.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChainBolt {
    /// Id in `attacks.ron` (la presentazione ne ricava lo sprite delle scintille).
    pub id: String,
    pub points: Vec<Vec2>,
    pub remaining: f32,
}

/// Bersagli di un fulmine: il primo è `first`, poi ogni salto va al candidato non ancora
/// colpito più vicino all'ultimo, entro `jump_range`. Restituisce i colpi in ordine con il loro
/// danno (`damage`, poi × `falloff` a ogni salto). A parità di distanza vince il primo.
pub fn chain_targets(
    first: (Entity, Vec2),
    candidates: &[(Entity, Vec2)],
    jumps: u32,
    jump_range: f32,
    damage: f32,
    falloff: f32,
) -> Vec<(Entity, Vec2, f32)> {
    let mut hits = vec![(first.0, first.1, damage)];
    let mut dealt = damage;
    for _ in 0..jumps {
        let (_, last, _) = *hits.last().unwrap();
        let free = candidates
            .iter()
            .copied()
            .filter(|(e, _)| hits.iter().all(|(hit, _, _)| hit != e));
        let Some(next) = nearest_target(last, free, jump_range) else {
            break;
        };
        dealt *= falloff;
        hits.push((next.0, next.1, dealt));
    }
    hits
}

/// Nella catena del combattimento, dopo proiettili e onde: i fulmini pronti scoccano.
pub(super) fn fire_chains(
    mut commands: Commands,
    time: Res<Time>,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    mut casters: Query<(&Position, &mut ChainAttack, Option<&Source>)>,
    mut enemies: Query<(&Position, &Hurtbox, &mut Health, Option<&Slowed>), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (position, mut attack, source) in &mut casters {
        if attack.charge + dt + CHARGE_EPSILON < attack.interval {
            attack.charge += dt; // in ricarica: niente ricerca del bersaglio
            continue;
        }
        let origin = position.0 + Vec2::Y * attack.muzzle_height;
        let alive = |center: Vec2, radius: f32| -> Vec<(Entity, Vec2)> {
            grid.candidates(center, radius)
                .filter_map(|entry| {
                    let (feet, hurtbox, health, _) = enemies.get(entry.entity).ok()?;
                    (!health.is_dead()).then(|| (entry.entity, hurtbox.center(feet.0)))
                })
                .collect()
        };
        let first = nearest_target(origin, alive(origin, attack.range), attack.range);
        let (charge, fire) = attack_tick(attack.charge, attack.interval, dt, first.is_some());
        attack.charge = charge;
        let (true, Some(first)) = (fire, first) else {
            continue; // armato: scocca appena un nemico entra nel raggio
        };
        // candidati ai salti: tutti i vivi a portata della catena intera dal primo bersaglio
        let reach = attack.jump_range * attack.jumps as f32;
        let nearby = alive(first.1, reach);
        let hits = chain_targets(
            first,
            &nearby,
            attack.jumps,
            attack.jump_range,
            attack.damage,
            attack.falloff,
        );
        let shock = Slowed {
            factor: attack.slow,
            remaining: attack.slow_s,
        };
        let mut points = vec![origin];
        for (enemy, at, damage) in hits {
            if let Ok((_, _, mut health, slowed)) = enemies.get_mut(enemy) {
                strike_enemy(&mut health, damage, source.copied(), &mut stats);
                if shock.remaining > 0.0 {
                    let slow = combine_slow(slowed.copied(), shock);
                    commands.entity(enemy).try_insert(slow);
                }
            }
            commands.entity(enemy).try_insert(HitFlash(HIT_FLASH_SECS));
            points.push(at);
        }
        commands.spawn((
            Name::new("Fulmine"),
            ChainBolt {
                id: attack.id.clone(),
                points,
                remaining: BOLT_S,
            },
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// I fulmini restano a schermo `BOLT_S` secondi.
pub(super) fn fade_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut ChainBolt)>,
) {
    for (entity, mut bolt) in &mut bolts {
        bolt.remaining -= time.delta_secs();
        if bolt.remaining <= 0.0 {
            commands.entity(entity).try_despawn();
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
    fn the_bolt_jumps_to_the_nearest_new_enemy_and_weakens() {
        let first = (e(1), Vec2::ZERO);
        let others = [
            (e(1), Vec2::ZERO),
            (e(2), Vec2::new(30.0, 0.0)),
            (e(3), Vec2::new(55.0, 0.0)),
            (e(4), Vec2::new(-20.0, 0.0)),
        ];
        let hits = chain_targets(first, &others, 3, 40.0, 10.0, 0.5);
        let order: Vec<Entity> = hits.iter().map(|h| h.0).collect();
        // 1, poi il più vicino (4 a 20 px), poi da 4 il più vicino nuovo entro 40: 2 dista 50,
        // fuori; nessun altro: la catena si ferma
        assert_eq!(order, [e(1), e(4)]);
        assert_eq!(hits[1].2, 5.0);
    }

    #[test]
    fn jumps_stop_at_the_limit_and_never_repeat_a_target() {
        let line: Vec<(Entity, Vec2)> = (0..6)
            .map(|i| (e(i + 1), Vec2::new(i as f32 * 25.0, 0.0)))
            .collect();
        let hits = chain_targets(line[0], &line, 3, 30.0, 8.0, 0.8);
        let order: Vec<Entity> = hits.iter().map(|h| h.0).collect();
        assert_eq!(order, [e(1), e(2), e(3), e(4)], "1 + 3 salti");
        let damage: Vec<f32> = hits.iter().map(|h| h.2).collect();
        assert!((damage[3] - 8.0 * 0.8 * 0.8 * 0.8).abs() < 1e-5);
        // senza salti: solo il primo
        assert_eq!(chain_targets(line[0], &line, 0, 30.0, 8.0, 0.8).len(), 1);
    }
}
