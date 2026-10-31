//! Spawn director: decide quanti nemici nascono, di che archetipo e dove, secondo le ondate di
//! `stages.ron`. Ogni ondata accumula un "budget" di nemici al ritmo corrente (cresce col
//! tempo); quando il budget arriva a 1 nasce un nemico, se i limiti lo permettono e se c'è un
//! punto valido: nell'anello attorno al player, fuori dalla vista della camera, dentro l'arena
//! e fuori dagli ostacoli. Tutto da un solo RNG con seed: stessa run per lo stesso seed.
//!
//! Arco della run (`stages.ron: run`): i nemici nascono più forti col passare dei minuti, le
//! élite arrivano a intervalli fissi, a fine durata le ondate si fermano e nasce il boss.

use bevy::prelude::*;

use super::SimSet;
use super::enemies::{Boss, Enemy, Scale, spawn_enemy};
use super::kinematics::Position;
use super::player::{PLAYER_SPAWN, Player};
use super::run::RunClock;
use crate::app::{AppState, RunSeed};
use crate::data::schema::WaveData;
use crate::data::{DataId, GameConfig};
use crate::rng::Pcg32;
use crate::world::obstacles::ObstacleField;
use crate::world::view::view_rect;
use crate::world::{ArenaBounds, WorldSetup};

/// Stream RNG dedicato agli spawn (vedi `Pcg32::new`).
const SPAWN_STREAM: u64 = 1;
/// Margine oltre la vista: con i piedi fuori dal rettangolo di almeno 32 px anche lo sprite
/// più alto (26 px) è tutto fuori schermo.
pub const SPAWN_VIEW_MARGIN: f32 = 32.0;
/// Tentativi per trovare un punto valido; se falliscono tutti si riprova al passo successivo.
const SPAWN_ATTEMPTS: u32 = 16;
/// Fascia di riserva oltre l'anello, quando nell'anello non c'è un punto fuori vista.
pub(super) const FALLBACK_BAND: f32 = 160.0;

#[derive(Resource, Debug, Clone)]
pub struct SpawnDirector {
    rng: Pcg32,
    /// Nemici "maturati" e non ancora nati, per ondata.
    budget: Vec<f32>,
    /// Numero progressivo dei nemici nati (per il nome di debug).
    pub spawned: u32,
    /// Secondo della run a cui nasce la prossima élite.
    pub next_elite_s: f32,
    pub elites_spawned: u32,
    /// Il boss è nato (da qui la sua sconfitta è la vittoria).
    pub boss_spawned: bool,
}

/// Punto casuale nell'anello `[r_min, r_max)` attorno a `center`.
pub fn ring_point(rng: &mut Pcg32, center: Vec2, r_min: f32, r_max: f32) -> Vec2 {
    let angle = rng.range_f32(0.0, std::f32::consts::TAU);
    let radius = rng.range_f32(r_min, r_max);
    center + Vec2::from_angle(angle) * radius
}

/// Nemici al minuto dell'ondata al secondo `t` della run (0 prima che inizi).
pub fn wave_rate_per_min(wave: &WaveData, t: f32) -> f32 {
    if t < wave.start_s {
        0.0
    } else {
        wave.rate_per_min + wave.rate_growth_per_min * (t - wave.start_s) / 60.0
    }
}

/// Dove può nascere un nemico di raggio `body_radius`: fuori dalla vista (con margine),
/// dentro i limiti dell'arena, fuori dalle impronte degli ostacoli.
pub fn is_valid_spawn(
    at: Vec2,
    player: Vec2,
    arena: &ArenaBounds,
    field: Option<&ObstacleField>,
    body_radius: f32,
) -> bool {
    let in_view = view_rect(player).inflate(SPAWN_VIEW_MARGIN).contains(at);
    let in_obstacle = field.is_some_and(|field| {
        field
            .near(at, body_radius)
            .any(|o| o.footprint_center().distance(at) < o.footprint_radius() + body_radius)
    });
    !in_view && arena.contains(at) && !in_obstacle
}

