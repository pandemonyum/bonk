//! Simulazione di gioco: gira in `FixedUpdate` a 60 Hz e non dipende dal rendering.

pub mod enemies;
pub mod input;
pub mod kinematics;
pub mod player;

use bevy::prelude::*;

use crate::app::RunPhase;

/// Frequenza della logica di gioco.
pub const SIM_HZ: f64 = 60.0;

/// Ordine dei sistemi di simulazione dentro `FixedUpdate`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Il player si muove per primo: chi lo insegue legge la posizione del passo corrente,
    /// in modo deterministico qualunque sia l'ordine di registrazione dei sistemi.
    Player,
    /// Tutte le altre entità calcolano il proprio spostamento (inseguimento, ...).
    Movement,
    /// Correzioni dopo il movimento: separazione delle orde, limiti dell'arena.
    Resolve,
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(SIM_HZ))
            .configure_sets(
                FixedUpdate,
                (SimSet::Player, SimSet::Movement, SimSet::Resolve)
                    .chain()
                    .run_if(in_state(RunPhase::Playing)),
            )
            .add_plugins((
                kinematics::KinematicsPlugin,
                player::PlayerPlugin,
                enemies::EnemyPlugin,
            ));
    }
}
