//! Animazioni del player dallo sprite sheet di `player.ron`: fermo e in cammino nelle quattro
//! direzioni (la sinistra è la destra specchiata), colpito e sconfitto. Solo presentazione:
//! legge spostamento, invulnerabilità, HP e fase della run, non cambia nulla della simulazione.
//!
//! Contratto con il foglio (`docs/ART_BIBLE.md`, D-050): fotogrammi di 36×38 texel, 4 colonne, righe
//! 0–2 fermo giù/su/destra, 3–5 cammino giù/su/destra, 6 colpito, 7 sconfitto (l'ultimo
//! fotogramma resta a terra). Piedi sul bordo basso del fotogramma, ancora in basso al centro.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::sprites::HIT_TINT;
use crate::app::RunPhase;
use crate::data::GameConfig;
use crate::gameplay::combat::{Health, HitFlash};
use crate::gameplay::damage::Invulnerability;
use crate::gameplay::kinematics::{Position, PreviousPosition};
use crate::gameplay::player::Player;

/// Fotogramma in texel e griglia del foglio (lo sprite dell'utente, personaggio alto ~30).
pub const FRAME: UVec2 = UVec2::new(36, 38);
pub const COLUMNS: u32 = 4;
pub const ROWS: u32 = 8;
/// Ritmo dei cicli: respiro lento da fermi, passo a 8 fotogrammi al secondo alla velocità base
/// (più svelto con gli upgrade di movimento, così i piedi non scivolano).
const IDLE_FPS: f32 = 3.0;
const WALK_FPS: f32 = 8.0;
/// Durata di colpito e caduta: la caduta usa il tempo reale (a run persa il tempo di gioco è
/// fermo) e i risultati aspettano che finisca (`DEFEAT_HOLD_S`).
pub const HURT_S: f32 = 0.32;
pub const DEATH_S: f32 = 0.7;
/// Dalla sconfitta alla schermata dei risultati: la caduta e un attimo a terra.
pub const DEFEAT_HOLD_S: f32 = DEATH_S + 0.5;
/// Sotto questo spostamento per passo (px) il player è fermo.
const MIN_STEP: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pose {
    #[default]
    Idle,
    Walk,
    Hurt,
    Death,
}

/// Riga del foglio per posa e direzione, e se va specchiata (la sinistra).
pub fn row_of(pose: Pose, facing: Facing) -> (u32, bool) {
    let (direction, flip) = match facing {
        Facing::Down => (0, false),
        Facing::Up => (1, false),
        Facing::Right => (2, false),
        Facing::Left => (2, true),
    };
    match pose {
        Pose::Idle => (direction, flip),
        Pose::Walk => (3 + direction, flip),
        // colpito e caduta sono disegnati di fronte
        Pose::Hurt => (6, false),
        Pose::Death => (7, false),
    }
}

