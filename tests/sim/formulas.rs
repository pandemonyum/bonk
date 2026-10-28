//! P0.7, formule misurate nella simulazione: danno per colpo e cadenza dei quattro pattern di
//! attacco con gli stack di danno e di cadenza, livelli dall'XP raccolta. I valori attesi si
//! calcolano dai dati (`GameConfig`) con le formule del GDD (percentuali sommate sul valore base,
//! XP per livello `base + per_level × (livello − 1)`), non dai componenti che il codice ha già
//! calcolato: il test misura colpi e HP tolti a un bersaglio fermo.

use std::f32::consts::TAU;

use crate::common::{
    app_starting_with, companion_with, config, enemy_at, player_position, progression, run_steps,
    xp_app,
};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::{AttackData, Effect};
use pixel_monster_survivor::gameplay::combat::{Health, Hurtbox, RunStats};
use pixel_monster_survivor::gameplay::kinematics::{Position, PreviousPosition, body};
use pixel_monster_survivor::gameplay::progression::{XpGem, xp_to_next};
use pixel_monster_survivor::gameplay::upgrades::UpgradeStacks;

/// Secondi di misura: abbastanza colpi perché l'errore di ±1 colpo ai bordi sia piccolo.
const WINDOW_S: f32 = 20.0;
/// HP del bersaglio: non muore e in f32 i decimali del danno restano esatti al centesimo.
const DUMMY_HP: f32 = 1.0e5;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pattern {
    Projectile,
    Pulse,
    Orbit,
    Chain,
}

/// Indice dell'upgrade con l'effetto cercato e valore per stack.
fn upgrade_value(config: &GameConfig, wanted: impl Fn(&Effect) -> Option<f32>) -> (usize, f32) {
    config
        .upgrades
        .iter()
        .enumerate()
        .find_map(|(i, u)| wanted(&u.effect).map(|v| (i, v)))
        .expect("i dati hanno l'upgrade")
}

fn damage_upgrade(config: &GameConfig) -> (usize, f32) {
    upgrade_value(config, |e| match e {
        Effect::CompanionDamagePct(v) => Some(*v),
        _ => None,
    })
}

fn rate_upgrade(config: &GameConfig) -> (usize, f32) {
    upgrade_value(config, |e| match e {
        Effect::CompanionIntervalPct(v) => Some(*v),
        _ => None,
    })
}

/// Valore base con `stacks` stack da `per_stack` punti percentuali (GDD: si sommano).
fn with_pct(base: f32, per_stack: f32, stacks: u32) -> f32 {
    base * (1.0 + per_stack * stacks as f32 / 100.0)
}

fn uses(pattern: Pattern) -> impl Fn(&AttackData) -> bool {
    move |a| match pattern {
        Pattern::Projectile => a.projectile().is_some(),
        Pattern::Pulse => a.pulse().is_some(),
        Pattern::Orbit => a.orbit().is_some(),
        Pattern::Chain => a.chain().is_some(),
    }
}

struct Measured {
    hits: u32,
    damage: f32,
}

/// Run con il solo compagno del `pattern`, gli stack dati e un bersaglio immobile messo dove
/// l'attacco lo raggiunge sempre: a metà gittata per i proiettili, a metà raggio per l'onda,
/// sull'orbita per le foglie. Misura colpi e danno nei `WINDOW_S` secondi.
fn against_dummy(pattern: Pattern, damage_stacks: u32, rate_stacks: u32) -> (Measured, AttackData) {
    let data_config = GameConfig::embedded();
    let id = companion_with(&data_config, uses(pattern));
    let mut app = app_starting_with(&id, |_| {});
    let config = config(&app);
    let attack = config
        .companion_attack(config.companion(&id).unwrap())
        .clone();
    {
        let mut stacks = app.world_mut().resource_mut::<UpgradeStacks>();
        stacks.0[damage_upgrade(&config).0] = damage_stacks;
        stacks.0[rate_upgrade(&config).0] = rate_stacks;
    }
    app.update(); // ricalcolo degli attacchi dagli stack

    let feet = player_position(&mut app);
    let (center, distance) = match pattern {
        Pattern::Projectile | Pattern::Chain => (feet, attack.range * 0.5),
        Pattern::Pulse => {
            let p = attack.pulse().unwrap();
            (feet + Vec2::Y * p.center_height, attack.range * 0.5)
        }
        Pattern::Orbit => {
            let o = attack.orbit().unwrap();
            (feet + Vec2::Y * o.center_height, attack.range)
        }
    };
    let dummy = enemy_at(&mut app, feet, DUMMY_HP, 9_999);
    // il centro della hurtbox (non i piedi) va nel punto voluto
    let offset = app.world().get::<Hurtbox>(dummy).unwrap().center_offset;
    let at = center + Vec2::X * distance - Vec2::Y * offset;
    app.world_mut().get_mut::<Position>(dummy).unwrap().0 = at;
    app.world_mut()
        .get_mut::<PreviousPosition>(dummy)
        .unwrap()
        .0 = at;

    let before = app.world().resource::<RunStats>().hits;
    run_steps(&mut app, (WINDOW_S * 60.0) as usize);
    let hits = app.world().resource::<RunStats>().hits - before;
    let hp = app.world().get::<Health>(dummy).unwrap().current;
    (
        Measured {
            hits,
            damage: DUMMY_HP - hp,
        },
        attack,
    )
}

