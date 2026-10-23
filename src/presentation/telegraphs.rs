//! Segni di pericolo a terra: punti rossi in fila (la carica dell'élite) o in cerchio (il
//! colpo del boss), figli dell'entità `Telegraph` e allineati al pixel come le gocce dell'onda.
//! Lampeggiano sempre più in fretta verso lo scadere: "sta per colpire".

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Z_PICKUPS;
use super::sprites::DepthBase;
use crate::gameplay::threats::{Telegraph, TelegraphShape};

/// Distanza tra due punti del segno.
const DOT_SPACING: f32 = 6.0;
const DOT_SPRITE: &str = "sprites/telegraph.png";
/// Periodo del lampeggio all'inizio e alla fine del preavviso.
const BLINK_SLOW_S: f32 = 0.24;
const BLINK_FAST_S: f32 = 0.06;
/// Opacità dei punti nella fase "spenta" del lampeggio.
const DIM_ALPHA: f32 = 0.35;

#[derive(Component)]
struct TelegraphDot;

pub struct TelegraphArtPlugin;

impl Plugin for TelegraphArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_dots)
            .add_systems(Update, blink_dots);
    }
}

/// Posizioni dei punti rispetto all'entità: lungo la linea (escluso il punto di partenza, dove
/// sta il nemico) o sul cerchio.
pub fn dot_offsets(shape: &TelegraphShape) -> Vec<Vec2> {
    match *shape {
        TelegraphShape::Line { direction, length } => {
            let count = (length / DOT_SPACING).floor() as u32;
            (1..=count)
                .map(|i| direction * DOT_SPACING * i as f32)
                .collect()
        }
        TelegraphShape::Ring { radius } => {
            let count = ((TAU * radius / DOT_SPACING).round() as u32).clamp(8, 64);
            (0..count)
                .map(|i| Vec2::from_angle(TAU * i as f32 / count as f32) * radius)
                .collect()
        }
    }
}

/// Acceso o attenuato: il periodo si accorcia da `BLINK_SLOW_S` a `BLINK_FAST_S` man mano che
/// il preavviso scade.
pub fn blink_alpha(remaining: f32, total: f32) -> f32 {
    let progress = if total > 0.0 {
        (1.0 - remaining / total).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let period = BLINK_SLOW_S + (BLINK_FAST_S - BLINK_SLOW_S) * progress;
    let elapsed = (total - remaining).max(0.0);
    if ((elapsed / period) as u32).is_multiple_of(2) {
        1.0
    } else {
        DIM_ALPHA
    }
}

fn attach_dots(
    add: On<Add<Telegraph>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    telegraphs: Query<&Telegraph>,
) {
    let Ok(telegraph) = telegraphs.get(add.entity) else {
        return;
    };
    let image = assets.load(DOT_SPRITE);
    let offsets = dot_offsets(&telegraph.shape);
    commands
        .entity(add.entity)
        .insert((
            Transform::default(),
            Visibility::default(),
            DepthBase(Z_PICKUPS),
        ))
        .with_children(|dots| {
            for offset in offsets {
                dots.spawn((
                    TelegraphDot,
                    Sprite::from_image(image.clone()),
                    Transform::from_translation(offset.round().extend(0.0)),
                ));
            }
        });
}

fn blink_dots(
    telegraphs: Query<(&Telegraph, &Children)>,
    mut dots: Query<&mut Sprite, With<TelegraphDot>>,
) {
    for (telegraph, children) in &telegraphs {
        let alpha = blink_alpha(telegraph.remaining, telegraph.total);
        for child in children.iter() {
            if let Ok(mut sprite) = dots.get_mut(child)
                && sprite.color.alpha() != alpha
            {
                sprite.color.set_alpha(alpha);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_and_rings_are_drawn_with_evenly_spaced_dots() {
        let line = dot_offsets(&TelegraphShape::Line {
            direction: Vec2::X,
            length: 30.0,
        });
        assert_eq!(line.len(), 5);
        assert_eq!(line[0], Vec2::new(DOT_SPACING, 0.0));
        let ring = dot_offsets(&TelegraphShape::Ring { radius: 44.0 });
        assert_eq!(ring.len(), 46);
        assert!(ring.iter().all(|p| (p.length() - 44.0).abs() < 1e-3));
    }

    #[test]
    fn the_blink_speeds_up_towards_the_end() {
        // all'inizio acceso, poi alterna; verso la fine cambia più spesso
        assert_eq!(blink_alpha(1.0, 1.0), 1.0);
        let changes = |from: f32, to: f32| {
            let mut last = blink_alpha(from, 1.0);
            let mut n = 0;
            let mut r = from;
            while r > to {
                r -= 0.005;
                let a = blink_alpha(r, 1.0);
                n += u32::from(a != last);
                last = a;
            }
            n
        };
        assert!(changes(0.3, 0.0) > changes(1.0, 0.7));
    }
}
