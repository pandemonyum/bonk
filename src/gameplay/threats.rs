//! Minacce telegrafate di élite e boss. Prima un segno rosso a terra resta visibile per almeno
//! [`MIN_TELEGRAPH_S`] (la linea della carica, il cerchio del colpo), poi l'attacco: chi guarda
//! ha il tempo di spostarsi. Valori da `enemies.ron` (`charge`, `slam`).

use bevy::prelude::*;

use super::SimSet;
use super::combat::{HIT_FLASH_SECS, Health, HitFlash, RunStats};
use super::damage::{GodMode, Invulnerability, Toughness, strike_player};
use super::kinematics::{Position, body};
use super::player::Player;
use crate::app::AppState;
use crate::data::schema::{ChargeData, SlamData};

/// Durata minima di un preavviso (validata nei dati): sotto, il segno non si fa in tempo a
/// vedere e l'attacco sembra a sorpresa.
pub const MIN_TELEGRAPH_S: f32 = 0.5;
/// Tolleranza sui timer: sommare 1/60 s a ogni passo accumula errore di arrotondamento e senza
/// tolleranza una fase da 0,25 s durerebbe 16 passi invece di 15.
const TIMER_EPSILON: f32 = 1.0e-4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChargeState {
    /// Insegue come gli altri; `cooldown` secondi alla prossima carica possibile.
    Chasing { cooldown: f32 },
    /// Fermo, con la traiettoria segnata a terra.
    Windup { remaining: f32, direction: Vec2 },
    /// Scatta in linea retta.
    Dashing { remaining: f32, direction: Vec2 },
}

/// Nemico che carica (élite).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Charger {
    pub data: ChargeData,
    pub state: ChargeState,
}

impl Charger {
    pub fn new(data: ChargeData) -> Self {
        Self {
            data,
            state: ChargeState::Chasing {
                cooldown: data.every_s,
            },
        }
    }

    /// Insegue normalmente (non è fermo a prendere la mira né in carica).
    pub fn is_chasing(&self) -> bool {
        matches!(self.state, ChargeState::Chasing { .. })
    }
}

/// Nemico che colpisce ad area (boss).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Slammer {
    pub data: SlamData,
    pub cooldown: f32,
}

impl Slammer {
    pub fn new(data: SlamData) -> Self {
        Self {
            data,
            cooldown: data.every_s,
        }
    }
}

/// Forma del segno di pericolo: `Line` parte dalla `Position` dell'entità verso `direction`,
/// `Ring` è centrato sulla `Position`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TelegraphShape {
    Line { direction: Vec2, length: f32 },
    Ring { radius: f32 },
}

/// Segno di pericolo a terra, visibile per `remaining` secondi. Sparisce prima, senza effetto,
/// se chi l'ha lanciato (`source`) muore o non c'è più.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Telegraph {
    pub source: Entity,
    pub shape: TelegraphShape,
    pub remaining: f32,
    /// Durata totale, per la grafica (il segno lampeggia più in fretta alla fine).
    pub total: f32,
}

/// Colpo in arrivo: allo scadere del suo `Telegraph`, chi sta nel cerchio prende `damage`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PendingSlam {
    pub radius: f32,
    pub damage: f32,
}

/// Un passo della carica. `to_player` è il vettore dal nemico al player. Restituisce il nuovo
/// stato e se in questo passo è iniziato il preavviso (va disegnata la linea).
pub fn charge_tick(
    state: ChargeState,
    data: &ChargeData,
    dt: f32,
    to_player: Vec2,
) -> (ChargeState, bool) {
    match state {
        ChargeState::Chasing { cooldown } => {
            let cooldown = (cooldown - dt).max(0.0);
            if cooldown <= TIMER_EPSILON && to_player.length() <= data.range {
                let direction = to_player.normalize_or(Vec2::X);
                let windup = ChargeState::Windup {
                    remaining: data.windup_s,
                    direction,
                };
                (windup, true)
            } else {
                (ChargeState::Chasing { cooldown }, false)
            }
        }
        ChargeState::Windup {
            remaining,
            direction,
        } => {
            let remaining = remaining - dt;
            if remaining <= TIMER_EPSILON {
                let dash = ChargeState::Dashing {
                    remaining: data.duration_s,
                    direction,
                };
                (dash, false)
            } else {
                (
                    ChargeState::Windup {
                        remaining,
                        direction,
                    },
                    false,
                )
            }
        }
        ChargeState::Dashing {
            remaining,
            direction,
        } => {
            let remaining = remaining - dt;
            if remaining <= TIMER_EPSILON {
                (
                    ChargeState::Chasing {
                        cooldown: data.every_s,
                    },
                    false,
                )
            } else {
                (
                    ChargeState::Dashing {
                        remaining,
                        direction,
                    },
                    false,
                )
            }
        }
    }
}

pub struct ThreatsPlugin;

impl Plugin for ThreatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            // la carica decide prima dell'inseguimento chi si muove in questo passo
            (charge_step.before(super::enemies::chase_player), slam_step).in_set(SimSet::Movement),
        )
        .add_systems(
            FixedUpdate,
            land_telegraphs
                .in_set(SimSet::Damage)
                .after(super::damage::DamageSteps),
        );
    }
}

