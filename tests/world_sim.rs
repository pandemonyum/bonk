//! P0.1 headless: ostacoli generati dal seed, collisioni di player e nemici.

mod common;

use bevy::prelude::*;
use common::{count, enemy_positions, player_position, run_steps};
use pixel_monster_survivor::gameplay::collision::{ENEMY_FOOTPRINT, PLAYER_FOOTPRINT};
use pixel_monster_survivor::gameplay::enemies::CONTACT_DISTANCE;
use pixel_monster_survivor::gameplay::kinematics::{Position, PreviousPosition};
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::headless_app;
use pixel_monster_survivor::world::obstacles::{
    Obstacle, ObstacleField, ObstacleSpec, generate_obstacles,
};

fn field(app: &App) -> ObstacleField {
    app.world().resource::<ObstacleField>().clone()
}

fn teleport_player(app: &mut App, to: Vec2) {
    let world = app.world_mut();
    let player = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .unwrap();
    world
        .entity_mut(player)
        .insert((Position(to), PreviousPosition(to)));
}

fn nearest_obstacle(field: &ObstacleField, to: Vec2) -> ObstacleSpec {
    *field
        .obstacles
        .iter()
        .min_by(|a, b| {
            let (da, db) = (a.base.distance(to), b.base.distance(to));
            da.total_cmp(&db)
        })
        .unwrap()
}

#[test]
fn the_run_spawns_the_seeded_obstacles() {
    let mut app = headless_app(21);
    app.update();
    let field = field(&app);
    assert_eq!(field.obstacles, generate_obstacles(21));
    assert_eq!(count::<Obstacle>(&mut app), field.obstacles.len());
}

#[test]
fn player_slides_around_an_obstacle_without_entering_it() {
    let mut app = headless_app(1);
    app.update();
    let obstacle = nearest_obstacle(&field(&app), Vec2::ZERO);
    let center = obstacle.footprint_center();
    let min_distance = obstacle.footprint_radius() + PLAYER_FOOTPRINT.0;
    // Leggermente fuori asse: chi punta dritto al centro si ferma, chi è di lato scivola.
    teleport_player(&mut app, center + Vec2::new(-30.0, 2.0));
    app.insert_resource(MoveIntent(Vec2::X));
    for _ in 0..90 {
        app.update();
        let at = player_position(&mut app);
        assert!(
            at.distance(center) >= min_distance - 1e-3,
            "dentro l'ostacolo: {at:?}"
        );
    }
    assert!(
        player_position(&mut app).x > center.x + 10.0,
        "non è riuscito a passare"
    );
}

#[test]
fn a_player_aimed_at_the_center_stops_at_the_edge() {
    let mut app = headless_app(1);
    app.update();
    let obstacle = nearest_obstacle(&field(&app), Vec2::ZERO);
    let center = obstacle.footprint_center();
    teleport_player(&mut app, center + Vec2::new(-30.0, 0.0));
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60);
    let at = player_position(&mut app);
    let expected = obstacle.footprint_radius() + PLAYER_FOOTPRINT.0;
    assert!((at.distance(center) - expected).abs() < 1e-3, "{at:?}");
}

#[test]
fn enemies_stay_out_of_obstacles_within_the_documented_bound() {
    let mut app = headless_app(8);
    app.update();
    let field = field(&app);
    // La respinta dal player viene per ultima e può spingere un nemico dentro un'impronta
    // di al massimo CONTACT_DISTANCE (docs/DECISIONS.md, D-021).
    let tolerance = CONTACT_DISTANCE + 1e-3;
    for step in 0..60 * 20 {
        if step % 30 == 0 {
            let t = step as f32 / 60.0;
            app.insert_resource(MoveIntent(Vec2::from_angle(t * 0.7)));
        }
        app.update();
        for enemy in enemy_positions(&mut app) {
            for obstacle in field.near(enemy, ENEMY_FOOTPRINT.0) {
                let min = obstacle.footprint_radius() + ENEMY_FOOTPRINT.0;
                let d = enemy.distance(obstacle.footprint_center());
                assert!(
                    d >= min - tolerance,
                    "passo {step}: nemico a {d} px (min {min})"
                );
            }
        }
    }
}
