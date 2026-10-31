use super::*;

#[test]
fn the_real_data_files_are_valid() {
    // Tutti i riferimenti tra file si risolvono (gli accessori non vanno in panic).
    let config = GameConfig::embedded();
    let companion = config.starting_companion();
    assert_eq!(config.companion_attack(companion).id, companion.attack);
    for wave in &config.stage.spawns.waves {
        assert_eq!(config.wave_enemy(wave).id, wave.enemy);
    }
    assert_eq!(
        config.first_wave_enemy().id,
        config.stage.spawns.waves[0].enemy
    );
}

/// D-068: ogni stage è validato come stage giocato, non solo il primo.
#[test]
fn every_stage_is_validated_not_only_the_first() {
    let config = GameConfig::embedded();
    assert_eq!(config.stage, config.stages[0]);
    let last = config.stages.last().unwrap().id.clone();
    assert_ne!(last, config.stage.id, "serve un secondo stage");
    let pattern = format!("biome: \"{}\"", config.stages.last().unwrap().biome);
    let broken = EMBEDDED.stages.replacen(&pattern, "biome: \"lava\"", 1);
    let errors = GameConfig::parse(&DataSources {
        stages: &broken,
        ..EMBEDDED
    })
    .unwrap_err();
    let wanted = format!("stages.ron: {last}.biome = \"lava\"");
    assert!(errors.iter().any(|e| e.starts_with(&wanted)), "{errors:?}");
    // gli errori comuni a tutti gli stage escono una volta sola
    let unique: std::collections::HashSet<&String> = errors.iter().collect();
    assert_eq!(unique.len(), errors.len(), "{errors:?}");
}

/// Lo stage si sceglie per id e non tocca il compagno né il resto della configurazione.
#[test]
fn a_stage_is_chosen_by_id_independently_of_the_companion() {
    let config = GameConfig::embedded();
    let forest = config.clone().with_stage("forest").unwrap();
    assert_eq!(forest.stage.id, "forest");
    assert_eq!(forest.biome().id, forest.stage.biome);
    assert_eq!(forest.companions, config.companions);
    assert_eq!(forest.stages, config.stages);
    let error = config.clone().with_stage("volcano").unwrap_err();
    assert!(
        error.contains("\"volcano\" non esiste") && error.contains("forest"),
        "{error}"
    );
    // ogni stage ha un bioma diverso dagli altri finora: la scelta cambia la mappa
    let biomes: Vec<&str> = config.stages.iter().map(|s| s.biome.as_str()).collect();
    assert!(biomes.contains(&"meadow") && biomes.contains(&"forest"));
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
