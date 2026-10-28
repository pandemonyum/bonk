//! Sprite e animazioni di nemici, élite e boss. Con `frame` in `enemies.ron` lo sprite è un
//! foglio di 4 colonne: riga 0 movimento, 1 sconfitta e, per chi carica (élite) o colpisce ad
//! area (boss), 2 preparazione e 3 attacco; disegnato verso destra, specchiato verso sinistra.
//! Alla sconfitta resta per un attimo un'entità di sola grafica che cade e svanisce dove il
//! nemico è morto. Senza `frame` il nemico è un'immagine fissa, come prima dei fogli.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::sprites::data_sprite;
use crate::app::AppState;
use crate::data::schema::EnemyData;
use crate::data::{DataId, GameConfig};
use crate::gameplay::combat::Health;
use crate::gameplay::enemies::Enemy;
use crate::gameplay::kinematics::{Position, PreviousPosition};
use crate::gameplay::threats::{ChargeState, Charger, PendingSlam, Slammer, Telegraph};

pub const COLUMNS: u32 = 4;
/// Passo dei cicli: i piccoli saltellano svelti, il boss pesta piano, lo scatto è un lampo.
const MOVE_FPS: f32 = 8.0;
const BOSS_MOVE_FPS: f32 = 5.0;
const DASH_FPS: f32 = 12.0;
/// Colpo del boss dopo il preavviso, caduta e dissolvenza dei resti.
const IMPACT_S: f32 = 0.4;
pub const DEATH_S: f32 = 0.4;
const FADE_S: f32 = 0.3;
/// Sotto questo spostamento orizzontale per passo (px) il verso resta quello di prima.
const MIN_STEP_X: f32 = 0.01;

/// Righe del foglio di un nemico: movimento e sconfitta, più preparazione e attacco per chi
/// carica o colpisce ad area.
pub fn sheet_rows(enemy: &EnemyData) -> u32 {
    if enemy.charge.is_some() || enemy.slam.is_some() {
        4
    } else {
        2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyPose {
    #[default]
    Move,
    WindUp,
    /// Scatto dell'élite.
    Dash,
    /// Colpo del boss.
    Slam,
}

/// Stato dell'animazione di un nemico con foglio.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct EnemyAnimation {
    pub pose: EnemyPose,
    pub clock: f32,
    pub facing_left: bool,
    /// Secondi di colpo rimasti (boss).
    pub impact_left: f32,
    /// Il boss stava preparando il colpo al fotogramma prima.
    winding: bool,
    /// Durata della preparazione (dai dati di carica o colpo).
    windup_s: f32,
}

/// Resti di un nemico sconfitto: la caduta, poi la dissolvenza.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Remains {
    pub clock: f32,
}

/// Una griglia per ogni misura di fotogramma e numero di righe.
#[derive(Resource, Default)]
struct SheetLayouts(HashMap<(u32, u32, u32), Handle<TextureAtlasLayout>>);

pub struct EnemyAnimPlugin;

impl Plugin for EnemyAnimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SheetLayouts>()
            .add_observer(attach_enemy_sprite)
            .add_observer(leave_remains)
            .add_systems(Update, (animate_enemies, fade_remains));
    }
}

/// Fotogramma di un ciclo a `fps` dopo `t` secondi.
pub fn cycle_frame(t: f32, fps: f32) -> u32 {
    ((t.max(0.0) * fps) as u32) % COLUMNS
}

/// Fotogramma di un'animazione da `duration` secondi che si ferma sull'ultimo.
pub fn once_frame(t: f32, duration: f32) -> u32 {
    if duration <= 0.0 {
        return COLUMNS - 1;
    }
    ((t.max(0.0) / duration * COLUMNS as f32) as u32).min(COLUMNS - 1)
}

/// Un passo dell'animazione. `charge`: stato della carica (élite), che decide anche il verso;
/// `slam_winding`: c'è un colpo ad area di questo nemico in preparazione (boss); `step`:
/// spostamento dell'ultimo passo fisso.
pub fn advance(
    anim: &mut EnemyAnimation,
    charge: Option<&ChargeState>,
    slam_winding: bool,
    step: Vec2,
    dt: f32,
) {
    let previous = anim.pose;
    let aim_x = match charge {
        Some(ChargeState::Windup { direction, .. } | ChargeState::Dashing { direction, .. }) => {
            Some(direction.x)
        }
        _ => None,
    };
    let dx = aim_x.unwrap_or(step.x);
    if dx.abs() >= MIN_STEP_X {
        anim.facing_left = dx < 0.0;
    }
    anim.pose = match charge {
        Some(ChargeState::Windup { .. }) => EnemyPose::WindUp,
        Some(ChargeState::Dashing { .. }) => EnemyPose::Dash,
        _ if slam_winding => EnemyPose::WindUp,
        _ => {
            if anim.winding {
                anim.impact_left = IMPACT_S; // il preavviso è appena finito: colpo
            }
            if anim.impact_left > 0.0 {
                anim.impact_left = (anim.impact_left - dt).max(0.0);
                EnemyPose::Slam
            } else {
                EnemyPose::Move
            }
        }
    };
    anim.winding = slam_winding;
    anim.clock = if anim.pose == previous {
        anim.clock + dt
    } else {
        0.0
    };
}

