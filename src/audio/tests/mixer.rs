//! Regole del mixer (progetto § 3.5): cooldown, istanze, posti e furto, coalescenza, budget del
//! fotogramma, cull. Il margine è in `margin.rs`, il tempo in `mixer_time.rs`. L'orologio è in
//! mano al test.

use bevy::math::{Rect, Vec2};

use super::{catalog_with, cue_data};
use crate::audio::catalog::{AudioCatalog, Coalesce, CueData, Cull, OnLimit, Priority};
use crate::audio::cues::CueRequest;
use crate::audio::mixer::*;

/// Limiti larghi: solo la regola sotto prova conta.
pub(super) fn limits(
    max_voices: usize,
    reserved_critical: usize,
    starts: usize,
    headroom: f32,
) -> Limits {
    Limits {
        max_voices,
        reserved_critical,
        starts_per_frame: starts,
        sfx_headroom: headroom,
    }
}

pub(super) fn wide() -> Limits {
    limits(64, 0, 64, 1000.0)
}

pub(super) fn mixer(cues: Vec<(&str, CueData)>, limits: Limits) -> (AudioCatalog, Mixer) {
    let catalog = catalog_with(cues);
    let mixer = Mixer::with(&catalog, limits, AUDIO_SEED);
    (catalog, mixer)
}

pub(super) fn req(catalog: &AudioCatalog, name: &str) -> CueRequest {
    CueRequest {
        cue: catalog.exact(name).expect("cue di prova"),
        count: 1,
        at: None,
    }
}

pub(super) fn started(decisions: &[Decision]) -> Vec<VoiceId> {
    decisions
        .iter()
        .filter_map(|d| match d {
            Decision::Start { voice, .. } => Some(*voice),
            Decision::Stop { .. } => None,
        })
        .collect()
}

pub(super) fn stopped(decisions: &[Decision]) -> Vec<VoiceId> {
    decisions
        .iter()
        .filter_map(|d| match d {
            Decision::Stop { voice } => Some(*voice),
            Decision::Start { .. } => None,
        })
        .collect()
}

/// Un cue di priorità `priority` le cui voci non scadono durante il test (un fotogramma al
/// secondo).
pub(super) fn cue(priority: Priority) -> CueData {
    CueData {
        priority,
        max_instances: 16,
        length_s: 100.0,
        ..cue_data()
    }
}

#[test]
fn a_cue_waits_its_cooldown_in_real_time() {
    let (c, mut m) = mixer(
        vec![(
            "shot.ember",
            CueData {
                cooldown_ms: 60,
                ..cue_data()
            },
        )],
        wide(),
    );
    let shot = [req(&c, "shot.ember")];
    assert_eq!(started(&m.decide(0.0, &shot, None)).len(), 1);
    assert!(started(&m.decide(0.03, &shot, None)).is_empty());
    assert!(started(&m.decide(0.059, &shot, None)).is_empty());
    assert_eq!(started(&m.decide(0.06, &shot, None)).len(), 1);
    assert_eq!(m.stats().cooldown, 2);
    assert_eq!(m.stats().started, 2);
}

#[test]
fn max_instances_drops_or_restarts_the_oldest() {
    let capped = |on_limit| CueData {
        max_instances: 2,
        on_limit,
        ..cue_data()
    };
    let (c, mut m) = mixer(vec![("hit", capped(OnLimit::Drop))], wide());
    let hit = [req(&c, "hit")];
    for frame in 0..3 {
        m.decide(f64::from(frame) * 0.1, &hit, None);
    }
    assert_eq!(m.voices().len(), 2);
    assert_eq!(m.stats().instances, 1);

    let (c, mut m) = mixer(vec![("hit", capped(OnLimit::Restart))], wide());
    let hit = [req(&c, "hit")];
    let first = started(&m.decide(0.0, &hit, None))[0];
    m.decide(0.1, &hit, None);
    let third = m.decide(0.2, &hit, None);
    assert_eq!(stopped(&third), [first], "si ferma la più vecchia");
    assert_eq!(started(&third).len(), 1);
    assert!(matches!(third[0], Decision::Stop { .. }), "prima lo stop");
    assert_eq!(m.voices().len(), 2);
    assert_eq!(m.stats().restarted, 1);
}

