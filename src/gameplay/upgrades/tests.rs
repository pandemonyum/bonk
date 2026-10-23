//! Estrazione del draft e calcolo degli upgrade. Id e valori vengono dai dati veri, salvo
//! le liste costruite apposta per provare una regola.

use super::*;
use crate::data::schema::{AttackData, Rarity, RarityWeights, UpgradeData};

/// Squadra vuota e senza posti: per le liste di carte costruite dai test senza compagni.
const NO_PARTY: PartyView = PartyView {
    members: Vec::new(),
    free_slots: 0,
    has_projectiles: true,
    has_counted: true,
};

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
        let choices = draw_choices(
            &mut rng,
            &config.upgrades,
            &stacks,
            &PartyView::of(&config, &stacks, &config.stage.starting_companion),
            &WEIGHTS,
            3,
            false,
        );
        assert_eq!(choices.len(), 3);
        for (k, &i) in choices.iter().enumerate() {
            assert!(is_eligible(
                &config.upgrades,
                &stacks,
                i,
                &PartyView::of(&config, &stacks, &config.stage.starting_companion)
            ));
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
    assert_eq!(
        draw_choices(&mut rng, &upgrades, &fresh, &NO_PARTY, &no_rare, 3, false),
        [0]
    );
    // "a" al massimo: resta "b"
    let after_a = UpgradeStacks(vec![1, 0, 0]);
    assert_eq!(
        draw_choices(&mut rng, &upgrades, &after_a, &NO_PARTY, &no_rare, 3, false),
        [1]
    );
    // tutto al massimo: draft vuoto
    let maxed = UpgradeStacks(vec![1, 2, 0]);
    assert!(draw_choices(&mut rng, &upgrades, &maxed, &NO_PARTY, &no_rare, 3, false).is_empty());
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
        .filter(|_| {
            draw_choices(&mut rng, &upgrades, &stacks, &NO_PARTY, &WEIGHTS, 1, false) == [1]
        })
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
            .map(|_| {
                draw_choices(
                    &mut rng,
                    &config.upgrades,
                    &stacks,
                    &PartyView::of(&config, &stacks, &config.stage.starting_companion),
                    &WEIGHTS,
                    3,
                    false,
                )
            })
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

fn attack_with(config: &GameConfig, has: impl Fn(&AttackData) -> bool) -> &AttackData {
    config
        .attacks
        .iter()
        .find(|a| has(a))
        .expect("i dati hanno un attacco con questo pattern")
}

#[test]
fn a_modified_attack_still_reaches_its_range() {
    let config = GameConfig::embedded();
    let data = attack_with(&config, |a| a.projectile().is_some());
    let p = data.projectile().unwrap();
    let m = Modifiers {
        range: 1.6,
        projectile_speed: 1.4,
        damage: 2.0,
        extra_pierce: 2,
        ..Modifiers::default()
    };
    let attack = m.projectile(data, p);
    assert_eq!(attack.range, data.range * 1.6);
    assert_eq!(attack.damage, data.damage * 2.0);
    assert_eq!(attack.pierce, p.pierce + 2);
    let travel = attack.projectile_speed * attack.projectile_lifetime;
    let base_travel = p.speed * p.lifetime_s;
    assert!((travel - base_travel * 1.6).abs() < 1e-3);
    assert!(travel >= attack.range);
}

/// Gli upgrade "Companion" valgono anche per onda e foglie: area e orbita crescono con la
/// gittata, le foglie in più vengono da "+1 proiettile".
#[test]
fn companion_upgrades_apply_to_every_pattern() {
    let config = GameConfig::embedded();
    let m = Modifiers {
        damage: 1.5,
        interval: 0.8,
        range: 1.2,
        extra_projectiles: 2,
        ..Modifiers::default()
    };
    let wave = attack_with(&config, |a| a.pulse().is_some());
    let pulse = m.pulse(wave, wave.pulse().unwrap());
    assert_eq!(pulse.damage, wave.damage * 1.5);
    assert_eq!(pulse.interval, wave.interval_s * 0.8);
    assert_eq!(pulse.radius, wave.range * 1.2);
    let leaves = attack_with(&config, |a| a.orbit().is_some());
    let orbit = m.orbit(leaves, leaves.orbit().unwrap());
    assert_eq!(orbit.damage, leaves.damage * 1.5);
    assert_eq!(orbit.rehit_interval, leaves.interval_s * 0.8);
    assert_eq!(orbit.radius, leaves.range * 1.2);
    assert_eq!(orbit.leaves, leaves.orbit().unwrap().count + 2);
}

/// Le carte "nuovo compagno" escono solo con un posto libero e una volta sola.
#[test]
fn companion_cards_need_a_free_slot() {
    let upgrades = vec![
        upgrade(
            "join",
            Rarity::Common,
            1,
            &[],
            Effect::JoinCompanion("x".into()),
        ),
        upgrade("snack", Rarity::Common, 9, &[], Effect::Heal(1.0)),
    ];
    let party = |members: &[&str], free_slots| PartyView {
        members: members.iter().map(|m| m.to_string()).collect(),
        free_slots,
        has_projectiles: true,
        has_counted: true,
    };
    let fresh = UpgradeStacks(vec![0, 0]);
    assert!(is_eligible(&upgrades, &fresh, 0, &party(&["a"], 1)));
    assert!(
        !is_eligible(&upgrades, &fresh, 0, &party(&["a"], 0)),
        "nessun posto libero"
    );
    assert!(
        is_eligible(&upgrades, &fresh, 1, &party(&["a"], 0)),
        "le altre carte non dipendono dai posti"
    );
    // rilievo del code review P0.5: il compagno è già in squadra (per esempio è quello iniziale)
    assert!(
        !is_eligible(&upgrades, &fresh, 0, &party(&["x"], 2)),
        "già in squadra"
    );
    let joined = UpgradeStacks(vec![1, 0]);
    assert!(
        !is_eligible(&upgrades, &joined, 0, &party(&["a"], 1)),
        "già arrivato"
    );
}

#[test]
fn the_party_view_follows_the_data_and_the_cards() {
    let config = GameConfig::embedded();
    let none = UpgradeStacks(vec![0; config.upgrades.len()]);
    let slots = config.player.companion_slots.len();
    let start = PartyView::of(&config, &none, &config.stage.starting_companion);
    assert_eq!(
        start.members,
        std::slice::from_ref(&config.stage.starting_companion)
    );
    assert_eq!(start.free_slots, slots - 1);
    let (first_join, id) = config
        .upgrades
        .iter()
        .enumerate()
        .find_map(|(i, u)| match &u.effect {
            Effect::JoinCompanion(id) if *id != config.stage.starting_companion => {
                Some((i, id.clone()))
            }
            _ => None,
        })
        .expect("i dati hanno una carta per un compagno diverso dall'iniziale");
    let mut one = none.clone();
    one.0[first_join] = 1;
    let after = PartyView::of(&config, &one, &config.stage.starting_companion);
    assert_eq!(after.free_slots, slots - 2);
    assert!(after.members.contains(&id));
    // una carta per il compagno iniziale non occupa un posto in più
    let mut same = config.clone();
    same.stage.starting_companion = id.clone();
    let view = PartyView::of(&same, &one, &same.stage.starting_companion);
    assert_eq!((view.members.len(), view.free_slots), (1, slots - 1));
}

#[test]
fn the_auto_pick_takes_a_new_companion_when_offered() {
    let upgrades = vec![
        upgrade(
            "dmg",
            Rarity::Common,
            5,
            &[],
            Effect::CompanionDamagePct(15.0),
        ),
        upgrade(
            "join",
            Rarity::Common,
            1,
            &[],
            Effect::JoinCompanion("x".into()),
        ),
        upgrade("snack", Rarity::Common, 9, &[], Effect::Heal(1.0)),
    ];
    assert_eq!(auto_choice(&upgrades, &[0, 1, 2]), 1);
    assert_eq!(
        auto_choice(&upgrades, &[2, 0]),
        0,
        "senza compagni la prima carta"
    );
    assert_eq!(auto_choice(&upgrades, &[]), 0);
}

/// Rilievo del code review P0.6: senza nessuno che spara, perforazione e velocità dei
/// proiettili non servono; "+1" serve a proiettili e foglie, non all'onda.
#[test]
fn cards_only_appear_when_someone_can_use_them() {
    let config = GameConfig::embedded();
    let none = UpgradeStacks(vec![0; config.upgrades.len()]);
    let with = |has: fn(&AttackData) -> bool| {
        config
            .companions
            .iter()
            .find(|c| config.attack(&c.attack).is_some_and(has))
            .unwrap()
            .id
            .clone()
    };
    let shooter = PartyView::of(&config, &none, &with(|a| a.projectile().is_some()));
    assert!(shooter.has_projectiles && shooter.has_counted);
    let wave = PartyView::of(&config, &none, &with(|a| a.pulse().is_some()));
    assert!(!wave.has_projectiles && !wave.has_counted);
    let leaves = PartyView::of(&config, &none, &with(|a| a.orbit().is_some()));
    assert!(!leaves.has_projectiles && leaves.has_counted);
    let pierce = config
        .upgrades
        .iter()
        .position(|u| matches!(u.effect, Effect::CompanionPierce(_)))
        .unwrap();
    assert!(is_eligible(&config.upgrades, &none, pierce, &shooter));
    assert!(!is_eligible(&config.upgrades, &none, pierce, &wave));
}

/// Nei primi draft, con un posto libero, c'è sempre un "nuovo compagno"; a squadra piena la
/// garanzia non inventa carte, e senza garanzia l'estrazione resta quella di sempre.
#[test]
fn early_drafts_always_offer_a_companion_while_a_slot_is_free() {
    let config = GameConfig::embedded();
    let stacks = UpgradeStacks(vec![0; config.upgrades.len()]);
    let party = PartyView::of(&config, &stacks, &config.stage.starting_companion);
    assert!(party.free_slots > 0);
    let is_join = |i: &usize| matches!(config.upgrades[*i].effect, Effect::JoinCompanion(_));
    let mut without = 0;
    for seed in 0..400 {
        let draw = |guaranteed| {
            let mut rng = Pcg32::new(seed, 2);
            draw_choices(
                &mut rng,
                &config.upgrades,
                &stacks,
                &party,
                &WEIGHTS,
                3,
                guaranteed,
            )
        };
        let plain = draw(false);
        let guaranteed = draw(true);
        assert_eq!(guaranteed.len(), 3);
        assert!(
            guaranteed.iter().any(is_join),
            "seed {seed}: {guaranteed:?}"
        );
        if plain.iter().any(is_join) {
            assert_eq!(plain, guaranteed, "seed {seed}: cambia solo se serve");
        } else {
            without += 1;
            assert_eq!(
                plain[..2],
                guaranteed[..2],
                "seed {seed}: cambia solo l'ultima"
            );
        }
    }
    assert!(
        without > 0,
        "senza garanzia qualche draft resta senza compagni"
    );
    // squadra piena: nessuna carta compagno da garantire
    let full: Vec<u32> = config
        .upgrades
        .iter()
        .map(|u| u32::from(matches!(u.effect, Effect::JoinCompanion(_))))
        .collect();
    let full = UpgradeStacks(full);
    let party = PartyView::of(&config, &full, &config.stage.starting_companion);
    let mut rng = Pcg32::new(7, 2);
    let choices = draw_choices(&mut rng, &config.upgrades, &full, &party, &WEIGHTS, 3, true);
    assert!(!choices.iter().any(is_join));
}
