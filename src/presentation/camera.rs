//! Camera pixel-perfect: il mondo è disegnato su una texture a bassa risoluzione
//! (`WorldCamera`), poi la texture è mostrata a schermo con scala intera e filtro nearest
//! (`ScreenCamera`). La scala si sceglie perché la vista minima 480×270 ci stia; se la finestra
//! non è un multiplo esatto la texture cresce fino a coprirla, fino a 640×360 pixel di mondo
//! (D-062): più mappa invece delle bande nere, che restano solo oltre quel limite. La UI usa la
//! `ScreenCamera`.

use bevy::camera::RenderTarget;
use bevy::camera::Viewport;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::PrimaryWindow;

use super::icons::ui_scale;
use super::sprites::SyncTransforms;
use crate::gameplay::player::Player;
use crate::world::view::{
    MAX_VIEW_HEIGHT, MAX_VIEW_WIDTH, VIEW_HEIGHT as VIRTUAL_HEIGHT, VIEW_WIDTH as VIRTUAL_WIDTH,
    view_center,
};

/// Il mondo (layer 0, default di tutte le entità) va sulla texture a bassa risoluzione.
const WORLD_LAYER: RenderLayers = RenderLayers::layer(0);
/// La texture e la UI vanno a schermo.
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

/// Fuori dall'arena: lo stesso verde della siepe, così il bordo non "finisce nel vuoto".
const WORLD_CLEAR: Color = Color::srgb_u8(0x2c, 0x6b, 0x3a);

#[derive(Component)]
pub struct WorldCamera;

#[derive(Component)]
pub struct ScreenCamera;

/// Lo sprite che mostra a schermo la texture del mondo.
#[derive(Component)]
struct Canvas;

/// Metà della parte di mondo davvero a schermo (pixel di mondo), per gli indicatori ai bordi.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct VisibleArea {
    pub half: Vec2,
}

impl Default for VisibleArea {
    fn default() -> Self {
        Self {
            half: Vec2::new(VIRTUAL_WIDTH as f32, VIRTUAL_HEIGHT as f32) / 2.0,
        }
    }
}

/// Il rettangolo di mondo a schermo con il player in `player` (stesso centro della camera).
pub fn visible_rect(player: Vec2, area: &VisibleArea) -> Rect {
    Rect::from_center_half_size(view_center(player), area.half)
}

/// Misura della texture del mondo per una finestra di `physical` pixel a scala `scale`: quanto
/// basta a coprirla (per eccesso, pari, così il centro resta sul pixel), tra la vista minima e
/// la più grande. In riduzione (finestre minuscole) la vista minima.
pub fn canvas_texels(physical: UVec2, scale: f32) -> UVec2 {
    let min = UVec2::new(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
    if scale < 1.0 {
        return min;
    }
    let cover = |pixels: u32| {
        let texels = (pixels as f32 / scale).ceil() as u32;
        texels + texels % 2
    };
    UVec2::new(cover(physical.x), cover(physical.y))
        .clamp(min, UVec2::new(MAX_VIEW_WIDTH, MAX_VIEW_HEIGHT))
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<VisibleArea>()
            .add_systems(Startup, spawn_cameras)
            .add_systems(
                PostUpdate,
                (follow_player.after(SyncTransforms), fit_canvas_to_window)
                    .before(TransformSystems::Propagate),
            );
    }
}

/// Fattore di ingrandimento del canvas per una finestra di `physical` pixel: intero quando
/// il canvas ci sta almeno una volta, frazionario (riduzione) solo in finestre minuscole.
pub fn canvas_scale(physical: UVec2) -> Option<f32> {
    let fit =
        (physical.x as f32 / VIRTUAL_WIDTH as f32).min(physical.y as f32 / VIRTUAL_HEIGHT as f32);
    if fit <= 0.0 {
        None // finestra minimizzata
    } else if fit >= 1.0 {
        Some(fit.floor())
    } else {
        Some(fit)
    }
}

/// Spostamento del canvas (unità della `ScreenCamera`) che allinea i bordi ai pixel fisici.
/// Con una finestra larga o alta un numero dispari di pixel il centro cade a metà pixel e,
/// senza questa correzione, il nearest-neighbor duplicherebbe righe o colonne a caso.
pub fn canvas_offset(physical: UVec2, scale: f32) -> Vec2 {
    if scale < 1.0 || scale.fract() != 0.0 {
        return Vec2::ZERO; // in riduzione l'allineamento esatto non è possibile
    }
    // A scala intera `n` un pixel fisico vale 1/n unità: mezzo pixel = 0.5/n.
    let half_pixel = 0.5 / scale;
    let odd = |pixels: u32| if pixels % 2 == 1 { half_pixel } else { 0.0 };
    Vec2::new(odd(physical.x), odd(physical.y))
}

fn spawn_cameras(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // La texture nasce alla misura più grande e non cambia mai: la camera del mondo disegna solo
    // nell'angolo che serve (viewport) e lo sprite mostra solo quello (`fit_canvas_to_window`).
    // RGBA e senza pixel iniziali: sul web (WebGL2) il caricamento degli zeri in BGRA veniva
    // rifiutato ("texSubImage2D: invalid format") e ridimensionare la texture bloccava il disegno.
    let mut canvas = Image::new_target_texture(
        MAX_VIEW_WIDTH,
        MAX_VIEW_HEIGHT,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    canvas.data = None;
    let canvas = images.add(canvas);

    commands.spawn((
        Name::new("WorldCamera"),
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(WORLD_CLEAR),
            ..default()
        },
        RenderTarget::Image(canvas.clone().into()),
        Msaa::Off,
        WorldCamera,
        WORLD_LAYER,
    ));
    commands.spawn((
        Name::new("Canvas"),
        Canvas,
        Sprite::from_image(canvas),
        SCREEN_LAYER,
    ));
    commands.spawn((
        Name::new("ScreenCamera"),
        Camera2d,
        Msaa::Off,
        ScreenCamera,
        IsDefaultUiCamera,
        SCREEN_LAYER,
    ));
}

