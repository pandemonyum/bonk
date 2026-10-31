//! Pannelli e liste di pulsanti del menu (pagina iniziale, opzioni), come nell'illustrazione di
//! riferimento dell'utente (D-063): pannello scuro con il bordo di bronzo e un fregio in ogni
//! angolo, pulsanti con icona e testo, quello evidenziato con il bordo d'oro e la freccia. Una
//! sola lista a schermo alla volta: `ListSelection` è la voce evidenziata, che segue W/S, le
//! frecce e il mouse.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::fonts;
use super::icons::{UI_PX_PER_TEXEL, ui_icon};
use super::menu::GOLD;
use crate::app::AppState;

pub(super) const FRAME: Color = Color::srgb_u8(0x9a, 0x74, 0x3c);
const FILL: Color = Color::srgba(0.07, 0.09, 0.14, 0.92);
const BUTTON: Color = Color::srgb_u8(0x1c, 0x23, 0x33);
const BUTTON_ON: Color = Color::srgb_u8(0x33, 0x2c, 0x1c);
pub(super) const EDGE: Color = Color::srgb_u8(0x46, 0x50, 0x68);
pub(super) const TEXT: Color = Color::srgb_u8(0xdf, 0xe6, 0xf2);

/// Pulsante numero `n` della lista a schermo.
#[derive(Component)]
pub(super) struct ListButton(pub(super) usize);

#[derive(Component)]
struct ListArrow(usize);

#[derive(Component)]
struct ListLabel(usize);

/// Voce evidenziata della lista a schermo.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct ListSelection(pub(super) usize);

impl ListSelection {
    /// La voce dopo W/S o le frecce su una lista di `count` voci, in giro.
    pub(super) fn stepped(self, keys: &ButtonInput<KeyCode>, count: usize) -> Self {
        let count = count.max(1);
        let mut index = self.0;
        if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            index = (index + count - 1) % count;
        }
        if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
            index = (index + 1) % count;
        }
        Self(index)
    }
}

/// W/S o frecce sulla voce evidenziata, scritta solo se cambia.
pub(super) fn step_selection(
    selection: &mut ResMut<ListSelection>,
    keys: &ButtonInput<KeyCode>,
    count: usize,
) {
    let next = selection.stepped(keys, count);
    if **selection != next {
        **selection = next;
    }
}

/// Invio o spazio, consumati: se la voce apre il pannello dei talenti, lo stesso tasto non vi
/// impara anche un talento. Da chiamare con `bypass_change_detection`, come fa Bevy con i tasti.
pub(super) fn confirmed(keys: &mut ButtonInput<KeyCode>) -> bool {
    const CONFIRM: [KeyCode; 3] = [KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space];
    let pressed = keys.any_just_pressed(CONFIRM);
    if pressed {
        for key in CONFIRM {
            keys.clear_just_pressed(key);
        }
    }
    pressed
}

/// Hover e colori delle liste: dopo i tasti delle schermate (`ListKeys`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ListKeys;

pub struct MenuListPlugin;

impl Plugin for MenuListPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ListSelection>().add_systems(
            Update,
            (hover_list, highlight_list)
                .chain()
                .after(ListKeys)
                .run_if(in_state(AppState::MainMenu)),
        );
    }
}

pub(super) fn panel_node(width: Val) -> Node {
    Node {
        width,
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        padding: UiRect::all(px(14)),
        border: UiRect::all(px(4)),
        ..default()
    }
}

/// Pannello scuro con il bordo di bronzo e un fregio in ogni angolo (`node` da `panel_node`).
pub(super) fn spawn_panel(commands: &mut Commands, assets: &AssetServer, node: Node) -> Entity {
    let panel = commands
        .spawn((node, BackgroundColor(FILL), BorderColor::all(FRAME)))
        .id();
    add_corners(commands, assets, panel);
    panel
}

