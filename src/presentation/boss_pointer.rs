//! Indicatore del boss fuori schermo: il teschio su un riquadro rosso al bordo della vista, nella
//! direzione del boss, finché il boss non entra a schermo. Il boss nasce fuori vista (come ogni
//! nemico) e senza indicatore si poteva girare a lungo senza trovarlo.

use bevy::prelude::*;

use super::icons::{UI_PX_PER_TEXEL, ui_icon};
use crate::app::AppState;
use crate::gameplay::enemies::Boss;
use crate::gameplay::kinematics::Position;
use crate::gameplay::player::Player;
use crate::world::view::view_rect;

/// Lato del riquadro in texel.
const SIDE_TEXELS: f32 = 14.0;
/// Distanza del centro del riquadro dai bordi della vista, in texel: in alto resta sotto la
/// fascia di timer, barra del boss e pannelli di stato (~43 texel), ai lati e in basso vicino
/// al bordo.
const MARGIN_SIDE: f32 = 12.0;
const MARGIN_TOP: f32 = 50.0;
const MARGIN_BOTTOM: f32 = 12.0;
/// Sopra il resto dell'HUD, sotto il pannello degli errori dei dati (10).
const POINTER_Z: i32 = 5;
/// Palette v1: rosso pericolo scuro `j` e contorno `k`.
const BACK: Color = Color::srgb_u8(0x5c, 0x1a, 0x2c);
const OUTLINE: Color = Color::srgb_u8(0x1b, 0x14, 0x28);

#[derive(Component)]
struct BossPointer;

pub struct BossPointerPlugin;

impl Plugin for BossPointerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_pointer)
            .add_systems(Update, point_at_boss.run_if(in_state(AppState::InRun)));
    }
}

/// Dove indicare il boss, in texel dal centro della vista (y in alto), lungo la sua direzione:
/// sul rettangolo da `inner_min` a `inner_max` (rispetto al centro, `inner_min` negativo).
/// `None` se il boss è a schermo.
pub fn edge_point(offset: Vec2, half_view: Vec2, inner_min: Vec2, inner_max: Vec2) -> Option<Vec2> {
    if offset.x.abs() <= half_view.x && offset.y.abs() <= half_view.y {
        return None;
    }
    // per asse: quanto si può avanzare verso il lato dalla parte del boss (infinito se nullo)
    let limit = |v: f32, low: f32, high: f32| {
        if v > 0.0 {
            high / v
        } else if v < 0.0 {
            low / v
        } else {
            f32::INFINITY
        }
    };
    let t =
        limit(offset.x, inner_min.x, inner_max.x).min(limit(offset.y, inner_min.y, inner_max.y));
    Some(offset * t)
}

/// Rettangolo dell'indicatore per una vista di mezze misure `half_view`.
fn inner_rect(half_view: Vec2) -> (Vec2, Vec2) {
    (
        Vec2::new(MARGIN_SIDE - half_view.x, MARGIN_BOTTOM - half_view.y),
        Vec2::new(half_view.x - MARGIN_SIDE, half_view.y - MARGIN_TOP),
    )
}

/// Un nodo a dimensione zero al centro della finestra (che è anche il centro del canvas) fa da
/// origine: l'indicatore si sposta da lì in unità dell'interfaccia.
fn spawn_pointer(mut commands: Commands, assets: Res<AssetServer>) {
    let side = SIDE_TEXELS * UI_PX_PER_TEXEL;
    commands.spawn((
        Name::new("BossPointerOrigin"),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            ..default()
        },
        GlobalZIndex(POINTER_Z),
        DespawnOnExit(AppState::InRun),
        children![(
            BossPointer,
            Node {
                position_type: PositionType::Absolute,
                width: px(side),
                height: px(side),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(UI_PX_PER_TEXEL)),
                ..default()
            },
            BorderColor::all(OUTLINE),
            BackgroundColor(BACK),
            Visibility::Hidden,
            children![ui_icon(&assets, "ui/skull.png", 10.0, 10.0)],
        )],
    ));
}

fn point_at_boss(
    player: Query<&Position, With<Player>>,
    boss: Query<&Position, With<Boss>>,
    pointer: Single<(&mut Node, &mut Visibility), With<BossPointer>>,
) {
    let (mut node, mut visibility) = pointer.into_inner();
    let spot = match (player.single(), boss.single()) {
        (Ok(me), Ok(boss)) => {
            let view = view_rect(me.0);
            let (low, high) = inner_rect(view.half_size());
            edge_point(boss.0 - view.center(), view.half_size(), low, high)
        }
        _ => None,
    };
    let Some(spot) = spot else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    // texel interi, poi unità dell'interfaccia (y verso il basso), centrato sul punto
    let half = SIDE_TEXELS * UI_PX_PER_TEXEL / 2.0;
    let left = px(spot.x.round() * UI_PX_PER_TEXEL - half);
    let top = px(-spot.y.round() * UI_PX_PER_TEXEL - half);
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    if *visibility != Visibility::Inherited {
        *visibility = Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF: Vec2 = Vec2::new(240.0, 135.0);

    fn spot(offset: Vec2) -> Option<Vec2> {
        let (low, high) = inner_rect(HALF);
        edge_point(offset, HALF, low, high)
    }

    #[test]
    fn a_boss_on_screen_needs_no_pointer() {
        assert_eq!(spot(Vec2::new(100.0, -50.0)), None);
        assert_eq!(spot(Vec2::new(240.0, 135.0)), None);
    }

    #[test]
    fn the_pointer_sits_on_the_inner_edge_towards_the_boss() {
        // dritto a destra e a sinistra: sui lati rientrati
        assert_eq!(spot(Vec2::new(900.0, 0.0)), Some(Vec2::new(228.0, 0.0)));
        assert_eq!(spot(Vec2::new(-900.0, 0.0)), Some(Vec2::new(-228.0, 0.0)));
        // in basso vicino al bordo, in alto sotto la fascia dell'HUD
        assert_eq!(spot(Vec2::new(0.0, -400.0)), Some(Vec2::new(0.0, -123.0)));
        assert_eq!(spot(Vec2::new(0.0, 400.0)), Some(Vec2::new(0.0, 85.0)));
        // in diagonale tocca prima il lato più vicino, nella stessa direzione
        let up_right = spot(Vec2::new(500.0, 500.0)).unwrap();
        assert_eq!(up_right.y, 85.0);
        assert!((up_right.x - 85.0).abs() < 1e-4);
        let down_left = spot(Vec2::new(-500.0, -500.0)).unwrap();
        assert_eq!(down_left.y, -123.0);
        assert!((down_left.x + 123.0).abs() < 1e-4);
    }
}
