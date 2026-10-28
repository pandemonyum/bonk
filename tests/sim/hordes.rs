//! Orde: all'istante dei dati nasce tutta la massa annunciata, tutta attorno al player e fuori
//! dalla vista, entro la sua durata; stessa seed, stessa orda. Senza ondate né compagni che
//! sparano, per contare solo i nemici dell'orda.

use crate::common::{clock, despawn_all, enemy_positions, god, player_position, quiet_config};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::gameplay::attacks::AutoAttack;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::hordes::HordeState;
use pixel_monster_survivor::headless_app_with;

const AT_S: f32 = 3.0;

/// I dati veri con la prima orda anticipata a `AT_S` e senza ondate continue.
fn horde_app(seed: u64) -> (App, u32) {
    let mut config = quiet_config();
    let hordes = &mut config.stage.run.hordes;
    hordes.warning_s = 1.0;
    hordes.list.truncate(1);
    hordes.list[0].at_s = AT_S;
    let total = hordes.list[0].total();
    let mut app = headless_app_with(seed, config);
    god(&mut app);
    app.update();
    despawn_all::<Enemy>(&mut app);
    // nessuno colpisce: i nemici dell'orda restano tutti in campo
    despawn_all::<AutoAttack>(&mut app);
    (app, total)
}

fn run_until(app: &mut App, t: f32) {
    while clock(app) < t {
        app.update();
    }
}

#[test]
fn the_horde_surrounds_the_player_off_screen_within_its_duration() {
    let (mut app, total) = horde_app(11);
    let duration = GameConfig::embedded().stage.run.hordes.duration_s;
    run_until(&mut app, AT_S - 0.1);
    assert!(enemy_positions(&mut app).is_empty(), "prima dell'orda");
    run_until(&mut app, AT_S + duration * 0.5);
    let half = enemy_positions(&mut app).len() as u32;
    assert!(
        half > total / 4 && half < total,
        "a metà durata {half} su {total}"
    );
    run_until(&mut app, AT_S + duration + 0.2);
    let player = player_position(&mut app);
    let enemies = enemy_positions(&mut app);
    // qualche posto può cadere su un ostacolo: niente nemico, ma quasi tutti nascono
    assert!(
        enemies.len() as u32 >= total * 9 / 10,
        "{} nati su {total}",
        enemies.len()
    );
    assert!(app.world().resource::<HordeState>().active.is_none());
    // tutto attorno: nemici in ognuno degli otto spicchi
    let mut sectors = [0; 8];
    for at in &enemies {
        let turn = (*at - player).to_angle().rem_euclid(std::f32::consts::TAU);
        sectors[(turn / std::f32::consts::TAU * 8.0) as usize % 8] += 1;
    }
    assert!(sectors.iter().all(|&n| n > 0), "spicchi {sectors:?}");
}

#[test]
fn the_same_seed_gives_the_same_horde() {
    let positions = |seed| {
        let (mut app, _) = horde_app(seed);
        run_until(&mut app, AT_S + 1.0);
        enemy_positions(&mut app)
    };
    let first = positions(5);
    assert!(!first.is_empty());
    assert_eq!(first, positions(5));
    assert_ne!(first, positions(6));
}
