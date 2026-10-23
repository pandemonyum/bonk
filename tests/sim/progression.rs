//! P0.4 headless: gemme XP (una per nemico, attrazione, raccolta, limite a terra), livelli e
//! draft che ferma la run (anche con pausa, morte o restart nello stesso momento). Gli effetti
//! delle carte sono in `upgrades.rs`. Distanze, XP e numero di carte vengono dai dati veri.

use crate::common::{
    config, count, despawn_all, enemy_at, gems, give_levels, in_companion_range, phase, player_mut,
    player_position, progression, run_steps, take, xp_app, xp_app_with, xp_config,
};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use pixel_monster_survivor::app::{AppState, RunPhase, restart_run};
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::gameplay::combat::{Health, Hurtbox, Projectile, RunStats};
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::damage::GodMode;
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::kinematics::{Position, PreviousPosition, body};
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::gameplay::progression::XpGem;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::gameplay::upgrades::{
    ChooseUpgrade, Draft, PartyView, UpgradeStacks, is_eligible,
};

/// Un punto fuori dal raggio di raccolta, con margine.
fn out_of_reach(app: &App) -> f32 {
    config(app).player.pickup_radius * 3.0 + 50.0
}

#[test]
fn a_defeated_enemy_drops_exactly_one_gem_with_its_xp() {
    let mut app = xp_app();
    despawn_all::<Companion>(&mut app);
    let far = player_position(&mut app) + Vec2::X * out_of_reach(&app);
    let enemy = enemy_at(&mut app, far, 1.0, 0);
    let world = app.world();
    let center = world
        .get::<Hurtbox>(enemy)
        .unwrap()
        .center(world.get::<Position>(enemy).unwrap().0);
    // due proiettili letali nello stesso passo: un solo morto, una sola gemma
    for _ in 0..2 {
        app.world_mut().spawn((
            Projectile {
                velocity: Vec2::ZERO,
                damage: 5.0,
                radius: 3.0,
                remaining_secs: 1.0,
                hits_left: 1,
                already_hit: Vec::new(),
            },
            body(center),
        ));
    }
    run_steps(&mut app, 1);
    let xp = config(&app).first_wave_enemy().xp;
    assert_eq!(gems(&mut app), [(far, xp)]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    run_steps(&mut app, 30); // lontana dal player: resta dov'è
    assert_eq!(gems(&mut app), [(far, xp)]);
    assert_eq!(app.world().resource::<RunStats>().defeated, 1);
}

#[test]
fn gems_in_the_pickup_radius_fly_to_the_player_and_give_xp() {
    let mut app = xp_app();
    let c = config(&app);
    let radius = c.player.pickup_radius;
    let from = player_position(&mut app);
    let far = from - Vec2::X * (radius + 10.0);
    app.world_mut()
        .spawn((XpGem { value: 1 }, body(from + Vec2::X * (radius - 1.0))));
    app.world_mut().spawn((XpGem { value: 1 }, body(far)));
    // tempo di volo alla velocità del magnete, più un passo di margine
    let steps = (radius / c.progression.gem.magnet_speed * 60.0).ceil() as usize + 1;
    run_steps(&mut app, steps);
    assert_eq!(progression(&app).total_xp, 1);
    assert_eq!(gems(&mut app), [(far, 1)]);
}

/// Rilievo del code review P0.4: la versione precedente restava sempre dentro il raggio e
/// passava anche se l'attrazione non durava. Qui il player esce dal raggio dopo l'aggancio.
#[test]
fn an_attracted_gem_keeps_chasing_a_player_who_leaves_the_radius() {
    let mut app = xp_app();
    let radius = config(&app).player.pickup_radius;
    let from = player_position(&mut app);
    app.world_mut()
        .spawn((XpGem { value: 1 }, body(from - Vec2::X * (radius - 1.0))));
    run_steps(&mut app, 1); // attirata
    let away = from + Vec2::X * radius * 2.0;
    let mut player = player_mut(&mut app);
    player.get_mut::<Position>().unwrap().0 = away;
    player.get_mut::<PreviousPosition>().unwrap().0 = away;
    let gem = gems(&mut app)[0].0;
    assert!(
        gem.distance(away) > radius,
        "il player deve essere fuori dal raggio"
    );
    app.insert_resource(MoveIntent(Vec2::X));
    run_steps(&mut app, 120);
    assert!(
        gems(&mut app).is_empty(),
        "la gemma attirata non l'ha raggiunto"
    );
    assert_eq!(progression(&app).total_xp, 1);
}

#[test]
fn beyond_the_cap_new_xp_merges_into_gems_on_the_field() {
    let mut config = xp_config();
    config.progression.gem.max_on_field = 3;
    let mut app = xp_app_with(config);
    despawn_all::<Companion>(&mut app);
    let far = out_of_reach(&app);
    for i in 0..10 {
        let at = Vec2::new(far + 20.0 * i as f32, far);
        enemy_at(&mut app, at, 0.0, i);
    }
    run_steps(&mut app, 1);
    for i in 0..4 {
        let at = Vec2::new(-far, -far - 20.0 * i as f32);
        enemy_at(&mut app, at, 0.0, 10 + i);
    }
    run_steps(&mut app, 1);
    let on_field = gems(&mut app);
    let xp = app.world().resource::<GameConfig>().first_wave_enemy().xp;
    assert_eq!(on_field.len(), 3);
    assert_eq!(
        on_field.iter().map(|g| g.1).sum::<u32>(),
        14 * xp,
        "XP persa"
    );
}

#[test]
fn a_level_up_freezes_the_run_on_a_draft_until_a_card_is_chosen() {
    let mut app = xp_app();
    let target = in_companion_range(&mut app);
    enemy_at(&mut app, target, 1.0e6, 0);
    run_steps(&mut app, 120); // il compagno ha un bersaglio e spara
    let c = config(&app);
    give_levels(&mut app, 1);
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::LevelUp);
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    let p = progression(&app);
    assert_eq!((p.level, p.pending_levels), (2, 1));
    // tante carte quante ne chiede il draft, se ce ne sono abbastanza disponibili
    let stacks = app.world().resource::<UpgradeStacks>().clone();
    let weights = c.progression.rarity_weights;
    let available = (0..c.upgrades.len())
        .filter(|&i| {
            let party = PartyView::of(&c, &stacks, &c.stage.starting_companion);
            is_eligible(&c.upgrades, &stacks, i, &party) && weights.of(c.upgrades[i].rarity) > 0.0
        })
        .count();
    let choices = app.world().resource::<Draft>().choices.clone();
    let draft_size = c.progression.draft_size as usize;
    assert_eq!(choices.len(), draft_size.min(available));
    let clock = app.world().resource::<RunClock>().elapsed;
    let shots = app.world().resource::<RunStats>().projectiles_fired;
    assert!(shots > 0, "senza colpi il controllo sotto non prova nulla");
    run_steps(&mut app, 120);
    assert_eq!(
        app.world().resource::<RunClock>().elapsed,
        clock,
        "tempo fermo"
    );
    assert_eq!(app.world().resource::<RunStats>().projectiles_fired, shots);
    // una scelta fuori dal draft viene ignorata
    app.world_mut().write_message(ChooseUpgrade(choices.len()));
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::LevelUp);
    let last = choices.len() - 1;
    app.world_mut().write_message(ChooseUpgrade(last));
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::Playing);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    let stacks = app.world().resource::<UpgradeStacks>().clone();
    assert_eq!(stacks.of(choices[last]), 1);
    assert_eq!(stacks.0.iter().sum::<u32>(), 1);
    assert_eq!(progression(&app).pending_levels, 0);
}

