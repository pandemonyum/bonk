//! Opzioni: schermo intero, animazioni e righe dell'audio, nel menu senza finestra
//! (`home::tests::menu_app`).

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
fn the_animations_switch_follows_the_profile() {
    let mut ambient = HomeAmbient::default();
    let mut profile = Profile::default();
    set_animations(&mut ambient, Some(&mut profile), false);
    assert!(!ambient.enabled && profile.data.still_home);
    set_animations(&mut ambient, Some(&mut profile), true);
    assert!(ambient.enabled && !profile.data.still_home);
    set_animations(&mut ambient, None, false);
    assert!(!ambient.enabled);
}

#[test]
fn profiles_saved_before_the_option_still_load() {
    let old: ProfileData = ron::from_str("(version: 1, companions: [])").unwrap();
    assert!(!old.fullscreen);
    assert!(!old.still_home, "le animazioni restano accese");
    assert_eq!(
        window_mode(true),
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    );
}

// --- Righe dell'audio (AUDIO-2) ---

use bevy::camera::{ManualTextureViewHandle, NormalizedRenderTarget};
use bevy::picking::backend::HitData;
use bevy::picking::events::{Pointer, PointerClick};
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::ui::Pressed;

use super::super::options_audio::NotchFill;
use crate::audio::settings::STEPS;
use crate::audio::{AudioCatalog, CueQueue};
use crate::presentation::home::tests::{go, menu_app, named, screen, tap};
use crate::presentation::menu_list::ListButton;

/// Le Opzioni con le impostazioni dell'audio (muto della sessione acceso, come con `--mute`) e
/// la coda dei suoni con il catalogo vero, senza mixer: le richieste restano in coda.
fn options_app() -> App {
    let mut app = menu_app(Profile::default());
    app.insert_resource(AudioCatalog::embedded())
        .init_resource::<CueQueue>()
        .insert_resource(AudioSettings {
            session_muted: true,
            ..default()
        });
    go(&mut app, MenuScreen::Options);
    app
}

fn select(app: &mut App, entry: usize) {
    app.world_mut().resource_mut::<ListSelection>().0 = entry;
}

/// I nomi dei suoni chiesti dall'ultima volta.
fn asked(app: &mut App) -> Vec<String> {
    let requests = app.world_mut().resource_mut::<CueQueue>().take();
    let catalog = app.world().resource::<AudioCatalog>();
    requests
        .iter()
        .map(|r| catalog.cue(r.cue).unwrap().name.clone())
        .collect()
}

fn settings(app: &App) -> AudioSettings {
    app.world().resource::<AudioSettings>().clone()
}

fn saved(app: &App) -> AudioSettings {
    app.world().resource::<Profile>().data.audio.clone()
}

#[test]
fn the_options_have_every_audio_row_before_back() {
    let mut app = options_app();
    assert_eq!(ENTRIES, 8);
    assert_eq!(slider_at(FIRST_SLIDER), Some(Slider::ALL[0]));
    assert_eq!(slider_at(UNFOCUSED), None);
    assert_eq!(slider_at(FULLSCREEN), None);
    let world = app.world_mut();
    let mut buttons = world.query::<&ListButton>();
    let mut indices: Vec<usize> = buttons.iter(world).map(|b| b.0).collect();
    indices.sort();
    assert_eq!(indices, (0..ENTRIES).collect::<Vec<_>>());
    for name in [
        "Barra Volume generale",
        "Barra Musica",
        "Barra Effetti",
        "Barra Ambiente",
    ] {
        assert_eq!(named(&mut app, name), 1);
    }
}

