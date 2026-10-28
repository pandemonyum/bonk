//! Riepilogo di fine run: ogni nemico finito e ogni punto di danno vanno al compagno che li ha
//! fatti, con tutti e quattro i pattern; le cure di carte e rigenerazione si contano a parte,
//! senza l'eccesso oltre gli HP massimi.

use crate::common::{
    app_starting_with, enemy_at, player_mut, player_position, run_steps, take, upgrade_index,
};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::Effect;
use pixel_monster_survivor::gameplay::combat::{Health, RunStats, Source};
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::upgrades::AutoPick;

const HP: f32 = 40.0;

#[test]
fn every_kill_and_point_of_damage_goes_to_the_companion_that_did_it() {
    let ids: Vec<String> = GameConfig::embedded()
        .companions
        .iter()
        .map(|c| c.id.clone())
        .collect();
    for (index, id) in ids.iter().enumerate() {
        let mut app = app_starting_with(id, |_| {});
        // anello stretto attorno al player: lo raggiungono proiettili, onda, foglie e fulmine
        let center = player_position(&mut app);
        let count = 8;
        for k in 0..count {
            let angle = k as f32 / count as f32 * std::f32::consts::TAU;
            let at = center + Vec2::from_angle(angle) * (30.0 + 4.0 * k as f32);
            enemy_at(&mut app, at, HP, 900 + k);
        }
        run_steps(&mut app, 20 * 60);
        let stats = app.world().resource::<RunStats>().clone();
        let world = app.world_mut();
        let alive: Vec<f32> = world
            .query_filtered::<&Health, With<Enemy>>()
            .iter(world)
            .map(|h| h.current)
            .collect();
        let lost: f32 = alive.iter().map(|hp| HP - hp.max(0.0)).sum::<f32>()
            + (count as usize - alive.len()) as f32 * HP;
        let tally = stats.tally(Source(index));
        assert!(stats.defeated > 0, "{id}: nessun nemico finito in 20 s");
        assert_eq!(tally.kills, stats.defeated, "{id}: sconfitti");
        assert!(
            (tally.damage - lost).abs() < 0.01,
            "{id}: {} danni accreditati, {lost} HP tolti",
            tally.damage
        );
        let others: f32 = (0..ids.len())
            .filter(|&i| i != index)
            .map(|i| stats.tally(Source(i)).damage)
            .sum();
        assert_eq!(others, 0.0, "{id}: danni accreditati ad altri");
    }
}

#[test]
fn card_healing_counts_only_what_reaches_max_hp() {
    let data = GameConfig::embedded();
    let mut app = app_starting_with(&data.stage.starting_companion, |_| {});
    // le carte le sceglie il test, non l'autoplay
    app.world_mut().remove_resource::<AutoPick>();
    let snack = upgrade_index(&app, |e| matches!(e, Effect::Heal(_)));
    let Effect::Heal(amount) = data.upgrades[snack].effect else {
        unreachable!()
    };
    let max = player_mut(&mut app).get::<Health>().unwrap().max;
    // mancano meno HP di quanti ne cura la carta: conta solo quello che manca
    let missing = amount * 0.5;
    player_mut(&mut app).get_mut::<Health>().unwrap().current = max - missing;
    take(&mut app, snack);
    let healed = app.world().resource::<RunStats>().healing;
    assert!(
        (healed - missing).abs() < 1e-3,
        "{healed} curati, {missing} mancanti"
    );
    // la rigenerazione si somma secondo per secondo
    let regen = upgrade_index(&app, |e| matches!(e, Effect::RegenPerSecond(_)));
    let Effect::RegenPerSecond(per_s) = data.upgrades[regen].effect else {
        unreachable!()
    };
    player_mut(&mut app).get_mut::<Health>().unwrap().current = max * 0.5;
    take(&mut app, regen);
    let before = app.world().resource::<RunStats>().healing;
    run_steps(&mut app, 10 * 60);
    let regenerated = app.world().resource::<RunStats>().healing - before;
    assert!(
        (regenerated - per_s * 10.0).abs() < per_s * 0.1,
        "{regenerated} rigenerati in 10 s, attesi {}",
        per_s * 10.0
    );
}
