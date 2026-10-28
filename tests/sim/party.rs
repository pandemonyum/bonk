//! P0.5 headless: la squadra. Le carte "nuovo compagno" portano il compagno nel primo posto
//! libero con gli upgrade già presi, gli upgrade successivi valgono per tutti, a squadra piena
//! le carte non escono più, la formazione tiene i compagni separati, il restart riparte dal
//! compagno iniziale. Carte, posti e valori si ricavano dai dati.

use crate::common::{
    config, count, phase, player_position, run_steps, take, upgrade_index, xp_app, xp_app_with,
    xp_config,
};
use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::data::schema::{AttackData, Effect};
use pixel_monster_survivor::data::{DataId, GameConfig};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::orbit::OrbitAttack;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::pulse::PulseAttack;
use pixel_monster_survivor::gameplay::upgrades::{PartyView, UpgradeStacks, is_eligible};

/// Indice della carta "nuovo compagno" per il compagno con l'attacco cercato.
fn join_card(c: &GameConfig, has: impl Fn(&AttackData) -> bool) -> usize {
    c.upgrades
        .iter()
        .position(|u| match &u.effect {
            Effect::JoinCompanion(id) => c
                .companion(id)
                .and_then(|data| c.attack(&data.attack))
                .is_some_and(&has),
            _ => false,
        })
        .expect("i dati hanno una carta per questo compagno")
}

fn pct(c: &GameConfig, index: usize) -> f32 {
    match c.upgrades[index].effect {
        Effect::CompanionDamagePct(v) | Effect::CompanionRangePct(v) => v,
        _ => unreachable!(),
    }
}

/// (id, posto, posizione) di ogni compagno, in ordine di posto.
fn party(app: &mut App) -> Vec<(String, usize, Vec2)> {
    let world = app.world_mut();
    let mut members: Vec<_> = world
        .query::<(&DataId, &Companion, &Position)>()
        .iter(world)
        .map(|(id, companion, at)| (id.0.clone(), companion.slot, at.0))
        .collect();
    members.sort_by_key(|m| m.1);
    members
}

fn single<T: Component + Clone>(app: &mut App) -> T {
    let world = app.world_mut();
    world.query::<&T>().single(world).unwrap().clone()
}

#[test]
fn a_new_companion_arrives_with_the_upgrades_already_taken() {
    let mut app = xp_app();
    let c = config(&app);
    let damage = upgrade_index(&app, |e| matches!(e, Effect::CompanionDamagePct(_)));
    take(&mut app, damage);
    let join = join_card(&c, |a| a.pulse().is_some());
    take(&mut app, join);
    let members = party(&mut app);
    let Effect::JoinCompanion(joined) = &c.upgrades[join].effect else {
        unreachable!()
    };
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].0, c.stage.starting_companion);
    assert_eq!((members[1].0.as_str(), members[1].1), (joined.as_str(), 1));
    let base = c
        .attack(&c.companion(joined).unwrap().attack)
        .unwrap()
        .damage;
    let wave: PulseAttack = single(&mut app);
    let expected = base * (1.0 + pct(&c, damage) / 100.0);
    assert!(
        (wave.damage - expected).abs() < 1e-4,
        "{} invece di {expected}",
        wave.damage
    );
}

