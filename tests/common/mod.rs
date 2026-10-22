//! Utilità condivise dai test di simulazione headless.
#![allow(dead_code)] // ogni file di test usa solo una parte di queste funzioni

use bevy::prelude::*;
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::Player;

pub fn run_steps(app: &mut App, steps: usize) {
    for _ in 0..steps {
        app.update();
    }
}

pub fn player_position(app: &mut App) -> Vec2 {
    single_position::<Player>(app)
}

pub fn companion_position(app: &mut App) -> Vec2 {
    single_position::<Companion>(app)
}

fn single_position<T: Component>(app: &mut App) -> Vec2 {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<T>>()
        .single(world)
        .expect("esattamente un'entità")
        .0
}

pub fn enemy_positions(app: &mut App) -> Vec<Vec2> {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<Enemy>>()
        .iter(world)
        .map(|p| p.0)
        .collect()
}

pub fn count<T: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), With<T>>().iter(world).count()
}

/// Rimuove tutte le entità con `T` (es. i compagni, per provare il movimento senza combattere).
pub fn despawn_all<T: Component>(app: &mut App) {
    let world = app.world_mut();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<T>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
}

pub fn mean_distance(points: &[Vec2], to: Vec2) -> f32 {
    points.iter().map(|p| p.distance(to)).sum::<f32>() / points.len() as f32
}
