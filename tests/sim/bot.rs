//! "Giocatore medio" per le sonde di bilanciamento (P0.7): non è l'autoplay del gioco (movimento
//! cieco) né il bot che scappa da tutto. Si allontana dai nemici vicini (di più da élite e boss)
//! girando loro attorno invece di fuggire dritto, esce dai segni di pericolo, prende le gemme
//! più convenienti (dritto quando è tranquillo, di passaggio quando è stretto), gira attorno
//! agli ostacoli e ai bordi e sceglie le carte con una priorità ragionevole (compagni, poi danno
//! e cadenza, cure quando servono). Raccoglie anche le monete e, quando bastano, va al forziere
//! più vicino. Una stima di chi gioca, non di chi gioca bene.
//!
//! Prima versione: fuggiva dritto, e una volta inseguito dall'orda attraversava l'arena senza
//! più raccogliere (XP ferma per minuti, gemme al limite di 300 lasciate indietro).

use bevy::prelude::*;
use pixel_monster_survivor::app::RunPhase;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::Effect;
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::combat::Health;
use pixel_monster_survivor::gameplay::enemies::{Boss, Enemy};
use pixel_monster_survivor::gameplay::kinematics::Position;
use pixel_monster_survivor::gameplay::player::{MoveIntent, Player};
use pixel_monster_survivor::gameplay::progression::XpGem;
use pixel_monster_survivor::gameplay::threats::{Charger, Telegraph, TelegraphShape};
use pixel_monster_survivor::gameplay::treasure::{Chest, Coin, Treasury};
use pixel_monster_survivor::gameplay::upgrades::{AutoPick, ChooseUpgrade, Draft};
use pixel_monster_survivor::world::ArenaBounds;
use pixel_monster_survivor::world::obstacles::ObstacleField;

/// Distanza a cui un nemico comune preoccupa; élite e boss da più lontano.
const DANGER: f32 = 60.0;
const BIG_DANGER: f32 = 110.0;
/// Fuori da un segno di pericolo con questo margine.
const TELEGRAPH_MARGIN: f32 = 14.0;
/// Gemme considerate e quanto si è disposti a rischiare per andarci.
const GEM_REACH: f32 = 220.0;
const SAFE_ENOUGH: f32 = 0.35;
/// Quanto della spinta dei nemici diventa movimento attorno a loro (in senso antiorario).
const CIRCLE: f32 = 0.9;
/// Una moneta vale come una gemma da tanti punti XP; il forziere tira quanto una gemma vicina.
const COIN_WORTH: f32 = 4.0;
const CHEST_PULL: f32 = 0.6;

/// Prepara una run per il bot: il draft aspetta la sua scelta, non quella automatica.
pub fn take_over(app: &mut App) {
    app.world_mut().remove_resource::<AutoPick>();
}

/// Un passo del bot: nel draft sceglie una carta, in gioco imposta la direzione.
pub fn drive(app: &mut App, t: f32) {
    let phase = *app.world().resource::<State<RunPhase>>().get();
    if phase == RunPhase::LevelUp {
        let draft = app.world().resource::<Draft>().clone();
        if !draft.choices.is_empty() {
            let config = app.world().resource::<GameConfig>().clone();
            let hp = hp_fraction(app);
            let pick = choose(&config, &draft.choices, hp);
            app.world_mut().write_message(ChooseUpgrade(pick));
        }
        return;
    }
    let direction = direction(app, t);
    app.insert_resource(MoveIntent(direction));
}

fn hp_fraction(app: &mut App) -> f32 {
    let world = app.world_mut();
    world
        .query_filtered::<&Health, With<Player>>()
        .single(world)
        .map_or(1.0, |h| (h.current / h.max.max(1.0)).clamp(0.0, 1.0))
}

/// Priorità di una carta: compagni prima di tutto, poi danno e cadenza; vita e cure salgono
/// quando gli HP sono bassi.
fn score(effect: &Effect, hp: f32) -> i32 {
    match effect {
        Effect::JoinCompanion(_) => 100,
        Effect::Heal(_) if hp < 0.4 => 90,
        Effect::MaxHp(_) if hp < 0.5 => 75,
        Effect::CompanionDamagePct(_) => 60,
        Effect::CompanionIntervalPct(_) => 58,
        Effect::CompanionProjectiles(_) => 55,
        Effect::RegenPerSecond(_) => 45,
        Effect::CompanionRangePct(_) => 42,
        Effect::CompanionPierce(_) => 40,
        Effect::MoveSpeedPct(_) => 38,
        Effect::MaxHp(_) => 35,
        Effect::PickupRadiusPct(_) => 30,
        Effect::ProjectileSpeedPct(_) => 20,
        Effect::CoinGainPct(_) => 15,
        Effect::Heal(_) => 5,
    }
}

