//! Talenti nel menu: T apre l'albero del compagno selezionato. Livello, esperienza e punti in
//! alto; due rami da tre nodi (icona, nome, testo, stato). 1–3 imparano il prossimo nodo del
//! primo ramo, 4–6 quelli del secondo (o clic sul nodo), R azzera, ←/→ cambia compagno, T o
//! Esc chiudono. Ogni modifica si salva subito nel profilo.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::fonts;
use super::icons::UI_PX_PER_TEXEL;
use super::menu::MenuChoice;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::data::schema::TalentNode;
use crate::profile::Profile;

const PANEL: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
const CARD: Color = Color::srgb_u8(0x1b, 0x26, 0x40);
const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);
const MUTED: Color = Color::srgb_u8(0x5a, 0x6d, 0x99);
const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.45);
const LOCKED_ICON: Color = Color::srgb(0.35, 0.35, 0.42);

/// Il pannello dei talenti è aperto (sul compagno di `MenuChoice`).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TalentPanel(pub bool);

pub fn panel_closed(panel: Res<TalentPanel>) -> bool {
    !panel.0
}

#[derive(Component)]
struct PanelRoot;

/// Stato di un nodo per un compagno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Learned,
    Available,
    Locked,
}

pub fn node_state(
    profile: &Profile,
    config: &GameConfig,
    id: &str,
    node: &TalentNode,
) -> NodeState {
    if profile.progress(id).learned.contains(&node.id) {
        NodeState::Learned
    } else if profile.can_learn(config, id, &node.id) {
        NodeState::Available
    } else {
        NodeState::Locked
    }
}

/// Riga dei punti: livello, esperienza verso il prossimo, punti da spendere.
pub fn points_line(profile: &Profile, config: &GameConfig, id: &str) -> String {
    let curve = &config.progression.companion_xp;
    let xp = profile.progress(id).xp;
    let (level, earned, spent) = profile.points(config, id);
    let progress = if level >= curve.max_level {
        "livello massimo".to_string()
    } else {
        let into = xp.saturating_sub(curve.total_for(level));
        format!("{into}/{} esperienza", curve.to_next(level))
    };
    let free = earned.saturating_sub(spent);
    let points = match free {
        0 => "nessun punto da spendere".to_string(),
        1 => "1 punto da spendere".to_string(),
        n => format!("{n} punti da spendere"),
    };
    format!("Livello {level} · {progress} · {points}")
}

pub struct TalentsMenuPlugin;

impl Plugin for TalentsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TalentPanel>()
            .add_systems(OnExit(AppState::MainMenu), close_panel)
            .add_systems(
                Update,
                (talent_keys, show_panel)
                    .chain()
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

fn close_panel(mut panel: ResMut<TalentPanel>) {
    panel.0 = false;
}

fn learn(profile: &mut Profile, config: &GameConfig, id: &str, node: &str) {
    if profile.learn(config, id, node) {
        info!("talento imparato: {node}");
        profile.save();
    }
}

fn talent_keys(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut panel: ResMut<TalentPanel>,
    mut choice: ResMut<MenuChoice>,
    mut profile: ResMut<Profile>,
) {
    if !panel.0 {
        if keys.just_pressed(KeyCode::KeyT) {
            panel.0 = true;
        }
        return;
    }
    if keys.any_just_pressed([KeyCode::KeyT, KeyCode::Escape]) {
        panel.0 = false;
        return;
    }
    let ids: Vec<&str> = config.companions.iter().map(|c| c.id.as_str()).collect();
    let count = ids.len().max(1);
    let mut index = ids.iter().position(|id| *id == choice.0).unwrap_or(0);
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        index = (index + count - 1) % count;
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        index = (index + 1) % count;
    }
    if let Some(id) = ids.get(index)
        && choice.0 != *id
    {
        choice.0 = id.to_string();
    }
    let Some(companion) = config.companion(&choice.0) else {
        return;
    };
    let id = companion.id.clone();
    if keys.just_pressed(KeyCode::KeyR) {
        profile.reset(&id);
        profile.save();
        return;
    }
    const KEYS: [[KeyCode; 2]; 6] = [
        [KeyCode::Digit1, KeyCode::Numpad1],
        [KeyCode::Digit2, KeyCode::Numpad2],
        [KeyCode::Digit3, KeyCode::Numpad3],
        [KeyCode::Digit4, KeyCode::Numpad4],
        [KeyCode::Digit5, KeyCode::Numpad5],
        [KeyCode::Digit6, KeyCode::Numpad6],
    ];
    if let Some(k) = KEYS.iter().position(|pair| keys.any_just_pressed(*pair)) {
        let node = companion
            .talents
            .get(k / 3)
            .and_then(|b| b.nodes.get(k % 3))
            .map(|n| n.id.clone());
        if let Some(node) = node {
            learn(&mut profile, &config, &id, &node);
        }
    }
}

