//! Simulazione di gioco: gira in `FixedUpdate` a 60 Hz e non dipende dal rendering.

pub mod attacks;
pub mod collision;
pub mod combat;
pub mod companions;
pub mod damage;
pub mod enemies;
pub mod input;
pub mod kinematics;
pub mod orbit;
pub mod player;
pub mod progression;
pub mod pulse;
pub mod run;
pub mod spatial;
pub mod spawning;
pub mod threats;
pub mod upgrades;

use bevy::prelude::*;

use crate::app::RunPhase;
use combat::Health;
use enemies::Boss;
use player::Player;
use progression::Progression;
use spawning::SpawnDirector;

/// Frequenza della logica di gioco.
pub const SIM_HZ: f64 = 60.0;

/// La simulazione avanza: run in corso, non in pausa, player vivo, nessun livello in attesa
/// del draft, boss non ancora sconfitto. Il passo in cui il player muore, sale di livello o
/// abbatte il boss è l'ultimo anche se il fotogramma ne prevede altri (tempo accelerato):
/// orologio, colpi e spawn si fermano lì, e sconfitta, vittoria e draft non dipendono dagli FPS.
pub fn simulation_running(
    phase: Option<Res<State<RunPhase>>>,
    progression: Option<Res<Progression>>,
    director: Option<Res<SpawnDirector>>,
    player: Query<&Health, With<Player>>,
    bosses: Query<(), With<Boss>>,
) -> bool {
    let boss_defeated = director.is_some_and(|d| d.boss_spawned) && bosses.is_empty();
    phase.is_some_and(|phase| *phase.get() == RunPhase::Playing)
        && progression.is_none_or(|p| p.pending_levels == 0)
        && player.iter().all(|health| !health.is_dead())
        && !boss_defeated
}

/// Ordine dei sistemi di simulazione dentro `FixedUpdate` (in catena, solo in `Playing`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Il player si muove per primo: chi lo insegue legge la posizione del passo corrente,
    /// in modo deterministico qualunque sia l'ordine di registrazione dei sistemi.
    Player,
    /// Tutte le altre entità si spostano (inseguimento, compagni, proiettili).
    Movement,
    /// Correzioni dopo il movimento: separazione delle orde, ostacoli, limiti dell'arena.
    Resolve,
    /// Attacchi, colpi, danni ai nemici e morti, sulle posizioni già corrette.
    Combat,
    /// Danni al player dai nemici rimasti vivi dopo il combattimento.
    Damage,
    /// Gemme XP attirate e raccolte, rigenerazione.
    Pickup,
    /// Nuovi nemici dello spawn director.
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
                    SimSet::Damage,
                    SimSet::Pickup,
                    SimSet::Spawn,
                )
                    .chain()
                    .run_if(simulation_running),
            )
            .add_plugins((
                kinematics::KinematicsPlugin,
                run::RunPlugin,
                spatial::SpatialPlugin,
                player::PlayerPlugin,
                companions::CompanionsPlugin,
                enemies::EnemyPlugin,
                spawning::SpawningPlugin,
                combat::CombatPlugin,
                damage::DamagePlugin,
                progression::ProgressionPlugin,
                upgrades::UpgradesPlugin,
                threats::ThreatsPlugin,
            ));
    }
}
