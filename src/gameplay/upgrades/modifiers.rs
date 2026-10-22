//! Effetto complessivo degli stack: fattori sui valori base e bonus da sommare, calcolati dai
//! soli dati e stack. Lo usa il ricalcolo a caldo; `pct_factor` anche la validazione dei dati.

use bevy::prelude::*;

use super::UpgradeStacks;
use crate::data::schema::{AttackData, Effect, UpgradeData};
use crate::gameplay::attacks::AutoAttack;

/// Fattore di un bonus percentuale sommato: +30 → 1,3; −50 → 0,5.
pub fn pct_factor(total_pct: f32) -> f32 {
    1.0 + total_pct / 100.0
}

/// Effetto complessivo degli stack: fattori sui valori base e bonus da sommare.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Modifiers {
    pub damage: f32,
    pub interval: f32,
    pub range: f32,
    pub projectile_speed: f32,
    pub move_speed: f32,
    pub pickup_radius: f32,
    pub extra_projectiles: u32,
    pub extra_pierce: u32,
    pub max_hp_bonus: f32,
    pub regen_per_s: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::from_stacks(&[], &UpgradeStacks::default())
    }
}

impl Modifiers {
    pub fn from_stacks(upgrades: &[UpgradeData], stacks: &UpgradeStacks) -> Self {
        let (mut damage, mut interval, mut range, mut speed, mut moving, mut pickup) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut m = Self {
            damage: 1.0,
            interval: 1.0,
            range: 1.0,
            projectile_speed: 1.0,
            move_speed: 1.0,
            pickup_radius: 1.0,
            extra_projectiles: 0,
            extra_pierce: 0,
            max_hp_bonus: 0.0,
            regen_per_s: 0.0,
        };
        for (i, upgrade) in upgrades.iter().enumerate() {
            let n = stacks.of(i);
            let k = n as f32;
            match upgrade.effect {
                Effect::CompanionDamagePct(v) => damage += v * k,
                Effect::CompanionIntervalPct(v) => interval += v * k,
                Effect::CompanionRangePct(v) => range += v * k,
                Effect::ProjectileSpeedPct(v) => speed += v * k,
                Effect::MoveSpeedPct(v) => moving += v * k,
                Effect::PickupRadiusPct(v) => pickup += v * k,
                Effect::CompanionProjectiles(v) => {
                    m.extra_projectiles = m.extra_projectiles.saturating_add(v.saturating_mul(n));
                }
                Effect::CompanionPierce(v) => {
                    m.extra_pierce = m.extra_pierce.saturating_add(v.saturating_mul(n));
                }
                Effect::MaxHp(v) => m.max_hp_bonus += v * k,
                Effect::RegenPerSecond(v) => m.regen_per_s += v * k,
                Effect::Heal(_) => {} // immediato, nessun effetto permanente
            }
        }
        m.damage = pct_factor(damage);
        m.interval = pct_factor(interval);
        m.range = pct_factor(range);
        m.projectile_speed = pct_factor(speed);
        m.move_speed = pct_factor(moving);
        m.pickup_radius = pct_factor(pickup);
        m
    }

    /// Attacco dai dati base con gli upgrade. La durata del proiettile segue gittata e
    /// velocità, così la portata cresce con la gittata e resta almeno pari a essa.
    pub fn attack(&self, data: &AttackData) -> AutoAttack {
        let mut attack = AutoAttack::from_data(data);
        attack.damage *= self.damage;
        attack.interval *= self.interval;
        attack.range *= self.range;
        attack.projectile_speed *= self.projectile_speed;
        attack.projectile_lifetime *= self.range / self.projectile_speed;
        attack.projectiles = attack.projectiles.saturating_add(self.extra_projectiles);
        // saturating: `pierce` enorme (es. u32::MAX = "infinito") resta tale
        attack.pierce = attack.pierce.saturating_add(self.extra_pierce);
        attack
    }
}
