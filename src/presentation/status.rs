//! Stato del giocatore nell'HUD, nello stile dell'art pass v1: in alto a sinistra cuori (uno
//! ogni 20 HP) con gli HP in numeri, barra XP e la fila delle icone dei compagni in squadra; in
//! alto a destra livello e nemici sconfitti con il teschio. Nascono entrando in `InRun`.

use bevy::prelude::*;

use super::icons::{element_icon_path, ui_icon};
use crate::app::AppState;
use crate::data::{DataId, GameConfig};
use crate::gameplay::combat::{Health, RunStats};
use crate::gameplay::companions::Companion;
use crate::gameplay::player::Player;
use crate::gameplay::progression::Progression;

/// HP rappresentati da un cuore.
pub const HP_PER_HEART: f32 = 20.0;
/// Colori dalla palette v1 (`docs/art/STYLE_HANDOFF.md`).
const XP_FILL: Color = Color::srgb_u8(0x4f, 0xa4, 0xe0);
const BAR_BACK: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
const OUTLINE: Color = Color::srgb_u8(0x1b, 0x14, 0x28);
const TEXT: Color = Color::srgb_u8(0xf4, 0xf1, 0xe8);
const TEXT_DIM: Color = Color::srgb_u8(0xa3, 0xb3, 0xd3);
const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heart {
    Full,
    Half,
    Empty,
}

/// Cuori per `current` su `max` HP: ogni cuore vale 20 HP (l'ultimo il resto, se `max` non è un
/// multiplo di 20); pieno se è tutto, mezzo se ne resta qualcosa (si vede sempre di essere
/// vivi), vuoto a zero. Almeno un cuore.
pub fn hearts(current: f32, max: f32) -> Vec<Heart> {
    let count = (max / HP_PER_HEART).ceil().max(1.0) as usize;
    (0..count)
        .map(|i| {
            let start = HP_PER_HEART * i as f32;
            let capacity = (max - start).clamp(0.0, HP_PER_HEART);
            let left = current - start;
            if left >= capacity - 1e-3 && left > 0.0 {
                Heart::Full
            } else if left > 0.0 {
                Heart::Half
            } else {
                Heart::Empty
            }
        })
        .collect()
}

fn heart_path(heart: Heart) -> &'static str {
    match heart {
        Heart::Full => "ui/heart_full.png",
        Heart::Half => "ui/heart_half.png",
        Heart::Empty => "ui/heart_empty.png",
    }
}

#[derive(Component)]
struct HeartsRow;

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct XpFill;

#[derive(Component)]
struct PartyRow;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct KillsText;

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), (spawn_left, spawn_right))
            .add_systems(
                Update,
                (
                    show_hearts,
                    show_xp.run_if(resource_changed::<Progression>),
                    show_party,
                    show_right,
                ),
            );
    }
}

fn label(text: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text.to_string()),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn spawn_left(mut commands: Commands) {
    commands.spawn((
        Name::new("StatusLeft"),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(10),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        DespawnOnExit(AppState::InRun),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                children![
                    (
                        HeartsRow,
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(2),
                            ..default()
                        },
                    ),
                    (HpText, label("", 13.0, TEXT_DIM)),
                ],
            ),
            (
                Node {
                    width: px(240),
                    height: px(9),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(OUTLINE),
                BackgroundColor(BAR_BACK),
                children![(
                    XpFill,
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(XP_FILL),
                )],
            ),
            (
                PartyRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4),
                    ..default()
                },
            ),
        ],
    ));
}

fn spawn_right(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Name::new("StatusRight"),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            right: px(12),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: px(4),
            ..default()
        },
        DespawnOnExit(AppState::InRun),
        children![
            (LevelText, label("", 18.0, GOLD)),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                },
                children![
                    ui_icon(&assets, "ui/skull.png", 10.0, 10.0),
                    (KillsText, label("", 16.0, TEXT)),
                ],
            ),
        ],
    ));
}

/// Cuori ricostruiti solo quando cambiano (HP o massimo); gli HP in numeri accanto.
fn show_hearts(
    mut commands: Commands,
    assets: Res<AssetServer>,
    player: Query<&Health, With<Player>>,
    row: Single<Entity, With<HeartsRow>>,
    mut text: Single<&mut Text, With<HpText>>,
    mut shown: Local<Vec<Heart>>,
) {
    let Ok(health) = player.single() else {
        return;
    };
    let line = format!("{:.0}/{:.0}", health.current.max(0.0), health.max);
    if text.0 != line {
        text.0 = line;
    }
    let wanted = hearts(health.current, health.max);
    if *shown == wanted {
        return;
    }
    commands.entity(*row).despawn_related::<Children>();
    for heart in &wanted {
        let icon = commands
            .spawn(ui_icon(&assets, heart_path(*heart), 10.0, 9.0))
            .id();
        commands.entity(*row).add_child(icon);
    }
    *shown = wanted;
}

fn show_xp(progression: Res<Progression>, mut fill: Single<&mut Node, With<XpFill>>) {
    fill.width = percent(100.0 * progression.fraction().clamp(0.0, 1.0));
}

/// Un'icona dell'elemento per ogni compagno, in ordine di posto.
fn show_party(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Option<Res<GameConfig>>,
    companions: Query<(&DataId, &Companion)>,
    row: Single<Entity, With<PartyRow>>,
    mut shown: Local<Vec<String>>,
) {
    let Some(config) = config else { return };
    let mut members: Vec<(usize, String)> = companions
        .iter()
        .map(|(id, c)| (c.slot, id.0.clone()))
        .collect();
    members.sort();
    let ids: Vec<String> = members.into_iter().map(|(_, id)| id).collect();
    if *shown == ids {
        return;
    }
    commands.entity(*row).despawn_related::<Children>();
    for id in &ids {
        if let Some(data) = config.companion(id) {
            let path = element_icon_path(data.element);
            let icon = commands.spawn(ui_icon(&assets, path, 16.0, 16.0)).id();
            commands.entity(*row).add_child(icon);
        }
    }
    *shown = ids;
}

fn show_right(
    progression: Res<Progression>,
    stats: Res<RunStats>,
    mut level: Single<&mut Text, (With<LevelText>, Without<KillsText>)>,
    mut kills: Single<&mut Text, (With<KillsText>, Without<LevelText>)>,
) {
    let line = format!("LV {}", progression.level);
    if level.0 != line {
        level.0 = line;
    }
    let line = stats.defeated.to_string();
    if kills.0 != line {
        kills.0 = line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_heart_every_twenty_hp_with_a_half_for_what_is_left() {
        use Heart::*;
        assert_eq!(hearts(100.0, 100.0), [Full, Full, Full, Full, Full]);
        assert_eq!(hearts(85.0, 100.0), [Full, Full, Full, Full, Half]);
        assert_eq!(hearts(41.0, 100.0), [Full, Full, Half, Empty, Empty]);
        assert_eq!(hearts(0.0, 100.0), [Empty; 5]);
        assert_eq!(hearts(-5.0, 100.0), [Empty; 5]);
        // HP massimi non multipli di 20: l'ultimo cuore vale il resto, pieno a vita piena
        assert_eq!(hearts(110.0, 110.0), [Full; 6]);
        assert_eq!(hearts(105.0, 110.0), [Full, Full, Full, Full, Full, Half]);
        assert_eq!(hearts(5.0, 5.0), [Full]);
    }
}
