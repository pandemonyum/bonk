//! Icone dell'interfaccia (`assets/ui/`) alla stessa scala intera del canvas: l'HUD resta a
//! risoluzione nativa (D-013), quindi un'icona di 16 texel occupa 16 × scala pixel fisici, come
//! uno sprite del mondo, e con il sampler nearest resta nitida. La scala la aggiorna la camera
//! (`PixelScale`), le icone si ridimensionano da sole.

use bevy::prelude::*;

use crate::data::schema::Element;

/// Pixel logici dell'interfaccia per un texel del canvas: scala intera del canvas divisa per il
/// fattore di scala della finestra (DPI).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct PixelScale(pub f32);

impl Default for PixelScale {
    /// Finestra di default 960×540 a DPI 100%: canvas ×2.
    fn default() -> Self {
        Self(2.0)
    }
}

/// Icona disegnata a `width` × `height` texel.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PixelIcon {
    pub width: f32,
    pub height: f32,
}

pub struct IconsPlugin;

impl Plugin for IconsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PixelScale>()
            .add_systems(PostUpdate, size_icons);
    }
}

/// Immagine dell'interfaccia di `width` × `height` texel (la dimensione vera la mette
/// `size_icons` appena nasce e a ogni cambio di scala).
pub fn ui_icon(assets: &AssetServer, path: &str, width: f32, height: f32) -> impl Bundle {
    (
        ImageNode::new(assets.load(path.to_string())),
        Node {
            width: px(width * PixelScale::default().0),
            height: px(height * PixelScale::default().0),
            flex_shrink: 0.0,
            ..default()
        },
        PixelIcon { width, height },
    )
}

/// Icona dell'elemento di un compagno.
pub fn element_icon_path(element: Element) -> &'static str {
    match element {
        Element::Fire => "ui/icon_fire.png",
        Element::Water => "ui/icon_water.png",
        Element::Nature => "ui/icon_nature.png",
    }
}

fn size_icons(scale: Res<PixelScale>, mut icons: Query<(Ref<PixelIcon>, &mut Node)>) {
    for (icon, mut node) in &mut icons {
        if scale.is_changed() || icon.is_added() {
            node.width = px(icon.width * scale.0);
            node.height = px(icon.height * scale.0);
        }
    }
}
