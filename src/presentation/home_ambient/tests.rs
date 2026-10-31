//! Animazioni della home senza finestra: layout incorporato e maschere coerenti, nodi figli
//! dell'illustrazione che non si duplicano, interruttori, fasi senza salti, posizione esatta.

use bevy::state::app::StatesPlugin;

use super::*;
use crate::presentation::home::{BackdropImage, HomePlugin, MenuScreen};

fn layout() -> AmbientLayout {
    AmbientLayout::parse(layout::LAYOUT_TEXT).expect("il layout incorporato è valido")
}

/// Larghezza e altezza di un PNG dall'intestazione IHDR, senza decodificarlo.
fn png_size(path: &str) -> (u32, u32) {
    let bytes = std::fs::read(format!("{}/assets/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(&bytes[1..4], b"PNG", "{path}");
    let word = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (word(16), word(20))
}

#[test]
fn the_layout_matches_the_illustration_and_its_masks() {
    let layout = layout();
    assert_eq!(layout.image, png_size("ui/home_background.png"));
    let names: Vec<_> = layout.effects.iter().map(|e| e.name.as_str()).collect();
    for wanted in ["portal", "falls_right", "river", "flame"] {
        assert!(names.contains(&wanted), "{wanted} manca: {names:?}");
    }
    for effect in &layout.effects {
        let rect = (effect.rect.2, effect.rect.3);
        assert_eq!(png_size(&effect.mask), rect, "{}", effect.name);
        if let Some(plate) = &effect.plate {
            assert_eq!(png_size(plate), rect, "{}", effect.name);
        }
    }
}

#[test]
fn a_broken_layout_is_refused() {
    let text =
        layout::LAYOUT_TEXT.replace("rect: (1330, 205, 195, 240)", "rect: (1600, 205, 195, 240)");
    assert!(AmbientLayout::parse(&text).unwrap_err().contains("fuori"));
    let first = layout().effects[0].name.clone();
    let doubled = layout::LAYOUT_TEXT.replacen(
        &format!("name: \"{}\"", layout().effects[1].name),
        &format!("name: \"{first}\""),
        1,
    );
    assert!(
        AmbientLayout::parse(&doubled)
            .unwrap_err()
            .contains("ripetuto")
    );
}

/// Le fasi sono frazioni che a 60 fps avanzano di poco: lo shader usa solo funzioni periodiche
/// di una fase (verificato leggendolo, non da questo test), quindi il giro non salta.
#[test]
fn the_phases_are_fractions_that_advance_a_little_per_frame() {
    for effect in layout().effects {
        let step = 1.0 / 60.0;
        let mut previous = phases(effect.kind, 0.0);
        for frame in 1..(60 * 120) {
            let now = phases(effect.kind, f64::from(frame) * step);
            for (a, b) in previous.to_array().into_iter().zip(now.to_array()) {
                let jump = (b - a).rem_euclid(1.0).min((a - b).rem_euclid(1.0));
                assert!(
                    jump < 0.05,
                    "{} salta di {jump} al frame {frame}",
                    effect.name
                );
            }
            previous = now;
        }
        assert!(rates(effect.kind).iter().all(|r| *r >= 0.0 && *r < 3.0));
    }
}

/// Lo shader non ha costanti sue per l'acqua: il periodo arriva con le uniform.
#[test]
fn waterfalls_send_their_flow_period_to_the_shader() {
    let layout = layout();
    for effect in &layout.effects {
        let params = AmbientParams::new(effect, layout.image_size());
        if matches!(effect.kind, AmbientKind::Waterfall { .. }) {
            assert_eq!(params.shape.w, material::FLOW_PERIOD, "{}", effect.name);
        }
    }
    let shader = include_str!("home_ambient.wesl");
    assert!(!shader.contains("const FLOW_PERIOD"));
}

#[test]
fn the_node_reads_the_illustration_where_the_backdrop_draws_it() {
    // illustrazione 1672x941 portata a 1920x1080 e spostata a sinistra dal taglio
    let scale = 1080.0 / 941.0;
    let image = Rect::from_corners(
        Vec2::new(-50.0, 0.0),
        Vec2::new(-50.0 + 1672.0 * scale, 1080.0),
    );
    let node = Rect::from_corners(
        image.min + Vec2::new(1330.0, 205.0) * scale,
        image.min + Vec2::new(1525.0, 445.0) * scale,
    );
    let uv = node_uv(node, image).unwrap();
    assert!((uv.x * 1672.0 - 1330.0).abs() < 1e-2 && (uv.y * 941.0 - 205.0).abs() < 1e-2);
    assert!((uv.z * 1672.0 - 195.0).abs() < 1e-2 && (uv.w * 941.0 - 240.0).abs() < 1e-2);
    // un nodo arrotondato di mezzo pixel resta allineato: lo shader legge dal punto vero
    let rounded = Rect::from_corners(node.min.round(), node.max.round());
    let uv = node_uv(rounded, image).unwrap();
    assert!(((image.min.x + uv.x * image.width()) - rounded.min.x).abs() < 1e-3);
    assert_eq!(node_uv(node, Rect::default()), None);
}

/// Il fondo mette l'illustrazione intera nel suo nodo arrotondato: un nodo di 1280x720 la
/// disegna larga 1279,3 e centrata, e lo shader deve leggere da lì.
#[test]
fn the_drawn_illustration_is_centered_in_its_rounded_node() {
    let node = Rect::from_corners(Vec2::ZERO, Vec2::new(1280.0, 720.0));
    let drawn = drawn_rect(node, Vec2::new(1672.0, 941.0));
    assert!((drawn.height() - 720.0).abs() < 1e-3);
    assert!((drawn.width() - 1672.0 * 720.0 / 941.0).abs() < 1e-3);
    assert!((drawn.min.x - (1280.0 - drawn.width()) / 2.0).abs() < 1e-3);
    assert_eq!(drawn_rect(node, Vec2::ZERO), node);
}

#[test]
fn the_switches_hide_effects_one_by_one_or_all() {
    let mut settings = HomeAmbient::default();
    assert!(settings.shows("portal") && settings.shows("river"));
    settings.only = Some(vec!["portal".into()]);
    assert!(settings.shows("portal") && !settings.shows("river"));
    settings.enabled = false;
    assert!(!settings.shows("portal"));
}

/// La pagina iniziale senza finestra, con le animazioni.
fn menu_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            ..default()
        },
        bevy::input::InputPlugin,
        bevy::image::ImagePlugin::default(),
    ))
    .init_asset::<AmbientMaterial>()
    .insert_state(AppState::MainMenu)
    .insert_resource(crate::data::GameConfig::embedded())
    .insert_resource(crate::app::RunSetup::default())
    .insert_resource(crate::app::AfterLoading(AppState::MainMenu))
    .insert_resource(crate::profile::Profile::default())
    .add_plugins((
        HomePlugin,
        crate::presentation::menu_list::MenuListPlugin,
        crate::presentation::menu::MenuPlugin,
        crate::presentation::options_menu::OptionsMenuPlugin,
        crate::presentation::talents_menu::TalentsMenuPlugin,
        HomeAmbientLogicPlugin,
    ));
    // senza render nessuno registra il caricatore dei PNG (lo fa il plugin delle texture)
    app.register_asset_loader(bevy::image::ImageLoader::new(
        bevy::image::CompressedImageFormats::NONE,
    ));
    app.update();
    app
}

