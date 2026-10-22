//! Camera pixel-perfect: il mondo è disegnato su una texture 480×270 (`WorldCamera`), poi la
//! texture è mostrata a schermo con scala intera e filtro nearest (`ScreenCamera`), con bande
//! nere quando la finestra non è un multiplo esatto. La UI usa la `ScreenCamera`.

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::PrimaryWindow;

use super::sprites::SyncTransforms;
use crate::gameplay::player::Player;
use crate::world::view::{VIEW_HEIGHT as VIRTUAL_HEIGHT, VIEW_WIDTH as VIRTUAL_WIDTH, view_center};

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

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
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
    let canvas = Image::new_target_texture(
        VIRTUAL_WIDTH,
        VIRTUAL_HEIGHT,
        TextureFormat::Bgra8UnormSrgb,
        None,
    );
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
    mut canvas: Single<&mut Transform, With<Canvas>>,
) {
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
    fn tiny_or_minimized_windows() {
        assert_eq!(canvas_scale(UVec2::new(240, 135)), Some(0.5));
        assert_eq!(canvas_scale(UVec2::new(0, 0)), None);
    }
}
