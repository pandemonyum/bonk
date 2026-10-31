//! Lo stato della simulazione che il confronto della trasparenza guarda, e la prima differenza
//! fra due run. Le posizioni sono in bit (un ulp conta) e ordinate come in `movement.rs`.

use std::cmp::Ordering;
use std::fmt::Debug;

use bevy::prelude::*;
use pixel_monster_survivor::app::{AppState, RunPhase};
use pixel_monster_survivor::gameplay::combat::RunStats;
use pixel_monster_survivor::gameplay::progression::Progression;
use pixel_monster_survivor::gameplay::run::RunClock;
use pixel_monster_survivor::gameplay::treasure::Treasury;

use crate::common::{enemy_positions, gems, player_position};

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub app_state: Option<AppState>,
    pub run_phase: Option<RunPhase>,
    pub clock: Option<RunClock>,
    pub stats: Option<RunStats>,
    pub progression: Option<Progression>,
    pub treasury: Option<Treasury>,
    pub player: [u32; 2],
    pub enemies: Vec<[u32; 2]>,
    pub gems: Vec<([u32; 2], u32)>,
}

fn bits(p: Vec2) -> [u32; 2] {
    [p.x.to_bits(), p.y.to_bits()]
}

fn by_position(a: &Vec2, b: &Vec2) -> Ordering {
    a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y))
}

pub fn snapshot(app: &mut App) -> Snapshot {
    fn cloned<T: Resource + Clone>(app: &App) -> Option<T> {
        app.world().get_resource::<T>().cloned()
    }
    fn state<S: States>(app: &App) -> Option<S> {
        app.world()
            .get_resource::<State<S>>()
            .map(|s| s.get().clone())
    }
    let mut enemies = enemy_positions(app);
    enemies.sort_by(by_position);
    let mut gems = gems(app);
    gems.sort_by(|a, b| by_position(&a.0, &b.0).then(a.1.cmp(&b.1)));
    Snapshot {
        app_state: state(app),
        run_phase: state(app),
        clock: cloned(app),
        stats: cloned(app),
        progression: cloned(app),
        treasury: cloned(app),
        player: bits(player_position(app)),
        enemies: enemies.into_iter().map(bits).collect(),
        gems: gems.into_iter().map(|(p, v)| (bits(p), v)).collect(),
    }
}

/// Il primo campo diverso, con i due valori (le liste lunghe dal primo elemento diverso).
pub fn first_difference(a: &Snapshot, b: &Snapshot) -> Option<String> {
    fn field<T: PartialEq + Debug>(name: &str, a: &T, b: &T) -> Option<String> {
        (a != b).then(|| format!("{name}: senza audio {a:?}, con l'audio {b:?}"))
    }
    fn list<T: PartialEq + Debug>(name: &str, a: &[T], b: &[T]) -> Option<String> {
        let at = a.iter().zip(b).position(|(x, y)| x != y);
        match at {
            Some(i) => Some(format!(
                "{name}[{i}]: senza audio {:?}, con l'audio {:?}",
                a[i], b[i]
            )),
            None => (a.len() != b.len())
                .then(|| format!("{name}: {} senza audio, {} con l'audio", a.len(), b.len())),
        }
    }
    field("AppState", &a.app_state, &b.app_state)
        .or_else(|| field("RunPhase", &a.run_phase, &b.run_phase))
        .or_else(|| field("RunClock", &a.clock, &b.clock))
        .or_else(|| field("RunStats", &a.stats, &b.stats))
        .or_else(|| field("Progression", &a.progression, &b.progression))
        .or_else(|| field("Treasury", &a.treasury, &b.treasury))
        .or_else(|| field("player (bit)", &a.player, &b.player))
        .or_else(|| list("nemici (bit)", &a.enemies, &b.enemies))
        .or_else(|| list("gemme (bit, valore)", &a.gems, &b.gems))
}

/// Il primo secondo (da 1) in cui `b` diverge da `a`, con la differenza, sui secondi che le due
/// run hanno entrambe. Lo usano sia il confronto vero sia la prova con il player spostato.
pub fn divergence(a: &[Snapshot], b: &[Snapshot]) -> Option<(usize, String)> {
    a.iter()
        .zip(b)
        .enumerate()
        .find_map(|(i, (x, y))| first_difference(x, y).map(|difference| (i + 1, difference)))
}
