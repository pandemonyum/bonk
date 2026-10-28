//! Regole per `attacks.ron`: campi comuni e parametri di ogni pattern. I limiti fisici (un colpo
//! che salta oltre un nemico tra due passi) valgono qui con i valori base e in `progression`
//! con tutti gli upgrade al massimo.

use super::Report;
use crate::data::GameConfig;
use crate::data::schema::AttackPattern;
use crate::gameplay::SIM_HZ;

/// La hurtbox più piccola tra i nemici: il riferimento per "non attraversare un nemico".
pub(super) fn smallest_hurtbox(config: &GameConfig) -> f32 {
    config
        .enemies
        .iter()
        .map(|e| e.hurtbox.radius)
        .fold(f32::INFINITY, f32::min)
}

/// Spostamento per passo di una foglia (px) con orbita di raggio `radius`.
pub(super) fn leaf_step(angular_speed_deg: f32, radius: f32) -> f32 {
    angular_speed_deg.to_radians().abs() * radius / SIM_HZ as f32
}

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    let smallest = smallest_hurtbox(config);
    r.unique_ids("attacks.ron", config.attacks.iter().map(|a| &a.id));
    for a in &config.attacks {
        let at = |field: &str| format!("attacks.ron: {}.{field}", a.id);
        r.positive(&at("damage"), a.damage);
        r.positive(&at("interval_s"), a.interval_s);
        r.positive(&at("range"), a.range);
        r.sprite(&at("pattern.sprite"), a.sprite());
        match &a.pattern {
            AttackPattern::Projectile(p) => {
                r.positive(&at("pattern.speed"), p.speed);
                r.positive(&at("pattern.radius"), p.radius);
                r.positive(&at("pattern.lifetime_s"), p.lifetime_s);
                let travel = p.speed * p.lifetime_s;
                if travel < a.range {
                    r.0.push(format!(
                        "{}: il proiettile percorre {travel:.0} px, meno della gittata {} px: \
                         sparisce prima di raggiungere i nemici in raggio",
                        at("pattern.lifetime_s"),
                        a.range
                    ));
                }
                r.non_negative(&at("pattern.spread_deg"), p.spread_deg);
                r.non_negative(&at("pattern.muzzle_height"), p.muzzle_height);
                if p.count == 0 {
                    r.0.push(format!("{} deve essere >= 1", at("pattern.count")));
                }
                // Il proiettile non deve attraversare la hurtbox più piccola tra due passi.
                let step = p.speed / SIM_HZ as f32;
                let max_step = p.radius + smallest;
                if step > max_step {
                    r.0.push(format!(
                        "{}: {} px/s = {step:.1} px per passo, oltre {max_step} attraversa i \
                         nemici senza colpirli",
                        at("pattern.speed"),
                        p.speed
                    ));
                }
            }
            AttackPattern::Pulse(p) => {
                r.positive(&at("pattern.speed"), p.speed);
                if !(p.slow > 0.0 && p.slow <= 1.0) {
                    r.0.push(format!(
                        "{} = {}: è la frazione di velocità che resta, serve tra 0 (escluso) e 1",
                        at("pattern.slow"),
                        p.slow
                    ));
                }
                r.non_negative(&at("pattern.slow_s"), p.slow_s);
                r.non_negative(&at("pattern.center_height"), p.center_height);
            }
            AttackPattern::Orbit(o) => {
                if o.count == 0 {
                    r.0.push(format!("{} deve essere >= 1", at("pattern.count")));
                }
                if !(o.angular_speed_deg.is_finite() && o.angular_speed_deg != 0.0) {
                    r.0.push(format!(
                        "{} = {}: serve un numero finito diverso da 0",
                        at("pattern.angular_speed_deg"),
                        o.angular_speed_deg
                    ));
                }
                r.positive(&at("pattern.radius"), o.radius);
                r.non_negative(&at("pattern.center_height"), o.center_height);
                let step = leaf_step(o.angular_speed_deg, a.range);
                let max_step = o.radius + smallest;
                if step > max_step {
                    r.0.push(format!(
                        "{}: le foglie fanno {step:.1} px per passo, oltre {max_step} \
                         attraversano i nemici senza colpirli",
                        at("pattern.angular_speed_deg")
                    ));
                }
            }
            AttackPattern::Chain(c) => {
                r.positive(&at("pattern.jump_range"), c.jump_range);
                if !(c.falloff > 0.0 && c.falloff <= 1.0) {
                    r.0.push(format!(
                        "{} = {}: è il danno di un salto rispetto al precedente, serve tra 0 \
                         (escluso) e 1",
                        at("pattern.falloff"),
                        c.falloff
                    ));
                }
                r.non_negative(&at("pattern.muzzle_height"), c.muzzle_height);
                if !(c.slow > 0.0 && c.slow <= 1.0) {
                    r.0.push(format!(
                        "{} = {}: è la frazione di velocità che resta, serve tra 0 (escluso) e 1",
                        at("pattern.slow"),
                        c.slow
                    ));
                }
                r.non_negative(&at("pattern.slow_s"), c.slow_s);
            }
        }
    }
}

#[cfg(test)]
mod tests;
