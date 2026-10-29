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
pub mod profile;
pub mod rng;
pub mod web;
pub mod world;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowResolution;

use app::AppState;
use data::{DataStatus, GameConfig};
use gameplay::damage::GodMode;
use gameplay::upgrades::AutoPick;
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
                // sul web `assets-<versione>`: niente asset vecchi dalla cache del browser
                file_path: web::versioned_dir("assets"),
                watch_for_changes_override: Some(options.watch_files()),
                // nessun asset ha un file .meta: sul web ognuno costerebbe una richiesta in 404
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Monster Survivor".into(),
                    resolution,
                    // sul web: il canvas della pagina, grande quanto il suo contenitore, e i
                    // tasti (frecce, spazio, Tab) al gioco invece che alla pagina
                    canvas: cfg!(target_arch = "wasm32").then(|| "#game".to_string()),
                    fit_canvas_to_parent: cfg!(target_arch = "wasm32"),
                    prevent_default_event_handling: true,
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
    .insert_resource(GodMode(options.god));
    // il profilo del dispositivo; le run automatiche, invulnerabili o accelerate non lo salvano
    if let Some(path) = profile::default_path() {
        let saves = !options.autoplay && !options.god && options.time_scale == 1.0;
        let mut profile = profile::Profile::open(path);
        if !saves {
            profile.path = None;
        }
        app.insert_resource(profile);
    }
    // il menu all'avvio; con l'autoplay si parte subito con il compagno dello stage
    let first = if options.autoplay {
        AppState::InRun
    } else {
        AppState::MainMenu
    };
    app.insert_resource(app::AfterLoading(first));
    if options.autoplay {
        // il draft resta a schermo un attimo: si vede negli screenshot e a occhio
        app.insert_resource(AutoPick {
            delay_secs: AUTOPLAY_PICK_DELAY_SECS,
        });
    }
    let time_scale = options.time_scale;
    app.add_systems(Startup, move |mut time: ResMut<Time<Virtual>>| {
        time.set_relative_speed(time_scale);
    })
    .insert_resource(options);
    app
}

/// Secondi reali per cui l'autoplay lascia aperto il draft prima di scegliere la prima carta.
pub const AUTOPLAY_PICK_DELAY_SECS: f32 = 0.6;

/// Simulazione senza finestra né GPU, per test e benchmark, con i dati incorporati di
/// `game-data/`. Ogni `app.update()` esegue esattamente un passo fisso da 1/60 s e la run
/// esiste già dopo il primo. Nel draft viene scelta subito la prima carta ([`AutoPick`]):
/// i test del draft tolgono la risorsa per scegliere da sé.
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
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .insert_resource(AutoPick { delay_secs: 0.0 });
    app
}
