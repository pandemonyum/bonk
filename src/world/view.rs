//! Geometria della vista: quale parte dell'arena è a schermo. Sta nel mondo e non nella
//! presentazione perché anche la simulazione ne ha bisogno (lo spawn director non deve far
//! nascere nemici in vista). La camera della presentazione usa le stesse funzioni.

use bevy::math::{Rect, Vec2};

use super::ARENA_HALF_EXTENT;

/// Risoluzione virtuale: pixel di mondo visibili a schermo.
pub const VIEW_WIDTH: u32 = 480;
pub const VIEW_HEIGHT: u32 = 270;
/// Metà della diagonale della vista (~275 px): oltre questa distanza dal centro un punto è
/// fuori vista in qualunque direzione.
pub const VIEW_HALF_DIAGONAL: f32 = 275.363_5; // √(240² + 135²)

fn half_view() -> Vec2 {
    Vec2::new(VIEW_WIDTH as f32, VIEW_HEIGHT as f32) / 2.0
}

/// Centro della vista per un player in `player`: lo segue, ma senza mostrare fuori dall'arena.
pub fn view_center(player: Vec2) -> Vec2 {
    let limit = Vec2::splat(ARENA_HALF_EXTENT) - half_view();
    player.clamp(-limit, limit)
}

/// Rettangolo dell'arena a schermo quando il player è in `player`.
pub fn view_rect(player: Vec2) -> Rect {
    Rect::from_center_half_size(view_center(player), half_view())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_diagonal_matches_the_view_size() {
        assert!((half_view().length() - VIEW_HALF_DIAGONAL).abs() < 1e-3);
    }

    #[test]
    fn view_follows_the_player_but_never_leaves_the_arena() {
        assert_eq!(view_center(Vec2::new(10.0, -20.0)), Vec2::new(10.0, -20.0));
        assert_eq!(
            view_center(Vec2::splat(5000.0)),
            Vec2::new(2048.0 - 240.0, 2048.0 - 135.0)
        );
        let rect = view_rect(Vec2::ZERO);
        assert_eq!(rect.min, Vec2::new(-240.0, -135.0));
        assert_eq!(rect.max, Vec2::new(240.0, 135.0));
    }
}
