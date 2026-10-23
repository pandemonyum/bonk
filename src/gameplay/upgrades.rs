//! Draft a ogni livello e upgrade applicati a caldo. Al livello nuovo la run passa a
//! `RunPhase::LevelUp` (tempo fermo) con `draft_size` carte estratte per rarità; la carta
//! scelta aggiunge uno stack e i valori di compagni e player si ricalcolano subito dai dati
//! base più tutti gli stack, quindi nessun arrotondamento si accumula tra un livello e l'altro.

use bevy::prelude::*;

use super::SimSet;
use super::attacks::AutoAttack;
use super::combat::Health;
use super::companions::join_companions;
use super::orbit::OrbitAttack;
use super::player::Player;
use super::progression::{PickupRadius, Progression};
use super::pulse::PulseAttack;
use crate::app::{AppState, RunPhase, RunSeed, RunSetup};
use crate::data::GameConfig;
use crate::data::schema::Effect;
use crate::rng::Pcg32;

/// Stream RNG dedicato al draft (vedi `Pcg32::new`).
const DRAFT_STREAM: u64 = 2;
/// Carte al massimo in un draft (validato nei dati): tasti 1-3 e tre carte in 960 px.
pub const MAX_DRAFT_SIZE: u32 = 3;

/// Stack presi per ogni upgrade, nello stesso ordine di `GameConfig::upgrades`.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct UpgradeStacks(pub Vec<u32>);

impl UpgradeStacks {
    pub fn of(&self, index: usize) -> u32 {
        self.0.get(index).copied().unwrap_or(0)
    }
}

mod modifiers;
pub use modifiers::{Modifiers, pct_factor};

/// Carte del draft aperto: indici in `GameConfig::upgrades`.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct Draft {
    pub choices: Vec<usize>,
}

/// Scelta di una carta del draft aperto: indice in `Draft::choices` (0 = la prima).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChooseUpgrade(pub usize);

/// Sceglie da sola una carta dopo `delay_secs` secondi reali (autoplay, simulazioni headless),
/// come farebbe un giocatore alle prime armi: un compagno nuovo se c'è, se no la prima carta
/// ([`auto_choice`]). Senza questa risorsa il draft aspetta il giocatore.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct AutoPick {
    pub delay_secs: f32,
}

#[derive(Resource, Debug, Clone)]
struct DraftRng(Pcg32);

mod rules;
pub use rules::{PartyView, auto_choice, draw_choices, is_eligible};

pub struct UpgradesPlugin;

impl Plugin for UpgradesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpgradeStacks>()
            .init_resource::<Modifiers>()
            .init_resource::<Draft>()
            .insert_resource(DraftRng(Pcg32::new(0, DRAFT_STREAM)))
            .add_message::<ChooseUpgrade>()
            .add_systems(OnEnter(AppState::InRun), reset_upgrades)
            .add_systems(OnEnter(RunPhase::LevelUp), deal_draft)
            .add_systems(FixedUpdate, regenerate.in_set(SimSet::Pickup))
            .add_systems(
                Update,
                (
                    open_draft.run_if(in_state(RunPhase::Playing)),
                    auto_pick
                        .run_if(in_state(RunPhase::LevelUp))
                        .run_if(resource_exists::<AutoPick>),
                    apply_choice.run_if(in_state(RunPhase::LevelUp)),
                    // in `Loading` la configurazione può non esserci ancora (client)
                    (refresh_modifiers, join_companions)
                        .chain()
                        .run_if(in_state(AppState::InRun))
                        .run_if(resource_changed::<UpgradeStacks>),
                )
                    .chain(),
            );
    }
}

fn reset_upgrades(mut commands: Commands, config: Res<GameConfig>, seed: Res<RunSeed>) {
    commands.insert_resource(UpgradeStacks(vec![0; config.upgrades.len()]));
    commands.insert_resource(Modifiers::default());
    commands.insert_resource(Draft::default());
    commands.insert_resource(DraftRng(Pcg32::new(seed.0, DRAFT_STREAM)));
}

/// Con un livello da scegliere passa al draft. Le carte si estraggono entrando in `LevelUp`
/// ([`deal_draft`]): se nello stesso fotogramma vince una pausa (Esc, perdita di focus; vale
/// l'ultimo `NextState`) nessuna estrazione va sprecata e lo stesso seed dà le stesse carte.
/// Se non resta nessun upgrade disponibile i livelli passano senza fermare la run.
fn open_draft(
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    stacks: Res<UpgradeStacks>,
    mut progression: ResMut<Progression>,
    player: Query<&Health, With<Player>>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    if progression.pending_levels == 0 || player.iter().any(Health::is_dead) {
        return; // da sconfitti decide `declare_defeat`
    }
    let weights = &config.progression.rarity_weights;
    let party = PartyView::of(&config, &stacks, &setup.starter(&config).id);
    let available = (0..config.upgrades.len()).any(|i| {
        is_eligible(&config.upgrades, &stacks, i, &party)
            && weights.of(config.upgrades[i].rarity) > 0.0
    });
    if !available {
        info!(
            "nessun upgrade disponibile: livello {} senza draft",
            progression.level
        );
        progression.pending_levels = 0;
        return;
    }
    next.set(RunPhase::LevelUp);
}

fn deal_draft(
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    stacks: Res<UpgradeStacks>,
    mut progression: ResMut<Progression>,
    mut rng: ResMut<DraftRng>,
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let p = &config.progression;
    let level = progression.level + 1 - progression.pending_levels;
    draft.choices = draw_choices(
        &mut rng.0,
        &config.upgrades,
        &stacks,
        &PartyView::of(&config, &stacks, &setup.starter(&config).id),
        &p.rarity_weights,
        p.draft_size as usize,
        level <= p.companion_card_until_level,
    );
    if draft.choices.is_empty() {
        // impossibile con la configurazione che ha aperto il draft: si torna a giocare
        progression.pending_levels = 0;
        next.set(RunPhase::Playing);
        return;
    }
    log_draft(&config, level, &draft.choices);
}

