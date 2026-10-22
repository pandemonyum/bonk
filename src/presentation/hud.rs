//! HUD minimo: promemoria comandi, schermata di pausa, stato dei dati (caricamento ed errori).
//! L'HUD di gioco completo arriva con la P0.7.

use bevy::prelude::*;

use crate::app::RunPhase;
use crate::data::DataStatus;

const HINT: &str = "WASD/frecce muovi   Esc pausa   F5 ricomincia   F3 debug   F12 screenshot";

#[derive(Component)]
struct DataStatusPanel;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_hint, spawn_data_status_panel))
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
    fn status_messages() {
        assert_eq!(data_status_text(&DataStatus::Ready), None);
        assert!(data_status_text(&DataStatus::Loading).is_some());
        let error = data_status_text(&DataStatus::Error(vec!["enemies.ron: x".into()])).unwrap();
        assert!(error.contains("- enemies.ron: x"));
    }
}