/// Direzione da uno spostamento: l'asse dominante (in parità vince l'orizzontale, come nelle
/// diagonali dei giochi a quattro direzioni); da fermi resta quella di prima.
pub fn facing_from(delta: Vec2, previous: Facing) -> Facing {
    if delta.length() < MIN_STEP {
        return previous;
    }
    if delta.x.abs() >= delta.y.abs() {
        if delta.x > 0.0 {
            Facing::Right
        } else {
            Facing::Left
        }
    } else if delta.y > 0.0 {
        Facing::Up
    } else {
        Facing::Down
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

/// Stato dell'animazione del player.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerAnimation {
    pub facing: Facing,
    pub pose: Pose,
    /// Secondi nella posa corrente.
    pub clock: f32,
    /// Secondi di "colpito" rimasti.
    pub hurt_left: f32,
    /// Invulnerabilità vista al fotogramma prima: se risale, è arrivato un colpo.
    last_invulnerable: f32,
}

#[derive(Resource)]
struct PlayerSheet(Handle<TextureAtlasLayout>);

pub struct PlayerAnimPlugin;

impl Plugin for PlayerAnimPlugin {
    fn build(&self, app: &mut App) {
        let layout = TextureAtlasLayout::from_grid(FRAME, COLUMNS, ROWS, None, None);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<TextureAtlasLayout>>()
            .add(layout);
        app.insert_resource(PlayerSheet(handle))
            .add_observer(attach_player_sprite)
            .add_systems(Update, animate_player);
    }
}

fn attach_player_sprite(
    add: On<Add<Player>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    sheet: Res<PlayerSheet>,
) {
    let image = assets.load(config.player.sprite.clone());
    let atlas = TextureAtlas {
        layout: sheet.0.clone(),
        index: 0,
    };
    commands.entity(add.entity).insert((
        Sprite::from_atlas_image(image, atlas),
        Anchor::BOTTOM_CENTER,
        PlayerAnimation::default(),
    ));
}

/// Un passo dell'animazione: sconfitta > colpito > cammino > fermo.
pub fn advance(
    anim: &mut PlayerAnimation,
    step: Vec2,
    invulnerable: f32,
    dead: bool,
    dt: f32,
    real_dt: f32,
) {
    let previous = anim.pose;
    if dead {
        anim.pose = Pose::Death;
        anim.clock = if previous == Pose::Death {
            anim.clock + real_dt
        } else {
            0.0
        };
        return;
    }
    if invulnerable > anim.last_invulnerable + 1e-4 {
        anim.hurt_left = HURT_S;
    }
    anim.last_invulnerable = invulnerable;
    anim.facing = facing_from(step, anim.facing);
    anim.pose = if anim.hurt_left > 0.0 {
        anim.hurt_left = (anim.hurt_left - dt).max(0.0);
        Pose::Hurt
    } else if step.length() >= MIN_STEP {
        Pose::Walk
    } else {
        Pose::Idle
    };
    anim.clock = if anim.pose == previous {
        anim.clock + dt
    } else {
        0.0
    };
}

/// Indice nel foglio e specchio per lo stato corrente; `walk_rate` scala il passo con la
/// velocità del player (1 alla velocità base).
pub fn frame_of(anim: &PlayerAnimation, walk_rate: f32) -> (usize, bool) {
    let (row, flip) = row_of(anim.pose, anim.facing);
    let frame = match anim.pose {
        Pose::Idle => cycle_frame(anim.clock, IDLE_FPS),
        Pose::Walk => cycle_frame(anim.clock, WALK_FPS * walk_rate),
        Pose::Hurt => once_frame(HURT_S - anim.hurt_left, HURT_S),
        Pose::Death => once_frame(anim.clock, DEATH_S),
    };
    ((row * COLUMNS + frame) as usize, flip)
}

fn animate_player(
    time: Res<Time>,
    real: Res<Time<Real>>,
    config: Option<Res<GameConfig>>,
    phase: Option<Res<State<RunPhase>>>,
    mut players: Query<
        (
            &Position,
            &PreviousPosition,
            &Health,
            &Player,
            Option<&Invulnerability>,
            Has<HitFlash>,
            &mut PlayerAnimation,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    let defeated = phase.is_some_and(|p| *p.get() == RunPhase::Defeated);
    let base_speed = config.map_or(0.0, |c| c.player.speed);
    for (position, previous, health, player, invulnerability, flashing, mut anim, mut sprite) in
        &mut players
    {
        let step = position.0 - previous.0;
        let invulnerable = invulnerability.map_or(0.0, |i| i.remaining);
        let dead = defeated || health.is_dead();
        advance(
            &mut anim,
            step,
            invulnerable,
            dead,
            time.delta_secs(),
            real.delta_secs(),
        );
        let rate = if base_speed > 0.0 {
            (player.speed / base_speed).clamp(0.5, 2.0)
        } else {
            1.0
        };
        let (index, flip) = frame_of(&anim, rate);
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        if sprite.flip_x != flip {
            sprite.flip_x = flip;
        }
        // a terra niente tinta: a run persa la simulazione è ferma e il lampo del colpo non
        // scadrebbe più
        let tint = if flashing && anim.pose != Pose::Death {
            HIT_TINT
        } else {
            Color::WHITE
        };
        if sprite.color != tint {
            sprite.color = tint;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn the_dominant_axis_picks_the_direction_and_standing_keeps_it() {
        assert_eq!(
            facing_from(Vec2::new(1.0, 0.2), Facing::Down),
            Facing::Right
        );
        assert_eq!(
            facing_from(Vec2::new(-1.0, 0.9), Facing::Down),
            Facing::Left
        );
        assert_eq!(facing_from(Vec2::new(0.3, 1.0), Facing::Down), Facing::Up);
        assert_eq!(facing_from(Vec2::new(0.0, -1.0), Facing::Up), Facing::Down);
        // diagonale perfetta: orizzontale
        assert_eq!(facing_from(Vec2::new(-1.0, -1.0), Facing::Up), Facing::Left);
        // fermo: la direzione resta
        assert_eq!(facing_from(Vec2::ZERO, Facing::Left), Facing::Left);
    }

    #[test]
    fn rows_follow_the_sheet_contract_and_left_mirrors_right() {
        assert_eq!(row_of(Pose::Idle, Facing::Down), (0, false));
        assert_eq!(row_of(Pose::Idle, Facing::Up), (1, false));
        assert_eq!(row_of(Pose::Idle, Facing::Right), (2, false));
        assert_eq!(row_of(Pose::Idle, Facing::Left), (2, true));
        assert_eq!(row_of(Pose::Walk, Facing::Down), (3, false));
        assert_eq!(row_of(Pose::Walk, Facing::Left), (5, true));
        assert_eq!(row_of(Pose::Hurt, Facing::Left), (6, false));
        assert_eq!(row_of(Pose::Death, Facing::Up), (7, false));
        for pose in [Pose::Idle, Pose::Walk, Pose::Hurt, Pose::Death] {
            for facing in [Facing::Down, Facing::Up, Facing::Left, Facing::Right] {
                assert!(row_of(pose, facing).0 < ROWS);
            }
        }
    }

    #[test]
    fn cycles_loop_and_one_shots_stop_on_the_last_frame() {
        assert_eq!(cycle_frame(0.0, 8.0), 0);
        assert_eq!(cycle_frame(0.13, 8.0), 1);
        assert_eq!(cycle_frame(0.5, 8.0), 0); // 4 fotogrammi a 8/s: mezzo secondo per giro
        assert_eq!(once_frame(0.0, DEATH_S), 0);
        assert_eq!(once_frame(DEATH_S * 0.6, DEATH_S), 2);
        assert_eq!(once_frame(DEATH_S * 5.0, DEATH_S), COLUMNS - 1);
    }

    #[test]
    fn a_hit_plays_hurt_then_walking_resumes_and_defeat_falls_and_stays_down() {
        let mut anim = PlayerAnimation::default();
        advance(&mut anim, Vec2::new(1.25, 0.0), 0.0, false, DT, DT);
        assert_eq!((anim.pose, anim.facing), (Pose::Walk, Facing::Right));
        // colpo: l'invulnerabilità sale da 0 a 0,75
        advance(&mut anim, Vec2::new(1.25, 0.0), 0.75, false, DT, DT);
        assert_eq!(anim.pose, Pose::Hurt);
        let mut t = 0.0;
        while anim.pose == Pose::Hurt {
            advance(&mut anim, Vec2::new(1.25, 0.0), 0.7 - t, false, DT, DT);
            t += DT;
            assert!(t < HURT_S + 0.1, "colpito troppo lungo");
        }
        assert_eq!(anim.pose, Pose::Walk);
        // l'invulnerabilità che scende non è un colpo nuovo
        advance(&mut anim, Vec2::ZERO, 0.2, false, DT, DT);
        assert_eq!(anim.pose, Pose::Idle);
        // sconfitta: caduta col tempo reale anche a tempo di gioco fermo, poi resta a terra
        for _ in 0..120 {
            advance(&mut anim, Vec2::ZERO, 0.0, true, 0.0, DT);
        }
        assert_eq!(anim.pose, Pose::Death);
        let (index, flip) = frame_of(&anim, 1.0);
        assert_eq!((index, flip), ((7 * COLUMNS + COLUMNS - 1) as usize, false));
    }

    #[test]
    fn walking_left_uses_the_mirrored_side_row() {
        let mut anim = PlayerAnimation::default();
        advance(&mut anim, Vec2::new(-1.25, 0.1), 0.0, false, DT, DT);
        let (index, flip) = frame_of(&anim, 1.0);
        assert_eq!((index, flip), ((5 * COLUMNS) as usize, true));
    }

    /// Il PNG del player (dai dati) ha esattamente la griglia del contratto.
    #[test]
    fn the_player_sheet_matches_the_frame_grid() {
        let config = GameConfig::embedded();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(&config.player.sprite);
        let bytes = std::fs::read(&path).expect("foglio del player in assets/");
        assert_eq!(&bytes[1..4], b"PNG", "{}", path.display());
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!(
            (width, height),
            (FRAME.x * COLUMNS, FRAME.y * ROWS),
            "{}",
            path.display()
        );
    }
}
