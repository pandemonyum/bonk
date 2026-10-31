//! Statistiche iniziali della selezione, tutte dall'attacco del compagno in `attacks.ron`
//! (niente valori di gioco nell'interfaccia): danno al secondo su un nemico, cadenza e portata,
//! con barre relative al compagno più forte in ciascuna. I compagni non hanno punti salute.

use bevy::prelude::*;

use crate::data::GameConfig;
use crate::data::schema::{AttackData, CompanionData};

/// Danno al secondo su un nemico, secondi tra due colpi, portata in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackStats {
    pub dps: f32,
    pub interval_s: f32,
    pub range: f32,
}

impl AttackStats {
    pub fn of(attack: &AttackData) -> Self {
        Self {
            dps: attack.damage / attack.interval_s,
            interval_s: attack.interval_s,
            range: attack.range,
        }
    }

    /// Colpi al secondo (le foglie: colpi allo stesso nemico).
    pub fn rate(&self) -> f32 {
        1.0 / self.interval_s
    }
}

/// [danno al secondo, colpi al secondo, portata] di ogni compagno dei dati, divisi per il
/// massimo tra tutti: valori da 0 a 1 per le barre.
pub fn relative_stats(config: &GameConfig) -> Vec<[f32; 3]> {
    let raw: Vec<[f32; 3]> = config
        .companions
        .iter()
        .map(|c| {
            let s = AttackStats::of(config.companion_attack(c));
            [s.dps, s.rate(), s.range]
        })
        .collect();
    let mut max = [f32::MIN_POSITIVE; 3];
    for row in &raw {
        for (m, v) in max.iter_mut().zip(row) {
            *m = m.max(*v);
        }
    }
    raw.iter()
        .map(|row| [row[0] / max[0], row[1] / max[1], row[2] / max[2]])
        .collect()
}

/// Numero con al più un decimale e la virgola: 10,909 → "10,9", 4,0 → "4".
pub fn decimal(value: f32) -> String {
    let tenths = (value * 10.0).round() as i64;
    if tenths % 10 == 0 {
        format!("{}", tenths / 10)
    } else {
        format!("{},{}", tenths / 10, (tenths % 10).abs())
    }
}

/// Una riga delle statistiche: icona (percorso e texel), nome, valore, barra 0..1 e colore.
pub(super) struct StatRow {
    pub icon: (&'static str, f32, f32),
    pub label: &'static str,
    pub value: String,
    pub fill: f32,
    pub color: Color,
}

/// Le tre righe del compagno `companion` (barre dell'handoff: fuoco, teal, lime).
pub(super) fn stat_rows(config: &GameConfig, companion: &CompanionData) -> [StatRow; 3] {
    let stats = AttackStats::of(config.companion_attack(companion));
    let index = config.companion_index(&companion.id).unwrap_or(0);
    let relative = relative_stats(config)
        .get(index)
        .copied()
        .unwrap_or([0.0; 3]);
    [
        StatRow {
            icon: ("ui/menu_new_run.png", 16.0, 16.0),
            label: "Danno",
            value: format!("{}/s", decimal(stats.dps)),
            fill: relative[0],
            color: Color::srgb_u8(0xff, 0xaa, 0x45),
        },
        StatRow {
            icon: ("ui/clock.png", 10.0, 10.0),
            label: "Cadenza",
            value: format!("ogni {} s", decimal(stats.interval_s)),
            fill: relative[1],
            color: Color::srgb_u8(0x6c, 0xc7, 0xc2),
        },
        StatRow {
            icon: ("ui/upgrade_ember_range.png", 16.0, 16.0),
            label: "Portata",
            value: format!("{} px", decimal(stats.range)),
            fill: relative[2],
            color: Color::srgb_u8(0xb8, 0xe0, 0x5a),
        },
    ]
}
