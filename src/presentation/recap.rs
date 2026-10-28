//! Riepilogo della squadra a fine run: per ogni compagno danni, quota sul danno della squadra,
//! nemici finiti, HP curati ed esperienza guadagnata (con il livello nuovo, se sale: un talento
//! da spendere nel menu); in fondo le cure di carte e rigenerazione. Il compagno che ha fatto
//! più danni è in oro. Nomi in Pixelify Sans, numeri in Tiny5 a 16 px.

use bevy::prelude::*;

use super::fonts;
use super::icons::{element_icon_path, ui_icon};
use crate::data::GameConfig;
use crate::data::schema::Element;
use crate::gameplay::combat::{RunStats, Source, Tally};
use crate::profile::RunXp;

const GOLD: Color = Color::srgb(0.95, 0.78, 0.3);
const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.6);
/// Larghezze delle colonne (px dell'interfaccia, pari): compagno, danni, quota, sconfitti,
/// cure, esperienza.
const COLUMNS: [f32; 6] = [180.0, 100.0, 60.0, 96.0, 70.0, 120.0];

/// Una riga del riepilogo.
#[derive(Debug, Clone, PartialEq)]
pub struct PartyLine {
    pub name: String,
    pub element: Element,
    pub tally: Tally,
    /// Quota del danno di tutta la squadra, tra 0 e 1.
    pub share: f32,
    /// Esperienza guadagnata nella run e livello nuovo del compagno, se è salito.
    pub xp: u32,
    pub level_up: Option<u32>,
}

/// Le righe dei compagni `members` (posto in squadra, fonte dei colpi), in ordine di posto.
pub fn party_lines(
    config: &GameConfig,
    stats: &RunStats,
    run_xp: &RunXp,
    members: impl Iterator<Item = (usize, Source)>,
) -> Vec<PartyLine> {
    let mut members: Vec<(usize, Source)> = members.collect();
    members.sort_by_key(|(slot, _)| *slot);
    let total: f32 = members.iter().map(|(_, s)| stats.tally(*s).damage).sum();
    members
        .into_iter()
        .filter_map(|(_, source)| {
            let data = config.companions.get(source.0)?;
            let tally = stats.tally(source);
            let xp = run_xp.0.iter().find(|l| l.id == data.id);
            Some(PartyLine {
                name: data.display_name.clone(),
                element: data.element,
                tally,
                share: if total > 0.0 {
                    tally.damage / total
                } else {
                    0.0
                },
                xp: xp.map_or(0, |l| l.gain.total()),
                level_up: xp
                    .filter(|l| l.level_after > l.level_before)
                    .map(|l| l.level_after),
            })
        })
        .collect()
}

/// Numero intero con il punto delle migliaia: 12345.6 → "12.346".
pub fn grouped(value: f32) -> String {
    let digits = (value.max(0.0).round() as u64).to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(digit);
    }
    out
}

/// Cella di testo larga quanto la sua colonna, numeri allineati a destra.
fn cell(text: String, column: usize, font: impl Bundle, color: Color) -> impl Bundle {
    (
        Node {
            width: px(COLUMNS[column]),
            justify_content: if column == 0 {
                JustifyContent::Start
            } else {
                JustifyContent::End
            },
            flex_shrink: 0.0,
            ..default()
        },
        children![(Text::new(text), font, TextColor(color))],
    )
}

fn row(commands: &mut Commands) -> Entity {
    commands
        .spawn(Node {
            align_items: AlignItems::Center,
            ..default()
        })
        .id()
}

