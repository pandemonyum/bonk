//! Barre delle caratteristiche del menu.

use super::*;

#[test]
fn stat_bars_are_relative_to_the_strongest_companion() {
    let config = GameConfig::embedded();
    let stats = relative_stats(&config);
    assert_eq!(stats.len(), config.companions.len());
    for (dps, reach) in &stats {
        assert!((0.0..=1.0).contains(dps) && (0.0..=1.0).contains(reach));
    }
    assert!(
        stats.iter().any(|s| s.0 == 1.0),
        "il più forte ha la barra piena"
    );
    assert!(stats.iter().any(|s| s.1 == 1.0));
}
