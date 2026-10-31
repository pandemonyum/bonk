//! Tasti della selezione: frecce o WASD nella griglia (destra e sinistra un compagno, su e giù
//! una riga), 1–9 diretti, Invio o spazio per partire (solo con un compagno disponibile), Esc o
//! Backspace per tornare alla pagina iniziale. T apre i talenti (`talents_menu`).

use bevy::prelude::*;

use super::cards::COLUMNS;
use super::{MenuChoice, StartRun};
use crate::data::GameConfig;
use crate::presentation::home::MenuScreen;
use crate::presentation::menu_list::confirmed;

/// Un passo nella griglia dei compagni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStep {
    Left,
    Right,
    Up,
    Down,
}

/// La carta dopo un passo da `index` in una griglia di `count` carte su `columns` colonne:
/// destra e sinistra scorrono tutta la lista in giro, su e giù cambiano riga se la carta c'è.
pub fn grid_step(index: usize, count: usize, columns: usize, step: GridStep) -> usize {
    if count == 0 {
        return 0;
    }
    let index = index.min(count - 1);
    let columns = columns.max(1);
    match step {
        GridStep::Left => (index + count - 1) % count,
        GridStep::Right => (index + 1) % count,
        GridStep::Up => index.checked_sub(columns).unwrap_or(index),
        GridStep::Down => Some(index + columns)
            .filter(|&next| next < count)
            .unwrap_or(index),
    }
}

const DIGITS: [[KeyCode; 2]; 9] = [
    [KeyCode::Digit1, KeyCode::Numpad1],
    [KeyCode::Digit2, KeyCode::Numpad2],
    [KeyCode::Digit3, KeyCode::Numpad3],
    [KeyCode::Digit4, KeyCode::Numpad4],
    [KeyCode::Digit5, KeyCode::Numpad5],
    [KeyCode::Digit6, KeyCode::Numpad6],
    [KeyCode::Digit7, KeyCode::Numpad7],
    [KeyCode::Digit8, KeyCode::Numpad8],
    [KeyCode::Digit9, KeyCode::Numpad9],
];

const STEPS: [([KeyCode; 2], GridStep); 4] = [
    ([KeyCode::ArrowLeft, KeyCode::KeyA], GridStep::Left),
    ([KeyCode::ArrowRight, KeyCode::KeyD], GridStep::Right),
    ([KeyCode::ArrowUp, KeyCode::KeyW], GridStep::Up),
    ([KeyCode::ArrowDown, KeyCode::KeyS], GridStep::Down),
];

pub(super) fn select_keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut choice: ResMut<MenuChoice>,
    mut start: StartRun,
    mut screen: ResMut<NextState<MenuScreen>>,
) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) {
        screen.set(MenuScreen::Home);
        return;
    }
    let ids: Vec<&str> = config.companions.iter().map(|c| c.id.as_str()).collect();
    if ids.is_empty() {
        return;
    }
    let mut index = ids.iter().position(|id| *id == choice.0).unwrap_or(0);
    if let Some(slot) = DIGITS.iter().position(|pair| keys.any_just_pressed(*pair))
        && slot < ids.len()
    {
        index = slot;
    }
    for (pair, step) in STEPS {
        if keys.any_just_pressed(pair) {
            index = grid_step(index, ids.len(), COLUMNS, step);
        }
    }
    if choice.0 != ids[index] {
        choice.0 = ids[index].to_string();
    }
    // consumati: lo stesso Invio non arriva anche ad altri sistemi del menu
    if confirmed(keys.bypass_change_detection()) {
        let _ = start.go(&choice.0);
    }
}
