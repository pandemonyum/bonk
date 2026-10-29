//! Geometria della vista: quale parte dell'arena può essere a schermo. Sta nel mondo e non
//! nella presentazione perché anche la simulazione ne ha bisogno (lo spawn director e i
//! forzieri non devono nascere in vista). La camera della presentazione usa le stesse funzioni.
//!
//! La vista minima è 480×270 pixel di mondo: la scala intera del canvas si sceglie perché ci
//! stia (D-013). Se la finestra non è un multiplo esatto il canvas cresce invece delle bande
//! nere, fino a 640×360 (D-062): per la simulazione "in vista" è sempre tutto questo
//! rettangolo, così nessun nemico nasce sotto gli occhi in nessuna finestra e il gioco resta
//! lo stesso con ogni finestra.

use bevy::math::{Rect, Vec2};

use super::ARENA_HALF_EXTENT;

/// Vista minima: pixel di mondo sempre a schermo (la scala del canvas si sceglie su questa).
pub const VIEW_WIDTH: u32 = 480;
pub const VIEW_HEIGHT: u32 = 270;
/// Vista più grande che si mostra, con le finestre che non sono un multiplo esatto.
pub const MAX_VIEW_WIDTH: u32 = 640;
pub const MAX_VIEW_HEIGHT: u32 = 360;
/// Metà della diagonale della vista più grande (~367 px): oltre questa distanza dal centro un
/// punto è fuori vista in qualunque direzione e con qualunque finestra.
pub const VIEW_HALF_DIAGONAL: f32 = 367.151_2; // √(320² + 180²)

fn half_view() -> Vec2 {
    Vec2::new(MAX_VIEW_WIDTH as f32, MAX_VIEW_HEIGHT as f32) / 2.0
}

/// Centro della vista per un player in `player`: lo segue, ma senza mostrare fuori dall'arena
/// nemmeno con la vista più grande.
pub fn view_center(player: Vec2) -> Vec2 {
    let limit = Vec2::splat(ARENA_HALF_EXTENT) - half_view();
    player.clamp(-limit, limit)
}

/// Rettangolo dell'arena che può essere a schermo quando il player è in `player` (la vista più
/// grande: quella vera, che dipende dalla finestra, ci sta dentro).
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
            Vec2::new(2048.0 - 320.0, 2048.0 - 180.0)
        );
        let rect = view_rect(Vec2::ZERO);
        assert_eq!(rect.min, Vec2::new(-320.0, -180.0));
        assert_eq!(rect.max, Vec2::new(320.0, 180.0));
        // la vista minima ci sta dentro
        assert!(rect.contains(Vec2::new(VIEW_WIDTH as f32, VIEW_HEIGHT as f32) / 2.0));
    }
}