/// Posizione nel draft della carta con la priorità più alta (a pari merito la prima).
pub fn choose(config: &GameConfig, choices: &[usize], hp: f32) -> usize {
    let mut best = (0, i32::MIN);
    for (slot, &index) in choices.iter().enumerate() {
        if let Some(upgrade) = config.upgrades.get(index) {
            let s = score(&upgrade.effect, hp);
            if s > best.1 {
                best = (slot, s);
            }
        }
    }
    best.0
}

/// Spinta via da un punto pericoloso entro `reach`, più forte da vicino.
fn away(from: Vec2, me: Vec2, reach: f32) -> Vec2 {
    let delta = me - from;
    let d = delta.length();
    if d >= reach || d < 1e-3 {
        return Vec2::ZERO;
    }
    let k = (reach - d) / reach;
    delta / d * k * k
}

fn direction(app: &mut App, t: f32) -> Vec2 {
    let arena = *app.world().resource::<ArenaBounds>();
    let afford = {
        let plan = &app.world().resource::<GameConfig>().stage.chests;
        app.world().resource::<Treasury>().can_afford(plan)
    };
    let world = app.world_mut();
    let Ok(me) = world
        .query_filtered::<&Position, With<Player>>()
        .single(world)
        .map(|p| p.0)
    else {
        return Vec2::ZERO;
    };
    let mut threat = Vec2::ZERO;
    for (at, boss, charger) in world
        .query_filtered::<(&Position, Has<Boss>, Has<Charger>), With<Enemy>>()
        .iter(world)
    {
        if boss || charger {
            threat += away(at.0, me, BIG_DANGER) * 2.0;
        } else {
            threat += away(at.0, me, DANGER);
        }
    }
    // chi è stretto gira attorno all'orda come un giocatore che "kita in cerchio": resta vicino
    // a dove i compagni lasciano le gemme invece di portarsi dietro i nemici per tutta l'arena
    let mut push = threat + Vec2::new(-threat.y, threat.x) * CIRCLE;
    for (at, telegraph) in world.query::<(&Position, &Telegraph)>().iter(world) {
        let closest = match telegraph.shape {
            TelegraphShape::Ring { radius } => {
                if me.distance(at.0) < radius + TELEGRAPH_MARGIN {
                    push += (me - at.0).normalize_or(Vec2::X) * 4.0;
                }
                continue;
            }
            TelegraphShape::Line { direction, length } => {
                let along = (me - at.0).dot(direction).clamp(0.0, length);
                at.0 + direction * along
            }
        };
        let off = me - closest;
        if off.length() < TELEGRAPH_MARGIN + 10.0 {
            let side = if off.length() > 1e-3 {
                off.normalize()
            } else {
                Vec2::new(-closest.y, closest.x).normalize_or(Vec2::Y)
            };
            push += side * 4.0;
        }
    }
    // gemme: la più conveniente (XP per distanza); dritti da tranquilli, di passaggio da stretti
    let mut pull = Vec2::ZERO;
    let gems: Vec<(Vec2, f32)> = world
        .query::<(&Position, &XpGem)>()
        .iter(world)
        .map(|(g, gem)| (g.0, gem.value as f32))
        .collect();
    let coins: Vec<(Vec2, f32)> = world
        .query::<(&Position, &Coin)>()
        .iter(world)
        .map(|(c, coin)| (c.0, coin.value * COIN_WORTH))
        .collect();
    let best = gems
        .into_iter()
        .chain(coins)
        .filter(|(at, _)| at.distance(me) < GEM_REACH)
        .map(|(at, worth)| (at, worth / (at.distance(me) + 20.0)))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((gem, _)) = best {
        let eager = if threat.length() < SAFE_ENOUGH {
            0.6
        } else {
            0.25
        };
        pull = (gem - me).normalize_or_zero() * eager;
    }
    // con le monete che bastano, verso il forziere più vicino (girando attorno a chi c'è in mezzo)
    if afford {
        let chest = world
            .query_filtered::<&Position, With<Chest>>()
            .iter(world)
            .map(|c| c.0)
            .min_by(|a, b| a.distance(me).total_cmp(&b.distance(me)));
        if let Some(chest) = chest {
            pull += (chest - me).normalize_or_zero() * CHEST_PULL;
        }
    }
    // ostacoli: si gira attorno invece di restarci incastrati
    let field = world.resource::<ObstacleField>();
    for obstacle in field.near(me, 24.0) {
        let reach = obstacle.footprint_radius() + 14.0;
        push += away(obstacle.footprint_center(), me, reach) * 1.5;
    }
    // bordi: verso il centro, più forte da vicino
    let margin = 220.0;
    let edge = Vec2::new(
        ((arena.min.x + margin - me.x).max(0.0) - (me.x - (arena.max.x - margin)).max(0.0))
            / margin,
        ((arena.min.y + margin - me.y).max(0.0) - (me.y - (arena.max.y - margin)).max(0.0))
            / margin,
    );
    // senza pericoli né gemme si cammina (fermi si viene circondati)
    let wander = autoplay_direction(t) * 0.25;
    (push + pull + edge * 0.6 + wander).normalize_or_zero()
}
