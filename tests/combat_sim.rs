//! P0.2 headless: compagno che segue e attacca da solo, danni, morte, popolazione.

mod common;

use bevy::prelude::*;
use common::{companion_position, count, despawn_all, player_position, run_steps};
use pixel_monster_survivor::gameplay::combat::{
    AutoAttack, Health, HitFlash, PROJECTILE_RADIUS, Projectile, RunStats,
};
use pixel_monster_survivor::gameplay::companions::{Companion, FOLLOW_DISTANCE};
use pixel_monster_survivor::gameplay::enemies::{
    ENEMY_HURTBOX, ENEMY_MAX_HP, ENEMY_POPULATION, Enemy,
};
use pixel_monster_survivor::gameplay::kinematics::{Position, body};
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::headless_app;

fn stats(app: &App) -> RunStats {
    app.world().resource::<RunStats>().clone()
}

fn first_enemy(app: &mut App) -> (Entity, Vec2) {
    let world = app.world_mut();
    let (entity, feet) = world
        .query_filtered::<(Entity, &Position), With<Enemy>>()
        .iter(world)
        .next()
        .expect("almeno un nemico");
    (entity, feet.0)
}

/// Proiettile fermo, già sovrapposto alla hurtbox del nemico con i piedi in `feet`.
fn projectile_on(feet: Vec2, damage: f32) -> impl Bundle {
    (
        Projectile {
            velocity: Vec2::ZERO,
            damage,
            radius: PROJECTILE_RADIUS,
            remaining_secs: 1.0,
            hits_left: 1,
            already_hit: Vec::new(),
        },
        body(ENEMY_HURTBOX.center(feet)),
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
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 120);
    let (player, companion) = (player_position(&mut app), companion_position(&mut app));
    assert!(companion.x < player.x, "il compagno sta dietro");
    // In corsa resta indietro di FOLLOW_DISTANCE + velocità/guadagno (75/6 ≈ 12,5 px).
    assert!(player.distance(companion) < FOLLOW_DISTANCE + 15.0);

    app.insert_resource(MoveIntent(Vec2::ZERO));
    run_steps(&mut app, 60);
    let gap = player_position(&mut app).distance(companion_position(&mut app));
    assert!((gap - FOLLOW_DISTANCE).abs() < 1.0, "fermo a {gap} px");
}

#[test]
fn companion_defeats_enemies_without_any_input() {
    let mut app = headless_app(1);
    run_steps(&mut app, 60 * 15);
    let s = stats(&app);
    assert!(s.defeated >= 3, "sconfitti {} in 15 s", s.defeated);
    // 30 HP / 12 danni = 3 colpi per nemico: servono almeno 3 proiettili per sconfitta.
    assert!(s.projectiles_fired >= 3 * s.defeated, "{s:?}");
}

#[test]
fn a_hit_deals_damage_and_flashes() {
    let mut app = headless_app(1);
    app.update();
    despawn_all::<Companion>(&mut app);
    let (enemy, feet) = first_enemy(&mut app);
    app.world_mut().spawn(projectile_on(feet, 12.0));
    app.update();
    let health = *app.world().get::<Health>(enemy).unwrap();
    assert_eq!(health.current, ENEMY_MAX_HP - 12.0);
    assert!(app.world().get::<HitFlash>(enemy).is_some());
    assert_eq!(count::<Projectile>(&mut app), 0, "il proiettile si consuma");
}

#[test]
fn an_enemy_dies_exactly_once_even_if_hit_twice() {
    let mut app = headless_app(1);
    app.update();
    despawn_all::<Companion>(&mut app);
    let (enemy, feet) = first_enemy(&mut app);
    app.world_mut().get_mut::<Health>(enemy).unwrap().current = 1.0;
    app.world_mut().spawn(projectile_on(feet, 12.0));
    app.world_mut().spawn(projectile_on(feet, 12.0));
    app.update();
    assert_eq!(stats(&app).defeated, 1);
    assert!(app.world().get_entity(enemy).is_err(), "despawnato");
    // Il secondo proiettile ignora il nemico già a 0 HP e resta in volo.
    assert_eq!(count::<Projectile>(&mut app), 1);
    // Il rimpiazzo arriva nello stesso passo: la popolazione resta piena.
    assert_eq!(count::<Enemy>(&mut app), ENEMY_POPULATION);
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
    companion_attack(&mut app).range = 1.0e6; // bersaglio sempre disponibile
    run_steps(&mut app, 599);
    // 1,1 s = 66 passi: spari ai passi 2, 68, ..., 596 → 10 entro il passo 600.
    assert_eq!(stats(&app).projectiles_fired, 10);
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
    app.world_mut().spawn((
        Projectile {
            velocity: Vec2::new(0.0, 220.0),
            damage: 12.0,
            radius: PROJECTILE_RADIUS,
            remaining_secs: 0.5,
            hits_left: 1,
            already_hit: Vec::new(),
        },
        body(Vec2::new(0.0, 1000.0)),
    ));
    run_steps(&mut app, 29);
    assert_eq!(count::<Projectile>(&mut app), 1);
    run_steps(&mut app, 2);
    assert_eq!(count::<Projectile>(&mut app), 0);
}

#[test]
fn population_never_exceeds_the_cap() {
    let mut app = headless_app(5);
    for _ in 0..60 * 30 {
        app.update();
        assert!(count::<Enemy>(&mut app) <= ENEMY_POPULATION);
    }
    assert_eq!(count::<Enemy>(&mut app), ENEMY_POPULATION);
    assert!(stats(&app).defeated > 0);
}
