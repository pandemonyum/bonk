//! Animazioni ambientali della pagina iniziale (D-067): il vortice del portale che gira, le
//! rune che pulsano, le cascate che scendono, i riflessi del fiume, la fiamma di Fiammetto.
//!
//! Nessun video e nessun pixel nuovo: ogni effetto del layout (`layout.rs`, generato con le
//! maschere da `scripts/art_home_ambient.py`) è un nodo figlio dell'illustrazione, messo sul suo
//! rettangolo in percentuale dell'immagine, con un materiale (`home_ambient.wesl`) che ridisegna
//! solo i pixel della sua maschera a partire dall'illustrazione stessa. Così segue la misura e il
//! taglio del fondo, sta sotto logo e pannello e nasce e muore con l'illustrazione (niente
//! entità o timer in più rientrando nel menu). Un effetto compare solo quando le sue immagini
//! sono caricate e l'illustrazione è quella del layout: altrimenti resta il fondo fermo.
//! Si spengono tutti da Opzioni → Animazioni (salvato nel profilo) e uno per uno con
//! `--ambient`.

mod layout;
mod material;

pub use layout::{AmbientEffect, AmbientKind, AmbientLayout};
pub use material::{AmbientMaterial, AmbientParams, drawn_rect, node_uv, phases, rates};

use bevy::asset::embedded_asset;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::home::BackdropImage;
use crate::app::AppState;
use crate::options::LaunchOptions;
use crate::profile::Profile;

/// Secondi in cui un effetto appena comparso passa dal fondo fermo all'animazione.
pub const FADE_IN_SECS: f64 = 0.8;

/// Quali animazioni si vedono.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct HomeAmbient {
    /// Interruttore generale (Opzioni → Animazioni).
    pub enabled: bool,
    /// Solo questi effetti, per nome del layout (`None` = tutti; `--ambient`).
    pub only: Option<Vec<String>>,
}

impl Default for HomeAmbient {
    fn default() -> Self {
        Self {
            enabled: true,
            only: None,
        }
    }
}

impl HomeAmbient {
    pub fn shows(&self, name: &str) -> bool {
        self.enabled
            && self
                .only
                .as_ref()
                .is_none_or(|o| o.iter().any(|n| n == name))
    }
}

/// Il layout incorporato, letto una volta.
#[derive(Resource, Debug, Clone, Deref)]
pub struct Layout(pub AmbientLayout);

/// Nodo di un effetto: `index` nel layout, `since` = secondi reali in cui è comparso.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AmbientNode {
    pub index: usize,
    pub since: Option<f64>,
}

/// Tutto, render compreso.
pub struct HomeAmbientPlugin;

impl Plugin for HomeAmbientPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "home_ambient.wesl");
        app.add_plugins((
            UiMaterialPlugin::<AmbientMaterial>::default(),
            HomeAmbientLogicPlugin,
        ));
    }
}

/// Solo la logica, senza render: la usano i test senza finestra (con
/// `init_asset::<AmbientMaterial>()`).
pub struct HomeAmbientLogicPlugin;

impl Plugin for HomeAmbientLogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Layout(AmbientLayout::embedded()))
            .init_resource::<HomeAmbient>()
            .add_systems(Startup, read_settings)
            .add_observer(spawn_effects)
            .add_systems(
                PostUpdate,
                (show_effects, animate_effects)
                    .chain()
                    .after(bevy::ui::UiSystems::Layout)
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

/// Interruttore dal profilo, effetti scelti da riga di comando.
fn read_settings(
    mut settings: ResMut<HomeAmbient>,
    layout: Res<Layout>,
    profile: Option<Res<Profile>>,
    options: Option<Res<LaunchOptions>>,
) {
    if let Some(profile) = profile {
        settings.enabled = !profile.data.still_home;
    }
    if let Some(only) = options.and_then(|o| o.ambient.clone()) {
        let names: Vec<_> = layout.effects.iter().map(|e| e.name.as_str()).collect();
        for unknown in only.iter().filter(|n| !names.contains(&n.as_str())) {
            warn!(
                "--ambient: nessuna animazione '{unknown}' (ci sono: {})",
                names.join(", ")
            );
        }
        settings.only = Some(only);
    }
}

fn load_linear(assets: &AssetServer, path: &str) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false;
            settings.sampler = ImageSampler::linear();
        })
        .load(path.to_string())
}

