//! Monete e forzieri. Alcuni nemici sconfitti lasciano monete (`enemies.ron: coins`) che si
//! raccolgono con la stessa calamita delle gemme; i forzieri (`stages.ron: chests`) stanno
//! lontani dal player, fuori vista: passandoci sopra con abbastanza monete si aprono e la run
//! passa al draft delle reliquie (`upgrades`). Aperto uno ne nasce un altro; uno rimasto troppo
//! lontano rinasce nella fascia. Servono a far girare il player per la mappa.
//!
//! Un RNG tutto suo: stessa seed, stesse monete e stessi forzieri.

use bevy::prelude::*;

use super::SimSet;
use super::combat::Health;
use super::enemies::Enemy;
use super::kinematics::{Position, body};
use super::player::{PLAYER_SPAWN, Player};
use super::progression::{Attracted, PickupRadius};
use super::spawning::{is_valid_spawn, ring_point};
use super::upgrades::Modifiers;
use crate::app::{AppState, RunSeed};
use crate::data::schema::ChestPlan;
use crate::data::{DataId, GameConfig};
use crate::rng::Pcg32;
use crate::world::obstacles::ObstacleField;
use crate::world::{ArenaBounds, WorldSetup};

/// Stream RNG di monete e forzieri (vedi `Pcg32::new`; 1 spawn, 2 draft, 3 orde).
const TREASURE_STREAM: u64 = 4;
/// Tentativi per fascia nella ricerca del posto di un forziere.
const CHEST_ATTEMPTS: u32 = 32;
/// Impronta del forziere contro gli ostacoli (px).
const CHEST_RADIUS: f32 = 10.0;

/// Moneta a terra; `value` è quante monete vale (si sommano quando ce ne sono troppe).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Coin {
    pub value: f32,
}

/// Forziere chiuso sulla mappa.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chest;

/// Monete della run e forzieri aperti.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Treasury {
    /// Monete in tasca (con il Salvadanaio una moneta vale di più: si mostrano per difetto).
    pub coins: f32,
    /// Monete raccolte in tutta la run.
    pub collected: f32,
    pub opened: u32,
    /// Un forziere appena aperto aspetta la scelta della reliquia.
    pub pending: bool,
}

impl Treasury {
    pub fn price(&self, plan: &ChestPlan) -> u32 {
        plan.price(self.opened)
    }

    pub fn can_afford(&self, plan: &ChestPlan) -> bool {
        self.coins + 1e-3 >= self.price(plan) as f32
    }
}

/// Un forziere si è aperto in `at` (per l'animazione dell'apertura).
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ChestOpened {
    pub at: Vec2,
}

#[derive(Resource, Debug, Clone)]
pub(super) struct TreasureRng(Pcg32);

pub struct TreasurePlugin;

impl Plugin for TreasurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Treasury>()
            .insert_resource(TreasureRng(Pcg32::new(0, TREASURE_STREAM)))
            .add_message::<ChestOpened>()
            .add_systems(
                OnEnter(AppState::InRun),
                start_treasure.after(WorldSetup), // i forzieri evitano gli ostacoli
            )
            .add_systems(
                FixedUpdate,
                (collect_coins, tend_chests).chain().in_set(SimSet::Pickup),
            );
    }
}

/// Dentro l'arena con almeno `margin` px dal bordo.
fn away_from_edge(at: Vec2, arena: &ArenaBounds, margin: f32) -> bool {
    at.x - arena.min.x >= margin
        && arena.max.x - at.x >= margin
        && at.y - arena.min.y >= margin
        && arena.max.y - at.y >= margin
}