#[test]
fn upgrades_taken_later_reach_every_companion() {
    let mut app = xp_app();
    let c = config(&app);
    take(&mut app, join_card(&c, |a| a.pulse().is_some()));
    take(&mut app, join_card(&c, |a| a.orbit().is_some()));
    let (wave, leaves): (PulseAttack, OrbitAttack) = (single(&mut app), single(&mut app));
    let range = upgrade_index(&app, |e| matches!(e, Effect::CompanionRangePct(_)));
    take(&mut app, range);
    let factor = 1.0 + pct(&c, range) / 100.0;
    let (wave_after, leaves_after): (PulseAttack, OrbitAttack) =
        (single(&mut app), single(&mut app));
    assert!((wave_after.radius - wave.radius * factor).abs() < 1e-3);
    assert!((leaves_after.radius - leaves.radius * factor).abs() < 1e-3);
    // +1 proiettile = +1 foglia (i requisiti della carta vanno presi prima)
    let shot = upgrade_index(&app, |e| matches!(e, Effect::CompanionProjectiles(_)));
    let Effect::CompanionProjectiles(extra) = c.upgrades[shot].effect else {
        unreachable!()
    };
    for required in &c.upgrades[shot].requires {
        take(
            &mut app,
            c.upgrades.iter().position(|u| u.id == *required).unwrap(),
        );
    }
    take(&mut app, shot);
    let leaves_now: OrbitAttack = single(&mut app);
    assert_eq!(leaves_now.leaves, leaves.leaves + extra);
    run_steps(&mut app, 2);
    let world = app.world_mut();
    let on_field = world
        .query::<&pixel_monster_survivor::gameplay::orbit::Leaf>()
        .iter(world)
        .count();
    assert_eq!(on_field as u32, leaves_now.leaves, "foglie in campo");
}

#[test]
fn a_full_party_gets_no_more_companion_cards() {
    // due soli posti: dopo il primo compagno arrivato l'altra carta non esce più
    let mut config = xp_config();
    config.player.companion_slots.truncate(2);
    let mut app = xp_app_with(config);
    let c = crate::common::config(&app);
    let wave = join_card(&c, |a| a.pulse().is_some());
    let leaves = join_card(&c, |a| a.orbit().is_some());
    let fresh = UpgradeStacks(vec![0; c.upgrades.len()]);
    let party = PartyView::of(&c, &fresh, &c.stage.starting_companion);
    assert!(is_eligible(&c.upgrades, &fresh, leaves, &party));
    take(&mut app, wave);
    let stacks = app.world().resource::<UpgradeStacks>().clone();
    let party = PartyView::of(&c, &stacks, &c.stage.starting_companion);
    assert_eq!(party.free_slots, 0);
    assert!(!is_eligible(&c.upgrades, &stacks, leaves, &party));
    assert!(!is_eligible(&c.upgrades, &stacks, wave, &party));
    assert_eq!(count::<Companion>(&mut app), 2);
}

#[test]
fn companions_keep_separate_places_in_the_formation() {
    let mut app = xp_app();
    let c = config(&app);
    take(&mut app, join_card(&c, |a| a.pulse().is_some()));
    take(&mut app, join_card(&c, |a| a.orbit().is_some()));
    for direction in [Vec2::X, Vec2::new(-0.6, 0.8), Vec2::ZERO] {
        app.insert_resource(MoveIntent(direction.normalize_or_zero()));
        run_steps(&mut app, 120);
    }
    let members = party(&mut app);
    assert_eq!(members.iter().map(|m| m.1).collect::<Vec<_>>(), [0, 1, 2]);
    let player = player_position(&mut app);
    for (id, _, at) in &members {
        let distance = c.companion(id).unwrap().follow.distance;
        assert!(
            (at.distance(player) - distance).abs() < 1.0,
            "{id} a {} px dal player, atteso {distance}",
            at.distance(player)
        );
    }
    for (i, a) in members.iter().enumerate() {
        for b in &members[i + 1..] {
            assert!(
                a.2.distance(b.2) > 8.0,
                "{} e {} si sovrappongono",
                a.0,
                b.0
            );
        }
    }
}

