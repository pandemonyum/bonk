//! Righe dell'audio nelle Opzioni (AUDIO-2, progetto § 3.10): "Volume generale", "Musica",
//! "Effetti" e "Ambiente" con una barra di 10 tacche fatta di nodi UI (nessun glifo nuovo, D-044;
//! misure pari, D-045) e "Audio fuori dalla finestra" con Sì/No. Sinistra e destra (A/D) cambiano
//! la tacca della riga evidenziata, Invio passa alla tacca dopo (dopo l'ultima riparte da 0), un
//! clic su una tacca la imposta (sulla prima, quando è l'unica accesa, la spegne). Le azioni
//! (impostazioni, profilo salvato a ogni cambio, `ui.tick` e `ui.toggle`) stanno in
//! `options_menu::OptionsActions`; qui le righe, le regole delle tacche e la loro vista.

use bevy::picking::events::PointerClick;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::fonts;
use super::menu::{GOLD, PANEL};
use super::menu_list::{EDGE, ListButton, ListSelection, TEXT, spawn_list_button};
use super::options_menu::OptionsActions;
use crate::audio::settings::{AudioSettings, Bus, STEPS, Slider};
use crate::profile::Profile;

/// Cue di un cambio di tacca.
pub(super) const TICK: &str = "ui.tick";
/// Cue di un interruttore Sì/No.
pub(super) const TOGGLE: &str = "ui.toggle";

/// Tacca: larghezza, spazio dopo, altezza della prima e crescita per tacca (unità pari, D-045).
const NOTCH_WIDTH: f32 = 10.0;
const NOTCH_GAP: f32 = 4.0;
const NOTCH_LOW: f32 = 10.0;
const NOTCH_RISE: f32 = 2.0;
const NOTCH_BORDER: f32 = 2.0;
/// Altezza della barra, quella dell'ultima tacca: 28, così nei 40 interni della riga resta 6
/// sopra e sotto e la barra sta su unità pari (un texel intero, D-045).
const BAR_HEIGHT: f32 = NOTCH_LOW + NOTCH_RISE * (STEPS as f32 - 1.0);

pub(super) fn label(slider: Slider) -> &'static str {
    match slider {
        Slider::Master => "Volume generale",
        Slider::Bus(Bus::Music) => "Musica",
        Slider::Bus(Bus::Sfx) => "Effetti",
        Slider::Bus(Bus::Ambience) => "Ambiente",
    }
}

/// Invio: una tacca in più; dopo l'ultima si riparte da 0.
pub fn next_step(steps: u8) -> u8 {
    if steps >= STEPS { 0 } else { steps + 1 }
}

/// Sinistra o destra: una tacca in meno o in più, entro 0 e [`STEPS`].
pub fn nudged(steps: u8, right: bool) -> u8 {
    if right {
        steps.saturating_add(1).min(STEPS)
    } else {
        steps.min(STEPS).saturating_sub(1)
    }
}

/// Clic sulla tacca `notch` (da 1 a [`STEPS`]): quella. Sulla prima quando è l'unica accesa: 0,
/// così il silenzio si raggiunge anche con il mouse.
pub fn clicked(steps: u8, notch: u8) -> u8 {
    if notch == 1 && steps == 1 {
        0
    } else {
        notch.min(STEPS)
    }
}

/// Le impostazioni adesso: la risorsa dell'audio, o senza l'audio quelle del profilo.
pub(super) fn current(
    settings: Option<&AudioSettings>,
    profile: Option<&Profile>,
) -> AudioSettings {
    settings
        .cloned()
        .or_else(|| profile.map(|p| p.data.audio.clone()))
        .unwrap_or_default()
}

/// Il riquadro cliccabile di una tacca (alto quanto la barra, spazio compreso).
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct NotchCell {
    pub(super) slider: Slider,
    pub(super) step: u8,
}

/// La tacca disegnata, accesa se il cursore arriva almeno a `step`.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct NotchFill {
    pub(super) row: usize,
    pub(super) slider: Slider,
    pub(super) step: u8,
}

/// Il "Sì"/"No" accanto ad "Audio fuori dalla finestra".
#[derive(Component)]
pub(super) struct UnfocusedValue;