fn effect_nodes(app: &mut App) -> Vec<(Entity, Entity)> {
    let world = app.world_mut();
    let mut nodes = world.query_filtered::<(Entity, &ChildOf), With<AmbientNode>>();
    nodes.iter(world).map(|(e, c)| (e, c.parent())).collect()
}

#[test]
fn effects_live_under_the_illustration_and_never_duplicate() {
    let mut app = menu_app();
    let count = layout().effects.len();
    let backdrop = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<BackdropImage>>();
        q.single(world).unwrap()
    };
    let nodes = effect_nodes(&mut app);
    assert_eq!(nodes.len(), count);
    assert!(nodes.iter().all(|(_, parent)| *parent == backdrop));
    // le altre schermate del menu non li toccano
    for screen in [MenuScreen::Options, MenuScreen::Home] {
        app.world_mut()
            .resource_mut::<NextState<MenuScreen>>()
            .set(screen);
        app.update();
        assert_eq!(effect_nodes(&mut app).len(), count, "{screen:?}");
    }
    // la run se li porta via con l'illustrazione; tornando al menu rinascono una volta sola
    for state in [
        AppState::InRun,
        AppState::MainMenu,
        AppState::InRun,
        AppState::MainMenu,
    ] {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(state);
        app.update();
        let expected = if state == AppState::MainMenu {
            count
        } else {
            0
        };
        assert_eq!(effect_nodes(&mut app).len(), expected, "{state:?}");
    }
    // i materiali delle illustrazioni passate si liberano con i loro nodi
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Assets<AmbientMaterial>>().len(),
        count
    );
}

/// Finché le maschere non sono caricate, o se l'interruttore è spento, resta il fondo fermo.
#[test]
fn effects_stay_hidden_until_ready_and_when_switched_off() {
    let mut app = menu_app();
    let hidden = |app: &mut App| {
        let world = app.world_mut();
        let mut q = world.query_filtered::<&Visibility, With<AmbientNode>>();
        q.iter(world).all(|v| *v == Visibility::Hidden)
    };
    let loaded = |app: &mut App| {
        let world = app.world_mut();
        let mut q = world.query::<&MaterialNode<AmbientMaterial>>();
        let handles: Vec<_> = q.iter(world).map(|m| m.0.clone()).collect();
        let (materials, images) = (
            world.resource::<Assets<AmbientMaterial>>(),
            world.resource::<Assets<Image>>(),
        );
        handles.iter().all(|h| {
            materials.get(h).is_some_and(|m| {
                images.contains(&m.art) && images.contains(&m.mask) && images.contains(&m.plate)
            })
        })
    };
    assert!(
        hidden(&mut app),
        "appena nati, prima di qualunque caricamento"
    );
    app.world_mut().resource_mut::<HomeAmbient>().enabled = false;
    // il caricamento dipende dalla macchina (test in parallelo): si aspetta fino a una scadenza
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !loaded(&mut app) {
        assert!(
            std::time::Instant::now() < deadline,
            "immagini non caricate in 60 s"
        );
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.update();
    assert!(hidden(&mut app), "caricate ma spente da Opzioni");
    app.world_mut().resource_mut::<HomeAmbient>().enabled = true;
    app.update();
    let world = app.world_mut();
    let mut q = world.query_filtered::<&Visibility, With<AmbientNode>>();
    let shown = q
        .iter(world)
        .filter(|v| **v == Visibility::Inherited)
        .count();
    assert_eq!(
        shown,
        layout().effects.len(),
        "caricate e accese, compaiono tutte"
    );
}
