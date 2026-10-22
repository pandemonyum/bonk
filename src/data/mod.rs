//! Dati di gioco da `game-data/*.ron`: schema, parsing, validazione, accesso tipizzato.
//!
//! Il client li carica con l'AssetServer dalla sorgente `data://` (cartella `game-data/`,
//! funziona anche via HTTP sul web) e li ricarica a caldo nelle build di sviluppo (`--watch`). La
//! simulazione headless usa la copia incorporata a compile time degli stessi file.

pub mod loader;
pub mod schema;
mod validate;

use bevy::prelude::*;

use schema::*;

/// File di `game-data/`, nell'ordine dei campi di [`DataSources`].
pub const DATA_FILES: [&str; 5] = [
    "player.ron",
    "companions.ron",
    "attacks.ron",
    "enemies.ron",
    "stages.ron",
];

/// Testo grezzo dei file di dati.
#[derive(Debug, Clone, Copy)]
pub struct DataSources<'a> {
    pub player: &'a str,
    pub companions: &'a str,
    pub attacks: &'a str,
    pub enemies: &'a str,
    pub stages: &'a str,
}

/// Copia dei file incorporata nel binario: per test e simulazioni headless.
pub const EMBEDDED: DataSources<'static> = DataSources {
    player: include_str!("../../game-data/player.ron"),
    companions: include_str!("../../game-data/companions.ron"),
    attacks: include_str!("../../game-data/attacks.ron"),
    enemies: include_str!("../../game-data/enemies.ron"),
    stages: include_str!("../../game-data/stages.ron"),
};

/// Identificativo nei dati (es. `"blob"`, `"ember"`, `"ember_bolt"`) di un'entità di gioco:
/// la presentazione lo usa per trovare lo sprite.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DataId(pub String);

/// Stato del caricamento dei dati, mostrato a schermo dalla presentazione.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub enum DataStatus {
    #[default]
    Loading,
    Ready,
    /// Errori leggibili ("file: campo ..."). Se c'era già una configurazione valida resta in uso.
    Error(Vec<String>),
}

/// Configurazione validata: tutti i riferimenti tra file esistono e i valori sono coerenti.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct GameConfig {
    pub player: PlayerFile,
    pub companions: Vec<CompanionData>,
    pub attacks: Vec<AttackData>,
    pub enemies: Vec<EnemyData>,
    /// Lo stage giocato: il primo di `stages.ron`.
    pub stage: StageData,
}

impl GameConfig {
    /// Parsing e validazione; in caso di errori li restituisce tutti, non solo il primo.
    pub fn parse(sources: &DataSources) -> Result<Self, Vec<String>> {
        let mut errors = Vec::new();
        let player = parse_file::<PlayerFile>("player.ron", sources.player, &mut errors);
        let companions =
            parse_file::<CompanionsFile>("companions.ron", sources.companions, &mut errors);
        let attacks = parse_file::<AttacksFile>("attacks.ron", sources.attacks, &mut errors);
        let enemies = parse_file::<EnemiesFile>("enemies.ron", sources.enemies, &mut errors);
        let stages = parse_file::<StagesFile>("stages.ron", sources.stages, &mut errors);
        let (Some(player), Some(companions), Some(attacks), Some(enemies), Some(stages)) =
            (player, companions, attacks, enemies, stages)
        else {
            return Err(errors);
        };

        for (file, version) in [
            ("player.ron", player.version),
            ("companions.ron", companions.version),
            ("attacks.ron", attacks.version),
            ("enemies.ron", enemies.version),
            ("stages.ron", stages.version),
        ] {
            if version != SCHEMA_VERSION {
                errors.push(format!(
                    "{file}: version {version} non supportata (attesa {SCHEMA_VERSION})"
                ));
            }
        }
        let Some(stage) = stages.stages.into_iter().next() else {
            errors.push("stages.ron: serve almeno uno stage".into());
            return Err(errors);
        };
        let config = Self {
            player,
            companions: companions.companions,
            attacks: attacks.attacks,
            enemies: enemies.enemies,
            stage,
        };
        errors.extend(validate::validate(&config));
        if errors.is_empty() {
            Ok(config)
        } else {
            Err(errors)
        }
    }

