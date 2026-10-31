//! La schermata della selezione: un velo leggero sull'illustrazione, a sinistra il cartiglio del
//! titolo e la griglia, a destra i dettagli, al centro niente (si vede l'illustrazione), in basso
//! Indietro, la riga dei tasti e Inizia Partita (spento, con il lucchetto, per un compagno da
//! sbloccare). Misure in unità dell'interfaccia (2 per texel), pari: tutto sta in 960×540 unità
//! (1920×1080); nelle finestre più grandi cresce solo lo spazio vuoto al centro.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, Button};

use super::cards::{CARD_W, COLUMNS, GAP, GRID_PAD, spawn_grid};
use super::details::spawn_details;
use super::{GOLD, MenuChoice, SECONDARY, StartRun, TITLE};
use crate::data::GameConfig;
use crate::presentation::fonts;
use crate::presentation::home::MenuScreen;
use crate::presentation::icons::{UI_PX_PER_TEXEL, ui_icon};
use crate::presentation::menu_list::{FRAME, TEXT, add_corners, spawn_panel};
use crate::profile::{Profile, is_unlocked};

const TITLE_TEXT: &str = "SELEZIONA IL TUO COMPAGNO INIZIALE";
/// Bordo e margini dei pannelli di bronzo attorno alla griglia.
const PANEL_EDGE: f32 = 4.0;
const PANEL_PAD: f32 = 4.0;
/// Inizia Partita: verdi della palette (`v`, `E`, `e`, `i`), spento nei blu dei pannelli.
const GO_FILL: Color = Color::srgb_u8(0x3c, 0x65, 0x26);
const GO_FILL_HOVER: Color = Color::srgb_u8(0x4f, 0x86, 0x31);
const GO_EDGE: Color = Color::srgb_u8(0xb8, 0xe0, 0x5a);
const GO_EDGE_HOVER: Color = Color::srgb_u8(0xe8, 0xf7, 0xa0);
const OFF_FILL: Color = Color::srgb_u8(0x1b, 0x26, 0x40);
const OFF_EDGE: Color = Color::srgb_u8(0x5a, 0x6d, 0x99);
const BACK_FILL: Color = Color::srgba(0.07, 0.09, 0.14, 0.92);
const BACK_FILL_HOVER: Color = Color::srgba(0.2, 0.17, 0.11, 0.95);

#[derive(Component)]
pub(super) struct StartButton;

#[derive(Component)]
pub(super) struct BackButton;

/// Testo di un pulsante, che cambia colore con lui.
#[derive(Component)]
pub(super) struct ButtonLabel;

/// Freccia d'oro di Inizia Partita, visibile solo se si può partire.
#[derive(Component)]
pub(super) struct StartArrow;

/// Lucchetto di Inizia Partita spento.
#[derive(Component)]
pub(super) struct StartLock;

/// Larghezza della colonna di sinistra: due carte e i bordi.
fn column_width() -> f32 {
    COLUMNS as f32 * CARD_W
        + (COLUMNS as f32 - 1.0) * GAP
        + 2.0 * (GRID_PAD + PANEL_PAD + PANEL_EDGE)
}

pub(super) fn spawn_select(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    profile: Res<Profile>,
) {
    let root = commands
        .spawn((
            Name::new("Selezione"),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: px(16),
                    right: px(16),
                    top: px(14),
                    bottom: px(12),
                },
                row_gap: px(8),
                ..default()
            },
            // l'illustrazione resta ben visibile sotto
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
            DespawnOnExit(MenuScreen::Select),
        ))
        .id();
    let body = commands
        .spawn(Node {
            flex_grow: 1.0,
            min_height: px(0),
            flex_direction: FlexDirection::Row,
            // griglia a sinistra, dettagli a destra, in mezzo lo sfondo
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(10),
            ..default()
        })
        .id();
    let left = spawn_left(&mut commands, &assets, &config, &profile);
    let details = spawn_details(&mut commands, &assets);
    commands.entity(body).add_children(&[left, details]);
    let bottom = spawn_bottom(&mut commands, &assets, config.companions.len());
    commands.entity(root).add_children(&[body, bottom]);
}

/// Cartiglio del titolo e pannello della griglia.
fn spawn_left(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    profile: &Profile,
) -> Entity {
    let width = column_width();
    let column = commands
        .spawn(Node {
            width: px(width),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: px(8),
            ..default()
        })
        .id();
    let title = spawn_panel(
        commands,
        assets,
        Node {
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(10), px(8)),
            border: UiRect::all(px(PANEL_EDGE)),
            ..default()
        },
    );
    let words = commands
        .spawn((
            Text::new(TITLE_TEXT),
            fonts::ui(22.0),
            TextColor(TITLE),
            TextLayout::no_wrap(),
        ))
        .id();
    commands.entity(title).add_child(words);
    let grid = spawn_panel(
        commands,
        assets,
        Node {
            padding: UiRect::all(px(PANEL_PAD)),
            border: UiRect::all(px(PANEL_EDGE)),
            ..default()
        },
    );
    spawn_grid(commands, assets, config, profile, grid);
    commands.entity(column).add_children(&[title, grid]);
    column
}