/// La camera segue il player a pixel interi (niente sub-pixel, niente pixel che "nuotano").
fn follow_player(
    player: Single<&Transform, (With<Player>, Without<WorldCamera>)>,
    mut camera: Single<&mut Transform, With<WorldCamera>>,
) {
    let center = view_center(player.translation.truncate()).round();
    camera.translation.x = center.x;
    camera.translation.y = center.y;
}

fn fit_canvas_to_window(
    window: Single<&Window, With<PrimaryWindow>>,
    mut projection: Single<&mut Projection, With<ScreenCamera>>,
    canvas: Single<(&mut Transform, &mut Sprite), With<Canvas>>,
    mut world: Single<&mut Camera, With<WorldCamera>>,
    mut interface: ResMut<UiScale>,
    mut visible: ResMut<VisibleArea>,
) {
    let (mut canvas, mut sprite) = canvas.into_inner();
    let Projection::Orthographic(ortho) = &mut **projection else {
        return;
    };
    let physical = window.physical_size();
    let Some(scale) = canvas_scale(physical) else {
        return;
    };
    // L'ortografica lavora in pixel logici: con DPI 125% servono 1.25/scala unità per pixel
    // logico perché ogni texel copra esattamente `scala` pixel fisici.
    let target = window.scale_factor() / scale;
    if (ortho.scale - target).abs() > f32::EPSILON {
        ortho.scale = target;
    }
    // l'interfaccia segue la stessa scala intera del canvas (D-045)
    let wanted = ui_scale(scale, window.scale_factor());
    if (interface.0 - wanted).abs() > f32::EPSILON {
        interface.0 = wanted;
    }
    // la texture copre la finestra (fino alla vista più grande): più mappa, niente bande
    let texels = canvas_texels(physical, scale);
    let half = texels.as_vec2() / 2.0;
    if visible.half != half {
        visible.half = half;
    }
    // solo l'angolo della texture che serve: la camera ci disegna, lo sprite lo mostra
    let viewport = Viewport {
        physical_position: UVec2::ZERO,
        physical_size: texels,
        ..default()
    };
    if world.viewport.as_ref().map(|v| v.physical_size) != Some(texels) {
        world.viewport = Some(viewport);
    }
    let shown = Some(Rect::new(0.0, 0.0, texels.x as f32, texels.y as f32));
    if sprite.rect != shown {
        sprite.rect = shown;
    }
    let offset = canvas_offset(physical, scale);
    if canvas.translation.truncate() != offset {
        canvas.translation.x = offset.x;
        canvas.translation.y = offset.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_for_common_resolutions() {
        assert_eq!(canvas_scale(UVec2::new(1920, 1080)), Some(4.0));
        assert_eq!(canvas_scale(UVec2::new(960, 540)), Some(2.0));
        // portatile 1920×1080 al 125% con finestra quasi massimizzata
        assert_eq!(canvas_scale(UVec2::new(1536, 864)), Some(3.0));
        assert_eq!(canvas_scale(UVec2::new(2560, 1080)), Some(4.0));
        assert_eq!(canvas_scale(UVec2::new(1000, 300)), Some(1.0));
    }

    #[test]
    fn odd_window_sizes_shift_canvas_by_half_a_physical_pixel() {
        assert_eq!(canvas_offset(UVec2::new(1366, 768), 2.0), Vec2::ZERO);
        assert_eq!(
            canvas_offset(UVec2::new(1365, 768), 2.0),
            Vec2::new(0.25, 0.0)
        );
        assert_eq!(
            canvas_offset(UVec2::new(1365, 767), 3.0),
            Vec2::splat(0.5 / 3.0)
        );
        assert_eq!(canvas_offset(UVec2::new(241, 135), 0.5), Vec2::ZERO);
    }

    #[test]
    fn the_canvas_grows_to_cover_the_window_up_to_the_largest_view() {
        // multipli esatti: la vista minima
        assert_eq!(
            canvas_texels(UVec2::new(960, 540), 2.0),
            UVec2::new(480, 270)
        );
        assert_eq!(
            canvas_texels(UVec2::new(1920, 1080), 4.0),
            UVec2::new(480, 270)
        );
        // 1280×720 a ×2: tutta la finestra, 640×360
        assert_eq!(
            canvas_texels(UVec2::new(1280, 720), 2.0),
            UVec2::new(640, 360)
        );
        // 1366×768 a ×2: oltre il limite, restano due bande sottili
        assert_eq!(
            canvas_texels(UVec2::new(1366, 768), 2.0),
            UVec2::new(640, 360)
        );
        // 1536×864 a ×3 e dispari per eccesso: pari, il centro resta sul pixel
        assert_eq!(
            canvas_texels(UVec2::new(1536, 864), 3.0),
            UVec2::new(512, 288)
        );
        assert_eq!(
            canvas_texels(UVec2::new(1601, 901), 3.0),
            UVec2::new(534, 302)
        );
        // finestre minuscole: la vista minima ridotta
        assert_eq!(
            canvas_texels(UVec2::new(240, 135), 0.5),
            UVec2::new(480, 270)
        );
    }

    #[test]
    fn tiny_or_minimized_windows() {
        assert_eq!(canvas_scale(UVec2::new(240, 135)), Some(0.5));
        assert_eq!(canvas_scale(UVec2::new(0, 0)), None);
    }
}
