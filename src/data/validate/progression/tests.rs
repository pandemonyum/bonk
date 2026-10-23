//! Test delle regole di `progression.ron` e `upgrades.ron`. Come per gli altri file, id e
//! numeri sono ricavati dai dati veri; le carte aggiunte dai test sono copie modificate.

use super::super::tests::{assert_error, embedded, errors_after};
use crate::data::GameConfig;
use crate::data::schema::{Effect, Rarity};
use crate::gameplay::SIM_HZ;
use crate::gameplay::attacks::MAX_PROJECTILES;
use crate::gameplay::progression::MAX_ENEMY_XP;
use crate::gameplay::upgrades::MAX_DRAFT_SIZE;

fn upgrade_where(c: &GameConfig, wanted: impl Fn(&Effect) -> bool) -> usize {
    c.upgrades
        .iter()
        .position(|u| wanted(&u.effect))
        .expect("effetto presente nei dati")
}

#[test]
fn progression_values_are_checked() {
    assert_error(
        |c| c.progression.xp_to_level.base = 0.0,
        "progression.ron: xp_to_level.base deve essere > 0",
    );
    assert_error(
        |c| c.progression.draft_size = 0,
        "progression.ron: draft_size = 0: serve da 1",
    );
    assert_error(
        |c| c.progression.gem.max_on_field = 0,
        "gem.max_on_field deve essere >= 1",
    );
    assert_error(
        |c| c.progression.rarity_weights.rare = -1.0,
        "rarity_weights.rare deve essere >= 0",
    );
    assert_error(
        |c| c.progression.gem.sprite = "gem.bmp".into(),
        "progression.ron: gem.sprite",
    );
    assert_error(
        |c| c.player.pickup_radius = 0.0,
        "player.ron: pickup_radius deve essere > 0",
    );
}

#[test]
fn upgrade_references_and_stacks_are_checked() {
    let c = embedded();
    let id = c.upgrades[0].id.clone();
    assert_error(
        |c| c.upgrades[0].requires = vec!["missing".into()],
        &format!("upgrades.ron: {id}.requires = \"missing\": non esiste in upgrades.ron"),
    );
    assert_error(
        |c| c.upgrades[0].requires = vec![id.clone()],
        "richiede sé stesso",
    );
    assert_error(
        |c| c.upgrades[0].max_stack = 0,
        &format!("upgrades.ron: {id}.max_stack deve essere >= 1"),
    );
    assert_error(
        |c| {
            let copy = c.upgrades[0].clone();
            c.upgrades.push(copy);
        },
        &format!("upgrades.ron: id \"{id}\" duplicato"),
    );
    // due upgrade che si richiedono a vicenda non verranno mai proposti
    assert_error(
        |c| {
            let (a, b) = (c.upgrades[0].id.clone(), c.upgrades[1].id.clone());
            c.upgrades[0].requires = vec![b];
            c.upgrades[1].requires = vec![a];
        },
        "requisiti circolari",
    );
    // senza upgrade liberi il primo draft sarebbe vuoto
    assert_error(
        |c| {
            let first = c.upgrades[0].id.clone();
            for u in c.upgrades.iter_mut().skip(1) {
                u.requires = vec![first.clone()];
            }
            c.upgrades[0].requires = vec![c.upgrades[1].id.clone()];
        },
        "il primo draft è vuoto",
    );
}

#[test]
fn upgrade_effects_must_do_something_and_keep_values_positive() {
    let c = embedded();
    let rate = upgrade_where(&c, |e| matches!(e, Effect::CompanionIntervalPct(_)));
    let max_stack = c.upgrades[rate].max_stack as f32;
    // −100% / max_stack a ogni stack: al massimo l'intervallo arriva a 0
    assert_error(
        |c| c.upgrades[rate].effect = Effect::CompanionIntervalPct(-100.0 / max_stack),
        "le carte CompanionIntervalPct negative, tutte al massimo, portano il fattore a 0",
    );
    assert_error(
        |c| c.upgrades[rate].effect = Effect::CompanionIntervalPct(f32::NAN),
        "serve un numero finito diverso da 0",
    );
    let shot = upgrade_where(&c, |e| matches!(e, Effect::CompanionProjectiles(_)));
    assert_error(
        |c| c.upgrades[shot].effect = Effect::CompanionProjectiles(0),
        "serve almeno 1 per stack",
    );
    let heal = upgrade_where(&c, |e| matches!(e, Effect::Heal(_)));
    assert_error(
        |c| c.upgrades[heal].effect = Effect::Heal(0.0),
        "deve essere > 0",
    );
}

