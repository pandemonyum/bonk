//! Opzioni (dalla pagina iniziale, `MenuScreen::Options`): per ora lo schermo intero. W/S o
//! frecce per la voce, Invio, spazio, A/D o clic per cambiarla, Esc o "Indietro" per tornare.
//! Sul desktop la scelta si salva nel profilo e vale anche dall'avvio dopo. Nel browser vale per
//! la visita (lo schermo intero si può chiedere solo subito dopo un clic o un tasto) e il valore
//! mostrato è quello vero della pagina, che ne esce anche da sola con Esc.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use super::fonts;
use super::home::MenuScreen;
use super::menu::GOLD;
use super::menu_list::{
    ListKeys, ListSelection, TEXT, confirmed, panel_node, spawn_list_button, spawn_panel,
    step_selection,
};
use super::talents_menu::{panel_closed, talent_keys};
use crate::options::LaunchOptions;
use crate::profile::Profile;

const FULLSCREEN: usize = 0;
const BACK: usize = 1;
const ENTRIES: usize = 2;

/// Il "Sì"/"No" accanto a "Schermo intero".
#[derive(Component)]
struct FullscreenValue;

pub fn window_mode(fullscreen: bool) -> WindowMode {
    if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    }
}

/// Lo schermo intero adesso: sul web lo stato vero della pagina, sul desktop la finestra.
fn fullscreen_now(window: &Window) -> bool {
    if cfg!(target_arch = "wasm32") {
        web_fullscreen()
    } else {
        !matches!(window.mode, WindowMode::Windowed)
    }
}

#[cfg(target_arch = "wasm32")]
fn web_fullscreen() -> bool {
    crate::web::fullscreen()
}

#[cfg(not(target_arch = "wasm32"))]
fn web_fullscreen() -> bool {
    false
}

/// Si parte a schermo intero se è salvato e il profilo si scrive (non in autoplay, `--god`,
/// `--time-scale`), tranne nelle run automatiche: screenshot e uscita a tempo restano nella
/// finestra chiesta, a misura nota.
pub fn starts_fullscreen(profile: &Profile, options: Option<&LaunchOptions>) -> bool {
    let automated = options.is_some_and(|o| o.screenshot.is_some() || o.exit_after_secs.is_some());
    profile.data.fullscreen && profile.path.is_some() && !automated
}

pub struct OptionsMenuPlugin;

impl Plugin for OptionsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_saved_mode)
            .add_systems(OnEnter(MenuScreen::Options), spawn_options)
            .add_systems(
                Update,
                (
                    options_keys
                        .run_if(panel_closed)
                        .before(talent_keys)
                        .in_set(ListKeys),
                    show_value,
                )
                    .run_if(in_state(MenuScreen::Options)),
            );
    }
}

/// Sul desktop lo schermo intero salvato vale dall'avvio (`starts_fullscreen`).
fn apply_saved_mode(
    profile: Option<Res<Profile>>,
    options: Option<Res<LaunchOptions>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    if let (Some(profile), Ok(mut window)) = (profile, windows.single_mut())
        && starts_fullscreen(&profile, options.as_deref())
    {
        window.mode = window_mode(true);
    }
}

fn spawn_options(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut selection: ResMut<ListSelection>,
) {
    *selection = ListSelection::default();
    let root = commands
        .spawn((
            Name::new("Opzioni"),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            DespawnOnExit(MenuScreen::Options),
        ))
        .id();
    let panel = spawn_panel(&mut commands, &assets, panel_node(px(340)));
    let title = commands
        .spawn((
            Text::new("Opzioni"),
            fonts::ui(32.0),
            TextColor(GOLD),
            Node {
                align_self: AlignSelf::Center,
                margin: UiRect::bottom(px(4)),
                ..default()
            },
        ))
        .id();
    let fullscreen = spawn_list_button(
        &mut commands,
        &assets,
        FULLSCREEN,
        Some("ui/menu_options.png"),
        "Schermo intero",
    );
    let value = commands
        .spawn((
            FullscreenValue,
            Text::new(""),
            fonts::ui(22.0),
            TextColor(TEXT),
            Node {
                margin: UiRect::left(Val::Auto),
                ..default()
            },
        ))
        .id();
    commands
        .entity(fullscreen)
        .add_child(value)
        .observe(|_: On<Activate>, mut actions: OptionsActions| actions.activate(FULLSCREEN));
    let back = spawn_list_button(
        &mut commands,
        &assets,
        BACK,
        Some("ui/menu_quit.png"),
        "Indietro",
    );
    commands
        .entity(back)
        .observe(|_: On<Activate>, mut actions: OptionsActions| actions.activate(BACK));
    commands
        .entity(panel)
        .add_children(&[title, fullscreen, back]);
    commands.entity(root).add_child(panel);
}

