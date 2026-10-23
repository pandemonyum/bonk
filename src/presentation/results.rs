//! Schermata dei risultati a fine run, per la sconfitta e per la vittoria: durata, livello,
//! nemici sconfitti, danni subiti, squadra e potenziamenti presi. Invio o F5 per una run nuova
//! con lo stesso compagno, M per il menu (`gameplay::input`). Testi solo ASCII (D-033).

use bevy::prelude::*;

use crate::app::RunPhase;
use crate::data::schema::Effect;
use crate::data::{DataId, GameConfig};
use crate::gameplay::combat::RunStats;
use crate::gameplay::companions::Companion;
use crate::gameplay::progression::Progression;
use crate::gameplay::run::RunClock;
use crate::gameplay::upgrades::UpgradeStacks;

const PANEL: Color = Color::srgba(0.05, 0.05, 0.12, 0.96);
const GOLD: Color = Color::srgb(0.95, 0.78, 0.3);
const LOSS: Color = Color::srgb(0.89, 0.34, 0.3);

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RunPhase::Defeated), spawn_defeat)
            .add_systems(OnEnter(RunPhase::Victory), spawn_victory);
    }
}

/// Potenziamenti presi, nell'ordine dei dati: "Fiamma viva x3". I compagni arrivati con una
/// carta sono nella riga della squadra, non qui; la cura non lascia stack da elencare.
pub fn build_summary(config: &GameConfig, stacks: &UpgradeStacks) -> Vec<String> {
    config
        .upgrades
        .iter()
        .enumerate()
        .filter(|(i, u)| {
            stacks.of(*i) > 0 && !matches!(u.effect, Effect::JoinCompanion(_) | Effect::Heal(_))
        })
        .map(|(i, u)| format!("{} x{}", u.name, stacks.of(i)))
        .collect()
}

fn spawn_defeat(
    commands: Commands,
    config: Res<GameConfig>,
    stats: Res<RunStats>,
    clock: Res<RunClock>,
    progression: Res<Progression>,
    stacks: Res<UpgradeStacks>,
    party: Query<(&DataId, &Companion)>,
) {
    let party = party_names(&config, party.iter().map(|(id, c)| (c.slot, id)));
    let summary = Summary {
        victory: false,
        config: &config,
        stats: &stats,
        clock: &clock,
        progression: &progression,
        stacks: &stacks,
        party,
    };
    spawn_results(commands, summary);
}

fn spawn_victory(
    commands: Commands,
    config: Res<GameConfig>,
    stats: Res<RunStats>,
    clock: Res<RunClock>,
    progression: Res<Progression>,
    stacks: Res<UpgradeStacks>,
    party: Query<(&DataId, &Companion)>,
) {
    let party = party_names(&config, party.iter().map(|(id, c)| (c.slot, id)));
    let summary = Summary {
        victory: true,
        config: &config,
        stats: &stats,
        clock: &clock,
        progression: &progression,
        stacks: &stacks,
        party,
    };
    spawn_results(commands, summary);
}

/// Nomi dei compagni in ordine di posto.
fn party_names<'a>(
    config: &GameConfig,
    members: impl Iterator<Item = (usize, &'a DataId)>,
) -> Vec<String> {
    let mut members: Vec<(usize, String)> = members
        .map(|(slot, id)| {
            let name = config
                .companion(&id.0)
                .map_or(id.0.clone(), |c| c.display_name.clone());
            (slot, name)
        })
        .collect();
    members.sort();
    members.into_iter().map(|(_, name)| name).collect()
}

struct Summary<'a> {
    victory: bool,
    config: &'a GameConfig,
    stats: &'a RunStats,
    clock: &'a RunClock,
    progression: &'a Progression,
    stacks: &'a UpgradeStacks,
    party: Vec<String>,
}

fn spawn_results(mut commands: Commands, s: Summary) {
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
        format!("{} sconfitto: {} salva!", boss, s.config.stage.display_name)
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
    let party = format!("Squadra: {}", s.party.join(", "));
    let build = build_summary(s.config, s.stacks);
    let build = if build.is_empty() {
        "Potenziamenti: nessuno".to_string()
    } else {
        format!("Potenziamenti: {}", build.join(", "))
    };
    commands.spawn((
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
        children![(
            Node {
                width: px(620),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                padding: UiRect::all(px(18)),
                border: UiRect::all(px(3)),
                ..default()
            },
            BorderColor::all(accent),
            BackgroundColor(PANEL),
            children![
                (
                    Text::new(title),
                    TextFont::from_font_size(34.0),
                    TextColor(accent),
                ),
                (
                    Text::new(subtitle),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                ),
                (
                    Text::new(numbers),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Text::new(damage),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                ),
                (
                    Text::new(party),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::WHITE),
                ),
                (
                    Text::new(build),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    TextLayout::justify(Justify::Center),
                ),
                (
                    Text::new("Invio o F5: nuova run      M: menu"),
                    TextFont::from_font_size(14.0),
                    TextColor(GOLD),
                ),
            ],
        )],
    ));
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
        assert_eq!(build_summary(&config, &stacks), [format!("{name} x3")]);
        assert!(build_summary(&config, &UpgradeStacks(vec![0; config.upgrades.len()])).is_empty());
    }
}