/// Un fregio di bronzo in ogni angolo di `panel` (bordo di 4 unità), come nei pannelli.
pub(super) fn add_corners(commands: &mut Commands, assets: &AssetServer, panel: Entity) {
    for (flip_x, flip_y) in [(false, false), (true, false), (false, true), (true, true)] {
        // il fregio è disegnato per l'angolo in alto a sinistra: specchiato per gli altri
        let mut image = ImageNode::new(assets.load("ui/frame_corner.png"));
        image.flip_x = flip_x;
        image.flip_y = flip_y;
        let edge = px(-8);
        let corner = commands
            .spawn((
                image,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(20),
                    height: px(20),
                    left: if flip_x { Val::Auto } else { edge },
                    right: if flip_x { edge } else { Val::Auto },
                    top: if flip_y { Val::Auto } else { edge },
                    bottom: if flip_y { edge } else { Val::Auto },
                    ..default()
                },
            ))
            .id();
        commands.entity(panel).add_child(corner);
    }
}

/// Pulsante `index` di una lista con l'icona di 16×16 texel; senza icona uno spazio della stessa
/// misura, così i testi restano in colonna. Il testo è un figlio a parte, per le schermate che lo
/// cambiano.
pub(super) fn spawn_list_button(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
    icon: Option<&str>,
    label: &str,
) -> Entity {
    let button = commands
        .spawn((
            Name::new(format!("Voce {label}")),
            ListButton(index),
            Button,
            Hovered::default(),
            Node {
                width: percent(100),
                height: px(44),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(14),
                padding: UiRect::horizontal(px(14)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(EDGE),
            BackgroundColor(BUTTON),
            children![(
                ListArrow(index),
                ImageNode::new(assets.load("ui/menu_arrow.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(-24),
                    top: px(9),
                    width: px(14),
                    height: px(22),
                    ..default()
                },
                Visibility::Hidden,
            )],
        ))
        .id();
    let icon = match icon {
        Some(icon) => commands.spawn(ui_icon(assets, icon, 16.0, 16.0)).id(),
        None => commands
            .spawn(Node {
                width: px(16.0 * UI_PX_PER_TEXEL),
                height: px(16.0 * UI_PX_PER_TEXEL),
                flex_shrink: 0.0,
                ..default()
            })
            .id(),
    };
    commands.entity(button).add_child(icon);
    let text = commands
        .spawn((
            ListLabel(index),
            Text::new(label.to_string()),
            fonts::ui_unligated(22.0),
            TextColor(TEXT),
        ))
        .id();
    commands.entity(button).add_child(text);
    button
}

/// Il mouse sopra un pulsante lo evidenzia.
fn hover_list(
    buttons: Query<(&ListButton, &Hovered), Changed<Hovered>>,
    mut selection: ResMut<ListSelection>,
) {
    for (button, hovered) in &buttons {
        if hovered.get() && selection.0 != button.0 {
            selection.0 = button.0;
        }
    }
}

fn highlight_list(
    selection: Res<ListSelection>,
    mut buttons: Query<(&ListButton, &mut BorderColor, &mut BackgroundColor)>,
    mut labels: Query<(&ListLabel, &mut TextColor)>,
    mut arrows: Query<(&ListArrow, &mut Visibility)>,
) {
    for (button, mut border, mut fill) in &mut buttons {
        let (edge, color) = if button.0 == selection.0 {
            (GOLD, BUTTON_ON)
        } else {
            (EDGE, BUTTON)
        };
        if fill.0 != color {
            *border = BorderColor::all(edge);
            fill.0 = color;
        }
    }
    for (label, mut text) in &mut labels {
        let color = if label.0 == selection.0 { GOLD } else { TEXT };
        if text.0 != color {
            text.0 = color;
        }
    }
    for (arrow, mut visibility) in &mut arrows {
        let wanted = if arrow.0 == selection.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_move_the_selection_around_the_list() {
        let mut keys = ButtonInput::<KeyCode>::default();
        let mut selection = ListSelection(0);
        keys.press(KeyCode::ArrowUp);
        selection = selection.stepped(&keys, 5);
        assert_eq!(selection.0, 4);
        keys.clear();
        keys.release(KeyCode::ArrowUp);
        keys.clear();
        keys.press(KeyCode::KeyS);
        selection = selection.stepped(&keys, 5);
        assert_eq!(selection.0, 0);
        keys.press(KeyCode::Enter);
        assert!(confirmed(&mut keys));
        assert!(!keys.just_pressed(KeyCode::Enter), "il tasto è consumato");
        assert!(!confirmed(&mut keys));
    }
}
