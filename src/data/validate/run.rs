//! Regole per l'arco della run (`stages.ron: run`) e per i comportamenti telegrafati dei nemici
//! (`enemies.ron: charge`, `slam`): riferimenti, tempi, e un preavviso abbastanza lungo da
//! essere visto e schivato.

use super::Report;
use crate::data::GameConfig;
use crate::gameplay::SIM_HZ;
use crate::gameplay::threats::MIN_TELEGRAPH_S;

/// Orde: nemici esistenti, istanti crescenti dentro la run con il preavviso, e spazio per
/// nascere oltre ai vivi delle ondate.
fn hordes(r: &mut Report, config: &GameConfig, at: &dyn Fn(&str) -> String) {
    let run = &config.stage.run;
    let h = &run.hordes;
    if h.list.is_empty() {
        return;
    }
    r.non_negative(&at("hordes.warning_s"), h.warning_s);
    r.positive(&at("hordes.duration_s"), h.duration_s);
    if h.max_alive <= config.stage.spawns.max_alive {
        r.0.push(format!(
            "{} = {}: deve superare spawns.max_alive ({}), se no a campo pieno l'orda non nasce",
            at("hordes.max_alive"),
            h.max_alive,
            config.stage.spawns.max_alive
        ));
    }
    let mut previous_end = f32::NEG_INFINITY;
    for (i, horde) in h.list.iter().enumerate() {
        let here = |field: &str| at(&format!("hordes.list[{i}].{field}"));
        if horde.at_s - h.warning_s < previous_end || horde.at_s - h.warning_s < 0.0 {
            r.0.push(format!(
                "{} = {}: con il preavviso di {} s si sovrappone all'orda prima o all'inizio",
                here("at_s"),
                horde.at_s,
                h.warning_s
            ));
        }
        if horde.at_s + h.duration_s > run.duration_s {
            r.0.push(format!(
                "{} = {}: l'orda finisce dopo il boss (duration_s = {})",
                here("at_s"),
                horde.at_s,
                run.duration_s
            ));
        }
        previous_end = horde.at_s + h.duration_s;
        if horde.total() == 0 {
            r.0.push(format!("{}: l'orda non ha nemici", here("groups")));
        }
        for (k, group) in horde.groups.iter().enumerate() {
            let exists = config.enemy(&group.enemy).is_some();
            r.reference(
                &here(&format!("groups[{k}].enemy")),
                &group.enemy,
                exists,
                "enemies.ron",
            );
        }
    }
}

/// Monete e forzieri: probabilità sensate, fasce di distanza ordinate e fuori vista, un
/// prezzo, un raggio per aprirli.
fn treasure(r: &mut Report, config: &GameConfig) {
    let coin = &config.progression.coin;
    r.sprite("progression.ron: coin.sprite", &coin.sprite);
    if coin.max_on_field == 0 {
        r.0.push("progression.ron: coin.max_on_field deve essere >= 1".into());
    }
    r.non_negative("progression.ron: coin.scatter", coin.scatter);
    for e in &config.enemies {
        if !(0.0..=1.0).contains(&e.coins.chance) {
            r.0.push(format!(
                "enemies.ron: {}.coins.chance = {}: è una probabilità, serve tra 0 e 1",
                e.id, e.coins.chance
            ));
        }
    }
    let c = &config.stage.chests;
    let at = |field: &str| format!("stages.ron: {}.chests.{field}", config.stage.id);
    r.sprite(&at("sprite"), &c.sprite);
    r.sheet_frame(
        &at("frame_w"),
        Some(crate::data::schema::SheetFrame {
            w: c.frame_w,
            h: c.frame_h,
        }),
    );
    for (field, band) in [
        ("distance", c.distance),
        ("fallback_distance", c.fallback_distance),
    ] {
        if !(band.0 > 0.0 && band.0 < band.1) {
            r.0.push(format!(
                "{} = {band:?}: serve (min, max) con 0 < min < max",
                at(field)
            ));
        }
    }
    r.positive(&at("open_radius"), c.open_radius);
    r.positive(&at("respawn_beyond"), c.respawn_beyond);
    if c.respawn_beyond <= c.fallback_distance.1 {
        r.0.push(format!(
            "{} = {}: deve superare fallback_distance.1 ({}), se no un forziere appena nato \
             rinascerebbe subito",
            at("respawn_beyond"),
            c.respawn_beyond,
            c.fallback_distance.1
        ));
    }
    r.non_negative(&at("min_from_edge"), c.min_from_edge);
    r.non_negative(&at("min_apart"), c.min_apart);
    if c.on_map > 0 && c.price_base == 0 {
        r.0.push(format!("{} deve essere >= 1", at("price_base")));
    }
}

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
    hordes(r, config, &at);
    treasure(r, config);
    r.ui_text(
        &format!("stages.ron: {}.display_name", s.id),
        &s.display_name,
    );

    for e in &config.enemies {
        let at = |field: &str| format!("enemies.ron: {}.{field}", e.id);
        r.ui_text(&at("display_name"), &e.display_name);
        r.sheet_frame(&at("frame"), e.frame);
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
