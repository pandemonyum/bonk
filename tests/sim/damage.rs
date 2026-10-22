//! P0.3 headless: danni al player. Contatto, somma dei nemici, finestra di invulnerabilità,
//! modalità invulnerabile, sconfitta che ferma la simulazione, restart dopo la sconfitta.

use crate::common::{config, despawn_all, enemy_positions, god, player_position, quiet_config};
use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::gameplay::combat::Health;
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::{Position, PreviousPosition};
use pixel_monster_survivor::gameplay::player::Player;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::headless_app_with;

/// Run senza nemici nuovi né compagno, con `count` nemici fermi a contatto attorno al player.
fn surrounded(count: usize) -> App {
    let mut app = headless_app_with(1, quiet_config());
    app.update();
    despawn_all::<Companion>(&mut app);
    let world = app.world_mut();
    let enemies: Vec<Entity> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    assert!(enemies.len() >= count);
    for (i, &entity) in enemies.iter().enumerate() {
        if i >= count {
            world.despawn(entity);
            continue;
        }
        let contact = world.get::<Enemy>(entity).unwrap().contact_distance;
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let at = Vec2::from_angle(angle) * contact;
        world
            .entity_mut(entity)
            .insert((Position(at), PreviousPosition(at)));
        world.get_mut::<Enemy>(entity).unwrap().speed = 0.0;
    }
    app
}

fn player_hp(app: &mut App) -> f32 {
    let world = app.world_mut();
    world
        .query_filtered::<&Health, With<Player>>()
        .single(world)
        .unwrap()
        .current
}

fn phase(app: &App) -> RunPhase {
    *app.world().resource::<State<RunPhase>>().get()
}

#[test]
fn contact_hurts_once_per_invulnerability_window() {
    let mut app = surrounded(1);
    let config = config(&app);
    let (max, damage) = (
        config.player.max_hp,
        config.first_wave_enemy().contact_damage,
    );
    let window = (config.player.invulnerability_s * 60.0).round() as usize;
    app.update();
    assert_eq!(player_hp(&mut app), max - damage);
    // Durante la finestra di invulnerabilità niente danni...
    for _ in 0..window - 3 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max - damage);
    // ...poi il nemico, ancora a contatto, colpisce di nuovo.
    for _ in 0..6 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max - 2.0 * damage);
}

#[test]
fn damage_from_several_touching_enemies_adds_up() {
    let mut app = surrounded(3);
    let config = config(&app);
    let (max, damage) = (
        config.player.max_hp,
        config.first_wave_enemy().contact_damage,
    );
    app.update();
    assert_eq!(player_hp(&mut app), max - 3.0 * damage);
}

#[test]
fn enemies_out_of_contact_do_no_damage() {
    let mut app = surrounded(1);
    let max = config(&app).player.max_hp;
    let world = app.world_mut();
    let enemy = world
        .query_filtered::<Entity, With<Enemy>>()
        .single(world)
        .unwrap();
    let far = Vec2::new(60.0, 0.0);
    world
        .entity_mut(enemy)
        .insert((Position(far), PreviousPosition(far)));
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max);
}

#[test]
fn god_mode_takes_no_damage() {
    let mut app = surrounded(3);
    god(&mut app);
    let max = config(&app).player.max_hp;
    for _ in 0..120 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max);
}

#[test]
fn the_run_is_lost_at_zero_hp_and_the_simulation_stops() {
    let mut app = surrounded(2);
    {
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        world.get_mut::<Health>(player).unwrap().current = 3.0;
    }
    app.update(); // colpo mortale: la fase cambia al frame successivo
    app.update();
    assert_eq!(phase(&app), RunPhase::Defeated);
    let clock = app.world().resource::<RunClock>().elapsed;
    let (player, enemies) = (player_position(&mut app), enemy_positions(&mut app));
    for _ in 0..60 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<RunClock>().elapsed,
        clock,
        "orologio fermo"
    );
    assert_eq!(player_position(&mut app), player);
    assert_eq!(enemy_positions(&mut app), enemies);
}

#[test]
fn restarting_after_a_defeat_gives_a_fresh_run() {
    let mut app = surrounded(3);
    {
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        world.get_mut::<Health>(player).unwrap().current = 1.0;
    }
    app.update();
    app.update();
    assert_eq!(phase(&app), RunPhase::Defeated);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    app.update();
    app.update();
    assert_eq!(phase(&app), RunPhase::Playing);
    assert_eq!(player_hp(&mut app), config(&app).player.max_hp);
    assert_eq!(app.world().resource::<RunClock>().elapsed, 1.0 / 60.0);
}
