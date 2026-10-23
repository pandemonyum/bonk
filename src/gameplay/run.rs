//! Stato della run: orologio di simulazione (si ferma in pausa e a run finita), conto alla
//! rovescia fino al boss, vittoria quando il boss nato a fine durata non c'è più.

use bevy::prelude::*;

use super::combat::Health;
use super::enemies::Boss;
use super::player::Player;
use super::simulation_running;
use super::spawning::SpawnDirector;
use crate::app::{AppState, RunPhase};

/// Secondi di gioco trascorsi nella run (tempo simulato, non reale).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct RunClock {
    pub elapsed: f32,
}

/// "mm:ss" di una durata in secondi (arrotondata per difetto, mai negativa).
pub fn mm_ss(secs: f32) -> String {
    let total = secs.max(0.0) as u32;
    format!("{:02}:{:02}", total / 60, total % 60)
}

impl RunClock {
    /// Tempo trascorso, "mm:ss", per HUD e risultati.
    pub fn label(&self) -> String {
        mm_ss(self.elapsed)
    }

    /// Tempo che manca al boss, "mm:ss" (per eccesso: "00:01" fino all'ultimo istante).
    pub fn countdown(&self, duration_s: f32) -> String {
        mm_ss((duration_s - self.elapsed).max(0.0).ceil())
    }
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunClock>()
            .add_systems(OnEnter(AppState::InRun), reset_clock)
            .add_systems(FixedPreUpdate, tick_clock.run_if(simulation_running))
            // dopo la sconfitta: se player e boss cadono insieme vince la sconfitta (`alive`)
            .add_systems(
                PostUpdate,
                declare_victory.after(super::damage::declare_defeat),
            );
    }
}

fn reset_clock(mut clock: ResMut<RunClock>) {
    *clock = RunClock::default();
}

fn tick_clock(time: Res<Time>, mut clock: ResMut<RunClock>) {
    clock.elapsed += time.delta_secs();
}

/// Vittoria: il boss è nato e non c'è più, con il player vivo. Come la sconfitta si decide in
/// `PostUpdate`, dopo l'input e il draft, così vince su una pausa o un livello chiesti nello
/// stesso fotogramma; da sconfitti non si vince.
fn declare_victory(
    phase: Option<Res<State<RunPhase>>>,
    next: Option<ResMut<NextState<RunPhase>>>,
    director: Option<Res<SpawnDirector>>,
    bosses: Query<(), With<Boss>>,
    player: Query<&Health, With<Player>>,
) {
    let (Some(phase), Some(mut next), Some(director)) = (phase, next, director) else {
        return;
    };
    let over = matches!(phase.get(), RunPhase::Defeated | RunPhase::Victory);
    let alive = player.iter().all(|health| !health.is_dead());
    if !over && alive && director.boss_spawned && bosses.is_empty() {
        next.set(RunPhase::Victory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_is_minutes_and_seconds() {
        assert_eq!(RunClock { elapsed: 0.0 }.label(), "00:00");
        assert_eq!(RunClock { elapsed: 61.9 }.label(), "01:01");
        assert_eq!(RunClock { elapsed: 600.0 }.label(), "10:00");
    }

    #[test]
    fn the_countdown_reaches_zero_at_the_boss() {
        assert_eq!(RunClock { elapsed: 0.0 }.countdown(600.0), "10:00");
        assert_eq!(RunClock { elapsed: 0.4 }.countdown(600.0), "10:00");
        assert_eq!(RunClock { elapsed: 599.5 }.countdown(600.0), "00:01");
        assert_eq!(RunClock { elapsed: 650.0 }.countdown(600.0), "00:00");
    }
}
