//! L'albero dei talenti nella schermata (`talents_view`): intestazione del compagno (ritratto,
//! nome, livello, barra dell'esperienza, punti) e una colonna colorata per ramo, con i nodi
//! alla loro riga e colonna, il grado sotto e i collegamenti dai requisiti. Clic su un nodo:
//! la prima volta lo sceglie, la seconda gli dà un grado.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::fonts;
use super::icons::{UI_PX_PER_TEXEL, ui_icon};
use super::menu::{element_label, portrait_path};
use super::talents_menu::{NodeState, TalentPanel, learn, node_state};
use super::talents_view::{BACK, CARD, DIM, GOLD, GREEN, MUTED, PANEL, ViewTarget, text};
use crate::data::GameConfig;
use crate::data::schema::{BranchTheme, CompanionData, TalentBranch, TalentNode};
use crate::profile::Profile;

const LOCKED_ICON: Color = Color::srgb(0.32, 0.32, 0.4);
const AMBER: Color = Color::srgb_u8(0xe8, 0xa3, 0x3a);
const BLUE: Color = Color::srgb_u8(0x5a, 0x8e, 0xe0);

/// Colonna di un ramo e geometria della sua griglia.
const BRANCH_W: f32 = 150.0;
const ICON: f32 = 16.0 * UI_PX_PER_TEXEL;
const BOX: f32 = ICON + 8.0;
const ROW_PITCH: f32 = 62.0;
const ROWS: f32 = 5.0;
const LINE: f32 = 2.0;

/// Colore di un ramo: l'elemento del compagno, l'ambra del supporto, il blu dell'evoluzione.
pub fn theme_color(companion: &CompanionData, theme: BranchTheme) -> Color {
    match theme {
        BranchTheme::Element => element_label(companion.element).1,
        BranchTheme::Support => AMBER,
        BranchTheme::Evolution => BLUE,
    }
}

/// Icona dell'intestazione di un ramo (`assets/ui/branch_<compagno>_<el|su|ev>.png`).
pub fn branch_icon_path(companion: &CompanionData, theme: BranchTheme) -> String {
    let tag = match theme {
        BranchTheme::Element => "el",
        BranchTheme::Support => "su",
        BranchTheme::Evolution => "ev",
    };
    format!("ui/branch_{}_{tag}.png", companion.id)
}

/// Centro orizzontale di una colonna della griglia (0 e 1 i percorsi, 2 al centro).
fn col_center(col: u32) -> f32 {
    match col {
        0 => BRANCH_W * 0.27,
        1 => BRANCH_W * 0.73,
        _ => BRANCH_W * 0.5,
    }
}

/// Intestazione e colonne dei rami.
pub(super) fn spawn_tree(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    companion: &CompanionData,
    target: &ViewTarget,
) -> Entity {
    let curve = &config.progression.companion_xp;
    let (level, earned, spent) = profile.points(config, &companion.id);
    let xp = profile.progress(&companion.id).xp;
    let fill = if level >= curve.max_level {
        1.0
    } else {
        let into = xp.saturating_sub(curve.total_for(level)) as f32;
        (into / curve.to_next(level).max(1) as f32).clamp(0.0, 1.0)
    };
    let free = earned.saturating_sub(spent);
    let tree = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(6)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(MUTED),
            BackgroundColor(PANEL),
        ))
        .id();
    let header = commands
        .spawn((
            Node {
                column_gap: px(12),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            children![
                ui_icon(assets, &portrait_path(companion), 40.0, 36.0),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        flex_grow: 1.0,
                        ..default()
                    },
                    children![
                        text(
                            companion.display_name.clone(),
                            fonts::ui(33.0),
                            Color::WHITE
                        ),
                        (
                            Node {
                                column_gap: px(8),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            children![
                                text(format!("Lv. {level}"), fonts::hud(16.0), Color::WHITE),
                                (
                                    Node {
                                        width: px(160),
                                        height: px(10),
                                        border: UiRect::all(px(2)),
                                        ..default()
                                    },
                                    BorderColor::all(MUTED),
                                    BackgroundColor(BACK),
                                    children![(
                                        Node {
                                            width: percent(100.0 * fill),
                                            height: percent(100),
                                            ..default()
                                        },
                                        BackgroundColor(GOLD),
                                    )],
                                ),
                            ],
                        ),
                    ],
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(2),
                        ..default()
                    },
                    children![
                        text("PUNTI TALENTO", fonts::hud(16.0), DIM),
                        (
                            Node {
                                width: px(56),
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(px(2)),
                                ..default()
                            },
                            BorderColor::all(if free > 0 { GOLD } else { MUTED }),
                            BackgroundColor(BACK),
                            children![text(
                                free.to_string(),
                                fonts::hud(32.0),
                                if free > 0 { GOLD } else { DIM },
                            )],
                        ),
                    ],
                ),
            ],
        ))
        .id();
    let columns = commands
        .spawn(Node {
            column_gap: px(6),
            ..default()
        })
        .id();
    for branch in &companion.talents {
        let column = spawn_branch(commands, assets, config, profile, companion, branch, target);
        commands.entity(columns).add_child(column);
    }
    commands.entity(tree).add_children(&[header, columns]);
    tree
}