/// Un posto per un forziere: nella fascia di distanza dal player (poi in quella di riserva),
/// fuori vista, lontano dal bordo e dagli altri forzieri, fuori dagli ostacoli.
pub fn find_chest_spot(
    rng: &mut Pcg32,
    player: Vec2,
    others: &[Vec2],
    plan: &ChestPlan,
    arena: &ArenaBounds,
    field: Option<&ObstacleField>,
) -> Option<Vec2> {
    for band in [plan.distance, plan.fallback_distance] {
        for _ in 0..CHEST_ATTEMPTS {
            let at = ring_point(rng, player, band.0, band.1);
            if away_from_edge(at, arena, plan.min_from_edge)
                && others.iter().all(|o| o.distance(at) >= plan.min_apart)
                && is_valid_spawn(at, player, arena, field, CHEST_RADIUS)
            {
                return Some(at);
            }
        }
    }
    None
}

fn spawn_chest(commands: &mut Commands, at: Vec2) {
    commands.spawn((
        Name::new("Forziere"),
        Chest,
        body(at),
        DespawnOnExit(AppState::InRun),
    ));
}

fn start_treasure(
    mut commands: Commands,
    config: Res<GameConfig>,
    seed: Res<RunSeed>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
) {
    let mut rng = Pcg32::new(seed.0, TREASURE_STREAM);
    let plan = &config.stage.chests;
    let mut placed: Vec<Vec2> = Vec::new();
    for _ in 0..plan.on_map {
        let spot = find_chest_spot(
            &mut rng,
            PLAYER_SPAWN,
            &placed,
            plan,
            &arena,
            field.as_deref(),
        );
        if let Some(at) = spot {
            spawn_chest(&mut commands, at);
            placed.push(at);
        }
    }
    commands.insert_resource(Treasury::default());
    commands.insert_resource(TreasureRng(rng));
}

/// Nella catena del combattimento, dopo le gemme e prima della rimozione dei morti: ogni
/// nemico a 0 HP lascia le sue monete con la sua probabilità, sparse attorno al punto. Con
/// troppe monete a terra il valore va alla moneta più vicina.
pub(super) fn drop_coins(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut rng: ResMut<TreasureRng>,
    dead: Query<(&DataId, &Health, &Position), With<Enemy>>,
    mut coins: Query<(&Position, &mut Coin), Without<Enemy>>,
) {
    let coin = &config.progression.coin;
    let mut on_field = coins.iter().len();
    for (id, health, at) in &dead {
        if !health.is_dead() {
            continue;
        }
        let Some(drop) = config.enemy(&id.0).map(|e| e.coins) else {
            continue;
        };
        if drop.count == 0 || drop.chance <= 0.0 {
            continue;
        }
        if drop.chance < 1.0 && rng.0.next_f32() >= drop.chance {
            continue;
        }
        for k in 0..drop.count {
            if on_field >= coin.max_on_field as usize {
                let nearest = coins
                    .iter_mut()
                    .min_by(|a, b| {
                        let da = a.0.0.distance_squared(at.0);
                        da.total_cmp(&b.0.0.distance_squared(at.0))
                    })
                    .map(|(_, c)| c);
                if let Some(mut nearest) = nearest {
                    nearest.value += 1.0;
                    continue;
                }
            }
            let turn = (k as f32 + rng.0.range_f32(0.0, 1.0)) / drop.count as f32;
            let offset = Vec2::from_angle(turn * std::f32::consts::TAU) * coin.scatter;
            commands.spawn((
                Name::new("Moneta"),
                Coin { value: 1.0 },
                body(at.0 + offset),
                DespawnOnExit(AppState::InRun),
            ));
            on_field += 1;
        }
    }
}

/// Come le gemme: nel raggio di raccolta la moneta vola verso il player e a
/// `collect_distance` è presa; vale `coin_gain` (Salvadanaio).
fn collect_coins(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    modifiers: Res<Modifiers>,
    mut treasury: ResMut<Treasury>,
    player: Single<(&Position, &PickupRadius), With<Player>>,
    mut coins: Query<(Entity, &Coin, &mut Position, Has<Attracted>), Without<Player>>,
) {
    let gem = &config.progression.gem;
    let (target, radius) = (player.0.0, player.1.0);
    let step = gem.magnet_speed * time.delta_secs();
    for (entity, coin, mut at, attracted) in &mut coins {
        if !attracted {
            if at.0.distance(target) > radius {
                continue;
            }
            commands.entity(entity).try_insert(Attracted);
        }
        at.0 = at.0.move_towards(target, step);
        if at.0.distance(target) <= gem.collect_distance {
            let value = coin.value * modifiers.coin_gain;
            treasury.coins += value;
            treasury.collected += value;
            commands.entity(entity).try_despawn();
        }
    }
}

