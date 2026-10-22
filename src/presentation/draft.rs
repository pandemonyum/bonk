//! Schermata del draft (`RunPhase::LevelUp`): le carte estratte, scelta con 1/2/3 o clic.
//! Mostra soltanto: la scelta passa da `ChooseUpgrade` e la applica `gameplay::upgrades`.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::app::RunPhase;
use crate::data::GameConfig;
use crate::data::schema::{Effect, Rarity, UpgradeData};
use crate::gameplay::progression::Progression;
use crate::gameplay::upgrades::{ChooseUpgrade, Draft, UpgradeStacks};

const CARD_WIDTH: f32 = 190.0;
const CARD_HEIGHT: f32 = 200.0;
const CARD_FILL: Color = Color::srgba(0.07, 0.06, 0.12, 0.95);
const CARD_FILL_HOVER: Color = Color::srgba(0.16, 0.14, 0.26, 0.97);

/// Radice della schermata, ricostruita a ogni draft nuovo.
#[derive(Component)]
struct DraftScreen;

/// Carta del draft: posizione in `Draft::choices`.
#[derive(Component, Debug, Clone, Copy)]
struct DraftCard(usize);

pub struct DraftUiPlugin;

impl Plugin for DraftUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (show_draft, highlight_hovered_card).run_if(in_state(RunPhase::LevelUp)),
        );
    }
}

/// Nome della rarità sulla carta e colore del bordo.
pub fn rarity_style(rarity: Rarity) -> (&'static str, Color) {
    match rarity {
        Rarity::Common => ("Comune", Color::srgb(0.82, 0.82, 0.86)),
        Rarity::Uncommon => ("Non comune", Color::srgb(0.42, 0.85, 0.45)),
        Rarity::Rare => ("Rara", Color::srgb(0.95, 0.75, 0.3)),
    }
}

/// Riga in fondo alla carta: tasto e stack che si otterrebbe ("[1]   Lv 2/5"). La cura è
/// immediata e non lascia stack: niente livello.
pub fn card_footer(key: usize, upgrade: &UpgradeData, stacks_now: u32) -> String {
    if matches!(upgrade.effect, Effect::Heal(_)) {
        format!("[{key}]")
    } else {
        format!("[{key}]   Lv {}/{}", stacks_now + 1, upgrade.max_stack)
    }
}

/// Ricostruisce la schermata quando arriva un draft nuovo (anche più di uno di fila).
fn show_draft(
    mut commands: Commands,
    draft: Res<Draft>,
    config: Res<GameConfig>,
    stacks: Res<UpgradeStacks>,
    progression: Res<Progression>,
    screens: Query<Entity, With<DraftScreen>>,
) {
    if !draft.is_changed() && !screens.is_empty() {
        return;
    }
    for screen in &screens {
        commands.entity(screen).despawn();
    }
    if draft.choices.is_empty() {
        return; // scelta fatta: la fase sta per tornare a Playing
    }
    let levels_left = progression.pending_levels;
    let subtitle = if levels_left > 1 {
        format!(
            "Scegli un potenziamento: 1-{} o clic   ({levels_left} da scegliere)",
            draft.choices.len()
        )
    } else {
        format!("Scegli un potenziamento: 1-{} o clic", draft.choices.len())
    };
    let level = progression.level + 1 - levels_left;
    let root = commands
        .spawn((
            Name::new("DraftScreen"),
            DraftScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.08, 0.6)),
            DespawnOnExit(RunPhase::LevelUp),
            children![
                (
                    Text::new(format!("LIVELLO {level}")),
                    TextFont::from_font_size(30.0),
                    TextColor(Color::srgb(0.55, 0.8, 1.0)),
                ),
                (
                    Text::new(subtitle),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                ),
            ],
        ))
        .id();
    let row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(16),
            margin: UiRect::top(px(8)),
            ..default()
        })
        .id();
    commands.entity(root).add_child(row);
    for (slot, &index) in draft.choices.iter().enumerate() {
        let Some(upgrade) = config.upgrades.get(index) else {
            continue; // dati appena ricaricati: la run sta per ripartire
        };
        let card = spawn_card(&mut commands, slot, upgrade, stacks.of(index));
        commands.entity(row).add_child(card);
    }
}

fn spawn_card(commands: &mut Commands, slot: usize, upgrade: &UpgradeData, stacks: u32) -> Entity {
    let (rarity, border) = rarity_style(upgrade.rarity);
    commands
        .spawn((
            Name::new(format!("Carta {}", upgrade.id)),
            DraftCard(slot),
            Button,
            Hovered::default(),
            Node {
                width: px(CARD_WIDTH),
                height: px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(12)),
                row_gap: px(8),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(border),
            BackgroundColor(CARD_FILL),
            children![
                (
                    Text::new(rarity),
                    TextFont::from_font_size(11.0),
                    TextColor(border),
                ),
                (
                    Text::new(upgrade.name.clone()),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Text::new(upgrade.description.clone()),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ),
                (
                    Text::new(card_footer(slot + 1, upgrade, stacks)),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                ),
            ],
        ))
        .observe(
            |activate: On<Activate>,
             cards: Query<&DraftCard>,
             mut choose: MessageWriter<ChooseUpgrade>| {
                if let Ok(card) = cards.get(activate.entity) {
                    choose.write(ChooseUpgrade(card.0));
                }
            },
        )
        .id()
}

fn highlight_hovered_card(
    mut cards: Query<(&Hovered, &mut BackgroundColor), (With<DraftCard>, Changed<Hovered>)>,
) {
    for (hovered, mut fill) in &mut cards {
        fill.0 = if hovered.get() {
            CARD_FILL_HOVER
        } else {
            CARD_FILL
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upgrade(max_stack: u32, effect: Effect) -> UpgradeData {
        UpgradeData {
            id: "x".into(),
            name: "X".into(),
            description: "x".into(),
            rarity: Rarity::Common,
            max_stack,
            requires: Vec::new(),
            effect,
        }
    }

    #[test]
    fn the_footer_shows_the_key_and_the_next_stack() {
        let damage = Effect::CompanionDamagePct(15.0);
        assert_eq!(card_footer(1, &upgrade(5, damage), 0), "[1]   Lv 1/5");
        assert_eq!(card_footer(3, &upgrade(5, damage), 4), "[3]   Lv 5/5");
        assert_eq!(card_footer(2, &upgrade(99, Effect::Heal(30.0)), 7), "[2]");
    }
}
