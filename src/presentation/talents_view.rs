//! Disegno della schermata dei talenti (`talents_menu`): tre fasce come una scheda del
//! personaggio. A sinistra la lista dei compagni; al centro intestazione (ritratto, nome,
//! livello, barra dell'esperienza, punti) e una colonna colorata per ramo con i nodi alla loro
//! riga e colonna, il grado sotto e i collegamenti dai requisiti; a destra il nodo scelto.
//! Misure in unità dell'interfaccia (2 per texel), pari; Tiny5 solo a 16 e 32 px.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::fonts;
use super::icons::ui_icon;
use super::menu::{MenuChoice, portrait_path};
use super::talent_text::{kit_lines, node_kit};
use super::talents_menu::{NodeState, PanelRoot, TalentPanel, node_state};
use crate::app::AppState;
use crate::data::GameConfig;
use crate::data::schema::{CompanionData, TalentNode};
use crate::profile::Profile;

pub(super) const BACK: Color = Color::srgb_u8(0x0b, 0x10, 0x1c);
pub(super) const PANEL: Color = Color::srgb_u8(0x12, 0x1a, 0x2c);
pub(super) const CARD: Color = Color::srgb_u8(0x1b, 0x26, 0x40);
pub(super) const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);
pub(super) const GREEN: Color = Color::srgb_u8(0x7c, 0xe0, 0x6a);
pub(super) const MUTED: Color = Color::srgb_u8(0x4a, 0x58, 0x7a);
pub(super) const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.5);

/// Chi e cosa mostrare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewTarget {
    pub companion: String,
    pub selected: Option<String>,
}

pub(super) fn text(value: impl Into<String>, font: impl Bundle, color: Color) -> impl Bundle {
    (Text::new(value.into()), font, TextColor(color))
}

pub fn spawn_view(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    target: &ViewTarget,
) {
    let Some(companion) = config.companion(&target.companion) else {
        return;
    };
    let root = commands
        .spawn((
            Name::new("Talenti"),
            PanelRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GlobalZIndex(8),
            DespawnOnExit(AppState::MainMenu),
        ))
        .id();
    let frame = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(4)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(BACK),
        ))
        .id();
    commands.entity(root).add_child(frame);
    let body = commands
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .id();
    let list = spawn_list(commands, assets, config, profile, &companion.id);
    let tree =
        super::talents_tree::spawn_tree(commands, assets, config, profile, companion, target);
    let detail = spawn_detail(commands, assets, config, profile, companion, target);
    commands.entity(body).add_children(&[list, tree, detail]);
    let hint = commands
        .spawn(text(
            "Frecce o clic: scegli · Invio o clic: un grado · Tab: compagno · R: azzera · T o Esc: chiudi",
            fonts::hud(16.0),
            DIM,
        ))
        .id();
    commands.entity(frame).add_children(&[body, hint]);
}

/// La lista dei compagni con ritratto e livello; quello scelto in oro, clic per cambiarlo.
fn spawn_list(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    chosen: &str,
) -> Entity {
    let list = commands
        .spawn((
            Node {
                width: px(184),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(6)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(PANEL),
            children![text("COMPAGNI", fonts::ui(22.0), Color::WHITE)],
        ))
        .id();
    for c in &config.companions {
        let (level, _, _) = profile.points(config, &c.id);
        let here = c.id == chosen;
        let id = c.id.clone();
        let entry = commands
            .spawn((
                Button,
                Hovered::default(),
                Node {
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(2)),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BorderColor::all(if here { GOLD } else { MUTED }),
                BackgroundColor(CARD),
                children![
                    ui_icon(assets, &portrait_path(c), 40.0, 36.0),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        children![
                            text(c.display_name.clone(), fonts::ui(16.0), Color::WHITE),
                            text(format!("Lv. {level}"), fonts::hud(16.0), DIM),
                        ],
                    ),
                ],
            ))
            .observe(
                move |_: On<Activate>,
                      mut choice: ResMut<MenuChoice>,
                      mut panel: ResMut<TalentPanel>| {
                    if choice.0 != id {
                        choice.0 = id.clone();
                        panel.selected = None;
                    }
                },
            )
            .id();
        commands.entity(list).add_child(entry);
    }
    list
}

