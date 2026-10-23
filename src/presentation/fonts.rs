//! Font pixel dell'interfaccia (D-044), SIL OFL 1.1 con la licenza accanto in `assets/fonts/`:
//! Tiny5 per l'HUD (numeri ed etichette brevi), Pixelify Sans per menu, draft e risultati
//! (titoli e frasi). Chi costruisce un testo usa `hud(size)` o `ui(size)` al posto di
//! `TextFont::from_font_size(size)`: l'hook di `Typeface` mette il font giusto appena il testo
//! nasce, senza passare handle a ogni funzione dell'interfaccia.
//!
//! Dimensioni nitide: Tiny5 ha 8 pixel di disegno per em (maiuscole alte 5). L'interfaccia
//! scala con il canvas (`UiScale` = scala del canvas / 2, D-045), quindi a 16 px ogni pixel del
//! font è un texel del canvas e a 32 px due, a qualunque scala; 24 px darebbe mezzi pixel con
//! il canvas ×3. Si rasterizza senza antialiasing e con un'ombra di un pixel del font, per
//! leggerlo sopra l'erba.
//! Pixelify Sans ha circa 11 pixel per em e contorni non tutti sulla griglia: resta con
//! l'antialiasing, più pulito a 22 e 33 px.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

/// Font dell'HUD, relativo ad `assets/`.
pub const HUD_FONT: &str = "fonts/tiny5/Tiny5.ttf";
/// Font di menu, draft e risultati, relativo ad `assets/`.
pub const UI_FONT: &str = "fonts/pixelify-sans/PixelifySans.ttf";

/// Ombra dei testi Tiny5: contorno della palette v1.
const HUD_SHADOW: Color = Color::srgb_u8(0x1b, 0x14, 0x28);

/// Handle dei due font, caricati alla costruzione del plugin (prima di ogni testo).
#[derive(Resource, Debug, Clone)]
pub struct UiFonts {
    pub hud: Handle<Font>,
    pub ui: Handle<Font>,
}

impl FromWorld for UiFonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hud: assets.load(HUD_FONT),
            ui: assets.load(UI_FONT),
        }
    }
}

/// Famiglia del testo: l'hook la traduce in font e antialiasing di `TextFont`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[component(on_add = Typeface::apply)]
#[require(TextFont)]
pub enum Typeface {
    Hud,
    Ui,
}

impl Typeface {
    fn apply(mut world: DeferredWorld, context: HookContext) {
        let Some(fonts) = world.get_resource::<UiFonts>() else {
            return; // senza i font (test senza AssetServer) resta il font di default
        };
        let Some(face) = world.get::<Typeface>(context.entity).copied() else {
            return;
        };
        let (font, smoothing) = match face {
            Typeface::Hud => (fonts.hud.clone(), FontSmoothing::None),
            Typeface::Ui => (fonts.ui.clone(), FontSmoothing::AntiAliased),
        };
        if let Some(mut text_font) = world.get_mut::<TextFont>(context.entity) {
            text_font.font = font.into();
            text_font.font_smoothing = smoothing;
        }
    }
}

/// Testo dell'HUD in Tiny5 (`size` multiplo di 16 per pixel netti a ogni scala del canvas), con
/// l'ombra spostata di un pixel del font in basso a destra.
pub fn hud(size: f32) -> impl Bundle {
    (
        Typeface::Hud,
        TextFont::from_font_size(size),
        TextShadow {
            offset: Vec2::splat(size / 8.0),
            color: HUD_SHADOW,
        },
    )
}

/// Testo di menu, draft e risultati in Pixelify Sans.
pub fn ui(size: f32) -> impl Bundle {
    (Typeface::Ui, TextFont::from_font_size(size))
}

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFonts>();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy::asset::uuid_handle;
    use skrifa::{FontRef, MetadataProvider};

    use super::*;
    use crate::data::ui_drawable;

    #[test]
    fn both_fonts_draw_every_character_the_data_may_use() {
        for path in [HUD_FONT, UI_FONT] {
            let file = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(path);
            let bytes = std::fs::read(&file).expect("font in assets/fonts/");
            let font = FontRef::new(&bytes).expect("TTF leggibile");
            let charmap = font.charmap();
            let missing: String = (0..0x3000)
                .filter_map(char::from_u32)
                .filter(|c| ui_drawable(*c) && *c != '\n' && charmap.map(*c).is_none())
                .collect();
            assert!(missing.is_empty(), "{path} non disegna {missing:?}");
        }
        for c in "àèéìòùÀÈÉÌÒÙ«»×".chars() {
            assert!(ui_drawable(c), "{c} deve essere ammesso nei dati");
        }
    }

    #[test]
    fn the_typeface_picks_font_and_smoothing_when_the_text_is_born() {
        let fonts = UiFonts {
            hud: uuid_handle!("6f0d8a1e-0b1c-4e7a-9f45-1a2b3c4d5e01"),
            ui: uuid_handle!("6f0d8a1e-0b1c-4e7a-9f45-1a2b3c4d5e02"),
        };
        let mut world = World::new();
        world.insert_resource(fonts.clone());
        let hud_text = world.spawn((Text::new("LV 3"), hud(16.0))).id();
        let ui_text = world.spawn((Text::new("Città"), ui(22.0))).id();

        let hud_font = world.get::<TextFont>(hud_text).unwrap();
        assert_eq!(hud_font.font, FontSource::from(&fonts.hud));
        assert_eq!(hud_font.font_smoothing, FontSmoothing::None);
        assert_eq!(hud_font.font_size, FontSize::Px(16.0));
        // ombra di un pixel del font: 2 px a 16 px (8 pixel di disegno per em)
        let shadow = world.get::<TextShadow>(hud_text).unwrap();
        assert_eq!(shadow.offset, Vec2::splat(2.0));
        assert!(world.get::<TextShadow>(ui_text).is_none());
        let ui_font = world.get::<TextFont>(ui_text).unwrap();
        assert_eq!(ui_font.font, FontSource::from(&fonts.ui));
        assert_eq!(ui_font.font_smoothing, FontSmoothing::AntiAliased);
    }
}
