//! Fulmine a catena (Tuonny): colpisce il nemico più vicino in portata, poi salta ai vicini non
//! ancora colpiti, più debole a ogni salto, al massimo `jumps` volte; aspetta un bersaglio senza
//! accumulare colpi; la carta "+1 proiettile" aggiunge un salto. Valori dai dati.

use crate::common::{
    app_starting_with, companion_with, config, enemy_at, player_position, run_steps,
};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::Effect;
use pixel_monster_survivor::gameplay::chain::{ChainAttack, ChainBolt};
use pixel_monster_survivor::gameplay::combat::{Health, RunStats};
use pixel_monster_survivor::gameplay::upgrades::UpgradeStacks;

const HP: f32 = 1000.0;

fn chain_app() -> App {
    let id = companion_with(&GameConfig::embedded(), |a| a.chain().is_some());
    app_starting_with(&id, |_| {})
}

fn chain_attack(app: &mut App) -> ChainAttack {
    let world = app.world_mut();
    world.query::<&ChainAttack>().single(world).unwrap().clone()
}

fn hp(app: &App, enemy: Entity) -> f32 {
    app.world().get::<Health>(enemy).unwrap().current
}

/// Fila di nemici fermi a destra del player, il primo a metà portata, poi uno ogni `gap` px.
fn row_of_enemies(app: &mut App, count: u32, gap: f32) -> Vec<Entity> {
    let range = chain_attack(app).range;
    let start = player_position(app) + Vec2::X * range * 0.5;
    (0..count)
        .map(|i| enemy_at(app, start + Vec2::X * gap * i as f32, HP, 500 + i))
        .collect()
}

/// Gira finché scocca il primo fulmine (al massimo un intervallo).
fn until_first_bolt(app: &mut App) {
    let steps = (chain_attack(app).interval * 60.0).ceil() as usize + 2;
    for _ in 0..steps {
        run_steps(app, 1);
        if app.world().resource::<RunStats>().hits > 0 {
            return;
        }
    }
    panic!("nessun fulmine in un intervallo");
}

#[test]
fn a_bolt_hits_a_group_once_each_weaker_at_every_jump() {
    let mut app = chain_app();
    let attack = chain_attack(&mut app);
    let gap = attack.jump_range * 0.6;
    let enemies = row_of_enemies(&mut app, attack.jumps + 2, gap);
    until_first_bolt(&mut app);
    let mut expected = attack.damage;
    for (i, &enemy) in enemies.iter().enumerate() {
        let dealt = HP - hp(&app, enemy);
        if i as u32 <= attack.jumps {
            assert!(
                (dealt - expected).abs() < 1e-3,
                "nemico {i}: {dealt} danni, attesi {expected}"
            );
            expected *= attack.falloff;
        } else {
            assert_eq!(dealt, 0.0, "oltre i salti nessuno viene colpito");
        }
    }
    let hits = app.world().resource::<RunStats>().hits;
    assert_eq!(hits, attack.jumps + 1);
    // la scossa rallenta chi è stato colpito, non gli altri
    use pixel_monster_survivor::gameplay::pulse::Slowed;
    for (i, &enemy) in enemies.iter().enumerate() {
        let slowed = app.world().get::<Slowed>(enemy).copied();
        if i as u32 <= attack.jumps {
            let s = slowed.expect("colpito: rallentato");
            assert_eq!(s.factor, attack.slow);
        } else {
            assert!(slowed.is_none());
        }
    }
    // il fulmine resta a schermo un attimo con tutti i punti toccati, poi sparisce
    let world = app.world_mut();
    let bolt = world.query::<&ChainBolt>().single(world).unwrap().clone();
    assert_eq!(
        bolt.points.len() as u32,
        attack.jumps + 2,
        "partenza + bersagli"
    );
    run_steps(&mut app, 20);
    let world = app.world_mut();
    assert_eq!(world.query::<&ChainBolt>().iter(world).count(), 0);
}

#[test]
fn a_jump_never_reaches_beyond_its_range() {
    let mut app = chain_app();
    let attack = chain_attack(&mut app);
    // secondo nemico appena oltre il salto: colpito solo il primo
    let enemies = row_of_enemies(&mut app, 2, attack.jump_range + 15.0);
    until_first_bolt(&mut app);
    assert!(hp(&app, enemies[0]) < HP);
    assert_eq!(hp(&app, enemies[1]), HP);
}

#[test]
fn the_bolt_waits_for_a_target_then_keeps_its_cadence() {
    let mut app = chain_app();
    run_steps(&mut app, 120);
    assert_eq!(
        app.world().resource::<RunStats>().hits,
        0,
        "nessun bersaglio"
    );
    let attack = chain_attack(&mut app);
    let enemy = row_of_enemies(&mut app, 1, 0.0)[0];
    run_steps(&mut app, 1);
    assert_eq!(
        app.world().resource::<RunStats>().hits,
        1,
        "armato: scocca subito"
    );
    // poi uno per intervallo, senza raffiche di recupero dell'attesa
    let seconds = 6.0;
    run_steps(&mut app, (seconds * 60.0) as usize);
    let hits = app.world().resource::<RunStats>().hits as f32;
    let wanted = 1.0 + seconds / attack.interval;
    assert!(
        (hits - wanted).abs() <= 1.0,
        "{hits} colpi, attesi ~{wanted:.1}"
    );
    assert!(hp(&app, enemy) < HP);
}

#[test]
fn the_extra_projectile_card_adds_a_jump() {
    let mut app = chain_app();
    let base = chain_attack(&mut app).jumps;
    let (index, per_stack) = config(&app)
        .upgrades
        .iter()
        .enumerate()
        .find_map(|(i, u)| match u.effect {
            Effect::CompanionProjectiles(n) => Some((i, n)),
            _ => None,
        })
        .expect("i dati hanno la carta +1 proiettile");
    app.world_mut().resource_mut::<UpgradeStacks>().0[index] = 2;
    run_steps(&mut app, 2);
    assert_eq!(chain_attack(&mut app).jumps, base + 2 * per_stack);
}
