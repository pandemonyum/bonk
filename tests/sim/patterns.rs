//! P0.5 headless: i due pattern nuovi. Onda (Gocci): colpisce una volta ogni nemico che il
//! fronte raggiunge, solo dentro il raggio, rallenta e aspetta un bersaglio. Foglie (Frond):
//! equidistanti attorno al player, lo seguono, colpiscono lo stesso nemico al massimo una volta
//! ogni intervallo. Raggi, danni e tempi vengono dai dati; i compagni si cercano per pattern.

use crate::common::{
    app_starting_with, companion_with, config, count, enemy_at, player_position, run_steps,
};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::{AttackData, AttackPattern};
use pixel_monster_survivor::gameplay::combat::{Health, Hurtbox, RunStats};
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::orbit::{Leaf, OrbitAttack};
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::pulse::{Pulse, PulseAttack, Slowed};
use pixel_monster_survivor::headless_app_with;

fn wave_app() -> App {
    let id = companion_with(&GameConfig::embedded(), |a| a.pulse().is_some());
    app_starting_with(&id, |_| {})
}

fn wave_attack(app: &mut App) -> PulseAttack {
    let world = app.world_mut();
    world.query::<&PulseAttack>().single(world).unwrap().clone()
}

fn orbit_attack(app: &mut App) -> OrbitAttack {
    let world = app.world_mut();
    world.query::<&OrbitAttack>().single(world).unwrap().clone()
}

/// Nemico fermo con il centro della hurtbox a `distance` dal centro dell'onda (o dell'orbita),
/// verso `direction`.
fn enemy_from_center(
    app: &mut App,
    center: Vec2,
    direction: Vec2,
    distance: f32,
    serial: u32,
) -> Entity {
    let offset = config(app).first_wave_enemy().hurtbox.center_offset;
    let feet = center + direction.normalize() * distance - Vec2::Y * offset;
    enemy_at(app, feet, 1.0e6, serial)
}

fn hp(app: &App, enemy: Entity) -> f32 {
    app.world().get::<Health>(enemy).unwrap().current
}

#[test]
fn the_wave_hits_each_enemy_in_reach_once_and_slows_it() {
    let mut app = wave_app();
    let wave = wave_attack(&mut app);
    let center = player_position(&mut app) + Vec2::Y * wave.center_height;
    let hurtbox = config(&app).first_wave_enemy().hurtbox.radius;
    let near = enemy_from_center(&mut app, center, Vec2::X, wave.radius * 0.4, 0);
    let edge = enemy_from_center(
        &mut app,
        center,
        Vec2::NEG_Y,
        wave.radius + hurtbox * 0.5,
        1,
    );
    let far = enemy_from_center(
        &mut app,
        center,
        Vec2::NEG_X,
        wave.radius + hurtbox + 10.0,
        2,
    );
    // una sola onda: dalla partenza al raggio finale, poi sparisce
    let steps = (wave.radius / wave.speed * 60.0).ceil() as usize + 3;
    run_steps(&mut app, steps);
    assert_eq!(
        count::<Pulse>(&mut app),
        0,
        "l'onda arrivata al bordo sparisce"
    );
    for enemy in [near, edge] {
        assert!(
            (1.0e6 - hp(&app, enemy) - wave.damage).abs() < 1e-3,
            "colpito una volta"
        );
        let slowed = app.world().get::<Slowed>(enemy).expect("rallentato");
        assert_eq!(slowed.factor, wave.slow);
        // durata dell'onda, non un altro numero: al più `steps` passi già consumati
        let elapsed = steps as f32 / 60.0;
        assert!(
            slowed.remaining <= wave.slow_secs && slowed.remaining > wave.slow_secs - elapsed,
            "resta {} s su {}",
            slowed.remaining,
            wave.slow_secs
        );
    }
    assert_eq!(hp(&app, far), 1.0e6, "fuori dal raggio");
    assert!(app.world().get::<Slowed>(far).is_none());
}

#[test]
fn the_wave_waits_for_a_target_then_keeps_its_cadence() {
    let mut app = wave_app();
    // un'onda vive meno di un intervallo: ogni entità `Pulse` distinta è un'onda partita
    let waves = |app: &mut App, steps: usize| {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..steps {
            run_steps(app, 1);
            let world = app.world_mut();
            seen.extend(world.query_filtered::<Entity, With<Pulse>>().iter(world));
        }
        seen.len()
    };
    assert_eq!(
        waves(&mut app, 180),
        0,
        "senza nemici nel raggio nessuna onda"
    );
    let wave = wave_attack(&mut app);
    let center = player_position(&mut app) + Vec2::Y * wave.center_height;
    enemy_from_center(&mut app, center, Vec2::X, wave.radius * 0.5, 0);
    // armata: parte al primo passo con un bersaglio, poi una ogni intervallo
    let every = (wave.interval * 60.0).round() as usize;
    assert_eq!(waves(&mut app, 600), 1 + (600 - 1) / every);
}