/// Come per la sconfitta: col tempo accelerato il passo del livello nuovo è l'ultimo.
#[test]
fn the_level_up_step_is_the_last_one_even_with_accelerated_time() {
    let mut app = xp_app();
    give_levels(&mut app, 1);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(4));
    let before = app.world().resource::<RunClock>().elapsed;
    run_steps(&mut app, 1);
    let after = app.world().resource::<RunClock>().elapsed;
    assert!(
        (after - before - 1.0 / 60.0).abs() < 1e-6,
        "avanzato di {} s",
        after - before
    );
}

#[test]
fn several_levels_at_once_open_one_draft_each() {
    let mut app = xp_app();
    give_levels(&mut app, 2);
    run_steps(&mut app, 2);
    assert_eq!(progression(&app).pending_levels, 2);
    app.world_mut().write_message(ChooseUpgrade(0));
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::LevelUp, "secondo draft");
    assert_eq!(progression(&app).pending_levels, 1);
    app.world_mut().write_message(ChooseUpgrade(0));
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::Playing);
    assert_eq!(progression(&app).level, 3);
    let taken: u32 = app.world().resource::<UpgradeStacks>().0.iter().sum();
    assert_eq!(taken, 2);
}

/// Rilievo del code review P0.4: due scelte nello stesso fotogramma (tasto e clic) facevano
/// scegliere la seconda nel draft successivo, prima che il giocatore lo vedesse.
#[test]
fn extra_choices_in_the_same_frame_are_dropped() {
    let mut app = xp_app();
    give_levels(&mut app, 2);
    run_steps(&mut app, 2);
    app.world_mut().write_message(ChooseUpgrade(0));
    app.world_mut().write_message(ChooseUpgrade(0));
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::LevelUp, "il secondo draft aspetta");
    assert_eq!(progression(&app).pending_levels, 1);
}

