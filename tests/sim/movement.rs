//! Simulazione completa senza finestra né GPU: ogni `app.update()` = un passo fisso da 1/60 s.
//! Movimento, inseguimento, limiti, pausa, restart, valori dai dati. Il combattimento è in
//! `combat.rs`, gli ostacoli in `world.rs`, lo spawn director in `spawning.rs`, i danni al
//! player in `damage.rs`. Qui le run sono "tranquille" (`quiet_app`): niente nemici nuovi
//! dopo l'ondata iniziale e player invulnerabile, salvo dove serve il contrario.

use crate::common::{
    config, count, despawn_all, enemy_positions, god, mean_distance, player_position, quiet_app,
    quiet_config, run_steps,
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use pixel_monster_survivor::app::{AppState, RunPhase, SimulationPlugin, restart_run};
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::gameplay::combat::{Projectile, RunStats};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::gameplay::progression::Progression;
use pixel_monster_survivor::gameplay::spawning::SpawnDirector;
use pixel_monster_survivor::gameplay::upgrades::UpgradeStacks;
use pixel_monster_survivor::world::ArenaBounds;
use pixel_monster_survivor::world::obstacles::Obstacle;
use pixel_monster_survivor::{headless_app, headless_app_with};

#[test]
fn run_spawns_one_player_and_initial_wave() {
    let mut app = headless_app(1);
    app.update();
    assert_eq!(player_position(&mut app), Vec2::ZERO);
    let initial = config(&app).stage.spawns.initial_burst as usize;
    assert_eq!(enemy_positions(&mut app).len(), initial);
}

#[test]
fn player_moves_at_the_speed_in_the_data() {
    let mut app = quiet_app(1);
    app.update();
    let speed = config(&app).player.speed;
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60);
    let moved = player_position(&mut app).x;
    // 60 passi da 1/60 s: un secondo di movimento, con tolleranza di un passo.
    assert!(
        (moved - speed).abs() <= speed / 60.0 + 0.01,
        "spostamento {moved}"
    );
}

#[test]
fn changing_the_data_changes_the_game() {
    let mut fast = quiet_config();
    fast.player.speed *= 2.0;
    fast.stage.spawns.initial_burst = 3;
    let mut app = headless_app_with(1, fast.clone());
    app.update();
    assert_eq!(count::<Enemy>(&mut app), 3);
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60);
    let moved = player_position(&mut app).x;
    assert!((moved - fast.player.speed).abs() <= fast.player.speed / 60.0 + 0.01);
}

#[test]
fn player_never_leaves_the_arena() {
    let mut app = quiet_app(1);
    app.update();
    let arena = *app.world().resource::<ArenaBounds>();
    app.insert_resource(MoveIntent(Vec2::new(-1.0, 1.0).normalize()));
    // 60 s alla massima velocità: molto più della mezza arena.
    run_steps(&mut app, 60 * 60);
    let at = player_position(&mut app);
    assert!(arena.contains(at), "{at:?}");
    assert!(
        (at - Vec2::new(arena.min.x, arena.max.y)).length() < 0.01,
        "{at:?}"
    );
}

#[test]
fn enemies_never_enter_the_player_against_a_wall() {
    let mut app = quiet_app(3);
    app.update();
    let arena = *app.world().resource::<ArenaBounds>();
    let contact = config(&app).first_wave_enemy().contact_distance;
    // Il player corre fino al muro est (~27 s) e ci resta: i nemici lo schiacciano contro.
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 60 * 45);
    let player = player_position(&mut app);
    assert_eq!(player.x, arena.max.x);
    for (i, enemy) in enemy_positions(&mut app).iter().enumerate() {
        let distance = enemy.distance(player);
        assert!(distance >= contact - 0.01, "nemico {i} a {distance} px");
        // La respinta può superare i limiti solo della distanza di contatto.
        assert!(
            enemy.x <= arena.max.x + contact + 0.01,
            "nemico {i} oltre il muro"
        );
    }
}

