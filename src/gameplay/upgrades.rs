//! Draft a ogni livello e upgrade applicati a caldo. Al livello nuovo la run passa a
//! `RunPhase::LevelUp` (tempo fermo) con `draft_size` carte estratte per rarità; la carta
//! scelta aggiunge uno stack e i valori di compagni e player si ricalcolano subito dai dati
//! base più tutti gli stack, quindi nessun arrotondamento si accumula tra un livello e l'altro.

use bevy::prelude::*;

use super::SimSet;
use super::attacks::AutoAttack;
use super::chain::ChainAttack;
use super::combat::{Health, RunStats, heal};
use super::companions::join_companions;
use super::orbit::OrbitAttack;
use super::player::Player;
use super::progression::{PickupRadius, Progression};
use super::pulse::PulseAttack;
use super::talents::TalentKit;
use super::treasure::Treasury;
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

/// Che cosa propone il draft aperto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DraftKind {
    /// Le carte di un livello.
    #[default]
    Level,
    /// Un livello dei compagni: chi può unirsi alla squadra.
    Companions,
    /// Un forziere appena aperto: le reliquie.
    Chest,
}

/// Carte del draft aperto: indici in `GameConfig::upgrades`.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct Draft {
    pub choices: Vec<usize>,
    pub kind: DraftKind,
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
pub use rules::{
    PartyView, auto_choice, draw_choices, draw_relics, is_eligible, offers_companions,
};

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

/// Il prossimo draft: prima le reliquie di un forziere appena aperto, poi il primo livello
/// ancora da scegliere (ai livelli dei compagni, i compagni). Nessuna carta se non c'è nulla
/// da scegliere o nessun upgrade disponibile.
fn next_draft(
    config: &GameConfig,
    stacks: &UpgradeStacks,
    starter: &str,
    progression: &Progression,
    treasury: &Treasury,
    rng: &mut Pcg32,
) -> (Vec<usize>, DraftKind) {
    let p = &config.progression;
    if treasury.pending {
        let relics = draw_relics(rng, &config.upgrades, stacks, p.draft_size as usize);
        return (relics, DraftKind::Chest);
    }
    if progression.pending_levels == 0 {
        return (Vec::new(), DraftKind::Level);
    }
    let level = progression.level + 1 - progression.pending_levels;
    let party = PartyView::of(config, stacks, starter);
    let companion_level = p.is_companion_level(level);
    let choices = draw_choices(
        rng,
        &config.upgrades,
        stacks,
        &party,
        &p.rarity_weights,
        p.draft_size as usize,
        companion_level,
    );
    let kind = if offers_companions(&config.upgrades, stacks, &party, companion_level) {
        DraftKind::Companions
    } else {
        DraftKind::Level
    };
    (choices, kind)
}

/// Con un livello da scegliere o un forziere aperto passa al draft. Le carte si estraggono
/// entrando in `LevelUp` ([`deal_draft`]): se nello stesso fotogramma vince una pausa (Esc,
/// perdita di focus; vale l'ultimo `NextState`) nessuna estrazione va sprecata e lo stesso seed
/// dà le stesse carte. Se non resta nessun upgrade disponibile i livelli passano senza fermare
/// la run.
fn open_draft(
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    stacks: Res<UpgradeStacks>,
    treasury: Res<Treasury>,
    mut progression: ResMut<Progression>,
    player: Query<&Health, With<Player>>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    if player.iter().any(Health::is_dead) {
        return; // da sconfitti decide `declare_defeat`
    }
    if treasury.pending {
        next.set(RunPhase::LevelUp);
        return;
    }
    if progression.pending_levels == 0 {
        return;
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
    mut treasury: ResMut<Treasury>,
    mut rng: ResMut<DraftRng>,
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let starter = setup.starter(&config).id.clone();
    let (choices, kind) = next_draft(
        &config,
        &stacks,
        &starter,
        &progression,
        &treasury,
        &mut rng.0,
    );
    if choices.is_empty() {
        // impossibile con la configurazione che ha aperto il draft: si torna a giocare
        progression.pending_levels = 0;
        treasury.pending = false;
        next.set(RunPhase::Playing);
        return;
    }
    log_draft(&config, &progression, kind, &choices);
    *draft = Draft { choices, kind };
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
/// un nuovo draft se restano livelli da scegliere, altrimenti si torna a giocare. La scelta di
/// un forziere non consuma livelli.
#[allow(clippy::too_many_arguments)]
fn apply_choice(
    mut choices: MessageReader<ChooseUpgrade>,
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    mut stacks: ResMut<UpgradeStacks>,
    mut progression: ResMut<Progression>,
    mut treasury: ResMut<Treasury>,
    mut rng: ResMut<DraftRng>,
    mut draft: ResMut<Draft>,
    mut stats: ResMut<RunStats>,
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
            stats.healing += heal(&mut health, amount);
        }
    }
    match draft.kind {
        DraftKind::Chest => treasury.pending = false,
        DraftKind::Level | DraftKind::Companions => {
            progression.pending_levels = progression.pending_levels.saturating_sub(1);
        }
    }
    let starter = setup.starter(&config).id.clone();
    let (again, kind) = next_draft(
        &config,
        &stacks,
        &starter,
        &progression,
        &treasury,
        &mut rng.0,
    );
    if again.is_empty() {
        progression.pending_levels = 0;
        treasury.pending = false;
        *draft = Draft::default();
        next.set(RunPhase::Playing);
    } else {
        log_draft(&config, &progression, kind, &again);
        *draft = Draft {
            choices: again,
            kind,
        };
    }
}

