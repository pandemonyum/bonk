//! Talenti dei compagni nella run: con i nodi imparati il compagno esplode, soffia, brucia,
//! cura, ripete l'attacco, colpisce a intervalli e ha le statistiche del ramo; tutto conta come
//! suo danno. Senza talenti nulla cambia (gli altri test). Valori dai dati.

use crate::common::{
    despawn_all, enemy_at, god, player_mut, player_position, quiet_config, run_steps,
};
use bevy::prelude::*;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::gameplay::combat::{Health, RunStats, Source};
use pixel_monster_survivor::gameplay::enemies::Enemy;
use pixel_monster_survivor::gameplay::orbit::OrbitAttack;
use pixel_monster_survivor::gameplay::pulse::PulseAttack;
use pixel_monster_survivor::gameplay::talent_effects::Burning;
use pixel_monster_survivor::gameplay::talents::TalentKit;
use pixel_monster_survivor::headless_app_with;
use pixel_monster_survivor::profile::{CompanionProgress, Profile, ProfileData};

const HP: f32 = 1000.0;

/// Run tranquilla e invulnerabile con `companion` iniziale e i nodi `nodes` imparati.
fn app_with_talents(companion: &str, nodes: &[&str]) -> App {
    let mut config = quiet_config();
    config.stage.starting_companion = companion.into();
    let mut app = headless_app_with(1, config);
    god(&mut app);
    // come nel gioco: i nodi stanno nel profilo, che li passa alla run
    let learned = nodes.iter().map(|n| n.to_string()).collect();
    app.insert_resource(Profile {
        data: ProfileData {
            companions: vec![CompanionProgress {
                id: companion.to_string(),
                xp: u32::MAX / 2,
                learned,
            }],
            ..default()
        },
        path: None,
    });
    app.update();
    despawn_all::<Enemy>(&mut app);
    app
}

fn kit(app: &mut App) -> TalentKit {
    let world = app.world_mut();
    *world.query::<&TalentKit>().single(world).unwrap()
}

fn hp(app: &App, enemy: Entity) -> f32 {
    app.world().get::<Health>(enemy).map_or(0.0, |h| h.current)
}

fn index_of(id: &str) -> usize {
    GameConfig::embedded().companion_index(id).unwrap()
}

#[test]
fn fireballs_explode_on_the_neighbours_of_the_hit_enemy() {
    let mut app = app_with_talents("ember", &["ember_el_a1", "ember_el_a1", "ember_el_a1"]);
    let blast = kit(&mut app).explode.expect("il nodo dà l'esplosione");
    let me = player_position(&mut app);
    let target = enemy_at(&mut app, me + Vec2::X * 60.0, HP, 1);
    // accanto al bersaglio, dietro rispetto al player: il proiettile non lo raggiunge
    let neighbour = enemy_at(&mut app, me + Vec2::new(60.0, blast.radius * 0.6), HP, 2);
    run_steps(&mut app, 90);
    assert!(hp(&app, target) < HP);
    let lost = HP - hp(&app, neighbour);
    assert!(lost > 0.0, "il vicino prende l'esplosione");
    let tally = app
        .world()
        .resource::<RunStats>()
        .tally(Source(index_of("ember")));
    assert!((tally.damage - (2.0 * HP - hp(&app, target) - hp(&app, neighbour))).abs() < 0.01);
}

#[test]
fn the_fire_breath_comes_every_few_volleys_and_burns_come_from_direct_hits() {
    let mut app = app_with_talents(
        "ember",
        &["ember_el_a1", "ember_el_a2", "ember_el_a3", "ember_el_b1"],
    );
    let k = kit(&mut app);
    let (cone, burn) = (k.cone.unwrap(), k.burn.unwrap());
    let me = player_position(&mut app);
    let target = enemy_at(&mut app, me + Vec2::X * 40.0, HP, 1);
    run_steps(&mut app, 30);
    let burning = app
        .world()
        .get::<Burning>(target)
        .copied()
        .expect("colpito: brucia");
    assert!(burning.remaining > 0.0 && burning.remaining <= burn.duration);
    // la bruciatura continua a togliere HP anche senza colpi nuovi per un po'
    let before = hp(&app, target);
    run_steps(&mut app, 10);
    assert!(hp(&app, target) < before);
    assert!(cone.every > 1);
}

