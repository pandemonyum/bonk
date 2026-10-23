//! Presentazione (solo client): camera pixel-perfect, sprite, interpolazione, HUD, draft, menu,
//! risultati, segni di pericolo.

pub mod abilities;
pub mod attacks;
pub mod boss_bar;
pub mod boss_pointer;
pub mod camera;
pub mod draft;
pub mod fonts;
pub mod health_bar;
pub mod hud;
pub mod icons;
pub mod menu;
pub mod results;
pub mod sprites;
pub mod status;
pub mod telegraphs;

use bevy::prelude::*;

/// Livelli di profondità. Attori e proiettili sottraggono un piccolo y-sort (max ±0,2).
pub const Z_GROUND: f32 = 0.0;
/// Gemme e oggetti a terra: sopra il suolo, sotto chiunque ci passi sopra.
pub const Z_PICKUPS: f32 = 5.0;
pub const Z_ACTORS: f32 = 10.0;
pub const Z_PROJECTILES: f32 = 20.0;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            fonts::FontsPlugin,
            camera::CameraPlugin,
            sprites::SpritesPlugin,
            hud::HudPlugin,
            abilities::AbilitiesPlugin,
            status::StatusPlugin,
            boss_bar::BossBarPlugin,
            boss_pointer::BossPointerPlugin,
            health_bar::HealthBarPlugin,
            draft::DraftUiPlugin,
            attacks::AttackArtPlugin,
            telegraphs::TelegraphArtPlugin,
            menu::MenuPlugin,
            results::ResultsPlugin,
        ));
    }
}
