//! Danni al player: i nemici a contatto infliggono il loro `contact_damage`, sommato, una volta
//! per finestra di invulnerabilità (`player.ron: invulnerability_s`). Essere circondati costa
//! caro; uscirne in tempo no. A 0 HP la simulazione si ferma subito (`simulation_running`) e
//! la run passa a `RunPhase::Defeated`.

use bevy::prelude::*;

use super::SimSet;
use super::combat::{HIT_FLASH_SECS, Health, HitFlash, RunStats};
use super::enemies::Enemy;
use super::kinematics::Position;
use super::player::Player;
use super::spatial::EnemyGrid;
use crate::app::RunPhase;

/// Tolleranza sul contatto: i nemici si fermano esattamente a `contact_distance`, l'errore di
/// arrotondamento non deve decidere se toccano.
pub const TOUCH_TOLERANCE: f32 = 1.0;

/// Finestra di invulnerabilità del player dopo un colpo.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Invulnerability {
    pub remaining: f32,
    pub duration: f32,
}

/// Player che non subisce danni (prove automatiche, screenshot di run lunghe: `--god`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct GodMode(pub bool);

/// Il contatto con i nemici in `SimSet::Damage`: i colpi ad area (`threats`) vengono dopo.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamageSteps;

/// Un colpo al player con le regole di tutti i danni: niente in modalità invulnerabile, durante
/// la finestra di invulnerabilità o da morto; se va a segno parte una nuova finestra.
/// Restituisce se il colpo è andato a segno.
pub fn strike_player(
    health: &mut Health,
    invulnerability: &mut Invulnerability,
    god: bool,
    stats: &mut RunStats,
    damage: f32,
) -> bool {
    if god || invulnerability.remaining > 0.0 || health.is_dead() || damage <= 0.0 {
        return false;
    }
    health.current -= damage;
    stats.damage_taken += damage;
    invulnerability.remaining = invulnerability.duration;
    true
}

/// Danno totale dai nemici a contatto con il player in `player`.
pub fn contact_damage(player: Vec2, enemies: impl IntoIterator<Item = (Vec2, Enemy)>) -> f32 {
    enemies
        .into_iter()
        .filter(|(at, enemy)| at.distance(player) <= enemy.contact_distance + TOUCH_TOLERANCE)
        .map(|(_, enemy)| enemy.contact_damage)
        .sum()
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodMode>()
            .add_systems(
                FixedUpdate,
                hurt_player.in_set(SimSet::Damage).in_set(DamageSteps),
            )
            .add_systems(PostUpdate, declare_defeat);
    }
}

/// La sconfitta si decide dagli HP, non da una richiesta fatta al momento del colpo: gira in
/// `PostUpdate`, dopo l'input, quindi vince su una pausa chiesta nello stesso fotogramma (Esc
/// o perdita di focus) e viene ripetuta finché la fase non è davvero `Defeated`.
pub(super) fn declare_defeat(
    phase: Option<Res<State<RunPhase>>>,
    next: Option<ResMut<NextState<RunPhase>>>,
    player: Query<&Health, With<Player>>,
) {
    let (Some(phase), Some(mut next)) = (phase, next) else {
        return;
    };
    if *phase.get() != RunPhase::Defeated && player.iter().any(Health::is_dead) {
        next.set(RunPhase::Defeated);
    }
}

fn hurt_player(
    mut commands: Commands,
    time: Res<Time>,
    god: Res<GodMode>,
    grid: Res<EnemyGrid>,
    mut stats: ResMut<RunStats>,
    enemies: Query<(&Enemy, &Health), Without<Player>>,
    player: Single<(Entity, &Position, &mut Health, &mut Invulnerability), With<Player>>,
) {
    let (entity, position, mut health, mut invulnerability) = player.into_inner();
    invulnerability.remaining = (invulnerability.remaining - time.delta_secs()).max(0.0);
    if god.0 || invulnerability.remaining > 0.0 || health.is_dead() {
        return;
    }
    // La griglia è aggiornata all'inizio del combattimento: i nemici morti in questo passo
    // hanno HP <= 0 e non colpiscono.
    let touching = grid
        .candidates(position.0, TOUCH_TOLERANCE)
        .filter_map(|entry| {
            let (enemy, enemy_health) = enemies.get(entry.entity).ok()?;
            (!enemy_health.is_dead()).then_some((entry.position, *enemy))
        });
    let damage = contact_damage(position.0, touching);
    if strike_player(&mut health, &mut invulnerability, god.0, &mut stats, damage) {
        commands.entity(entity).try_insert(HitFlash(HIT_FLASH_SECS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enemy(contact_distance: f32, contact_damage: f32) -> Enemy {
        Enemy {
            speed: 40.0,
            contact_distance,
            separation_radius: 14.0,
            contact_damage,
            xp: 1,
        }
    }

    #[test]
    fn only_touching_enemies_hurt_and_damage_adds_up() {
        let near = (Vec2::new(10.0, 0.0), enemy(10.0, 5.0));
        let edge = (Vec2::new(0.0, 10.9), enemy(10.0, 3.0));
        let far = (Vec2::new(30.0, 0.0), enemy(10.0, 100.0));
        assert_eq!(contact_damage(Vec2::ZERO, [near, edge, far]), 8.0);
        assert_eq!(contact_damage(Vec2::ZERO, [far]), 0.0);
        assert_eq!(contact_damage(Vec2::ZERO, []), 0.0);
    }

    #[test]
    fn a_strike_respects_god_mode_invulnerability_and_death() {
        let fresh = || {
            (
                Health::full(100.0),
                Invulnerability {
                    remaining: 0.0,
                    duration: 0.75,
                },
                RunStats::default(),
            )
        };
        let (mut h, mut inv, mut stats) = fresh();
        assert!(strike_player(&mut h, &mut inv, false, &mut stats, 25.0));
        assert_eq!(
            (h.current, inv.remaining, stats.damage_taken),
            (75.0, 0.75, 25.0)
        );
        // nella finestra di invulnerabilità niente
        assert!(!strike_player(&mut h, &mut inv, false, &mut stats, 25.0));
        assert_eq!(h.current, 75.0);
        let (mut h, mut inv, mut stats) = fresh();
        assert!(
            !strike_player(&mut h, &mut inv, true, &mut stats, 25.0),
            "invulnerabile"
        );
        assert_eq!(h.current, 100.0);
        h.current = 0.0;
        assert!(
            !strike_player(&mut h, &mut inv, false, &mut stats, 25.0),
            "già morto"
        );
        let (mut h, mut inv, mut stats) = fresh();
        assert!(
            !strike_player(&mut h, &mut inv, false, &mut stats, 0.0),
            "danno nullo"
        );
        assert_eq!(inv.remaining, 0.0, "nessuna finestra per un colpo nullo");
    }
}