/// Riga `index` del cursore `slider`: etichetta e barra a destra.
pub(super) fn spawn_slider(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
    slider: Slider,
) -> Entity {
    let row = spawn_list_button(commands, assets, index, None, label(slider));
    let bar = commands
        .spawn((
            Name::new(format!("Barra {}", label(slider))),
            Node {
                margin: UiRect::left(Val::Auto),
                height: px(BAR_HEIGHT),
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    for step in 1..=STEPS {
        let gap = if step < STEPS { NOTCH_GAP } else { 0.0 };
        let height = NOTCH_LOW + NOTCH_RISE * f32::from(step - 1);
        let fill = commands
            .spawn((
                NotchFill {
                    row: index,
                    slider,
                    step,
                },
                Node {
                    width: px(NOTCH_WIDTH),
                    height: px(height),
                    border: UiRect::all(px(NOTCH_BORDER)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
            ))
            .id();
        let cell = commands
            .spawn((
                NotchCell { slider, step },
                Node {
                    width: px(NOTCH_WIDTH + gap),
                    height: px(BAR_HEIGHT),
                    align_items: AlignItems::FlexEnd,
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .add_child(fill)
            .observe(click_notch)
            .id();
        commands.entity(bar).add_child(cell);
    }
    commands.entity(row).add_child(bar);
    row
}

/// Riga `index` di "Audio fuori dalla finestra", con il Sì/No a destra.
pub(super) fn spawn_unfocused(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
) -> Entity {
    let row = spawn_list_button(commands, assets, index, None, "Audio fuori dalla finestra");
    let value = commands
        .spawn((
            UnfocusedValue,
            Text::new(""),
            fonts::ui(22.0),
            TextColor(TEXT),
            Node {
                margin: UiRect::left(Val::Auto),
                ..default()
            },
        ))
        .id();
    commands.entity(row).add_child(value);
    row
}

/// Un clic su una tacca: con il tasto sinistro la imposta, con gli altri nulla. Non arriva mai al
/// pulsante della riga, che passerebbe alla tacca dopo.
fn click_notch(mut click: On<PointerClick>, cells: Query<&NotchCell>, mut actions: OptionsActions) {
    let Ok(cell) = cells.get(click.entity) else {
        return;
    };
    // nessun clic sulla barra arriva alla riga: il pulsante tratterebbe anche il destro come
    // Invio (da 10 a 0, il silenzio)
    click.propagate(false);
    if click.button == PointerButton::Primary {
        actions.click_notch(cell.slider, cell.step);
    }
}

/// Clic (o `Activate`) su una riga dell'audio o su "Indietro": la voce del suo indice.
pub(super) fn activate_row(
    activate: On<Activate>,
    rows: Query<&ListButton>,
    mut actions: OptionsActions,
) {
    if let Ok(row) = rows.get(activate.entity) {
        actions.activate(row.0);
    }
}

/// Colori delle tacche: accese in oro sulla riga evidenziata, chiare sulle altre, spente scure.
pub(super) fn show_sliders(
    settings: Option<Res<AudioSettings>>,
    profile: Option<Res<Profile>>,
    selection: Res<ListSelection>,
    mut fills: Query<(&NotchFill, &mut BackgroundColor, &mut BorderColor)>,
) {
    let now = current(settings.as_deref(), profile.as_deref());
    for (notch, mut fill, mut border) in &mut fills {
        let lit = now.slider(notch.slider) >= notch.step;
        let (inside, edge) = match (lit, notch.row == selection.0) {
            (true, true) => (GOLD, GOLD),
            (true, false) => (TEXT, TEXT),
            (false, _) => (PANEL, EDGE),
        };
        if fill.0 != inside {
            fill.0 = inside;
            *border = BorderColor::all(edge);
        }
    }
}

/// "Sì" o "No" come l'audio fuori dalla finestra adesso.
pub(super) fn show_unfocused(
    settings: Option<Res<AudioSettings>>,
    profile: Option<Res<Profile>>,
    mut values: Query<&mut Text, With<UnfocusedValue>>,
) {
    let on = current(settings.as_deref(), profile.as_deref()).play_unfocused;
    let wanted = if on { "Sì" } else { "No" };
    for mut text in &mut values {
        if text.0 != wanted {
            text.0 = wanted.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_goes_to_the_next_step_and_wraps_after_the_last() {
        assert_eq!(next_step(0), 1);
        assert_eq!(next_step(9), 10);
        assert_eq!(next_step(STEPS), 0);
        let mut steps = 4;
        for _ in 0..=STEPS {
            steps = next_step(steps);
        }
        assert_eq!(steps, 4, "un giro completo torna dov'era");
    }

    #[test]
    fn left_and_right_stop_at_the_ends() {
        assert_eq!(nudged(5, true), 6);
        assert_eq!(nudged(5, false), 4);
        assert_eq!(nudged(STEPS, true), STEPS);
        assert_eq!(nudged(0, false), 0);
        assert_eq!(nudged(STEPS + 3, false), STEPS - 1);
    }

    #[test]
    fn a_click_sets_the_step_and_the_first_one_alone_turns_off() {
        assert_eq!(clicked(8, 3), 3);
        assert_eq!(clicked(0, 1), 1);
        assert_eq!(clicked(5, 1), 1);
        assert_eq!(clicked(1, 1), 0);
        assert_eq!(clicked(2, STEPS), STEPS);
    }

    #[test]
    fn the_bar_keeps_even_measures() {
        for value in [
            NOTCH_WIDTH,
            NOTCH_GAP,
            NOTCH_LOW,
            NOTCH_RISE,
            NOTCH_BORDER,
            BAR_HEIGHT,
        ] {
            assert_eq!(value % 2.0, 0.0, "{value}");
        }
        // la tacca più bassa ha un interno oltre il bordo
        const { assert!(NOTCH_LOW > 2.0 * NOTCH_BORDER) };
        let labels = Slider::ALL.map(label);
        for text in labels
            .iter()
            .chain(&["Audio fuori dalla finestra", "Sì", "No"])
        {
            assert!(text.chars().all(crate::data::ui_drawable), "{text}");
        }
    }
}
