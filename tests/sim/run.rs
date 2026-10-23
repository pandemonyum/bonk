//! P0.6 headless: arco della run. A fine durata le ondate si fermano e nasce il boss (una
//! volta, con i suoi valori); sconfiggerlo è la vittoria, morire nello stesso passo resta una
//! sconfitta; i nemici nascono più forti con i minuti; le élite arrivano in orario; il menu
//! sceglie il compagno e i flussi menu/restart vanno nel posto giusto; una run completa con
//! l'autoplay arriva alla vittoria e il restart riparte identico a una run nuova. Attacchi
//! telegrafati in `threats.rs`. Tempi e valori dai dati (in parte accorciati dal test).

use crate::common::{
    clock, config, count, despawn_all, director, enemy_at, god, phase, player_mut, player_position,
    run_steps, short_run, until_boss,
};
use bevy::prelude::*;
use pixel_monster_survivor::app::{
    AfterLoading, AppState, RunPhase, RunSetup, back_to_menu, restart_run, start_run,
};
use pixel_monster_survivor::data::schema::Effect;
use pixel_monster_survivor::data::{DataId, GameConfig};
use pixel_monster_survivor::gameplay::combat::{Health, Projectile, RunStats};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::damage::{GodMode, Invulnerability};
use pixel_monster_survivor::gameplay::enemies::{Boss, Enemy, Scale};
use pixel_monster_survivor::gameplay::orbit::Leaf;
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::gameplay::progression::{Progression, XpGem};
use pixel_monster_survivor::gameplay::pulse::Pulse;
use pixel_monster_survivor::gameplay::threats::{Charger, PendingSlam, Telegraph};
use pixel_monster_survivor::gameplay::upgrades::{PartyView, UpgradeStacks, is_eligible};
use pixel_monster_survivor::world::obstacles::Obstacle;
use pixel_monster_survivor::{headless_app, headless_app_with};

fn app_state(app: &App) -> AppState {
    *app.world().resource::<State<AppState>>().get()
}

#[test]
fn at_the_end_of_the_run_the_waves_stop_and_one_boss_arrives() {
    let duration = 20.0;
    let mut app = headless_app_with(1, short_run(duration));
    god(&mut app);
    let boss = until_boss(&mut app, 60 * 30);
    assert!(clock(&app) >= duration, "nato a {} s", clock(&app));
    let born = director(&app).spawned;
    run_steps(&mut app, 60 * 10);
    assert_eq!(
        director(&app).spawned,
        born,
        "dopo la fine niente più ondate né élite"
    );
    assert_eq!(count::<Boss>(&mut app), 1);
    // il boss ha i valori dei dati, senza la crescita per minuto
    let data = config(&app)
        .enemy(&config(&app).stage.run.boss)
        .unwrap()
        .clone();
    let health = *app.world().get::<Health>(boss).unwrap();
    assert!((health.max - data.max_hp).abs() < 1e-3, "{}", health.max);
    let enemy = *app.world().get::<Enemy>(boss).unwrap();
    assert!((enemy.contact_damage - data.contact_damage).abs() < 1e-4);
}

#[test]
fn defeating_the_boss_wins_and_freezes_the_run() {
    let mut app = headless_app_with(1, short_run(5.0));
    god(&mut app);
    let boss = until_boss(&mut app, 60 * 15);
    assert_eq!(phase(&app), RunPhase::Playing);
    app.world_mut().get_mut::<Health>(boss).unwrap().current = 0.0;
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::Victory);
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    let frozen = clock(&app);
    run_steps(&mut app, 60);
    assert_eq!(clock(&app), frozen, "tempo fermo sui risultati");
}

/// Rilievo del code review P0.6: il caso vero. Nello stesso passo il combattimento rimuove il
/// boss (a 0 HP) e il contatto uccide il player (a 1 HP): vince la sconfitta.
#[test]
fn dying_together_with_the_boss_is_a_defeat() {
    let mut app = headless_app_with(1, short_run(5.0));
    god(&mut app);
    let boss = until_boss(&mut app, 60 * 15);
    crate::common::despawn_all_but_boss(&mut app);
    despawn_all::<Companion>(&mut app);
    let contact = config(&app).first_wave_enemy().contact_distance;
    let touching = player_position(&mut app) + Vec2::X * contact;
    enemy_at(&mut app, touching, 1.0e6, 0);
    app.insert_resource(GodMode(false));
    let mut player = player_mut(&mut app);
    player.get_mut::<Health>().unwrap().current = 1.0;
    player.get_mut::<Invulnerability>().unwrap().remaining = 0.0;
    app.world_mut().get_mut::<Health>(boss).unwrap().current = 0.0;
    run_steps(&mut app, 2);
    assert_eq!(
        count::<Boss>(&mut app),
        0,
        "il boss è caduto nello stesso passo"
    );
    assert!(player_mut(&mut app).get::<Health>().unwrap().is_dead());
    assert_eq!(phase(&app), RunPhase::Defeated);
}