/// Indietro, la riga dei tasti e Inizia Partita.
fn spawn_bottom(commands: &mut Commands, assets: &AssetServer, companions: usize) -> Entity {
    let back = button_shell(commands, assets, "Indietro");
    let back_arrow = commands.spawn(arrow(assets, true)).id();
    let back_label = commands
        .spawn((
            ButtonLabel,
            Text::new("Indietro"),
            fonts::ui(22.0),
            TextColor(TEXT),
        ))
        .id();
    commands
        .entity(back)
        .insert((BackButton, BackgroundColor(BACK_FILL)))
        .add_children(&[back_arrow, back_label])
        .observe(
            |_: On<Activate>, mut screen: ResMut<NextState<MenuScreen>>| {
                screen.set(MenuScreen::Home)
            },
        );
    let start = button_shell(commands, assets, "Inizia Partita");
    let lock = commands
        .spawn((
            StartLock,
            ui_icon(assets, "ui/lock.png", 12.0, 14.0),
            Visibility::Hidden,
        ))
        .id();
    let start_label = commands
        .spawn((
            ButtonLabel,
            Text::new("Inizia Partita"),
            fonts::ui(22.0),
            TextColor(TITLE),
        ))
        .id();
    let start_arrow = commands.spawn((StartArrow, arrow(assets, false))).id();
    commands
        .entity(start)
        .insert((
            StartButton,
            BackgroundColor(GO_FILL),
            BorderColor::all(GO_EDGE),
        ))
        .add_children(&[lock, start_label, start_arrow])
        .observe(
            |_: On<Activate>, choice: Res<MenuChoice>, mut start: StartRun| {
                let _ = start.go(&choice.0);
            },
        );
    let keys = format!(
        "Frecce, 1–{} o clic: scegli · Invio: inizia · Esc: indietro · T: talenti",
        companions.clamp(1, 9)
    );
    // su una striscia scura: sotto c'è l'illustrazione, piena di dettagli
    let hint = commands
        .spawn((
            Node {
                flex_shrink: 1.0,
                margin: UiRect::horizontal(px(12)),
                padding: UiRect::axes(px(10), px(4)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.1, 0.72)),
            children![(
                Text::new(keys),
                fonts::hud(16.0),
                TextColor(TEXT),
                TextLayout::justify(Justify::Center),
            )],
        ))
        .id();
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .add_children(&[back, hint, start])
        .id()
}

/// Pulsante con il bordo e i fregi dei pannelli; il contenuto lo aggiunge chi chiama.
fn button_shell(commands: &mut Commands, assets: &AssetServer, name: &str) -> Entity {
    let button = commands
        .spawn((
            Name::new(format!("Pulsante {name}")),
            Button,
            Hovered::default(),
            Node {
                height: px(48),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
                padding: UiRect::horizontal(px(20)),
                border: UiRect::all(px(PANEL_EDGE)),
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor::all(FRAME),
        ))
        .id();
    add_corners(commands, assets, button);
    button
}

/// La freccia d'oro del menu (7×11 texel), girata verso sinistra per Indietro.
fn arrow(assets: &AssetServer, back: bool) -> impl Bundle {
    let mut image = ImageNode::new(assets.load("ui/menu_arrow.png"));
    image.flip_x = back;
    (
        image,
        Node {
            width: px(7.0 * UI_PX_PER_TEXEL),
            height: px(11.0 * UI_PX_PER_TEXEL),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// Inizia Partita acceso, sotto il mouse o spento (compagno da sbloccare: niente clic, il
/// lucchetto al posto della freccia); Indietro d'oro sotto il mouse.
pub(super) fn style_buttons(
    mut commands: Commands,
    config: Res<GameConfig>,
    profile: Res<Profile>,
    choice: Res<MenuChoice>,
    mut starts: Query<
        (
            Entity,
            &Hovered,
            Has<InteractionDisabled>,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (With<StartButton>, Without<BackButton>),
    >,
    mut backs: Query<
        (&Hovered, &mut BackgroundColor, &mut BorderColor, &Children),
        (With<BackButton>, Without<StartButton>),
    >,
    mut labels: Query<&mut TextColor, With<ButtonLabel>>,
    mut arrows: Query<&mut Visibility, (With<StartArrow>, Without<StartLock>)>,
    mut locks: Query<&mut Visibility, (With<StartLock>, Without<StartArrow>)>,
) {
    let open = is_unlocked(&config, &profile, &choice.0);
    for (button, hovered, disabled, fill, border, children) in &mut starts {
        if disabled == open {
            if open {
                commands.entity(button).remove::<InteractionDisabled>();
            } else {
                commands.entity(button).insert(InteractionDisabled);
            }
        }
        let (back, edge, text) = match (open, hovered.get()) {
            (false, _) => (OFF_FILL, OFF_EDGE, SECONDARY.with_alpha(0.7)),
            (true, true) => (GO_FILL_HOVER, GO_EDGE_HOVER, TITLE),
            (true, false) => (GO_FILL, GO_EDGE, TITLE),
        };
        set_colors(fill, border, back, edge);
        for child in children {
            if let Ok(mut color) = labels.get_mut(*child)
                && color.0 != text
            {
                color.0 = text;
            }
            if let Ok(visibility) = arrows.get_mut(*child) {
                show(visibility, open);
            }
            if let Ok(visibility) = locks.get_mut(*child) {
                show(visibility, !open);
            }
        }
    }
    for (hovered, fill, border, children) in &mut backs {
        let (back, edge, text) = if hovered.get() {
            (BACK_FILL_HOVER, GOLD, GOLD)
        } else {
            (BACK_FILL, FRAME, TEXT)
        };
        set_colors(fill, border, back, edge);
        for child in children {
            if let Ok(mut color) = labels.get_mut(*child)
                && color.0 != text
            {
                color.0 = text;
            }
        }
    }
}

/// Scrive solo se cambia: niente rilevamento dei cambi a ogni fotogramma.
fn set_colors(
    mut fill: Mut<BackgroundColor>,
    mut border: Mut<BorderColor>,
    back: Color,
    edge: Color,
) {
    if fill.0 != back {
        fill.0 = back;
    }
    if border.top != edge {
        *border = BorderColor::all(edge);
    }
}

fn show(mut visibility: Mut<Visibility>, on: bool) {
    let wanted = if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != wanted {
        *visibility = wanted;
    }
}
