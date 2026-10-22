//! Utilità condivise dai test di simulazione headless.
#![allow(dead_code)] // ogni file di test usa solo una parte di queste funzioni

use bevy::prelude::*;
use pixel_monster_survivor::app::RunPhase;
use pixel_monster_survivor::data::GameConfig;
use pixel_monster_survivor::data::schema::{AttackData, Effect};
use pixel_monster_survivor::gameplay::attacks::AutoAttack;
use pixel_monster_survivor::gameplay::combat::Health;
use pixel_monster_survivor::gameplay::companions::Companion;
use pixel_monster_survivor::gameplay::damage::GodMode;
use pixel_monster_survivor::gameplay::enemies::{Enemy, enemy_bundle};
use pixel_monster_survivor::gameplay::kinematics::{Position, body};
use pixel_monster_survivor::gameplay::player::Player;
use pixel_monster_survivor::gameplay::progression::{Progression, XpGem, xp_to_next};
use pixel_monster_survivor::gameplay::upgrades::{AutoPick, ChooseUpgrade, Draft};
use pixel_monster_survivor::headless_app_with;

/// Configurazione in uso nella simulazione (i file di `game-data/`, salvo modifiche del test).
pub fn config(app: &App) -> GameConfig {
    app.world().resource::<GameConfig>().clone()
}

/// I dati veri, ma senza nemici nuovi dopo l'ondata iniziale e senza gemme XP (quindi senza
/// livelli e upgrade): per i test di movimento e combattimento che misurano i valori base.
pub fn quiet_config() -> GameConfig {
    let mut config = GameConfig::embedded();
    for wave in &mut config.stage.spawns.waves {
        wave.rate_per_min = 0.0;
        wave.rate_growth_per_min = 0.0;
    }
    for enemy in &mut config.enemies {
        enemy.xp = 0;
    }
    config
}

/// Run con `quiet_config` e player invulnerabile (`GodMode`): nulla interferisce con ciò che
/// il test vuole misurare.
pub fn quiet_app(seed: u64) -> App {
    let mut app = headless_app_with(seed, quiet_config());
    god(&mut app);
    app
}

/// Player invulnerabile.
pub fn god(app: &mut App) {
    app.insert_resource(GodMode(true));
}

pub fn run_steps(app: &mut App, steps: usize) {
    for _ in 0..steps {
        app.update();
    }
}

pub fn player_position(app: &mut App) -> Vec2 {
    single_position::<Player>(app)
}

pub fn companion_position(app: &mut App) -> Vec2 {
    single_position::<Companion>(app)
}

fn single_position<T: Component>(app: &mut App) -> Vec2 {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<T>>()
        .single(world)
        .expect("esattamente un'entità")
        .0
}

pub fn enemy_positions(app: &mut App) -> Vec<Vec2> {
    let world = app.world_mut();
    world
        .query_filtered::<&Position, With<Enemy>>()
        .iter(world)
        .map(|p| p.0)
        .collect()
}

pub fn count<T: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), With<T>>().iter(world).count()
}

/// Rimuove tutte le entità con `T` (es. i compagni, per provare il movimento senza combattere).
pub fn despawn_all<T: Component>(app: &mut App) {
    let world = app.world_mut();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<T>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
}

pub fn mean_distance(points: &[Vec2], to: Vec2) -> f32 {
    points.iter().map(|p| p.distance(to)).sum::<f32>() / points.len() as f32
}

// --- Progressione (P0.4): condivisi da `progression.rs` e `upgrades.rs` ---

/// I dati di `quiet_config` con le XP vere dei nemici.
pub fn xp_config() -> GameConfig {
    let mut config = quiet_config();
    config.enemies = GameConfig::embedded().enemies;
    config
}

/// Run tranquilla e invulnerabile con le XP vere, senza nemici in campo e con il draft che
/// aspetta la scelta del test (niente `AutoPick`).
pub fn xp_app_with(config: GameConfig) -> App {
    let mut app = headless_app_with(1, config);
    god(&mut app);
    app.world_mut().remove_resource::<AutoPick>();
    app.update();
    despawn_all::<Enemy>(&mut app);
    app
}

