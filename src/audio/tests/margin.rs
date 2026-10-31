//! Margine del mix (progetto § 3.4 e § 12.9): oltre `sfx_headroom` una non `Critical` perde il
//! boost, poi ferma voci di priorità minore finché ci sta, altrimenti si scarta; le `Critical`
//! partono sempre e il loro guadagno conta nella somma.

use super::mixer::{limits, mixer, req, started, stopped, wide};
use super::{cue_data, file_with};
use crate::audio::catalog::{AudioCatalog, Coalesce, CueBus, CueData, OnLimit, Priority};
use crate::audio::cues::CueRequest;
use crate::audio::mixer::*;
use crate::rng::Pcg32;

/// Un cue di guadagno `gain` (nessuna variazione) che non scade durante il test.
fn loud(gain: f32, priority: Priority) -> CueData {
    CueData {
        gain,
        priority,
        max_instances: 16,
        length_s: 100.0,
        ..cue_data()
    }
}

#[test]
fn over_the_headroom_the_boost_goes_first() {
    let (c, mut m) = mixer(
        vec![
            ("low", loud(0.4, Priority::Low)),
            (
                "boom",
                CueData {
                    coalesce: Coalesce::Boost { max_db: 6.0 },
                    ..loud(0.5, Priority::Normal)
                },
            ),
        ],
        limits(64, 0, 64, 1.0),
    );
    m.decide(0.0, &[req(&c, "low")], None);
    let four = [CueRequest {
        count: 4,
        ..req(&c, "boom")
    }];
    // 0,4 + 0,5 × 1,41 (+3 dB) > 1: parte senza boost (0,4 + 0,5 ≤ 1), senza fermare nessuno
    let decisions = m.decide(0.1, &four, None);
    assert!(matches!(decisions[..], [Decision::Start { gain, .. }] if gain == 0.5));
    assert_eq!(m.stats().boost_cut, 1);
    assert_eq!(m.stats().headroom_stolen, 0);
}

#[test]
fn a_voice_over_the_headroom_stops_lower_ones_until_it_fits() {
    let (c, mut m) = mixer(
        vec![
            ("low", loud(0.4, Priority::Low)),
            ("normal", loud(0.5, Priority::Normal)),
        ],
        limits(64, 0, 64, 1.0),
    );
    let low_old = started(&m.decide(0.0, &[req(&c, "low")], None))[0];
    let low_new = started(&m.decide(0.1, &[req(&c, "low")], None))[0];
    // una Low in più supera il tetto e non ha nessuno sotto da fermare
    assert!(m.decide(0.2, &[req(&c, "low")], None).is_empty());
    assert_eq!(m.stats().headroom, 1);
    // 0,8 + 0,5 > 1: si ferma la Low più vecchia
    let first = m.decide(0.3, &[req(&c, "normal")], None);
    assert_eq!(stopped(&first), [low_old]);
    // 0,9 + 0,5 > 1: si ferma l'altra (0,5 + 0,5 = 1 ci sta)
    let second = m.decide(0.4, &[req(&c, "normal")], None);
    assert_eq!(stopped(&second), [low_new]);
    // 1,0 + 0,5 e nessuna voce più bassa: scartata, nessuno fermato
    assert!(m.decide(0.5, &[req(&c, "normal")], None).is_empty());
    assert_eq!(m.stats().headroom, 2);
    assert_eq!(m.stats().headroom_stolen, 2);
    assert!(m.voices().mix_gain(&[]) <= 1.0 + 1e-6);
}

#[test]
fn critical_voices_always_start_and_count_in_the_sum() {
    let (c, mut m) = mixer(
        vec![
            ("normal", loud(0.5, Priority::Normal)),
            ("high", loud(0.3, Priority::High)),
            ("critical", loud(0.8, Priority::Critical)),
        ],
        limits(64, 0, 64, 1.0),
    );
    m.decide(0.0, &[req(&c, "normal")], None);
    m.decide(0.1, &[req(&c, "normal")], None);
    let critical = m.decide(0.2, &[req(&c, "critical")], None);
    assert_eq!((started(&critical).len(), stopped(&critical).len()), (1, 0));
    assert!(
        (m.voices().mix_gain(&[]) - 1.8).abs() < 1e-6,
        "oltre il tetto solo per lei"
    );
    // una High dovrebbe fermare anche la Critical per starci: scartata, nessuno fermato
    assert!(m.decide(0.3, &[req(&c, "high")], None).is_empty());
    assert_eq!(m.voices().len(), 3);
    // un'altra Critical parte comunque
    assert_eq!(
        started(&m.decide(0.4, &[req(&c, "critical")], None)).len(),
        1
    );
}

#[test]
fn the_headroom_counts_the_bus_trim() {
    let mut file = file_with(vec![(
        "ui.tick",
        CueData {
            bus: CueBus::Ui,
            gain: 0.8,
            ..cue_data()
        },
    )]);
    file.trims.ui = 0.5;
    let catalog = AudioCatalog::compile(file).unwrap();
    let mut m = Mixer::with(&catalog, wide(), AUDIO_SEED);
    let decisions = m.decide(0.0, &[req(&catalog, "ui.tick")], None);
    // la decisione porta il guadagno del cue; il trim lo mette la riproduzione
    assert!(matches!(decisions[..], [Decision::Start { gain, .. }] if gain == 0.8));
    assert!((m.voices().mix_gain(&[]) - 0.4).abs() < 1e-6);
}

