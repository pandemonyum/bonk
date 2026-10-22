//! Presentazione (solo client): camera pixel-perfect, sprite, interpolazione, HUD.

pub mod camera;
pub mod hud;
pub mod sprites;

use bevy::prelude::*;

/// Livelli di profondità. Attori e proiettili sottraggono un piccolo y-sort (max ±0,2).
pub const Z_GROUND: f32 = 0.0;
pub const Z_ACTORS: f32 = 10.0;
pub const Z_PROJECTILES: f32 = 20.0;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::CameraPlugin, sprites::SpritesPlugin, hud::HudPlugin));
    }
}
