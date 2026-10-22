//! Regole di coerenza dei dati, oltre alla sintassi. Ogni errore nomina file, voce e campo.

use std::collections::HashSet;

use super::GameConfig;
use crate::gameplay::SIM_HZ;
use crate::world::obstacles::{MIN_FOOTPRINT_RADIUS, MIN_OBSTACLE_GAP};

/// Il corpo più largo che passa sempre tra due ostacoli (vedi `world::obstacles`).
const MAX_FOOTPRINT_RADIUS: f32 = MIN_OBSTACLE_GAP / 2.0;

struct Report(Vec<String>);

impl Report {
    fn positive(&mut self, at: &str, value: f32) {
        if !(value.is_finite() && value > 0.0) {
            self.0
                .push(format!("{at} deve essere > 0, trovato {value}"));
        }
    }

    fn non_negative(&mut self, at: &str, value: f32) {
        if !(value.is_finite() && value >= 0.0) {
            self.0
                .push(format!("{at} deve essere >= 0, trovato {value}"));
        }
    }

    fn footprint(&mut self, at: &str, value: f32) {
        self.positive(at, value);
        if value > MAX_FOOTPRINT_RADIUS {
            self.0.push(format!(
                "{at} = {value}: oltre {MAX_FOOTPRINT_RADIUS} il corpo non passa tra gli ostacoli"
            ));
        }
    }

    fn sprite(&mut self, at: &str, path: &str) {
        if !path.ends_with(".png") || path.starts_with('/') || path.contains("..") {
            self.0.push(format!(
                "{at} = \"{path}\": serve un percorso .png relativo ad assets/"
            ));
        }
    }

    fn unique_ids<'a>(&mut self, file: &str, ids: impl Iterator<Item = &'a String>) {
        let mut seen = HashSet::new();
        for id in ids {
            if id.is_empty() {
                self.0.push(format!("{file}: id vuoto"));
            } else if !seen.insert(id) {
                self.0.push(format!("{file}: id \"{id}\" duplicato"));
            }
        }
    }

    fn reference(&mut self, at: &str, id: &str, exists: bool, target: &str) {
        if !exists {
            self.0
                .push(format!("{at} = \"{id}\": non esiste in {target}"));
        }
    }

    fn ring(&mut self, at: &str, (min, max): (f32, f32)) {
        if !(min.is_finite() && max.is_finite() && 0.0 < min && min < max) {
            self.0
                .push(format!("{at} = ({min}, {max}): serve 0 < min < max"));
        }
    }

    fn probability(&mut self, at: &str, value: f32) {
        if !(0.0..=1.0).contains(&value) {
            self.0
                .push(format!("{at} deve essere tra 0 e 1, trovato {value}"));
        }
    }

    /// Un corpo che si sposta di più di (impronta minima + proprio raggio) in un passo può
    /// saltare oltre il centro di un tronco: la respinta lo manda dall'altra parte.
    fn body_speed(&mut self, at: &str, speed: f32, footprint_radius: f32) {
        let step = speed / SIM_HZ as f32;
        let max_step = MIN_FOOTPRINT_RADIUS + footprint_radius;
        if step > max_step {
            self.0.push(format!(
                "{at} = {speed} px/s = {step:.1} px per passo: oltre {max_step} attraversa gli \
                 ostacoli"
            ));
        }
    }
}

