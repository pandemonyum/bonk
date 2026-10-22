//! P0.3 headless: danni al player. Contatto, somma dei nemici, finestra di invulnerabilità,
//! modalità invulnerabile, sconfitta che ferma la simulazione (anche col tempo accelerato e
//! con una pausa chiesta nello stesso fotogramma), restart dopo la sconfitta.

use crate::common::{config, despawn_all, enemy_positions, god, player_position, quiet_config};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::gameplay::combat::Health;
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::enemies::{Enemy, enemy_bundle};
use pixel_monster_survivor::gameplay::player::Player;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::headless_app_with;

/// Run senza nemici nuovi né compagno, con `count` nemici della prima ondata fermi a contatto
/// attorno al player (creati qui: non dipende da `initial_burst`).
fn surrounded(count: usize) -> App {
    let mut app = headless_app_with(1, quiet_config());
    app.update();
    despawn_all::<Companion>(&mut app);
    despawn_all::<Enemy>(&mut app);
    let config = config(&app);
    let data = config.first_wave_enemy();
    let world = app.world_mut();
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let at = Vec2::from_angle(angle) * data.contact_distance;
        let enemy = world.spawn(enemy_bundle(data, at, i as u32)).id();
        world.get_mut::<Enemy>(enemy).unwrap().speed = 0.0;
    }
    app
}

fn player_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .unwrap()
}

fn player_hp(app: &mut App) -> f32 {
    let player = player_entity(app);
    app.world().get::<Health>(player).unwrap().current
}

/// HP del player tali che il prossimo colpo di `touching` nemici sia mortale.
fn one_hit_from_death(app: &mut App) {
    let damage = config(app).first_wave_enemy().contact_damage;
    let player = player_entity(app);
    app.world_mut().get_mut::<Health>(player).unwrap().current = damage * 0.5;
}

fn phase(app: &App) -> RunPhase {
    *app.world().resource::<State<RunPhase>>().get()
}

fn virtual_paused(app: &App) -> bool {
    app.world().resource::<Time<Virtual>>().is_paused()
}

#[test]
fn contact_hurts_once_per_invulnerability_window() {
    let mut app = surrounded(1);
    let config = config(&app);
    let (max, damage) = (
        config.player.max_hp,
        config.first_wave_enemy().contact_damage,
    );
    let window = (config.player.invulnerability_s * 60.0).round() as usize;
    app.update();
    assert_eq!(player_hp(&mut app), max - damage);
    // Durante la finestra di invulnerabilità niente danni...
    for _ in 0..window - 3 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max - damage);
    // ...poi il nemico, ancora a contatto, colpisce di nuovo.
    for _ in 0..6 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max - 2.0 * damage);
}

#[test]
fn damage_from_several_touching_enemies_adds_up() {
    let mut app = surrounded(3);
    let config = config(&app);
    let (max, damage) = (
        config.player.max_hp,
        config.first_wave_enemy().contact_damage,
    );
    app.update();
    assert_eq!(player_hp(&mut app), max - 3.0 * damage);
}

#[test]
fn enemies_out_of_contact_do_no_damage() {
    let mut app = surrounded(0);
    let max = config(&app).player.max_hp;
    let config = config(&app);
    let data = config.first_wave_enemy();
    let far = Vec2::new(data.contact_distance * 6.0, 0.0);
    let world = app.world_mut();
    let enemy = world.spawn(enemy_bundle(data, far, 0)).id();
    world.get_mut::<Enemy>(enemy).unwrap().speed = 0.0;
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max);
}

#[test]
fn god_mode_takes_no_damage() {
    let mut app = surrounded(3);
    god(&mut app);
    let max = config(&app).player.max_hp;
    for _ in 0..120 {
        app.update();
    }
    assert_eq!(player_hp(&mut app), max);
}

#[test]
fn the_run_is_lost_at_zero_hp_and_the_world_freezes() {
    let mut app = surrounded(2);
    one_hit_from_death(&mut app);
    app.update(); // colpo mortale: la fase cambia al frame successivo
    app.update();
    assert_eq!(phase(&app), RunPhase::Defeated);
    assert!(
        virtual_paused(&app),
        "da sconfitti il tempo è fermo (niente tremolio)"
    );
    let clock = app.world().resource::<RunClock>().elapsed;
    let (player, enemies) = (player_position(&mut app), enemy_positions(&mut app));
    for _ in 0..60 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<RunClock>().elapsed,
        clock,
        "orologio fermo"
    );
    assert_eq!(player_position(&mut app), player);
    assert_eq!(enemy_positions(&mut app), enemies);
}

/// Rilievo del code review P0.3: con il tempo accelerato (più passi per fotogramma) la
/// simulazione continuava dopo il colpo mortale; ora il passo della morte è l'ultimo.
#[test]
fn no_simulation_step_runs_after_the_killing_one() {
    let mut app = surrounded(2);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(4));
    one_hit_from_death(&mut app);
    let before = app.world().resource::<RunClock>().elapsed;
    app.update(); // 4 passi previsti: il primo uccide, gli altri 3 non girano
    let after = app.world().resource::<RunClock>().elapsed;
    assert!(
        (after - before - 1.0 / 60.0).abs() < 1e-6,
        "avanzato di {} s",
        after - before
    );
}

/// Rilievo del code review P0.3: una pausa chiesta nel fotogramma del colpo mortale (Esc,
/// perdita di focus) sovrascriveva la sconfitta e il player restava a 0 HP, immortale.
#[test]
fn a_pause_on_the_killing_frame_does_not_cancel_the_defeat() {
    let mut app = surrounded(2);
    // Simula l'input: chiede la pausa quando la fase è Playing e il player è appena morto.
    app.add_systems(
        Update,
        |phase: Res<State<RunPhase>>,
         player: Query<&Health, With<Player>>,
         mut next: ResMut<NextState<RunPhase>>| {
            if *phase.get() == RunPhase::Playing && player.iter().any(Health::is_dead) {
                next.set(RunPhase::Paused);
            }
        },
    );
    one_hit_from_death(&mut app);
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(phase(&app), RunPhase::Defeated);
}

#[test]
fn restarting_after_a_defeat_gives_a_fresh_run() {
    let mut app = surrounded(3);
    one_hit_from_death(&mut app);
    app.update();
    app.update();
    assert_eq!(phase(&app), RunPhase::Defeated);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    app.update();
    app.update();
    assert_eq!(phase(&app), RunPhase::Playing);
    assert!(!virtual_paused(&app), "il tempo riparte con la run nuova");
    assert_eq!(player_hp(&mut app), config(&app).player.max_hp);
    assert_eq!(app.world().resource::<RunClock>().elapsed, 1.0 / 60.0);
}
