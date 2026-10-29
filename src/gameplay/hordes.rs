//! Orde (`stages.ron: run.hordes`): a istanti fissi della run arriva tutta insieme una massa di
//! nemici, annunciata `warning_s` secondi prima. Nascono in cerchio attorno al player, fuori
//! dalla vista, distribuiti in `duration_s` secondi e oltre i limiti delle ondate continue, fino
//! a `max_alive` vivi in tutto: per uscirne bisogna aprirsi un varco. Le ondate continuano.
//! Un RNG tutto loro: stessa seed, stessa orda, e gli spawn normali restano quelli di prima.

use bevy::prelude::*;

use super::SimSet;
use super::enemies::{Enemy, Scale, spawn_enemy};
use super::kinematics::Position;
use super::player::Player;
use super::run::RunClock;
use super::spawning::{FALLBACK_BAND, SpawnDirector, is_valid_spawn, run_director};
use crate::app::{AppState, RunSeed};
use crate::data::GameConfig;
use crate::data::schema::HordePlan;
use crate::rng::Pcg32;
use crate::world::ArenaBounds;
use crate::world::obstacles::ObstacleField;

/// Stream RNG delle orde (vedi `Pcg32::new`; 1 spawn, 2 draft).
const HORDE_STREAM: u64 = 3;
/// Tentativi per un punto valido attorno al suo angolo; se falliscono il nemico non nasce.
const ATTEMPTS: u32 = 8;

#[derive(Resource, Debug, Clone)]
pub struct HordeState {
    rng: Pcg32,
    /// Prossima orda da far partire (indice in `run.hordes.list`).
    pub next: usize,
    pub active: Option<ActiveHorde>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveHorde {
    pub index: usize,
    /// Secondi da quando l'orda ha cominciato a nascere.
    pub elapsed: f32,
    /// Nemici già nati (o saltati per mancanza di spazio) per gruppo.
    pub born: Vec<u32>,
    /// Posti già occupati sul cerchio, tutti i gruppi insieme.
    pub placed: u32,
}

/// Cosa annunciare al secondo `t`: un'orda in arrivo (secondi che mancano) o in corso.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HordeAlert {
    Coming { index: usize, in_s: f32 },
    Here { index: usize },
}

pub fn horde_alert(plan: &HordePlan, t: f32) -> Option<HordeAlert> {
    plan.list.iter().enumerate().find_map(|(index, h)| {
        if t >= h.at_s - plan.warning_s && t < h.at_s {
            Some(HordeAlert::Coming {
                index,
                in_s: h.at_s - t,
            })
        } else if t >= h.at_s && t < h.at_s + plan.duration_s {
            Some(HordeAlert::Here { index })
        } else {
            None
        }
    })
}

/// Nemici di un gruppo da `count` che devono essere nati dopo `elapsed` secondi su `duration`.
pub fn due(count: u32, elapsed: f32, duration: f32) -> u32 {
    let fraction = (elapsed / duration.max(f32::EPSILON)).clamp(0.0, 1.0);
    ((count as f32 * fraction).ceil() as u32).min(count)
}

/// Il posto `k` su `total` del cerchio attorno a `center`, con un po' di scarto casuale
/// nell'angolo e la distanza nell'anello `ring`.
pub fn circle_point(rng: &mut Pcg32, center: Vec2, ring: (f32, f32), k: u32, total: u32) -> Vec2 {
    let slot = (k as f32 + rng.range_f32(0.0, 1.0)) / total.max(1) as f32;
    let radius = rng.range_f32(ring.0, ring.1);
    center + Vec2::from_angle(slot * std::f32::consts::TAU) * radius
}

pub struct HordesPlugin;

impl Plugin for HordesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HordeState {
            rng: Pcg32::new(0, HORDE_STREAM),
            next: 0,
            active: None,
        })
        .add_systems(OnEnter(AppState::InRun), reset_hordes)
        .add_systems(
            FixedUpdate,
            run_hordes.in_set(SimSet::Spawn).after(run_director),
        );
    }
}

fn reset_hordes(mut commands: Commands, seed: Res<RunSeed>) {
    commands.insert_resource(HordeState {
        rng: Pcg32::new(seed.0, HORDE_STREAM),
        next: 0,
        active: None,
    });
}