pub(super) fn validate(config: &GameConfig) -> Vec<String> {
    let mut r = Report(Vec::new());

    let p = &config.player;
    r.positive("player.ron: speed", p.speed);
    r.footprint("player.ron: footprint_radius", p.footprint_radius);
    r.body_speed("player.ron: speed", p.speed, p.footprint_radius);
    r.sprite("player.ron: sprite", &p.sprite);

    r.unique_ids("companions.ron", config.companions.iter().map(|c| &c.id));
    for c in &config.companions {
        let at = |field: &str| format!("companions.ron: {}.{field}", c.id);
        r.non_negative(&at("follow.distance"), c.follow.distance);
        r.positive(&at("follow.max_speed"), c.follow.max_speed);
        r.positive(&at("follow.gain"), c.follow.gain);
        r.sprite(&at("sprite"), &c.sprite);
        let exists = config.attack(&c.attack).is_some();
        r.reference(&at("attack"), &c.attack, exists, "attacks.ron");
    }

    // Il proiettile non deve attraversare la hurtbox più piccola tra due passi da 1/60 s.
    let smallest_hurtbox = config
        .enemies
        .iter()
        .map(|e| e.hurtbox.radius)
        .fold(f32::INFINITY, f32::min);
    r.unique_ids("attacks.ron", config.attacks.iter().map(|a| &a.id));
    for a in &config.attacks {
        let at = |field: &str| format!("attacks.ron: {}.{field}", a.id);
        r.positive(&at("damage"), a.damage);
        r.positive(&at("interval_s"), a.interval_s);
        r.positive(&at("range"), a.range);
        r.positive(&at("projectile_speed"), a.projectile_speed);
        r.positive(&at("projectile_radius"), a.projectile_radius);
        r.positive(&at("projectile_lifetime_s"), a.projectile_lifetime_s);
        let travel = a.projectile_speed * a.projectile_lifetime_s;
        if travel < a.range {
            r.0.push(format!(
                "{}: il proiettile percorre {travel:.0} px, meno della gittata {} px: sparisce \
                 prima di raggiungere i nemici in raggio",
                at("projectile_lifetime_s"),
                a.range
            ));
        }
        r.non_negative(&at("spread_deg"), a.spread_deg);
        r.non_negative(&at("muzzle_height"), a.muzzle_height);
        r.sprite(&at("projectile_sprite"), &a.projectile_sprite);
        if a.projectiles == 0 {
            r.0.push(format!("{} deve essere >= 1", at("projectiles")));
        }
        let step = a.projectile_speed / SIM_HZ as f32;
        let max_step = a.projectile_radius + smallest_hurtbox;
        if step > max_step {
            r.0.push(format!(
                "{}: {} px/s = {step:.1} px per passo, oltre {max_step} attraversa i nemici \
                 senza colpirli",
                at("projectile_speed"),
                a.projectile_speed
            ));
        }
    }

    r.unique_ids("enemies.ron", config.enemies.iter().map(|e| &e.id));
    for e in &config.enemies {
        let at = |field: &str| format!("enemies.ron: {}.{field}", e.id);
        r.positive(&at("max_hp"), e.max_hp);
        r.non_negative(&at("speed"), e.speed);
        r.positive(&at("hurtbox.radius"), e.hurtbox.radius);
        r.non_negative(&at("hurtbox.center_offset"), e.hurtbox.center_offset);
        r.footprint(&at("footprint_radius"), e.footprint_radius);
        r.body_speed(&at("speed"), e.speed, e.footprint_radius);
        r.non_negative(&at("contact_distance"), e.contact_distance);
        r.non_negative(&at("separation_radius"), e.separation_radius);
        r.sprite(&at("sprite"), &e.sprite);
    }

    let s = &config.stage;
    let at = |field: &str| format!("stages.ron: {}.{field}", s.id);
    let companion_exists = config.companion(&s.starting_companion).is_some();
    r.reference(
        &at("starting_companion"),
        &s.starting_companion,
        companion_exists,
        "companions.ron",
    );
    let enemy_exists = config.enemy(&s.enemy).is_some();
    r.reference(&at("enemy"), &s.enemy, enemy_exists, "enemies.ron");
    if s.population_cap == 0 {
        r.0.push(format!("{} deve essere >= 1", at("population_cap")));
    }
    if s.initial_enemies > s.population_cap {
        r.0.push(format!(
            "{} = {} supera population_cap = {}",
            at("initial_enemies"),
            s.initial_enemies,
            s.population_cap
        ));
    }
    r.ring(&at("initial_ring"), s.initial_ring);
    r.ring(&at("respawn_ring"), s.respawn_ring);
    r.probability(&at("obstacles.density"), s.obstacles.density);
    r.non_negative(&at("obstacles.safe_radius"), s.obstacles.safe_radius);
    r.probability(&at("obstacles.tree_chance"), s.obstacles.tree_chance);

    r.0
}

#[cfg(test)]
mod tests {
    use super::super::GameConfig;
    use super::*;

    fn errors_after(change: impl FnOnce(&mut GameConfig)) -> Vec<String> {
        let mut config = GameConfig::embedded();
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
        assert_eq!(validate(&GameConfig::embedded()), Vec::<String>::new());
    }

    #[test]
    fn numbers_out_of_range_are_reported_with_file_and_field() {
        assert_error(
            |c| c.player.speed = 0.0,
            "player.ron: speed deve essere > 0",
        );
        assert_error(
            |c| c.enemies[0].speed = -3.0,
            "enemies.ron: blob.speed deve essere >= 0, trovato -3",
        );
        assert_error(
            |c| c.attacks[0].interval_s = f32::NAN,
            "attacks.ron: ember_bolt.interval_s",
        );
        assert_error(|c| c.attacks[0].projectiles = 0, "ember_bolt.projectiles");
    }

    #[test]
    fn broken_references_are_reported() {
        assert_error(
            |c| c.companions[0].attack = "missing".into(),
            "companions.ron: ember.attack = \"missing\": non esiste in attacks.ron",
        );
        assert_error(
            |c| c.stage.enemy = "dragon".into(),
            "stages.ron: meadow.enemy = \"dragon\"",
        );
        assert_error(
            |c| c.stage.starting_companion = "x".into(),
            "starting_companion",
        );
    }

    #[test]
    fn physical_limits_are_enforced() {
        // corpo troppo largo per i varchi tra ostacoli
        assert_error(
            |c| c.enemies[0].footprint_radius = 10.0,
            "non passa tra gli ostacoli",
        );
        // proiettile che salterebbe oltre un nemico in un passo
        assert_error(
            |c| c.attacks[0].projectile_speed = 2000.0,
            "attraversa i nemici",
        );
        assert_error(|c| c.stage.initial_ring = (240.0, 170.0), "0 < min < max");
        assert_error(|c| c.stage.initial_enemies = 99, "supera population_cap");
        // corpi abbastanza veloci da saltare oltre un tronco (4 + 5 px per passo = 540 px/s)
        assert_error(
            |c| c.player.speed = 600.0,
            "player.ron: speed = 600 px/s = 10.0 px per passo: oltre 9 attraversa gli ostacoli",
        );
        assert_error(|c| c.enemies[0].speed = 700.0, "blob.speed = 700 px/s");
        assert!(errors_after(|c| c.player.speed = 540.0).is_empty());
        // proiettile che sparisce prima della gittata
        assert_error(
            |c| c.attacks[0].projectile_lifetime_s = 0.5,
            "il proiettile percorre 110 px, meno della gittata 150 px",
        );
    }

    #[test]
    fn obstacle_parameters_are_checked() {
        assert_error(
            |c| c.stage.obstacles.density = 1.5,
            "stages.ron: meadow.obstacles.density deve essere tra 0 e 1",
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
        assert_error(
            |c| {
                let copy = c.enemies[0].clone();
                c.enemies.push(copy);
            },
            "enemies.ron: id \"blob\" duplicato",
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
}
