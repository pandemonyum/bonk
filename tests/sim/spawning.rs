//! P0.3 headless: spawn director. Ogni nemico nasce fuori dalla vista, dentro l'arena e fuori
//! dagli ostacoli; i limiti per archetipo e totali reggono; gli archetipi entrano al loro
//! momento; il ritmo segue le curve di `stages.ron`.

use std::collections::{HashMap, HashSet};

use crate::common::{config, god, player_position, quiet_config};
use bevy::prelude::*;
use pixel_monster_survivor::data::{DataId, GameConfig};
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::collision::Footprint;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::gameplay::spawning::{SPAWN_VIEW_MARGIN, SpawnDirector};
use pixel_monster_survivor::world::ArenaBounds;
use pixel_monster_survivor::world::obstacles::ObstacleField;
use pixel_monster_survivor::world::view::view_rect;
use pixel_monster_survivor::{headless_app, headless_app_with};

/// Controlla ogni nemico nuovo comparso nell'ultimo passo; restituisce i vivi per archetipo.
fn check_new_spawns(
    app: &mut App,
    known: &mut HashSet<Entity>,
    first_seen: &mut HashMap<String, f32>,
) -> HashMap<String, usize> {
    let player = player_position(app);
    let clock = app.world().resource::<RunClock>().elapsed;
    let arena = *app.world().resource::<ArenaBounds>();
    let field = app.world().resource::<ObstacleField>().clone();
    let world = app.world_mut();
    let mut enemies =
        world.query_filtered::<(Entity, &Position, &DataId, &Footprint), With<Enemy>>();
    let mut alive = HashMap::new();
    for (entity, at, id, footprint) in enemies.iter(world) {
        *alive.entry(id.0.clone()).or_insert(0) += 1;
        if known.insert(entity) {
            // Controlli scritti qui, indipendenti da `is_valid_spawn` del director.
            let where_ = format!("{} in {:?} (player {player:?}, t {clock:.1})", id.0, at.0);
            let off_screen = view_rect(player).inflate(SPAWN_VIEW_MARGIN);
            assert!(!off_screen.contains(at.0), "nato in vista: {where_}");
            assert!(arena.contains(at.0), "nato fuori dall'arena: {where_}");
            for o in field.near(at.0, footprint.0) {
                let gap = o.footprint_center().distance(at.0) - o.footprint_radius();
                assert!(gap >= footprint.0, "nato dentro un ostacolo: {where_}");
            }
            first_seen.entry(id.0.clone()).or_insert(clock);
        }
    }
    alive
}

fn assert_caps(config: &GameConfig, alive: &HashMap<String, usize>) {
    let plan = &config.stage.spawns;
    let total: usize = alive.values().sum();
    assert!(total <= plan.max_alive as usize, "{total} nemici vivi");
    for wave in &plan.waves {
        let n = alive.get(&wave.enemy).copied().unwrap_or(0);
        assert!(n <= wave.max_alive as usize, "{} vivi: {n}", wave.enemy);
    }
}

#[test]
fn every_spawn_is_valid_caps_hold_and_kinds_enter_on_time() {
    let mut app = headless_app(11);
    god(&mut app);
    let config = GameConfig::embedded();
    let (mut known, mut first_seen) = (HashSet::new(), HashMap::new());
    // 3:20 di gioco: entrano tutti e tre gli archetipi (il corazzato a 2:30).
    for step in 0..60 * 200 {
        app.insert_resource(MoveIntent(autoplay_direction(step as f32 / 60.0)));
        app.update();
        let alive = check_new_spawns(&mut app, &mut known, &mut first_seen);
        assert_caps(&config, &alive);
    }
    for wave in &config.stage.spawns.waves {
        let t = first_seen
            .get(&wave.enemy)
            .unwrap_or_else(|| panic!("{} mai comparso", wave.enemy));
        assert!(*t >= wave.start_s - 1e-3, "{} comparso a {t} s", wave.enemy);
    }
    assert!(known.len() > 200, "solo {} nemici in 3:20", known.len());
}

/// Nell'angolo la camera non è centrata sul player e metà dell'anello è fuori dall'arena: gli
/// spawn devono continuare, sempre validi.
#[test]
fn spawns_keep_working_in_a_corner() {
    let mut app = headless_app(12);
    god(&mut app);
    let (mut known, mut first_seen) = (HashSet::new(), HashMap::new());
    app.insert_resource(MoveIntent(Vec2::ONE.normalize()));
    for _ in 0..60 * 40 {
        app.update(); // ~38 s per arrivare nell'angolo nord-est
        check_new_spawns(&mut app, &mut known, &mut first_seen);
    }
    let arena = *app.world().resource::<ArenaBounds>();
    assert!(player_position(&mut app).distance(arena.max) < 1.0);
    let before = known.len();
    for _ in 0..60 * 30 {
        app.update();
        check_new_spawns(&mut app, &mut known, &mut first_seen);
    }
    assert!(
        known.len() > before + 10,
        "nell'angolo: {} nuovi",
        known.len() - before
    );
}

/// Nati attesi tra `from` e `to` secondi: integrale esatto delle rette di `stages.ron`,
/// calcolato qui in modo indipendente dal director.
fn expected_spawns(config: &GameConfig, from: f32, to: f32) -> f32 {
    config
        .stage
        .spawns
        .waves
        .iter()
        .map(|w| {
            let (t0, t1) = (from.max(w.start_s), to);
            if t1 <= t0 {
                return 0.0;
            }
            let (a, b) = (t0 - w.start_s, t1 - w.start_s); // secondi dall'inizio dell'ondata
            w.rate_per_min * (b - a) / 60.0 + w.rate_growth_per_min * (b * b - a * a) / 7200.0
        })
        .sum()
}

/// Senza limiti di popolazione il numero di nati segue l'integrale del ritmo, minuto per minuto.
#[test]
fn the_spawn_rate_follows_the_stage_curves() {
    let mut config = GameConfig::embedded();
    config.stage.spawns.max_alive = 100_000;
    for wave in &mut config.stage.spawns.waves {
        wave.max_alive = 100_000;
    }
    let mut app = headless_app_with(13, config.clone());
    god(&mut app);
    let spawned = |app: &App| app.world().resource::<SpawnDirector>().spawned;
    for _ in 0..60 * 60 {
        app.update();
    }
    let first_minute = spawned(&app) - config.stage.spawns.initial_burst;
    for _ in 0..60 * 60 {
        app.update();
    }
    let after_two = spawned(&app);
    for _ in 0..60 * 60 {
        app.update();
    }
    let third_minute = spawned(&app) - after_two;
    // Il budget è discreto (un nemico quando arriva a 1): scarto massimo di ~1 per ondata.
    let tolerance = config.stage.spawns.waves.len() as f32;
    for (label, got, want) in [
        (
            "primo minuto",
            first_minute,
            expected_spawns(&config, 0.0, 60.0),
        ),
        (
            "terzo minuto",
            third_minute,
            expected_spawns(&config, 120.0, 180.0),
        ),
    ] {
        assert!(
            (got as f32 - want).abs() <= tolerance,
            "{label}: nati {got}, attesi {want:.2}"
        );
    }
    assert!(third_minute > first_minute * 2, "il ritmo deve crescere");
}

#[test]
fn quiet_runs_spawn_only_the_initial_burst() {
    let mut app = headless_app_with(14, quiet_config());
    god(&mut app);
    for _ in 0..60 * 30 {
        app.update();
    }
    let burst = config(&app).stage.spawns.initial_burst;
    assert_eq!(app.world().resource::<SpawnDirector>().spawned, burst);
}
