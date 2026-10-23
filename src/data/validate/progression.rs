//! Regole per `progression.ron` e `upgrades.ron`. I limiti si controllano con le carte al
//! massimo, separate per segno: il valore più basso raggiungibile somma solo le carte che
//! tolgono, il più alto solo quelle che aggiungono (un giocatore può prendere le une senza le
//! altre). Un valore valido a inizio run deve restarlo anche dopo venti livelli.

use std::collections::HashSet;

use super::Report;
use super::attacks::{leaf_step, smallest_hurtbox};
use crate::data::GameConfig;
use crate::data::schema::{AttackPattern, Effect, Rarity};
use crate::gameplay::SIM_HZ;
use crate::gameplay::attacks::MAX_PROJECTILES;
use crate::gameplay::progression::MAX_ENEMY_XP;
use crate::gameplay::upgrades::{MAX_DRAFT_SIZE, pct_factor};

/// Effetti percentuali, con il nome usato nei messaggi.
const PCT_EFFECTS: [&str; 6] = [
    "CompanionDamagePct",
    "CompanionIntervalPct",
    "CompanionRangePct",
    "ProjectileSpeedPct",
    "MoveSpeedPct",
    "PickupRadiusPct",
];

fn pct_value(effect: &Effect) -> Option<(&'static str, f32)> {
    Some(match *effect {
        Effect::CompanionDamagePct(v) => (PCT_EFFECTS[0], v),
        Effect::CompanionIntervalPct(v) => (PCT_EFFECTS[1], v),
        Effect::CompanionRangePct(v) => (PCT_EFFECTS[2], v),
        Effect::ProjectileSpeedPct(v) => (PCT_EFFECTS[3], v),
        Effect::MoveSpeedPct(v) => (PCT_EFFECTS[4], v),
        Effect::PickupRadiusPct(v) => (PCT_EFFECTS[5], v),
        _ => return None,
    })
}

/// Somme di un effetto con ogni carta al massimo degli stack: (solo negative, solo positive).
fn signed_totals(config: &GameConfig, value: impl Fn(&Effect) -> Option<f32>) -> (f32, f32) {
    let (mut negative, mut positive) = (0.0, 0.0);
    for u in &config.upgrades {
        if let Some(v) = value(&u.effect) {
            let total = v * u.max_stack as f32;
            if total < 0.0 {
                negative += total;
            } else {
                positive += total;
            }
        }
    }
    (negative, positive)
}

/// Fattore più basso e più alto raggiungibili per un effetto percentuale.
fn pct_extremes(config: &GameConfig, kind: &str) -> (f32, f32) {
    let (negative, positive) = signed_totals(config, |e| {
        pct_value(e).filter(|(k, _)| *k == kind).map(|(_, v)| v)
    });
    (pct_factor(negative), pct_factor(positive))
}

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    let p = &config.progression;
    r.positive("progression.ron: xp_to_level.base", p.xp_to_level.base);
    r.non_negative(
        "progression.ron: xp_to_level.per_level",
        p.xp_to_level.per_level,
    );
    r.sprite("progression.ron: gem.sprite", &p.gem.sprite);
    r.positive(
        "progression.ron: gem.collect_distance",
        p.gem.collect_distance,
    );
    if p.gem.max_on_field == 0 {
        r.0.push("progression.ron: gem.max_on_field deve essere >= 1".into());
    }
    if !(1..=MAX_DRAFT_SIZE).contains(&p.draft_size) {
        r.0.push(format!(
            "progression.ron: draft_size = {}: serve da 1 a {MAX_DRAFT_SIZE} (tasti 1-3 e \
             larghezza dello schermo)",
            p.draft_size
        ));
    }
    for rarity in [Rarity::Common, Rarity::Uncommon, Rarity::Rare] {
        let at = format!("progression.ron: rarity_weights.{rarity:?}").to_lowercase();
        r.non_negative(&at, p.rarity_weights.of(rarity));
    }
    r.positive("player.ron: pickup_radius", config.player.pickup_radius);
    for e in &config.enemies {
        if e.xp > MAX_ENEMY_XP {
            r.0.push(format!(
                "enemies.ron: {}.xp = {}: al massimo {MAX_ENEMY_XP}",
                e.id, e.xp
            ));
        }
    }

    r.unique_ids("upgrades.ron", config.upgrades.iter().map(|u| &u.id));
    for u in &config.upgrades {
        let at = |field: &str| format!("upgrades.ron: {}.{field}", u.id);
        if u.name.trim().is_empty() || u.description.trim().is_empty() {
            r.0.push(format!(
                "{}: nome e descrizione della carta servono",
                at("name")
            ));
        }
        r.ui_text(&at("name"), &u.name);
        r.ui_text(&at("description"), &u.description);
        if u.max_stack == 0 {
            r.0.push(format!("{} deve essere >= 1", at("max_stack")));
        }
        for required in &u.requires {
            if *required == u.id {
                r.0.push(format!(
                    "{} = \"{required}\": richiede sé stesso",
                    at("requires")
                ));
            } else {
                let exists = config.upgrade(required).is_some();
                r.reference(&at("requires"), required, exists, "upgrades.ron");
            }
        }
        effect(r, &at("effect"), &u.effect);
    }
    join_cards(r, config);
    reachability(r, config);
    limits_at_max(r, config);
}

