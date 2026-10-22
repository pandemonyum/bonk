//! Estrazione del draft e calcolo degli upgrade. Id e valori vengono dai dati veri, salvo
//! le liste costruite apposta per provare una regola.

use super::*;
use crate::data::schema::Rarity;

fn upgrade(
    id: &str,
    rarity: Rarity,
    max_stack: u32,
    requires: &[&str],
    effect: Effect,
) -> UpgradeData {
    UpgradeData {
        id: id.into(),
        name: id.into(),
        description: id.into(),
        rarity,
        max_stack,
        requires: requires.iter().map(|r| r.to_string()).collect(),
        effect,
    }
}

const WEIGHTS: RarityWeights = RarityWeights {
    common: 10.0,
    uncommon: 5.0,
    rare: 2.0,
};

#[test]
fn choices_are_distinct_eligible_and_at_most_the_draft_size() {
    let config = GameConfig::embedded();
    let stacks = UpgradeStacks(vec![0; config.upgrades.len()]);
    let mut rng = Pcg32::new(1, DRAFT_STREAM);
    for _ in 0..200 {
        let choices = draw_choices(&mut rng, &config.upgrades, &stacks, &WEIGHTS, 3);
        assert_eq!(choices.len(), 3);
        for (k, &i) in choices.iter().enumerate() {
            assert!(is_eligible(&config.upgrades, &stacks, i));
            assert!(!choices[..k].contains(&i), "doppione in {choices:?}");
        }
    }
}

#[test]
fn requirements_max_stacks_and_zero_weights_are_respected() {
    let upgrades = vec![
        upgrade("a", Rarity::Common, 1, &[], Effect::Heal(1.0)),
        upgrade("b", Rarity::Common, 2, &["a"], Effect::Heal(1.0)),
        upgrade("c", Rarity::Rare, 5, &[], Effect::Heal(1.0)),
    ];
    let no_rare = RarityWeights {
        rare: 0.0,
        ..WEIGHTS
    };
    let mut rng = Pcg32::new(3, DRAFT_STREAM);
    // all'inizio solo "a": "b" richiede "a", "c" ha peso 0
    let fresh = UpgradeStacks(vec![0, 0, 0]);
    assert_eq!(draw_choices(&mut rng, &upgrades, &fresh, &no_rare, 3), [0]);
    // "a" al massimo: resta "b"
    let after_a = UpgradeStacks(vec![1, 0, 0]);
    assert_eq!(
        draw_choices(&mut rng, &upgrades, &after_a, &no_rare, 3),
        [1]
    );
    // tutto al massimo: draft vuoto
    let maxed = UpgradeStacks(vec![1, 2, 0]);
    assert!(draw_choices(&mut rng, &upgrades, &maxed, &no_rare, 3).is_empty());
}

#[test]
fn rarer_cards_come_up_less_often() {
    let upgrades = vec![
        upgrade("common", Rarity::Common, 9, &[], Effect::Heal(1.0)),
        upgrade("rare", Rarity::Rare, 9, &[], Effect::Heal(1.0)),
    ];
    let stacks = UpgradeStacks(vec![0, 0]);
    let mut rng = Pcg32::new(7, DRAFT_STREAM);
    let draws = 6000;
    let rare = (0..draws)
        .filter(|_| draw_choices(&mut rng, &upgrades, &stacks, &WEIGHTS, 1) == [1])
        .count();
    let expected = draws as f32 * WEIGHTS.rare / (WEIGHTS.common + WEIGHTS.rare);
    assert!(
        (rare as f32 - expected).abs() < expected * 0.1,
        "rare {rare}, attese ~{expected:.0}"
    );
}

#[test]
fn the_same_seed_draws_the_same_cards() {
    let config = GameConfig::embedded();
    let stacks = UpgradeStacks(vec![0; config.upgrades.len()]);
    let draw = |seed| {
        let mut rng = Pcg32::new(seed, DRAFT_STREAM);
        (0..20)
            .map(|_| draw_choices(&mut rng, &config.upgrades, &stacks, &WEIGHTS, 3))
            .collect::<Vec<_>>()
    };
    assert_eq!(draw(5), draw(5));
    assert_ne!(draw(5), draw(6));
}

#[test]
fn stacks_add_up_on_the_base_values() {
    let upgrades = vec![
        upgrade(
            "dmg",
            Rarity::Common,
            5,
            &[],
            Effect::CompanionDamagePct(15.0),
        ),
        upgrade(
            "rate",
            Rarity::Common,
            5,
            &[],
            Effect::CompanionIntervalPct(-10.0),
        ),
        upgrade(
            "shot",
            Rarity::Rare,
            3,
            &[],
            Effect::CompanionProjectiles(1),
        ),
        upgrade("hp", Rarity::Common, 5, &[], Effect::MaxHp(20.0)),
        upgrade("snack", Rarity::Common, 99, &[], Effect::Heal(30.0)),
    ];
    let m = Modifiers::from_stacks(&upgrades, &UpgradeStacks(vec![2, 3, 1, 2, 4]));
    assert!((m.damage - 1.3).abs() < 1e-6);
    assert!((m.interval - 0.7).abs() < 1e-6);
    assert_eq!(m.extra_projectiles, 1);
    assert_eq!(m.max_hp_bonus, 40.0);
    assert_eq!(m.regen_per_s, 0.0);
    assert_eq!(
        Modifiers::default(),
        Modifiers::from_stacks(&upgrades, &UpgradeStacks(vec![0; 5]))
    );
}

#[test]
fn a_modified_attack_still_reaches_its_range() {
    let config = GameConfig::embedded();
    let data = &config.attacks[0];
    let m = Modifiers {
        range: 1.6,
        projectile_speed: 1.4,
        damage: 2.0,
        extra_pierce: 2,
        ..Modifiers::default()
    };
    let attack = m.attack(data);
    assert_eq!(attack.range, data.range * 1.6);
    assert_eq!(attack.damage, data.damage * 2.0);
    assert_eq!(attack.pierce, data.pierce + 2);
    let travel = attack.projectile_speed * attack.projectile_lifetime;
    let base_travel = data.projectile_speed * data.projectile_lifetime_s;
    assert!((travel - base_travel * 1.6).abs() < 1e-3);
    assert!(travel >= attack.range);
}