    /// Configurazione dai file incorporati. I test garantiscono che siano validi.
    pub fn embedded() -> Self {
        match Self::parse(&EMBEDDED) {
            Ok(config) => config,
            Err(errors) => panic!("game-data/ non valido:\n{}", errors.join("\n")),
        }
    }

    pub fn companion(&self, id: &str) -> Option<&CompanionData> {
        self.companions.iter().find(|c| c.id == id)
    }

    pub fn attack(&self, id: &str) -> Option<&AttackData> {
        self.attacks.iter().find(|a| a.id == id)
    }

    pub fn enemy(&self, id: &str) -> Option<&EnemyData> {
        self.enemies.iter().find(|e| e.id == id)
    }

    /// Compagno iniziale dello stage (esiste: verificato dalla validazione).
    pub fn starting_companion(&self) -> &CompanionData {
        self.companion(&self.stage.starting_companion)
            .expect("riferimento validato in GameConfig::parse")
    }

    /// Attacco di un compagno (esiste: verificato dalla validazione).
    pub fn companion_attack(&self, companion: &CompanionData) -> &AttackData {
        self.attack(&companion.attack)
            .expect("riferimento validato in GameConfig::parse")
    }

    /// Nemico di un'ondata dello stage (esiste: verificato dalla validazione).
    pub fn wave_enemy(&self, wave: &WaveData) -> &EnemyData {
        self.enemy(&wave.enemy)
            .expect("riferimento validato in GameConfig::parse")
    }

    /// Nemico della prima ondata: quello dell'`initial_burst` (la validazione garantisce che
    /// esista almeno un'ondata).
    pub fn first_wave_enemy(&self) -> &EnemyData {
        self.wave_enemy(&self.stage.spawns.waves[0])
    }
}

fn parse_file<T: serde::de::DeserializeOwned>(
    file: &str,
    text: &str,
    errors: &mut Vec<String>,
) -> Option<T> {
    match ron::from_str::<T>(text) {
        Ok(value) => Some(value),
        Err(err) => {
            // SpannedError: "riga:colonna: messaggio"
            errors.push(format!("{file}: {err}"));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_real_data_files_are_valid() {
        let config = GameConfig::embedded();
        assert_eq!(config.stage.id, "meadow");
        assert_eq!(config.starting_companion().id, "ember");
        assert_eq!(
            config.companion_attack(config.starting_companion()).id,
            "ember_bolt"
        );
        assert_eq!(config.first_wave_enemy().id, "blob");
        let ids: Vec<&str> = config.enemies.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["blob", "swarm", "armored"]);
    }

    #[test]
    fn syntax_errors_name_the_file_and_position() {
        let broken = DataSources {
            enemies: "(version: 1, enemies: [ (id: \"blob\" ",
            ..EMBEDDED
        };
        let errors = GameConfig::parse(&broken).unwrap_err();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].starts_with("enemies.ron: 1:"), "{}", errors[0]);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let typo = EMBEDDED.player.replace("speed:", "sped:");
        let errors = GameConfig::parse(&DataSources {
            player: &typo,
            ..EMBEDDED
        })
        .unwrap_err();
        assert!(
            errors[0].contains("player.ron") && errors[0].contains("sped"),
            "{errors:?}"
        );
    }

    #[test]
    fn every_broken_file_is_reported_at_once() {
        let errors = GameConfig::parse(&DataSources {
            player: "(",
            stages: "[",
            ..EMBEDDED
        })
        .unwrap_err();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn unsupported_versions_are_rejected() {
        let future = EMBEDDED
            .attacks
            .replace(&format!("version: {SCHEMA_VERSION}"), "version: 99");
        let errors = GameConfig::parse(&DataSources {
            attacks: &future,
            ..EMBEDDED
        })
        .unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("attacks.ron: version 99")),
            "{errors:?}"
        );
    }
}
