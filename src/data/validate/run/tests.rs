//! Test delle regole dell'arco della run e degli attacchi telegrafati. Élite e boss si
//! cercano tramite `stages.ron`, i valori si modificano solo nel campo provato.

use super::super::tests::{assert_error, embedded, errors_after};
use crate::data::GameConfig;
use crate::data::schema::{ChargeData, SlamData};
use crate::gameplay::threats::MIN_TELEGRAPH_S;

fn charge_mut(c: &mut GameConfig) -> &mut ChargeData {
    let id = c.stage.run.elites.enemy.clone();
    let elite = c.enemies.iter_mut().find(|e| e.id == id).unwrap();
    elite.charge.as_mut().expect("l'élite dei dati carica")
}

fn slam_mut(c: &mut GameConfig) -> &mut SlamData {
    let id = c.stage.run.boss.clone();
    let boss = c.enemies.iter_mut().find(|e| e.id == id).unwrap();
    boss.slam
        .as_mut()
        .expect("il boss dei dati colpisce ad area")
}

#[test]
fn the_run_plan_is_checked() {
    let stage = embedded().stage.id;
    assert_error(
        |c| c.stage.run.duration_s = 0.0,
        &format!("stages.ron: {stage}.run.duration_s deve essere > 0"),
    );
    assert_error(
        |c| c.stage.run.hp_growth_per_min = -0.1,
        "run.hp_growth_per_min",
    );
    assert_error(
        |c| c.stage.run.boss = "nobody".into(),
        "run.boss = \"nobody\": non esiste in enemies.ron",
    );
    assert_error(
        |c| c.stage.run.elites.enemy = "nobody".into(),
        "run.elites.enemy = \"nobody\"",
    );
    assert_error(
        |c| c.stage.run.elites.every_s = 0.0,
        "run.elites.every_s deve essere > 0",
    );
}

#[test]
fn telegraphs_last_long_enough_to_react() {
    let c = embedded();
    let elite = c.stage.run.elites.enemy.clone();
    let boss = c.stage.run.boss.clone();
    assert_error(
        |c| charge_mut(c).windup_s = MIN_TELEGRAPH_S * 0.5,
        &format!("enemies.ron: {elite}.charge.windup_s"),
    );
    assert_error(
        |c| slam_mut(c).windup_s = MIN_TELEGRAPH_S * 0.5,
        &format!("enemies.ron: {boss}.slam.windup_s"),
    );
    // il preavviso del colpo non deve sovrapporsi al colpo successivo
    assert_error(
        |c| {
            let slam = slam_mut(c);
            slam.windup_s = slam.every_s;
        },
        "deve finire prima del colpo successivo",
    );
}

#[test]
fn charge_and_slam_values_are_checked() {
    let c = embedded();
    let elite = c.stage.run.elites.enemy.clone();
    // abbastanza veloce da saltare oltre un tronco in un passo
    assert_error(
        |c| charge_mut(c).speed = 5000.0,
        &format!("{elite}.charge.speed = 5000"),
    );
    assert_error(
        |c| charge_mut(c).range = 0.0,
        "charge.range deve essere > 0",
    );
    assert_error(|c| slam_mut(c).radius = 0.0, "slam.radius deve essere > 0");
    assert_error(
        |c| slam_mut(c).damage = -1.0,
        "slam.damage deve essere >= 0",
    );
}

/// Nomi a schermo: accentate sì (i font le disegnano), simboli fuori dai font no.
#[test]
fn names_shown_on_screen_use_characters_the_fonts_draw() {
    let c = embedded();
    let boss = c.stage.run.boss.clone();
    let accented = errors_after(|c| {
        c.stage.display_name = "Città dei Germogli".into();
        c.companions[0].display_name = "Fiammà".into();
    });
    assert!(accented.is_empty(), "{accented:?}");
    assert_error(
        |c| {
            c.enemies
                .iter_mut()
                .find(|e| e.id == boss)
                .unwrap()
                .display_name = "Re ★ Gelatina".into()
        },
        &format!("enemies.ron: {boss}.display_name contiene '★' (U+2605)"),
    );
    assert_error(
        |c| c.stage.display_name = "Prateria\tdei Germogli".into(),
        &format!("stages.ron: {}.display_name contiene", c.stage.id),
    );
    assert_error(
        |c| c.companions[0].display_name = "Brasa 🔥".into(),
        &format!(
            "companions.ron: {}.display_name contiene",
            c.companions[0].id
        ),
    );
}

/// Rilievo del code review P0.6: dal centro del cerchio si deve poter uscire prima del colpo.
#[test]
fn a_slam_can_always_be_escaped() {
    let c = embedded();
    let speed = c.player.speed;
    assert_error(
        |c| {
            let slam = slam_mut(c);
            slam.radius = speed * slam.windup_s * 1.2;
        },
        "il colpo non si può schivare",
    );
}

/// Rilievo del code review P0.6: una carica troppo veloce salterebbe oltre il player.
#[test]
fn a_charge_cannot_jump_over_the_player() {
    let c = embedded();
    let elite = c.enemy(&c.stage.run.elites.enemy).unwrap().clone();
    let too_fast = (elite.contact_distance + 1.0) * crate::gameplay::SIM_HZ as f32;
    assert_error(|c| charge_mut(c).speed = too_fast, "attraversa il player");
}

/// Il fotogramma di un foglio animato: non vuoto e largo un numero pari di texel.
#[test]
fn enemy_sheet_frames_are_checked() {
    use crate::data::schema::SheetFrame;
    let id = embedded().enemies[0].id.clone();
    assert_error(
        |c| c.enemies[0].frame = Some(SheetFrame { w: 27, h: 24 }),
        &format!("enemies.ron: {id}.frame = (27, 24): serve un fotogramma"),
    );
    assert_error(
        |c| c.enemies[0].frame = Some(SheetFrame { w: 28, h: 0 }),
        &format!("enemies.ron: {id}.frame = (28, 0)"),
    );
    let ok = errors_after(|c| c.enemies[0].frame = Some(SheetFrame { w: 28, h: 24 }));
    assert!(!ok.iter().any(|e| e.contains(".frame")), "{ok:?}");
}