#[allow(clippy::too_many_arguments)]
fn run_hordes(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    clock: Res<RunClock>,
    mut state: ResMut<HordeState>,
    mut director: ResMut<SpawnDirector>,
    arena: Res<ArenaBounds>,
    field: Option<Res<ObstacleField>>,
    player: Single<&Position, With<Player>>,
    enemies: Query<(), With<Enemy>>,
) {
    let run = &config.stage.run;
    let plan = &run.hordes;
    let t = clock.elapsed;
    let HordeState { rng, next, active } = &mut *state;
    if active.is_none() {
        let Some(horde) = plan.list.get(*next) else {
            return;
        };
        if t < horde.at_s {
            return;
        }
        info!("arriva l'orda {}: {} nemici", *next + 1, horde.total());
        *active = Some(ActiveHorde {
            index: *next,
            elapsed: 0.0,
            born: vec![0; horde.groups.len()],
            placed: 0,
        });
        *next += 1;
    }
    let Some(current) = active.as_mut() else {
        return;
    };
    let Some(horde) = plan.list.get(current.index) else {
        *active = None; // dati ricaricati con meno orde
        return;
    };
    current.elapsed += time.delta_secs();
    let scale = Scale::at(run.hp_growth_per_min, run.damage_growth_per_min, t);
    let ring = config.stage.spawns.ring;
    let total = horde.total();
    let mut alive = enemies.iter().count();
    for (g, group) in horde.groups.iter().enumerate() {
        let Some(data) = config.enemy(&group.enemy) else {
            continue;
        };
        let wanted = due(group.count, current.elapsed, plan.duration_s);
        while current.born[g] < wanted && alive < plan.max_alive as usize {
            // come gli spawn normali: se l'anello è in vista (ai lati, negli angoli), la fascia
            // appena fuori
            let outer = (ring.1, ring.1 + FALLBACK_BAND);
            let found = [ring, outer].into_iter().find_map(|band| {
                (0..ATTEMPTS)
                    .map(|_| circle_point(rng, player.0, band, current.placed, total))
                    .find(|at| {
                        is_valid_spawn(
                            *at,
                            player.0,
                            &arena,
                            field.as_deref(),
                            data.footprint_radius,
                        )
                    })
            });
            if let Some(at) = found {
                spawn_enemy(&mut commands, data, at, director.spawned, scale);
                director.spawned += 1;
                alive += 1;
            }
            current.born[g] += 1;
            current.placed += 1;
        }
    }
    let all_born = horde
        .groups
        .iter()
        .zip(&current.born)
        .all(|(group, born)| *born >= group.count);
    // a campo pieno l'orda aspetta, ma non per sempre: finita la sua finestra lunga lascia stare
    if all_born || current.elapsed > plan.duration_s * 4.0 {
        *active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::schema::{HordeData, HordeGroup};

    fn plan() -> HordePlan {
        HordePlan {
            warning_s: 4.0,
            duration_s: 10.0,
            max_alive: 300,
            list: [300.0, 600.0]
                .map(|at_s| HordeData {
                    at_s,
                    groups: vec![HordeGroup {
                        enemy: "blob".into(),
                        count: 10,
                    }],
                })
                .to_vec(),
        }
    }

    #[test]
    fn the_alert_comes_before_and_stays_during_each_horde() {
        let p = plan();
        assert_eq!(horde_alert(&p, 295.9), None);
        assert_eq!(
            horde_alert(&p, 297.0),
            Some(HordeAlert::Coming {
                index: 0,
                in_s: 3.0
            })
        );
        assert_eq!(horde_alert(&p, 300.0), Some(HordeAlert::Here { index: 0 }));
        assert_eq!(horde_alert(&p, 309.9), Some(HordeAlert::Here { index: 0 }));
        assert_eq!(horde_alert(&p, 310.0), None);
        assert_eq!(horde_alert(&p, 605.0), Some(HordeAlert::Here { index: 1 }));
    }

    #[test]
    fn enemies_are_due_evenly_over_the_duration() {
        assert_eq!(due(10, 0.0, 10.0), 0);
        assert_eq!(due(10, 0.05, 10.0), 1);
        assert_eq!(due(10, 5.0, 10.0), 5);
        assert_eq!(due(10, 10.0, 10.0), 10);
        assert_eq!(due(10, 50.0, 10.0), 10);
    }

    #[test]
    fn circle_points_go_all_around_the_player() {
        let mut rng = Pcg32::new(1, HORDE_STREAM);
        let total = 8;
        for k in 0..total {
            let at = circle_point(&mut rng, Vec2::ZERO, (290.0, 360.0), k, total);
            let d = at.length();
            assert!((290.0..=360.0).contains(&d), "{d}");
            // ogni posto resta nel suo spicchio
            let turn = at.to_angle().rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
            let sector = (turn * total as f32).floor() as u32;
            assert_eq!(sector, k, "posto {k}");
        }
    }
}
