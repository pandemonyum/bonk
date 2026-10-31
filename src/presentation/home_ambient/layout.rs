//! Layout delle animazioni ambientali (`assets/ui/home_ambient.ron`, generato da
//! `scripts/art_home_ambient.py` insieme alle maschere): quali effetti ci sono, dove stanno
//! nell'illustrazione e con quali parametri. Incorporato nel binario come i dati di gioco: le
//! maschere invece arrivano dall'AssetServer e, se mancano, l'effetto non compare.

use bevy::math::{URect, UVec2, Vec2, Vec4};
use serde::Deserialize;

pub const LAYOUT_TEXT: &str = include_str!("../../../assets/ui/home_ambient.ron");

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AmbientLayout {
    /// Misura dell'illustrazione per cui sono stati fatti rettangoli e maschere.
    pub image: (u32, u32),
    pub effects: Vec<AmbientEffect>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AmbientEffect {
    /// Nome per accenderlo o spegnerlo da solo (`--ambient`).
    pub name: String,
    /// Maschera in `assets/`, grande quanto `rect`.
    pub mask: String,
    /// x, y, larghezza, altezza in pixel dell'illustrazione.
    pub rect: (u32, u32, u32, u32),
    /// Pixel da far muovere che l'illustrazione non ha (il vortice dietro le rovine), grande
    /// quanto `rect`; senza, lo shader rilegge l'illustrazione.
    #[serde(default)]
    pub plate: Option<String>,
    pub kind: AmbientKind,
}

/// Tipi di effetto; i canali delle maschere sono scritti in `scripts/art_home_ambient.py`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum AmbientKind {
    /// Vortice che ruota attorno a `center` dentro il superellisse `radii`/`squareness` (nel
    /// verso `spin`, 1 o -1), rune che pulsano, alone e particelle.
    Portal {
        center: (f32, f32),
        radii: (f32, f32),
        squareness: f32,
        spin: f32,
    },
    /// Acqua che scende da `lip` a `base` a `speed` texel/s, schiuma e cerchi nella pozza
    /// `pool` (cx, cy, rx, ry; rx 0 = nessuna).
    Waterfall {
        lip: f32,
        base: f32,
        speed: f32,
        pool: (f32, f32, f32, f32),
    },
    /// Riflessi e piccoli movimenti della superficie.
    River,
    /// Fiamma che guizza verso l'alto, di più quanto più è sopra la riga `base`.
    Flame { base: f32 },
}

impl AmbientKind {
    /// Codice del tipo per lo shader (`home_ambient.wesl`).
    pub fn code(self) -> f32 {
        match self {
            Self::Portal { .. } => 0.0,
            Self::Waterfall { .. } => 1.0,
            Self::River => 2.0,
            Self::Flame { .. } => 3.0,
        }
    }

    /// Parametri di forma per lo shader, in texel dell'illustrazione.
    pub fn shape(self) -> [Vec4; 2] {
        match self {
            Self::Portal {
                center,
                radii,
                squareness,
                spin,
            } => [
                Vec4::new(center.0, center.1, radii.0, radii.1),
                Vec4::new(squareness, spin, 0.0, 0.0),
            ],
            Self::Waterfall {
                lip,
                base,
                speed,
                pool,
            } => [
                Vec4::new(lip, base, speed, 0.0),
                Vec4::new(pool.0, pool.1, pool.2, pool.3),
            ],
            Self::River => [Vec4::ZERO; 2],
            Self::Flame { base } => [Vec4::new(0.0, base, 0.0, 0.0), Vec4::ZERO],
        }
    }
}

impl AmbientEffect {
    pub fn rect(&self) -> URect {
        let (x, y, w, h) = self.rect;
        URect::from_corners(UVec2::new(x, y), UVec2::new(x + w, y + h))
    }

    /// Angolo e misura del rettangolo in frazioni dell'illustrazione (0..1).
    pub fn fraction(&self, image: Vec2) -> (Vec2, Vec2) {
        let rect = self.rect().as_rect();
        (rect.min / image, rect.size() / image)
    }
}

impl AmbientLayout {
    pub fn parse(text: &str) -> Result<Self, String> {
        let layout: Self = ron::from_str(text).map_err(|e| format!("home_ambient.ron: {e}"))?;
        layout.validate()?;
        Ok(layout)
    }

    /// Il layout incorporato (sempre valido: lo controllano i test).
    pub fn embedded() -> Self {
        Self::parse(LAYOUT_TEXT).unwrap_or_else(|e| {
            bevy::log::warn!("animazioni della home spente: {e}");
            Self {
                image: (0, 0),
                effects: Vec::new(),
            }
        })
    }

    pub fn image_size(&self) -> Vec2 {
        Vec2::new(self.image.0 as f32, self.image.1 as f32)
    }

    fn validate(&self) -> Result<(), String> {
        let image = URect::new(0, 0, self.image.0, self.image.1);
        let mut names = Vec::new();
        for effect in &self.effects {
            let rect = effect.rect();
            if rect.is_empty() || image.union(rect) != image {
                return Err(format!(
                    "{}: rettangolo {:?} fuori dall'illustrazione",
                    effect.name, effect.rect
                ));
            }
            if names.contains(&effect.name.as_str()) {
                return Err(format!("{}: nome ripetuto", effect.name));
            }
            names.push(effect.name.as_str());
        }
        Ok(())
    }
}
