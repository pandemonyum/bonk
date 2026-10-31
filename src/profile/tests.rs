//! Profilo: talenti, esperienza, file e sblocchi dei compagni iniziali.

use super::*;

#[test]
fn learning_needs_points_ranks_and_requirements() {
    let config = GameConfig::embedded();
    let ember = config.companion("ember").unwrap();
    let first = &ember.talents[0].nodes[0];
    let after = ember.talents[0]
        .nodes
        .iter()
        .find(|n| n.requires.contains(&first.id))
        .expect("un nodo richiede il primo");
    let mut profile = Profile::default();
    assert_eq!(profile.points(&config, "ember"), (1, 0, 0));
    assert!(
        !profile.learn(&config, "ember", &first.id),
        "nessun punto a livello 1"
    );
    // tanti punti quanti i gradi del primo nodo più uno
    let curve = config.progression.companion_xp;
    let points = first.max_rank() + 1;
    profile.entry("ember").xp = curve.total_for(points + 1);
    assert!(
        !profile.can_learn(&config, "ember", &after.id),
        "serve il nodo prima"
    );
    for rank in 1..=first.max_rank() {
        assert!(profile.learn(&config, "ember", &first.id));
        assert_eq!(profile.rank(&config, "ember", &first.id), rank);
    }
    assert!(!profile.learn(&config, "ember", &first.id), "al massimo");
    assert!(
        profile.learn(&config, "ember", &after.id),
        "aperto dal primo"
    );
    assert_eq!(
        profile.points(&config, "ember"),
        (points + 1, points, points)
    );
    assert!(
        !profile.can_learn(&config, "ember", &after.id),
        "punti finiti"
    );
    profile.reset("ember");
    assert_eq!(profile.points(&config, "ember").2, 0);
}

#[test]
fn a_run_awards_time_kills_and_victory() {
    let config = GameConfig::embedded();
    let mut profile = Profile::default();
    let lines = award(
        &config,
        &mut profile,
        &[("ember".into(), 960.0, 1000), ("tide".into(), 30.0, 0)],
        true,
    );
    let curve = config.progression.companion_xp;
    assert_eq!(lines[0].gain, curve.earned(960.0, 1000, true));
    assert_eq!(profile.progress("ember").xp, lines[0].gain.total());
    assert_eq!(lines[0].level_before, 1);
    assert_eq!(lines[0].level_after, curve.level(lines[0].gain.total()));
    assert_eq!(
        profile.progress("tide").xp,
        curve.earned(30.0, 0, true).total()
    );
}

#[test]
fn the_profile_round_trips_through_its_file() {
    let dir = std::env::temp_dir().join(format!("pms-profile-{}", std::process::id()));
    let path = dir.join(FILE_NAME);
    let mut profile = Profile::open(path.clone());
    assert_eq!(
        profile.data,
        ProfileData::default(),
        "file assente: profilo nuovo"
    );
    profile.entry("spark").xp = 321;
    profile.entry("spark").learned.push("spark_a1".into());
    profile.save();
    let again = Profile::open(path.clone());
    assert_eq!(again.data, profile.data);
    // un file rovinato non si perde: resta come .bak
    std::fs::write(&path, "(version: 1, companions: [rotto").unwrap();
    let broken = Profile::open(path.clone());
    assert_eq!(broken.data, ProfileData::default());
    assert!(path.with_extension("ron.bak").exists());
    let _ = std::fs::remove_dir_all(dir);
}

/// Un profilo nuovo ha solo i compagni `Start` dei dati: oggi Fiammetto.
#[test]
fn a_new_profile_starts_with_only_the_start_companions() {
    let config = GameConfig::embedded();
    let profile = Profile::default();
    let available: Vec<&str> = config
        .companions
        .iter()
        .filter(|c| is_unlocked(&config, &profile, &c.id))
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(available, ["ember"]);
    assert_eq!(
        unlock_status(&config, &profile, "tide"),
        Some(UnlockStatus::Locked)
    );
    assert_eq!(unlock_status(&config, &profile, "nessuno"), None);
    assert!(!is_unlocked(&config, &profile, "nessuno"));
    assert_eq!(
        first_unlocked(&config, &profile).map(|c| c.id.as_str()),
        Some("ember")
    );
    // l'ultimo usato vale solo se è ancora disponibile
    assert_eq!(proposed_starter(&config, &profile, None), "ember");
    assert_eq!(proposed_starter(&config, &profile, Some("tide")), "ember");
    assert_eq!(proposed_starter(&config, &profile, Some("ember")), "ember");
}

