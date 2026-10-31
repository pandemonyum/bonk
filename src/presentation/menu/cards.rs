//! Griglia dei compagni della selezione: una carta per compagno dei dati, su `COLUMNS` colonne,
//! con ritratto, nome e stato ("Disponibile" con l'elemento, oppure ritratto scurito, lucchetto e
//! "Da sbloccare"). La carta scelta ha il bordo d'oro con l'alone, quella sotto il mouse si
//! schiarisce. Oltre `VISIBLE_ROWS` righe la griglia scorre (rotella, o da sola per tenere in
//! vista la carta scelta). Clic su una carta: la sceglie, anche se bloccata, per guardarla.

use bevy::picking::events::PointerScroll;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::{GLOW, GOLD, MenuChoice, SECONDARY, TITLE, element_label, portrait_path};
use crate::data::GameConfig;
use crate::data::schema::CompanionData;
use crate::presentation::fonts;
use crate::presentation::icons::{element_icon_path, ui_icon};
use crate::presentation::menu_list::add_corners;
use crate::profile::{Profile, is_unlocked};

/// Colonne della griglia.
pub(super) const COLUMNS: usize = 2;
/// Righe in vista senza scorrere.
const VISIBLE_ROWS: usize = 2;
/// Misure di una carta e spazio tra le carte (unità dell'interfaccia, pari).
pub(super) const CARD_W: f32 = 182.0;
const CARD_H: f32 = 160.0;
pub(super) const GAP: f32 = 10.0;
/// Spazio attorno alle carte dentro la parte che scorre: fregi e alone non vengono tagliati.
pub(super) const GRID_PAD: f32 = 10.0;

const FILL: Color = Color::srgba(0.106, 0.149, 0.251, 0.92);
const FILL_HOVER: Color = Color::srgba(0.169, 0.227, 0.369, 0.95);
/// Bordo di bronzo scuro (`h`), più chiaro sotto il mouse (`G`), d'oro se scelta (`g`).
const EDGE: Color = Color::srgb_u8(0x74, 0x49, 0x1f);
const EDGE_HOVER: Color = Color::srgb_u8(0xb8, 0x86, 0x2e);
/// Tinta del ritratto di un compagno da sbloccare (moltiplica i colori: resta riconoscibile).
const LOCKED_TINT: Color = Color::srgb(0.30, 0.33, 0.43);
/// "Disponibile": pillola verde della palette (`v`, `E`, `e`).
const PILL_FILL: Color = Color::srgb_u8(0x3c, 0x65, 0x26);
const PILL_EDGE: Color = Color::srgb_u8(0x74, 0xa8, 0x3c);
const PILL_TEXT: Color = Color::srgb_u8(0xe8, 0xf7, 0xa0);

/// Carta del compagno con questo id e la sua posizione nella griglia.
#[derive(Component)]
pub(super) struct CompanionCard {
    pub(super) id: String,
    index: usize,
}

/// La parte della griglia che scorre.
#[derive(Component)]
pub(super) struct CardGrid;

