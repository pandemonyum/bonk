//! Regole di coerenza dei dati, oltre alla sintassi. Ogni errore nomina file, voce e campo.

use std::collections::HashSet;

use super::GameConfig;
use crate::gameplay::SIM_HZ;
use crate::world::ArenaBounds;
use crate::world::obstacles::{MIN_FOOTPRINT_RADIUS, MIN_OBSTACLE_GAP};
use crate::world::view::VIEW_HALF_DIAGONAL;

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
    r.positive("player.ron: max_hp", p.max_hp);
    r.non_negative("player.ron: invulnerability_s", p.invulnerability_s);
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
        r.non_negative(&at("contact_damage"), e.contact_damage);
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
    let plan = &s.spawns;
    r.ring(&at("spawns.ring"), plan.ring);
    if plan.ring.1 <= VIEW_HALF_DIAGONAL {
        r.0.push(format!(
            "{} = {:?}: con max {} ogni punto dell'anello è dentro la vista (mezza diagonale \
             {VIEW_HALF_DIAGONAL:.0} px), i nemici non nascerebbero mai",
            at("spawns.ring"),
            plan.ring,
            plan.ring.1
        ));
    }
    if plan.max_alive == 0 {
        r.0.push(format!("{} deve essere >= 1", at("spawns.max_alive")));
    }
    if plan.initial_burst > plan.max_alive {
        r.0.push(format!(
            "{} = {} supera spawns.max_alive = {}",
            at("spawns.initial_burst"),
            plan.initial_burst,
            plan.max_alive
        ));
    }
    // Dal punto di partenza (centro) l'anello deve arrivare dentro l'arena, altrimenti nessun
    // punto è valido e la run resta vuota senza errori.
    let arena = ArenaBounds::default();
    let reach = arena.max.min(-arena.min).min_element();
    if plan.ring.0 >= reach {
        r.0.push(format!(
            "{} = {:?}: con min {} ogni punto è fuori dall'arena (raggio utile {reach:.0} px \
             dal centro), i nemici non nascerebbero mai",
            at("spawns.ring"),
            plan.ring,
            plan.ring.0
        ));
    }
    match plan.waves.first() {
        None => {
            r.0.push(format!("{}: serve almeno un'ondata", at("spawns.waves")))
        }
        // L'ondata iniziale è fatta di nemici della prima ondata e ne rispetta le regole.
        Some(first) if plan.initial_burst > 0 => {
            if plan.initial_burst > first.max_alive {
                r.0.push(format!(
                    "{} = {} supera spawns.waves[0].max_alive = {}",
                    at("spawns.initial_burst"),
                    plan.initial_burst,
                    first.max_alive
                ));
            }
            if first.start_s > 0.0 {
                r.0.push(format!(
                    "{} = {}: la prima ondata inizia a {} s, ma initial_burst la fa comparire \
                     all'avvio (metti start_s: 0.0 o initial_burst: 0)",
                    at("spawns.initial_burst"),
                    plan.initial_burst,
                    first.start_s
                ));
            }
        }
        Some(_) => {}
    }
    for (i, wave) in plan.waves.iter().enumerate() {
        let at = |field: &str| format!("stages.ron: {}.spawns.waves[{i}].{field}", s.id);
        let exists = config.enemy(&wave.enemy).is_some();
        r.reference(&at("enemy"), &wave.enemy, exists, "enemies.ron");
        r.non_negative(&at("start_s"), wave.start_s);
        r.non_negative(&at("rate_per_min"), wave.rate_per_min);
        r.non_negative(&at("rate_growth_per_min"), wave.rate_growth_per_min);
        if wave.max_alive == 0 {
            r.0.push(format!("{} deve essere >= 1", at("max_alive")));
        }
    }
    r.probability(&at("obstacles.density"), s.obstacles.density);
    r.non_negative(&at("obstacles.safe_radius"), s.obstacles.safe_radius);
    r.probability(&at("obstacles.tree_chance"), s.obstacles.tree_chance);

    progression::check(&mut r, config);
    r.0
}

mod progression;
#[cfg(test)]
mod tests;
