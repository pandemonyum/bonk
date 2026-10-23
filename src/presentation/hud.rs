//! HUD della run: livello, HP e tempo, barra XP, promemoria comandi, pausa (timer e boss in
//! `boss_bar`); stato dei dati (caricamento ed errori) sempre. Gli
//! elementi della run nascono entrando in `InRun` e spariscono uscendo: nel menu non restano.
//! L'HUD di gioco completo (compagni, abilità) arriva con la P0.7.

use bevy::prelude::*;

use crate::app::{AppState, RunPhase};
use crate::data::DataStatus;
use crate::gameplay::combat::{Health, RunStats};
use crate::gameplay::player::Player;
use crate::gameplay::progression::Progression;
use crate::gameplay::run::RunClock;

const HINT: &str = "WASD/frecce muovi   Esc pausa   F5 ricomincia   F3 debug   F12 screenshot";

#[derive(Component)]
struct DataStatusPanel;

#[derive(Component)]
struct VitalsText;

/// Parte piena della barra XP.
#[derive(Component)]
struct XpFill;

/// Altezza della barra XP sul bordo superiore della finestra.
pub(super) const XP_BAR_HEIGHT: f32 = 5.0;
/// Blu delle gemme (`u` della palette): la barra si legge come "esperienza".
const XP_BLUE: Color = Color::srgb_u8(0x4f, 0xa4, 0xe0);
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_data_status_panel)
            .add_systems(
                OnEnter(AppState::InRun),
                (spawn_hint, spawn_xp_bar, spawn_vitals),
            )
            .add_systems(OnEnter(RunPhase::Paused), spawn_pause_banner)
            .add_systems(
                Update,
                (
                    show_vitals,
                    show_xp.run_if(resource_changed::<Progression>),
                    show_data_status.run_if(resource_changed::<DataStatus>),
                ),
            );
    }
}

/// Riga in alto a sinistra: "Lv 3   HP 85/100   02:31   sconfitti 57".
pub fn vitals_text(
    level: u32,
    health: Option<&Health>,
    clock: &RunClock,
    stats: &RunStats,
) -> String {
    let hp = health.map_or("-".to_string(), |h| {
        format!("{:.0}/{:.0}", h.current.max(0.0), h.max)
    });
    format!(
        "Lv {level}   HP {hp}   {}   sconfitti {}",
        clock.label(),
        stats.defeated
    )
}

fn spawn_xp_bar(mut commands: Commands) {
    commands.spawn((
        Name::new("XpBar"),
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: px(XP_BAR_HEIGHT),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        DespawnOnExit(AppState::InRun),
        children![(
            XpFill,
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(XP_BLUE),
        )],
    ));
}

fn show_xp(progression: Res<Progression>, mut fill: Single<&mut Node, With<XpFill>>) {
    fill.width = percent(100.0 * progression.fraction().clamp(0.0, 1.0));
}

fn spawn_vitals(mut commands: Commands) {
    commands.spawn((
        Name::new("Vitals"),
        VitalsText,
        Text::new(""),
        TextFont::from_font_size(14.0),
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        Node {
            position_type: PositionType::Absolute,
            top: px(XP_BAR_HEIGHT + 4.0),
            left: px(8),
            padding: UiRect::axes(px(6), px(2)),
            ..default()
        },
        DespawnOnExit(AppState::InRun),
    ));
}

fn show_vitals(
    clock: Res<RunClock>,
    stats: Res<RunStats>,
    progression: Res<Progression>,
    player: Query<&Health, With<Player>>,
    mut text: Single<&mut Text, With<VitalsText>>,
) {
    let line = vitals_text(progression.level, player.single().ok(), &clock, &stats);
    if text.0 != line {
        text.0 = line;
    }
}

/// Testo per lo stato dei dati; `None` quando non c'è nulla da mostrare.
pub fn data_status_text(status: &DataStatus) -> Option<String> {
    match status {
        DataStatus::Ready => None,
        DataStatus::Loading => Some("Caricamento dati...".into()),
        DataStatus::Error(errors) => Some(format!(
            "Errori nei dati di gioco (game-data/):\n\n{}\n\nCorreggi e salva il file: con la \
             ricarica a caldo (default nella build di sviluppo) il gioco riparte da solo, \
             altrimenti riavvialo.",
            errors
                .iter()
                .map(|e| format!("- {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        )),
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
        DespawnOnExit(AppState::InRun),
    ));
}

fn spawn_data_status_panel(mut commands: Commands) {
    commands.spawn((
        Name::new("DataStatus"),
        DataStatusPanel,
        Text::new(""),
        TextFont::from_font_size(14.0),
        TextColor(Color::srgb(1.0, 0.85, 0.8)),
        BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: px(40),
            left: px(24),
            right: px(24),
            padding: UiRect::all(px(10)),
            ..default()
        },
        // sopra menu e schermate
        GlobalZIndex(10),
        Visibility::Hidden,
    ));
}

fn show_data_status(
    status: Res<DataStatus>,
    panel: Single<(&mut Text, &mut Visibility), With<DataStatusPanel>>,
) {
    let (mut text, mut visibility) = panel.into_inner();
    match data_status_text(&status) {
        Some(message) => {
            text.0 = message;
            *visibility = Visibility::Visible;
        }
        None => *visibility = Visibility::Hidden,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vitals_line() {
        let health = Health {
            current: 84.6,
            max: 100.0,
        };
        let stats = RunStats {
            defeated: 57,
            ..default()
        };
        let clock = RunClock { elapsed: 151.0 };
        assert_eq!(
            vitals_text(3, Some(&health), &clock, &stats),
            "Lv 3   HP 85/100   02:31   sconfitti 57"
        );
        let dead = Health {
            current: -7.0,
            max: 100.0,
        };
        assert!(vitals_text(1, Some(&dead), &clock, &stats).contains("HP 0/100"));
        assert!(vitals_text(1, None, &clock, &stats).contains("HP -"));
    }

    #[test]
    fn status_messages() {
        assert_eq!(data_status_text(&DataStatus::Ready), None);
        assert!(data_status_text(&DataStatus::Loading).is_some());
        let error = data_status_text(&DataStatus::Error(vec!["enemies.ron: x".into()])).unwrap();
        assert!(error.contains("- enemies.ron: x"));
    }
}