#[derive(SystemParam)]
struct OptionsActions<'w, 's> {
    windows: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
    profile: Option<ResMut<'w, Profile>>,
    screen: ResMut<'w, NextState<MenuScreen>>,
}

impl OptionsActions<'_, '_> {
    fn activate(&mut self, entry: usize) {
        match entry {
            FULLSCREEN => {
                let Ok(mut window) = self.windows.single_mut() else {
                    return;
                };
                let on = !fullscreen_now(&window);
                toggle_fullscreen(&mut window, self.profile.as_deref_mut(), on);
            }
            _ => self.screen.set(MenuScreen::Home),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_fullscreen(window: &mut Window, profile: Option<&mut Profile>, on: bool) {
    window.mode = window_mode(on);
    if let Some(profile) = profile {
        profile.data.fullscreen = on;
        profile.save();
    }
}

/// Nel browser lo chiede alla pagina (`Window.mode` non torna mai indietro quando il browser ne
/// esce da solo) e non si salva.
#[cfg(target_arch = "wasm32")]
fn toggle_fullscreen(_: &mut Window, _: Option<&mut Profile>, on: bool) {
    crate::web::set_fullscreen(on);
}

fn options_keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut selection: ResMut<ListSelection>,
    mut actions: OptionsActions,
) {
    if keys.just_pressed(KeyCode::Escape) {
        actions.activate(BACK);
        return;
    }
    step_selection(&mut selection, &keys, ENTRIES);
    let selection = *selection;
    let sideways = [
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ];
    let toggle = selection.0 == FULLSCREEN && keys.any_just_pressed(sideways);
    if confirmed(keys.bypass_change_detection()) || toggle {
        actions.activate(selection.0);
    }
}

/// "Sì" o "No" come lo schermo adesso (nel browser anche dopo un'uscita con Esc).
fn show_value(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut values: Query<&mut Text, With<FullscreenValue>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let wanted = if fullscreen_now(window) { "Sì" } else { "No" };
    for mut text in &mut values {
        if text.0 != wanted {
            text.0 = wanted.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::profile::ProfileData;

    #[test]
    fn saved_fullscreen_applies_only_to_real_runs_with_a_writable_profile() {
        let profile = |fullscreen: bool, writable: bool| Profile {
            data: ProfileData {
                fullscreen,
                ..default()
            },
            path: writable.then(|| "profile.ron".into()),
        };
        let normal = LaunchOptions::default();
        assert!(starts_fullscreen(&profile(true, true), Some(&normal)));
        assert!(starts_fullscreen(&profile(true, true), None));
        assert!(!starts_fullscreen(&profile(false, true), Some(&normal)));
        // autoplay, --god, --time-scale: profilo in sola lettura
        assert!(!starts_fullscreen(&profile(true, false), Some(&normal)));
        let screenshot = LaunchOptions {
            screenshot: Some("s.png".into()),
            ..default()
        };
        assert!(!starts_fullscreen(&profile(true, true), Some(&screenshot)));
        let timed = LaunchOptions {
            exit_after_secs: Some(3.0),
            ..default()
        };
        assert!(!starts_fullscreen(&profile(true, true), Some(&timed)));
    }

    #[test]
    fn profiles_saved_before_the_option_still_load() {
        let old: ProfileData = ron::from_str("(version: 1, companions: [])").unwrap();
        assert!(!old.fullscreen);
        assert_eq!(
            window_mode(true),
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        );
    }
}
