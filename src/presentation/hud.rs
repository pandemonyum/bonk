//! HUD minimo S0: promemoria comandi e schermata di pausa. L'HUD completo arriva con la P0.7.

use bevy::prelude::*;

use crate::app::RunPhase;

const HINT: &str = "WASD/frecce muovi   Esc pausa   F3 debug   F12 screenshot";

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hint)
            .add_systems(OnEnter(RunPhase::Paused), spawn_pause_banner);
    }
}

fn spawn_hint(mut commands: Commands) {
    commands.spawn((
        Name::new("ControlsHint"),
        Text::new(HINT),
        TextFont::from_font_size(12.0),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(6),
            left: px(8),
            ..default()
        },
    ));
}

fn spawn_pause_banner(mut commands: Commands) {
    commands.spawn((
        Name::new("PauseBanner"),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        DespawnOnExit(RunPhase::Paused),
        children![(
            Text::new("PAUSA - Esc per riprendere"),
            TextFont::from_font_size(28.0),
            TextColor(Color::WHITE),
        )],
    ));
}