fn log_draft(config: &GameConfig, progression: &Progression, kind: DraftKind, choices: &[usize]) {
    let ids: Vec<&str> = choices
        .iter()
        .filter_map(|&i| Some(config.upgrades.get(i)?.id.as_str()))
        .collect();
    match kind {
        DraftKind::Chest => info!("forziere: reliquie {ids:?}"),
        _ => {
            let level = progression.level + 1 - progression.pending_levels;
            info!("livello {level}: draft {ids:?}");
        }
    }
}

/// Ricalcola i valori di compagni e player dai dati base più gli stack. La carica degli
/// attacchi resta quella di prima; gli HP massimi in più curano della stessa quantità.
fn refresh_modifiers(
    config: Res<GameConfig>,
    stacks: Res<UpgradeStacks>,
    mut modifiers: ResMut<Modifiers>,
    mut projectiles: Query<(&mut AutoAttack, Option<&TalentKit>)>,
    mut pulses: Query<(&mut PulseAttack, Option<&TalentKit>)>,
    mut orbits: Query<(&mut OrbitAttack, Option<&TalentKit>)>,
    mut chains: Query<(&mut ChainAttack, Option<&TalentKit>)>,
    mut stats: ResMut<RunStats>,
    mut player: Query<(&mut Player, &mut Health, &mut PickupRadius)>,
) {
    let m = Modifiers::from_stacks(&config.upgrades, &stacks);
    *modifiers = m;
    // lo stato in corso (carica, angolo, colpi recenti) resta quello di prima
    // ogni compagno: le carte più i suoi talenti
    let kit_of = |kit: Option<&TalentKit>| kit.copied().unwrap_or_default();
    for (mut attack, kit) in &mut projectiles {
        let Some((data, p)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.projectile()?)))
        else {
            continue;
        };
        let charge = attack.charge;
        *attack = AutoAttack {
            charge,
            ..kit_of(kit).modifiers(&m).projectile(data, p)
        };
    }
    for (mut attack, kit) in &mut pulses {
        let Some((data, p)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.pulse()?)))
        else {
            continue;
        };
        let kit = kit_of(kit);
        let charge = attack.charge;
        *attack = PulseAttack {
            charge,
            ..kit.modifiers(&m).pulse(data, p)
        };
        kit.tune_pulse(&mut attack);
    }
    for (mut attack, kit) in &mut orbits {
        let Some((data, o)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.orbit()?)))
        else {
            continue;
        };
        let kit = kit_of(kit);
        let (angle, recent) = (attack.angle, std::mem::take(&mut attack.recent));
        *attack = OrbitAttack {
            angle,
            recent,
            ..kit.modifiers(&m).orbit(data, o)
        };
        kit.tune_orbit(&mut attack);
    }
    for (mut attack, kit) in &mut chains {
        let Some((data, c)) = config
            .attack(&attack.id)
            .and_then(|a| Some((a, a.chain()?)))
        else {
            continue;
        };
        let kit = kit_of(kit);
        let charge = attack.charge;
        *attack = ChainAttack {
            charge,
            ..kit.modifiers(&m).chain(data, c)
        };
        kit.tune_chain(&mut attack);
    }
    let base = &config.player;
    for (mut player, mut health, mut pickup) in &mut player {
        player.speed = base.speed * m.move_speed;
        pickup.0 = base.pickup_radius * m.pickup_radius;
        let max = base.max_hp + m.max_hp_bonus;
        let gained = (max - health.max).max(0.0);
        health.max = max;
        stats.healing += heal(&mut health, gained);
    }
}

fn regenerate(
    time: Res<Time>,
    modifiers: Res<Modifiers>,
    mut stats: ResMut<RunStats>,
    mut player: Query<&mut Health, With<Player>>,
) {
    if modifiers.regen_per_s <= 0.0 {
        return;
    }
    for mut health in &mut player {
        stats.healing += heal(&mut health, modifiers.regen_per_s * time.delta_secs());
    }
}

#[cfg(test)]
mod tests;
