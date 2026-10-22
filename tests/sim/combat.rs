//! P0.2 headless: compagno che segue e attacca da solo, danni, morte, popolazione.
//! I valori attesi vengono dalla configurazione (`game-data/`), non da costanti nei test.

use crate::common::{companion_position, config, count, despawn_all, player_position, run_steps};
use bevy::prelude::*;
use pixel_monster_survivor::gameplay::combat::{
    AutoAttack, Health, HitFlash, Hurtbox, Projectile, RunStats,
};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::{Position, body};
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::headless_app;

fn stats(app: &App) -> RunStats {
    app.world().resource::<RunStats>().clone()
}

/// Un nemico e il centro della sua hurtbox.
fn first_enemy(app: &mut App) -> (Entity, Vec2) {
    let world = app.world_mut();
    let (entity, feet, hurtbox) = world
        .query_filtered::<(Entity, &Position, &Hurtbox), With<Enemy>>()
        .iter(world)
        .next()
        .expect("almeno un nemico");
    (entity, hurtbox.center(feet.0))
}

/// Proiettile fermo in `at` con il raggio dell'attacco del compagno iniziale.
fn projectile_at(app: &App, at: Vec2, damage: f32, velocity: Vec2, secs: f32) -> impl Bundle {
    let config = config(app);
    let attack = config.companion_attack(config.starting_companion());
    (
        Projectile {
            velocity,
            damage,
            radius: attack.projectile_radius,
            remaining_secs: secs,
            hits_left: 1,
            already_hit: Vec::new(),
        },
        body(at),
    )
}

fn companion_attack(app: &mut App) -> Mut<'_, AutoAttack> {
    let world = app.world_mut();
    let entity = world
        .query_filtered::<Entity, With<Companion>>()
        .single(world)
        .unwrap();
    world.get_mut::<AutoAttack>(entity).unwrap()
}

#[test]
fn companion_follows_behind_the_player_and_settles() {
    let mut app = headless_app(1);
    app.update();
    let follow = config(&app).starting_companion().follow;
    let speed = config(&app).player.speed;
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 120);
    let (player, companion) = (player_position(&mut app), companion_position(&mut app));
    assert!(companion.x < player.x, "il compagno sta dietro");
    // In corsa resta indietro di distance + velocità/guadagno (75/6 ≈ 12,5 px), più margine.
    let lag = follow.distance + speed / follow.gain;
    assert!(player.distance(companion) < lag + 3.0);

    app.insert_resource(MoveIntent(Vec2::ZERO));
    run_steps(&mut app, 60);
    let gap = player_position(&mut app).distance(companion_position(&mut app));
    assert!((gap - follow.distance).abs() < 1.0, "fermo a {gap} px");
}

#[test]
fn companion_defeats_enemies_without_any_input() {
    let mut app = headless_app(1);
    run_steps(&mut app, 60 * 15);
    let s = stats(&app);
    assert!(s.defeated >= 3, "sconfitti {} in 15 s", s.defeated);
    // 30 HP / 12 danni = 3 colpi per nemico: servono almeno 3 proiettili per sconfitta.
    let config = config(&app);
    let shots_per_kill = (config.stage_enemy().max_hp
        / config.companion_attack(config.starting_companion()).damage)
        .ceil() as u32;
    assert!(s.projectiles_fired >= shots_per_kill * s.defeated, "{s:?}");
}

#[test]
fn a_hit_deals_damage_and_flashes() {
    let mut app = headless_app(1);
    app.update();
    despawn_all::<Companion>(&mut app);
    let max_hp = config(&app).stage_enemy().max_hp;
    let (enemy, center) = first_enemy(&mut app);
    let projectile = projectile_at(&app, center, 12.0, Vec2::ZERO, 1.0);
    app.world_mut().spawn(projectile);
    app.update();
    let health = *app.world().get::<Health>(enemy).unwrap();
    assert_eq!(health.current, max_hp - 12.0);
    assert!(app.world().get::<HitFlash>(enemy).is_some());
    assert_eq!(count::<Projectile>(&mut app), 0, "il proiettile si consuma");
}

#[test]
fn an_enemy_dies_exactly_once_even_if_hit_twice() {
    let mut app = headless_app(1);
    app.update();
    despawn_all::<Companion>(&mut app);
    let cap = config(&app).stage.population_cap as usize;
    let (enemy, center) = first_enemy(&mut app);
    app.world_mut().get_mut::<Health>(enemy).unwrap().current = 1.0;
    for _ in 0..2 {
        let projectile = projectile_at(&app, center, 12.0, Vec2::ZERO, 1.0);
        app.world_mut().spawn(projectile);
    }
    app.update();
    assert_eq!(stats(&app).defeated, 1);
    assert!(app.world().get_entity(enemy).is_err(), "despawnato");
    // Il secondo proiettile ignora il nemico già a 0 HP e resta in volo.
    assert_eq!(count::<Projectile>(&mut app), 1);
    // Il rimpiazzo arriva nello stesso passo: la popolazione resta piena.
    assert_eq!(count::<Enemy>(&mut app), cap);
    run_steps(&mut app, 5);
    assert_eq!(stats(&app).defeated, 1);
}

#[test]
fn attack_fires_at_its_exact_cadence() {
    let mut app = headless_app(1);
    app.update();
    assert_eq!(
        stats(&app).projectiles_fired,
        0,
        "al primo passo nessuno è in raggio"
    );
    let interval = companion_attack(&mut app).interval;
    companion_attack(&mut app).range = 1.0e6; // bersaglio sempre disponibile
    run_steps(&mut app, 599);
    // Spari ai passi 2, 2+k, 2+2k, ... con k = intervallo in passi (1,1 s = 66): entro il
    // passo 600 sono 1 + (600 - 2) / k.
    let step = (interval * 60.0).round() as u32;
    assert_eq!(stats(&app).projectiles_fired, 1 + (600 - 2) / step);
}

#[test]
fn more_projectiles_per_attack_means_more_projectiles_on_screen() {
    let mut app = headless_app(1);
    app.update();
    {
        let mut attack = companion_attack(&mut app);
        attack.range = 1.0e6;
        attack.projectiles = 3;
    }
    app.update();
    assert_eq!(stats(&app).projectiles_fired, 3);
    assert_eq!(count::<Projectile>(&mut app), 3);
}

#[test]
fn missed_projectiles_expire() {
    let mut app = headless_app(1);
    app.update();
    despawn_all::<Companion>(&mut app);
    let projectile = projectile_at(&app, Vec2::new(0.0, 1000.0), 12.0, Vec2::Y * 220.0, 0.5);
    app.world_mut().spawn(projectile);
    run_steps(&mut app, 29);
    assert_eq!(count::<Projectile>(&mut app), 1);
    run_steps(&mut app, 2);
    assert_eq!(count::<Projectile>(&mut app), 0);
}

#[test]
fn population_never_exceeds_the_cap() {
    let mut app = headless_app(5);
    app.update();
    let cap = config(&app).stage.population_cap as usize;
    for _ in 0..60 * 30 {
        app.update();
        assert!(count::<Enemy>(&mut app) <= cap);
    }
    assert_eq!(count::<Enemy>(&mut app), cap);
    assert!(stats(&app).defeated > 0);
}
