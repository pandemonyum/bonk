//! Regole di coerenza dei dati, oltre alla sintassi. Ogni errore nomina file, voce e campo.

use std::collections::HashSet;

use super::GameConfig;
use crate::gameplay::SIM_HZ;
use crate::gameplay::companions::MAX_COMPANION_SLOTS;
use crate::world::ArenaBounds;
use crate::world::obstacles::{MIN_FOOTPRINT_RADIUS, MIN_OBSTACLE_GAP};
use crate::world::view::VIEW_HALF_DIAGONAL;

/// Il corpo più largo che passa sempre tra due ostacoli (vedi `world::obstacles`).
const MAX_FOOTPRINT_RADIUS: f32 = MIN_OBSTACLE_GAP / 2.0;

/// Segni oltre il Latin-1 che Tiny5 e Pixelify Sans disegnano tutti e due.
pub const UI_EXTRA_CHARS: &str = "‘’“”–—…€";

/// Carattere che i due font dell'interfaccia (`assets/fonts/`) disegnano: ASCII stampabile, a
/// capo, Latin-1 (lettere accentate comprese) tranne il trattino morbido e la barra spezzata,
/// che mancano, e `UI_EXTRA_CHARS`. Lo verifica un test sul cmap dei file
/// (`presentation::fonts`).
pub fn ui_drawable(c: char) -> bool {
    matches!(c, ' '..='~' | '\n' | '\u{a0}'..='\u{ff}') && !matches!(c, '\u{a6}' | '\u{ad}')
        || UI_EXTRA_CHARS.contains(c)
}

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

    /// Fotogramma di un foglio animato: non vuoto e largo un numero pari di texel (lo sprite è
    /// ancorato in basso al centro).
    fn sheet_frame(&mut self, at: &str, frame: Option<crate::data::schema::SheetFrame>) {
        if let Some(f) = frame
            && (f.w == 0 || f.h == 0 || f.w % 2 != 0)
        {
            self.0.push(format!(
                "{at} = ({}, {}): serve un fotogramma non vuoto e largo un numero pari di texel \
                 (lo sprite è ancorato in basso al centro)",
                f.w, f.h
            ));
        }
    }

    /// Testo mostrato a schermo: solo caratteri che i font dell'interfaccia disegnano (D-044).
    fn ui_text(&mut self, at: &str, text: &str) {
        if let Some(c) = text.chars().find(|c| !ui_drawable(*c)) {
            self.0.push(format!(
                "{at} contiene '{c}' (U+{:04X}): i font dell'interfaccia non lo disegnano \
                 (vanno bene ASCII, lettere accentate e segni Latin-1, {UI_EXTRA_CHARS})",
                c as u32
            ));
        }
    }

    /// Testo mostrato a schermo che non può mancare: non vuoto e disegnabile dai font.
    fn text(&mut self, at: &str, text: &str) {
        if text.trim().is_empty() {
            self.0
                .push(format!("{at}: serve un testo, trovato \"{text}\""));
        }
        self.ui_text(at, text);
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
    let slots = p.companion_slots.len();
    if !(1..=MAX_COMPANION_SLOTS).contains(&slots) {
        r.0.push(format!(
            "player.ron: companion_slots ha {slots} posti: servono da 1 (il compagno iniziale) \
             a {MAX_COMPANION_SLOTS}"
        ));
    }
    for (i, slot) in p.companion_slots.iter().enumerate() {
        if !slot.angle_deg.is_finite() {
            r.0.push(format!(
                "player.ron: companion_slots[{i}].angle_deg non è un numero"
            ));
        }
    }

    companions::check(&mut r, config);
    attacks::check(&mut r, config);

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
    run::check(&mut r, config);
    talents::check(&mut r, config);
    r.0
}

mod attacks;
mod companions;
mod progression;
mod run;
mod talents;
#[cfg(test)]
mod tests;
