//! Overlay di debug (F3): FPS, frame-time, nemici ed entità attive, seed, fase della run.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::LaunchOptions;
use crate::app::{RunPhase, RunSeed};
use crate::gameplay::enemies::Enemy;
use crate::gameplay::kinematics::Position;
use crate::gameplay::player::Player;

/// Aggiornamento del testo 4 volte al secondo: leggibile e senza costo per frame.
const REFRESH_SECS: f32 = 0.25;

#[derive(Component)]
struct OverlayText;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_overlay)
            .add_systems(Update, (toggle_overlay, refresh_overlay));
    }
}

fn spawn_overlay(mut commands: Commands, options: Res<LaunchOptions>) {
    commands.spawn((
        Name::new("DebugOverlay"),
        OverlayText,
        Text::new("..."),
        TextFont::from_font_size(12.0),
        TextColor(Color::srgb(1.0, 1.0, 0.6)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: px(6),
            right: px(6),
            padding: UiRect::all(px(4)),
            ..default()
        },
        if options.show_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
}

fn toggle_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: Single<&mut Visibility, With<OverlayText>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        overlay.toggle_visible_hidden();
    }
}

fn refresh_overlay(
    time: Res<Time<Real>>,
    mut since_refresh: Local<f32>,
    diagnostics: Res<DiagnosticsStore>,
    seed: Res<RunSeed>,
    phase: Option<Res<State<RunPhase>>>,
    enemies: Query<(), With<Enemy>>,
    entities: Query<()>,
    player: Query<&Position, With<Player>>,
    overlay: Single<(&mut Text, &Visibility), With<OverlayText>>,
) {
    *since_refresh += time.delta_secs();
    let (mut text, visibility) = overlay.into_inner();
    if *since_refresh < REFRESH_SECS || *visibility == Visibility::Hidden {
        return;
    }
    *since_refresh = 0.0;
    let smoothed = |path| {
        diagnostics
            .get(path)
            .and_then(|d| d.smoothed())
            .unwrap_or(f64::NAN)
    };
    let phase = phase.map_or("-".to_string(), |p| format!("{:?}", p.get()));
    let player = player
        .single()
        .map_or("-".to_string(), |p| format!("{:.0},{:.0}", p.x, p.y));
    text.0 = format!(
        "FPS {:.0}  frame {:.2} ms\nnemici {}  entita {}\nplayer {}  fase {}\nseed {}",
        smoothed(&FrameTimeDiagnosticsPlugin::FPS),
        smoothed(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
        enemies.iter().count(),
        entities.iter().count(),
        player,
        phase,
        seed.0,
    );
}