#[test]
fn critical_cues_have_reserved_seats() {
    let (c, mut m) = mixer(
        vec![
            ("normal", cue(Priority::Normal)),
            ("critical", cue(Priority::Critical)),
        ],
        limits(4, 1, 8, 1000.0),
    );
    let normal = [req(&c, "normal")];
    let critical = [req(&c, "critical")];
    for frame in 0..4 {
        m.decide(f64::from(frame), &normal, None);
    }
    // 4 posti, 1 riservato: la quarta Normal resta fuori (nessuna più bassa da rubare)
    assert_eq!(m.voices().len(), 3);
    assert_eq!(m.stats().seats, 1);
    let first_critical = m.decide(4.0, &critical, None);
    assert_eq!(started(&first_critical).len(), 1);
    assert!(
        stopped(&first_critical).is_empty(),
        "il posto riservato era libero"
    );
    // posti finiti anche per le Critical: ruba la Normal più vecchia
    let second_critical = m.decide(5.0, &critical, None);
    assert_eq!(stopped(&second_critical), [VoiceId(0)]);
    assert_eq!(started(&second_critical).len(), 1);
    assert_eq!(m.voices().len(), 4);
}

#[test]
fn stealing_takes_the_lowest_and_oldest_never_an_equal_or_higher_one() {
    let (c, mut m) = mixer(
        vec![
            ("low", cue(Priority::Low)),
            ("normal", cue(Priority::Normal)),
            ("high", cue(Priority::High)),
        ],
        limits(3, 0, 8, 1000.0),
    );
    let normal = started(&m.decide(0.0, &[req(&c, "normal")], None))[0];
    let low_old = started(&m.decide(1.0, &[req(&c, "low")], None))[0];
    let low_new = started(&m.decide(2.0, &[req(&c, "low")], None))[0];
    // pieno: una Low non ruba a nessuno (le altre sono pari o più alte)
    assert!(m.decide(3.0, &[req(&c, "low")], None).is_empty());
    let high = [req(&c, "high")];
    assert_eq!(stopped(&m.decide(4.0, &high, None)), [low_old]);
    assert_eq!(stopped(&m.decide(5.0, &high, None)), [low_new]);
    assert_eq!(stopped(&m.decide(6.0, &high, None)), [normal]);
    // tutte High: la quarta High si scarta, nessun furto tra pari
    assert!(m.decide(7.0, &high, None).is_empty());
    assert_eq!(m.stats().stolen, 3);
    assert_eq!(m.stats().seats, 2);
    assert!(m.voices().iter().all(|v| v.priority == Priority::High));
}

#[test]
fn requests_of_a_frame_become_one_voice_with_a_bounded_boost() {
    let boosted = CueData {
        coalesce: Coalesce::Boost { max_db: 3.0 },
        ..cue_data()
    };
    let (c, mut m) = mixer(vec![("die", boosted), ("single", cue_data())], wide());
    let ten = vec![req(&c, "die"); 10];
    let decisions = m.decide(0.0, &ten, None);
    assert_eq!(decisions.len(), 1, "{decisions:?}");
    let Decision::Start { gain, .. } = decisions[0] else {
        panic!("attesa una voce: {decisions:?}");
    };
    let max = cue_data().gain * db_to_gain(3.0);
    assert!(gain <= max * 1.0001 && gain > cue_data().gain, "{gain}");
    assert_eq!(m.stats().requests, 10);

    // Single: una voce al guadagno del cue, anche con la quantità in una richiesta sola
    let many = [CueRequest {
        count: 50,
        ..req(&c, "single")
    }];
    let decisions = m.decide(1.0, &many, None);
    assert!(matches!(decisions[..], [Decision::Start { gain, .. }] if gain == cue_data().gain));

    // 1,5 dB per raddoppio, fino al massimo
    assert_eq!(boost_db(1, 3.0), 0.0);
    assert!((boost_db(2, 3.0) - 1.5).abs() < 1e-6);
    assert!((boost_db(4, 6.0) - 3.0).abs() < 1e-6);
    assert_eq!(boost_db(1000, 3.0), 3.0);
    assert_eq!(boost_db(0, 3.0), 0.0);
}

#[test]
fn only_starts_per_frame_voices_start_by_priority_then_id() {
    let (c, mut m) = mixer(
        vec![
            ("a.low", cue(Priority::Low)),
            ("b.normal", cue(Priority::Normal)),
            ("c.high", cue(Priority::High)),
            ("d.critical", cue(Priority::Critical)),
            ("e.normal", cue(Priority::Normal)),
        ],
        limits(64, 0, 2, 1000.0),
    );
    let started_cues = |decisions: Vec<Decision>| -> Vec<String> {
        decisions
            .iter()
            .filter_map(|d| match d {
                Decision::Start { cue, .. } => Some(c.cue(*cue).unwrap().name.clone()),
                Decision::Stop { .. } => None,
            })
            .collect()
    };
    let all: Vec<CueRequest> = ["e.normal", "a.low", "b.normal", "c.high", "d.critical"]
        .iter()
        .map(|n| req(&c, n))
        .collect();
    // la Critical non consuma il budget: partono lei e due altre
    assert_eq!(
        started_cues(m.decide(0.0, &all, None)),
        ["d.critical", "c.high", "b.normal"]
    );
    assert_eq!(m.stats().frame_budget, 2);
    let normals: Vec<CueRequest> = ["e.normal", "a.low", "b.normal"]
        .iter()
        .map(|n| req(&c, n))
        .collect();
    assert_eq!(
        started_cues(m.decide(1.0, &normals, None)),
        ["b.normal", "e.normal"]
    );
    assert_eq!(m.stats().frame_budget, 3);
}

