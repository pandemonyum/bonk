//! Monete e forzieri: l'élite lascia le sue monete e la calamita le porta in tasca; un forziere
//! si apre solo con abbastanza monete, ferma la run sul draft delle reliquie, costa di più a
//! ogni apertura e lascia il posto a un forziere nuovo; uno rimasto troppo lontano rinasce
//! vicino. Le reliquie non escono mai nel draft dei livelli.

use crate::common::{
    config, count, despawn_all, enemy_at, phase, player_mut, player_position, run_steps, xp_app,
};
use bevy::prelude::*;
use pixel_monster_survivor::app::RunPhase;
use pixel_monster_survivor::data::DataId;
use pixel_monster_survivor::data::schema::Rarity;
use pixel_monster_survivor::gameplay::combat::Health;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::treasure::{Chest, Coin, Treasury};
use pixel_monster_survivor::gameplay::upgrades::{ChooseUpgrade, Draft, DraftKind, UpgradeStacks};

fn treasury(app: &App) -> Treasury {
    app.world().resource::<Treasury>().clone()
}

fn chests(app: &mut App) -> Vec<Vec2> {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<Chest>>()
        .iter(world)
        .map(|p| p.0)
        .collect()
}

fn give_coins(app: &mut App, coins: f32) {
    app.world_mut().resource_mut::<Treasury>().coins = coins;
}

/// Porta il player sopra il forziere più vicino.
fn stand_on_chest(app: &mut App) -> Vec2 {
    let me = player_position(app);
    let at = chests(app)
        .into_iter()
        .min_by(|a, b| a.distance(me).total_cmp(&b.distance(me)))
        .expect("ci sono forzieri");
    player_mut(app).get_mut::<Position>().unwrap().0 = at;
    at
}

#[test]
fn the_elite_drops_its_coins_and_the_magnet_brings_them_in() {
    let mut app = xp_app();
    let c = config(&app);
    let elite = c.enemy(&c.stage.run.elites.enemy).unwrap().clone();
    assert_eq!(elite.coins.chance, 1.0, "i dati: l'élite paga sempre");
    // un nemico qualsiasi trasformato nell'élite, a un passo dal player: muore subito
    let at = player_position(&mut app) + Vec2::X * 12.0;
    let enemy = enemy_at(&mut app, at, 1.0, 700);
    app.world_mut().get_mut::<DataId>(enemy).unwrap().0 = elite.id.clone();
    app.world_mut().get_mut::<Health>(enemy).unwrap().current = 0.0;
    run_steps(&mut app, 2);
    let on_field = count::<Coin>(&mut app) as u32;
    let collected = treasury(&app).collected;
    assert_eq!(
        on_field + collected as u32,
        elite.coins.count,
        "a terra o già prese"
    );
    run_steps(&mut app, 120);
    assert_eq!(count::<Coin>(&mut app), 0);
    assert_eq!(treasury(&app).coins, elite.coins.count as f32);
}

#[test]
fn a_chest_opens_only_with_enough_coins_and_offers_relics() {
    let mut app = xp_app();
    let c = config(&app);
    let plan = c.stage.chests.clone();
    assert_eq!(chests(&mut app).len() as u32, plan.on_map);
    // senza monete: ci si passa sopra e non succede nulla
    give_coins(&mut app, plan.price_base as f32 - 1.0);
    let first = stand_on_chest(&mut app);
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::Playing);
    assert!(chests(&mut app).contains(&first));
    // con le monete: si apre, la run si ferma sulle reliquie
    give_coins(&mut app, plan.price_base as f32 + 3.0);
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::LevelUp);
    let draft = app.world().resource::<Draft>().clone();
    assert_eq!(draft.kind, DraftKind::Chest);
    assert_eq!(draft.choices.len(), c.progression.draft_size as usize);
    assert!(
        draft
            .choices
            .iter()
            .all(|&i| c.upgrades[i].rarity == Rarity::Relic)
    );
    let t = treasury(&app);
    assert_eq!((t.coins, t.opened, t.pending), (3.0, 1, true));
    assert!(
        !chests(&mut app).contains(&first),
        "quello aperto non c'è più"
    );
    // la scelta dà la reliquia, non consuma livelli, e si torna a giocare
    let level_before = app
        .world()
        .resource::<pixel_monster_survivor::gameplay::progression::Progression>()
        .level;
    app.world_mut().write_message(ChooseUpgrade(0));
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::Playing);
    assert_eq!(
        app.world().resource::<UpgradeStacks>().of(draft.choices[0]),
        1
    );
    assert!(!treasury(&app).pending);
    assert_eq!(
        app.world()
            .resource::<pixel_monster_survivor::gameplay::progression::Progression>()
            .level,
        level_before
    );
    // ne nasce un altro, e il prossimo costa di più
    run_steps(&mut app, 2);
    assert_eq!(chests(&mut app).len() as u32, plan.on_map);
    assert_eq!(
        treasury(&app).price(&plan),
        plan.price_base + plan.price_per_open
    );
}

#[test]
fn a_chest_left_far_behind_comes_back_near() {
    let mut app = xp_app();
    let plan = config(&app).stage.chests.clone();
    despawn_all::<Chest>(&mut app);
    let me = player_position(&mut app);
    app.world_mut().spawn((
        Chest,
        pixel_monster_survivor::gameplay::kinematics::body(
            me + Vec2::X * (plan.respawn_beyond + 50.0),
        ),
    ));
    run_steps(&mut app, 3);
    let me = player_position(&mut app);
    let all = chests(&mut app);
    assert_eq!(all.len() as u32, plan.on_map, "più quello nuovo");
    for at in all {
        let d = at.distance(me);
        assert!(d <= plan.fallback_distance.1 + 1.0, "forziere a {d} px");
    }
}

#[test]
fn relics_never_appear_in_level_drafts() {
    let mut app = xp_app();
    let c = config(&app);
    for _ in 0..12 {
        crate::common::give_levels(&mut app, 1);
        run_steps(&mut app, 2);
        assert_eq!(phase(&app), RunPhase::LevelUp);
        let draft = app.world().resource::<Draft>().clone();
        assert_ne!(draft.kind, DraftKind::Chest);
        assert!(
            draft
                .choices
                .iter()
                .all(|&i| c.upgrades[i].rarity != Rarity::Relic),
            "{draft:?}"
        );
        app.world_mut().write_message(ChooseUpgrade(0));
        run_steps(&mut app, 2);
    }
}
