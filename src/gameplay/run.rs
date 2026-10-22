//! Stato della run: orologio di simulazione (si ferma in pausa e a run finita). Il timer dei
//! 10 minuti, la vittoria e la schermata dei risultati arrivano con la P0.6.

use bevy::prelude::*;

use super::simulation_running;
use crate::app::AppState;

/// Secondi di gioco trascorsi nella run (tempo simulato, non reale).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct RunClock {
    pub elapsed: f32,
}

impl RunClock {
    /// "mm:ss" per HUD e schermate.
    pub fn label(&self) -> String {
        let total = self.elapsed.max(0.0) as u32;
        format!("{:02}:{:02}", total / 60, total % 60)
    }
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunClock>()
            .add_systems(OnEnter(AppState::InRun), reset_clock)
            .add_systems(FixedPreUpdate, tick_clock.run_if(simulation_running));
    }
}

fn reset_clock(mut clock: ResMut<RunClock>) {
    *clock = RunClock::default();
}

fn tick_clock(time: Res<Time>, mut clock: ResMut<RunClock>) {
    clock.elapsed += time.delta_secs();
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
}