#[test]
fn granted_companions_stay_sorted_unique_and_saved() {
    let config = GameConfig::embedded();
    let dir = std::env::temp_dir().join(format!("pms-unlocks-{}", std::process::id()));
    let path = dir.join(FILE_NAME);
    let mut profile = Profile::open(path.clone());
    assert!(profile.grant("tide"));
    assert!(profile.grant("spark"));
    assert!(!profile.grant("tide"), "già sbloccato");
    assert_eq!(profile.data.unlocked, ["spark", "tide"]);
    assert!(is_unlocked(&config, &profile, "tide"));
    assert_eq!(proposed_starter(&config, &profile, Some("tide")), "tide");
    profile.save();
    let again = Profile::open(path.clone());
    assert_eq!(again.data.unlocked, ["spark", "tide"]);
    assert!(is_unlocked(&config, &again, "spark"));
    assert!(!is_unlocked(&config, &again, "sprout"));
    // un file scritto a mano in disordine e con doppioni si rimette in ordine alla lettura
    std::fs::write(
        &path,
        "(version: 1, unlocked: [\"tide\", \"spark\", \"tide\"])",
    )
    .unwrap();
    assert_eq!(Profile::open(path).data.unlocked, ["spark", "tide"]);
    let _ = std::fs::remove_dir_all(dir);
}

/// Un profilo salvato prima degli sblocchi si legge senza perdere nulla e, come uno nuovo, ha
/// solo i compagni `Start`: niente diritti per chi ha già giocato con gli altri (D-066).
#[test]
fn an_old_profile_keeps_its_progress_and_only_the_start_companions() {
    let config = GameConfig::embedded();
    let old = r#"(
    version: 1,
    companions: [
        (id: "ember", xp: 420, learned: ["ember_el_a1", "ember_el_a1"]),
        (id: "tide", xp: 99, learned: ["tide_el_a1"]),
    ],
    fullscreen: true,
)"#;
    let data: ProfileData = ron::from_str(old).unwrap();
    assert_eq!(data.version, PROFILE_VERSION);
    assert!(data.fullscreen);
    assert_eq!(data.companions.len(), 2);
    assert_eq!(data.companions[0].xp, 420);
    assert_eq!(data.companions[0].learned, ["ember_el_a1", "ember_el_a1"]);
    assert_eq!(data.companions[1].learned, ["tide_el_a1"]);
    assert!(data.unlocked.is_empty());
    let profile = Profile { data, path: None };
    assert!(is_unlocked(&config, &profile, "ember"));
    assert!(
        !is_unlocked(&config, &profile, "tide"),
        "ci ha giocato, ma resta da sbloccare"
    );
}

/// Una run in cui altri compagni si uniscono alla squadra dà loro esperienza ma non li sblocca
/// come compagni iniziali (D-055, D-066).
#[test]
fn a_run_with_joined_companions_unlocks_nobody() {
    use crate::app::RunPhase;
    use crate::data::schema::Effect;
    use crate::gameplay::upgrades::UpgradeStacks;

    let mut app = crate::headless_app(1);
    app.update();
    let config = app.world().resource::<GameConfig>().clone();
    let slots = config.player.companion_slots.len();
    let joins: Vec<(usize, String)> = config
        .upgrades
        .iter()
        .enumerate()
        .filter_map(|(i, u)| match &u.effect {
            Effect::JoinCompanion(id) if *id != config.stage.starting_companion => {
                Some((i, id.clone()))
            }
            _ => None,
        })
        .take(slots - 1)
        .collect();
    assert!(!joins.is_empty(), "i dati hanno carte nuovo compagno");
    {
        let mut stacks = app.world_mut().resource_mut::<UpgradeStacks>();
        for (i, _) in &joins {
            stacks.0[*i] = 1;
        }
    }
    for _ in 0..3 {
        app.update();
    }
    let world = app.world_mut();
    let party = world
        .query_filtered::<&crate::data::DataId, With<Companion>>()
        .iter(world)
        .count();
    assert_eq!(party, joins.len() + 1, "i compagni si sono uniti");
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::Defeated);
    app.update();
    let profile = app.world().resource::<Profile>();
    assert!(
        !app.world().resource::<RunXp>().0.is_empty(),
        "run assegnata"
    );
    assert!(profile.data.unlocked.is_empty());
    for (_, id) in &joins {
        assert!(
            profile.data.companions.iter().any(|c| c.id == *id),
            "{id} ha la sua riga di esperienza"
        );
        assert!(
            !is_unlocked(&config, profile, id),
            "{id} resta da sbloccare"
        );
    }
}