/// Un punto di spawn valido, se lo trova in `SPAWN_ATTEMPTS` tentativi nell'anello o, se no,
/// nella fascia appena fuori (`FALLBACK_BAND` px). Negli angoli dell'arena la camera si ferma
/// al bordo e la vista copre gran parte dell'anello: senza la fascia lì nascerebbero metà dei
/// nemici e l'angolo diventerebbe un riparo.
pub fn find_spawn_point(
    rng: &mut Pcg32,
    player: Vec2,
    ring: (f32, f32),
    arena: &ArenaBounds,
    field: Option<&ObstacleField>,
    body_radius: f32,
) -> Option<Vec2> {
    let outer = (ring.1, ring.1 + FALLBACK_BAND);
    [ring, outer].into_iter().find_map(|(r_min, r_max)| {
        (0..SPAWN_ATTEMPTS)
            .map(|_| ring_point(rng, player, r_min, r_max))
            .find(|at| is_valid_spawn(*at, player, arena, field, body_radius))
    })
}

pub struct SpawningPlugin;

impl Plugin for SpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InRun),
            start_director.after(WorldSetup), // gli ostacoli devono già esistere
        )
        .add_systems(FixedUpdate, run_director.in_set(SimSet::Spawn));
    }
}

fn start_director(
    mut commands: Commands,
    config: Res<GameConfig>,
    seed: Res<RunSeed>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
) {
    let plan = &config.stage.spawns;
    let mut director = SpawnDirector {
        rng: Pcg32::new(seed.0, SPAWN_STREAM),
        budget: vec![0.0; plan.waves.len()],
        spawned: 0,
        next_elite_s: config.stage.run.elites.first_s,
        elites_spawned: 0,
        boss_spawned: false,
    };
    let data = config.first_wave_enemy();
    for _ in 0..plan.initial_burst {
        let field = field.as_deref();
        let found = find_spawn_point(
            &mut director.rng,
            PLAYER_SPAWN,
            plan.ring,
            &arena,
            field,
            data.footprint_radius,
        );
        if let Some(at) = found {
            spawn_enemy(&mut commands, data, at, director.spawned, Scale::BASE);
            director.spawned += 1;
        }
    }
    commands.insert_resource(director);
}