/// Riga e colonna del foglio per lo stato corrente; `boss` rallenta il passo.
pub fn frame_of(anim: &EnemyAnimation, boss: bool) -> (u32, u32) {
    match anim.pose {
        EnemyPose::Move => (
            0,
            cycle_frame(anim.clock, if boss { BOSS_MOVE_FPS } else { MOVE_FPS }),
        ),
        EnemyPose::WindUp => (2, once_frame(anim.clock, anim.windup_s)),
        EnemyPose::Dash => (3, cycle_frame(anim.clock, DASH_FPS)),
        EnemyPose::Slam => (3, once_frame(IMPACT_S - anim.impact_left, IMPACT_S)),
    }
}

fn attach_enemy_sprite(
    add: On<Add<Enemy>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    ids: Query<&DataId>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut grids: ResMut<SheetLayouts>,
) {
    let data = ids.get(add.entity).ok().and_then(|id| config.enemy(&id.0));
    let path = data_sprite("nemico", ids.get(add.entity).ok(), |id| {
        config.enemy(id).map(|e| e.sprite.clone())
    });
    let Some(path) = path else {
        return;
    };
    let image = assets.load(path);
    let Some((data, frame)) = data.and_then(|d| Some((d, d.frame?))) else {
        commands
            .entity(add.entity)
            .insert((Sprite::from_image(image), Anchor::BOTTOM_CENTER));
        return;
    };
    let rows = sheet_rows(data);
    let layout = grids
        .0
        .entry((frame.w, frame.h, rows))
        .or_insert_with(|| {
            layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(frame.w, frame.h),
                COLUMNS,
                rows,
                None,
                None,
            ))
        })
        .clone();
    let windup_s = data
        .charge
        .map(|c| c.windup_s)
        .or(data.slam.map(|s| s.windup_s))
        .unwrap_or(0.0);
    commands.entity(add.entity).insert((
        Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 }),
        Anchor::BOTTOM_CENTER,
        EnemyAnimation {
            windup_s,
            ..default()
        },
    ));
}

fn animate_enemies(
    time: Res<Time>,
    slams: Query<&Telegraph, With<PendingSlam>>,
    mut enemies: Query<
        (
            Entity,
            &Position,
            &PreviousPosition,
            Option<&Charger>,
            Has<Slammer>,
            &mut EnemyAnimation,
            &mut Sprite,
        ),
        With<Enemy>,
    >,
) {
    let winding: HashSet<Entity> = slams.iter().map(|t| t.source).collect();
    let dt = time.delta_secs();
    for (entity, position, previous, charger, boss, mut anim, mut sprite) in &mut enemies {
        let step = position.0 - previous.0;
        advance(
            &mut anim,
            charger.map(|c| &c.state),
            winding.contains(&entity),
            step,
            dt,
        );
        let (row, column) = frame_of(&anim, boss);
        let index = (row * COLUMNS + column) as usize;
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        if sprite.flip_x != anim.facing_left {
            sprite.flip_x = anim.facing_left;
        }
    }
}

/// Un nemico sconfitto lascia i suoi resti animati; chi sparisce per altri motivi (restart,
/// fine della run) no.
fn leave_remains(
    remove: On<Remove<Enemy>>,
    mut commands: Commands,
    state: Option<Res<State<AppState>>>,
    enemies: Query<(&Health, &Sprite, &Transform)>,
) {
    if state.is_none_or(|s| *s.get() != AppState::InRun) {
        return;
    }
    let Ok((health, sprite, transform)) = enemies.get(remove.entity) else {
        return;
    };
    let Some(atlas) = sprite.texture_atlas.as_ref() else {
        return; // immagine fissa: sparisce come prima
    };
    if !health.is_dead() {
        return;
    }
    let mut remains = Sprite::from_atlas_image(
        sprite.image.clone(),
        TextureAtlas {
            layout: atlas.layout.clone(),
            index: COLUMNS as usize,
        },
    );
    remains.flip_x = sprite.flip_x;
    commands.spawn((
        Name::new("Resti"),
        remains,
        Anchor::BOTTOM_CENTER,
        *transform,
        Remains { clock: 0.0 },
        DespawnOnExit(AppState::InRun),
    ));
}

