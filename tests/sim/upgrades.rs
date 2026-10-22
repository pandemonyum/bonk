//! P0.4 headless: effetti delle carte applicati a caldo (danno, proiettili, HP, cura,
//! rigenerazione, velocità, raccolta), restart e scelta automatica. Gli upgrade si cercano per
//! tipo di effetto e i valori attesi si ricavano dai dati veri.

use crate::common::{
    companion_attack, config, count, despawn_all, enemy_at, give_levels, god, in_companion_range,
    phase, player_mut, player_position, progression, run_steps, take, upgrade_index, xp_app,
};
use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::data::schema::Effect;
use pixel_monster_survivor::gameplay::combat::{Health, Projectile, RunStats};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::progression::PickupRadius;
use pixel_monster_survivor::gameplay::upgrades::UpgradeStacks;
use pixel_monster_survivor::headless_app;

#[test]
fn the_damage_upgrade_raises_the_damage_of_every_hit() {
    let mut app = xp_app();
    let index = upgrade_index(&app, |e| matches!(e, Effect::CompanionDamagePct(_)));
    let Effect::CompanionDamagePct(pct) = config(&app).upgrades[index].effect else {
        unreachable!()
    };
    let base = companion_attack(&mut app).damage;
    take(&mut app, index);
    let expected = base * (1.0 + pct / 100.0);
    assert!((companion_attack(&mut app).damage - expected).abs() < 1e-4);
    // il colpo vero toglie il danno nuovo
    let near = in_companion_range(&mut app);
    let target = enemy_at(&mut app, near, 1.0e6, 0);
    for _ in 0..200 {
        run_steps(&mut app, 1);
        let hp = app.world().get::<Health>(target).unwrap().current;
        if hp < 1.0e6 {
            assert!((1.0e6 - hp - expected).abs() < 0.1, "danno {}", 1.0e6 - hp);
            return;
        }
    }
    panic!("nessun colpo in 200 passi");
}

#[test]
fn the_multishot_upgrade_fires_one_more_projectile_per_attack() {
    let mut app = xp_app();
    let index = upgrade_index(&app, |e| matches!(e, Effect::CompanionProjectiles(_)));
    let c = config(&app);
    let Effect::CompanionProjectiles(extra) = c.upgrades[index].effect else {
        unreachable!()
    };
    for required in &c.upgrades[index].requires {
        let r = c.upgrades.iter().position(|u| u.id == *required).unwrap();
        take(&mut app, r);
    }
    take(&mut app, index);
    let base = c
        .companion_attack(c.starting_companion())
        .projectile()
        .expect("compagno iniziale a proiettili")
        .count;
    assert_eq!(companion_attack(&mut app).projectiles, base + extra);
    let near = in_companion_range(&mut app);
    enemy_at(&mut app, near, 1.0e6, 0);
    despawn_all::<Projectile>(&mut app);
    let fired = app.world().resource::<RunStats>().projectiles_fired;
    for _ in 0..200 {
        run_steps(&mut app, 1);
        if app.world().resource::<RunStats>().projectiles_fired > fired {
            break;
        }
    }
    let burst = app.world().resource::<RunStats>().projectiles_fired - fired;
    assert_eq!(burst, base + extra);
    assert_eq!(
        count::<Projectile>(&mut app) as u32,
        base + extra,
        "visibili in campo"
    );
}