/// Forzieri: si apre quello su cui sta il player se le monete bastano (uno per volta, poi il
/// draft delle reliquie); chi è rimasto troppo lontano rinasce nella fascia; ne nasce uno
/// nuovo finché non sono `on_map`.
#[allow(clippy::too_many_arguments)]
fn tend_chests(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut rng: ResMut<TreasureRng>,
    mut treasury: ResMut<Treasury>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
    player: Single<(&Position, &Health), With<Player>>,
    mut chests: Query<(Entity, &mut Position), (With<Chest>, Without<Player>)>,
    mut opened: MessageWriter<ChestOpened>,
) {
    let plan = &config.stage.chests;
    let (me, health) = (player.0.0, player.1);
    let mut remaining: Vec<Vec2> = Vec::new();
    for (entity, mut at) in &mut chests {
        let here = !treasury.pending
            && !health.is_dead()
            && at.0.distance(me) <= plan.open_radius
            && treasury.can_afford(plan);
        if here {
            treasury.coins -= treasury.price(plan) as f32;
            treasury.opened += 1;
            treasury.pending = true;
            opened.write(ChestOpened { at: at.0 });
            commands.entity(entity).try_despawn();
            continue;
        }
        if at.0.distance(me) > plan.respawn_beyond {
            let others: Vec<Vec2> = remaining.clone();
            if let Some(spot) =
                find_chest_spot(&mut rng.0, me, &others, plan, &arena, field.as_deref())
            {
                at.0 = spot;
            }
        }
        remaining.push(at.0);
    }
    if remaining.len() < plan.on_map as usize
        && let Some(spot) =
            find_chest_spot(&mut rng.0, me, &remaining, plan, &arena, field.as_deref())
    {
        spawn_chest(&mut commands, spot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::world::view::view_rect;

    #[test]
    fn the_price_grows_with_every_chest_opened() {
        let plan = GameConfig::embedded().stage.chests;
        let mut treasury = Treasury::default();
        assert_eq!(treasury.price(&plan), plan.price_base);
        treasury.opened = 2;
        assert_eq!(
            treasury.price(&plan),
            plan.price_base + 2 * plan.price_per_open
        );
        treasury.coins = treasury.price(&plan) as f32 - 0.5;
        assert!(!treasury.can_afford(&plan));
        treasury.coins += 0.5;
        assert!(treasury.can_afford(&plan));
    }

    #[test]
    fn chests_land_far_off_screen_inside_and_apart() {
        let plan = GameConfig::embedded().stage.chests;
        let arena = ArenaBounds::default();
        let mut rng = Pcg32::new(9, TREASURE_STREAM);
        for player in [
            Vec2::ZERO,
            Vec2::new(1900.0, 0.0),
            Vec2::new(-1900.0, -1900.0),
        ] {
            let mut placed: Vec<Vec2> = Vec::new();
            for _ in 0..plan.on_map {
                let at = find_chest_spot(&mut rng, player, &placed, &plan, &arena, None)
                    .expect("c'è sempre posto");
                let d = at.distance(player);
                assert!(
                    d >= plan.fallback_distance.0 && d <= plan.fallback_distance.1,
                    "{d}"
                );
                assert!(!view_rect(player).contains(at));
                assert!(away_from_edge(at, &arena, plan.min_from_edge));
                assert!(placed.iter().all(|o| o.distance(at) >= plan.min_apart));
                placed.push(at);
            }
        }
    }
}
