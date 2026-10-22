//! Test della validazione. Id, indici e numeri attesi sono ricavati dai dati veri
//! (`GameConfig::embedded`), così i test non si rompono quando il designer cambia i valori.

use super::super::GameConfig;
use super::*;

fn embedded() -> GameConfig {
    GameConfig::embedded()
}

fn errors_after(change: impl FnOnce(&mut GameConfig)) -> Vec<String> {
    let mut config = embedded();
    change(&mut config);
    validate(&config)
}

fn assert_error(change: impl FnOnce(&mut GameConfig), expected: &str) {
    let errors = errors_after(change);
    assert!(
        errors.iter().any(|e| e.contains(expected)),
        "atteso '{expected}' in {errors:?}"
    );
}

#[test]
fn the_embedded_data_has_no_errors() {
    assert_eq!(validate(&embedded()), Vec::<String>::new());
}

#[test]
fn numbers_out_of_range_are_reported_with_file_and_field() {
    let c = embedded();
    let (enemy, attack) = (&c.enemies[0].id, &c.attacks[0].id);
    assert_error(
        |c| c.player.speed = 0.0,
        "player.ron: speed deve essere > 0",
    );
    assert_error(
        |c| c.enemies[0].speed = -3.0,
        &format!("enemies.ron: {enemy}.speed deve essere >= 0, trovato -3"),
    );
    assert_error(
        |c| c.attacks[0].interval_s = f32::NAN,
        &format!("attacks.ron: {attack}.interval_s"),
    );
    assert_error(
        |c| c.attacks[0].projectiles = 0,
        &format!("{attack}.projectiles"),
    );
    assert_error(
        |c| c.player.max_hp = 0.0,
        "player.ron: max_hp deve essere > 0",
    );
    let last_enemy = c.enemies.len() - 1;
    assert_error(
        |c| c.enemies[last_enemy].contact_damage = -2.0,
        &format!(
            "enemies.ron: {}.contact_damage deve essere >= 0",
            c.enemies[last_enemy].id
        ),
    );
}

#[test]
fn broken_references_are_reported() {
    let c = embedded();
    let last = c.stage.spawns.waves.len() - 1;
    assert_error(
        |c| c.companions[0].attack = "missing".into(),
        &format!(
            "companions.ron: {}.attack = \"missing\": non esiste in attacks.ron",
            c.companions[0].id
        ),
    );
    assert_error(
        |c| c.stage.spawns.waves[last].enemy = "dragon".into(),
        &format!(
            "stages.ron: {}.spawns.waves[{last}].enemy = \"dragon\": non esiste in enemies.ron",
            c.stage.id
        ),
    );
    assert_error(
        |c| c.stage.starting_companion = "x".into(),
        "starting_companion",
    );
}

#[test]
fn physical_limits_are_enforced() {
    let c = embedded();
    // corpo troppo largo per i varchi tra ostacoli
    assert_error(
        |c| c.enemies[0].footprint_radius = MAX_FOOTPRINT_RADIUS + 1.0,
        "non passa tra gli ostacoli",
    );
    // proiettile che salterebbe oltre un nemico in un passo
    assert_error(
        |c| c.attacks[0].projectile_speed *= 50.0,
        "attraversa i nemici",
    );
    // corpi abbastanza veloci da saltare oltre un tronco in un passo
    let limit = (MIN_FOOTPRINT_RADIUS + c.player.footprint_radius) * SIM_HZ as f32;
    assert_error(|c| c.player.speed = limit + 60.0, "player.ron: speed = ");
    assert!(
        !errors_after(|c| c.player.speed = limit)
            .iter()
            .any(|e| e.starts_with("player.ron: speed")),
        "al limite esatto la velocità è ammessa"
    );
    let enemy = &c.enemies[0];
    let enemy_limit = (MIN_FOOTPRINT_RADIUS + enemy.footprint_radius) * SIM_HZ as f32;
    assert_error(
        |c| c.enemies[0].speed = enemy_limit + 60.0,
        &format!("{}.speed = ", enemy.id),
    );
    // proiettile che sparisce prima della gittata
    let a = &c.attacks[0];
    assert_error(
        |c| c.attacks[0].projectile_lifetime_s = a.range / a.projectile_speed * 0.5,
        &format!("meno della gittata {} px", a.range),
    );
}

