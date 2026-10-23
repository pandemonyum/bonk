//! Conto alla rovescia al centro in alto fino al boss e, quando arriva, il suo nome con la
//! barra degli HP. Nascono entrando in `InRun` e spariscono uscendo.

use bevy::prelude::*;

use super::hud::XP_BAR_HEIGHT;
use crate::app::AppState;
use crate::data::{DataId, GameConfig};
use crate::gameplay::combat::Health;
use crate::gameplay::enemies::Boss;
use crate::gameplay::run::RunClock;
use crate::gameplay::spawning::SpawnDirector;

/// Conto alla rovescia al centro in alto.
#[derive(Component)]
struct TimerText;

/// Riquadro del boss (nome e barra), visibile solo con il boss in campo.
#[derive(Component)]
struct BossPanel;

#[derive(Component)]
struct BossName;

#[derive(Component)]
struct BossFill;

/// Rosso pericolo (`r` della palette) per il boss e gli ultimi secondi.
const DANGER_RED: Color = Color::srgb_u8(0xe2, 0x57, 0x4c);
/// Sotto questi secondi il conto alla rovescia diventa rosso.
const COUNTDOWN_WARNING_S: f32 = 30.0;

pub struct BossBarPlugin;

impl Plugin for BossBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_timer)
            .add_systems(Update, (show_timer, show_boss));
    }
}

/// Testo del timer: il tempo che manca al boss, poi "BOSS" da quando è arrivato.
pub fn timer_text(clock: &RunClock, duration_s: f32, boss_arrived: bool) -> String {
    if boss_arrived {
        "BOSS".into()
    } else {
        clock.countdown(duration_s)
    }
}

/// Frazione degli HP del boss per la barra (0 senza HP massimi validi).
pub fn boss_fraction(health: &Health) -> f32 {
    if health.max > 0.0 {
        (health.current / health.max).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Timer al centro in alto e, sotto, il riquadro del boss (nascosto finché non arriva).
fn spawn_timer(mut commands: Commands) {
    commands.spawn((
        Name::new("RunTimer"),
        Node {
            position_type: PositionType::Absolute,
            top: px(XP_BAR_HEIGHT + 2.0),
            left: px(0),
            right: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        DespawnOnExit(AppState::InRun),
        children![
            (
                TimerText,
                Text::new(""),
                TextFont::from_font_size(22.0),
                TextColor(Color::WHITE),
            ),
            (
                BossPanel,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(2),
                    ..default()
                },
                Visibility::Hidden,
                children![
                    (
                        BossName,
                        Text::new(""),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::WHITE),
                    ),
                    (
                        Node {
                            width: px(260),
                            height: px(8),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.1, 0.05, 0.1)),
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                        children![(
                            BossFill,
                            Node {
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(DANGER_RED),
                        )],
                    ),
                ],
            ),
        ],
    ));
}

fn show_timer(
    clock: Res<RunClock>,
    config: Option<Res<GameConfig>>,
    director: Option<Res<SpawnDirector>>,
    timer: Single<(&mut Text, &mut TextColor), With<TimerText>>,
) {
    let Some(config) = config else { return };
    let duration = config.stage.run.duration_s;
    let arrived = director.is_some_and(|d| d.boss_spawned);
    let (mut text, mut color) = timer.into_inner();
    let line = timer_text(&clock, duration, arrived);
    if text.0 != line {
        text.0 = line;
    }
    let urgent = arrived || duration - clock.elapsed <= COUNTDOWN_WARNING_S;
    let wanted = if urgent { DANGER_RED } else { Color::WHITE };
    if color.0 != wanted {
        color.0 = wanted;
    }
}

fn show_boss(
    config: Option<Res<GameConfig>>,
    boss: Query<(&Health, &DataId), With<Boss>>,
    mut panel: Single<&mut Visibility, With<BossPanel>>,
    mut name: Single<&mut Text, With<BossName>>,
    mut fill: Single<&mut Node, With<BossFill>>,
) {
    let Ok((health, id)) = boss.single() else {
        **panel = Visibility::Hidden;
        return;
    };
    **panel = Visibility::Inherited;
    let title = config
        .as_ref()
        .and_then(|c| c.enemy(&id.0))
        .map_or(id.0.clone(), |e| e.display_name.clone());
    if name.0 != title {
        name.0 = title;
    }
    fill.width = percent(100.0 * boss_fraction(health));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_counts_down_then_announces_the_boss() {
        let clock = RunClock { elapsed: 90.0 };
        assert_eq!(timer_text(&clock, 600.0, false), "08:30");
        assert_eq!(timer_text(&clock, 600.0, true), "BOSS");
        let full = Health::full(500.0);
        assert_eq!(boss_fraction(&full), 1.0);
        let half = Health {
            current: 250.0,
            max: 500.0,
        };
        assert_eq!(boss_fraction(&half), 0.5);
        let dead = Health {
            current: -3.0,
            max: 500.0,
        };
        assert_eq!(boss_fraction(&dead), 0.0);
    }
}
