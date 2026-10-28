//! Regole per i talenti (`companions.ron: talents`) e per l'esperienza dei compagni
//! (`progression.ron: companion_xp`): id unici in tutto il gioco, requisiti nello stesso ramo e
//! più in alto (niente cicli), posti dentro la griglia della schermata, testi disegnabili,
//! valori sensati, e abbastanza livelli per spendere tutti i punti.

use std::collections::HashSet;

use super::Report;
use crate::data::GameConfig;
use crate::data::schema::TalentEffect;

/// Colonne (0 e 1 i percorsi, 2 al centro) e righe della schermata dei talenti.
pub const MAX_COL: u32 = 2;
pub const MAX_ROW: u32 = 4;
/// Gradi al massimo per nodo.
pub const MAX_RANK: u32 = 5;

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    let xp = &config.progression.companion_xp;
    let at = |field: &str| format!("progression.ron: companion_xp.{field}");
    r.positive(&at("seconds_per_point"), xp.seconds_per_point);
    if xp.kills_per_point == 0 || xp.first_level == 0 || xp.max_level < 2 {
        r.0.push(format!(
            "{}: servono kills_per_point e first_level >= 1 e max_level >= 2",
            at("")
        ));
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for c in &config.companions {
        let mut points = 0;
        for (b, branch) in c.talents.iter().enumerate() {
            let here = format!("companions.ron: {}.talents[{b}]", c.id);
            r.ui_text(&format!("{here}.name"), &branch.name);
            for node in &branch.nodes {
                let at = |field: &str| format!("{here}.{}.{field}", node.id);
                if !seen.insert(node.id.as_str()) {
                    r.0.push(format!("{here}: id \"{}\" duplicato", node.id));
                }
                r.ui_text(&at("name"), &node.name);
                r.ui_text(&at("description"), &node.description);
                if node.col > MAX_COL || node.row > MAX_ROW {
                    r.0.push(format!(
                        "{}: ({}, {}) fuori dalla griglia (colonne 0-{MAX_COL}, righe 0-{MAX_ROW})",
                        at("col"),
                        node.col,
                        node.row
                    ));
                }
                if node.ranks.is_empty() || node.max_rank() > MAX_RANK {
                    r.0.push(format!("{}: servono da 1 a {MAX_RANK} gradi", at("ranks")));
                }
                points += node.max_rank();
                for required in &node.requires {
                    let above = branch
                        .nodes
                        .iter()
                        .find(|n| n.id == *required)
                        .map(|n| n.row < node.row);
                    match above {
                        None => r.0.push(format!(
                            "{}: \"{required}\" non è un nodo di questo ramo",
                            at("requires")
                        )),
                        Some(false) => r.0.push(format!(
                            "{}: \"{required}\" deve stare in una riga più in alto",
                            at("requires")
                        )),
                        Some(true) => {}
                    }
                }
                for effect in node.ranks.iter().flatten() {
                    effect_values(r, &at("ranks"), effect);
                }
            }
        }
        // il Supporto aiuta ma non annulla il pericolo (proposta del game-designer)
        let all: Vec<String> = c
            .talents
            .iter()
            .flat_map(|b| b.nodes.iter())
            .flat_map(|n| std::iter::repeat_n(n.id.clone(), n.max_rank() as usize))
            .collect();
        let kit = crate::gameplay::talents::TalentKit::resolve(&c.talents, &all);
        if kit.player.damage_taken_pct < -20.0 || kit.player.invuln_secs > 0.15 {
            r.0.push(format!(
                "companions.ron: {}: al massimo danni subiti {}% e invulnerabilità +{} s \
                 (limiti -20% e 0,15 s)",
                c.id, kit.player.damage_taken_pct, kit.player.invuln_secs
            ));
        }
        if points >= xp.max_level {
            r.0.push(format!(
                "companions.ron: {}: l'albero ha {points} gradi ma i punti sono {} \
                 (companion_xp.max_level − 1)",
                c.id,
                xp.max_level - 1
            ));
        }
    }
}

/// Valori di un effetto: percentuali e quantità finite, frazioni tra 0 e 1, tempi positivi.
fn effect_values(r: &mut Report, at: &str, effect: &TalentEffect) {
    let ok = match *effect {
        TalentEffect::SlowFactor(v) | TalentEffect::ChainFalloff(v) => v > 0.0 && v <= 1.0,
        TalentEffect::SlowSecs(v)
        | TalentEffect::PlayerMaxHp(v)
        | TalentEffect::PlayerRegen(v)
        | TalentEffect::PlayerInvulnSecs(v) => v.is_finite() && v > 0.0,
        TalentEffect::PlayerDamageTakenPct(v) => v.is_finite() && v < 0.0 && v > -20.0,
        TalentEffect::DamagePct(v)
        | TalentEffect::IntervalPct(v)
        | TalentEffect::RangePct(v)
        | TalentEffect::AngularSpeedPct(v)
        | TalentEffect::PlayerSpeedPct(v)
        | TalentEffect::PlayerPickupPct(v)
        | TalentEffect::PlayerCoinPct(v) => v.is_finite() && v != 0.0 && v > -90.0,
        TalentEffect::ExtraCount(n) => n > 0,
        TalentEffect::Explode { radius, damage_pct } => radius > 0.0 && damage_pct > 0.0,
        TalentEffect::Cone {
            every,
            half_angle_deg,
            range,
            damage_pct,
        } => {
            every > 0
                && half_angle_deg > 0.0
                && half_angle_deg < 90.0
                && range > 0.0
                && damage_pct > 0.0
        }
        TalentEffect::HealPerHit { hp, cap } => hp > 0.0 && cap >= hp,
        TalentEffect::Echo {
            delay_s,
            damage_pct,
        } => delay_s > 0.0 && damage_pct > 0.0,
        TalentEffect::Strike {
            every_s,
            range,
            radius,
            damage_pct,
        } => every_s > 0.0 && range > 0.0 && radius > 0.0 && damage_pct > 0.0,
        TalentEffect::Burn {
            dps_pct,
            duration_s,
        } => dps_pct > 0.0 && duration_s > 0.0,
    };
    if !ok {
        r.0.push(format!("{at}: {effect:?} ha un valore fuori dai limiti"));
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{assert_error, errors_after};

    #[test]
    fn talents_are_checked() {
        assert!(errors_after(|_| {}).is_empty());
        assert_error(
            |c| {
                let id = c.companions[0].talents[0].nodes[0].id.clone();
                c.companions[1].talents[0].nodes[0].id = id;
            },
            "duplicato",
        );
        assert_error(
            |c| c.companions[0].talents[0].nodes[0].requires = vec!["nessuno".into()],
            "non è un nodo di questo ramo",
        );
        assert_error(
            |c| {
                let below = c.companions[0].talents[0].nodes[1].id.clone();
                c.companions[0].talents[0].nodes[0].requires = vec![below];
            },
            "riga più in alto",
        );
        assert_error(
            |c| c.companions[0].talents[0].nodes[0].col = 7,
            "fuori dalla griglia",
        );
        assert_error(
            |c| c.companions[0].talents[0].nodes[0].ranks.clear(),
            "gradi",
        );
        assert_error(
            |c| c.companions[0].talents[0].nodes[0].name = "Freccia →".into(),
            "non lo disegnano",
        );
        assert_error(|c| c.progression.companion_xp.max_level = 2, "l'albero ha");
    }
}