#[test]
fn keys_move_the_steps_save_them_and_ask_for_a_tick() {
    let mut app = options_app();
    select(&mut app, FIRST_SLIDER); // volume generale, 8
    tap(&mut app, KeyCode::ArrowRight);
    assert_eq!(settings(&app).master, 9);
    assert_eq!(saved(&app).master, 9, "nel profilo");
    assert!(
        !saved(&app).session_muted,
        "il muto della sessione non si salva"
    );
    assert!(settings(&app).session_muted, "e resta nella sessione");
    assert_eq!(asked(&mut app), ["ui.tick"]);
    tap(&mut app, KeyCode::KeyD);
    tap(&mut app, KeyCode::ArrowRight); // già all'ultima tacca
    assert_eq!(settings(&app).master, 10);
    assert_eq!(asked(&mut app), ["ui.tick"], "solo i cambi veri suonano");
    tap(&mut app, KeyCode::Enter); // dopo l'ultima si riparte da 0
    assert_eq!(settings(&app).master, 0);
    tap(&mut app, KeyCode::ArrowLeft);
    assert_eq!(settings(&app).master, 0);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(settings(&app).master, 1);
    assert_eq!(asked(&mut app), ["ui.tick", "ui.tick"]);
    select(&mut app, FIRST_SLIDER + 2); // effetti, 8
    tap(&mut app, KeyCode::KeyA);
    assert_eq!((settings(&app).sfx, saved(&app).sfx), (7, 7));
    assert_eq!(settings(&app).music, AudioSettings::default().music);
    assert_eq!(asked(&mut app), ["ui.tick"]);
    assert_eq!(screen(&app), MenuScreen::Options);
}

#[test]
fn the_unfocused_switch_and_the_old_switches_ask_for_a_toggle() {
    let mut app = options_app();
    select(&mut app, UNFOCUSED);
    tap(&mut app, KeyCode::Enter);
    assert!(settings(&app).play_unfocused && saved(&app).play_unfocused);
    tap(&mut app, KeyCode::ArrowLeft);
    assert!(!settings(&app).play_unfocused && !saved(&app).play_unfocused);
    assert_eq!(asked(&mut app), ["ui.toggle", "ui.toggle"]);
    // le animazioni (senza `HomeAmbient` non cambiano: nessun suono)
    app.init_resource::<HomeAmbient>();
    select(&mut app, ANIMATIONS);
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<Profile>().data.still_home);
    assert_eq!(asked(&mut app), ["ui.toggle"]);
    // "Indietro" con le frecce non fa nulla
    select(&mut app, BACK);
    tap(&mut app, KeyCode::ArrowRight);
    assert!(asked(&mut app).is_empty());
    assert_eq!(screen(&app), MenuScreen::Options);
}

/// La riga numero `index` e la tacca disegnata `step` della sua barra.
fn row_and_notch(app: &mut App, index: usize, step: u8) -> (Entity, Entity) {
    let world = app.world_mut();
    let mut rows = world.query::<(Entity, &ListButton)>();
    let row = rows.iter(world).find(|(_, b)| b.0 == index).unwrap().0;
    let mut fills = world.query::<(Entity, &NotchFill)>();
    let fill = fills
        .iter(world)
        .find(|(_, n)| n.row == index && n.step == step)
        .unwrap()
        .0;
    (row, fill)
}

/// Un clic vero del mouse sull'entità `target`, con la riga `row` premuta (come dopo il
/// `PointerPress`): l'evento di Bevy, con la sua propagazione ai genitori (`PointerTraversal`).
fn click(app: &mut App, row: Entity, target: Entity, button: PointerButton) {
    app.world_mut().entity_mut(row).insert(Pressed);
    let location = Location {
        target: NormalizedRenderTarget::TextureView(ManualTextureViewHandle(0)),
        position: Vec2::ZERO,
    };
    app.world_mut().trigger(PointerClick {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, location),
        button,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        duration: std::time::Duration::from_millis(80),
        count: 1,
    });
    app.update();
}

/// Il menu con il `Button` di Bevy vero: un clic che arriva alla riga diventa `Activate`.
fn clickable_options() -> App {
    let mut app = options_app();
    app.add_plugins(bevy::ui_widgets::ButtonPlugin);
    app
}

