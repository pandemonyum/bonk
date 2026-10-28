//! Stato del giocatore nell'HUD, nello stile dell'art pass v1: in alto a sinistra cuori (uno
//! ogni 20 HP) con gli HP in numeri, barra XP e sotto la fila delle abilità (`abilities`); in
//! alto a destra livello e nemici sconfitti con il teschio. Nascono entrando in `InRun`.

use bevy::prelude::*;

use super::abilities::AbilityRow;
use super::fonts;
use super::icons::ui_icon;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::data::schema::ProgressionFile;
use crate::gameplay::combat::{Health, RunStats};
use crate::gameplay::companions::Companion;
use crate::gameplay::player::Player;
use crate::gameplay::progression::Progression;
use crate::gameplay::treasure::Treasury;

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
struct LevelText;

#[derive(Component)]
struct KillsText;

#[derive(Component)]
struct NextCompanionText;

#[derive(Component)]
struct CoinsText;

/// Riga sotto il livello: a che livello si sceglie il prossimo compagno, finché la squadra
/// (`members` su `slots` posti) ha un posto libero; vuota quando non ne arrivano altri.
pub fn next_companion_line(
    progression: &ProgressionFile,
    level: u32,
    members: usize,
    slots: usize,
) -> String {
    match progression.next_companion_level(level) {
        Some(next) if members < slots => format!("compagno al LV {next}"),
        _ => String::new(),
    }
}

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), (spawn_left, spawn_right))
            .add_systems(
                Update,
                (
                    show_hearts,
                    show_xp.run_if(resource_changed::<Progression>),
                    show_right.run_if(resource_exists::<GameConfig>),
                    show_coins,
                ),
            );
    }
}

fn label(text: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text.to_string()),
        fonts::hud(size),
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
                    (HpText, label("", 16.0, TEXT_DIM)),
                ],
            ),
            (
                Node {
                    width: px(240),
                    height: px(10),
                    border: UiRect::all(px(2)),
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
                AbilityRow,
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
            (LevelText, label("", 32.0, GOLD)),
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
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                },
                children![
                    ui_icon(&assets, "ui/coin.png", 16.0, 16.0),
                    (CoinsText, label("", 16.0, TEXT)),
                ],
            ),
            (NextCompanionText, label("", 16.0, TEXT)),
        ],
    ));
}

/// Monete in tasca per difetto (con il Salvadanaio possono avere decimali).
fn show_coins(treasury: Res<Treasury>, mut coins: Single<&mut Text, With<CoinsText>>) {
    let line = ((treasury.coins + 1e-3).floor() as u32).to_string();
    if coins.0 != line {
        coins.0 = line;
    }
}

/// Cuori ricostruiti solo quando cambiano (HP o massimo) o quando la fila è nuova (run
/// ricominciata con gli stessi HP); gli HP in numeri accanto.
fn show_hearts(
    mut commands: Commands,
    assets: Res<AssetServer>,
    player: Query<&Health, With<Player>>,
    row: Single<Entity, With<HeartsRow>>,
    mut text: Single<&mut Text, With<HpText>>,
    mut shown: Local<Option<(Entity, Vec<Heart>)>>,
) {
    let Ok(health) = player.single() else {
        return;
    };
    let line = format!("{:.0}/{:.0}", health.current.max(0.0), health.max);
    if text.0 != line {
        text.0 = line;
    }
    let wanted = (*row, hearts(health.current, health.max));
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    commands.entity(*row).despawn_related::<Children>();
    for heart in &wanted.1 {
        let icon = commands
            .spawn(ui_icon(&assets, heart_path(*heart), 10.0, 9.0))
            .id();
        commands.entity(*row).add_child(icon);
    }
    *shown = Some(wanted);
}

fn show_xp(progression: Res<Progression>, mut fill: Single<&mut Node, With<XpFill>>) {
    fill.width = percent(100.0 * progression.fraction().clamp(0.0, 1.0));
}

type OnlyLevel = (
    With<LevelText>,
    Without<KillsText>,
    Without<NextCompanionText>,
    Without<CoinsText>,
);
type OnlyKills = (
    With<KillsText>,
    Without<LevelText>,
    Without<NextCompanionText>,
    Without<CoinsText>,
);
type OnlyNext = (
    With<NextCompanionText>,
    Without<LevelText>,
    Without<KillsText>,
    Without<CoinsText>,
);

fn show_right(
    config: Res<GameConfig>,
    progression: Res<Progression>,
    stats: Res<RunStats>,
    companions: Query<(), With<Companion>>,
    mut level: Single<&mut Text, OnlyLevel>,
    mut kills: Single<&mut Text, OnlyKills>,
    mut next: Single<&mut Text, OnlyNext>,
) {
    let line = format!("LV {}", progression.level);
    if level.0 != line {
        level.0 = line;
    }
    let line = stats.defeated.to_string();
    if kills.0 != line {
        kills.0 = line;
    }
    let line = next_companion_line(
        &config.progression,
        progression.level,
        companions.iter().count(),
        config.player.companion_slots.len(),
    );
    if next.0 != line {
        next.0 = line;
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

    #[test]
    fn the_next_companion_level_shows_while_a_slot_is_free() {
        let mut p = GameConfig::embedded().progression;
        p.companion_levels = vec![4, 10];
        assert_eq!(next_companion_line(&p, 1, 1, 3), "compagno al LV 4");
        assert_eq!(next_companion_line(&p, 4, 2, 3), "compagno al LV 10");
        assert_eq!(next_companion_line(&p, 10, 3, 3), "");
        assert_eq!(next_companion_line(&p, 12, 2, 3), "", "nessun livello dopo");
        assert_eq!(next_companion_line(&p, 2, 3, 3), "", "squadra piena");
    }
}
