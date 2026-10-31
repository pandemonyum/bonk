//! Opzioni (dalla pagina iniziale, `MenuScreen::Options`): schermo intero, animazioni della
//! pagina iniziale (`home_ambient`, salvate nel profilo anche sul web) e audio (volumi e audio
//! fuori dalla finestra, `options_audio`). W/S o frecce per la voce, Invio, spazio, A/D o clic
//! per cambiarla, Esc o "Indietro" per tornare. Ogni cambio si salva subito nel profilo e vale
//! anche dall'avvio dopo, tranne lo schermo intero nel browser: lì vale per la visita (si può
//! chiedere solo subito dopo un clic o un tasto) e il valore mostrato è quello vero della pagina,
//! che ne esce anche da sola con Esc. Un cambio di tacca chiede `ui.tick`, un Sì/No `ui.toggle`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use super::fonts;
use super::home::MenuScreen;
use super::home_ambient::HomeAmbient;
use super::menu::GOLD;
use super::menu_list::{
    ListKeys, ListSelection, TEXT, confirmed, panel_node, spawn_list_button, spawn_panel,
    step_selection,
};
use super::options_audio::{
    TICK, TOGGLE, activate_row, clicked, current, next_step, nudged, show_sliders, show_unfocused,
    spawn_slider, spawn_unfocused,
};
use super::talents_menu::{panel_closed, talent_keys};
use crate::audio::{AudioManager, AudioSettings, Slider};
use crate::options::LaunchOptions;
use crate::profile::Profile;

const FULLSCREEN: usize = 0;
const ANIMATIONS: usize = 1;
/// I quattro cursori dell'audio, nell'ordine di `Slider::ALL`.
const FIRST_SLIDER: usize = 2;
const UNFOCUSED: usize = FIRST_SLIDER + Slider::ALL.len();
const BACK: usize = UNFOCUSED + 1;
const ENTRIES: usize = BACK + 1;
/// Pannello largo quanto serve ad "Audio fuori dalla finestra" e al suo Sì/No; righe più vicine
/// (6 invece di 8) perché otto voci stiano larghe anche in 960×540 unità (1920×1080, D-045).
const PANEL: (f32, f32) = (480.0, 6.0);

/// Il cursore della voce `entry`, se è una riga del volume.
fn slider_at(entry: usize) -> Option<Slider> {
    Slider::ALL.get(entry.checked_sub(FIRST_SLIDER)?).copied()
}

/// Il "Sì"/"No" accanto a "Schermo intero".
#[derive(Component)]
struct FullscreenValue;