#[test]
fn a_left_click_on_a_notch_sets_it_with_one_tick_and_never_reaches_the_row() {
    let mut app = clickable_options();
    let music = FIRST_SLIDER + 1;
    let (row, fill) = row_and_notch(&mut app, music, 3);
    click(&mut app, row, fill, PointerButton::Primary);
    assert_eq!((settings(&app).music, saved(&app).music), (3, 3));
    assert_eq!(
        asked(&mut app),
        ["ui.tick"],
        "un clic, un suono, nessun Activate"
    );
    // sul riquadro della tacca (lo spazio sopra la tacca bassa) vale lo stesso
    let cell = app.world().get::<ChildOf>(fill).unwrap().parent();
    let (_, first) = row_and_notch(&mut app, music, 1);
    let first_cell = app.world().get::<ChildOf>(first).unwrap().parent();
    click(&mut app, row, first_cell, PointerButton::Primary);
    assert_eq!(settings(&app).music, 1);
    click(&mut app, row, first, PointerButton::Primary);
    assert_eq!(settings(&app).music, 0, "la prima, sola accesa, si spegne");
    assert_eq!(asked(&mut app), ["ui.tick", "ui.tick"]);
    click(&mut app, row, cell, PointerButton::Primary);
    assert_eq!(settings(&app).music, 3);
    assert_eq!(asked(&mut app), ["ui.tick"]);

    // controllo: un clic sulla riga fuori dalla barra arriva al `Button` ed è un Invio
    click(&mut app, row, row, PointerButton::Primary);
    assert_eq!(settings(&app).music, 4);
    assert_eq!(asked(&mut app), ["ui.tick"]);
}

#[test]
fn other_buttons_on_a_notch_do_nothing() {
    let mut app = clickable_options();
    let master = FIRST_SLIDER;
    let before = settings(&app);
    for button in [PointerButton::Secondary, PointerButton::Middle] {
        for step in [1, 5, STEPS] {
            let (row, fill) = row_and_notch(&mut app, master, step);
            click(&mut app, row, fill, button);
        }
    }
    // senza fermare la propagazione il `Button` lo tratterebbe come Invio: da 8 a 9, poi 10, 0…
    assert_eq!(settings(&app), before);
    assert_eq!(saved(&app), AudioSettings::default());
    assert!(asked(&mut app).is_empty());
}

/// Misure pari (D-045) lette dai nodi creati, non dalle costanti: la barra centrata nella riga
/// e ogni tacca appoggiata in fondo alla barra cadono su unità pari.
#[test]
fn the_bar_sits_on_even_units_inside_its_row() {
    fn px(value: Val) -> f32 {
        match value {
            Val::Px(px) => px,
            other => panic!("misura non in px: {other:?}"),
        }
    }
    let mut app = options_app();
    for (k, slider) in Slider::ALL.into_iter().enumerate() {
        let (row, fill) = row_and_notch(&mut app, FIRST_SLIDER + k, 1);
        let world = app.world();
        let cell = world.get::<ChildOf>(fill).unwrap().parent();
        let bar = world.get::<ChildOf>(cell).unwrap().parent();
        assert_eq!(world.get::<ChildOf>(bar).unwrap().parent(), row);
        let row_node = world.get::<Node>(row).unwrap();
        let bar_node = world.get::<Node>(bar).unwrap();
        assert_eq!(row_node.align_items, AlignItems::Center, "{slider:?}");
        let inner = px(row_node.height)
            - px(row_node.border.top)
            - px(row_node.border.bottom)
            - px(row_node.padding.top)
            - px(row_node.padding.bottom);
        let bar_height = px(bar_node.height);
        let gap = inner - bar_height - px(bar_node.margin.top) - px(bar_node.margin.bottom);
        let top = px(row_node.border.top) + px(row_node.padding.top) + gap / 2.0;
        assert_eq!(
            top % 2.0,
            0.0,
            "{slider:?}: barra a {top} unità dal bordo della riga"
        );
        assert!(gap >= 0.0);
        for step in 1..=STEPS {
            let (_, fill) = row_and_notch(&mut app, FIRST_SLIDER + k, step);
            let height = px(app.world().get::<Node>(fill).unwrap().height);
            assert_eq!((bar_height - height) % 2.0, 0.0, "{slider:?} tacca {step}");
        }
    }
}

#[test]
fn without_the_audio_the_rows_still_work_on_the_profile() {
    let mut app = menu_app(Profile::default());
    go(&mut app, MenuScreen::Options);
    select(&mut app, FIRST_SLIDER + 3); // ambiente
    tap(&mut app, KeyCode::ArrowRight);
    assert_eq!(saved(&app).ambience, AudioSettings::default().ambience + 1);
}