#[test]
fn health_upgrades_raise_the_maximum_heal_and_regenerate() {
    let mut app = xp_app();
    let c = config(&app);
    let max_hp = upgrade_index(&app, |e| matches!(e, Effect::MaxHp(_)));
    let heal = upgrade_index(&app, |e| matches!(e, Effect::Heal(_)));
    let regen = upgrade_index(&app, |e| matches!(e, Effect::RegenPerSecond(_)));
    let (Effect::MaxHp(bonus), Effect::Heal(amount), Effect::RegenPerSecond(per_s)) = (
        c.upgrades[max_hp].effect.clone(),
        c.upgrades[heal].effect.clone(),
        c.upgrades[regen].effect.clone(),
    ) else {
        unreachable!()
    };
    // HP bassi rispetto al massimo, ricavati dai dati: c'è spazio per cura e rigenerazione
    let low = c.player.max_hp * 0.1;
    player_mut(&mut app).get_mut::<Health>().unwrap().current = low;
    take(&mut app, max_hp);
    let h = *player_mut(&mut app).get::<Health>().unwrap();
    assert_eq!(h.max, c.player.max_hp + bonus);
    assert!(
        (h.current - (low + bonus)).abs() < 1e-4,
        "cura degli HP in più: {h:?}"
    );
    take(&mut app, heal);
    let after_heal = player_mut(&mut app).get::<Health>().unwrap().current;
    assert!((after_heal - (low + bonus + amount).min(h.max)).abs() < 1e-4);
    player_mut(&mut app).get_mut::<Health>().unwrap().current = low;
    take(&mut app, regen);
    let before = player_mut(&mut app).get::<Health>().unwrap().current;
    run_steps(&mut app, 120);
    let after = player_mut(&mut app).get::<Health>().unwrap().current;
    let expected = (before + 2.0 * per_s).min(h.max);
    assert!((after - expected).abs() < 0.05, "{before} → {after}");
}

#[test]
fn movement_and_magnet_upgrades_change_the_player() {
    let mut app = xp_app();
    let c = config(&app);
    let swift = upgrade_index(&app, |e| matches!(e, Effect::MoveSpeedPct(_)));
    let magnet = upgrade_index(&app, |e| matches!(e, Effect::PickupRadiusPct(_)));
    let (Effect::MoveSpeedPct(speed_pct), Effect::PickupRadiusPct(radius_pct)) = (
        c.upgrades[swift].effect.clone(),
        c.upgrades[magnet].effect.clone(),
    ) else {
        unreachable!()
    };
    take(&mut app, swift);
    take(&mut app, magnet);
    let radius = player_mut(&mut app).get::<PickupRadius>().unwrap().0;
    assert!((radius - c.player.pickup_radius * (1.0 + radius_pct / 100.0)).abs() < 1e-4);
    despawn_all::<Companion>(&mut app);
    let from = player_position(&mut app);
    app.insert_resource(MoveIntent(Vec2::Y));
    run_steps(&mut app, 60);
    let moved = player_position(&mut app).distance(from);
    let expected = c.player.speed * (1.0 + speed_pct / 100.0);
    assert!(
        (moved - expected).abs() < 0.5,
        "{moved} px in 1 s, attesi {expected}"
    );
}

#[test]
fn restarting_resets_levels_and_upgrades() {
    let mut app = xp_app();
    let damage = upgrade_index(&app, |e| matches!(e, Effect::CompanionDamagePct(_)));
    let base = companion_attack(&mut app);
    take(&mut app, damage);
    assert_ne!(companion_attack(&mut app).damage, base.damage);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );
    assert_eq!(progression(&app).level, 1);
    assert!(
        app.world()
            .resource::<UpgradeStacks>()
            .0
            .iter()
            .all(|&n| n == 0)
    );
    assert_eq!(companion_attack(&mut app).damage, base.damage);
}

/// Con la scelta automatica (default headless) una run sale di livello e continua: ogni
/// livello è uno stack e la run torna sempre a `Playing`. I primi livelli sono dati da una
/// gemma, così il test non dipende dal bilanciamento; il resto viene dal gioco vero.
#[test]
fn with_auto_pick_the_run_levels_up_and_keeps_going() {
    let mut app = headless_app(3);
    god(&mut app);
    app.update();
    give_levels(&mut app, 2);
    run_steps(&mut app, 60 * 60);
    let p = progression(&app);
    assert!(p.level >= 3, "livello {}", p.level);
    let taken: u32 = app.world().resource::<UpgradeStacks>().0.iter().sum();
    assert_eq!(taken + p.pending_levels, p.level - 1);
    let back_to_playing = (0..4).any(|_| {
        run_steps(&mut app, 1);
        phase(&app) == RunPhase::Playing
    });
    assert!(back_to_playing, "la run resta ferma in {:?}", phase(&app));
}
