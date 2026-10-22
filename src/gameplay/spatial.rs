//! Griglia spaziale dei nemici, ricostruita a ogni passo fisso: separazione, mira, colpi dei
//! proiettili e danni da contatto chiedono "chi c'è qui vicino?" in O(vicini) invece di
//! confrontare ogni coppia (con 220 nemici: ~50 confronti invece di ~24 mila per la separazione).
//!
//! Struttura: le voci sono ordinate per cella (`y * CELLS + x`) e `cell_start` indica dove
//! inizia ogni cella, quindi le celle di una riga sono contigue e una query legge poche fette.
//! L'ordinamento è stabile: a parità di cella vale l'ordine di inserimento (deterministico).

use bevy::prelude::*;

use super::SimSet;
use super::enemies::Enemy;
use super::kinematics::Position;
use crate::world::ARENA_HALF_EXTENT;

/// Lato di una cella: dell'ordine dei raggi di separazione e di contatto.
const CELL_SIZE: f32 = 32.0;
const CELLS: usize = (2.0 * ARENA_HALF_EXTENT / CELL_SIZE) as usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridEntry {
    pub entity: Entity,
    /// Piedi del nemico al momento della ricostruzione.
    pub position: Vec2,
    pub separation_radius: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyGrid {
    cell_start: Vec<u32>,
    entries: Vec<GridEntry>,
    sorting: Vec<(u32, GridEntry)>,
    /// Quanto un nemico può "sporgere" dai propri piedi (hurtbox, contatto, separazione): le
    /// query lo aggiungono al raggio richiesto, poi chi chiama filtra con la distanza esatta.
    max_reach: f32,
}

impl Default for EnemyGrid {
    fn default() -> Self {
        Self {
            cell_start: vec![0; CELLS * CELLS + 1],
            entries: Vec::new(),
            sorting: Vec::new(),
            max_reach: 0.0,
        }
    }
}

/// Cella di un punto; fuori dall'arena si usa la cella di bordo più vicina, così anche un
/// nemico respinto oltre i limiti resta trovabile.
fn cell_xy(point: Vec2) -> (usize, usize) {
    let local = ((point + Vec2::splat(ARENA_HALF_EXTENT)) / CELL_SIZE).floor();
    let clamp = |v: f32| (v.max(0.0) as usize).min(CELLS - 1);
    (clamp(local.x), clamp(local.y))
}

impl EnemyGrid {
    /// Ricostruisce la griglia da `(voce, sporgenza)`.
    pub fn rebuild(&mut self, items: impl IntoIterator<Item = (GridEntry, f32)>) {
        self.sorting.clear();
        self.max_reach = 0.0;
        for (entry, reach) in items {
            let (x, y) = cell_xy(entry.position);
            self.sorting.push(((y * CELLS + x) as u32, entry));
            self.max_reach = self.max_reach.max(reach);
        }
        self.sorting.sort_by_key(|(cell, _)| *cell); // stabile
        self.cell_start.fill(0);
        for (cell, _) in &self.sorting {
            self.cell_start[*cell as usize + 1] += 1;
        }
        for i in 1..self.cell_start.len() {
            self.cell_start[i] += self.cell_start[i - 1];
        }
        self.entries.clear();
        self.entries
            .extend(self.sorting.iter().map(|(_, entry)| *entry));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Voci che possono stare entro `radius` da `center` (più la sporgenza massima). È un
    /// superinsieme: chi chiama controlla la distanza esatta.
    pub fn candidates(&self, center: Vec2, radius: f32) -> impl Iterator<Item = &GridEntry> {
        let reach = Vec2::splat(radius + self.max_reach);
        let (x0, y0) = cell_xy(center - reach);
        let (x1, y1) = cell_xy(center + reach);
        (y0..=y1).flat_map(move |y| {
            let row = y * CELLS;
            let start = self.cell_start[row + x0] as usize;
            let end = self.cell_start[row + x1 + 1] as usize;
            self.entries[start..end].iter()
        })
    }
}

pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyGrid>()
            // Prima della separazione (posizioni dopo il movimento) e di nuovo prima del
            // combattimento (posizioni dopo separazione, ostacoli e limiti).
            .add_systems(
                FixedUpdate,
                rebuild_before_resolve
                    .in_set(SimSet::Resolve)
                    .before(super::enemies::EnemyResolve),
            )
            .add_systems(
                FixedUpdate,
                rebuild_before_combat
                    .in_set(SimSet::Combat)
                    .before(super::combat::CombatSteps),
            );
    }
}