/// Una carta da sola: ogni stack deve fare qualcosa, con un numero finito.
fn effect(r: &mut Report, at: &str, effect: &Effect) {
    match *effect {
        Effect::CompanionDamagePct(v)
        | Effect::CompanionIntervalPct(v)
        | Effect::CompanionRangePct(v)
        | Effect::ProjectileSpeedPct(v)
        | Effect::MoveSpeedPct(v)
        | Effect::PickupRadiusPct(v)
        | Effect::MaxHp(v) => {
            if !(v.is_finite() && v != 0.0) {
                r.0.push(format!("{at} = {v}: serve un numero finito diverso da 0"));
            }
        }
        Effect::CompanionProjectiles(n) | Effect::CompanionPierce(n) => {
            if n == 0 {
                r.0.push(format!("{at}: serve almeno 1 per stack"));
            }
        }
        Effect::RegenPerSecond(v) => r.positive(at, v),
        Effect::Heal(v) => r.positive(at, v),
        Effect::JoinCompanion(_) => {} // vedi `join_cards`
    }
}

/// Carte "nuovo compagno": il compagno esiste, arriva una volta sola e c'è almeno un posto
/// oltre a quello del compagno iniziale. Una carta per il compagno scelto nel menu non viene
/// proposta (`upgrades::PartyView`), quindi ogni compagno può avere la sua.
fn join_cards(r: &mut Report, config: &GameConfig) {
    let mut seen: Vec<&str> = Vec::new();
    for u in &config.upgrades {
        let Effect::JoinCompanion(id) = &u.effect else {
            continue;
        };
        let at = format!("upgrades.ron: {}.effect", u.id);
        let exists = config.companion(id).is_some();
        r.reference(&at, id, exists, "companions.ron");
        if u.max_stack != 1 {
            r.0.push(format!(
                "upgrades.ron: {}.max_stack = {}: un compagno arriva una volta sola, serve 1",
                u.id, u.max_stack
            ));
        }
        if seen.contains(&id.as_str()) {
            r.0.push(format!("{at}: \"{id}\" ha già un'altra carta"));
        }
        seen.push(id);
        if config.player.companion_slots.len() < 2 {
            r.0.push(format!(
                "{at}: player.ron ha un solo posto (companion_slots), la carta non verrà mai \
                 proposta"
            ));
        }
    }
}

/// Upgrade che non verranno mai proposti (requisiti circolari o che hanno peso di rarità 0),
/// e primo draft vuoto.
fn reachability(r: &mut Report, config: &GameConfig) {
    let weights = &config.progression.rarity_weights;
    let offered = |u: &crate::data::schema::UpgradeData| weights.of(u.rarity) > 0.0;
    let mut reachable: HashSet<&str> = HashSet::new();
    loop {
        let before = reachable.len();
        for u in config.upgrades.iter().filter(|u| offered(u)) {
            if u.requires
                .iter()
                .all(|req| reachable.contains(req.as_str()))
            {
                reachable.insert(&u.id);
            }
        }
        if reachable.len() == before {
            break;
        }
    }
    for u in config.upgrades.iter().filter(|u| offered(u)) {
        let all_exist = u.requires.iter().all(|req| config.upgrade(req).is_some());
        if all_exist && !reachable.contains(u.id.as_str()) {
            r.0.push(format!(
                "upgrades.ron: {}.requires = {:?}: non verrà mai proposto (requisiti circolari \
                 o con peso di rarità 0)",
                u.id, u.requires
            ));
        }
    }
    let first_draft = config
        .upgrades
        .iter()
        .any(|u| u.requires.is_empty() && offered(u));
    if !first_draft {
        r.0.push(
            "upgrades.ron: serve almeno un upgrade senza requisiti con peso di rarità > 0, \
             altrimenti il primo draft è vuoto"
                .into(),
        );
    }
}