/// Rilievo del code review P0.5: un compagno arrivato mentre il player andava verso ovest
/// nasceva davanti a lui (direzione di marcia fissa) e poi lo attraversava.
#[test]
fn a_new_companion_starts_behind_the_player() {
    let mut app = xp_app();
    let c = config(&app);
    app.insert_resource(MoveIntent(Vec2::NEG_X));
    run_steps(&mut app, 60);
    app.insert_resource(MoveIntent(Vec2::ZERO));
    take(&mut app, join_card(&c, |a| a.pulse().is_some()));
    let player = player_position(&mut app);
    let world = app.world_mut();
    let newcomer = world
        .query::<(&Companion, &Position)>()
        .iter(world)
        .find(|(companion, _)| companion.slot == 1)
        .map(|(companion, at)| (*companion, at.0))
        .expect("compagno nel posto 1");
    assert_eq!(
        newcomer.0.trail,
        Vec2::NEG_X,
        "stessa direzione della squadra"
    );
    assert!(
        newcomer.1.x > player.x,
        "dietro (a est) di chi va verso ovest: {newcomer:?}"
    );
}

#[test]
fn restarting_brings_back_only_the_starting_companion() {
    let mut app = xp_app();
    let c = config(&app);
    take(&mut app, join_card(&c, |a| a.pulse().is_some()));
    take(&mut app, join_card(&c, |a| a.orbit().is_some()));
    assert_eq!(count::<Companion>(&mut app), 3);
    // in campo anche foglie, un'onda e un nemico rallentato
    let near = player_position(&mut app) + Vec2::X * 20.0;
    crate::common::enemy_at(&mut app, near, 1.0e6, 0);
    let waves = (0..240).any(|_| {
        run_steps(&mut app, 1);
        count::<PulseAttack>(&mut app) > 0
            && count::<pixel_monster_survivor::gameplay::pulse::Pulse>(&mut app) > 0
    });
    assert!(waves, "nessuna onda partita");
    assert!(count::<pixel_monster_survivor::gameplay::orbit::Leaf>(&mut app) > 0);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::Playing);
    let members = party(&mut app);
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].0, c.stage.starting_companion);
    assert_eq!(count::<PulseAttack>(&mut app), 0);
    assert_eq!(
        count::<pixel_monster_survivor::gameplay::pulse::Pulse>(&mut app),
        0
    );
    assert_eq!(
        count::<pixel_monster_survivor::gameplay::orbit::Leaf>(&mut app),
        0
    );
    assert_eq!(
        count::<pixel_monster_survivor::gameplay::pulse::Slowed>(&mut app),
        0
    );
}

/// Si parte con un compagno solo; ai livelli dei compagni il draft propone soltanto chi può
/// unirsi (e lo dice), negli altri draft nessun compagno. Alla fine la squadra è piena.
#[test]
fn companions_are_chosen_only_at_the_companion_levels() {
    use crate::common::give_levels;
    use pixel_monster_survivor::gameplay::upgrades::{ChooseUpgrade, Draft};
    let mut app = xp_app();
    let c = config(&app);
    let levels = c.progression.companion_levels.clone();
    let last = *levels.last().expect("i dati hanno livelli dei compagni");
    assert_eq!(count::<Companion>(&mut app), 1, "si parte da soli");
    let is_join = |i: &usize| matches!(c.upgrades[*i].effect, Effect::JoinCompanion(_));
    for level in 2..=last + 2 {
        give_levels(&mut app, 1);
        run_steps(&mut app, 2);
        assert_eq!(phase(&app), RunPhase::LevelUp, "livello {level}");
        let draft = app.world().resource::<Draft>().clone();
        let joins = draft.choices.iter().filter(|i| is_join(i)).count();
        if levels.contains(&level) {
            assert!(draft.companions, "livello {level}");
            assert!(
                joins > 0 && joins == draft.choices.len(),
                "livello {level}: {draft:?}"
            );
        } else {
            assert!(!draft.companions, "livello {level}");
            assert_eq!(joins, 0, "livello {level}: {draft:?}");
        }
        app.world_mut().write_message(ChooseUpgrade(0));
        run_steps(&mut app, 2);
        assert_eq!(phase(&app), RunPhase::Playing);
    }
    assert_eq!(count::<Companion>(&mut app), 1 + levels.len());
    assert_eq!(count::<Companion>(&mut app), c.player.companion_slots.len());
}