/// Sporgenza di un nemico oltre i piedi: la più grande tra hurtbox, contatto e separazione.
pub fn enemy_reach(enemy: &Enemy, hurtbox: &super::combat::Hurtbox) -> f32 {
    (hurtbox.radius + hurtbox.center_offset)
        .max(enemy.contact_distance)
        .max(enemy.separation_radius)
}

fn rebuild(
    grid: &mut EnemyGrid,
    enemies: &Query<(Entity, &Enemy, &super::combat::Hurtbox, &Position)>,
) {
    grid.rebuild(enemies.iter().map(|(entity, enemy, hurtbox, position)| {
        let entry = GridEntry {
            entity,
            position: position.0,
            separation_radius: enemy.separation_radius,
        };
        (entry, enemy_reach(enemy, hurtbox))
    }));
}

fn rebuild_before_resolve(
    mut grid: ResMut<EnemyGrid>,
    enemies: Query<(Entity, &Enemy, &super::combat::Hurtbox, &Position)>,
) {
    rebuild(&mut grid, &enemies);
}

fn rebuild_before_combat(
    mut grid: ResMut<EnemyGrid>,
    enemies: Query<(Entity, &Enemy, &super::combat::Hurtbox, &Position)>,
) {
    rebuild(&mut grid, &enemies);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Pcg32;

    fn entry(index: u32, position: Vec2) -> GridEntry {
        GridEntry {
            entity: Entity::from_raw_u32(index).unwrap(),
            position,
            separation_radius: 14.0,
        }
    }

    #[test]
    fn candidates_contain_every_point_within_the_radius() {
        let mut rng = Pcg32::new(4, 0);
        let half = ARENA_HALF_EXTENT + 20.0; // anche punti appena fuori dall'arena
        let points: Vec<Vec2> = (0..600)
            .map(|_| Vec2::new(rng.range_f32(-half, half), rng.range_f32(-half, half)))
            .collect();
        let mut grid = EnemyGrid::default();
        grid.rebuild(
            points
                .iter()
                .enumerate()
                .map(|(i, p)| (entry(i as u32 + 1, *p), 0.0)),
        );
        assert_eq!(grid.len(), points.len());
        let mut probes: Vec<Vec2> = points.iter().step_by(7).copied().collect();
        probes.extend([Vec2::splat(half), Vec2::splat(-half), Vec2::ZERO]);
        for probe in probes {
            for radius in [5.0, 40.0, 150.0] {
                let found: Vec<Vec2> = grid
                    .candidates(probe, radius)
                    .map(|e| e.position)
                    .filter(|p| p.distance(probe) <= radius)
                    .collect();
                let expected = points
                    .iter()
                    .filter(|p| p.distance(probe) <= radius)
                    .count();
                assert_eq!(found.len(), expected, "sonda {probe:?} raggio {radius}");
            }
        }
    }

    #[test]
    fn candidates_are_a_small_subset_and_reach_extends_the_query() {
        let mut grid = EnemyGrid::default();
        let far = Vec2::new(1000.0, 1000.0);
        grid.rebuild([(entry(1, Vec2::ZERO), 0.0), (entry(2, far), 0.0)]);
        assert_eq!(grid.candidates(Vec2::ZERO, 10.0).count(), 1);
        // con una sporgenza di 20 px un nemico a 25 px torna candidato per una query di 10 px
        grid.rebuild([(entry(1, Vec2::new(25.0, 0.0)), 20.0)]);
        assert_eq!(grid.candidates(Vec2::ZERO, 10.0).count(), 1);
    }

    #[test]
    fn same_cell_keeps_insertion_order() {
        let mut grid = EnemyGrid::default();
        let items = (1..=5).map(|i| (entry(i, Vec2::splat(i as f32)), 0.0));
        grid.rebuild(items);
        let order: Vec<u32> = grid
            .candidates(Vec2::ZERO, 10.0)
            .map(|e| e.entity.index_u32())
            .collect();
        assert_eq!(order, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn an_empty_grid_has_no_candidates() {
        let grid = EnemyGrid::default();
        assert!(grid.is_empty());
        assert_eq!(grid.candidates(Vec2::ZERO, 500.0).count(), 0);
    }
}
