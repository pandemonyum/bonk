//! P0.6 headless: attacchi telegrafati. L'élite mostra la carica e poi scatta; il colpo del
//! boss cade dove stava il player e rispetta le regole di tutti i danni; un preavviso sparisce
//! con chi l'ha lanciato e con il restart; élite e boss sono deterministici a parità di seed.

use crate::common::{
    config, count, despawn_all, despawn_all_but_boss, god, player_mut, player_position, run_steps,
    short_run, until_boss,
};
use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::gameplay::combat::{Health, RunStats};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::damage::{GodMode, Invulnerability};
use pixel_monster_survivor::gameplay::enemies::{Boss, Enemy, Scale, spawn_enemy};
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::threats::{
    ChargeState, Charger, PendingSlam, Telegraph, TelegraphShape,
};
use pixel_monster_survivor::headless_app_with;

/// Preavviso: ferma, con la linea a terra lunga quanto la carica; poi scatta della distanza
/// prevista verso dove stava il player.
#[test]
fn an_elite_shows_its_charge_then_dashes() {
    let mut config = GameConfig::embedded();
    for wave in &mut config.stage.spawns.waves {
        wave.rate_per_min = 0.0;
        wave.rate_growth_per_min = 0.0;
    }
    config.stage.run.elites.first_s = 1.0e6;
    let mut app = headless_app_with(1, config);
    god(&mut app);
    app.update();
    despawn_all::<Enemy>(&mut app);
    despawn_all::<Companion>(&mut app);
    let c = crate::common::config(&app);
    let data = c.enemy(&c.stage.run.elites.enemy).unwrap().clone();
    let charge = data.charge.unwrap();
    let start = player_position(&mut app) + Vec2::X * charge.range * 0.9;
    let elite = {
        let mut commands = app.world_mut().commands();
        let e = spawn_enemy(&mut commands, &data, start, 0, Scale::BASE);
        app.world_mut().flush();
        e
    };
    // ferma l'inseguimento e accorcia l'attesa: la carica parte da lontano, non a contatto
    app.world_mut().get_mut::<Enemy>(elite).unwrap().speed = 0.0;
    app.world_mut().get_mut::<Charger>(elite).unwrap().state =
        ChargeState::Chasing { cooldown: 0.1 };
    let state = |app: &App| app.world().get::<Charger>(elite).unwrap().state;
    let at = |app: &App| app.world().get::<Position>(elite).unwrap().0;
    let mut steps = 0;
    while !matches!(state(&app), ChargeState::Windup { .. }) {
        run_steps(&mut app, 1);
        steps += 1;
        assert!(steps < 60 * 20, "nessuna carica");
    }
    let windup_at = at(&app);
    let world = app.world_mut();
    let line = world
        .query::<&Telegraph>()
        .iter(world)
        .find_map(|t| match t.shape {
            TelegraphShape::Line { length, .. } => Some(length),
            _ => None,
        })
        .expect("linea del preavviso");
    assert!((line - charge.speed * charge.duration_s).abs() < 1e-3);
    let windup_steps = (charge.windup_s * 60.0).round() as usize;
    run_steps(&mut app, windup_steps - 1);
    assert!(
        at(&app).distance(windup_at) < 1.0,
        "fermo durante il preavviso"
    );
    let dash_steps = (charge.duration_s * 60.0).round() as usize;
    run_steps(&mut app, dash_steps + 1);
    let dashed = at(&app).distance(windup_at);
    let expected = charge.speed * charge.duration_s;
    assert!(
        (dashed - expected).abs() < expected * 0.1,
        "scatto di {dashed} px, attesi {expected}"
    );
}

/// Run accorciata con il boss fermo a metà portata e nessun altro nemico; restituisce l'app,
/// il boss e i dati del suo colpo.
fn boss_alone() -> (App, Entity, pixel_monster_survivor::data::schema::SlamData) {
    let mut app = headless_app_with(1, short_run(2.0));
    god(&mut app);
    let boss = until_boss(&mut app, 60 * 10);
    despawn_all::<Companion>(&mut app);
    despawn_all_but_boss(&mut app);
    let slam = {
        let c = config(&app);
        c.enemy(&c.stage.run.boss).unwrap().slam.unwrap()
    };
    let near = player_position(&mut app) + Vec2::X * slam.range * 0.5;
    app.world_mut().get_mut::<Position>(boss).unwrap().0 = near;
    app.world_mut().get_mut::<Enemy>(boss).unwrap().speed = 0.0;
    (app, boss, slam)
}