pub(super) fn run_director(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    clock: Res<RunClock>,
    mut director: ResMut<SpawnDirector>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
    player: Single<&Position, With<Player>>,
    enemies: Query<&DataId, With<Enemy>>,
) {
    let plan = &config.stage.spawns;
    let run = &config.stage.run;
    let t = clock.elapsed;
    if t >= run.duration_s {
        // fine delle orde: il boss, una volta sola, appena c'è un punto valido
        if !director.boss_spawned {
            let boss = config.enemy(&run.boss).expect("riferimento validato");
            let found = find_spawn_point(
                &mut director.rng,
                player.0,
                plan.ring,
                &arena,
                field.as_deref(),
                boss.footprint_radius,
            );
            if let Some(at) = found {
                let entity = spawn_enemy(&mut commands, boss, at, director.spawned, Scale::BASE);
                commands.entity(entity).insert(Boss);
                director.spawned += 1;
                director.boss_spawned = true;
                info!("arriva il boss: {}", boss.display_name);
            }
        }
        return;
    }
    let scale = Scale::at(run.hp_growth_per_min, run.damage_growth_per_min, t);
    if t >= director.next_elite_s {
        let elite = config
            .enemy(&run.elites.enemy)
            .expect("riferimento validato");
        let found = find_spawn_point(
            &mut director.rng,
            player.0,
            plan.ring,
            &arena,
            field.as_deref(),
            elite.footprint_radius,
        );
        if let Some(at) = found {
            spawn_enemy(&mut commands, elite, at, director.spawned, scale);
            director.spawned += 1;
            director.elites_spawned += 1;
            director.next_elite_s += run.elites.every_s;
        }
    }
    let dt = time.delta_secs();
    let mut alive_total = enemies.iter().count();
    // Nati in questo passo per archetipo: la query non li vede ancora (comandi differiti), e
    // più ondate dello stesso archetipo condividono il conteggio dei vivi.
    let mut born_now: Vec<(&str, usize)> = Vec::new();
    for (i, wave) in plan.waves.iter().enumerate() {
        director.budget[i] += wave_rate_per_min(wave, clock.elapsed) / 60.0 * dt;
        if director.budget[i] < 1.0 {
            continue;
        }
        let id = wave.enemy.as_str();
        let born = born_now
            .iter()
            .find(|(e, _)| *e == id)
            .map_or(0, |(_, n)| *n);
        let mut alive = enemies.iter().filter(|d| d.0 == id).count() + born;
        let data = config.wave_enemy(wave);
        while director.budget[i] >= 1.0 {
            let capped = alive >= wave.max_alive as usize || alive_total >= plan.max_alive as usize;
            let found = if capped {
                None
            } else {
                find_spawn_point(
                    &mut director.rng,
                    player.0,
                    plan.ring,
                    &arena,
                    field.as_deref(),
                    data.footprint_radius,
                )
            };
            let Some(at) = found else {
                // Limite raggiunto o nessun punto libero: il budget non si accumula, così
                // quando c'è di nuovo spazio non parte una raffica.
                director.budget[i] = director.budget[i].min(1.0);
                break;
            };
            spawn_enemy(&mut commands, data, at, director.spawned, scale);
            director.spawned += 1;
            director.budget[i] -= 1.0;
            alive += 1;
            alive_total += 1;
            match born_now.iter_mut().find(|(e, _)| *e == id) {
                Some((_, n)) => *n += 1,
                None => born_now.push((id, 1)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::world::biome::Biome;
    use crate::world::obstacles::generate_obstacles;

    fn wave(start_s: f32, rate: f32, growth: f32) -> WaveData {
        WaveData {
            enemy: "blob".into(),
            start_s,
            rate_per_min: rate,
            rate_growth_per_min: growth,
            max_alive: 10,
        }
    }

    #[test]
    fn rate_is_zero_before_start_and_grows_linearly() {
        let w = wave(60.0, 20.0, 15.0);
        assert_eq!(wave_rate_per_min(&w, 59.9), 0.0);
        assert_eq!(wave_rate_per_min(&w, 60.0), 20.0);
        assert_eq!(wave_rate_per_min(&w, 180.0), 50.0);
    }

    #[test]
    fn ring_points_stay_in_ring_and_are_deterministic() {
        let mut rng = Pcg32::new(99, SPAWN_STREAM);
        let mut again = Pcg32::new(99, SPAWN_STREAM);
        for _ in 0..500 {
            let p = ring_point(&mut rng, Vec2::new(10.0, -5.0), 170.0, 240.0);
            let d = p.distance(Vec2::new(10.0, -5.0));
            assert!((170.0 - 1e-3..240.0 + 1e-3).contains(&d), "{d}");
            assert_eq!(
                p,
                ring_point(&mut again, Vec2::new(10.0, -5.0), 170.0, 240.0)
            );
        }
    }

    /// Su ogni stage (D-068: il bosco ha più ostacoli).
    #[test]
    fn spawn_points_are_off_screen_inside_the_arena_and_out_of_obstacles() {
        let embedded = GameConfig::embedded();
        let arena = ArenaBounds::default();
        for stage in &embedded.stages {
            let config = embedded.clone().with_stage(&stage.id).unwrap();
            let biome = Biome::of_stage(&config);
            let field = ObstacleField::new(generate_obstacles(3, &biome, &config.stage.obstacles));
            let ring = config.stage.spawns.ring;
            let mut rng = Pcg32::new(5, SPAWN_STREAM);
            // al centro, vicino a un bordo e in un angolo (la camera lì non è centrata sul player)
            for player in [
                Vec2::ZERO,
                Vec2::new(1990.0, 0.0),
                Vec2::new(-2000.0, -2000.0),
            ] {
                let mut found = 0;
                for _ in 0..300 {
                    if let Some(at) =
                        find_spawn_point(&mut rng, player, ring, &arena, Some(&field), 8.0)
                    {
                        found += 1;
                        assert!(!view_rect(player).inflate(SPAWN_VIEW_MARGIN).contains(at));
                        assert!(arena.contains(at));
                        for o in field.near(at, 8.0) {
                            let clear = o.footprint_radius() + 8.0;
                            assert!(o.footprint_center().distance(at) >= clear, "{}", stage.id);
                        }
                    }
                }
                let at = (&stage.id, player);
                assert!(found > 200, "{at:?}: solo {found} spawn su 300");
            }
        }
    }
}