#[test]
fn the_wave_heals_the_player_up_to_its_cap_and_echoes() {
    let mut app = app_with_talents("tide", &["tide_ev_a1", "tide_ev_a2", "tide_su_a1"]);
    let k = kit(&mut app);
    let (heal, echo) = (k.heal.unwrap(), k.echo.unwrap());
    let me = player_position(&mut app);
    for i in 0..8 {
        let angle = i as f32 / 8.0 * std::f32::consts::TAU;
        enemy_at(&mut app, me + Vec2::from_angle(angle) * 30.0, HP, 10 + i);
    }
    player_mut(&mut app).get_mut::<Health>().unwrap().current = 50.0;
    // la prima onda è armata: parte al primo passo; poco prima dell'eco la cura è al tetto
    let echo_steps = (echo.delay * 60.0) as usize;
    run_steps(&mut app, echo_steps - 2);
    let healed = player_mut(&mut app).get::<Health>().unwrap().current - 50.0;
    assert!(
        (healed - heal.cap.min(8.0 * heal.hp)).abs() < 1e-3,
        "curati {healed}"
    );
    let tally = app
        .world()
        .resource::<RunStats>()
        .tally(Source(index_of("tide")));
    assert!(
        (tally.healing - healed).abs() < 1e-3,
        "cura accreditata a Ondino"
    );
    // poi arriva l'eco: altri 8 colpi, senza cura
    let hits_before = app.world().resource::<RunStats>().hits;
    run_steps(&mut app, 40);
    let hits = app.world().resource::<RunStats>().hits - hits_before;
    assert_eq!(hits, 8, "l'eco colpisce tutti una volta");
    let after = player_mut(&mut app).get::<Health>().unwrap().current - 50.0;
    assert!((after - healed).abs() < 1e-3, "l'eco non cura");
}

#[test]
fn roots_strike_the_nearest_enemy_on_their_own_rhythm() {
    let mut app = app_with_talents("sprout", &["sprout_el_b1"]);
    let strike = kit(&mut app).strike.unwrap();
    // fuori dalla portata delle foglie, dentro quella delle radici
    let me = player_position(&mut app);
    // la portata conta dal compagno, che sta ~30 px dietro al player
    let far = enemy_at(&mut app, me + Vec2::X * strike.range * 0.6, HP, 1);
    run_steps(&mut app, (strike.every * 60.0) as usize - 10);
    assert_eq!(hp(&app, far), HP, "non ancora");
    run_steps(&mut app, 20);
    let config = GameConfig::embedded();
    let leaves = config
        .attack(&config.companion("sprout").unwrap().attack)
        .unwrap();
    let expected = leaves.damage * strike.damage_pct / 100.0;
    let lost = HP - hp(&app, far);
    assert!(
        (lost - expected).abs() < 1e-3,
        "{lost} persi, attesi {expected}"
    );
}

#[test]
fn branch_stats_add_to_the_attack() {
    let config = GameConfig::embedded();
    let mut base = app_with_talents("sprout", &[]);
    let mut grown = app_with_talents(
        "sprout",
        &[
            "sprout_ev_a1",
            "sprout_ev_a2",
            "sprout_ev_a3",
            "sprout_ev_b1",
            "sprout_ev_b2",
            "sprout_ev_b3",
            "sprout_ev_c1",
            "sprout_ev_c2",
        ],
    );
    let leaves = |app: &mut App| {
        let world = app.world_mut();
        world.query::<&OrbitAttack>().single(world).unwrap().clone()
    };
    let (a, b) = (leaves(&mut base), leaves(&mut grown));
    assert_eq!(b.leaves, a.leaves + 1);
    assert!(b.radius > a.radius && b.angular_speed.abs() > a.angular_speed.abs());
    let mut tide = app_with_talents("tide", &["tide_ev_b1"]);
    let world = tide.world_mut();
    let pulse = world.query::<&PulseAttack>().single(world).unwrap().clone();
    let base_slow = config
        .attack(&config.companion("tide").unwrap().attack)
        .unwrap()
        .pulse()
        .unwrap()
        .slow;
    assert!(pulse.slow < base_slow, "Marea lenta rallenta di più");
}

#[test]
fn support_talents_help_the_player_while_the_companion_is_in_the_squad() {
    use pixel_monster_survivor::gameplay::damage::{Invulnerability, Toughness};
    let config = GameConfig::embedded();
    let full_branch = |companion: &str, theme: &str| -> Vec<String> {
        let c = config.companion(companion).unwrap();
        let branch = c.talents.iter().find(|b| b.name == theme).unwrap();
        branch
            .nodes
            .iter()
            .flat_map(|n| std::iter::repeat_n(n.id.clone(), n.max_rank() as usize))
            .collect()
    };
    // Foglin: meno danni subiti; Fiammetto: invulnerabilità più lunga
    let bark = full_branch("sprout", "Supporto");
    let bark: Vec<&str> = bark.iter().map(String::as_str).collect();
    let mut app = app_with_talents("sprout", &bark);
    run_steps(&mut app, 2);
    let k = kit(&mut app);
    assert!(k.player.damage_taken_pct < 0.0);
    let taken = player_mut(&mut app).get::<Toughness>().unwrap().0;
    assert!((taken - (1.0 + k.player.damage_taken_pct / 100.0)).abs() < 1e-5);
    let hearth = full_branch("ember", "Supporto");
    let hearth: Vec<&str> = hearth.iter().map(String::as_str).collect();
    let mut app = app_with_talents("ember", &hearth);
    run_steps(&mut app, 2);
    let k = kit(&mut app);
    let duration = player_mut(&mut app)
        .get::<Invulnerability>()
        .unwrap()
        .duration;
    let expected = config.player.invulnerability_s + k.player.invuln_secs;
    assert!(k.player.invuln_secs > 0.0 && (duration - expected).abs() < 1e-5);
    let max_hp = player_mut(&mut app).get::<Health>().unwrap().max;
    assert!((max_hp - (config.player.max_hp + k.player.max_hp)).abs() < 1e-3);
}
