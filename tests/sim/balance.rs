//! Sonde di bilanciamento e prestazioni (non test di correttezza): stampano misure. Eseguire con
//! `cargo test --test sim balance -- --ignored --nocapture --test-threads 1`.

use std::time::Instant;

use crate::common::{god, quiet_config, run_steps};
use bevy::prelude::*;
use pixel_monster_survivor::app::RunPhase;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::combat::RunStats;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::progression::Progression;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::{headless_app, headless_app_with};

#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_hit_rate_over_one_minute() {
    for seed in [1, 2, 3, 4, 5] {
        for autoplay in [false, true] {
            let mut app = headless_app(seed);
            god(&mut app);
            for step in 0..60 * 60 {
                if autoplay {
                    let t = step as f32 / 60.0;
                    app.insert_resource(MoveIntent(autoplay_direction(t)));
                }
                run_steps(&mut app, 1);
            }
            let s = app.world().resource::<RunStats>().clone();
            println!(
                "seed {seed} autoplay {autoplay:5}: sparati {:3} colpi {:3} ({:4.0}%) sconfitti {:3}",
                s.projectiles_fired,
                s.projectile_hits,
                100.0 * s.projectile_hits as f32 / s.projectiles_fired.max(1) as f32,
                s.defeated
            );
        }
    }
}

/// Quanto dura una run senza upgrade con i danni attivi (nemici senza XP: nessun livello),
/// da fermi e con l'autoplay. Termine di paragone per la sonda con gli upgrade.
#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_survival_time_without_upgrades() {
    let mut config = GameConfig::embedded();
    for enemy in &mut config.enemies {
        enemy.xp = 0;
    }
    for seed in [1, 2, 3, 4, 5] {
        for autoplay in [false, true] {
            let mut app = headless_app_with(seed, config.clone());
            for step in 0..60 * 60 * 10 {
                if autoplay {
                    let t = step as f32 / 60.0;
                    app.insert_resource(MoveIntent(autoplay_direction(t)));
                }
                app.update();
                if *app.world().resource::<State<RunPhase>>().get() == RunPhase::Defeated {
                    break;
                }
            }
            let clock = app.world().resource::<RunClock>().label();
            let s = app.world().resource::<RunStats>().clone();
            println!(
                "seed {seed} autoplay {autoplay:5}: sopravvive {clock}, sconfitti {:3}, \
                 danni subiti {:.0}",
                s.defeated, s.damage_taken
            );
        }
    }
}

/// Progressione con la scelta automatica della prima carta: quando arrivano i livelli 2, 5 e
/// 10 (obiettivo GDD: primo level-up entro 30-45 s), livello finale e sopravvivenza. Le righe
/// "invulnerabile" misurano il ritmo dei livelli su 10 minuti senza che la morte lo interrompa.
#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_level_ups_with_auto_pick() {
    for seed in [1, 2, 3, 4, 5] {
        for (autoplay, invulnerable) in [(false, false), (true, false), (true, true)] {
            let mut app = headless_app(seed);
            if invulnerable {
                god(&mut app);
            }
            let mut reached: [Option<String>; 3] = [None, None, None];
            for step in 0..60 * 60 * 10 {
                if autoplay {
                    let t = step as f32 / 60.0;
                    app.insert_resource(MoveIntent(autoplay_direction(t)));
                }
                app.update();
                let level = app.world().resource::<Progression>().level;
                for (slot, wanted) in reached.iter_mut().zip([2, 5, 10]) {
                    if slot.is_none() && level >= wanted {
                        *slot = Some(app.world().resource::<RunClock>().label());
                    }
                }
                if *app.world().resource::<State<RunPhase>>().get() == RunPhase::Defeated {
                    break;
                }
            }
            let at = |slot: &Option<String>| slot.clone().unwrap_or("  -  ".into());
            let p = app.world().resource::<Progression>();
            println!(
                "seed {seed} autoplay {autoplay:5}{}: Lv2 {}  Lv5 {}  Lv10 {}  finale Lv {:2} a {}",
                if invulnerable { " invulnerabile" } else { "" },
                at(&reached[0]),
                at(&reached[1]),
                at(&reached[2]),
                p.level,
                app.world().resource::<RunClock>().label()
            );
        }
    }
}