/// Ogni nemico delle ondate nasce con HP e danno della sua ora: `1 + crescita × minuti`
/// (crescita alta e ondate fitte: bastano 20 s per vedere la differenza).
#[test]
fn enemies_are_born_stronger_as_the_minutes_pass() {
    let mut config = GameConfig::embedded();
    config.stage.run.hp_growth_per_min = 6.0;
    config.stage.run.damage_growth_per_min = 3.0;
    config.stage.run.elites.first_s = 1.0e6;
    config.stage.spawns.waves[0].rate_per_min = 300.0;
    let run = config.stage.run.clone();
    let mut app = headless_app_with(2, config.clone());
    god(&mut app);
    app.update();
    let world = app.world_mut();
    // l'ondata iniziale nasce prima del primo passo, con i valori base
    let mut seen: std::collections::HashSet<Entity> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    let mut checked = 0;
    for _ in 0..60 * 20 {
        run_steps(&mut app, 1);
        let t = clock(&app); // lo spawn usa l'orologio già avanzato di questo passo
        let world = app.world_mut();
        let fresh: Vec<_> = world
            .query_filtered::<(Entity, &DataId, &Health, &Enemy), Without<Boss>>()
            .iter(world)
            .filter(|(e, ..)| !seen.contains(e))
            .map(|(e, id, h, enemy)| (e, id.0.clone(), h.max, enemy.contact_damage))
            .collect();
        for (entity, id, max_hp, damage) in fresh {
            seen.insert(entity);
            let data = config.enemy(&id).unwrap();
            let scale = Scale::at(run.hp_growth_per_min, run.damage_growth_per_min, t);
            let expected = data.max_hp * scale.hp;
            assert!(
                (max_hp - expected).abs() < expected * 1e-4,
                "{id} a {t} s: {max_hp}"
            );
            let expected = data.contact_damage * scale.damage;
            assert!((damage - expected).abs() < expected * 1e-4 + 1e-6);
            checked += 1;
        }
    }
    assert!(checked > 20, "pochi nemici controllati: {checked}");
}

#[test]
fn elites_arrive_on_schedule() {
    let mut config = GameConfig::embedded();
    config.stage.run.elites.first_s = 5.0;
    config.stage.run.elites.every_s = 10.0;
    let mut app = headless_app_with(3, config);
    god(&mut app);
    run_steps(&mut app, 60 * 36);
    // a 5, 15, 25 e 35 s (un punto valido c'è quasi sempre al primo passo)
    assert_eq!(director(&app).elites_spawned, 4);
    assert!(count::<Charger>(&mut app) >= 1, "le élite caricano");
}

/// Il compagno scelto nel menu parte nel primo posto; la sua carta non esce, quella degli altri
/// sì, e senza nessuno che spara non escono perforazione e velocità dei proiettili.
#[test]
fn the_companion_chosen_in_the_menu_starts_the_run() {
    let c = GameConfig::embedded();
    let wave = c
        .companions
        .iter()
        .find(|x| c.attack(&x.attack).is_some_and(|a| a.pulse().is_some()))
        .unwrap()
        .id
        .clone();
    let mut app = headless_app_with(1, c.clone());
    app.insert_resource(RunSetup {
        companion: Some(wave.clone()),
    });
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    let world = app.world_mut();
    let members: Vec<String> = world
        .query::<(&DataId, &Companion)>()
        .iter(world)
        .map(|(id, _)| id.0.clone())
        .collect();
    assert_eq!(members, std::slice::from_ref(&wave));
    let stacks = app.world().resource::<UpgradeStacks>().clone();
    let party = PartyView::of(&c, &stacks, &wave);
    for (i, u) in c.upgrades.iter().enumerate() {
        let eligible = is_eligible(&c.upgrades, &stacks, i, &party);
        match &u.effect {
            Effect::JoinCompanion(id) => assert_eq!(eligible, *id != wave, "{}", u.id),
            Effect::CompanionPierce(_) | Effect::ProjectileSpeedPct(_) => {
                assert!(!eligible, "{}: nessuno spara", u.id)
            }
            _ => {}
        }
    }
}

/// Flussi dell'app senza finestra: dal menu si parte con il compagno scelto, F5 e Invio
/// ripartono senza menu, M torna al menu.
#[test]
fn menu_restart_and_back_to_menu_go_where_they_should() {
    let mut app = headless_app(1);
    app.update();
    let mut after = *app.world().resource::<AfterLoading>();
    let mut next = NextState::<AppState>::default();
    back_to_menu(&mut next, &mut after);
    app.insert_resource(after);
    app.insert_resource(next);
    run_steps(&mut app, 3);
    assert_eq!(app_state(&app), AppState::MainMenu);
    assert_eq!(count::<Player>(&mut app), 0, "la run è chiusa");
    let other = config(&app)
        .companions
        .iter()
        .find(|c| c.id != config(&app).stage.starting_companion)
        .unwrap()
        .id
        .clone();
    let mut setup = app.world().resource::<RunSetup>().clone();
    let mut after = *app.world().resource::<AfterLoading>();
    let mut next = NextState::<AppState>::default();
    start_run(&mut setup, &mut after, &mut next, &other);
    app.insert_resource(setup);
    app.insert_resource(after);
    app.insert_resource(next);
    run_steps(&mut app, 2);
    assert_eq!(app_state(&app), AppState::InRun);
    // restart: di nuovo in run, con lo stesso compagno, senza passare dal menu
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    assert_eq!(app_state(&app), AppState::InRun);
    let world = app.world_mut();
    let ids: Vec<String> = world
        .query_filtered::<&DataId, With<Companion>>()
        .iter(world)
        .map(|id| id.0.clone())
        .collect();
    assert_eq!(ids, [other]);
}