/// Limiti con le carte al massimo (vedi l'intestazione del modulo).
fn limits_at_max(r: &mut Report, config: &GameConfig) {
    for kind in PCT_EFFECTS {
        let (lowest, _) = pct_extremes(config, kind);
        if !(lowest.is_finite() && lowest > 0.0) {
            r.0.push(format!(
                "upgrades.ron: le carte {kind} negative, tutte al massimo, portano il fattore a \
                 {lowest:.2}: deve restare > 0"
            ));
        }
    }
    let (fewer_hp, _) = signed_totals(config, |e| match e {
        Effect::MaxHp(v) => Some(*v),
        _ => None,
    });
    let lowest_hp = config.player.max_hp + fewer_hp;
    if lowest_hp.is_nan() || lowest_hp <= 0.0 {
        r.0.push(format!(
            "upgrades.ron: le carte MaxHp negative, tutte al massimo, portano gli HP massimi a \
             {lowest_hp}: devono restare > 0"
        ));
    }

    let (_, speed) = pct_extremes(config, "ProjectileSpeedPct");
    let (interval, _) = pct_extremes(config, "CompanionIntervalPct");
    let (_, move_speed) = pct_extremes(config, "MoveSpeedPct");
    let extra_projectiles: u64 = config
        .upgrades
        .iter()
        .map(|u| match u.effect {
            Effect::CompanionProjectiles(n) => u64::from(n) * u64::from(u.max_stack),
            _ => 0,
        })
        .sum();
    let (_, range) = pct_extremes(config, "CompanionRangePct");
    let smallest = smallest_hurtbox(config);
    let step = 1.0 / SIM_HZ as f32;
    for a in &config.attacks {
        // fattore <= 0 già segnalato sopra
        if interval > 0.0 && a.interval_s * interval < step {
            r.0.push(format!(
                "upgrades.ron: con tutti gli upgrade di cadenza {}.interval_s scende a {:.3} s, \
                 sotto un passo ({step:.3} s): la cadenza in più andrebbe persa",
                a.id,
                a.interval_s * interval
            ));
        }
        let base_count = match &a.pattern {
            AttackPattern::Projectile(p) => Some(p.count),
            AttackPattern::Orbit(o) => Some(o.count),
            AttackPattern::Pulse(_) => None,
        };
        let most = base_count.map(|n| u64::from(n) + extra_projectiles);
        if let Some(most) = most.filter(|&n| n > u64::from(MAX_PROJECTILES)) {
            r.0.push(format!(
                "upgrades.ron: con tutti gli upgrade {} arriva a {most} proiettili o foglie, al \
                 massimo {MAX_PROJECTILES}",
                a.id
            ));
        }
        if let AttackPattern::Pulse(p) = &a.pattern {
            // un'onda finisce prima che parta la successiva: più onde insieme moltiplicano
            // colpi e gocce a schermo
            let longest = a.range * range / p.speed;
            let shortest = a.interval_s * interval;
            if interval > 0.0 && longest >= shortest {
                r.0.push(format!(
                    "attacks.ron: {}: con gli upgrade al massimo l'onda dura {longest:.2} s, non \
                     meno dell'intervallo più breve {shortest:.2} s (alza pattern.speed)",
                    a.id
                ));
            }
        }
        if let AttackPattern::Projectile(p) = &a.pattern {
            let fastest = p.speed * speed;
            let max_step = p.radius + smallest;
            if fastest * step > max_step {
                r.0.push(format!(
                    "upgrades.ron: con tutti gli upgrade di velocità {}.pattern.speed arriva a \
                     {fastest:.0} px/s = {:.1} px per passo, oltre {max_step} attraversa i nemici",
                    a.id,
                    fastest * step
                ));
            }
        }
        if let AttackPattern::Orbit(o) = &a.pattern {
            // l'orbita più larga fa correre di più le foglie
            let fastest = leaf_step(o.angular_speed_deg, a.range * range);
            let max_step = o.radius + smallest;
            if fastest > max_step {
                r.0.push(format!(
                    "upgrades.ron: con tutti gli upgrade di gittata le foglie di {} fanno \
                     {fastest:.1} px per passo, oltre {max_step} attraversano i nemici",
                    a.id
                ));
            }
        }
    }
    let player = &config.player;
    let fastest_player = player.speed * move_speed;
    r.body_speed(
        "upgrades.ron: player.speed con tutti gli upgrade di velocità",
        fastest_player,
        player.footprint_radius,
    );
    // Gemme attirate e compagni devono raggiungere il player anche alla sua velocità massima.
    let magnet = config.progression.gem.magnet_speed;
    if !(magnet.is_finite() && magnet > fastest_player) {
        r.0.push(format!(
            "progression.ron: gem.magnet_speed = {magnet} deve superare la velocità massima del \
             player ({fastest_player:.0} px/s con tutti gli upgrade), o le gemme non lo raggiungono"
        ));
    }
    for c in &config.companions {
        if c.follow.max_speed < fastest_player {
            r.0.push(format!(
                "companions.ron: {}.follow.max_speed = {} è sotto la velocità massima del player \
                 ({fastest_player:.0} px/s con tutti gli upgrade): resterebbe indietro",
                c.id, c.follow.max_speed
            ));
        }
    }
}

#[cfg(test)]
mod pattern_tests;
#[cfg(test)]
mod tests;
