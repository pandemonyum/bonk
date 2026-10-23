//! HUD della run: promemoria della pausa e schermata di pausa con i comandi e gli upgrade presi
//! (stato del giocatore in `status`, abilità in `abilities`, timer e boss in `boss_bar`); stato
//! dei dati (caricamento ed errori) sempre. Gli elementi della run nascono entrando in `InRun` e
//! spariscono uscendo: nel menu non restano.

use bevy::prelude::*;

use super::fonts;
use super::results::build_summary;
use crate::app::{AppState, RunPhase};
use crate::data::{DataStatus, GameConfig};
use crate::gameplay::upgrades::UpgradeStacks;

/// In gioco resta solo il promemoria della pausa: i comandi sono nella schermata di pausa.
const HINT: &str = "Esc: pausa e comandi";
/// Comandi nella schermata di pausa.
pub const CONTROLS: [&str; 3] = [
    "WASD o frecce: muoviti (gli attacchi partono da soli)",
    "Esc o P: riprendi    F5: ricomincia",
    "F3: dati di debug    F12: screenshot",
];
const PANEL: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
const MUTED: Color = Color::srgb_u8(0x5a, 0x6d, 0x99);
const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);

#[derive(Component)]
struct DataStatusPanel;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_data_status_panel)
            .add_systems(OnEnter(AppState::InRun), spawn_hint)
            .add_systems(OnEnter(RunPhase::Paused), spawn_pause_banner)
            .add_systems(
                Update,
                show_data_status.run_if(resource_changed::<DataStatus>),
            );
    }
}

/// Testo per lo stato dei dati; `None` quando non c'è nulla da mostrare.
pub fn data_status_text(status: &DataStatus) -> Option<String> {
    match status {
        DataStatus::Ready => None,
        DataStatus::Loading => Some("Caricamento dati…".into()),
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
        fonts::hud(16.0),
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
        fonts::ui(16.0),
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

/// Pausa: titolo, comandi e potenziamenti presi finora.
fn spawn_pause_banner(
    mut commands: Commands,
    config: Option<Res<GameConfig>>,
    stacks: Res<UpgradeStacks>,
) {
    let build = config.map_or(Vec::new(), |c| build_summary(&c, &stacks));
    let build = if build.is_empty() {
        "Potenziamenti: nessuno".to_string()
    } else {
        format!("Potenziamenti: {}", build.join(", "))
    };
    let panel = commands
        .spawn((
            Node {
                width: px(620),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                padding: UiRect::all(px(18)),
                border: UiRect::all(px(4)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(PANEL),
            children![(Text::new("PAUSA"), fonts::ui(44.0), TextColor(GOLD))],
        ))
        .id();
    for line in CONTROLS {
        let text = commands
            .spawn((Text::new(line), fonts::hud(16.0), TextColor(Color::WHITE)))
            .id();
        commands.entity(panel).add_child(text);
    }
    let build = commands
        .spawn((
            Text::new(build),
            fonts::ui(16.0),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            TextLayout::justify(Justify::Center),
            Node {
                margin: UiRect::top(px(6)),
                ..default()
            },
        ))
        .id();
    commands.entity(panel).add_child(build);
    commands
        .spawn((
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
        ))
        .add_child(panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_messages() {
        assert_eq!(data_status_text(&DataStatus::Ready), None);
        assert!(data_status_text(&DataStatus::Loading).is_some());
        let error = data_status_text(&DataStatus::Error(vec!["enemies.ron: x".into()])).unwrap();
        assert!(error.contains("- enemies.ron: x"));
    }
}