#[test]
fn critical_cues_all_start_even_beyond_the_frame_budget() {
    let names = ["a", "b", "c", "d", "e"];
    let cues = names
        .iter()
        .map(|&n| (n, cue(Priority::Critical)))
        .collect();
    let catalog = catalog_with(cues);
    // limiti del web del catalogo di prova: 4 partenze per fotogramma
    let mut m = Mixer::with(&catalog, Limits::web(&catalog.voices), AUDIO_SEED);
    assert_eq!(m.limits().starts_per_frame, 4);
    let all: Vec<CueRequest> = names.iter().map(|n| req(&catalog, n)).collect();
    assert_eq!(started(&m.decide(0.0, &all, None)).len(), 5);
    assert_eq!(m.stats().frame_budget, 0);
}

/// Riempie un mixer a 24 posti (3 riservati) con `critical` voci `Critical` e `low` voci `Low`,
/// poi chiede una `Normal`: gli Stop della sua partenza.
fn seat_stops(critical: usize, low: usize) -> (Vec<VoiceId>, Mixer) {
    let (c, mut m) = mixer(
        vec![
            (
                "critical",
                CueData {
                    max_instances: 24,
                    ..cue(Priority::Critical)
                },
            ),
            (
                "low",
                CueData {
                    max_instances: 24,
                    ..cue(Priority::Low)
                },
            ),
            ("normal", cue(Priority::Normal)),
        ],
        limits(24, 3, 64, 1000.0),
    );
    let mut now = 0.0;
    for (name, n) in [("critical", critical), ("low", low)] {
        for _ in 0..n {
            now += 0.01;
            assert_eq!(started(&m.decide(now, &[req(&c, name)], None)).len(), 1);
        }
    }
    assert_eq!(m.voices().len(), 24);
    let decisions = m.decide(now + 0.01, &[req(&c, "normal")], None);
    assert_eq!(started(&decisions).len(), 1, "{decisions:?}");
    (stopped(&decisions), m)
}

#[test]
fn a_start_steals_at_most_one_seat() {
    // 21 Low + 3 Critical: posti delle non Critical (21) e totale (24) pieni
    let (stops, m) = seat_stops(3, 21);
    assert_eq!(stops, [VoiceId(3)], "la Low più vecchia");
    assert_eq!(m.voices().non_critical(), 21);
    // 19 Low + 5 Critical: le Critical oltre la riserva tolgono posti al totale, non due furti
    let (stops, m) = seat_stops(5, 19);
    assert_eq!(stops, [VoiceId(5)]);
    assert_eq!((m.voices().len(), m.voices().non_critical()), (24, 19));
    assert_eq!(m.stats().stolen, 1);
}

#[test]
fn view_culled_cues_out_of_view_are_dropped() {
    let (c, mut m) = mixer(
        vec![
            (
                "die",
                CueData {
                    cull: Cull::View,
                    ..cue_data()
                },
            ),
            ("always", cue_data()),
        ],
        wide(),
    );
    let view = Some(Rect::new(-10.0, -10.0, 10.0, 10.0));
    let at = |name, x: f32| CueRequest {
        at: Some(Vec2::new(x, 0.0)),
        ..req(&c, name)
    };
    assert!(m.decide(0.0, &[at("die", 100.0)], view).is_empty());
    assert_eq!(m.stats().culled, 1);
    assert_eq!(
        started(&m.decide(1.0, &[at("always", 100.0)], view)).len(),
        1
    );
    assert_eq!(started(&m.decide(2.0, &[at("die", 5.0)], view)).len(), 1);
    // senza vista (menu) o senza posizione non si scarta
    assert_eq!(started(&m.decide(3.0, &[at("die", 100.0)], None)).len(), 1);
    assert_eq!(started(&m.decide(4.0, &[req(&c, "die")], view)).len(), 1);
    // nel gruppo contano solo le richieste in vista
    let mixed = [at("die", 100.0), at("die", 0.0)];
    assert_eq!(started(&m.decide(5.0, &mixed, view)).len(), 1);
    assert_eq!(m.stats().culled, 1);
}