/// Una colonna: intestazione del ramo e griglia con nodi e collegamenti.
fn spawn_branch(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    companion: &CompanionData,
    branch: &TalentBranch,
    target: &ViewTarget,
) -> Entity {
    let color = theme_color(companion, branch.theme);
    let column = commands
        .spawn((
            Node {
                width: px(BRANCH_W),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(4),
                padding: UiRect::vertical(px(6)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(color.with_alpha(0.8)),
            BackgroundColor(color.with_alpha(0.12)),
            children![
                ui_icon(
                    assets,
                    &branch_icon_path(companion, branch.theme),
                    16.0,
                    16.0
                ),
                text(branch.name.to_uppercase(), fonts::hud(16.0), color),
            ],
        ))
        .id();
    let grid = commands
        .spawn(Node {
            width: px(BRANCH_W - 4.0),
            height: px(ROWS * ROW_PITCH),
            ..default()
        })
        .id();
    commands.entity(column).add_child(grid);
    // collegamenti prima dei nodi, così passano sotto
    for node in &branch.nodes {
        for required in &node.requires {
            let Some(from) = branch.nodes.iter().find(|n| n.id == *required) else {
                continue;
            };
            let lit = profile.rank(config, &companion.id, &from.id) > 0;
            for segment in connector(from, node) {
                let line = commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(segment.min.x),
                            top: px(segment.min.y),
                            width: px(segment.width().max(LINE)),
                            height: px(segment.height().max(LINE)),
                            ..default()
                        },
                        BackgroundColor(if lit { GOLD } else { MUTED }),
                    ))
                    .id();
                commands.entity(grid).add_child(line);
            }
        }
    }
    for node in &branch.nodes {
        let state = node_state(profile, config, &companion.id, node);
        let rank = profile.rank(config, &companion.id, &node.id);
        let selected = target.selected.as_deref() == Some(node.id.as_str());
        let cell = spawn_node(commands, assets, &companion.id, node, state, rank, selected);
        commands.entity(grid).add_child(cell);
    }
    column
}

/// Segmenti (rettangoli nella griglia) del collegamento da `from` a `to`: giù, di lato, giù.
pub fn connector(from: &TalentNode, to: &TalentNode) -> Vec<Rect> {
    let (x0, x1) = (col_center(from.col), col_center(to.col));
    let bottom = from.row as f32 * ROW_PITCH + BOX + 18.0;
    let top = to.row as f32 * ROW_PITCH;
    let mid = (bottom + top) / 2.0;
    let half = LINE / 2.0;
    if (x0 - x1).abs() < 0.5 {
        return vec![Rect::new(x0 - half, bottom, x0 + half, top)];
    }
    vec![
        Rect::new(x0 - half, bottom, x0 + half, mid),
        Rect::new(x0.min(x1) - half, mid - half, x0.max(x1) + half, mid + half),
        Rect::new(x1 - half, mid, x1 + half, top),
    ]
}

fn spawn_node(
    commands: &mut Commands,
    assets: &AssetServer,
    companion: &str,
    node: &TalentNode,
    state: NodeState,
    rank: u32,
    selected: bool,
) -> Entity {
    let (border, rank_color) = match state {
        NodeState::Maxed => (GOLD, GOLD),
        NodeState::Learned => (GOLD.with_alpha(0.7), GREEN),
        NodeState::Available => (Color::WHITE, Color::WHITE),
        NodeState::Locked => (MUTED, DIM),
    };
    let mut image = ImageNode::new(assets.load(format!("ui/talent_{}.png", node.id)));
    if state == NodeState::Locked {
        image.color = LOCKED_ICON;
    }
    let left = col_center(node.col) - BOX / 2.0;
    let top = node.row as f32 * ROW_PITCH;
    let companion = companion.to_string();
    let id = node.id.clone();
    let cell = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px(left - 8.0),
            top: px(top),
            width: px(BOX + 16.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            Node {
                width: px(BOX),
                height: px(BOX),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(if selected { 4 } else { 2 })),
                ..default()
            },
            BorderColor::all(if selected { GOLD } else { border }),
            BackgroundColor(if selected { CARD.lighter(0.15) } else { CARD }),
            children![(
                image,
                Node {
                    width: px(ICON),
                    height: px(ICON),
                    ..default()
                },
            )],
        ))
        .observe(
            move |_: On<Activate>,
                  config: Res<GameConfig>,
                  mut panel: ResMut<TalentPanel>,
                  mut profile: ResMut<Profile>| {
                // il primo clic sceglie, il secondo sul nodo scelto impara
                if panel.selected.as_deref() == Some(id.as_str()) {
                    learn(&mut profile, &config, &companion, &id);
                } else {
                    panel.selected = Some(id.clone());
                }
            },
        )
        .id();
    let label = commands
        .spawn(text(
            format!("{rank}/{}", node.max_rank()),
            fonts::hud(16.0),
            rank_color,
        ))
        .id();
    commands.entity(cell).add_children(&[button, label]);
    cell
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(col: u32, row: u32) -> TalentNode {
        TalentNode {
            id: format!("n{col}{row}"),
            name: String::new(),
            description: String::new(),
            col,
            row,
            requires: vec![],
            ranks: vec![vec![]],
        }
    }

    #[test]
    fn straight_links_are_one_segment_and_merges_bend() {
        let straight = connector(&node(0, 0), &node(0, 1));
        assert_eq!(straight.len(), 1);
        assert!(straight[0].max.y <= ROW_PITCH && straight[0].min.y < straight[0].max.y);
        let merge = connector(&node(1, 2), &node(2, 3));
        assert_eq!(merge.len(), 3);
        // il tratto di lato va dalla colonna del percorso al centro
        let side = merge[1];
        assert!(side.min.x <= col_center(2) && side.max.x >= col_center(1));
    }

    #[test]
    fn branch_headers_have_their_icon() {
        let config = GameConfig::embedded();
        for c in &config.companions {
            for branch in &c.talents {
                let path = branch_icon_path(c, branch.theme);
                let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("assets")
                    .join(&path);
                assert!(file.exists(), "{path}");
            }
        }
    }
}