/// Rilievo del code review P0.4: una pausa nel fotogramma del livello nuovo (vince l'ultimo
/// `NextState`) scartava il draft già estratto e ne estraeva un altro alla ripresa. Ora le
/// carte si estraggono entrando nel draft: con o senza pausa lo stesso seed dà le stesse carte.
#[test]
fn a_pause_on_the_level_up_frame_does_not_change_the_cards() {
    let draft_after = |pause: bool| {
        let mut app = xp_app();
        give_levels(&mut app, 1);
        run_steps(&mut app, 1); // passo del livello: il draft è chiesto
        if pause {
            let mut next = app.world_mut().resource_mut::<NextState<RunPhase>>();
            next.set(RunPhase::Paused);
            run_steps(&mut app, 3);
            assert_eq!(phase(&app), RunPhase::Paused);
            let mut next = app.world_mut().resource_mut::<NextState<RunPhase>>();
            next.set(RunPhase::Playing);
            run_steps(&mut app, 1);
        }
        run_steps(&mut app, 2);
        assert_eq!(phase(&app), RunPhase::LevelUp);
        app.world().resource::<Draft>().choices.clone()
    };
    assert_eq!(draft_after(true), draft_after(false));
}

#[test]
fn dying_on_the_level_up_step_ends_the_run_without_a_draft() {
    let mut app = xp_app();
    app.insert_resource(GodMode(false));
    despawn_all::<Companion>(&mut app);
    let data = config(&app).first_wave_enemy().clone();
    let touching = player_position(&mut app) + Vec2::X * data.contact_distance;
    enemy_at(&mut app, touching, 1.0e6, 0);
    player_mut(&mut app).get_mut::<Health>().unwrap().current = data.contact_damage * 0.5;
    give_levels(&mut app, 1);
    run_steps(&mut app, 3);
    let p = progression(&app);
    assert_eq!(
        (p.level, p.pending_levels),
        (2, 1),
        "livello preso nel passo della morte"
    );
    assert_eq!(phase(&app), RunPhase::Defeated);
    assert!(app.world().resource::<Draft>().choices.is_empty());
}

#[test]
fn restarting_from_the_draft_gives_a_fresh_running_game() {
    let mut app = xp_app();
    give_levels(&mut app, 1);
    run_steps(&mut app, 2);
    assert_eq!(phase(&app), RunPhase::LevelUp);
    restart_run(&mut app.world_mut().resource_mut::<NextState<AppState>>());
    run_steps(&mut app, 3);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InRun
    );
    assert_eq!(phase(&app), RunPhase::Playing);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert_eq!(progression(&app).level, 1);
    assert!(app.world().resource::<Draft>().choices.is_empty());
    let clock = app.world().resource::<RunClock>().elapsed;
    run_steps(&mut app, 10);
    assert!(app.world().resource::<RunClock>().elapsed > clock);
}

/// Senza upgrade disponibili il livello passa senza fermare la run.
#[test]
fn with_no_card_left_a_level_passes_without_a_draft() {
    let mut config = xp_config();
    config.upgrades.truncate(1);
    config.upgrades[0].requires.clear();
    config.upgrades[0].max_stack = 1;
    let mut app = xp_app_with(config);
    take(&mut app, 0);
    give_levels(&mut app, 1);
    run_steps(&mut app, 3);
    assert_eq!(phase(&app), RunPhase::Playing);
    let p = progression(&app);
    assert_eq!((p.level, p.pending_levels), (3, 0));
    let clock = app.world().resource::<RunClock>().elapsed;
    run_steps(&mut app, 10);
    assert!(app.world().resource::<RunClock>().elapsed > clock);
}
