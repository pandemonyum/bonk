//! Carte dei compagni nella scelta: sprite, elemento, tipo di attacco e due barre (danno al
//! secondo su un nemico, portata) relative agli altri compagni.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::{CARD, CompanionCard, GOLD, MUTED, MenuChoice, element_label, portrait_path};
use crate::data::schema::CompanionData;
use crate::presentation::fonts;
use crate::presentation::icons::{element_icon_path, ui_icon};

pub(super) fn spawn_card(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
    companion: &CompanionData,
    pattern: &str,
    (dps, reach): (f32, f32),
    chosen: bool,
) -> Entity {
    let (element, color) = element_label(companion.element);
    // figura intera del compagno (D-052), come il fermo di fronte del suo foglio
    let portrait = ui_icon(assets, &portrait_path(companion), 40.0, 36.0);
    let element_icon = ui_icon(assets, element_icon_path(companion.element), 16.0, 16.0);
    let card = commands
        .spawn((
            Name::new(format!("Carta {}", companion.id)),
            CompanionCard(companion.id.clone()),
            Button,
            Hovered::default(),
            Node {
                width: px(170),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(4)),
                ..default()
            },
            // già giusto allo spawn: rientrando nella schermata la scelta non cambia
            BorderColor::all(if chosen { GOLD } else { MUTED }),
            BackgroundColor(CARD),
            children![
                (
                    Node {
                        width: percent(100),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    children![
                        (
                            Text::new(format!("[{}]", index + 1)),
                            fonts::hud(16.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        ),
                        element_icon,
                    ],
                ),
                // ritratto dell'art pass alla scala del canvas: i pixel restano netti
                portrait,
                (
                    Text::new(companion.display_name.clone()),
                    fonts::ui(22.0),
                    TextColor(Color::WHITE),
                ),
                (Text::new(element), fonts::hud(16.0), TextColor(color),),
                (
                    Text::new(pattern.to_string()),
                    fonts::ui(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    TextLayout::justify(Justify::Center),
                ),
            ],
        ))
        .observe(
            |activate: On<Activate>,
             cards: Query<&CompanionCard>,
             mut choice: ResMut<MenuChoice>| {
                if let Ok(card) = cards.get(activate.entity) {
                    choice.0 = card.0.clone();
                }
            },
        )
        .id();
    for (label, value) in [("DANNO", dps), ("PORTATA", reach)] {
        let bar = stat_bar(commands, label, value, color);
        commands.entity(card).add_child(bar);
    }
    card
}

/// "DANNO [#####-----]" come nodi: etichetta e barra piena per `value` (0..1).
fn stat_bar(commands: &mut Commands, label: &str, value: f32, color: Color) -> Entity {
    commands
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            children![
                (
                    Text::new(label.to_string()),
                    fonts::hud(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                    Node {
                        width: px(68),
                        ..default()
                    },
                ),
                (
                    Node {
                        flex_grow: 1.0,
                        height: px(6),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    children![(
                        Node {
                            width: percent(100.0 * value.clamp(0.0, 1.0)),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(color),
                    )],
                ),
            ],
        ))
        .id()
}