#[test]
fn slowed_enemies_chase_slower_until_the_slow_wears_off() {
    let mut app = wave_app();
    let wave = wave_attack(&mut app);
    let from = player_position(&mut app);
    let data = config(&app).first_wave_enemy().clone();
    let start = |x: f32| from + Vec2::new(x, 300.0);
    let normal = enemy_at(&mut app, start(-100.0), 1.0e6, 0);
    let slowed = enemy_at(&mut app, start(100.0), 1.0e6, 1);
    for enemy in [normal, slowed] {
        app.world_mut().get_mut::<Enemy>(enemy).unwrap().speed = data.speed;
    }
    let slow = Slowed {
        factor: wave.slow,
        remaining: wave.slow_secs,
    };
    app.world_mut().entity_mut(slowed).insert(slow);
    let moved = |app: &mut App, enemy: Entity, x: f32| {
        app.world()
            .get::<Position>(enemy)
            .unwrap()
            .0
            .distance(start(x))
    };
    run_steps(&mut app, 30);
    let ratio = moved(&mut app, slowed, 100.0) / moved(&mut app, normal, -100.0);
    assert!((ratio - wave.slow).abs() < 0.02, "rapporto {ratio}");
    // dura `slow_s`, non meno e non più: c'è ancora un passo prima, non c'è più un passo dopo
    let lasts = wave.slow_secs * 60.0;
    run_steps(&mut app, lasts.floor() as usize - 1 - 30);
    assert!(
        app.world().get::<Slowed>(slowed).is_some(),
        "finito troppo presto"
    );
    run_steps(
        &mut app,
        lasts.ceil() as usize + 1 - (lasts.floor() as usize - 1),
    );
    assert!(
        app.world().get::<Slowed>(slowed).is_none(),
        "rallentamento finito"
    );
}

/// Onda e foglie non usano numeri casuali: a parità di seed la run è identica, con nemici veri.
#[test]
fn waves_and_leaves_are_deterministic() {
    let embedded = GameConfig::embedded();
    for has in [
        (|a: &AttackData| a.pulse().is_some()) as fn(&AttackData) -> bool,
        |a: &AttackData| a.orbit().is_some(),
    ] {
        let id = companion_with(&embedded, has);
        let simulate = |seed| {
            let mut config = GameConfig::embedded();
            config.stage.starting_companion = id.clone();
            let mut app = headless_app_with(seed, config);
            crate::common::god(&mut app);
            app.insert_resource(MoveIntent(Vec2::new(0.6, 0.8)));
            run_steps(&mut app, 60 * 30);
            let mut enemies = crate::common::enemy_positions(&mut app);
            enemies.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
            let stats = app.world().resource::<RunStats>().clone();
            (enemies, stats)
        };
        let first = simulate(9);
        assert!(
            first.1.hits > 0,
            "{id}: nessun colpo, il confronto non prova nulla"
        );
        assert_eq!(first, simulate(9), "{id}");
    }
}

/// Tutta la squadra in mezzo a un'orda vera, con il movimento dell'autoplay: due run con la
/// stessa seed devono restare identiche. Prima della P0.7 i rallentamenti dell'onda si
/// scrivevano nell'ordine di una `HashMap` (casuale a ogni istanza) e l'ordine dei nemici nelle
/// query, quindi bersagli e separazioni, cambiava da una run all'altra.
#[test]
fn the_whole_party_in_a_crowd_is_deterministic() {
    use pixel_monster_survivor::data::schema::Effect;
    use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
    use pixel_monster_survivor::gameplay::upgrades::UpgradeStacks;
    // nemici più resistenti e più frequenti: l'orda resta fitta anche con tre compagni
    let mut tough = GameConfig::embedded();
    for enemy in &mut tough.enemies {
        enemy.max_hp *= 5.0;
    }
    for wave in &mut tough.stage.spawns.waves {
        wave.rate_per_min *= 4.0;
    }
    let simulate = |seed| {
        let mut app = headless_app_with(seed, tough.clone());
        crate::common::god(&mut app);
        app.update();
        let joins: Vec<usize> = config(&app)
            .upgrades
            .iter()
            .enumerate()
            .filter(|(_, u)| matches!(u.effect, Effect::JoinCompanion(_)))
            .map(|(i, _)| i)
            .collect();
        for i in joins {
            app.world_mut().resource_mut::<UpgradeStacks>().0[i] = 1;
        }
        for step in 0..60 * 90 {
            app.insert_resource(MoveIntent(autoplay_direction(step as f32 / 60.0)));
            app.update();
        }
        let world = app.world_mut();
        let mut enemies: Vec<(f32, f32, f32)> = world
            .query_filtered::<(&Position, &Health), With<Enemy>>()
            .iter(world)
            .map(|(p, h)| (p.0.x, p.0.y, h.current))
            .collect();
        enemies.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        let slowed = world.query::<&Slowed>().iter(world).count();
        let stats = app.world().resource::<RunStats>().clone();
        (enemies, slowed, stats)
    };
    let first = simulate(4);
    assert!(first.0.len() > 40, "orda troppo piccola: {}", first.0.len());
    assert!(first.2.hits > 500, "poca azione: {:?}", first.2);
    assert!(
        first.1 > 0,
        "nessun nemico rallentato: l'onda non ha lavorato"
    );
    assert_eq!(first, simulate(4));
}