/// Con FiraMono "più" diventava "pi" seguito da un quadratino (rilievo della prova P0.4): ora
/// le carte possono usare accentate e segni che Tiny5 e Pixelify Sans disegnano, il resto no.
#[test]
fn card_texts_use_only_characters_the_fonts_draw() {
    let id = embedded().upgrades[0].id.clone();
    let accented = errors_after(|c| {
        c.upgrades[0].name = "Velocità".into();
        c.upgrades[0].description = "Danno più alto… «sempre» ×2 – così è".into();
    });
    assert!(accented.is_empty(), "{accented:?}");
    assert_error(
        |c| c.upgrades[0].description = "Danno ↑".into(),
        &format!("upgrades.ron: {id}.description contiene '↑' (U+2191)"),
    );
    assert_error(
        |c| c.upgrades[0].name = "Fiamma\u{ad}viva".into(),
        &format!("upgrades.ron: {id}.name contiene"),
    );
}

/// I limiti fisici valgono anche con tutti gli upgrade al massimo.
#[test]
fn physical_limits_hold_with_every_upgrade_maxed() {
    let c = embedded();
    let speed = upgrade_where(&c, |e| matches!(e, Effect::ProjectileSpeedPct(_)));
    assert_error(
        |c| c.upgrades[speed].effect = Effect::ProjectileSpeedPct(500.0),
        "con tutti gli upgrade di velocità",
    );
    let swift = upgrade_where(&c, |e| matches!(e, Effect::MoveSpeedPct(_)));
    assert_error(
        |c| c.upgrades[swift].effect = Effect::MoveSpeedPct(1000.0),
        "player.speed con tutti gli upgrade di velocità",
    );
    let rate = upgrade_where(&c, |e| matches!(e, Effect::CompanionIntervalPct(_)));
    let max_stack = c.upgrades[rate].max_stack as f32;
    assert_error(
        |c| c.upgrades[rate].effect = Effect::CompanionIntervalPct(-99.5 / max_stack),
        "sotto un passo",
    );
    // gemme più lente del player: non lo raggiungerebbero mai
    assert_error(
        |c| c.progression.gem.magnet_speed = c.player.speed,
        "gem.magnet_speed",
    );
}

/// Una carta in più con effetto `effect` e `max_stack` stack (copia della prima carta).
fn add_card(c: &mut GameConfig, id: &str, effect: Effect, max_stack: u32) {
    let mut card = c.upgrades[0].clone();
    card.id = id.into();
    card.requires.clear();
    card.effect = effect;
    card.max_stack = max_stack;
    c.upgrades.push(card);
}

/// Rilievo del code review P0.4: ogni carta veniva controllata da sola, così due carte di
/// cadenza valide singolarmente potevano portare l'intervallo a 0 (60 raffiche al secondo).
#[test]
fn cards_with_the_same_effect_add_up() {
    let c = embedded();
    let rate = upgrade_where(&c, |e| matches!(e, Effect::CompanionIntervalPct(_)));
    let Effect::CompanionIntervalPct(per_stack) = c.upgrades[rate].effect else {
        unreachable!()
    };
    let taken = -per_stack * c.upgrades[rate].max_stack as f32; // es. 50%
    // una seconda carta che da sola resta valida ma insieme alla prima arriva a −100%
    let rest = 100.0 - taken;
    assert!(rest > 0.0 && rest < 100.0, "dati inattesi: {taken}%");
    let second = Effect::CompanionIntervalPct(-rest / 2.0);
    assert!(
        errors_after(|c| {
            c.upgrades[rate].effect = second.clone();
            c.upgrades[rate].max_stack = 2;
        })
        .is_empty()
    );
    assert_error(
        |c| add_card(c, "rate2", second, 2),
        "le carte CompanionIntervalPct negative",
    );
    let lowest_hp = Effect::MaxHp(-c.player.max_hp / 2.0);
    assert_error(
        |c| {
            add_card(c, "frail1", lowest_hp.clone(), 1);
            add_card(c, "frail2", lowest_hp, 1);
        },
        "portano gli HP massimi a 0",
    );
}

