//! Talenti dei compagni nella run. I nodi imparati (dal profilo salvato, `profile`) si
//! risolvono in un `TalentKit` sul compagno: statistiche che si sommano alle carte e meccaniche
//! nuove (esplosione, cono, cura, eco, colpo periodico, bruciatura) di cui vale l'ultimo nodo
//! preso del ramo. Le meccaniche sono in `talent_effects`.

use bevy::prelude::*;

use super::chain::ChainAttack;
use super::orbit::OrbitAttack;
use super::pulse::PulseAttack;
use super::upgrades::Modifiers;
use crate::data::schema::{TalentBranch, TalentEffect};

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
}

/// I nodi davvero attivi: in ogni ramo quelli imparati di fila dal primo (un nodo senza il
/// precedente non vale), nell'ordine dei dati.
pub fn active_nodes<'a>(
    branches: &'a [TalentBranch],
    learned: &[String],
) -> Vec<&'a crate::data::schema::TalentNode> {
    branches
        .iter()
        .flat_map(|b| b.nodes.iter().take_while(|n| learned.contains(&n.id)))
        .collect()
}

impl TalentKit {
    pub fn resolve(branches: &[TalentBranch], learned: &[String]) -> Self {
        let mut kit = Self::default();
        for node in active_nodes(branches, learned) {
            for effect in &node.effects {
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

    fn tree(id: &str) -> Vec<TalentBranch> {
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
        let ember = tree("ember");
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
    fn later_nodes_of_a_branch_replace_the_mechanic_and_stats_add_up() {
        let ember = tree("ember");
        let a: Vec<String> = ember[0].nodes.iter().map(|n| n.id.clone()).collect();
        let one = TalentKit::resolve(&ember, &a[..1]);
        let two = TalentKit::resolve(&ember, &a[..2]);
        let (e1, e2) = (one.explode.unwrap(), two.explode.unwrap());
        assert!(e2.radius > e1.radius && e2.damage_pct > e1.damage_pct);
        assert!(TalentKit::resolve(&ember, &a).burn.is_some());
        let sprout = tree("sprout");
        let all: Vec<String> = sprout
            .iter()
            .flat_map(|b| b.nodes.iter().map(|n| n.id.clone()))
            .collect();
        let kit = TalentKit::resolve(&sprout, &all);
        assert_eq!(kit.stats.extra_count, 1);
        assert!(kit.stats.range_pct > 0.0 && kit.stats.angular_pct > 0.0);
        assert!(kit.strike.is_some() && kit.burn.is_some());
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