/// Il nodo scelto: icona, nome, grado, testo, effetti al grado attuale, prossimo grado e cosa
/// serve per prenderlo.
fn spawn_detail(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    companion: &CompanionData,
    target: &ViewTarget,
) -> Entity {
    let detail = commands
        .spawn((
            Node {
                width: px(236),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(PANEL),
        ))
        .id();
    let node = target.selected.as_ref().and_then(|s| {
        companion
            .talents
            .iter()
            .flat_map(|b| b.nodes.iter())
            .find(|n| n.id == *s)
    });
    let Some(node) = node else {
        return detail;
    };
    let rank = profile.rank(config, &companion.id, &node.id);
    let max = node.max_rank();
    let state = node_state(profile, config, &companion.id, node);
    let title = commands
        .spawn((
            Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                ui_icon(assets, &format!("ui/talent_{}.png", node.id), 16.0, 16.0),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    children![
                        text(node.name.clone(), fonts::ui(22.0), GOLD),
                        text(
                            format!("Grado {rank}/{max}"),
                            fonts::hud(16.0),
                            Color::WHITE
                        ),
                    ],
                ),
            ],
        ))
        .id();
    let description = commands
        .spawn(text(
            node.description.clone(),
            fonts::ui(16.0),
            Color::WHITE,
        ))
        .id();
    commands.entity(detail).add_children(&[title, description]);
    let now = kit_lines(&node_kit(node, rank));
    for line in &now {
        let row = commands
            .spawn(text(line.clone(), fonts::ui(16.0), GREEN))
            .id();
        commands.entity(detail).add_child(row);
    }
    let next_lines = if rank < max {
        kit_lines(&node_kit(node, rank + 1))
    } else {
        Vec::new()
    };
    let next = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                padding: UiRect::all(px(6)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(BACK),
            children![text(
                if rank < max {
                    "PROSSIMO GRADO"
                } else {
                    "GRADO MASSIMO"
                },
                fonts::hud(16.0),
                DIM
            )],
        ))
        .id();
    if next_lines.is_empty() {
        let done = commands
            .spawn(text(
                "Grado massimo raggiunto.",
                fonts::ui(16.0),
                Color::WHITE,
            ))
            .id();
        commands.entity(next).add_child(done);
    }
    for line in next_lines {
        let row = commands
            .spawn(text(line, fonts::ui(16.0), Color::WHITE))
            .id();
        commands.entity(next).add_child(row);
    }
    let (status, color) = status_line(config, profile, companion, node, state);
    let status = commands.spawn(text(status, fonts::ui(16.0), color)).id();
    commands.entity(detail).add_children(&[next, status]);
    detail
}

/// Cosa succede premendo Invio sul nodo scelto.
pub fn status_line(
    config: &GameConfig,
    profile: &Profile,
    companion: &CompanionData,
    node: &TalentNode,
    state: NodeState,
) -> (String, Color) {
    let (_, earned, spent) = profile.points(config, &companion.id);
    match state {
        NodeState::Maxed => ("Al massimo.".to_string(), GOLD),
        NodeState::Available => ("Invio o clic: un grado (1 punto)".to_string(), GREEN),
        _ if spent >= earned => (
            "Nessun punto: si guadagnano giocando con questo compagno.".to_string(),
            DIM,
        ),
        _ => {
            let missing: Vec<&str> = node
                .requires
                .iter()
                .filter(|r| profile.rank(config, &companion.id, r) == 0)
                .filter_map(|r| {
                    companion
                        .talents
                        .iter()
                        .flat_map(|b| b.nodes.iter())
                        .find(|n| n.id == **r)
                        .map(|n| n.name.as_str())
                })
                .collect();
            (format!("Serve prima: {}", missing.join(", ")), DIM)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_says_what_is_missing() {
        let config = GameConfig::embedded();
        let ember = config.companion("ember").unwrap();
        let first = &ember.talents[0].nodes[0];
        let second = &ember.talents[0].nodes[1];
        let mut profile = Profile::default();
        let (line, _) = status_line(&config, &profile, ember, first, NodeState::Locked);
        assert!(line.starts_with("Nessun punto"), "{line}");
        let curve = config.progression.companion_xp;
        profile
            .data
            .companions
            .push(crate::profile::CompanionProgress {
                id: "ember".into(),
                xp: curve.total_for(3),
                learned: vec![],
            });
        let state = node_state(&profile, &config, "ember", second);
        let (line, _) = status_line(&config, &profile, ember, second, state);
        assert_eq!(line, format!("Serve prima: {}", first.name));
    }
}