/// La griglia con il suo pannello: una carta per compagno di `config`.
pub(super) fn spawn_grid(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
    panel: Entity,
) {
    let rows = VISIBLE_ROWS
        .min(config.companions.len().div_ceil(COLUMNS))
        .max(1);
    let window = rows as f32 * (CARD_H + GAP) - GAP + 2.0 * GRID_PAD;
    let grid = commands
        .spawn((
            Name::new("Griglia dei compagni"),
            CardGrid,
            Node {
                display: Display::Grid,
                grid_template_columns: vec![RepeatedGridTrack::px(COLUMNS as i32, CARD_W)],
                grid_auto_rows: vec![GridTrack::px(CARD_H)],
                column_gap: px(GAP),
                row_gap: px(GAP),
                padding: UiRect::all(px(GRID_PAD)),
                max_height: px(window),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
        ))
        .observe(scroll_with_wheel)
        .id();
    commands.entity(panel).add_child(grid);
    for (index, companion) in config.companions.iter().enumerate() {
        let open = is_unlocked(config, profile, &companion.id);
        let card = spawn_card(commands, assets, index, companion, open);
        commands.entity(grid).add_child(card);
    }
}

fn spawn_card(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
    companion: &CompanionData,
    open: bool,
) -> Entity {
    let mut portrait = ImageNode::new(assets.load(portrait_path(companion)));
    if !open {
        portrait.color = LOCKED_TINT;
    }
    let card = commands
        .spawn((
            Name::new(format!("Carta {}", companion.id)),
            CompanionCard {
                id: companion.id.clone(),
                index,
            },
            Button,
            Hovered::default(),
            Node {
                width: px(CARD_W),
                height: px(CARD_H),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
                padding: UiRect::all(px(6)),
                border: UiRect::all(px(4)),
                ..default()
            },
            BorderColor::all(EDGE),
            BackgroundColor(FILL),
            BoxShadow::default(),
        ))
        .observe(
            |activate: On<Activate>,
             cards: Query<&CompanionCard>,
             mut choice: ResMut<MenuChoice>| {
                if let Ok(card) = cards.get(activate.entity)
                    && choice.0 != card.id
                {
                    choice.0 = card.id.clone();
                }
            },
        )
        .id();
    add_corners(commands, assets, card);
    // ritratto dell'art pass (40×36 texel) alla scala del canvas, il lucchetto sopra
    let frame = commands
        .spawn(Node {
            width: px(80),
            height: px(72),
            flex_shrink: 0.0,
            ..default()
        })
        .with_child((
            portrait,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .id();
    if !open {
        let lock = commands
            .spawn((
                ImageNode::new(assets.load("ui/lock.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(28),
                    bottom: px(6),
                    width: px(24),
                    height: px(28),
                    ..default()
                },
            ))
            .id();
        commands.entity(frame).add_child(lock);
    }
    let name = commands
        .spawn((
            Text::new(companion.display_name.clone()),
            fonts::ui(22.0),
            TextColor(if open { TITLE } else { SECONDARY }),
        ))
        .id();
    let status = status_row(commands, assets, companion, open);
    commands.entity(card).add_children(&[frame, name, status]);
    card
}

/// "Disponibile" con l'icona dell'elemento, oppure "Da sbloccare".
fn status_row(
    commands: &mut Commands,
    assets: &AssetServer,
    companion: &CompanionData,
    open: bool,
) -> Entity {
    let row = commands
        .spawn(Node {
            height: px(32),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(4),
            ..default()
        })
        .id();
    if !open {
        let text = commands
            .spawn((
                Text::new("Da sbloccare"),
                fonts::ui(16.0),
                TextColor(SECONDARY),
            ))
            .id();
        commands.entity(row).add_child(text);
        return row;
    }
    let (element, _) = element_label(companion.element);
    let icon = commands
        .spawn((
            Name::new(format!("Elemento {element}")),
            ui_icon(assets, element_icon_path(companion.element), 16.0, 16.0),
        ))
        .id();
    let pill = commands
        .spawn((
            Node {
                padding: UiRect::axes(px(8), px(3)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(PILL_FILL),
            BorderColor::all(PILL_EDGE),
            children![(
                Text::new("Disponibile"),
                fonts::ui(16.0),
                TextColor(PILL_TEXT)
            )],
        ))
        .id();
    commands.entity(row).add_children(&[icon, pill]);
    row
}

/// Bordo, fondo e alone di ogni carta: scelta, sotto il mouse o normale.
pub(super) fn style_cards(
    choice: Res<MenuChoice>,
    mut cards: Query<(
        &CompanionCard,
        &Hovered,
        &mut BorderColor,
        &mut BackgroundColor,
        &mut BoxShadow,
    )>,
) {
    for (card, hovered, mut border, mut fill, mut shadow) in &mut cards {
        let chosen = card.id == choice.0;
        let edge = if chosen {
            GOLD
        } else if hovered.get() {
            EDGE_HOVER
        } else {
            EDGE
        };
        let back = if hovered.get() || chosen {
            FILL_HOVER
        } else {
            FILL
        };
        if border.top != edge {
            *border = BorderColor::all(edge);
        }
        if fill.0 != back {
            fill.0 = back;
        }
        let glow =
            chosen.then(|| BoxShadow::new(GLOW.with_alpha(0.55), px(0), px(0), px(2), px(10)));
        let wanted = glow.unwrap_or_default();
        if *shadow != wanted {
            *shadow = wanted;
        }
    }
}

/// Scorrimento che tiene in vista la riga `row` (righe alte `row_h`, separate da `gap`) in una
/// finestra alta `window`, partendo da `scroll`.
pub fn scroll_to_row(scroll: f32, row: usize, row_h: f32, gap: f32, window: f32) -> f32 {
    let top = row as f32 * (row_h + gap);
    let bottom = top + row_h;
    if top < scroll {
        top
    } else if bottom > scroll + window {
        bottom - window
    } else {
        scroll
    }
}

/// Con la tastiera la carta scelta resta in vista.
pub(super) fn follow_choice(
    choice: Res<MenuChoice>,
    cards: Query<&CompanionCard>,
    mut grids: Query<&mut ScrollPosition, With<CardGrid>>,
) {
    if !choice.is_changed() {
        return;
    }
    let Some(card) = cards.iter().find(|c| c.id == choice.0) else {
        return;
    };
    let window = VISIBLE_ROWS as f32 * (CARD_H + GAP) - GAP;
    for mut scroll in &mut grids {
        let y = scroll_to_row(scroll.0.y, card.index / COLUMNS, CARD_H, GAP, window);
        if scroll.0.y != y {
            scroll.0.y = y;
        }
    }
}

/// Rotella sopra la griglia: una riga di carte per scatto.
fn scroll_with_wheel(
    wheel: On<PointerScroll>,
    mut grids: Query<(&mut ScrollPosition, &ComputedNode), With<CardGrid>>,
) {
    let Ok((mut scroll, node)) = grids.single_mut() else {
        return;
    };
    let units = match wheel.unit {
        bevy::input::mouse::MouseScrollUnit::Line => wheel.y * (CARD_H + GAP),
        bevy::input::mouse::MouseScrollUnit::Pixel => wheel.y,
    };
    let room = (node.content_size().y - node.size().y).max(0.0) * node.inverse_scale_factor();
    let y = (scroll.0.y - units).clamp(0.0, room);
    if scroll.0.y != y {
        scroll.0.y = y;
    }
}