#[test]
fn enemies_chase_the_player_and_do_not_stack() {
    let mut app = quiet_app(7);
    app.update();
    let enemy = config(&app).first_wave_enemy().clone();
    despawn_all::<Companion>(&mut app); // solo movimento: nessun nemico sconfitto
    let before = mean_distance(&enemy_positions(&mut app), Vec2::ZERO);
    run_steps(&mut app, 60 * 10);
    let enemies = enemy_positions(&mut app);
    let after = mean_distance(&enemies, player_position(&mut app));
    assert!(after < before * 0.25, "prima {before}, dopo {after}");
    // Nessun nemico dentro il player; la separazione (una passata per passo) tiene le coppie
    // almeno a metà del raggio di separazione anche nella calca.
    let player = player_position(&mut app);
    for (i, a) in enemies.iter().enumerate() {
        assert!(
            a.distance(player) >= enemy.contact_distance - 0.01,
            "nemico {i} dentro il player: {}",
            a.distance(player)
        );
        for b in &enemies[i + 1..] {
            let gap = a.distance(*b);
            assert!(
                gap > enemy.separation_radius * 0.5,
                "nemici sovrapposti a {gap} px"
            );
        }
    }
}

/// Con lo spawn director, i danni e il combattimento attivi: due run con lo stesso seed sono
/// identiche passo per passo.
#[test]
fn same_seed_same_simulation() {
    let simulate = |seed| {
        let mut app = headless_app(seed);
        app.update();
        app.insert_resource(MoveIntent(Vec2::new(0.6, 0.8)));
        run_steps(&mut app, 60 * 40);
        let mut enemies = enemy_positions(&mut app);
        enemies.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        let stats = app.world().resource::<RunStats>().clone();
        // anche livelli e carte scelte (draft con stream RNG proprio e scelta automatica)
        let levels = app.world().resource::<Progression>().clone();
        let stacks = app.world().resource::<UpgradeStacks>().clone();
        (player_position(&mut app), enemies, stats, levels, stacks)
    };
    let first = simulate(42);
    assert!(
        first.1.len() > 8,
        "il director deve aver fatto nascere nemici"
    );
    assert_eq!(first, simulate(42));
    assert_ne!(first.1, simulate(43).1);
}

#[test]
fn pause_freezes_the_simulation() {
    let mut app = quiet_app(1);
    app.update();
    let speed = config(&app).player.speed;
    app.insert_resource(MoveIntent(Vec2::Y));
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Paused);
    app.update();
    let frozen = player_position(&mut app);
    run_steps(&mut app, 120);
    assert_eq!(player_position(&mut app), frozen);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );

    // Ripresa: il tempo virtuale riparte e il player torna a muoversi.
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Playing);
    run_steps(&mut app, 60);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    let resumed = player_position(&mut app).y - frozen.y;
    assert!(
        resumed > speed * 0.9,
        "dopo la pausa si è mosso di {resumed}"
    );
}

/// Snapshot delle entità della run: dopo un restart deve tornare identico a una run nuova.
fn run_census(app: &mut App) -> (usize, usize, usize, usize, usize, Vec2, u32) {
    let spawned = app.world().resource::<SpawnDirector>().spawned;
    (
        count::<Player>(app),
        count::<Companion>(app),
        count::<Enemy>(app),
        count::<Obstacle>(app),
        count::<Projectile>(app),
        player_position(app),
        spawned,
    )
}

#[test]
fn restart_rebuilds_a_clean_run_without_leftovers() {
    let mut fresh = headless_app(4);
    fresh.update();
    let expected = run_census(&mut fresh);

    let mut app = headless_app(4);
    god(&mut app);
    app.update();
    // Fermo e invulnerabile: i nemici arrivano, il compagno ne sconfigge, il director ne crea.
    run_steps(&mut app, 60 * 30);
    assert!(app.world().resource::<RunStats>().defeated > 0);
    assert!(app.world().resource::<SpawnDirector>().spawned > expected.6);
    app.insert_resource(MoveIntent(Vec2::new(1.0, 0.3).normalize()));
    run_steps(&mut app, 60);
    app.insert_resource(MoveIntent(Vec2::ZERO));

    // Anche da in pausa: il restart deve far ripartire il tempo.
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Paused);
    app.update();
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    // Loading → (configurazione presente) → InRun: due frame di transizione.
    run_steps(&mut app, 2);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert_eq!(run_census(&mut app), expected);
    assert_eq!(*app.world().resource::<RunStats>(), RunStats::default());
}

/// Come nel client: l'app parte in `Loading` senza `GameConfig` (i file arrivano in modo
/// asincrono) e nessun sistema deve chiederla prima che esista (rilievo P0.4: il ricalcolo
/// degli upgrade andava in panic all'avvio del gioco vero, mai in headless).
#[test]
fn the_app_waits_for_the_data_without_panicking_then_starts_the_run() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .add_plugins(SimulationPlugin { seed: 1 })
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    run_steps(&mut app, 10);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Loading
    );
    app.insert_resource(GameConfig::embedded());
    run_steps(&mut app, 3);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );
    assert_eq!(count::<Player>(&mut app), 1);
}