/// La tabella del riepilogo, da aggiungere al pannello dei risultati.
pub fn spawn_recap(
    commands: &mut Commands,
    assets: &AssetServer,
    lines: &[PartyLine],
    player_healing: f32,
) -> Entity {
    let table = commands
        .spawn((
            Name::new("Riepilogo squadra"),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        ))
        .id();
    let header = row(commands);
    let titles = ["Squadra", "Danni", "%", "Sconfitti", "Cure", "Esperienza"];
    for (column, title) in titles.into_iter().enumerate() {
        let title = commands
            .spawn(cell(title.to_string(), column, fonts::ui(16.0), DIM))
            .id();
        commands.entity(header).add_child(title);
    }
    commands.entity(table).add_child(header);
    let best = lines.iter().map(|l| l.tally.damage).fold(0.0_f32, f32::max);
    for line in lines {
        let color = if best > 0.0 && line.tally.damage >= best {
            GOLD
        } else {
            Color::WHITE
        };
        let entity = row(commands);
        let name = commands
            .spawn((
                Node {
                    width: px(COLUMNS[0]),
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                children![
                    ui_icon(assets, element_icon_path(line.element), 16.0, 16.0),
                    (
                        Text::new(line.name.clone()),
                        fonts::ui(16.0),
                        TextColor(color)
                    ),
                ],
            ))
            .id();
        let healing = if line.tally.healing >= 0.5 {
            grouped(line.tally.healing)
        } else {
            "-".to_string()
        };
        let numbers = [
            grouped(line.tally.damage),
            format!("{:.0}%", line.share * 100.0),
            grouped(line.tally.kills as f32),
            healing,
        ];
        commands.entity(entity).add_child(name);
        for (k, text) in numbers.into_iter().enumerate() {
            let number = commands
                .spawn(cell(text, k + 1, fonts::hud(16.0), color))
                .id();
            commands.entity(entity).add_child(number);
        }
        let (xp, xp_color) = match line.level_up {
            Some(level) => (format!("+{} Lv {level}!", line.xp), GOLD),
            None => (format!("+{}", line.xp), color),
        };
        let xp = commands.spawn(cell(xp, 5, fonts::hud(16.0), xp_color)).id();
        commands.entity(entity).add_child(xp);
        commands.entity(table).add_child(entity);
    }
    if player_healing >= 0.5 {
        let entity = row(commands);
        let label = commands
            .spawn(cell(
                "Carte e rigenerazione".to_string(),
                0,
                fonts::ui(16.0),
                DIM,
            ))
            .id();
        let spacer = commands
            .spawn(Node {
                width: px(COLUMNS[1] + COLUMNS[2] + COLUMNS[3]),
                ..default()
            })
            .id();
        let value = commands
            .spawn(cell(grouped(player_healing), 4, fonts::hud(16.0), DIM))
            .id();
        commands
            .entity(entity)
            .add_children(&[label, spacer, value]);
        commands.entity(table).add_child(entity);
    }
    let promoted: Vec<&str> = lines
        .iter()
        .filter(|l| l.level_up.is_some())
        .map(|l| l.name.as_str())
        .collect();
    if !promoted.is_empty() {
        let note = commands
            .spawn((
                Text::new(format!(
                    "Nuovo talento per {}: nel menu premi T",
                    promoted.join(", ")
                )),
                fonts::ui(16.0),
                TextColor(GOLD),
            ))
            .id();
        commands.entity(table).add_child(note);
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_get_a_dot() {
        assert_eq!(grouped(0.0), "0");
        assert_eq!(grouped(999.4), "999");
        assert_eq!(grouped(1000.0), "1.000");
        assert_eq!(grouped(12345.6), "12.346");
        assert_eq!(grouped(1234567.0), "1.234.567");
        assert_eq!(grouped(-3.0), "0");
    }

    #[test]
    fn lines_follow_the_slots_and_share_the_squad_damage() {
        let config = GameConfig::embedded();
        let mut stats = RunStats::default();
        stats.tally_mut(Source(1)).damage = 300.0;
        stats.tally_mut(Source(0)).damage = 100.0;
        stats.tally_mut(Source(0)).kills = 7;
        // il compagno 1 sta nel posto 0, lo 0 nel posto 2, il 2 non ha fatto nulla
        let run_xp = RunXp(vec![crate::profile::XpLine {
            id: config.companions[1].id.clone(),
            gain: crate::data::schema::XpGain {
                time: 40,
                kills: 5,
                victory: 20,
            },
            level_before: 1,
            level_after: 2,
        }]);
        let lines = party_lines(
            &config,
            &stats,
            &run_xp,
            [(2, Source(0)), (0, Source(1)), (1, Source(2))].into_iter(),
        );
        let names: Vec<&str> = lines.iter().map(|l| l.name.as_str()).collect();
        let expected: Vec<&str> = [1, 2, 0]
            .iter()
            .map(|&i| config.companions[i].display_name.as_str())
            .collect();
        assert_eq!(names, expected);
        assert_eq!(lines[0].share, 0.75);
        assert_eq!(lines[1].share, 0.0);
        assert_eq!(lines[2].share, 0.25);
        assert_eq!(lines[2].tally.kills, 7);
        assert_eq!((lines[0].xp, lines[0].level_up), (65, Some(2)));
        assert_eq!((lines[1].xp, lines[1].level_up), (0, None));
    }
}