/// Corre fino al preavviso del colpo del boss.
fn until_slam(app: &mut App, every_s: f32) {
    let mut steps = 0;
    while count::<PendingSlam>(app) == 0 {
        run_steps(app, 1);
        steps += 1;
        assert!(
            steps < (every_s * 60.0) as usize + 5,
            "nessun colpo in arrivo"
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Case {
    Stay,
    Escape,
    God,
    Invulnerable,
}

/// Il colpo arriva dove stava il player al preavviso: chi resta prende il danno, chi esce no; la
/// modalità invulnerabile e la finestra di invulnerabilità valgono come per il contatto.
#[test]
fn the_boss_slam_hits_only_who_stays_in_the_ring() {
    for case in [Case::Stay, Case::Escape, Case::God, Case::Invulnerable] {
        let (mut app, _, slam) = boss_alone();
        app.insert_resource(GodMode(case == Case::God));
        let player = player_position(&mut app);
        let hp_before = player_mut(&mut app).get::<Health>().unwrap().current;
        until_slam(&mut app, slam.every_s);
        match case {
            Case::Escape => {
                let away = player + Vec2::new(0.0, -(slam.radius + 30.0));
                player_mut(&mut app).get_mut::<Position>().unwrap().0 = away;
            }
            Case::Invulnerable => {
                let mut p = player_mut(&mut app);
                p.get_mut::<Invulnerability>().unwrap().remaining = slam.windup_s + 1.0;
            }
            _ => {}
        }
        run_steps(&mut app, (slam.windup_s * 60.0).ceil() as usize + 1);
        let hp_after = player_mut(&mut app).get::<Health>().unwrap().current;
        if case == Case::Stay {
            assert!(
                (hp_before - hp_after - slam.damage).abs() < 1e-3,
                "{hp_before} → {hp_after}"
            );
        } else {
            assert_eq!(hp_after, hp_before, "{case:?}: nessun danno");
        }
        assert_eq!(
            count::<PendingSlam>(&mut app),
            0,
            "il preavviso sparisce col colpo"
        );
    }
}

/// Rilievo del code review P0.6: il colpo di un boss appena sconfitto non arriva più.
#[test]
fn a_slam_vanishes_when_its_boss_falls() {
    let (mut app, boss, slam) = boss_alone();
    app.insert_resource(GodMode(false));
    let hp_before = player_mut(&mut app).get::<Health>().unwrap().current;
    until_slam(&mut app, slam.every_s);
    app.world_mut().get_mut::<Health>(boss).unwrap().current = 0.0;
    run_steps(&mut app, 2);
    assert_eq!(count::<Telegraph>(&mut app), 0);
    assert_eq!(
        player_mut(&mut app).get::<Health>().unwrap().current,
        hp_before
    );
    assert_eq!(
        *app.world().resource::<State<RunPhase>>().get(),
        RunPhase::Victory
    );
}

/// Restart con un colpo in arrivo: nessun preavviso resta nella run nuova.
#[test]
fn restarting_with_a_slam_coming_clears_it() {
    let (mut app, _, slam) = boss_alone();
    until_slam(&mut app, slam.every_s);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    assert_eq!(count::<Telegraph>(&mut app), 0);
    assert_eq!(count::<PendingSlam>(&mut app), 0);
    assert_eq!(count::<Boss>(&mut app), 0);
}

/// Élite, cariche e boss non usano numeri casuali propri: stesso seed, stessa run.
#[test]
fn elites_and_the_boss_are_deterministic() {
    let simulate = |seed| {
        let mut config = short_run(30.0);
        config.stage.run.elites.first_s = 3.0;
        config.stage.run.elites.every_s = 6.0;
        let mut app = headless_app_with(seed, config);
        god(&mut app);
        app.insert_resource(MoveIntent(Vec2::new(0.6, 0.8)));
        run_steps(&mut app, 60 * 45);
        let world = app.world_mut();
        let mut enemies: Vec<(Vec2, f32)> = world
            .query::<(&Position, &Health, &Enemy)>()
            .iter(world)
            .map(|(p, h, _)| (p.0, h.current))
            .collect();
        enemies.sort_by(|a, b| a.0.x.total_cmp(&b.0.x).then(a.0.y.total_cmp(&b.0.y)));
        let telegraphs = world.query::<&Telegraph>().iter(world).count();
        let stats = app.world().resource::<RunStats>().clone();
        (enemies, telegraphs, stats)
    };
    let first = simulate(4);
    assert!(first.2.defeated > 0);
    assert_eq!(first, simulate(4));
}
