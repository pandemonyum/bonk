//! Presentazione (solo client): camera pixel-perfect, sprite, interpolazione, HUD, draft, menu,
//! risultati, segni di pericolo.

pub mod abilities;
pub mod attacks;
pub mod boss_bar;
pub mod boss_pointer;
pub mod camera;
pub mod chest_ui;
pub mod companion_anim;
pub mod draft;
pub mod enemy_anim;
pub mod fonts;
pub mod health_bar;
pub mod home;
pub mod home_ambient;
pub mod horde_banner;
pub mod hud;
pub mod icons;
pub mod menu;
pub mod menu_list;
mod options_audio;
pub mod options_menu;
pub mod player_anim;
pub mod recap;
pub mod results;
pub mod sprites;
pub mod status;
pub mod talent_art;
pub mod talent_text;
pub mod talents_menu;
pub mod talents_tree;
pub mod talents_view;
pub mod telegraphs;
pub mod treasure_art;

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
        ))
        // `add_plugins` accetta al massimo 15 plugin per tupla
        .add_plugins((
            player_anim::PlayerAnimPlugin,
            enemy_anim::EnemyAnimPlugin,
            companion_anim::CompanionAnimPlugin,
            horde_banner::HordeBannerPlugin,
            treasure_art::TreasureArtPlugin,
            chest_ui::ChestUiPlugin,
            talent_art::TalentArtPlugin,
            talents_menu::TalentsMenuPlugin,
            home::HomePlugin,
            home_ambient::HomeAmbientPlugin,
            menu_list::MenuListPlugin,
            options_menu::OptionsMenuPlugin,
        ));
    }
}
