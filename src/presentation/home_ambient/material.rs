//! Materiale dei nodi animati: l'illustrazione, la maschera dell'effetto e (per il portale) il
//! vortice completo da far ruotare; tempi e posizione arrivano come uniform a ogni frame.

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::layout::{AmbientEffect, AmbientKind};

pub const SHADER: &str =
    "embedded://pixel_monster_survivor/presentation/home_ambient/home_ambient.wesl";

/// Texel di cui scorrono l'acqua e le sue strisce in un giro della fase (allo shader in
/// `shape.w` delle cascate).
pub const FLOW_PERIOD: f32 = 16.0;

/// Uniform dello shader: solo `Vec4`, allineati a 16 byte come vuole WebGL2.
#[derive(ShaderType, Debug, Clone, Copy, Default, PartialEq)]
pub struct AmbientParams {
    /// xy: angolo in alto a sinistra del nodo nell'illustrazione, zw: sua misura (uv 0..1).
    pub node_uv: Vec4,
    /// xy: misura dell'illustrazione in texel; z: tipo (`AmbientKind::code`); w: intensità.
    pub image: Vec4,
    /// Rettangolo della maschera in texel (x, y, larghezza, altezza).
    pub mask_rect: Vec4,
    /// Fasi 0..1 delle animazioni del tipo (`phases`).
    pub phases: Vec4,
    /// Parametri di forma (`AmbientKind::shape`).
    pub shape: Vec4,
    pub shape2: Vec4,
}

impl AmbientParams {
    pub fn new(effect: &AmbientEffect, image: Vec2) -> Self {
        let rect = effect.rect().as_rect();
        let (at, size) = effect.fraction(image);
        let [mut shape, shape2] = effect.kind.shape();
        if matches!(effect.kind, AmbientKind::Waterfall { .. }) {
            shape.w = FLOW_PERIOD;
        }
        Self {
            node_uv: Vec4::new(at.x, at.y, size.x, size.y),
            image: Vec4::new(image.x, image.y, effect.kind.code(), 0.0),
            mask_rect: Vec4::new(rect.min.x, rect.min.y, rect.width(), rect.height()),
            phases: Vec4::ZERO,
            shape,
            shape2,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AmbientMaterial {
    #[uniform(0)]
    pub params: AmbientParams,
    /// L'illustrazione della home (stessa immagine e stesso campionatore del fondo).
    #[texture(1)]
    #[sampler(2)]
    pub art: Handle<Image>,
    /// Maschera dell'effetto, lineare (non sRGB).
    #[texture(3)]
    #[sampler(4)]
    pub mask: Handle<Image>,
    /// Il vortice completo del portale; gli altri effetti ci rimettono l'illustrazione.
    #[texture(5)]
    #[sampler(6)]
    pub plate: Handle<Image>,
}

impl UiMaterial for AmbientMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

/// Giri al secondo delle quattro fasi di ogni tipo. Lo shader usa solo funzioni periodiche di
/// queste fasi (o di loro multipli interi): il ciclo non salta mai.
pub fn rates(kind: AmbientKind) -> [f64; 4] {
    match kind {
        // giro del vortice, bracci di luce, pulsare di rune e alone, particelle
        AmbientKind::Portal { .. } => [1.0 / 14.0, 0.22, 0.2, 1.0 / 24.0],
        // acqua e strisce, schiuma, cerchi nella pozza, scintille della schiuma
        AmbientKind::Waterfall { speed, .. } => [f64::from(speed / FLOW_PERIOD), 0.3, 0.45, 0.16],
        // riflessi, ondeggiare, creste
        AmbientKind::River => [1.0 / 9.0, 0.18, 0.12, 0.0],
        // guizzi (due rumori a velocità diverse) e respiro della luce
        AmbientKind::Flame { .. } => [1.1, 1.7, 0.6, 0.0],
        // giro lento in orizzontale e in verticale (un'ellisse storta), accendersi e spegnersi
        AmbientKind::Motes { .. } => [1.0 / 11.0, 1.0 / 13.0, 1.0 / 7.0, 0.0],
    }
}

/// Le fasi dopo `seconds` secondi di animazione.
pub fn phases(kind: AmbientKind, seconds: f64) -> Vec4 {
    Vec4::from_array(rates(kind).map(|rate| (seconds * rate).fract() as f32))
}

/// Dove il fondo disegna davvero l'illustrazione dentro il suo nodo: come `NodeImageMode::Auto`
/// di Bevy, intera e senza deformarla, centrata. Con le misure arrotondate dal layout resta un
/// margine di frazioni di pixel che sfaserebbe la lettura dello shader.
pub fn drawn_rect(node: Rect, source: Vec2) -> Rect {
    if source.cmple(Vec2::ZERO).any() {
        return node;
    }
    let size = source * (node.size() / source).min_element();
    Rect::from_center_size(node.center(), size)
}

/// `node_uv` dai rettangoli veri (pixel fisici, dopo gli arrotondamenti del layout) del nodo e
/// dell'illustrazione: lo shader legge l'illustrazione nello stesso punto in cui la disegna il
/// fondo, a qualunque misura e taglio della finestra.
pub fn node_uv(node: Rect, image: Rect) -> Option<Vec4> {
    let size = image.size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }
    let at = (node.min - image.min) / size;
    let span = node.size() / size;
    Some(Vec4::new(at.x, at.y, span.x, span.y))
}