fn auto_pick(
    auto: Res<AutoPick>,
    time: Res<Time<Real>>,
    config: Res<GameConfig>,
    draft: Res<Draft>,
    mut waited: Local<f32>,
    mut choose: MessageWriter<ChooseUpgrade>,
) {
    if draft.is_changed() {
        *waited = 0.0;
    } else {
        *waited += time.delta_secs();
    }
    if *waited >= auto.delay_secs {
        choose.write(ChooseUpgrade(auto_choice(&config.upgrades, &draft.choices)));
        *waited = f32::NEG_INFINITY; // una sola scelta per draft
    }
}

/// Applica la prima scelta valida del fotogramma e scarta le altre (tasto e clic insieme non
/// devono scegliere anche nel draft successivo): uno stack in più (la cura è immediata), poi
/// un nuovo draft se restano livelli da scegliere, altrimenti si torna a giocare.
fn apply_choice(
    mut choices: MessageReader<ChooseUpgrade>,
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    mut stacks: ResMut<UpgradeStacks>,
    mut progression: ResMut<Progression>,
    mut rng: ResMut<DraftRng>,
    mut draft: ResMut<Draft>,
    mut player: Query<&mut Health, With<Player>>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let mut picked = None;
    for choice in choices.read() {
        picked = picked.or(draft.choices.get(choice.0).copied());
    }
    // `get`: una ricarica dei dati nello stesso fotogramma può aver accorciato la lista
    let Some((index, chosen)) = picked.and_then(|i| Some((i, config.upgrades.get(i)?))) else {
        return;
    };
    if let Some(slot) = stacks.0.get_mut(index) {
        *slot += 1;
    }
    info!(
        "upgrade scelto: {} ({}/{})",
        chosen.id,
        stacks.of(index),
        chosen.max_stack
    );
    if let Effect::Heal(amount) = chosen.effect {
        for mut health in &mut player {
            health.current = (health.current + amount).min(health.max);
        }
    }
    progression.pending_levels = progression.pending_levels.saturating_sub(1);
    let p = &config.progression;
    // livello del draft successivo (il prossimo ancora da scegliere)
    let level = progression.level + 1 - progression.pending_levels;
    let again = if progression.pending_levels > 0 {
        draw_choices(
            &mut rng.0,
            &config.upgrades,
            &stacks,
            &PartyView::of(&config, &stacks, &setup.starter(&config).id),
            &p.rarity_weights,
            p.draft_size as usize,
            level <= p.companion_card_until_level,
        )
    } else {
        Vec::new()
    };
    if again.is_empty() {
        progression.pending_levels = 0;
        draft.choices.clear();
        next.set(RunPhase::Playing);
    } else {
        log_draft(&config, level, &again);
        draft.choices = again;
    }
}

fn log_draft(config: &GameConfig, level: u32, choices: &[usize]) {
    let ids: Vec<&str> = choices
        .iter()
        .filter_map(|&i| Some(config.upgrades.get(i)?.id.as_str()))
        .collect();
    info!("livello {level}: draft {ids:?}");
}

/// Ricalcola i valori di compagni e player dai dati base più gli stack. La carica degli
/// attacchi resta quella di prima; gli HP massimi in più curano della stessa quantità.
fn refresh_modifiers(
    config: Res<GameConfig>,
    stacks: Res<UpgradeStacks>,
    mut modifiers: ResMut<Modifiers>,
    mut projectiles: Query<&mut AutoAttack>,
    mut pulses: Query<&mut PulseAttack>,
    mut orbits: Query<&mut OrbitAttack>,
    mut player: Query<(&mut Player, &mut Health, &mut PickupRadius)>,
) {
    let m = Modifiers::from_stacks(&config.upgrades, &stacks);
    *modifiers = m;
    // lo stato in corso (carica, angolo, colpi recenti) resta quello di prima
    for mut attack in &mut projectiles {
        let Some((data, p)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.projectile()?)))
        else {
            continue;
        };
        let charge = attack.charge;
        *attack = AutoAttack {
            charge,
            ..m.projectile(data, p)
        };
    }
    for mut attack in &mut pulses {
        let Some((data, p)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.pulse()?)))
        else {
            continue;
        };
        let charge = attack.charge;
        *attack = PulseAttack {
            charge,
            ..m.pulse(data, p)
        };
    }
    for mut attack in &mut orbits {
        let Some((data, o)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.orbit()?)))
        else {
            continue;
        };
        let (angle, recent) = (attack.angle, std::mem::take(&mut attack.recent));
        *attack = OrbitAttack {
            angle,
            recent,
            ..m.orbit(data, o)
        };
    }
    let base = &config.player;
    for (mut player, mut health, mut pickup) in &mut player {
        player.speed = base.speed * m.move_speed;
        pickup.0 = base.pickup_radius * m.pickup_radius;
        let max = base.max_hp + m.max_hp_bonus;
        let gained = (max - health.max).max(0.0);
        health.max = max;
        health.current = (health.current + gained).min(max);
    }
}

fn regenerate(
    time: Res<Time>,
    modifiers: Res<Modifiers>,
    mut player: Query<&mut Health, With<Player>>,
) {
    if modifiers.regen_per_s <= 0.0 {
        return;
    }
    for mut health in &mut player {
        if !health.is_dead() {
            health.current =
                (health.current + modifiers.regen_per_s * time.delta_secs()).min(health.max);
        }
    }
}

#[cfg(test)]
mod tests;
