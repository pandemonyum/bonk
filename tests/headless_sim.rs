//! Simulazione completa senza finestra né GPU: ogni `app.update()` = un passo fisso da 1/60 s.

use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase};
use pixel_monster_survivor::gameplay::enemies::{
    CONTACT_DISTANCE, Enemy, INITIAL_ENEMIES, SEPARATION_RADIUS,
};
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::{MoveIntent, PLAYER_SPEED, Player};
use pixel_monster_survivor::headless_app;
use pixel_monster_survivor::world::ArenaBounds;

fn player_position(app: &mut App) -> Vec2 {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<Player>>()
        .single(world)
        .expect("esattamente un player")
        .0
}

fn enemy_positions(app: &mut App) -> Vec<Vec2> {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<Enemy>>()
        .iter(world)
        .map(|p| p.0)
        .collect()
}

fn mean_distance(enemies: &[Vec2], to: Vec2) -> f32 {
    enemies.iter().map(|e| e.distance(to)).sum::<f32>() / enemies.len() as f32
}

fn run_steps(app: &mut App, steps: usize) {
    for _ in 0..steps {
        app.update();
    }
}

#[test]
fn run_spawns_one_player_and_initial_wave() {
    let mut app = headless_app(1);
    app.update();
    assert_eq!(player_position(&mut app), Vec2::ZERO);
    assert_eq!(enemy_positions(&mut app).len(), INITIAL_ENEMIES);
}

#[test]
fn player_moves_at_configured_speed() {
    let mut app = headless_app(1);
    app.update();
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60);
    let moved = player_position(&mut app).x;
    // 60 passi da 1/60 s: un secondo di movimento, con tolleranza di un passo.
    assert!(
        (moved - PLAYER_SPEED).abs() <= PLAYER_SPEED / 60.0 + 0.01,
        "spostamento {moved}"
    );
}

#[test]
fn player_never_leaves_the_arena() {
    let mut app = headless_app(1);
    app.update();
    let arena = *app.world().resource::<ArenaBounds>();
    app.insert_resource(MoveIntent(Vec2::new(-1.0, 1.0).normalize()));
    // 60 s alla massima velocità: molto più della mezza arena.
    run_steps(&mut app, 60 * 60);
    let at = player_position(&mut app);
    assert!(arena.contains(at), "{at:?}");
    assert!(
        (at - Vec2::new(arena.min.x, arena.max.y)).length() < 0.01,
        "{at:?}"
    );
}

#[test]
fn enemies_never_enter_the_player_against_a_wall() {
    let mut app = headless_app(3);
    app.update();
    let arena = *app.world().resource::<ArenaBounds>();
    // Il player corre fino al muro est (~27 s) e ci resta: i nemici lo schiacciano contro.
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60 * 45);
    let player = player_position(&mut app);
    assert_eq!(player.x, arena.max.x);
    for (i, enemy) in enemy_positions(&mut app).iter().enumerate() {
        let distance = enemy.distance(player);
        assert!(
            distance >= CONTACT_DISTANCE - 0.01,
            "nemico {i} a {distance} px"
        );
        // La respinta può superare i limiti solo di CONTACT_DISTANCE.
        assert!(
            enemy.x <= arena.max.x + CONTACT_DISTANCE + 0.01,
            "nemico {i} oltre il muro"
        );
    }
}

#[test]
fn enemies_chase_the_player_and_do_not_stack() {
    let mut app = headless_app(7);
    app.update();
    let before = mean_distance(&enemy_positions(&mut app), Vec2::ZERO);
    run_steps(&mut app, 60 * 8);
    let enemies = enemy_positions(&mut app);
    let after = mean_distance(&enemies, player_position(&mut app));
    assert!(after < before * 0.25, "prima {before}, dopo {after}");
    // Nessun nemico dentro il player; la separazione (una passata per passo) tiene le coppie
    // almeno a metà di SEPARATION_RADIUS anche nella calca.
    let player = player_position(&mut app);
    for (i, a) in enemies.iter().enumerate() {
        assert!(
            a.distance(player) >= CONTACT_DISTANCE - 0.01,
            "nemico {i} dentro il player: {}",
            a.distance(player)
        );
        for b in &enemies[i + 1..] {
            let gap = a.distance(*b);
            assert!(
                gap > SEPARATION_RADIUS * 0.5,
                "nemici sovrapposti a {gap} px"
            );
        }
    }
}

#[test]
fn same_seed_same_simulation() {
    let simulate = |seed| {
        let mut app = headless_app(seed);
        app.update();
        app.insert_resource(MoveIntent(Vec2::new(0.6, 0.8)));
        run_steps(&mut app, 300);
        let mut enemies = enemy_positions(&mut app);
        enemies.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        (player_position(&mut app), enemies)
    };
    assert_eq!(simulate(42), simulate(42));
    assert_ne!(simulate(42).1, simulate(43).1);
}

#[test]
fn pause_freezes_the_simulation() {
    let mut app = headless_app(1);
    app.update();
    app.insert_resource(MoveIntent(Vec2::Y));
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Paused);
    app.update();
    let frozen = player_position(&mut app);
    run_steps(&mut app, 120);
    assert_eq!(player_position(&mut app), frozen);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );

    // Ripresa: il tempo virtuale riparte e il player torna a muoversi.
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Playing);
    run_steps(&mut app, 60);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    let resumed = player_position(&mut app).y - frozen.y;
    assert!(
        resumed > PLAYER_SPEED * 0.9,
        "dopo la pausa si è mosso di {resumed}"
    );
}
