//! Sprite e animazioni dei compagni. Con `frame` in `companions.ron` lo sprite è un foglio di 4
//! colonne: riga 0 fermo (di fronte), 1 cammino di profilo (verso destra, specchiato verso
//! sinistra), 2 attacco (di fronte), che parte quando il compagno colpisce: la carica del suo
//! attacco riparte da capo; 3 cammino verso l'alto (di schiena), 4 cammino verso il basso (di
//! fronte). La direzione del cammino è l'asse dominante dello spostamento. Le foglie girano
//! sempre, quindi chi le ha non mostra l'attacco. Senza `frame` il compagno è un'immagine fissa.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::enemy_anim::{COLUMNS, cycle_frame, once_frame};
use super::sprites::data_sprite;
use crate::data::{DataId, GameConfig};
use crate::gameplay::attacks::AutoAttack;
use crate::gameplay::chain::ChainAttack;
use crate::gameplay::companions::Companion;
use crate::gameplay::kinematics::{Position, PreviousPosition};
use crate::gameplay::pulse::PulseAttack;

pub const ROWS: u32 = 5;
const IDLE_FPS: f32 = 4.0;
const MOVE_FPS: f32 = 10.0;
pub const ATTACK_S: f32 = 0.3;
/// Sotto questo spostamento per passo (px) il compagno è fermo.
const MIN_STEP: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompanionPose {
    #[default]
    Idle,
    /// Cammino di profilo, in su (di schiena) o in giù (di fronte).
    Side,
    Up,
    Down,
    Attack,
}

/// Stato dell'animazione di un compagno con foglio.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct CompanionAnimation {
    pub pose: CompanionPose,
    pub clock: f32,
    pub facing_left: bool,
    /// Secondi di attacco rimasti.
    pub attack_left: f32,
    /// Carica dell'attacco vista al fotogramma prima: se scende, il compagno ha colpito.
    last_charge: Option<f32>,
}

#[derive(Resource, Default)]
struct CompanionLayouts(HashMap<(u32, u32), Handle<TextureAtlasLayout>>);

pub struct CompanionAnimPlugin;

impl Plugin for CompanionAnimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanionLayouts>()
            .add_observer(attach_companion_sprite)
            .add_systems(Update, animate_companions);
    }
}

/// Un passo dell'animazione: attacco > cammino > fermo. `charge`: carica dell'attacco a
/// proiettili o a onda (nessuna per le foglie).
pub fn advance(anim: &mut CompanionAnimation, step: Vec2, charge: Option<f32>, dt: f32) {
    let previous = anim.pose;
    if let (Some(now), Some(before)) = (charge, anim.last_charge)
        && now + 1e-4 < before
    {
        anim.attack_left = ATTACK_S;
    }
    anim.last_charge = charge;
    if step.x.abs() >= MIN_STEP {
        anim.facing_left = step.x < 0.0;
    }
    anim.pose = if anim.attack_left > 0.0 {
        anim.attack_left = (anim.attack_left - dt).max(0.0);
        CompanionPose::Attack
    } else if step.length() >= MIN_STEP {
        if step.x.abs() >= step.y.abs() {
            CompanionPose::Side
        } else if step.y > 0.0 {
            CompanionPose::Up
        } else {
            CompanionPose::Down
        }
    } else {
        CompanionPose::Idle
    };
    anim.clock = if anim.pose == previous {
        anim.clock + dt
    } else {
        0.0
    };
}

/// Riga, colonna e specchio per lo stato corrente: solo il cammino si gira.
pub fn frame_of(anim: &CompanionAnimation) -> (u32, u32, bool) {
    match anim.pose {
        CompanionPose::Idle => (0, cycle_frame(anim.clock, IDLE_FPS), false),
        CompanionPose::Side => (1, cycle_frame(anim.clock, MOVE_FPS), anim.facing_left),
        CompanionPose::Attack => (2, once_frame(ATTACK_S - anim.attack_left, ATTACK_S), false),
        CompanionPose::Up => (3, cycle_frame(anim.clock, MOVE_FPS), false),
        CompanionPose::Down => (4, cycle_frame(anim.clock, MOVE_FPS), false),
    }
}