/// Il "Sì"/"No" accanto ad "Animazioni".
#[derive(Component)]
struct AnimationsValue;

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
                    show_animations,
                    show_sliders,
                    show_unfocused,
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
    let node = Node {
        row_gap: px(PANEL.1),
        ..panel_node(px(PANEL.0))
    };
    let panel = spawn_panel(&mut commands, &assets, node);
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
    let animations = spawn_list_button(
        &mut commands,
        &assets,
        ANIMATIONS,
        Some("ui/menu_talents.png"),
        "Animazioni",
    );
    let animations_value = commands
        .spawn((
            AnimationsValue,
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
        .entity(animations)
        .add_child(animations_value)
        .observe(|_: On<Activate>, mut actions: OptionsActions| actions.activate(ANIMATIONS));
    let mut rows = Vec::new();
    for (k, slider) in Slider::ALL.into_iter().enumerate() {
        rows.push(spawn_slider(
            &mut commands,
            &assets,
            FIRST_SLIDER + k,
            slider,
        ));
    }
    rows.push(spawn_unfocused(&mut commands, &assets, UNFOCUSED));
    rows.push(spawn_list_button(
        &mut commands,
        &assets,
        BACK,
        Some("ui/menu_quit.png"),
        "Indietro",
    ));
    for &row in &rows {
        commands.entity(row).observe(activate_row);
    }
    commands
        .entity(panel)
        .add_children(&[title, fullscreen, animations])
        .add_children(&rows);
    commands.entity(root).add_child(panel);
}

/// Quello che le voci cambiano: finestra, profilo, animazioni, audio e schermata.
#[derive(SystemParam)]
pub(super) struct OptionsActions<'w, 's> {
    windows: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
    profile: Option<ResMut<'w, Profile>>,
    ambient: Option<ResMut<'w, HomeAmbient>>,
    settings: Option<ResMut<'w, AudioSettings>>,
    audio: AudioManager<'w>,
    screen: ResMut<'w, NextState<MenuScreen>>,
}

impl OptionsActions<'_, '_> {
    /// Invio, spazio o clic sulla voce `entry`.
    pub(super) fn activate(&mut self, entry: usize) {
        if let Some(slider) = slider_at(entry) {
            self.change_slider(slider, next_step);
            return;
        }
        match entry {
            FULLSCREEN => {
                let Ok(mut window) = self.windows.single_mut() else {
                    return;
                };
                let on = !fullscreen_now(&window);
                toggle_fullscreen(&mut window, self.profile.as_deref_mut(), on);
                self.audio.play(TOGGLE);
            }
            ANIMATIONS => {
                if let Some(ambient) = self.ambient.as_deref_mut() {
                    let on = !ambient.enabled;
                    set_animations(ambient, self.profile.as_deref_mut(), on);
                    self.audio.play(TOGGLE);
                }
            }
            UNFOCUSED => self.change_audio(TOGGLE, |s| s.play_unfocused = !s.play_unfocused),
            _ => self.screen.set(MenuScreen::Home),
        }
    }

    /// Sinistra o destra: una tacca sui volumi, il Sì/No sugli interruttori, nulla su
    /// "Indietro".
    fn sideways(&mut self, entry: usize, right: bool) {
        match slider_at(entry) {
            Some(slider) => self.change_slider(slider, |steps| nudged(steps, right)),
            None if entry != BACK => self.activate(entry),
            None => {}
        }
    }

    /// Un clic sulla tacca `notch` del cursore `slider`.
    pub(super) fn click_notch(&mut self, slider: Slider, notch: u8) {
        self.change_slider(slider, |steps| clicked(steps, notch));
    }

    fn change_slider(&mut self, slider: Slider, step: impl FnOnce(u8) -> u8) {
        self.change_audio(TICK, |s| {
            let steps = step(s.slider(slider));
            s.set_slider(slider, steps);
        });
    }

    /// Cambia le impostazioni dell'audio; se cambiano davvero le scrive nella risorsa e nel
    /// profilo (salvato subito, senza il muto della sessione) e chiede il suono `cue`.
    fn change_audio(&mut self, cue: &str, change: impl FnOnce(&mut AudioSettings)) {
        let now = current(self.settings.as_deref(), self.profile.as_deref());
        let mut next = now.clone();
        change(&mut next);
        if next == now {
            return;
        }
        if let Some(profile) = self.profile.as_deref_mut() {
            next.store(profile);
        }
        if let Some(settings) = self.settings.as_deref_mut() {
            *settings = next;
        }
        self.audio.play(cue);
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

/// Accende o spegne le animazioni della home e lo salva nel profilo (anche sul web).
fn set_animations(ambient: &mut HomeAmbient, profile: Option<&mut Profile>, on: bool) {
    ambient.enabled = on;
    if let Some(profile) = profile {
        profile.data.still_home = !on;
        profile.save();
    }
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
    let entry = selection.0;
    let left = keys.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keys.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    if confirmed(keys.bypass_change_detection()) {
        actions.activate(entry);
    } else if left != right {
        actions.sideways(entry, right);
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

/// "Sì" o "No" come le animazioni della home adesso.
fn show_animations(
    ambient: Option<Res<HomeAmbient>>,
    mut values: Query<&mut Text, With<AnimationsValue>>,
) {
    let on = ambient.is_some_and(|a| a.enabled);
    let wanted = if on { "Sì" } else { "No" };
    for mut text in &mut values {
        if text.0 != wanted {
            text.0 = wanted.to_string();
        }
    }
}

#[cfg(test)]
mod tests;