/// Costo di un passo di simulazione (build dev: nostro codice opt-level 1, dipendenze 3).
#[test]
#[ignore = "sonda manuale di prestazioni"]
fn print_step_cost_with_many_enemies() {
    for target in [50u32, 200, 500] {
        let mut config = quiet_config();
        config.stage.spawns.max_alive = target;
        config.stage.spawns.initial_burst = target;
        let mut app = headless_app_with(1, config);
        god(&mut app);
        app.update();
        let world = app.world_mut();
        let enemies = world
            .query_filtered::<(), With<Enemy>>()
            .iter(world)
            .count();
        run_steps(&mut app, 120); // i nemici si avvicinano e si ammassano
        let steps = 600;
        let start = Instant::now();
        run_steps(&mut app, steps);
        let ms = start.elapsed().as_secs_f64() * 1000.0 / steps as f64;
        println!("{enemies:3} nemici: {ms:.3} ms per passo (budget a 60 Hz: 16,7 ms)");
    }
}

/// Sopravvivenza vera (danni attivi) con la prima carta scelta in automatico: da fermi e con
/// l'autoplay, fino alla fine della run. Stampa durata, livello, squadra e come è finita.
#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_survival_with_upgrades() {
    use pixel_monster_survivor::gameplay::companions::Companion;
    for seed in [1, 2, 3, 4, 5] {
        for autoplay in [false, true] {
            let mut app = headless_app(seed);
            for step in 0..60 * 60 * 13 {
                if autoplay {
                    let t = step as f32 / 60.0;
                    app.insert_resource(MoveIntent(autoplay_direction(t)));
                }
                app.update();
                let phase = *app.world().resource::<State<RunPhase>>().get();
                if matches!(phase, RunPhase::Defeated | RunPhase::Victory) {
                    break;
                }
            }
            let phase = *app.world().resource::<State<RunPhase>>().get();
            let clock = app.world().resource::<RunClock>().label();
            let level = app.world().resource::<Progression>().level;
            let world = app.world_mut();
            let party = world.query::<&Companion>().iter(world).count();
            println!(
                "seed {seed} autoplay {autoplay:5}: {phase:?} a {clock}, livello {level:2}, \
                 compagni {party}"
            );
        }
    }
}

/// Direzione di un bot che "kita": si allontana dai nemici entro `DANGER` px (pesati per
/// vicinanza), resta lontano dai bordi dell'arena, altrimenti vaga come l'autoplay. Una stima
/// rozza di un giocatore che schiva, per le sonde (non usata dal gioco).
fn kiting_direction(app: &mut App, t: f32) -> Vec2 {
    use pixel_monster_survivor::gameplay::kinematics::Position;
    use pixel_monster_survivor::gameplay::player::Player;
    use pixel_monster_survivor::world::ArenaBounds;
    const DANGER: f32 = 90.0;
    let arena = *app.world().resource::<ArenaBounds>();
    let world = app.world_mut();
    let me = world
        .query_filtered::<&Position, With<Player>>()
        .single(world)
        .unwrap()
        .0;
    let mut away = Vec2::ZERO;
    for at in world.query_filtered::<&Position, With<Enemy>>().iter(world) {
        let delta = me - at.0;
        let d2 = delta.length_squared();
        if d2 < DANGER * DANGER && d2 > 1e-3 {
            away += delta / d2;
        }
    }
    // i bordi spingono verso il centro, più forte da vicino
    let margin = 200.0;
    let edge = Vec2::new(
        ((arena.min.x + margin - me.x).max(0.0) - (me.x - (arena.max.x - margin)).max(0.0))
            / margin,
        ((arena.min.y + margin - me.y).max(0.0) - (me.y - (arena.max.y - margin)).max(0.0))
            / margin,
    );
    let wander = autoplay_direction(t) * 0.01;
    (away + edge * 0.05 + wander).normalize_or_zero()
}

/// Come `print_survival_with_upgrades`, con il bot che kita: quanto si arriva lontano se si
/// schiva invece di camminare alla cieca.
#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_survival_kiting() {
    for seed in [1, 2, 3, 4, 5] {
        let mut app = headless_app(seed);
        app.update();
        for step in 0..60 * 60 * 13 {
            let t = step as f32 / 60.0;
            let direction = kiting_direction(&mut app, t);
            app.insert_resource(MoveIntent(direction));
            app.update();
            let phase = *app.world().resource::<State<RunPhase>>().get();
            if matches!(phase, RunPhase::Defeated | RunPhase::Victory) {
                break;
            }
        }
        let phase = *app.world().resource::<State<RunPhase>>().get();
        let clock = app.world().resource::<RunClock>().label();
        let level = app.world().resource::<Progression>().level;
        println!("seed {seed} kiting: {phase:?} a {clock}, livello {level:2}");
    }
}
