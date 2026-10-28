//! Annuncio delle orde in alto al centro: "ARRIVA L'ORDA! 3" nei secondi di preavviso, poi
//! "L'ORDA È QUI!" finché nascono i nemici (`gameplay::hordes::horde_alert`). Il testo pulsa
//! tra rosso e oro. Tiny5 a 32 px.

use bevy::prelude::*;

use super::fonts;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::gameplay::hordes::{HordeAlert, horde_alert};
use crate::gameplay::run::RunClock;

const RED: Color = Color::srgb(0.95, 0.32, 0.28);
const GOLD: Color = Color::srgb(0.95, 0.78, 0.3);
/// Pulsazioni al secondo.
const PULSE_HZ: f32 = 2.0;

#[derive(Component)]
struct HordeBanner;

/// Il testo dell'annuncio; vuoto senza orde vicine.
pub fn banner_text(alert: Option<HordeAlert>) -> String {
    match alert {
        Some(HordeAlert::Coming { in_s, .. }) => {
            format!("ARRIVA L'ORDA!  {}", in_s.ceil().max(1.0) as u32)
        }
        Some(HordeAlert::Here { .. }) => "L'ORDA È QUI!".to_string(),
        None => String::new(),
    }
}

pub struct HordeBannerPlugin;

impl Plugin for HordeBannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_banner)
            .add_systems(Update, show_banner.run_if(in_state(AppState::InRun)));
    }
}

fn spawn_banner(mut commands: Commands) {
    commands.spawn((
        Name::new("HordeBanner"),
        Node {
            position_type: PositionType::Absolute,
            top: px(72),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        DespawnOnExit(AppState::InRun),
        children![(HordeBanner, Text::new(""), fonts::hud(32.0), TextColor(RED))],
    ));
}

fn show_banner(
    config: Res<GameConfig>,
    clock: Res<RunClock>,
    time: Res<Time<Real>>,
    mut banner: Single<(&mut Text, &mut TextColor), With<HordeBanner>>,
) {
    let line = banner_text(horde_alert(&config.stage.run.hordes, clock.elapsed));
    let (text, color) = &mut *banner;
    if text.0 != line {
        text.0 = line;
    }
    let pulse = (time.elapsed_secs() * PULSE_HZ * std::f32::consts::TAU).sin() > 0.0;
    color.0 = if pulse { RED } else { GOLD };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banner_counts_down_then_announces_the_horde() {
        assert_eq!(banner_text(None), "");
        let coming = |in_s| Some(HordeAlert::Coming { index: 0, in_s });
        assert_eq!(banner_text(coming(3.2)), "ARRIVA L'ORDA!  4");
        assert_eq!(banner_text(coming(0.1)), "ARRIVA L'ORDA!  1");
        assert_eq!(
            banner_text(Some(HordeAlert::Here { index: 1 })),
            "L'ORDA È QUI!"
        );
    }
}
