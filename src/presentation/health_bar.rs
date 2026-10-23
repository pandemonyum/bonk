//! Barra degli HP sotto i piedi del player, nel mondo (texture 480×270): 16×2 texel rossi come
//! i cuori, con contorno. Sempre visibile, così gli HP si leggono senza staccare gli occhi dalla
//! calca; la larghezza va a texel interi come il resto della pixel art.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::gameplay::combat::Health;
use crate::gameplay::player::Player;

/// Larghezza e altezza del riempimento, in texel.
const WIDTH: f32 = 16.0;
const HEIGHT: f32 = 2.0;
/// Centro della barra rispetto ai piedi (ancora del player in basso al centro).
const BELOW_FEET: f32 = -3.0;
/// Sopra gli attori vicini (y-sort ±0,2), sotto i proiettili.
const ABOVE_ACTORS: f32 = 0.5;
/// Palette v1: contorno `k`, fondo `j`, riempimento `r`.
const OUTLINE: Color = Color::srgb_u8(0x1b, 0x14, 0x28);
const BACK: Color = Color::srgb_u8(0x5c, 0x1a, 0x2c);
const FILL: Color = Color::srgb_u8(0xe2, 0x57, 0x4c);

#[derive(Component)]
struct HealthBarFill;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_health_bar)
            .add_systems(Update, show_health);
    }
}

/// Texel di riempimento per `current` su `max` HP: interi, almeno uno finché si è vivi.
pub fn filled_texels(current: f32, max: f32) -> f32 {
    if current <= 0.0 || max <= 0.0 {
        return 0.0;
    }
    (WIDTH * (current / max).min(1.0)).floor().max(1.0)
}

fn attach_health_bar(add: On<Add<Player>>, mut commands: Commands) {
    let bar = commands
        .spawn((
            Name::new("HealthBar"),
            Sprite::from_color(OUTLINE, Vec2::new(WIDTH + 2.0, HEIGHT + 2.0)),
            Transform::from_xyz(0.0, BELOW_FEET, ABOVE_ACTORS),
            children![
                (
                    Sprite::from_color(BACK, Vec2::new(WIDTH, HEIGHT)),
                    Anchor::CENTER_LEFT,
                    Transform::from_xyz(-WIDTH / 2.0, 0.0, 0.01),
                ),
                (
                    HealthBarFill,
                    Sprite::from_color(FILL, Vec2::new(WIDTH, HEIGHT)),
                    Anchor::CENTER_LEFT,
                    Transform::from_xyz(-WIDTH / 2.0, 0.0, 0.02),
                ),
            ],
        ))
        .id();
    commands.entity(add.entity).add_child(bar);
}

fn show_health(
    player: Query<&Health, With<Player>>,
    mut fills: Query<&mut Sprite, With<HealthBarFill>>,
) {
    let Ok(health) = player.single() else {
        return;
    };
    let width = filled_texels(health.current, health.max);
    for mut sprite in &mut fills {
        let size = Vec2::new(width, HEIGHT);
        if sprite.custom_size != Some(size) {
            sprite.custom_size = Some(size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_fills_by_whole_texels_and_never_vanishes_while_alive() {
        assert_eq!(filled_texels(100.0, 100.0), 16.0);
        assert_eq!(filled_texels(50.0, 100.0), 8.0);
        assert_eq!(filled_texels(59.0, 100.0), 9.0);
        assert_eq!(filled_texels(0.5, 120.0), 1.0);
        assert_eq!(filled_texels(0.0, 100.0), 0.0);
        assert_eq!(filled_texels(-4.0, 100.0), 0.0);
        assert_eq!(filled_texels(130.0, 120.0), 16.0);
    }
}
