//! Collisioni semplici a cerchi sull'impronta a terra (piedi): corpi contro ostacoli statici.
//! Niente fisica: chi entra in un'impronta viene riportato sul bordo, e la componente del
//! moto tangente al bordo resta, così si scivola attorno agli ostacoli.

use bevy::prelude::*;

use crate::world::obstacles::ObstacleField;

/// Raggio dell'impronta a terra di un corpo mobile, centrata sulla `Position` (piedi).
/// Viene dai dati (`footprint_radius`), validato contro il varco minimo tra ostacoli.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Footprint(pub f32);

/// Riporta `point` sul bordo del cerchio `(center, radius)` se ci è entrato.
pub fn push_outside(point: Vec2, center: Vec2, radius: f32) -> Vec2 {
    let delta = point - center;
    let distance = delta.length();
    if distance >= radius {
        return point;
    }
    let direction = if distance > f32::EPSILON {
        delta / distance
    } else {
        Vec2::X
    };
    center + direction * radius
}

/// Posizione di un corpo di raggio `radius` dopo la respinta dagli ostacoli vicini. Una passata
/// basta: i varchi tra ostacoli (≥ 18 px) sono più larghi di qualunque corpo, quindi un corpo
/// non tocca mai due ostacoli insieme.
pub fn resolve_against_obstacles(point: Vec2, radius: f32, field: &ObstacleField) -> Vec2 {
    field.near(point, radius).fold(point, |at, obstacle| {
        push_outside(
            at,
            obstacle.footprint_center(),
            obstacle.footprint_radius() + radius,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::obstacles::{ObstacleKind, ObstacleSpec};

    #[test]
    fn push_outside_only_moves_points_inside_the_circle() {
        let pushed = push_outside(Vec2::new(3.0, 4.0), Vec2::ZERO, 10.0);
        assert!((pushed - Vec2::new(6.0, 8.0)).length() < 1e-4);
        assert_eq!(
            push_outside(Vec2::new(30.0, 0.0), Vec2::ZERO, 10.0),
            Vec2::new(30.0, 0.0)
        );
        assert_eq!(
            push_outside(Vec2::ONE, Vec2::ONE, 10.0),
            Vec2::new(11.0, 1.0)
        );
    }

    #[test]
    fn bodies_are_kept_out_of_obstacle_footprints() {
        let rock = ObstacleSpec {
            kind: ObstacleKind::Rock,
            tile: UVec2::new(140, 140),
            base: Vec2::new(200.0, 200.0),
        };
        let field = ObstacleField::new(vec![rock]);
        let center = rock.footprint_center();
        let inside = center + Vec2::new(2.0, 0.0);
        let resolved = resolve_against_obstacles(inside, 5.0, &field);
        assert!((resolved.distance(center) - (rock.footprint_radius() + 5.0)).abs() < 1e-4);
        let far = Vec2::new(-500.0, 0.0);
        assert_eq!(resolve_against_obstacles(far, 5.0, &field), far);
    }
}