#[test]
fn leaves_circle_the_player_evenly_and_follow_him() {
    let id = companion_with(&GameConfig::embedded(), |a| a.orbit().is_some());
    let mut app = app_starting_with(&id, |_| {});
    run_steps(&mut app, 2); // le foglie nascono al primo passo di movimento
    let orbit = orbit_attack(&mut app);
    let check = |app: &mut App| {
        let center = player_position(app) + Vec2::Y * orbit.center_height;
        let world = app.world_mut();
        let mut leaves: Vec<(u32, Vec2)> = world
            .query::<(&Leaf, &Position)>()
            .iter(world)
            .map(|(leaf, at)| (leaf.index, at.0))
            .collect();
        leaves.sort_by_key(|(index, _)| *index);
        assert_eq!(leaves.len() as u32, orbit.leaves);
        for (_, at) in &leaves {
            assert!((at.distance(center) - orbit.radius).abs() < 1e-3);
        }
        let gap = |a: Vec2, b: Vec2| {
            (a - center)
                .angle_to(b - center)
                .rem_euclid(std::f32::consts::TAU)
        };
        let even = std::f32::consts::TAU / orbit.leaves as f32;
        for pair in leaves.windows(2) {
            assert!((gap(pair[0].1, pair[1].1) - even).abs() < 1e-3);
        }
    };
    check(&mut app);
    app.insert_resource(MoveIntent(Vec2::new(1.0, 0.5).normalize()));
    run_steps(&mut app, 90);
    check(&mut app);
}

/// Con tante foglie ogni passaggio arriva prima dell'intervallo: i colpi sullo stesso nemico
/// sono distanziati proprio dall'intervallo.
#[test]
fn a_leaf_hits_the_same_enemy_at_most_once_per_interval() {
    let id = companion_with(&GameConfig::embedded(), |a| a.orbit().is_some());
    let mut app = app_starting_with(&id, |c| {
        let attack: &mut AttackData = c.attacks.iter_mut().find(|a| a.orbit().is_some()).unwrap();
        if let AttackPattern::Orbit(o) = &mut attack.pattern {
            o.count = 12;
        }
    });
    let orbit = orbit_attack(&mut app);
    let center = player_position(&mut app) + Vec2::Y * orbit.center_height;
    let enemy = enemy_from_center(&mut app, center, Vec2::X, orbit.radius, 0);
    let radius = app.world().get::<Hurtbox>(enemy).unwrap().radius;
    // tra due foglie vicine passano meno passi dell'intervallo: senza la regola colpirebbero di più
    let between = std::f32::consts::TAU / 12.0 / orbit.angular_speed.abs();
    assert!(between < orbit.rehit_interval, "dati del test inattesi");
    assert!(radius > 0.0);
    let mut hits = Vec::new();
    let mut last = hp(&app, enemy);
    for step in 0..300i32 {
        run_steps(&mut app, 1);
        let now = hp(&app, enemy);
        if now < last {
            assert!(
                (last - now - orbit.damage).abs() < 1e-3,
                "un colpo alla volta"
            );
            hits.push(step);
        }
        last = now;
    }
    let every = (orbit.rehit_interval * 60.0).round() as i32;
    assert!(hits.len() >= 4, "colpi {hits:?}");
    for pair in hits.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(
            (gap - every).abs() <= 1,
            "colpi a {hits:?}, attesi ogni {every} passi"
        );
    }
}
