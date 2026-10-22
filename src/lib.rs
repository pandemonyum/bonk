//! Pixel Monster Survivor — survivor roguelite 2D in pixel art.
//!
//! La simulazione (`app`, `gameplay`, `world`) non dipende dal rendering: i test la eseguono
//! con [`headless_app`]. [`client_app`] aggiunge finestra, grafica, input, dati da file e
//! diagnostica.

pub mod app;
pub mod data;
pub mod diagnostics;
pub mod gameplay;
pub mod options;
pub mod presentation;
pub mod rng;
pub mod world;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowResolution;

use app::AppState;
use data::{DataStatus, GameConfig};
pub use options::LaunchOptions;

/// App completa per desktop e web. I dati arrivano da `game-data/` tramite l'AssetServer.
pub fn client_app(options: LaunchOptions) -> App {
    let (width, height) = options.window_size;
    let mut resolution = WindowResolution::new(width, height);
    if let Some(factor) = options.scale_factor {
        resolution = resolution.with_scale_factor_override(factor);
    }
    let mut app = App::new();
    data::loader::register_data_source(&mut app);
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                watch_for_changes_override: Some(options.watch_files()),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Monster Survivor".into(),
                    resolution,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        app::SimulationPlugin { seed: options.seed },
        data::loader::DataLoaderPlugin,
        gameplay::input::PlayerControlsPlugin,
        world::render::WorldRenderPlugin,
        presentation::PresentationPlugin,
        diagnostics::DiagnosticsPlugin,
    ))
    .insert_resource(options);
    app
}

/// Simulazione senza finestra né GPU, per test e benchmark, con i dati incorporati di
/// `game-data/`. Ogni `app.update()` esegue esattamente un passo fisso da 1/60 s e la run
/// esiste già dopo il primo.
pub fn headless_app(seed: u64) -> App {
    headless_app_with(seed, GameConfig::embedded())
}

/// Come [`headless_app`] ma con una configurazione data (es. valori modificati in un test).
pub fn headless_app_with(seed: u64, config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .add_plugins(app::SimulationPlugin { seed })
        .insert_state(AppState::InRun)
        .insert_resource(config)
        .insert_resource(DataStatus::Ready)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app
}
