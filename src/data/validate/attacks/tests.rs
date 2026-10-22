//! Test delle regole dei pattern di attacco e dei posti dei compagni. Gli attacchi si cercano
//! per pattern nei dati veri e si modificano solo nel campo provato.

use super::super::tests::{assert_error, embedded, errors_after};
use super::leaf_step;
use crate::data::GameConfig;
use crate::data::schema::{AttackPattern, OrbitData, PulseData, SlotData};
use crate::gameplay::companions::MAX_COMPANION_SLOTS;

fn pulse_mut(c: &mut GameConfig) -> &mut PulseData {
    c.attacks
        .iter_mut()
        .find_map(|a| match &mut a.pattern {
            AttackPattern::Pulse(p) => Some(p),
            _ => None,
        })
        .expect("un attacco a onda nei dati")
}

fn orbit_index(c: &GameConfig) -> usize {
    c.attacks
        .iter()
        .position(|a| a.orbit().is_some())
        .expect("un attacco a foglie nei dati")
}

fn orbit_mut(c: &mut GameConfig) -> &mut OrbitData {
    let i = orbit_index(c);
    match &mut c.attacks[i].pattern {
        AttackPattern::Orbit(o) => o,
        _ => unreachable!(),
    }
}

#[test]
fn pulse_parameters_are_checked() {
    let id = embedded()
        .attacks
        .iter()
        .find(|a| a.pulse().is_some())
        .unwrap()
        .id
        .clone();
    assert_error(
        |c| pulse_mut(c).slow = 0.0,
        &format!("attacks.ron: {id}.pattern.slow = 0"),
    );
    assert_error(|c| pulse_mut(c).slow = 1.5, "pattern.slow = 1.5");
    assert_error(
        |c| pulse_mut(c).speed = 0.0,
        "pattern.speed deve essere > 0",
    );
    // nessun rallentamento (1) è legittimo
    assert!(errors_after(|c| pulse_mut(c).slow = 1.0).is_empty());
}

#[test]
fn orbit_parameters_and_speed_are_checked() {
    let c = embedded();
    let i = orbit_index(&c);
    let id = c.attacks[i].id.clone();
    assert_error(
        |c| orbit_mut(c).count = 0,
        &format!("attacks.ron: {id}.pattern.count deve essere >= 1"),
    );
    assert_error(
        |c| orbit_mut(c).angular_speed_deg = 0.0,
        "angular_speed_deg = 0",
    );
    // foglie abbastanza veloci da saltare oltre un nemico in un passo
    let o = c.attacks[i].orbit().unwrap();
    let smallest = c
        .enemies
        .iter()
        .map(|e| e.hurtbox.radius)
        .fold(f32::INFINITY, f32::min);
    let limit = (o.radius + smallest) / leaf_step(1.0, c.attacks[i].range);
    assert_error(
        |c| orbit_mut(c).angular_speed_deg = limit * 1.5,
        "attraversano i nemici",
    );
    // anche girando all'indietro
    assert_error(
        |c| orbit_mut(c).angular_speed_deg = -limit * 1.5,
        "attraversano i nemici",
    );
}

#[test]
fn companion_slots_are_checked() {
    assert_error(
        |c| c.player.companion_slots.clear(),
        "player.ron: companion_slots ha 0 posti",
    );
    assert_error(
        |c| c.player.companion_slots = vec![SlotData { angle_deg: 0.0 }; MAX_COMPANION_SLOTS + 1],
        &format!("ha {} posti", MAX_COMPANION_SLOTS + 1),
    );
    assert_error(
        |c| c.player.companion_slots[0].angle_deg = f32::NAN,
        "companion_slots[0].angle_deg",
    );
}

#[test]
fn every_pattern_field_is_checked() {
    assert_error(
        |c| pulse_mut(c).slow_s = -1.0,
        "pattern.slow_s deve essere >= 0",
    );
    assert_error(
        |c| pulse_mut(c).center_height = -1.0,
        "pattern.center_height deve essere >= 0",
    );
    assert_error(
        |c| orbit_mut(c).radius = 0.0,
        "pattern.radius deve essere > 0",
    );
    assert_error(
        |c| orbit_mut(c).center_height = -1.0,
        "pattern.center_height deve essere >= 0",
    );
}
