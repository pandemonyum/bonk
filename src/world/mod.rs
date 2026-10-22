//! Arena "Prateria dei Germogli": dimensioni, limiti percorribili, ostacoli, generazione a seed.
//!
//! Decisione S0 (docs/DECISIONS.md): arena chiusa di 256×256 tile, centrata nell'origine,
//! divisa in chunk da 64×64 tile. Niente streaming.

pub mod generation;
pub mod obstacles;
pub mod render;
pub mod tiles;

use bevy::prelude::*;

use crate::app::{AppState, RunSeed};
use crate::data::GameConfig;
use crate::gameplay::kinematics::body;
use obstacles::{Obstacle, ObstacleField, generate_obstacles};

pub const TILE_SIZE: u32 = 16;
pub const CHUNK_TILES: u32 = 64;
pub const ARENA_CHUNKS: u32 = 4;
pub const ARENA_TILES: u32 = CHUNK_TILES * ARENA_CHUNKS;
/// Spessore in tile della siepe che chiude l'arena.
pub const BORDER_TILES: u32 = 2;
/// Mezza larghezza dell'arena in pixel di mondo.
pub const ARENA_HALF_EXTENT: f32 = (ARENA_TILES * TILE_SIZE) as f32 / 2.0;
/// Bordo interno della siepe.
const HEDGE_INNER: f32 = ARENA_HALF_EXTENT - (BORDER_TILES * TILE_SIZE) as f32;

/// Ingombro dei corpi rispetto alla `Position`, che è il punto dei piedi (gli sprite hanno
/// l'ancora in basso al centro). Valori S0 presi dallo sprite più grande, l'evocatore 16×21;
/// dalla P0.1 l'ingombro arriva dai dati di ogni entità.
const BODY_HALF_WIDTH: f32 = 8.0;
const BODY_HEIGHT: f32 = 21.0;
const FEET_MARGIN: f32 = 2.0;

/// Rettangolo percorribile per la `Position` (piedi) delle entità.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ArenaBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Default for ArenaBounds {
    fn default() -> Self {
        Self {
            min: Vec2::new(-HEDGE_INNER + BODY_HALF_WIDTH, -HEDGE_INNER + FEET_MARGIN),
            max: Vec2::new(HEDGE_INNER - BODY_HALF_WIDTH, HEDGE_INNER - BODY_HEIGHT),
        }
    }
}

/// Dentro il quadrato dell'intera arena, siepe compresa. È il limite dei proiettili: non i
/// limiti dei piedi (`ArenaBounds`), perché un colpo parte più in alto dei piedi e un nemico
/// respinto dal player può sforare i limiti di qualche pixel.
pub fn inside_arena(point: Vec2) -> bool {
    point.abs().cmple(Vec2::splat(ARENA_HALF_EXTENT)).all()
}

impl ArenaBounds {
    pub fn clamp(&self, point: Vec2) -> Vec2 {
        point.clamp(self.min, self.max)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArenaBounds>()
            .add_systems(OnEnter(AppState::InRun), spawn_obstacles);
    }
}

/// Genera gli ostacoli della run: la griglia `ObstacleField` per le collisioni e un'entità
/// per ostacolo per il rendering.
fn spawn_obstacles(mut commands: Commands, seed: Res<RunSeed>, config: Res<GameConfig>) {
    let field = ObstacleField::new(generate_obstacles(seed.0, &config.stage.obstacles));
    for obstacle in &field.obstacles {
        commands.spawn((
            Name::new(format!(
                "{:?} {},{}",
                obstacle.kind, obstacle.tile.x, obstacle.tile.y
            )),
            Obstacle {
                kind: obstacle.kind,
            },
            body(obstacle.base),
            DespawnOnExit(AppState::InRun),
        ));
    }
    commands.insert_resource(field);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_sprite_stays_inside_the_hedge() {
        let arena = ArenaBounds::default();
        assert_eq!(ARENA_HALF_EXTENT, 2048.0);
        assert_eq!(HEDGE_INNER, 2016.0);
        // fianchi a 8 px dalla siepe, piedi a 2 px dal bordo sud, testa a filo del bordo nord
        assert_eq!(arena.min, Vec2::new(-2008.0, -2014.0));
        assert_eq!(arena.max, Vec2::new(2008.0, 1995.0));
        assert_eq!(arena.max.y + BODY_HEIGHT, HEDGE_INNER);
        assert!(arena.contains(Vec2::ZERO));
        assert!(!arena.contains(Vec2::new(0.0, 2000.0)));
    }

    #[test]
    fn clamp_keeps_points_inside() {
        let arena = ArenaBounds::default();
        let clamped = arena.clamp(Vec2::new(-9999.0, 5000.0));
        assert!(arena.contains(clamped));
        assert_eq!(clamped, Vec2::new(arena.min.x, arena.max.y));
        assert_eq!(arena.clamp(Vec2::new(3.0, -4.0)), Vec2::new(3.0, -4.0));
    }
}