#[test]
fn spawn_ring_must_reach_outside_the_view_and_inside_the_arena() {
    let c = embedded();
    let (min, max) = c.stage.spawns.ring;
    assert_error(|c| c.stage.spawns.ring = (max, min), "0 < min < max");
    // anello tutto dentro la vista
    assert_error(
        |c| c.stage.spawns.ring = (VIEW_HALF_DIAGONAL * 0.3, VIEW_HALF_DIAGONAL * 0.9),
        "dentro la vista",
    );
    // anello tutto fuori dall'arena (rilievo del code review P0.3)
    assert_error(
        |c| c.stage.spawns.ring = (3000.0, 3500.0),
        "fuori dall'arena",
    );
}

#[test]
fn the_spawn_plan_is_checked() {
    let c = embedded();
    let plan = &c.stage.spawns;
    let last = plan.waves.len() - 1;
    assert_error(
        |c| c.stage.spawns.initial_burst = plan.max_alive + 1,
        "supera spawns.max_alive",
    );
    assert_error(
        |c| c.stage.spawns.max_alive = 0,
        "spawns.max_alive deve essere >= 1",
    );
    assert_error(|c| c.stage.spawns.waves.clear(), "serve almeno un'ondata");
    assert_error(
        |c| c.stage.spawns.waves[0].rate_per_min = -1.0,
        "spawns.waves[0].rate_per_min deve essere >= 0",
    );
    assert_error(
        |c| c.stage.spawns.waves[last].max_alive = 0,
        &format!("spawns.waves[{last}].max_alive deve essere >= 1"),
    );
}

/// L'ondata iniziale è fatta di nemici della prima ondata: ne rispetta limite e inizio.
#[test]
fn the_initial_burst_follows_the_first_wave_rules() {
    assert_error(
        |c| {
            c.stage.spawns.initial_burst = 20;
            c.stage.spawns.max_alive = 500;
            c.stage.spawns.waves[0].max_alive = 10;
        },
        "supera spawns.waves[0].max_alive = 10",
    );
    assert_error(
        |c| {
            c.stage.spawns.initial_burst = 1;
            c.stage.spawns.waves[0].start_s = 150.0;
        },
        "la prima ondata inizia a 150 s",
    );
    // senza ondata iniziale la prima ondata può partire dopo
    assert!(
        errors_after(|c| {
            c.stage.spawns.initial_burst = 0;
            c.stage.spawns.waves[0].start_s = 150.0;
        })
        .is_empty()
    );
}

#[test]
fn obstacle_parameters_are_checked() {
    let stage = embedded().stage.id;
    assert_error(
        |c| c.stage.obstacles.density = 1.5,
        &format!("stages.ron: {stage}.obstacles.density deve essere tra 0 e 1"),
    );
    assert_error(
        |c| c.stage.obstacles.tree_chance = -0.1,
        "obstacles.tree_chance",
    );
    assert_error(
        |c| c.stage.obstacles.safe_radius = -1.0,
        "obstacles.safe_radius",
    );
}

#[test]
fn duplicate_ids_and_bad_sprite_paths_are_reported() {
    let enemy = embedded().enemies[0].id.clone();
    assert_error(
        |c| {
            let copy = c.enemies[0].clone();
            c.enemies.push(copy);
        },
        &format!("enemies.ron: id \"{enemy}\" duplicato"),
    );
    assert_error(
        |c| c.player.sprite = "../secret.png".into(),
        "player.ron: sprite",
    );
    assert_error(
        |c| c.enemies[0].sprite = "sprites/blob.gif".into(),
        "serve un percorso .png",
    );
}
