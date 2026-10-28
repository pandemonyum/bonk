//! I colpi ad area dei talenti a schermo (`gameplay::talent_effects::Blast`), con lo sprite
//! dell'attacco del compagno che li fa (palla di fuoco, goccia, foglia, scintilla): un anello
//! che si allarga per esplosioni e colpi periodici, un ventaglio che si allunga per il soffio.
//! Nessuna arte nuova: si legge di chi è il colpo dal suo sprite.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Z_PROJECTILES;
use crate::data::GameConfig;
use crate::gameplay::talent_effects::{BLAST_S, Blast, Shape};

/// Distanza tra due particelle sull'anello o sull'arco (px).
const SPACING: f32 = 9.0;
/// Il segno parte da questa frazione della misura finale.
const START: f32 = 0.35;

/// Una particella del segno: direzione dal centro e frazione della distanza finale.
#[derive(Component, Debug, Clone, Copy)]
struct Particle {
    direction: Vec2,
    reach: f32,
}

/// Particelle di un anello di raggio `radius` (almeno 6), equidistanti.
pub fn ring_directions(radius: f32) -> Vec<Vec2> {
    let count = ((TAU * radius / SPACING).round() as usize).max(6);
    (0..count)
        .map(|i| Vec2::from_angle(TAU * i as f32 / count as f32))
        .collect()
}

/// Particelle di un ventaglio: archi a metà e in fondo alla portata, più la linea di mezzo.
pub fn cone_particles(direction: Vec2, range: f32, half_angle: f32) -> Vec<(Vec2, f32)> {
    let mut out = Vec::new();
    for reach in [0.5, 1.0] {
        let arc = 2.0 * half_angle * range * reach;
        let count = ((arc / SPACING).round() as usize).max(2);
        for i in 0..=count {
            let angle = -half_angle + 2.0 * half_angle * i as f32 / count as f32;
            out.push((Vec2::from_angle(angle).rotate(direction), reach));
        }
    }
    out
}

/// Misura del segno dopo `left` secondi su `BLAST_S`: cresce in fretta, poi si ferma.
pub fn growth(left: f32) -> f32 {
    let t = (1.0 - left / BLAST_S).clamp(0.0, 1.0);
    START + (1.0 - START) * (1.0 - (1.0 - t) * (1.0 - t))
}

pub struct TalentArtPlugin;

impl Plugin for TalentArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_blast)
            .add_systems(Update, grow_blasts);
    }
}

fn attach_blast(
    add: On<Add<Blast>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    blasts: Query<&Blast>,
) {
    let Ok(blast) = blasts.get(add.entity) else {
        return;
    };
    let sprite = blast
        .source
        .and_then(|s| config.companions.get(s.0))
        .and_then(|c| config.attack(&c.attack))
        .map(|a| a.sprite().to_string());
    let Some(path) = sprite else {
        return;
    };
    let image: Handle<Image> = assets.load(path);
    let (origin, particles): (Vec2, Vec<(Vec2, f32)>) = match blast.shape {
        Shape::Circle { center, radius } => (
            center,
            ring_directions(radius)
                .into_iter()
                .map(|d| (d * radius, 1.0))
                .collect(),
        ),
        Shape::Cone {
            origin,
            direction,
            range,
            half_angle,
        } => (
            origin,
            cone_particles(direction, range, half_angle)
                .into_iter()
                .map(|(d, reach)| (d * range, reach))
                .collect(),
        ),
    };
    let at = origin.round();
    commands
        .entity(add.entity)
        .insert((
            Transform::from_translation(at.extend(Z_PROJECTILES + 0.5)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for (offset, reach) in particles {
                parent.spawn((
                    Particle {
                        direction: offset,
                        reach,
                    },
                    Sprite::from_image(image.clone()),
                    Transform::default(),
                ));
            }
        });
}

fn grow_blasts(
    blasts: Query<(&Blast, &Children)>,
    mut particles: Query<(&Particle, &mut Transform)>,
) {
    for (blast, children) in &blasts {
        let k = growth(blast.remaining);
        for child in children.iter() {
            if let Ok((particle, mut transform)) = particles.get_mut(child) {
                let at = (particle.direction * particle.reach * k).round();
                transform.translation = at.extend(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rings_and_fans_are_spaced_evenly() {
        assert_eq!(ring_directions(1.0).len(), 6, "almeno sei");
        let ring = ring_directions(30.0);
        assert!(ring.len() >= 18 && ring.len() <= 24, "{}", ring.len());
        let fan = cone_particles(Vec2::X, 72.0, 25f32.to_radians());
        assert!(
            fan.iter()
                .all(|(d, _)| d.angle_to(Vec2::X).abs() <= 25f32.to_radians() + 1e-4)
        );
        assert!(fan.iter().any(|(_, r)| *r == 1.0) && fan.iter().any(|(_, r)| *r == 0.5));
    }

    #[test]
    fn the_blast_grows_fast_then_settles() {
        assert!((growth(BLAST_S) - START).abs() < 1e-6);
        assert!((growth(0.0) - 1.0).abs() < 1e-6);
        assert!(growth(BLAST_S * 0.5) > 0.5 * (START + 1.0));
    }
}