/// Una carta negativa non deve nascondere i limiti di chi prende solo quelle positive: prima
/// le somme si compensavano e la velocità massima risultava sottostimata.
#[test]
fn negative_cards_do_not_hide_the_upper_limits() {
    let c = embedded();
    let a = c.attacks[super::super::tests::projectile_attack(&c)]
        .projectile()
        .unwrap()
        .clone();
    let smallest = c
        .enemies
        .iter()
        .map(|e| e.hurtbox.radius)
        .fold(f32::INFINITY, f32::min);
    // fattore di velocità oltre il quale il proiettile attraversa il nemico più piccolo
    let limit = (a.radius + smallest) * SIM_HZ as f32 / a.speed;
    let up = (1.2 * limit - 1.0) * 100.0; // solo le positive: oltre il limite
    let down = -0.3 * limit * 100.0; // con le negative: 0,9 × limite, sotto
    assert!(down > -100.0, "dati inattesi: limite {limit}");
    let speed = upgrade_where(&c, |e| matches!(e, Effect::ProjectileSpeedPct(_)));
    let stacks = c.upgrades[speed].max_stack as f32;
    assert_error(
        |c| {
            c.upgrades[speed].effect = Effect::ProjectileSpeedPct(up / stacks);
            add_card(c, "slow", Effect::ProjectileSpeedPct(down), 1);
        },
        "con tutti gli upgrade di velocità",
    );
}

#[test]
fn structural_caps_are_enforced() {
    let c = embedded();
    assert_error(
        |c| c.progression.draft_size = MAX_DRAFT_SIZE + 1,
        &format!(
            "draft_size = {}: serve da 1 a {MAX_DRAFT_SIZE}",
            MAX_DRAFT_SIZE + 1
        ),
    );
    assert_error(
        |c| c.enemies[0].xp = MAX_ENEMY_XP + 1,
        &format!("enemies.ron: {}.xp", c.enemies[0].id),
    );
    let shot = upgrade_where(&c, |e| matches!(e, Effect::CompanionProjectiles(_)));
    assert_error(
        |c| c.upgrades[shot].effect = Effect::CompanionProjectiles(MAX_PROJECTILES),
        &format!("al massimo {MAX_PROJECTILES}"),
    );
    let hp = upgrade_where(&c, |e| matches!(e, Effect::MaxHp(_)));
    assert_error(
        |c| c.upgrades[hp].effect = Effect::MaxHp(f32::INFINITY),
        "serve un numero finito diverso da 0",
    );
}

#[test]
fn companions_must_keep_up_with_the_fastest_player() {
    let c = embedded();
    assert_error(
        |c| c.companions[0].follow.max_speed = c.player.speed,
        &format!("companions.ron: {}.follow.max_speed", c.companions[0].id),
    );
}

/// Un prerequisito che non viene mai proposto (peso di rarità 0) blocca chi lo richiede.
#[test]
fn a_requirement_that_is_never_offered_is_reported() {
    let c = embedded();
    let dependent = c
        .upgrades
        .iter()
        .position(|u| !u.requires.is_empty())
        .expect("almeno un upgrade con requisiti");
    let required = c
        .upgrade(&c.upgrades[dependent].requires[0])
        .unwrap()
        .clone();
    let dependent_id = c.upgrades[dependent].id.clone();
    assert_error(
        |c| {
            // una rarità a peso 0 solo per il prerequisito
            let rarity = [Rarity::Common, Rarity::Uncommon, Rarity::Rare]
                .into_iter()
                .find(|r| *r != c.upgrades[dependent].rarity)
                .unwrap();
            c.upgrades
                .iter_mut()
                .find(|u| u.id == required.id)
                .unwrap()
                .rarity = rarity;
            match rarity {
                Rarity::Common => c.progression.rarity_weights.common = 0.0,
                Rarity::Uncommon => c.progression.rarity_weights.uncommon = 0.0,
                Rarity::Rare => c.progression.rarity_weights.rare = 0.0,
            }
        },
        &format!("upgrades.ron: {dependent_id}.requires"),
    );
}

#[test]
fn line_breaks_are_allowed_in_card_texts() {
    assert!(errors_after(|c| c.upgrades[0].description = "riga uno\nriga due".into()).is_empty());
}
