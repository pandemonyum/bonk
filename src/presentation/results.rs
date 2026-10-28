//! Schermata dei risultati a fine run, per la sconfitta e per la vittoria: durata, livello,
//! nemici sconfitti, danni subiti, riepilogo della squadra (`recap`) e potenziamenti presi. Invio o F5 per una run nuova
//! con lo stesso compagno, M per il menu (`gameplay::input`). Titolo e frasi in Pixelify Sans,
//! numeri e tasti in Tiny5.

use bevy::prelude::*;

use super::fonts;
use super::player_anim::DEFEAT_HOLD_S;
use super::recap::{PartyLine, party_lines, spawn_recap};
use crate::app::RunPhase;
use crate::data::GameConfig;
use crate::data::schema::Effect;
use crate::gameplay::combat::{RunStats, Source};
use crate::gameplay::companions::Companion;
use crate::gameplay::progression::Progression;
use crate::gameplay::run::RunClock;
use crate::gameplay::upgrades::UpgradeStacks;

const PANEL: Color = Color::srgba(0.05, 0.05, 0.12, 0.96);
const GOLD: Color = Color::srgb(0.95, 0.78, 0.3);
const LOSS: Color = Color::srgb(0.89, 0.34, 0.3);

/// Secondi reali che mancano al pannello della sconfitta: prima si vede il player cadere
/// (`player_anim`); a zero o sotto il pannello c'è già.
#[derive(Resource, Debug, Default)]
struct DefeatWait(f32);

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefeatWait>()
            .add_systems(
                OnEnter(RunPhase::Defeated),
                |mut wait: ResMut<DefeatWait>| {
                    wait.0 = DEFEAT_HOLD_S;
                },
            )
            .add_systems(Update, spawn_defeat.run_if(in_state(RunPhase::Defeated)))
            .add_systems(OnEnter(RunPhase::Victory), spawn_victory);
    }
}

/// Potenziamenti presi, nell'ordine dei dati: "Fiamma viva ×3". I compagni arrivati con una
/// carta sono nella riga della squadra, non qui; la cura non lascia stack da elencare.
pub fn build_summary(config: &GameConfig, stacks: &UpgradeStacks) -> Vec<String> {
    config
        .upgrades
        .iter()
        .enumerate()
        .filter(|(i, u)| {
            stacks.of(*i) > 0 && !matches!(u.effect, Effect::JoinCompanion(_) | Effect::Heal(_))
        })
        .map(|(i, u)| format!("{} ×{}", u.name, stacks.of(i)))
        .collect()
}

/// Il pannello nasce una volta, quando l'attesa della caduta scade.
fn spawn_defeat(
    time: Res<Time<Real>>,
    mut wait: ResMut<DefeatWait>,
    commands: Commands,
    config: Res<GameConfig>,
    stats: Res<RunStats>,
    clock: Res<RunClock>,
    progression: Res<Progression>,
    stacks: Res<UpgradeStacks>,
    assets: Res<AssetServer>,
    party: Query<(&Source, &Companion)>,
) {
    if wait.0 <= 0.0 {
        return;
    }
    wait.0 -= time.delta_secs();
    if wait.0 > 0.0 {
        return;
    }
    let party = party_lines(&config, &stats, party.iter().map(|(s, c)| (c.slot, *s)));
    let summary = Summary {
        victory: false,
        config: &config,
        stats: &stats,
        clock: &clock,
        progression: &progression,
        stacks: &stacks,
        party,
    };
    spawn_results(commands, &assets, summary);
}

fn spawn_victory(
    commands: Commands,
    config: Res<GameConfig>,
    stats: Res<RunStats>,
    clock: Res<RunClock>,
    progression: Res<Progression>,
    stacks: Res<UpgradeStacks>,
    assets: Res<AssetServer>,
    party: Query<(&Source, &Companion)>,
) {
    let party = party_lines(&config, &stats, party.iter().map(|(s, c)| (c.slot, *s)));
    let summary = Summary {
        victory: true,
        config: &config,
        stats: &stats,
        clock: &clock,
        progression: &progression,
        stacks: &stacks,
        party,
    };
    spawn_results(commands, &assets, summary);
}

