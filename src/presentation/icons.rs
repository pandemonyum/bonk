//! Interfaccia alla stessa scala intera del canvas. L'HUD resta a risoluzione nativa (D-013), ma
//! la camera imposta `UiScale` a "pixel logici per texel del canvas / 2": tutta l'interfaccia è
//! disegnata per la finestra di default (960×540, canvas ×2, cioè 2 unità per texel) e a ogni
//! finestra testi, pannelli e icone crescono insieme al mondo, con lo stesso pixel (D-045).
//! Un'icona di `assets/ui/` di 16 texel misura quindi sempre 32 unità dell'interfaccia.

use bevy::prelude::*;

use crate::data::schema::Element;

/// Unità dell'interfaccia (`px`) per texel del canvas.
pub const UI_PX_PER_TEXEL: f32 = 2.0;

/// `UiScale` per un canvas ingrandito `canvas_scale` volte in una finestra con fattore di scala
/// `scale_factor` (DPI): ogni texel delle icone copre gli stessi pixel fisici del mondo. Sotto
/// la scala 1 (finestre minuscole) l'interfaccia non si riduce oltre.
pub fn ui_scale(canvas_scale: f32, scale_factor: f32) -> f32 {
    canvas_scale.max(1.0) / scale_factor / UI_PX_PER_TEXEL
}

/// Immagine dell'interfaccia di `width` × `height` texel.
pub fn ui_icon(assets: &AssetServer, path: &str, width: f32, height: f32) -> impl Bundle {
    (
        ImageNode::new(assets.load(path.to_string())),
        Node {
            width: px(width * UI_PX_PER_TEXEL),
            height: px(height * UI_PX_PER_TEXEL),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// Icona dell'elemento di un compagno.
pub fn element_icon_path(element: Element) -> &'static str {
    match element {
        Element::Fire => "ui/icon_fire.png",
        Element::Water => "ui/icon_water.png",
        Element::Nature => "ui/icon_nature.png",
        Element::Electric => "ui/icon_electric.png",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_keeps_the_canvas_pixel() {
        // finestra di default: canvas ×2 a DPI 100% → nessuna scala
        assert_eq!(ui_scale(2.0, 1.0), 1.0);
        // 1920×1080: canvas ×4, interfaccia doppia
        assert_eq!(ui_scale(4.0, 1.0), 2.0);
        // 960×540 logici a DPI 150%: canvas ×3 su 1440×810 fisici, stessa interfaccia
        assert_eq!(ui_scale(3.0, 1.5), 1.0);
        // un texel di icona copre sempre `scala del canvas` pixel fisici
        for (canvas, dpi) in [(2.0, 1.25), (3.0, 1.0), (4.0, 2.0)] {
            let physical = UI_PX_PER_TEXEL * ui_scale(canvas, dpi) * dpi;
            assert!((physical - canvas).abs() < 1e-6, "{canvas} {dpi}");
        }
        // finestre minuscole: come a scala 1
        assert_eq!(ui_scale(0.5, 1.0), 0.5);
    }
}