pub fn xp_app() -> App {
    xp_app_with(xp_config())
}

pub fn phase(app: &App) -> RunPhase {
    *app.world().resource::<State<RunPhase>>().get()
}

pub fn progression(app: &App) -> Progression {
    app.world().resource::<Progression>().clone()
}

pub fn gems(app: &mut App) -> Vec<(Vec2, u32)> {
    let world = app.world_mut();
    world
        .query::<(&Position, &XpGem)>()
        .iter(world)
        .map(|(p, gem)| (p.0, gem.value))
        .collect()
}

/// L'entità del player, per leggerne o cambiarne i componenti.
pub fn player_mut(app: &mut App) -> EntityWorldMut<'_> {
    let world = app.world_mut();
    let entity = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .unwrap();
    world.entity_mut(entity)
}

pub fn companion_attack(app: &mut App) -> AutoAttack {
    let world = app.world_mut();
    world
        .query_filtered::<&AutoAttack, With<Companion>>()
        .single(world)
        .unwrap()
        .clone()
}

/// Nemico della prima ondata fermo in `at` con `hp` punti vita.
pub fn enemy_at(app: &mut App, at: Vec2, hp: f32, serial: u32) -> Entity {
    let data = config(app).first_wave_enemy().clone();
    let world = app.world_mut();
    let entity = world.spawn(enemy_bundle(&data, at, serial)).id();
    world.get_mut::<Enemy>(entity).unwrap().speed = 0.0;
    world.get_mut::<Health>(entity).unwrap().current = hp;
    entity
}

/// Punto a metà della gittata del compagno, a destra del player: sempre bersagliabile.
pub fn in_companion_range(app: &mut App) -> Vec2 {
    let range = companion_attack(app).range;
    player_position(app) + Vec2::X * range * 0.5
}

/// Gemma ai piedi del player che basta esattamente per `levels` livelli.
pub fn give_levels(app: &mut App, levels: u32) {
    let config = config(app);
    let p = progression(app);
    let mut needed = p.xp_to_next - p.xp;
    for level in p.level + 1..p.level + levels {
        needed += xp_to_next(&config.progression.xp_to_level, level);
    }
    let at = player_position(app);
    app.world_mut().spawn((XpGem { value: needed }, body(at)));
}

/// Indice del primo upgrade dei dati con l'effetto cercato.
pub fn upgrade_index(app: &App, wanted: impl Fn(&Effect) -> bool) -> usize {
    config(app)
        .upgrades
        .iter()
        .position(|u| wanted(&u.effect))
        .expect("i dati hanno un upgrade con questo effetto")
}

/// Prende l'upgrade `index` passando dal draft vero: livello, draft aperto, carta scelta.
/// I requisiti dell'upgrade vanno presi prima.
pub fn take(app: &mut App, index: usize) {
    give_levels(app, 1);
    run_steps(app, 2);
    assert_eq!(phase(app), RunPhase::LevelUp);
    app.world_mut().resource_mut::<Draft>().choices = vec![index];
    app.world_mut().write_message(ChooseUpgrade(0));
    run_steps(app, 2);
    assert_eq!(phase(app), RunPhase::Playing);
}

// --- Compagni e pattern di attacco (P0.5) ---

/// Id del primo compagno dei dati il cui attacco soddisfa `has` (es. `|a| a.pulse().is_some()`).
pub fn companion_with(config: &GameConfig, has: impl Fn(&AttackData) -> bool) -> String {
    config
        .companions
        .iter()
        .find(|c| config.attack(&c.attack).is_some_and(&has))
        .expect("i dati hanno un compagno con questo attacco")
        .id
        .clone()
}

/// Run tranquilla e invulnerabile, senza nemici e senza XP, con `companion` come compagno
/// iniziale al posto di quello dello stage.
pub fn app_starting_with(companion: &str, change: impl FnOnce(&mut GameConfig)) -> App {
    let mut config = quiet_config();
    config.stage.starting_companion = companion.into();
    change(&mut config);
    let mut app = headless_app_with(1, config);
    god(&mut app);
    app.update();
    despawn_all::<Enemy>(&mut app);
    app
}
