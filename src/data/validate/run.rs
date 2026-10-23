//! Regole per l'arco della run (`stages.ron: run`) e per i comportamenti telegrafati dei nemici
//! (`enemies.ron: charge`, `slam`): riferimenti, tempi, e un preavviso abbastanza lungo da
//! essere visto e schivato.

use super::Report;
use crate::data::GameConfig;
use crate::gameplay::SIM_HZ;
use crate::gameplay::threats::MIN_TELEGRAPH_S;

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    let s = &config.stage;
    let run = &s.run;
    let at = |field: &str| format!("stages.ron: {}.run.{field}", s.id);
    r.positive(&at("duration_s"), run.duration_s);
    r.non_negative(&at("hp_growth_per_min"), run.hp_growth_per_min);
    r.non_negative(&at("damage_growth_per_min"), run.damage_growth_per_min);
    let elite_exists = config.enemy(&run.elites.enemy).is_some();
    r.reference(
        &at("elites.enemy"),
        &run.elites.enemy,
        elite_exists,
        "enemies.ron",
    );
    r.non_negative(&at("elites.first_s"), run.elites.first_s);
    r.positive(&at("elites.every_s"), run.elites.every_s);
    let boss_exists = config.enemy(&run.boss).is_some();
    r.reference(&at("boss"), &run.boss, boss_exists, "enemies.ron");
    r.ui_text(
        &format!("stages.ron: {}.display_name", s.id),
        &s.display_name,
    );

    for e in &config.enemies {
        let at = |field: &str| format!("enemies.ron: {}.{field}", e.id);
        r.ui_text(&at("display_name"), &e.display_name);
        if let Some(c) = &e.charge {
            r.positive(&at("charge.every_s"), c.every_s);
            telegraph(r, &at("charge.windup_s"), c.windup_s);
            r.positive(&at("charge.speed"), c.speed);
            r.body_speed(&at("charge.speed"), c.speed, e.footprint_radius);
            // in un passo non deve superare il raggio di contatto: salterebbe oltre il player
            let step = c.speed / SIM_HZ as f32;
            if step > e.contact_distance {
                r.0.push(format!(
                    "{} = {}: {step:.1} px per passo, oltre contact_distance {} attraversa il player",
                    at("charge.speed"),
                    c.speed,
                    e.contact_distance
                ));
            }
            r.positive(&at("charge.duration_s"), c.duration_s);
            r.positive(&at("charge.range"), c.range);
        }
        if let Some(s) = &e.slam {
            r.positive(&at("slam.every_s"), s.every_s);
            telegraph(r, &at("slam.windup_s"), s.windup_s);
            r.positive(&at("slam.radius"), s.radius);
            r.non_negative(&at("slam.damage"), s.damage);
            r.positive(&at("slam.range"), s.range);
            // dal centro del cerchio si deve poter uscire prima del colpo, alla velocità base
            let escape_s = s.radius / config.player.speed;
            if escape_s > s.windup_s {
                r.0.push(format!(
                    "{} = {}: con player.speed {} servono {escape_s:.2} s per uscire, più del \
                     preavviso {} s: il colpo non si può schivare",
                    at("slam.radius"),
                    s.radius,
                    config.player.speed,
                    s.windup_s
                ));
            }
            if s.windup_s >= s.every_s {
                r.0.push(format!(
                    "{} = {}: il preavviso deve finire prima del colpo successivo (every_s {})",
                    at("slam.windup_s"),
                    s.windup_s,
                    s.every_s
                ));
            }
        }
    }
}

/// Un attacco telegrafato deve restare visibile abbastanza da reagire.
fn telegraph(r: &mut Report, at: &str, windup: f32) {
    if !(windup.is_finite() && windup >= MIN_TELEGRAPH_S) {
        r.0.push(format!(
            "{at} = {windup}: il preavviso dura almeno {MIN_TELEGRAPH_S} s, se no non si vede"
        ));
    }
}

#[cfg(test)]
mod tests;