/// Il pannello si ricostruisce quando cambiano apertura, compagno o profilo.
fn show_panel(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    panel: Res<TalentPanel>,
    choice: Res<MenuChoice>,
    profile: Res<Profile>,
    roots: Query<Entity, With<PanelRoot>>,
) {
    let changed = panel.is_changed() || choice.is_changed() || profile.is_changed();
    if !changed && (roots.is_empty() != panel.0) {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    if !panel.0 {
        return;
    }
    let Some(companion) = config.companion(&choice.0) else {
        return;
    };
    let id = companion.id.clone();
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(8),
            DespawnOnExit(AppState::MainMenu),
        ))
        .id();
    let card = commands
        .spawn((
            Node {
                width: px(820),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                padding: UiRect::all(px(16)),
                border: UiRect::all(px(4)),
                ..default()
            },
            BorderColor::all(GOLD),
            BackgroundColor(PANEL),
            children![
                (
                    Text::new(format!("Talenti di {}", companion.display_name)),
                    fonts::ui(33.0),
                    TextColor(GOLD),
                ),
                (
                    Text::new(points_line(&profile, &config, &id)),
                    fonts::ui(16.0),
                    TextColor(Color::WHITE),
                ),
            ],
        ))
        .id();
    commands.entity(root).add_child(card);
    let columns = commands
        .spawn(Node {
            column_gap: px(16),
            ..default()
        })
        .id();
    commands.entity(card).add_child(columns);
    for (b, branch) in companion.talents.iter().enumerate() {
        let column = commands
            .spawn((
                Node {
                    width: px(386),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                children![(
                    Text::new(branch.name.clone()),
                    fonts::ui(22.0),
                    TextColor(GOLD)
                )],
            ))
            .id();
        commands.entity(columns).add_child(column);
        for (k, node) in branch.nodes.iter().enumerate() {
            let state = node_state(&profile, &config, &id, node);
            let row = spawn_node(&mut commands, &assets, b * 3 + k + 1, node, state, &id);
            commands.entity(column).add_child(row);
        }
    }
    let hint = commands
        .spawn((
            Text::new("1–3 primo ramo, 4–6 secondo ramo o clic · R azzera · A/D compagno · T o Esc chiude"),
            fonts::hud(16.0),
            TextColor(DIM),
        ))
        .id();
    commands.entity(card).add_child(hint);
}

fn spawn_node(
    commands: &mut Commands,
    assets: &AssetServer,
    key: usize,
    node: &TalentNode,
    state: NodeState,
    companion: &str,
) -> Entity {
    let (label, color, border) = match state {
        NodeState::Learned => ("Imparato".to_string(), GOLD, GOLD),
        NodeState::Available => (format!("[{key}] Impara"), Color::WHITE, Color::WHITE),
        NodeState::Locked => ("Bloccato".to_string(), DIM, MUTED),
    };
    // icona a 16 texel come `ui_icon`, scurita se il nodo è bloccato
    let mut image = ImageNode::new(assets.load(format!("ui/talent_{}.png", node.id)));
    if state == NodeState::Locked {
        image.color = LOCKED_ICON;
    }
    let side = px(16.0 * UI_PX_PER_TEXEL);
    let icon = (
        image,
        Node {
            width: side,
            height: side,
            flex_shrink: 0.0,
            ..default()
        },
    );
    let companion = companion.to_string();
    let id = node.id.clone();
    commands
        .spawn((
            Button,
            Hovered::default(),
            Node {
                column_gap: px(10),
                align_items: AlignItems::Center,
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(border),
            BackgroundColor(CARD),
            children![
                icon,
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(2),
                        flex_grow: 1.0,
                        ..default()
                    },
                    children![
                        (
                            Text::new(node.name.clone()),
                            fonts::ui(16.0),
                            TextColor(color)
                        ),
                        (
                            Text::new(node.description.clone()),
                            fonts::ui(16.0),
                            TextColor(DIM)
                        ),
                    ],
                ),
                (Text::new(label), fonts::hud(16.0), TextColor(color)),
            ],
        ))
        .observe(
            move |_: On<Activate>, config: Res<GameConfig>, mut profile: ResMut<Profile>| {
                learn(&mut profile, &config, &companion, &id);
            },
        )
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_points_line_shows_level_progress_and_points() {
        let config = GameConfig::embedded();
        let mut profile = Profile::default();
        assert_eq!(
            points_line(&profile, &config, "ember"),
            "Livello 1 · 0/100 esperienza · nessun punto da spendere"
        );
        let curve = config.progression.companion_xp;
        profile
            .data
            .companions
            .push(crate::profile::CompanionProgress {
                id: "ember".into(),
                xp: curve.total_for(2) + 30,
                learned: vec![],
            });
        assert_eq!(
            points_line(&profile, &config, "ember"),
            format!(
                "Livello 2 · 30/{} esperienza · 1 punto da spendere",
                curve.to_next(2)
            )
        );
        let ember = config.companion("ember").unwrap();
        let first = &ember.talents[0].nodes[0];
        let second = &ember.talents[0].nodes[1];
        assert_eq!(
            node_state(&profile, &config, "ember", first),
            NodeState::Available
        );
        assert_eq!(
            node_state(&profile, &config, "ember", second),
            NodeState::Locked
        );
        profile.data.companions[0].xp = curve.total_for(curve.max_level) + 999;
        assert!(points_line(&profile, &config, "ember").contains("livello massimo"));
    }
}