#[test]
fn a_dropped_request_stops_nothing() {
    let (c, mut m) = mixer(
        vec![
            (
                "low",
                CueData {
                    max_instances: 1,
                    on_limit: OnLimit::Restart,
                    ..loud(0.6, Priority::Low)
                },
            ),
            ("critical", loud(0.5, Priority::Critical)),
        ],
        limits(64, 0, 64, 1.0),
    );
    m.decide(0.0, &[req(&c, "low")], None);
    m.decide(0.1, &[req(&c, "critical")], None);
    // Restart toglierebbe la Low vecchia, ma 0,5 + 0,6 supera il margine e sotto una Low non c'è
    // nulla da fermare: scartata, e la Low vecchia resta
    assert!(m.decide(0.2, &[req(&c, "low")], None).is_empty());
    assert_eq!(m.voices().len(), 2);
    assert_eq!(m.stats().restarted, 0);
    assert_eq!(m.stats().headroom, 1);
}

/// Dodici cue di ogni priorità, guadagno, cooldown, istanze, coalescenza e durata.
fn random_cues(rng: &mut Pcg32) -> Vec<(String, CueData)> {
    let priorities = [
        Priority::Low,
        Priority::Normal,
        Priority::High,
        Priority::Critical,
    ];
    (0..12)
        .map(|i| {
            let data = CueData {
                gain: rng.range_f32(0.1, 0.6),
                gain_jitter_db: 1.5,
                priority: priorities[i % 4],
                cooldown_ms: rng.next_u32() % 90,
                max_instances: 1 + rng.next_u32() % 4,
                on_limit: if i % 3 == 0 {
                    OnLimit::Restart
                } else {
                    OnLimit::Drop
                },
                coalesce: if i % 2 == 0 {
                    Coalesce::Boost { max_db: 4.0 }
                } else {
                    Coalesce::Single
                },
                length_s: rng.range_f32(0.1, 1.0),
                ..cue_data()
            };
            (format!("cue_{i}"), data)
        })
        .collect()
}

#[test]
fn only_critical_voices_ever_push_the_sum_over_the_headroom() {
    let mut rng = Pcg32::new(2026, 7);
    let cues = random_cues(&mut rng);
    let catalog = super::catalog_with(cues.iter().map(|(n, d)| (n.as_str(), d.clone())).collect());
    let limits = limits(12, 3, 4, 3.0);
    let mut m = Mixer::with(&catalog, limits, AUDIO_SEED);
    let critical =
        |cue: crate::audio::CueId| catalog.cue(cue).unwrap().data.priority == Priority::Critical;
    // voci avviate: (voce, quando suona, quando finisce); il 30% non suona mai (scade)
    let mut sounding: Vec<(VoiceId, f64, f64)> = Vec::new();
    let (mut now, mut over_by_critical, mut starts) = (0.0, 0, 0);
    for _ in 0..20_000 {
        now += f64::from(rng.range_f32(0.005, 0.040));
        sounding.retain(|&(voice, plays, ends)| {
            if now >= plays {
                m.playing(voice);
            }
            if now < ends {
                return true;
            }
            m.finished(voice);
            false
        });
        let mut requests: Vec<CueRequest> = Vec::new();
        for i in 0..catalog.cues().len() {
            if rng.next_f32() < 0.7 {
                requests.push(CueRequest {
                    cue: crate::audio::CueId(i as u16),
                    count: 1 + rng.next_u32() % 12,
                    at: None,
                });
            }
        }
        let decisions = m.decide(now, &requests, None);
        let mut non_critical_started = 0;
        for decision in &decisions {
            if let Decision::Start { voice, cue, .. } = *decision {
                starts += 1;
                if !critical(cue) {
                    non_critical_started += 1;
                }
                let length = f64::from(catalog.cue(cue).unwrap().data.length_s);
                if rng.next_f32() < 0.7 {
                    sounding.push((voice, now + 0.02, now + length * 0.8));
                }
            }
        }
        let voices = m.voices();
        let total = voices.mix_gain(&[]);
        assert!(voices.non_critical_mix_gain() <= limits.sfx_headroom + 1e-4);
        assert!(voices.len() <= limits.max_voices);
        assert!(voices.non_critical() <= limits.non_critical_seats());
        assert!(non_critical_started <= limits.starts_per_frame);
        if non_critical_started > 0 {
            assert!(total <= limits.sfx_headroom + 1e-4, "{total} a {now} s");
        }
        if total > limits.sfx_headroom + 1e-4 {
            assert!(voices.iter().any(|v| v.priority == Priority::Critical));
            over_by_critical += 1;
        }
    }
    let stats = m.stats();
    println!(
        "flusso casuale: {starts} avvii, {over_by_critical} fotogrammi oltre il tetto; {stats:?}"
    );
    // il flusso passa davvero dalle regole del margine e dei posti
    assert!(starts > 5_000);
    assert!(stats.headroom > 0 && stats.headroom_stolen > 0 && stats.boost_cut > 0);
    assert!(stats.expired > 0 && stats.finished > 0 && stats.frame_budget > 0);
    assert!(stats.seats > 0 && stats.stolen > 0 && over_by_critical > 0);
}
