//! Talenti dei compagni nella run. I nodi imparati (dal profilo salvato, `profile`) si
//! risolvono in un `TalentKit` sul compagno: statistiche che si sommano alle carte e meccaniche
//! nuove (esplosione, cono, cura, eco, colpo periodico, bruciatura) di cui vale l'ultimo nodo
//! preso del ramo. Le meccaniche sono in `talent_effects`.

use bevy::prelude::*;

use super::chain::ChainAttack;
use super::orbit::OrbitAttack;
use super::pulse::PulseAttack;
use super::upgrades::Modifiers;
use crate::data::schema::{TalentBranch, TalentEffect, TalentNode};

/// Nodi imparati per compagno (id del compagno, id dei nodi), dal profilo
/// (`profile::Profile::learned`) quando il compagno entra in squadra.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LearnedTalents(pub Vec<(String, Vec<String>)>);

impl LearnedTalents {
    pub fn of(&self, companion: &str) -> &[String] {
        self.0
            .iter()
            .find(|(id, _)| id == companion)
            .map_or(&[], |(_, nodes)| nodes.as_slice())
    }
}

/// Statistiche dei talenti di un compagno (si sommano alle carte).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TalentStats {
    pub damage_pct: f32,
    pub interval_pct: f32,
    pub range_pct: f32,
    pub extra_count: u32,
    pub slow_factor: Option<f32>,
    pub slow_secs: Option<f32>,
    pub falloff: Option<f32>,
    pub angular_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Explode {
    pub radius: f32,
    pub damage_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cone {
    pub every: u32,
    pub half_angle: f32,
    pub range: f32,
    pub damage_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealPerHit {
    pub hp: f32,
    pub cap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Echo {
    pub delay: f32,
    pub damage_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strike {
    pub every: f32,
    pub range: f32,
    pub radius: f32,
    pub damage_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burn {
    pub dps_pct: f32,
    pub duration: f32,
}

/// Tutto quello che i talenti imparati danno a un compagno.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct TalentKit {
    pub stats: TalentStats,
    pub explode: Option<Explode>,
    pub cone: Option<Cone>,
    pub heal: Option<HealPerHit>,
    pub echo: Option<Echo>,
    pub strike: Option<Strike>,
    pub burn: Option<Burn>,
    pub player: PlayerSupport,
}

/// Grado di un nodo nei nodi imparati (`learned` ripete l'id una volta per grado), al
/// massimo quello del nodo.
pub fn node_rank(learned: &[String], node: &TalentNode) -> u32 {
    (learned.iter().filter(|id| **id == node.id).count() as u32).min(node.max_rank())
}

/// I nodi attivi con il loro grado: imparati e con i requisiti soddisfatti (ognuno almeno al
/// grado 1), nell'ordine dei dati.
pub fn active_nodes<'a>(
    branches: &'a [TalentBranch],
    learned: &[String],
) -> Vec<(&'a TalentNode, u32)> {
    branches
        .iter()
        .flat_map(|b| b.nodes.iter())
        .filter_map(|node| {
            let rank = node_rank(learned, node);
            let open = node.requires.iter().all(|r| learned.contains(r));
            (rank > 0 && open).then_some((node, rank))
        })
        .collect()
}

/// Aiuto al player dei talenti di un compagno in squadra.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerSupport {
    pub max_hp: f32,
    pub regen: f32,
    pub speed_pct: f32,
    pub pickup_pct: f32,
    pub coin_pct: f32,
    pub damage_taken_pct: f32,
    pub invuln_secs: f32,
}

impl PlayerSupport {
    pub fn add(&mut self, other: &PlayerSupport) {
        self.max_hp += other.max_hp;
        self.regen += other.regen;
        self.speed_pct += other.speed_pct;
        self.pickup_pct += other.pickup_pct;
        self.coin_pct += other.coin_pct;
        self.damage_taken_pct += other.damage_taken_pct;
        self.invuln_secs += other.invuln_secs;
    }
}

impl TalentKit {
    pub fn resolve(branches: &[TalentBranch], learned: &[String]) -> Self {
        let mut kit = Self::default();
        for (node, rank) in active_nodes(branches, learned) {
            for effect in node.ranks.iter().take(rank as usize).flatten() {
                kit.apply(effect);
            }
        }
        kit
    }

    fn apply(&mut self, effect: &TalentEffect) {
        let s = &mut self.stats;
        match *effect {
            TalentEffect::DamagePct(v) => s.damage_pct += v,
            TalentEffect::IntervalPct(v) => s.interval_pct += v,
            TalentEffect::RangePct(v) => s.range_pct += v,
            TalentEffect::ExtraCount(n) => s.extra_count += n,
            TalentEffect::SlowFactor(v) => s.slow_factor = Some(v),
            TalentEffect::SlowSecs(v) => s.slow_secs = Some(v),
            TalentEffect::ChainFalloff(v) => s.falloff = Some(v),
            TalentEffect::AngularSpeedPct(v) => s.angular_pct += v,
            TalentEffect::Explode { radius, damage_pct } => {
                self.explode = Some(Explode { radius, damage_pct });
            }
            TalentEffect::Cone {
                every,
                half_angle_deg,
                range,
                damage_pct,
            } => {
                self.cone = Some(Cone {
                    every: every.max(1),
                    half_angle: half_angle_deg.to_radians(),
                    range,
                    damage_pct,
                });
            }
            TalentEffect::HealPerHit { hp, cap } => self.heal = Some(HealPerHit { hp, cap }),
            TalentEffect::Echo {
                delay_s,
                damage_pct,
            } => {
                self.echo = Some(Echo {
                    delay: delay_s,
                    damage_pct,
                });
            }
            TalentEffect::Strike {
                every_s,
                range,
                radius,
                damage_pct,
            } => {
                self.strike = Some(Strike {
                    every: every_s,
                    range,
                    radius,
                    damage_pct,
                });
            }
            TalentEffect::Burn {
                dps_pct,
                duration_s,
            } => {
                self.burn = Some(Burn {
                    dps_pct,
                    duration: duration_s,
                });
            }
            TalentEffect::PlayerMaxHp(v) => self.player.max_hp += v,
            TalentEffect::PlayerRegen(v) => self.player.regen += v,
            TalentEffect::PlayerSpeedPct(v) => self.player.speed_pct += v,
            TalentEffect::PlayerPickupPct(v) => self.player.pickup_pct += v,
            TalentEffect::PlayerCoinPct(v) => self.player.coin_pct += v,
            TalentEffect::PlayerDamageTakenPct(v) => self.player.damage_taken_pct += v,
            TalentEffect::PlayerInvulnSecs(v) => self.player.invuln_secs += v,
        }
    }

    /// Rallentamento dell'onda scelto dai talenti, se c'è.
    pub fn tune_pulse(&self, attack: &mut PulseAttack) {
        attack.slow = self.stats.slow_factor.unwrap_or(attack.slow);
        attack.slow_secs = self.stats.slow_secs.unwrap_or(attack.slow_secs);
    }

    /// Calo per salto del fulmine scelto dai talenti, se c'è.
    pub fn tune_chain(&self, attack: &mut ChainAttack) {
        attack.falloff = self.stats.falloff.unwrap_or(attack.falloff);
    }

    /// Foglie più veloci.
    pub fn tune_orbit(&self, attack: &mut OrbitAttack) {
        attack.angular_speed *= 1.0 + self.stats.angular_pct / 100.0;
    }

    /// I modificatori delle carte con in più le statistiche dei talenti (le percentuali si
    /// sommano a quelle delle carte).
    pub fn modifiers(&self, cards: &Modifiers) -> Modifiers {
        let s = &self.stats;
        Modifiers {
            damage: cards.damage + s.damage_pct / 100.0,
            interval: cards.interval + s.interval_pct / 100.0,
            range: cards.range + s.range_pct / 100.0,
            extra_projectiles: cards.extra_projectiles.saturating_add(s.extra_count),
            ..*cards
        }
    }
}

/// Stato delle meccaniche dei talenti di un compagno: attacchi contati per il cono, eco in
/// attesa, carica del colpo periodico.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct TalentState {
    pub attacks: u32,
    /// Secondi all'eco, se ce n'è una in attesa.
    pub echo_in: Option<f32>,
    pub strike_charge: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;

    fn tree_of(id: &str) -> Vec<TalentBranch> {
        GameConfig::embedded()
            .companion(id)
            .expect("compagno dei dati")
            .talents
            .clone()
    }

    fn ids(nodes: &[&str]) -> Vec<String> {
        nodes.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn a_node_counts_only_with_the_one_before_it() {
        let ember = tree_of("ember");
        let first = &ember[0].nodes[0].id;
        let second = &ember[0].nodes[1].id;
        assert_eq!(active_nodes(&ember, &ids(&[second])).len(), 0);
        assert_eq!(
            active_nodes(&ember, &[first.clone(), second.clone()]).len(),
            2
        );
        assert!(TalentKit::resolve(&ember, &[]) == TalentKit::default());
    }

    #[test]
    fn ranks_add_stats_and_later_ranks_replace_the_mechanic() {
        use crate::data::schema::{BranchTheme, TalentNode};
        let node = |id: &str, requires: &[&str], ranks: Vec<Vec<TalentEffect>>| TalentNode {
            id: id.into(),
            name: String::new(),
            description: String::new(),
            col: 0,
            row: 0,
            requires: requires.iter().map(|r| r.to_string()).collect(),
            ranks,
        };
        let blast = |radius, damage_pct| TalentEffect::Explode { radius, damage_pct };
        let branch = TalentBranch {
            name: String::new(),
            theme: BranchTheme::Element,
            nodes: vec![
                node(
                    "a",
                    &[],
                    vec![vec![blast(16.0, 4.0)], vec![blast(20.0, 6.0)]],
                ),
                node("b", &["a"], vec![vec![TalentEffect::DamagePct(2.0)]; 3]),
            ],
        };
        let tree = std::slice::from_ref(&branch);
        let one = TalentKit::resolve(tree, &ids(&["a"]));
        let two = TalentKit::resolve(tree, &ids(&["a", "a"]));
        assert_eq!(one.explode.unwrap().radius, 16.0);
        assert_eq!(two.explode.unwrap().radius, 20.0, "il grado 2 sostituisce");
        let stats = TalentKit::resolve(tree, &ids(&["a", "b", "b"]));
        assert_eq!(
            stats.stats.damage_pct, 4.0,
            "i gradi delle statistiche si sommano"
        );
        assert_eq!(TalentKit::resolve(tree, &ids(&["b"])), TalentKit::default());
        // gli alberi veri: tutto imparato dà ogni meccanica e l'aiuto al player
        let ember = tree_of("ember");
        let all: Vec<String> = ember
            .iter()
            .flat_map(|b| b.nodes.iter())
            .flat_map(|n| std::iter::repeat_n(n.id.clone(), n.max_rank() as usize))
            .collect();
        let kit = TalentKit::resolve(&ember, &all);
        assert!(kit.explode.is_some() && kit.cone.is_some() && kit.burn.is_some());
        assert!(kit.player.max_hp > 0.0 && kit.player.coin_pct > 0.0);
    }

    #[test]
    fn talent_stats_add_to_the_cards() {
        let cards = Modifiers::default();
        let kit = TalentKit {
            stats: TalentStats {
                damage_pct: 20.0,
                interval_pct: -10.0,
                extra_count: 1,
                ..default()
            },
            ..default()
        };
        let m = kit.modifiers(&cards);
        assert!((m.damage - 1.2).abs() < 1e-6);
        assert!((m.interval - 0.9).abs() < 1e-6);
        assert_eq!(m.extra_projectiles, 1);
        assert_eq!(m.move_speed, cards.move_speed);
    }
}