fn attach_companion_sprite(
    add: On<Add<Companion>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    ids: Query<&DataId>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut grids: ResMut<CompanionLayouts>,
) {
    let data = ids
        .get(add.entity)
        .ok()
        .and_then(|id| config.companion(&id.0));
    let path = data_sprite("compagno", ids.get(add.entity).ok(), |id| {
        config.companion(id).map(|c| c.sprite.clone())
    });
    let Some(path) = path else {
        return;
    };
    let image = assets.load(path);
    let Some(frame) = data.and_then(|d| d.frame) else {
        commands
            .entity(add.entity)
            .insert((Sprite::from_image(image), Anchor::BOTTOM_CENTER));
        return;
    };
    let layout = grids
        .0
        .entry((frame.w, frame.h))
        .or_insert_with(|| {
            layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(frame.w, frame.h),
                COLUMNS,
                ROWS,
                None,
                None,
            ))
        })
        .clone();
    commands.entity(add.entity).insert((
        Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 }),
        Anchor::BOTTOM_CENTER,
        CompanionAnimation::default(),
    ));
}

fn animate_companions(
    time: Res<Time>,
    mut companions: Query<
        (
            &Position,
            &PreviousPosition,
            Option<&AutoAttack>,
            Option<&PulseAttack>,
            Option<&ChainAttack>,
            &mut CompanionAnimation,
            &mut Sprite,
        ),
        With<Companion>,
    >,
) {
    for (position, previous, shots, wave, bolt, mut anim, mut sprite) in &mut companions {
        let charge = shots
            .map(|a| a.charge)
            .or(wave.map(|w| w.charge))
            .or(bolt.map(|c| c.charge));
        advance(
            &mut anim,
            position.0 - previous.0,
            charge,
            time.delta_secs(),
        );
        let (row, column, flip) = frame_of(&anim);
        let index = (row * COLUMNS + column) as usize;
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        if sprite.flip_x != flip {
            sprite.flip_x = flip;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn walking_turns_standing_faces_the_viewer() {
        let mut anim = CompanionAnimation::default();
        advance(&mut anim, Vec2::new(-1.0, 0.0), None, DT);
        assert_eq!(anim.pose, CompanionPose::Side);
        assert_eq!(frame_of(&anim), (1, 0, true));
        // in su di schiena, in giù di fronte, senza specchio
        advance(&mut anim, Vec2::new(0.2, 1.0), None, DT);
        assert_eq!(frame_of(&anim), (3, 0, false));
        advance(&mut anim, Vec2::new(-0.2, -1.0), None, DT);
        assert_eq!(frame_of(&anim), (4, 0, false));
        advance(&mut anim, Vec2::ZERO, None, DT);
        assert_eq!(anim.pose, CompanionPose::Idle);
        assert!(!frame_of(&anim).2, "da fermo è di fronte");
    }

    #[test]
    fn a_shot_plays_the_attack_once_then_back_to_walking() {
        let mut anim = CompanionAnimation::default();
        // la carica sale...
        advance(&mut anim, Vec2::new(1.0, 0.0), Some(0.5), DT);
        advance(&mut anim, Vec2::new(1.0, 0.0), Some(0.6), DT);
        assert_eq!(anim.pose, CompanionPose::Side);
        // ...e riparte: colpo
        advance(&mut anim, Vec2::new(1.0, 0.0), Some(0.01), DT);
        assert_eq!(anim.pose, CompanionPose::Attack);
        assert_eq!(frame_of(&anim).0, 2);
        let mut t = 0.0;
        let mut charge = 0.01;
        while anim.pose == CompanionPose::Attack {
            charge += DT;
            advance(&mut anim, Vec2::new(1.0, 0.0), Some(charge), DT);
            t += DT;
            assert!(t <= ATTACK_S + DT);
        }
        assert_eq!(anim.pose, CompanionPose::Side);
    }

    #[test]
    fn leaves_never_play_the_attack() {
        let mut anim = CompanionAnimation::default();
        for _ in 0..120 {
            advance(&mut anim, Vec2::ZERO, None, DT);
        }
        assert_eq!(anim.pose, CompanionPose::Idle);
    }

    /// Ogni compagno con un foglio nei dati ha un PNG di 4 colonne × 5 righe del suo `frame`.
    #[test]
    fn companion_sheets_match_their_frames() {
        let config = GameConfig::embedded();
        for companion in &config.companions {
            let Some(frame) = companion.frame else {
                continue;
            };
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(&companion.sprite);
            let bytes = std::fs::read(&path).expect("foglio in assets/");
            let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!(
                (width, height),
                (frame.w * COLUMNS, frame.h * ROWS),
                "{}",
                companion.id
            );
        }
    }
}