/// Opacità dei resti dopo `t` secondi: piena durante la caduta, poi a zero in `FADE_S`.
pub fn remains_alpha(t: f32) -> f32 {
    (1.0 - (t - DEATH_S) / FADE_S).clamp(0.0, 1.0)
}

fn fade_remains(
    mut commands: Commands,
    time: Res<Time>,
    mut remains: Query<(Entity, &mut Remains, &mut Sprite)>,
) {
    for (entity, mut r, mut sprite) in &mut remains {
        r.clock += time.delta_secs();
        if r.clock >= DEATH_S + FADE_S {
            commands.entity(entity).despawn();
            continue;
        }
        let index = (COLUMNS + once_frame(r.clock, DEATH_S)) as usize;
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        sprite.color = Color::WHITE.with_alpha(remains_alpha(r.clock));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn rows_depend_on_the_special_attacks() {
        let config = GameConfig::embedded();
        for enemy in &config.enemies {
            let rows = sheet_rows(enemy);
            let special = enemy.charge.is_some() || enemy.slam.is_some();
            assert_eq!(rows, if special { 4 } else { 2 }, "{}", enemy.id);
        }
    }

    #[test]
    fn walking_left_mirrors_and_standing_keeps_the_side() {
        let mut anim = EnemyAnimation::default();
        advance(&mut anim, None, false, Vec2::new(-0.5, 0.2), DT);
        assert!(anim.facing_left);
        advance(&mut anim, None, false, Vec2::new(0.0, 0.5), DT);
        assert!(anim.facing_left, "in verticale resta il verso di prima");
        advance(&mut anim, None, false, Vec2::new(0.5, 0.0), DT);
        assert!(!anim.facing_left);
        assert_eq!(frame_of(&anim, false).0, 0);
    }

    #[test]
    fn the_elite_shows_wind_up_then_dash_facing_its_charge() {
        let mut anim = EnemyAnimation {
            windup_s: 0.8,
            ..default()
        };
        let windup = ChargeState::Windup {
            remaining: 0.8,
            direction: Vec2::new(-1.0, 0.0),
        };
        advance(&mut anim, Some(&windup), false, Vec2::ZERO, DT);
        assert_eq!((anim.pose, anim.facing_left), (EnemyPose::WindUp, true));
        for _ in 0..60 {
            advance(&mut anim, Some(&windup), false, Vec2::ZERO, DT);
        }
        assert_eq!(
            frame_of(&anim, false),
            (2, COLUMNS - 1),
            "l'ultimo resta fermo"
        );
        let dash = ChargeState::Dashing {
            remaining: 0.5,
            direction: Vec2::new(-1.0, 0.0),
        };
        advance(&mut anim, Some(&dash), false, Vec2::new(-3.8, 0.0), DT);
        assert_eq!(frame_of(&anim, false).0, 3);
    }

    #[test]
    fn the_boss_winds_up_during_the_warning_then_slams() {
        let mut anim = EnemyAnimation {
            windup_s: 1.0,
            ..default()
        };
        for _ in 0..60 {
            advance(&mut anim, None, true, Vec2::ZERO, DT);
        }
        assert_eq!(frame_of(&anim, true), (2, COLUMNS - 1));
        // preavviso finito: colpo per IMPACT_S, poi di nuovo in cammino
        advance(&mut anim, None, false, Vec2::ZERO, DT);
        assert_eq!(anim.pose, EnemyPose::Slam);
        assert_eq!(frame_of(&anim, true).0, 3);
        let mut t = 0.0;
        while anim.pose == EnemyPose::Slam {
            advance(&mut anim, None, false, Vec2::ZERO, DT);
            t += DT;
            assert!(t <= IMPACT_S + DT);
        }
        assert_eq!(anim.pose, EnemyPose::Move);
    }

    #[test]
    fn remains_fall_then_fade_out() {
        assert_eq!(remains_alpha(0.0), 1.0);
        assert_eq!(remains_alpha(DEATH_S), 1.0);
        assert!((remains_alpha(DEATH_S + FADE_S / 2.0) - 0.5).abs() < 1e-5);
        assert_eq!(remains_alpha(DEATH_S + FADE_S), 0.0);
        assert_eq!(once_frame(DEATH_S * 2.0, DEATH_S), COLUMNS - 1);
    }

    /// Ogni nemico con un foglio nei dati ha un PNG grande esattamente 4 colonne per le sue
    /// righe di fotogrammi.
    #[test]
    fn enemy_sheets_match_their_frames() {
        let config = GameConfig::embedded();
        for enemy in &config.enemies {
            let Some(frame) = enemy.frame else { continue };
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(&enemy.sprite);
            let bytes = std::fs::read(&path).expect("foglio in assets/");
            let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!(
                (width, height),
                (frame.w * COLUMNS, frame.h * sheet_rows(enemy)),
                "{}",
                enemy.id
            );
        }
    }
}
