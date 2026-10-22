//! Posizioni di simulazione. Il rendering legge `Position`/`PreviousPosition` e interpola,
//! la logica non tocca mai `Transform`.

use bevy::prelude::*;

use super::simulation_running;

/// Posizione in pixel di mondo (1 unità = 1 pixel della risoluzione virtuale), y verso l'alto.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Position(pub Vec2);

/// Posizione al passo fisso precedente, per interpolare il rendering tra due passi.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct PreviousPosition(pub Vec2);

/// Componenti di posizione per un'entità che nasce in `at` (niente scatto al primo frame).
pub fn body(at: Vec2) -> (Position, PreviousPosition) {
    (Position(at), PreviousPosition(at))
}

pub struct KinematicsPlugin;

impl Plugin for KinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            store_previous_positions.run_if(simulation_running),
        );
    }
}

fn store_previous_positions(mut bodies: Query<(&Position, &mut PreviousPosition)>) {
    for (position, mut previous) in &mut bodies {
        previous.0 = position.0;
    }
}
