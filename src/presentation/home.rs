//! Pagina iniziale (`AppState::MainMenu`, schermata `MenuScreen::Home`, D-063): l'illustrazione
//! dell'utente a tutto schermo, il logo e il pannello con Nuova partita, Compagni, Talenti,
//! Opzioni ed Esci (non nel browser, dove non c'è niente da chiudere). W/S o frecce e Invio,
//! oppure il mouse. "Nuova partita" parte con il compagno scelto in "Compagni" (all'inizio
//! l'ultimo con cui si è giocato, scritto in fondo al pannello). L'illustrazione resta sotto
//! anche alle altre schermate.

use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::fonts;
use super::icons::ui_icon;
use super::menu::{MenuChoice, PANEL, StartRun};
use super::menu_list::{
    ListKeys, ListSelection, TEXT, confirmed, panel_node, spawn_list_button, spawn_panel,
    step_selection,
};
use super::talents_menu::{TalentPanel, panel_closed, talent_keys};
use crate::app::AppState;
use crate::data::GameConfig;

/// Schermate del menu, sopra l'illustrazione comune.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::MainMenu)]
pub enum MenuScreen {
    #[default]
    Home,
    /// Scelta del compagno (`menu`).
    Companions,
    /// `options_menu`.
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeEntry {
    NewRun,
    Companions,
    Talents,
    Options,
    Quit,
}

impl HomeEntry {
    pub fn label(self) -> &'static str {
        match self {
            Self::NewRun => "Nuova Partita",
            Self::Companions => "Compagni",
            Self::Talents => "Talenti",
            Self::Options => "Opzioni",
            Self::Quit => "Esci",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::NewRun => "ui/menu_new_run.png",
            Self::Companions => "ui/menu_companions.png",
            Self::Talents => "ui/menu_talents.png",
            Self::Options => "ui/menu_options.png",
            Self::Quit => "ui/menu_quit.png",
        }
    }
}

/// Le voci della pagina: nel browser niente Esci.
pub fn home_entries(web: bool) -> Vec<HomeEntry> {
    let mut entries = vec![
        HomeEntry::NewRun,
        HomeEntry::Companions,
        HomeEntry::Talents,
        HomeEntry::Options,
    ];
    if !web {
        entries.push(HomeEntry::Quit);
    }
    entries
}

fn entries() -> Vec<HomeEntry> {
    home_entries(cfg!(target_arch = "wasm32"))
}

const ILLUSTRATION: &str = "ui/home_background.png";
const LOGO: &str = "ui/logo.png";
/// Misura del logo in texel (`assets/ui/logo.png`).
const LOGO_SIZE: Vec2 = Vec2::new(222.0, 90.0);
/// Che parte dell'illustrazione resta quando la finestra ne taglia un pezzo (0 = il bordo
/// sinistro o in alto, 1 = il destro o in basso): player e compagni sono in basso a destra.
const FOCUS: Vec2 = Vec2::new(0.65, 0.75);

#[derive(Component)]
struct Backdrop;

/// "Parti con …" in fondo al pannello: anche i talenti (Tab) cambiano il compagno scelto.
#[derive(Component)]
struct ChosenCompanion;

#[derive(Component)]
struct BackdropImage;

/// Misura e angolo in alto a sinistra dell'immagine `image` che copre tutta `area` senza
/// deformarsi; quello che avanza si taglia secondo `focus`.
pub fn cover(area: Vec2, image: Vec2, focus: Vec2) -> (Vec2, Vec2) {
    if image.x <= 0.0 || image.y <= 0.0 {
        return (area, Vec2::ZERO);
    }
    let size = image * (area.x / image.x).max(area.y / image.y);
    (size, (area - size) * focus)
}

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MenuScreen>()
            .add_systems(OnEnter(AppState::MainMenu), spawn_backdrop)
            .add_systems(OnEnter(MenuScreen::Home), spawn_home)
            .add_systems(Update, fit_backdrop.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, show_choice.run_if(in_state(MenuScreen::Home)))
            .add_systems(
                Update,
                home_keys
                    .run_if(panel_closed)
                    .before(talent_keys)
                    .in_set(ListKeys)
                    .run_if(in_state(MenuScreen::Home)),
            );
    }
}

fn spawn_backdrop(mut commands: Commands, assets: Res<AssetServer>) {
    // l'illustrazione non è sulla griglia dei pixel del gioco: filtro lineare
    let image = assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::linear();
        })
        .load(ILLUSTRATION);
    commands.spawn((
        Name::new("Illustrazione"),
        Backdrop,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(PANEL),
        GlobalZIndex(-1),
        DespawnOnExit(AppState::MainMenu),
        children![(
            BackdropImage,
            ImageNode::new(image),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        )],
    ));
}

