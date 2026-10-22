//! Simulazione di gioco: gira in `FixedUpdate` a 60 Hz e non dipende dal rendering.

pub mod combat;
pub mod companions;
pub mod enemies;
pub mod input;
pub mod kinematics;
pub mod player;

use bevy::prelude::*;

use crate::app::RunPhase;

/// Frequenza della logica di gioco.
pub const SIM_HZ: f64 = 60.0;

/// Ordine dei sistemi di simulazione dentro `FixedUpdate` (in catena, solo in `Playing`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Il player si muove per primo: chi lo insegue legge la posizione del passo corrente,
    /// in modo deterministico qualunque sia l'ordine di registrazione dei sistemi.
    Player,
    /// Tutte le altre entità si spostano (inseguimento, compagni, proiettili).
    Movement,
    /// Correzioni dopo il movimento: separazione delle orde, limiti dell'arena.
    Resolve,
    /// Attacchi, colpi, danni e morti, sulle posizioni già corrette.
    Combat,
    /// Nuove entità della run (rimpiazzo dei nemici; spawn director in P0.3).
    Spawn,
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(SIM_HZ))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::Player,
                    SimSet::Movement,
                    SimSet::Resolve,
                    SimSet::Combat,
                    SimSet::Spawn,
                )
                    .chain()
                    .run_if(in_state(RunPhase::Playing)),
            )
            .add_plugins((
                kinematics::KinematicsPlugin,
                player::PlayerPlugin,
                companions::CompanionsPlugin,
                enemies::EnemyPlugin,
                combat::CombatPlugin,
            ));
    }
}