/// Le cariche avanzano: preavviso con la linea a terra, poi lo scatto. Ostacoli, limiti
/// dell'arena e respinta dal player valgono come per tutti (`SimSet::Resolve`).
fn charge_step(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<&Position, With<Player>>,
    mut chargers: Query<(Entity, &mut Charger, &mut Position), Without<Player>>,
) {
    let dt = time.delta_secs();
    for (source, mut charger, mut position) in &mut chargers {
        let data = charger.data;
        let (state, started) = charge_tick(charger.state, &data, dt, player.0 - position.0);
        charger.state = state;
        if started && let ChargeState::Windup { direction, .. } = state {
            commands.spawn((
                Name::new("Preavviso carica"),
                Telegraph {
                    source,
                    shape: TelegraphShape::Line {
                        direction,
                        length: data.speed * data.duration_s,
                    },
                    remaining: data.windup_s,
                    total: data.windup_s,
                },
                body(position.0),
                DespawnOnExit(AppState::InRun),
            ));
        }
        if let ChargeState::Dashing { direction, .. } = charger.state {
            position.0 += direction * data.speed * dt;
        }
    }
}

/// Il boss segna il punto dove sta il player: il colpo arriverà lì, non dove andrà.
fn slam_step(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<&Position, With<Player>>,
    mut slammers: Query<(Entity, &mut Slammer, &Position, &Health), Without<Player>>,
) {
    let dt = time.delta_secs();
    for (source, mut slammer, at, health) in &mut slammers {
        slammer.cooldown = (slammer.cooldown - dt).max(0.0);
        let data = slammer.data;
        if health.is_dead()
            || slammer.cooldown > TIMER_EPSILON
            || at.0.distance(player.0) > data.range
        {
            continue;
        }
        slammer.cooldown = data.every_s;
        commands.spawn((
            Name::new("Colpo in arrivo"),
            Telegraph {
                source,
                shape: TelegraphShape::Ring {
                    radius: data.radius,
                },
                remaining: data.windup_s,
                total: data.windup_s,
            },
            PendingSlam {
                radius: data.radius,
                damage: data.damage,
            },
            body(player.0),
            DespawnOnExit(AppState::InRun),
        ));
    }
}

/// I preavvisi scadono; un colpo ad area colpisce il player se è ancora nel cerchio (con le
/// stesse regole del contatto: invulnerabilità e modalità invulnerabile). Se chi l'ha lanciato
/// è morto (anche in questo passo) il segno sparisce senza colpire.
fn land_telegraphs(
    mut commands: Commands,
    time: Res<Time>,
    god: Res<GodMode>,
    mut stats: ResMut<RunStats>,
    mut telegraphs: Query<(Entity, &mut Telegraph, &Position, Option<&PendingSlam>)>,
    sources: Query<&Health, Without<Player>>,
    player: Single<
        (
            Entity,
            &Position,
            &mut Health,
            &mut Invulnerability,
            Option<&Toughness>,
        ),
        With<Player>,
    >,
) {
    let dt = time.delta_secs();
    let (player_entity, feet, mut health, mut invulnerability, toughness) = player.into_inner();
    let taken = toughness.map_or(1.0, |t| t.0);
    for (entity, mut telegraph, center, slam) in &mut telegraphs {
        let gone = sources.get(telegraph.source).map_or(true, Health::is_dead);
        if gone {
            commands.entity(entity).try_despawn();
            continue;
        }
        telegraph.remaining -= dt;
        if telegraph.remaining > TIMER_EPSILON {
            continue;
        }
        commands.entity(entity).try_despawn();
        let Some(slam) = slam else { continue };
        if feet.0.distance(center.0) > slam.radius {
            continue; // schivato
        }
        let hit = strike_player(
            &mut health,
            &mut invulnerability,
            god.0,
            &mut stats,
            slam.damage * taken,
        );
        if hit {
            commands
                .entity(player_entity)
                .try_insert(HitFlash(HIT_FLASH_SECS));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHARGE: ChargeData = ChargeData {
        every_s: 2.0,
        windup_s: 0.5,
        speed: 200.0,
        duration_s: 0.25,
        range: 100.0,
    };

    /// Simula la carica per `secs` secondi a 60 Hz con il player fermo a `to_player`.
    fn run(mut state: ChargeState, secs: f32, to_player: Vec2) -> (ChargeState, u32) {
        let mut windups = 0;
        for _ in 0..(secs * 60.0).round() as u32 {
            let (next, started) = charge_tick(state, &CHARGE, 1.0 / 60.0, to_player);
            windups += u32::from(started);
            state = next;
        }
        (state, windups)
    }

    #[test]
    fn a_charge_waits_its_cooldown_and_needs_the_player_in_range() {
        let start = ChargeState::Chasing {
            cooldown: CHARGE.every_s,
        };
        let far = Vec2::new(CHARGE.range + 1.0, 0.0);
        let (state, windups) = run(start, 10.0, far);
        assert_eq!(windups, 0, "player fuori portata: nessuna carica");
        assert!(matches!(state, ChargeState::Chasing { .. }));
        let near = Vec2::new(0.0, CHARGE.range * 0.5);
        let (state, windups) = run(start, CHARGE.every_s - 0.1, near);
        assert_eq!(windups, 0, "prima del cooldown nessuna carica");
        let (state, windups) = run(state, 0.2, near);
        assert_eq!(windups, 1);
        let ChargeState::Windup { direction, .. } = state else {
            panic!("atteso il preavviso, trovato {state:?}")
        };
        assert_eq!(direction, Vec2::Y, "mira verso il player");
    }

    #[test]
    fn windup_then_dash_then_back_to_chasing() {
        let windup = ChargeState::Windup {
            remaining: CHARGE.windup_s,
            direction: Vec2::X,
        };
        let (state, _) = run(windup, CHARGE.windup_s, Vec2::X);
        assert!(matches!(state, ChargeState::Dashing { .. }), "{state:?}");
        let (state, _) = run(state, CHARGE.duration_s, Vec2::X);
        assert_eq!(
            state,
            ChargeState::Chasing {
                cooldown: CHARGE.every_s
            }
        );
    }
}
