//! Sonde di bilanciamento e prestazioni (non test di correttezza): stampano misure. Eseguire con
//! `cargo test --test sim balance -- --ignored --nocapture --test-threads 1`.

use std::time::Instant;

use crate::bot;
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

/// Profilo con tutti i nodi di tutti i compagni.
fn veteran_profile(config: &GameConfig) -> pixel_monster_survivor::profile::Profile {
    use pixel_monster_survivor::profile::{CompanionProgress, Profile, ProfileData};
    let companions = config
        .companions
        .iter()
        .map(|c| CompanionProgress {
            id: c.id.clone(),
            xp: u32::MAX / 2,
            learned: c
                .talents
                .iter()
                .flat_map(|b| b.nodes.iter().map(|n| n.id.clone()))
                .collect(),
        })
        .collect();
    Profile {
        data: ProfileData {
            companions,
            ..ProfileData::default()
        },
        path: None,
    }
}

/// Run vera (danni attivi) del "giocatore medio" (`bot`): come finisce, livello a 5:00 e a
/// 10:00, compagni, durata dello scontro con il boss e danni subiti. È il termine di paragone
/// per la difficoltà (GDD, obiettivi di bilanciamento): l'autoplay cieco muore presto, il bot
/// che scappa da tutto non cresce. `PMS_SEEDS=1,2` sceglie le seed, `PMS_START=tide` il
/// compagno iniziale.
#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_survival_medium_player() {
    use pixel_monster_survivor::gameplay::companions::Companion;
    use pixel_monster_survivor::gameplay::spawning::SpawnDirector;
    let seeds: Vec<u64> = std::env::var("PMS_SEEDS")
        .ok()
        .map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect())
        .unwrap_or_else(|| (1..=8).collect());
    // `PMS_START=tide` (o un altro id di companions.ron) cambia il compagno iniziale
    let mut config = GameConfig::embedded();
    if let Ok(start) = std::env::var("PMS_START") {
        config.stage.starting_companion = start;
    }
    println!("compagno iniziale: {}", config.stage.starting_companion);
    let mut wins = 0;
    // `PMS_TALENTS=1`: tutti i compagni con l'albero completo (il veterano dopo ~20 run)
    let veteran = std::env::var("PMS_TALENTS").is_ok();
    if veteran {
        println!("talenti: tutti");
    }
    for &seed in &seeds {
        let mut app = headless_app_with(seed, config.clone());
        if veteran {
            app.insert_resource(veteran_profile(&config));
        }
        bot::take_over(&mut app);
        app.update();
        let (mut at5, mut at10, mut boss_at) = (None, None, None);
        let (mut lv2_at, mut xp5, mut party2) = (None, 0, 0);
        // quando arrivano il secondo e il terzo compagno
        let mut joined: Vec<String> = Vec::new();
        let trace = std::env::var("PMS_TRACE").is_ok();
        let mut last_pos = Vec2::ZERO;
        for step in 0..60 * 60 * 20 {
            bot::drive(&mut app, step as f32 / 60.0);
            app.update();
            let clock = app.world().resource::<RunClock>().elapsed;
            let level = app.world().resource::<Progression>().level;
            if lv2_at.is_none() && level >= 2 {
                lv2_at = Some(app.world().resource::<RunClock>().label());
            }
            if trace && step % 3600 == 0 {
                trace_minute(&mut app, &mut last_pos);
            }
            if step % 30 == 0 {
                let world = app.world_mut();
                let members = world.query::<&Companion>().iter(world).count();
                while joined.len() + 1 < members {
                    joined.push(app.world().resource::<RunClock>().label());
                }
            }
            if party2 == 0 && clock >= 120.0 {
                let world = app.world_mut();
                party2 = world.query::<&Companion>().iter(world).count();
            }
            if at5.is_none() && clock >= 300.0 {
                at5 = Some(level);
                xp5 = app.world().resource::<Progression>().total_xp;
            }
            if at10.is_none() && clock >= 600.0 {
                at10 = Some(level);
            }
            if boss_at.is_none() && app.world().resource::<SpawnDirector>().boss_spawned {
                boss_at = Some(clock);
            }
            let phase = *app.world().resource::<State<RunPhase>>().get();
            if matches!(phase, RunPhase::Defeated | RunPhase::Victory) {
                break;
            }
        }
        let phase = *app.world().resource::<State<RunPhase>>().get();
        let clock = app.world().resource::<RunClock>();
        let end = clock.elapsed;
        let label = clock.label();
        let level = app.world().resource::<Progression>().level;
        let stats = app.world().resource::<RunStats>().clone();
        let chests = app
            .world()
            .resource::<pixel_monster_survivor::gameplay::treasure::Treasury>()
            .opened;
        let world = app.world_mut();
        let mut members: Vec<(usize, String)> = world
            .query::<(&Companion, &pixel_monster_survivor::data::DataId)>()
            .iter(world)
            .map(|(c, id)| (c.slot, id.0.clone()))
            .collect();
        members.sort();
        let party: Vec<String> = members.into_iter().map(|(_, id)| id).collect();
        let party = party.join("+");
        let fight = boss_at.map_or("  -  ".to_string(), |b| format!("{:5.0} s", end - b));
        let lv = |l: Option<u32>| l.map_or(" -".to_string(), |l| format!("{l:2}"));
        if phase == RunPhase::Victory {
            wins += 1;
        }
        println!(
            "seed {seed:2} medio: {phase:?} a {label}, Lv2 a {}, compagni a 2:00 {party2} \
             (arrivi {}), Lv5:00 {} (XP {xp5:3}) Lv10:00 {} finale {level:2}, compagni {party}, \
             boss {fight}, danni subiti {:.0}, sconfitti {}",
            lv2_at.unwrap_or("  -  ".into()),
            joined.join(" "),
            lv(at5),
            lv(at10),
            stats.damage_taken,
            stats.defeated
        );
        println!("         forzieri aperti {chests}");
    }
    println!("vittorie {wins}/{}", seeds.len());
}

/// Una riga al minuto per `PMS_TRACE`: posizione, strada fatta nell'ultimo minuto, XP, gemme a
/// terra e nemici vivi (per capire se il bot si incastra o non raccoglie).
fn trace_minute(app: &mut App, last: &mut Vec2) {
    use pixel_monster_survivor::gameplay::kinematics::Position;
    use pixel_monster_survivor::gameplay::player::Player;
    use pixel_monster_survivor::gameplay::progression::XpGem;
    let clock = app.world().resource::<RunClock>().label();
    let total_xp = app.world().resource::<Progression>().total_xp;
    let world = app.world_mut();
    let me = world
        .query_filtered::<&Position, With<Player>>()
        .single(world)
        .map_or(Vec2::ZERO, |p| p.0);
    let gems = world.query::<&XpGem>().iter(world).count();
    let enemies = world.query::<&Enemy>().iter(world).count();
    println!(
        "  {clock}: player ({:5.0},{:5.0}), spostato {:5.0} px, XP {total_xp:4}, gemme {gems:3}, \
         nemici {enemies:3}",
        me.x,
        me.y,
        me.distance(*last)
    );
    *last = me;
}