struct Summary<'a> {
    victory: bool,
    config: &'a GameConfig,
    stats: &'a RunStats,
    clock: &'a RunClock,
    progression: &'a Progression,
    stacks: &'a UpgradeStacks,
    party: Vec<PartyLine>,
}

fn spawn_results(mut commands: Commands, assets: &AssetServer, s: Summary) {
    info!(
        "run finita: {} a {}, livello {}, sconfitti {}",
        if s.victory { "vittoria" } else { "sconfitta" },
        s.clock.label(),
        s.progression.level,
        s.stats.defeated
    );
    let (title, accent, phase) = if s.victory {
        ("VITTORIA!", GOLD, RunPhase::Victory)
    } else {
        ("SCONFITTO", LOSS, RunPhase::Defeated)
    };
    let boss = s
        .config
        .enemy(&s.config.stage.run.boss)
        .map_or(String::new(), |b| b.display_name.clone());
    let subtitle = if s.victory {
        format!(
            "{} sconfitto: {} è salva!",
            boss, s.config.stage.display_name
        )
    } else {
        s.config.stage.display_name.clone()
    };
    let numbers = format!(
        "Tempo {}   Livello {}   Nemici sconfitti {}",
        s.clock.label(),
        s.progression.level,
        s.stats.defeated
    );
    let damage = format!("Danni subiti {:.0}", s.stats.damage_taken);
    let build = build_summary(s.config, s.stacks);
    let build = if build.is_empty() {
        "Potenziamenti: nessuno".to_string()
    } else {
        format!("Potenziamenti: {}", build.join(", "))
    };
    let panel = commands
        .spawn((
            Node {
                width: px(620),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                padding: UiRect::all(px(18)),
                border: UiRect::all(px(4)),
                ..default()
            },
            BorderColor::all(accent),
            BackgroundColor(PANEL),
            children![
                (Text::new(title), fonts::ui(44.0), TextColor(accent),),
                (
                    Text::new(subtitle),
                    fonts::ui(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                ),
                (
                    Text::new(numbers),
                    fonts::hud(16.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Text::new(damage),
                    fonts::hud(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                ),
            ],
        ))
        .id();
    let recap = spawn_recap(&mut commands, assets, &s.party, s.stats.healing);
    let footer = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
            children![
                (
                    Text::new(build),
                    fonts::ui(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    TextLayout::justify(Justify::Center),
                ),
                (
                    Text::new("Invio o F5: nuova run      M: menu"),
                    fonts::hud(16.0),
                    TextColor(GOLD),
                ),
            ],
        ))
        .id();
    commands.entity(panel).add_children(&[recap, footer]);
    commands
        .spawn((
            Name::new("Results"),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            DespawnOnExit(phase),
        ))
        .add_child(panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_build_lists_the_cards_taken_with_their_stacks() {
        let config = GameConfig::embedded();
        let mut stacks = UpgradeStacks(vec![0; config.upgrades.len()]);
        let pick = |wanted: fn(&Effect) -> bool| {
            config
                .upgrades
                .iter()
                .position(|u| wanted(&u.effect))
                .unwrap()
        };
        let damage = pick(|e| matches!(e, Effect::CompanionDamagePct(_)));
        let join = pick(|e| matches!(e, Effect::JoinCompanion(_)));
        let heal = pick(|e| matches!(e, Effect::Heal(_)));
        stacks.0[damage] = 3;
        stacks.0[join] = 1;
        stacks.0[heal] = 2;
        let name = &config.upgrades[damage].name;
        assert_eq!(build_summary(&config, &stacks), [format!("{name} ×3")]);
        assert!(build_summary(&config, &UpgradeStacks(vec![0; config.upgrades.len()])).is_empty());
    }
}
