//! Regole del draft, pure e senza ECS: chi c'è in squadra, quali carte si possono proporre,
//! estrazione pesata per rarità, scelta automatica.

use super::UpgradeStacks;
use crate::data::GameConfig;
use crate::data::schema::{AttackPattern, Effect, RarityWeights, UpgradeData};
use crate::rng::Pcg32;

/// La squadra secondo il compagno iniziale e le carte prese: il primo, poi chi è arrivato con le
/// carte `JoinCompanion`, fino ai posti di `player.ron`. Serve al draft (stessa regola di
/// `companions::join_companions`, che crea davvero i compagni).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyView {
    pub members: Vec<String>,
    pub free_slots: usize,
    /// Qualcuno in squadra spara proiettili (perforazione e velocità servono solo a loro).
    pub has_projectiles: bool,
    /// Qualcuno ha proiettili o foglie ("+1 proiettile" vale per entrambi).
    pub has_counted: bool,
}

impl PartyView {
    /// `starter`: id del compagno iniziale della run (`RunSetup::starter`).
    pub fn of(config: &GameConfig, stacks: &UpgradeStacks, starter: &str) -> Self {
        let slots = config.player.companion_slots.len();
        let mut members = vec![starter.to_string()];
        for (i, upgrade) in config.upgrades.iter().enumerate() {
            if let Effect::JoinCompanion(id) = &upgrade.effect
                && stacks.of(i) > 0
                && !members.contains(id)
                && members.len() < slots
            {
                members.push(id.clone());
            }
        }
        let free_slots = slots.saturating_sub(members.len());
        let patterns: Vec<&AttackPattern> = members
            .iter()
            .filter_map(|id| config.companion(id))
            .map(|c| &config.companion_attack(c).pattern)
            .collect();
        let has_projectiles = patterns
            .iter()
            .any(|p| matches!(p, AttackPattern::Projectile(_)));
        let has_counted = patterns
            .iter()
            .any(|p| matches!(p, AttackPattern::Projectile(_) | AttackPattern::Orbit(_)));
        Self {
            members,
            free_slots,
            has_projectiles,
            has_counted,
        }
    }
}

/// Un upgrade si può proporre se non è al massimo, ha già tutti i requisiti e serve a
/// qualcuno in squadra (perforazione e velocità dei proiettili solo con chi spara, "+1" solo
/// con proiettili o foglie); una carta "nuovo compagno" solo se c'è un posto libero e il
/// compagno non è già in squadra.
pub fn is_eligible(
    upgrades: &[UpgradeData],
    stacks: &UpgradeStacks,
    index: usize,
    party: &PartyView,
) -> bool {
    let upgrade = &upgrades[index];
    let room = match &upgrade.effect {
        Effect::JoinCompanion(id) => party.free_slots > 0 && !party.members.contains(id),
        Effect::CompanionPierce(_) | Effect::ProjectileSpeedPct(_) => party.has_projectiles,
        Effect::CompanionProjectiles(_) => party.has_counted,
        _ => true,
    };
    room && stacks.of(index) < upgrade.max_stack
        && upgrade.requires.iter().all(|required| {
            upgrades
                .iter()
                .position(|u| u.id == *required)
                .is_some_and(|j| stacks.of(j) > 0)
        })
}

/// Fino a `count` upgrade diversi, estratti senza ripetizione con probabilità proporzionale
/// al peso della rarità. Meno carte se gli upgrade disponibili sono meno.
pub fn draw_choices(
    rng: &mut Pcg32,
    upgrades: &[UpgradeData],
    stacks: &UpgradeStacks,
    party: &PartyView,
    weights: &RarityWeights,
    count: usize,
) -> Vec<usize> {
    let mut pool: Vec<(usize, f32)> = (0..upgrades.len())
        .filter(|&i| is_eligible(upgrades, stacks, i, party))
        .map(|i| (i, weights.of(upgrades[i].rarity)))
        .filter(|&(_, weight)| weight > 0.0)
        .collect();
    let mut choices = Vec::with_capacity(count);
    while choices.len() < count && !pool.is_empty() {
        let total: f32 = pool.iter().map(|&(_, weight)| weight).sum();
        let mut roll = rng.next_f32() * total;
        let mut pick = pool.len() - 1; // ripiego per l'arrotondamento della somma
        for (k, &(_, weight)) in pool.iter().enumerate() {
            if roll < weight {
                pick = k;
                break;
            }
            roll -= weight;
        }
        choices.push(pool.remove(pick).0);
    }
    choices
}

/// Posizione nel draft della carta scelta da [`super::AutoPick`]: la prima "nuovo compagno" (la
/// squadra è il potenziamento più grande), altrimenti la prima carta.
pub fn auto_choice(upgrades: &[UpgradeData], choices: &[usize]) -> usize {
    choices
        .iter()
        .position(|&i| {
            upgrades
                .get(i)
                .is_some_and(|u| matches!(u.effect, Effect::JoinCompanion(_)))
        })
        .unwrap_or(0)
}