/// Ogni colpo toglie esattamente il danno base con gli stack di danno, e i colpi arrivano alla
/// cadenza base con gli stack di cadenza: proiettili e onda una volta per intervallo (±1 colpo
/// ai bordi della misura), foglie al massimo una volta per intervallo e almeno a ogni passaggio
/// utile di una foglia.
#[test]
fn damage_per_hit_and_cadence_follow_the_stacks_for_every_pattern() {
    let config = GameConfig::embedded();
    let (_, damage_pct) = damage_upgrade(&config);
    let (_, rate_pct) = rate_upgrade(&config);
    for pattern in [
        Pattern::Projectile,
        Pattern::Pulse,
        Pattern::Orbit,
        Pattern::Chain,
    ] {
        for (damage_stacks, rate_stacks) in [(0, 0), (2, 0), (0, 2), (3, 3)] {
            let (m, attack) = against_dummy(pattern, damage_stacks, rate_stacks);
            let case = format!("{pattern:?} danno×{damage_stacks} cadenza×{rate_stacks}");
            let per_hit = with_pct(attack.damage, damage_pct, damage_stacks);
            let interval = with_pct(attack.interval_s, rate_pct, rate_stacks);
            assert!(m.hits > 0, "{case}: nessun colpo");
            let expected = per_hit * m.hits as f32;
            assert!(
                (m.damage - expected).abs() < 0.01 * m.hits as f32,
                "{case}: {} danni per {} colpi, attesi {per_hit} l'uno",
                m.damage,
                m.hits
            );
            let hits = m.hits as f32;
            match pattern {
                Pattern::Projectile | Pattern::Pulse | Pattern::Chain => {
                    // il fulmine è istantaneo: colpisce subito e poi a ogni intervallo; gli
                    // altri devono arrivare (volo, fronte), quindi il primo e l'ultimo cadono ai
                    // bordi della misura
                    let instant = if pattern == Pattern::Chain { 1.0 } else { 0.0 };
                    let wanted = instant + WINDOW_S / interval;
                    assert!(
                        (hits - wanted).abs() <= 1.0,
                        "{case}: {hits} colpi in {WINDOW_S} s, attesi {wanted:.1} (intervallo {interval} s)"
                    );
                }
                Pattern::Orbit => {
                    let o = attack.orbit().unwrap();
                    let pass = TAU / (o.count as f32 * o.angular_speed_deg.to_radians());
                    let slowest = (interval / pass).ceil() * pass;
                    assert!(
                        hits <= WINDOW_S / interval + 1.0,
                        "{case}: {hits} colpi, più di uno ogni {interval} s"
                    );
                    assert!(
                        hits >= WINDOW_S / slowest - 1.0,
                        "{case}: {hits} colpi, meno di uno ogni {slowest:.2} s"
                    );
                }
            }
        }
    }
}

/// XP totale necessaria per arrivare al livello `level` partendo da 1.
fn xp_for_level(config: &GameConfig, level: u32) -> u32 {
    (1..level)
        .map(|l| xp_to_next(&config.progression.xp_to_level, l))
        .sum()
}

/// Una gemma da `total` XP porta al livello più alto la cui XP cumulata è ≤ `total`, con il
/// resto verso il successivo; la curva è quella del GDD, ricavata qui dai dati.
#[test]
fn gems_give_the_levels_of_the_xp_curve() {
    let config = GameConfig::embedded();
    let curve = &config.progression.xp_to_level;
    for level in 1..30 {
        let formula = (curve.base + curve.per_level * (level - 1) as f32)
            .ceil()
            .max(1.0) as u32;
        assert_eq!(xp_to_next(curve, level), formula, "livello {level}");
    }
    for (level, extra) in [(2, 0), (5, 0), (5, 1), (10, 0), (10, 7), (20, 3)] {
        let total = xp_for_level(&config, level) + extra;
        let mut app = xp_app();
        let at = player_position(&mut app);
        app.world_mut().spawn((XpGem { value: total }, body(at)));
        run_steps(&mut app, 2);
        let p = progression(&app);
        assert_eq!(p.level, level, "{total} XP");
        assert_eq!(p.xp, extra, "{total} XP: resto");
        assert_eq!(p.xp_to_next, xp_to_next(curve, level));
        assert_eq!(p.total_xp, total);
        assert_eq!(p.pending_levels, level - 1, "un draft per livello");
    }
    // un XP in meno del livello 5 resta al 4, a un passo dal 5
    let mut app = xp_app();
    let at = player_position(&mut app);
    let total = xp_for_level(&config, 5) - 1;
    app.world_mut().spawn((XpGem { value: total }, body(at)));
    run_steps(&mut app, 2);
    let p = progression(&app);
    assert_eq!((p.level, p.xp_to_next - p.xp), (4, 1));
}