fn load_color(assets: &AssetServer, path: &str) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::linear();
        })
        .load(path.to_string())
}

/// Un nodo per effetto, figlio dell'illustrazione appena nata (nascosto finché non è pronto).
fn spawn_effects(
    add: On<Add<BackdropImage>>,
    pictures: Query<&ImageNode>,
    layout: Res<Layout>,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<AmbientMaterial>>,
    mut commands: Commands,
) {
    let Ok(picture) = pictures.get(add.entity) else {
        return;
    };
    let image = layout.image_size();
    for (index, effect) in layout.effects.iter().enumerate() {
        let (at, size) = effect.fraction(image);
        let material = materials.add(AmbientMaterial {
            params: AmbientParams::new(effect, image),
            art: picture.image.clone(),
            mask: load_linear(&assets, &effect.mask),
            plate: effect
                .plate
                .as_deref()
                .map_or_else(|| picture.image.clone(), |p| load_color(&assets, p)),
        });
        commands.spawn((
            Name::new(format!("Animazione {}", effect.name)),
            AmbientNode { index, since: None },
            MaterialNode(material),
            Node {
                position_type: PositionType::Absolute,
                left: percent(at.x * 100.0),
                top: percent(at.y * 100.0),
                width: percent(size.x * 100.0),
                height: percent(size.y * 100.0),
                ..default()
            },
            Visibility::Hidden,
            // non deve mai prendere i clic destinati al menu
            Pickable::IGNORE,
            ChildOf(add.entity),
        ));
    }
}

/// Pronto = maschera (e vortice) caricati, illustrazione della misura del layout, maschera della
/// misura del rettangolo. Visibile = pronto e acceso.
fn show_effects(
    time: Res<Time<Real>>,
    settings: Res<HomeAmbient>,
    layout: Res<Layout>,
    images: Res<Assets<Image>>,
    materials: Res<Assets<AmbientMaterial>>,
    mut nodes: Query<(
        &mut AmbientNode,
        &MaterialNode<AmbientMaterial>,
        &mut Visibility,
    )>,
) {
    for (mut node, handle, mut visibility) in &mut nodes {
        let effect = &layout.effects[node.index];
        let ready = materials
            .get(handle)
            .is_some_and(|m| ready(effect, layout.image, m, &images));
        let shown = ready && settings.shows(&effect.name);
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let since = shown.then(|| node.since.unwrap_or(time.elapsed_secs_f64()));
        if node.since != since {
            node.since = since;
        }
    }
}

fn ready(
    effect: &AmbientEffect,
    image: (u32, u32),
    m: &AmbientMaterial,
    images: &Assets<Image>,
) -> bool {
    let size = |handle: &Handle<Image>| images.get(handle).map(|i| (i.width(), i.height()));
    let rect = (effect.rect.2, effect.rect.3);
    size(&m.art) == Some(image)
        && size(&m.mask) == Some(rect)
        && (effect.plate.is_none() || size(&m.plate) == Some(rect))
}

/// Fasi del tempo, dissolvenza d'ingresso e posizione vera del nodo sull'illustrazione.
fn animate_effects(
    time: Res<Time<Real>>,
    layout: Res<Layout>,
    pictures: Query<(&ComputedNode, &UiGlobalTransform), With<BackdropImage>>,
    nodes: Query<(
        &AmbientNode,
        &MaterialNode<AmbientMaterial>,
        &ComputedNode,
        &UiGlobalTransform,
        &ChildOf,
    )>,
    mut materials: ResMut<Assets<AmbientMaterial>>,
) {
    let now = time.elapsed_secs_f64();
    for (node, handle, computed, transform, child_of) in &nodes {
        let Some(since) = node.since else {
            continue;
        };
        let Ok((picture, picture_at)) = pictures.get(child_of.parent()) else {
            continue;
        };
        let drawn = drawn_rect(rect(picture, picture_at), layout.image_size());
        let Some(uv) = node_uv(rect(computed, transform), drawn) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(handle) else {
            continue;
        };
        let seconds = now - since;
        let params = &mut material.params;
        params.node_uv = uv;
        params.phases = phases(layout.effects[node.index].kind, seconds);
        params.image.w = (seconds / FADE_IN_SECS).clamp(0.0, 1.0) as f32;
    }
}

/// Rettangolo del nodo in pixel fisici (il centro è la traslazione della trasformata).
fn rect(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    Rect::from_center_size(transform.translation, node.size())
}

#[cfg(test)]
mod tests;
