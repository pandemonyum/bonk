//! Test dei limiti di onda e foglie con gli upgrade al massimo e delle carte "nuovo
//! compagno". Come per gli altri, id e numeri sono ricavati dai dati veri.

use super::super::tests::{assert_error, embedded, errors_after};
use crate::data::GameConfig;
use crate::data::schema::{AttackPattern, Effect, UpgradeData};
use crate::gameplay::SIM_HZ;
use crate::gameplay::attacks::MAX_PROJECTILES;

fn join_index(c: &GameConfig) -> usize {
    c.upgrades
        .iter()
        .position(|u| matches!(u.effect, Effect::JoinCompanion(_)))
        .expect("una carta nuovo compagno nei dati")
}

#[test]
fn companion_cards_are_checked() {
    let c = embedded();
    let join = join_index(&c);
    let id = c.upgrades[join].id.clone();
    assert_error(
        |c| c.upgrades[join].effect = Effect::JoinCompanion("ghost".into()),
        &format!("upgrades.ron: {id}.effect = \"ghost\": non esiste in companions.ron"),
    );
    assert_error(
        |c| c.upgrades[join].max_stack = 2,
        &format!("upgrades.ron: {id}.max_stack = 2: un compagno arriva una volta sola"),
    );
    assert_error(
        |c| {
            let copy = UpgradeData {
                id: "join_again".into(),
                ..c.upgrades[join].clone()
            };
            c.upgrades.push(copy);
        },
        "ha già un'altra carta",
    );
    assert_error(
        |c| c.player.companion_slots.truncate(1),
        "player.ron ha un solo posto",
    );
}

/// Gittata più ampia raggiungibile: tutte le carte `CompanionRangePct` positive al massimo.
fn widest_range(c: &GameConfig) -> f32 {
    1.0 + c
        .upgrades
        .iter()
        .filter_map(|u| match u.effect {
            Effect::CompanionRangePct(v) if v > 0.0 => Some(v * u.max_stack as f32),
            _ => None,
        })
        .sum::<f32>()
        / 100.0
}

fn orbit_attack_index(c: &GameConfig) -> usize {
    c.attacks
        .iter()
        .position(|a| a.orbit().is_some())
        .expect("un attacco a foglie nei dati")
}

/// Rilievo del code review P0.5: le foglie possono andare bene con l'orbita base e saltare i
/// nemici con l'orbita allargata dagli upgrade di gittata.
#[test]
fn leaves_are_checked_with_the_widest_orbit() {
    let c = embedded();
    let i = orbit_attack_index(&c);
    let (o, range) = (c.attacks[i].orbit().unwrap().clone(), c.attacks[i].range);
    let widest = widest_range(&c);
    assert!(widest > 1.0, "dati inattesi: nessuna carta di gittata");
    let smallest = c
        .enemies
        .iter()
        .map(|e| e.hurtbox.radius)
        .fold(f32::INFINITY, f32::min);
    // gradi/s a cui la foglia fa esattamente il passo massimo sull'orbita base
    let limit = (o.radius + smallest) * SIM_HZ as f32 / (range * 1f32.to_radians());
    let speed = limit / widest * 1.2; // sotto il limite con l'orbita base, sopra con la più larga
    let set = |c: &mut GameConfig| {
        if let AttackPattern::Orbit(o) = &mut c.attacks[i].pattern {
            o.angular_speed_deg = speed;
        }
    };
    let errors = errors_after(set);
    assert!(
        !errors
            .iter()
            .any(|e| e.starts_with("attacks.ron") && e.contains("attraversano")),
        "con l'orbita base deve andare bene: {errors:?}"
    );
    assert_error(set, "con tutti gli upgrade di gittata le foglie");
}

#[test]
fn leaves_count_towards_the_projectile_cap() {
    let c = embedded();
    let i = orbit_attack_index(&c);
    let id = c.attacks[i].id.clone();
    assert_error(
        |c| {
            if let AttackPattern::Orbit(o) = &mut c.attacks[i].pattern {
                o.count = MAX_PROJECTILES + 1;
            }
        },
        &format!("con tutti gli upgrade {id} arriva a"),
    );
}

/// Un'onda deve finire prima della successiva (rilievo del code review P0.5: un refuso nella
/// velocità faceva vivere decine di onde insieme).
#[test]
fn a_wave_ends_before_the_next_one() {
    let c = embedded();
    let i = c.attacks.iter().position(|a| a.pulse().is_some()).unwrap();
    let id = c.attacks[i].id.clone();
    assert_error(
        |c| {
            if let AttackPattern::Pulse(p) = &mut c.attacks[i].pattern {
                p.speed = 1.6;
            }
        },
        &format!("attacks.ron: {id}: con gli upgrade al massimo l'onda dura"),
    );
}
