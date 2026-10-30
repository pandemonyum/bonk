//! Pagina iniziale: copertura dell'illustrazione, voci, schermate e tasti tra le schermate e il
//! pannello dei talenti.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::state::app::StatesPlugin;

use super::*;
use crate::app::{AfterLoading, RunSetup};
use crate::presentation::menu::GOLD;
use crate::presentation::menu_list::ListSelection;
use crate::profile::{CompanionProgress, Profile};

#[test]
fn the_illustration_covers_any_window_without_stretching() {
    let image = Vec2::new(1448.0, 1086.0);
    // finestra 16:9 più larga dell'immagine 4:3: si taglia in altezza, più sopra che sotto
    let (size, at) = cover(Vec2::new(960.0, 540.0), image, FOCUS);
    assert_eq!(size.x, 960.0);
    assert!((size.y - 720.0).abs() < 1e-3);
    assert!((at.y - -135.0).abs() < 1e-3, "{at}");
    assert_eq!(at.x, 0.0);
    // finestra alta e stretta: si taglia ai lati
    let (size, at) = cover(Vec2::new(400.0, 800.0), image, FOCUS);
    assert_eq!(size.y, 800.0);
    assert!(at.x < 0.0 && at.y == 0.0);
    assert!((size.x / size.y - image.x / image.y).abs() < 1e-4);
}

#[test]
fn the_browser_has_no_quit_entry() {
    assert_eq!(home_entries(false).last(), Some(&HomeEntry::Quit));
    assert!(!home_entries(true).contains(&HomeEntry::Quit));
    assert_eq!(home_entries(true)[0], HomeEntry::NewRun);
}

/// Il menu senza finestra, con `profile` e tutte le schermate e il pannello dei talenti.
fn menu_app(profile: Profile) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin::default(),
        bevy::input::InputPlugin,
    ))
    .init_asset::<Image>()
    .insert_state(AppState::MainMenu)
    .insert_resource(GameConfig::embedded())
    .insert_resource(RunSetup::default())
    .insert_resource(AfterLoading(AppState::MainMenu))
    .insert_resource(profile)
    .add_plugins((
        HomePlugin,
        crate::presentation::menu_list::MenuListPlugin,
        crate::presentation::menu::MenuPlugin,
        crate::presentation::options_menu::OptionsMenuPlugin,
        crate::presentation::talents_menu::TalentsMenuPlugin,
    ));
    app.update();
    app
}

fn named(app: &mut App, name: &str) -> usize {
    let world = app.world_mut();
    let mut names = world.query::<&Name>();
    names.iter(world).filter(|n| n.as_str() == name).count()
}

fn go(app: &mut App, screen: MenuScreen) {
    app.world_mut()
        .resource_mut::<NextState<MenuScreen>>()
        .set(screen);
    app.update();
}

fn screen(app: &App) -> MenuScreen {
    *app.world().resource::<State<MenuScreen>>().get()
}

/// Un tasto premuto e lasciato, come dalla tastiera (passa dall'`InputPlugin`).
fn tap(app: &mut App, key: KeyCode) {
    let message = |state| KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    };
    app.world_mut().write_message(message(ButtonState::Pressed));
    app.update();
    app.world_mut()
        .write_message(message(ButtonState::Released));
    app.update();
}

/// Ogni schermata del menu si costruisce (un componente doppio in un bundle fa panic solo
/// allo spawn) e lascia il posto alla successiva.
#[test]
fn every_menu_screen_builds() {
    let mut app = menu_app(Profile::default());
    assert_eq!(named(&mut app, "Home"), 1);
    assert_eq!(named(&mut app, "Illustrazione"), 1);
    for (target, name) in [
        (MenuScreen::Companions, "MainMenu"),
        (MenuScreen::Options, "Opzioni"),
        (MenuScreen::Home, "Home"),
    ] {
        go(&mut app, target);
        assert_eq!(named(&mut app, name), 1, "{target:?}");
    }
    assert_eq!(
        named(&mut app, "Opzioni"),
        0,
        "le opzioni chiuse spariscono"
    );
    app.world_mut().resource_mut::<TalentPanel>().open = true;
    app.update();
    assert_eq!(named(&mut app, "Talenti"), 1);
}

/// Rientrando in "Compagni" la carta scelta è di nuovo evidenziata.
#[test]
fn the_chosen_card_is_highlighted_every_time() {
    let mut app = menu_app(Profile::default());
    let chosen = format!("Carta {}", app.world().resource::<MenuChoice>().0);
    for target in [
        MenuScreen::Companions,
        MenuScreen::Home,
        MenuScreen::Companions,
    ] {
        go(&mut app, target);
    }
    let world = app.world_mut();
    let mut cards = world.query::<(&Name, &BorderColor)>();
    let border = cards
        .iter(world)
        .find(|(name, _)| name.as_str() == chosen)
        .map(|(_, border)| *border);
    assert_eq!(border, Some(BorderColor::all(GOLD)));
}

/// Invio su "Talenti" apre il pannello e basta: non dà anche un grado al primo nodo.
#[test]
fn enter_on_talents_only_opens_the_panel() {
    let config = GameConfig::embedded();
    let starter = config.stage.starting_companion.clone();
    let rich = Profile {
        data: crate::profile::ProfileData {
            companions: vec![CompanionProgress {
                id: starter,
                xp: u32::MAX / 2,
                learned: vec![],
            }],
            ..default()
        },
        path: None,
    };
    let mut app = menu_app(rich);
    let talents = entries()
        .iter()
        .position(|e| *e == HomeEntry::Talents)
        .unwrap();
    app.world_mut().resource_mut::<ListSelection>().0 = talents;
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<TalentPanel>().open);
    let profile = app.world().resource::<Profile>();
    assert!(
        profile.data.companions.iter().all(|c| c.learned.is_empty()),
        "{:?}",
        profile.data.companions
    );
    // con il pannello aperto Invio impara davvero
    tap(&mut app, KeyCode::Enter);
    let profile = app.world().resource::<Profile>();
    assert_eq!(profile.data.companions[0].learned.len(), 1);
}

/// L'Esc che chiude il pannello dei talenti non torna anche alla pagina iniziale.
#[test]
fn escape_closes_the_talents_panel_before_leaving_a_screen() {
    let mut app = menu_app(Profile::default());
    for target in [MenuScreen::Companions, MenuScreen::Options] {
        go(&mut app, target);
        app.world_mut().resource_mut::<TalentPanel>().open = true;
        app.update();
        tap(&mut app, KeyCode::Escape);
        assert!(!app.world().resource::<TalentPanel>().open);
        assert_eq!(screen(&app), target, "resta sulla schermata");
        tap(&mut app, KeyCode::Escape);
        assert_eq!(screen(&app), MenuScreen::Home);
    }
}
