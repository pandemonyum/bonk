//! Esperienza: gemme lasciate dai nemici sconfitti, attrazione e raccolta, livelli. Cosa
//! succede a un livello nuovo (il draft delle carte) è in `upgrades`.

use bevy::prelude::*;

use super::SimSet;
use super::combat::Health;
use super::enemies::Enemy;
use super::kinematics::{Position, body};
use super::player::Player;
use crate::app::AppState;
use crate::data::GameConfig;
use crate::data::schema::XpCurve;

/// XP al massimo per nemico (validato nei dati): anche con la curva più bassa una gemma dà un
/// numero di livelli gestibile in un passo.
pub const MAX_ENEMY_XP: u32 = 1000;

/// Gemma a terra. Con troppe gemme a terra (`gem.max_on_field`) l'XP di una nuova si somma
/// alla gemma più vicina invece di creare un'entità.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpGem {
    pub value: u32,
}

/// Gemma entrata nel raggio di raccolta: vola verso il player finché non la prende.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attracted;

/// Distanza dai piedi del player entro cui le gemme vengono attirate.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PickupRadius(pub f32);

/// XP e livello della run.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Progression {
    pub level: u32,
    /// XP accumulata verso il prossimo livello.
    pub xp: u32,
    /// XP che servono per il prossimo livello.
    pub xp_to_next: u32,
    /// Livelli guadagnati e non ancora scelti nel draft.
    pub pending_levels: u32,
    /// XP raccolta in tutta la run.
    pub total_xp: u32,
}

/// XP per passare dal livello `level` al successivo: `base + per_level × (level − 1)`,
/// arrotondata per eccesso e almeno 1.
pub fn xp_to_next(curve: &XpCurve, level: u32) -> u32 {
    let needed = curve.base + curve.per_level * level.saturating_sub(1) as f32;
    needed.ceil().max(1.0) as u32
}

impl Progression {
    pub fn new(curve: &XpCurve) -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_to_next: xp_to_next(curve, 1),
            pending_levels: 0,
            total_xp: 0,
        }
    }

    /// Aggiunge XP; una gemma grande può far salire di più livelli in una volta.
    pub fn gain(&mut self, amount: u32, curve: &XpCurve) {
        self.xp = self.xp.saturating_add(amount);
        self.total_xp = self.total_xp.saturating_add(amount);
        while self.xp >= self.xp_to_next {
            self.xp -= self.xp_to_next;
            self.level += 1;
            self.pending_levels += 1;
            self.xp_to_next = xp_to_next(curve, self.level);
        }
    }

    /// Frazione del livello corrente già riempita, per la barra dell'HUD.
    pub fn fraction(&self) -> f32 {
        self.xp as f32 / self.xp_to_next.max(1) as f32
    }
}

impl Default for Progression {
    fn default() -> Self {
        Self::new(&XpCurve {
            base: 1.0,
            per_level: 0.0,
        })
    }
}

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Progression>()
            .add_systems(OnEnter(AppState::InRun), reset_progression)
            .add_systems(FixedUpdate, collect_gems.in_set(SimSet::Pickup));
    }
}

fn reset_progression(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(Progression::new(&config.progression.xp_to_level));
}

/// Nella catena del combattimento, tra i colpi e la rimozione dei morti: ogni nemico a 0 HP
/// lascia la sua gemma una sola volta, perché `resolve_deaths` lo rimuove nello stesso passo.
pub(super) fn drop_xp_gems(
    mut commands: Commands,
    config: Res<GameConfig>,
    dead: Query<(&Enemy, &Health, &Position)>,
    mut gems: Query<(&Position, &mut XpGem), Without<Enemy>>,
) {
    let max_on_field = config.progression.gem.max_on_field as usize;
    let mut on_field = gems.iter().len();
    let mut new_gems: Vec<(Vec2, u32)> = Vec::new();
    for (enemy, health, at) in &dead {
        if !health.is_dead() || enemy.xp == 0 {
            continue;
        }
        if on_field < max_on_field {
            new_gems.push((at.0, enemy.xp));
            on_field += 1;
            continue;
        }
        // Al limite: l'XP va alla gemma più vicina, già a terra o nata in questo passo.
        let nearest_old = gems
            .iter_mut()
            .map(|(p, gem)| (p.0.distance_squared(at.0), gem))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let nearest_new = new_gems
            .iter_mut()
            .map(|(p, value)| (p.distance_squared(at.0), value))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        match (nearest_old, nearest_new) {
            (Some((d_old, mut gem)), Some((d_new, _))) if d_old <= d_new => {
                gem.value = gem.value.saturating_add(enemy.xp);
            }
            (_, Some((_, value))) => *value = value.saturating_add(enemy.xp),
            (Some((_, mut gem)), None) => gem.value = gem.value.saturating_add(enemy.xp),
            (None, None) => new_gems.push((at.0, enemy.xp)), // limite 0: impedito dai dati
        }
    }
    for (at, value) in new_gems {
        commands.spawn((
            Name::new("Gemma XP"),
            XpGem { value },
            body(at),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// Le gemme nel raggio di raccolta vengono attirate e da lì in poi inseguono il player alla
/// velocità del magnete (più alta di quella massima del player: validato nei dati); a
/// `collect_distance` sono raccolte.
fn collect_gems(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut progression: ResMut<Progression>,
    player: Single<(&Position, &PickupRadius), With<Player>>,
    mut gems: Query<(Entity, &XpGem, &mut Position, Has<Attracted>), Without<Player>>,
) {
    let gem = &config.progression.gem;
    let (target, radius) = (player.0.0, player.1.0);
    let step = gem.magnet_speed * time.delta_secs();
    for (entity, xp, mut at, attracted) in &mut gems {
        let distance = at.0.distance(target);
        if !attracted {
            if distance > radius {
                continue;
            }
            commands.entity(entity).try_insert(Attracted);
        }
        at.0 = at.0.move_towards(target, step);
        if at.0.distance(target) <= gem.collect_distance {
            progression.gain(xp.value, &config.progression.xp_to_level);
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURVE: XpCurve = XpCurve {
        base: 5.0,
        per_level: 3.0,
    };

    #[test]
    fn the_curve_grows_linearly() {
        let needed: Vec<u32> = (1..=4).map(|level| xp_to_next(&CURVE, level)).collect();
        assert_eq!(needed, [5, 8, 11, 14]);
        let fractional = XpCurve {
            base: 2.2,
            per_level: 0.0,
        };
        assert_eq!(xp_to_next(&fractional, 1), 3);
    }

    #[test]
    fn xp_carries_over_and_a_big_gem_can_give_several_levels() {
        let mut p = Progression::new(&CURVE);
        p.gain(4, &CURVE);
        assert_eq!((p.level, p.xp, p.pending_levels), (1, 4, 0));
        p.gain(2, &CURVE); // 6 ≥ 5: livello 2 con 1 di resto
        assert_eq!(
            (p.level, p.xp, p.xp_to_next, p.pending_levels),
            (2, 1, 8, 1)
        );
        p.gain(30, &CURVE); // 31 = 8 + 11 + 12: livelli 3 e 4, resto 12 su 14
        assert_eq!(
            (p.level, p.xp, p.xp_to_next, p.pending_levels),
            (4, 12, 14, 3)
        );
        assert_eq!(p.total_xp, 36);
        assert!((p.fraction() - 12.0 / 14.0).abs() < 1e-6);
    }
}
