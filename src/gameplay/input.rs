//! Comandi del giocatore (solo client): tastiera → `MoveIntent`, pausa, pausa su perdita
//! focus, F5 per ricominciare la run, tasti 1/2/3 nel draft.

use bevy::prelude::*;
use bevy::window::WindowFocused;

use super::player::{MoveIntent, direction_from_keys};
use super::upgrades::ChooseUpgrade;
use crate::LaunchOptions;
use crate::app::{AppState, RunPhase, restart_run};

pub struct PlayerControlsPlugin;

impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                keyboard_intent.run_if(not(autoplay_enabled)),
                toggle_pause,
                restart_on_f5.run_if(in_state(AppState::InRun)),
                pause_on_focus_lost.run_if(not(autoplay_enabled)),
                choose_card_with_keys.run_if(in_state(RunPhase::LevelUp)),
            ),
        );
    }
}

/// F5 in qualunque momento della run; da sconfitti anche Invio o Spazio.
fn restart_on_f5(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Option<Res<State<RunPhase>>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let defeated = phase.is_some_and(|p| *p.get() == RunPhase::Defeated);
    let confirm = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]);
    if keys.just_pressed(KeyCode::F5) || (defeated && confirm) {
        restart_run(&mut next);
    }
}

fn autoplay_enabled(options: Res<LaunchOptions>) -> bool {
    options.autoplay
}

fn keyboard_intent(keys: Res<ButtonInput<KeyCode>>, mut intent: ResMut<MoveIntent>) {
    let pressed = |a: KeyCode, b: KeyCode| keys.pressed(a) || keys.pressed(b);
    intent.0 = direction_from_keys(
        pressed(KeyCode::KeyW, KeyCode::ArrowUp),
        pressed(KeyCode::KeyS, KeyCode::ArrowDown),
        pressed(KeyCode::KeyA, KeyCode::ArrowLeft),
        pressed(KeyCode::KeyD, KeyCode::ArrowRight),
    );
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Option<Res<State<RunPhase>>>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let Some(phase) = phase else { return };
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP) {
        match phase.get() {
            RunPhase::Playing => next.set(RunPhase::Paused),
            RunPhase::Paused => next.set(RunPhase::Playing),
            // nel draft si sceglie una carta; da sconfitti si può solo ricominciare
            RunPhase::LevelUp | RunPhase::Defeated => {}
        }
    }
}

/// Nel draft: 1, 2, 3 (anche dal tastierino) scelgono la carta in quella posizione.
fn choose_card_with_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut choose: MessageWriter<ChooseUpgrade>,
) {
    const KEYS: [[KeyCode; 2]; 3] = [
        [KeyCode::Digit1, KeyCode::Numpad1],
        [KeyCode::Digit2, KeyCode::Numpad2],
        [KeyCode::Digit3, KeyCode::Numpad3],
    ];
    if let Some(slot) = KEYS.iter().position(|pair| keys.any_just_pressed(*pair)) {
        choose.write(ChooseUpgrade(slot));
    }
}

/// Browser o finestra in secondo piano: la run si mette in pausa da sola.
fn pause_on_focus_lost(
    mut focus: MessageReader<WindowFocused>,
    phase: Option<Res<State<RunPhase>>>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let lost = focus.read().any(|event| !event.focused);
    if lost && phase.is_some_and(|p| *p.get() == RunPhase::Playing) {
        next.set(RunPhase::Paused);
    }
}