/// Tutto ciò che una run lascia in campo e nelle risorse: dopo il restart deve coincidere con
/// una run nuova dello stesso seed.
fn census(app: &mut App) -> Vec<(String, u64)> {
    let d = director(app);
    let p = app.world().resource::<Progression>().clone();
    let stacks: u32 = app.world().resource::<UpgradeStacks>().0.iter().sum();
    let stats = app.world().resource::<RunStats>().clone();
    vec![
        ("player".into(), count::<Player>(app) as u64),
        ("compagni".into(), count::<Companion>(app) as u64),
        ("nemici".into(), count::<Enemy>(app) as u64),
        ("boss".into(), count::<Boss>(app) as u64),
        ("cariche".into(), count::<Charger>(app) as u64),
        ("ostacoli".into(), count::<Obstacle>(app) as u64),
        ("proiettili".into(), count::<Projectile>(app) as u64),
        ("onde".into(), count::<Pulse>(app) as u64),
        ("foglie".into(), count::<Leaf>(app) as u64),
        ("gemme".into(), count::<XpGem>(app) as u64),
        ("preavvisi".into(), count::<Telegraph>(app) as u64),
        ("colpi in arrivo".into(), count::<PendingSlam>(app) as u64),
        ("nati".into(), u64::from(d.spawned)),
        ("élite".into(), u64::from(d.elites_spawned)),
        ("boss nato".into(), u64::from(d.boss_spawned)),
        ("livello".into(), u64::from(p.level)),
        ("stack".into(), u64::from(stacks)),
        ("sconfitti".into(), u64::from(stats.defeated)),
    ]
}

/// Backlog P0.6: run completa con i dati veri a seed fisso, autoplay e player invulnerabile,
/// fino alla vittoria senza panic; poi il restart riparte identico a una run nuova.
#[test]
fn a_full_run_with_autoplay_reaches_the_boss_and_wins() {
    let mut fresh = headless_app(5);
    god(&mut fresh);
    fresh.update();
    let expected = census(&mut fresh);

    let mut app = headless_app(5);
    god(&mut app);
    app.update(); // la fase della run esiste dal primo aggiornamento
    let duration = config(&app).stage.run.duration_s;
    let limit = ((duration + 240.0) * 60.0) as usize;
    let mut steps = 0;
    let mut boss_at = None;
    while phase(&app) != RunPhase::Victory {
        let t = steps as f32 / 60.0;
        let direction = pixel_monster_survivor::diagnostics::automation::autoplay_direction(t);
        app.insert_resource(MoveIntent(direction));
        app.update();
        steps += 1;
        if boss_at.is_none() && director(&app).boss_spawned {
            boss_at = Some(clock(&app));
        }
        assert!(
            steps < limit,
            "nessuna vittoria: {:?} a {:.0} s, boss nato a {boss_at:?}",
            phase(&app),
            clock(&app)
        );
    }
    let boss_at = boss_at.expect("boss nato");
    let level = app.world().resource::<Progression>().level;
    let defeated = app.world().resource::<RunStats>().defeated;
    println!(
        "vittoria a {:.0} s: boss a {boss_at:.0} s, combattimento {:.0} s, livello {level}, \
         sconfitti {defeated}",
        clock(&app),
        clock(&app) - boss_at,
    );
    assert!(boss_at >= duration);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::Playing);
    assert_eq!(census(&mut app), expected);
}

/// Rilievo del code review P0.6: con più passi per fotogramma (tempo accelerato, FPS bassi) la
/// simulazione continuava dopo la caduta del boss; ora quel passo è l'ultimo, come per la morte.
#[test]
fn the_boss_falling_is_the_last_step_even_with_accelerated_time() {
    let mut app = headless_app_with(1, short_run(5.0));
    god(&mut app);
    let boss = until_boss(&mut app, 60 * 15);
    app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(4));
    app.world_mut().get_mut::<Health>(boss).unwrap().current = 0.0;
    let before = clock(&app);
    app.update();
    let after = clock(&app);
    assert!(
        (after - before - 1.0 / 60.0).abs() < 1e-6,
        "avanzato di {} s",
        after - before
    );
}