/// L'illustrazione copre la finestra senza deformarsi, qualunque sia la sua forma.
fn fit_backdrop(
    roots: Query<&ComputedNode, With<Backdrop>>,
    mut pictures: Query<(&ImageNode, &mut Node), With<BackdropImage>>,
    images: Res<Assets<Image>>,
) {
    let (Ok(root), Ok((picture, mut node))) = (roots.single(), pictures.single_mut()) else {
        return;
    };
    let Some(image) = images.get(&picture.image) else {
        return;
    };
    let area = root.size() * root.inverse_scale_factor();
    let (size, at) = cover(area, image.size_f32(), FOCUS);
    let wanted = (px(size.x), px(size.y), px(at.x), px(at.y));
    if (node.width, node.height, node.left, node.top) != wanted {
        (node.width, node.height, node.left, node.top) = wanted;
    }
}

fn spawn_home(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut selection: ResMut<ListSelection>,
) {
    *selection = ListSelection::default();
    let root = commands
        .spawn((
            Name::new("Home"),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                padding: UiRect {
                    left: px(28),
                    top: px(12),
                    ..default()
                },
                row_gap: px(6),
                ..default()
            },
            DespawnOnExit(MenuScreen::Home),
        ))
        .id();
    let logo = commands
        .spawn(ui_icon(&assets, LOGO, LOGO_SIZE.x, LOGO_SIZE.y))
        .id();
    let panel = spawn_panel(
        &mut commands,
        &assets,
        Node {
            margin: UiRect::left(px(16)),
            ..panel_node(px(300))
        },
    );
    for (i, entry) in entries().into_iter().enumerate() {
        let button =
            spawn_list_button(&mut commands, &assets, i, Some(entry.icon()), entry.label());
        commands
            .entity(button)
            .observe(move |_: On<Activate>, mut actions: HomeActions| actions.activate(entry));
        commands.entity(panel).add_child(button);
    }
    let chosen = commands
        .spawn((
            ChosenCompanion,
            Text::new(""),
            fonts::hud(16.0),
            TextColor(TEXT.with_alpha(0.8)),
            Node {
                align_self: AlignSelf::Center,
                ..default()
            },
        ))
        .id();
    commands.entity(panel).add_child(chosen);
    let version = corner_text(
        &mut commands,
        format!("v{}", env!("CARGO_PKG_VERSION")),
        true,
    );
    let hint = corner_text(
        &mut commands,
        "W/S o frecce · Invio · T talenti".into(),
        false,
    );
    commands
        .entity(root)
        .add_children(&[logo, panel, version, hint]);
}

/// Scritta piccola in basso a sinistra o a destra (l'ombra del font HUD la fa leggere
/// sull'illustrazione).
fn corner_text(commands: &mut Commands, text: String, left: bool) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        bottom: px(8),
        ..default()
    };
    if left {
        node.left = px(10);
    } else {
        node.right = px(10);
    }
    commands
        .spawn((
            Text::new(text),
            fonts::hud(16.0),
            TextColor(TEXT.with_alpha(0.85)),
            node,
        ))
        .id()
}

fn show_choice(
    config: Res<GameConfig>,
    choice: Res<MenuChoice>,
    mut lines: Query<&mut Text, With<ChosenCompanion>>,
) {
    let name = config
        .companion(&choice.0)
        .map_or("", |c| c.display_name.as_str());
    let wanted = format!("Parti con {name}");
    for mut text in &mut lines {
        if text.0 != wanted {
            text.0.clone_from(&wanted);
        }
    }
}

/// Cosa fa ogni voce della pagina.
#[derive(SystemParam)]
struct HomeActions<'w> {
    start: StartRun<'w>,
    choice: Res<'w, MenuChoice>,
    screen: ResMut<'w, NextState<MenuScreen>>,
    panel: ResMut<'w, TalentPanel>,
    exit: MessageWriter<'w, AppExit>,
}

impl HomeActions<'_> {
    fn activate(&mut self, entry: HomeEntry) {
        match entry {
            HomeEntry::NewRun => {
                let id = self.choice.0.clone();
                self.start.go(&id);
            }
            HomeEntry::Companions => self.screen.set(MenuScreen::Companions),
            HomeEntry::Talents => self.panel.open = true,
            HomeEntry::Options => self.screen.set(MenuScreen::Options),
            HomeEntry::Quit => {
                self.exit.write(AppExit::Success);
            }
        }
    }
}

fn home_keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut selection: ResMut<ListSelection>,
    mut actions: HomeActions,
) {
    let list = entries();
    step_selection(&mut selection, &keys, list.len());
    if confirmed(keys.bypass_change_detection())
        && let Some(entry) = list.get(selection.0)
    {
        actions.activate(*entry);
    }
}

#[cfg(test)]
mod tests;
