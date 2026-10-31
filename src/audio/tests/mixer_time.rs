//! Tempo e caso del mixer: cooldown a credito e indipendenza dal frame rate, scadenze, varianti
//! (progetto § 3.5, § 3.6, § 12 punti 8 e 10).

use super::catalog_with;
use super::cue_data;
use super::mixer::{limits, mixer, req, started, wide};
use crate::audio::catalog::{Coalesce, CueData, Priority};
use crate::audio::cues::CueRequest;
use crate::audio::mixer::*;
use crate::rng::Pcg32;

/// Cooldown del test dei frame rate (ms), dal più breve sopra un fotogramma a 30 FPS in su.
const COOLDOWNS_MS: [u32; 11] = [35, 40, 45, 50, 60, 80, 120, 150, 200, 250, 300];
const FRAME_RATES: [f64; 3] = [30.0, 60.0, 144.0];

/// Avvii al secondo di 100 eventi/s per 10 s di tempo reale, raggruppati in fotogrammi a `fps`,
/// con un cue di cooldown `cooldown_ms`; la riproduzione finisce ogni voce 0,12 s dopo l'avvio.
/// Controlla anche l'intervallo minimo tra due avvii: `cooldown - fotogramma`, mai meno di un
/// fotogramma.
fn starts_per_second(cooldown_ms: u32, fps: f64) -> f64 {
    let (c, mut m) = mixer(
        vec![(
            "enemy.hit",
            CueData {
                priority: Priority::Low,
                cooldown_ms,
                max_instances: 8,
                coalesce: Coalesce::Boost { max_db: 3.0 },
                length_s: 0.15,
                ..cue_data()
            },
        )],
        limits(24, 3, 6, 100.0),
    );
    let hit = c.exact("enemy.hit").unwrap();
    let (events, seconds) = (1000_u32, 10.0);
    let cooldown = f64::from(cooldown_ms) / 1000.0;
    let frame = 1.0 / fps;
    let mut next_event = 0;
    let mut playing: Vec<(VoiceId, f64)> = Vec::new();
    let mut starts: Vec<f64> = Vec::new();
    for index in 1..=(seconds * fps).ceil() as u32 {
        let now = f64::from(index) / fps;
        playing.retain(|&(voice, end)| {
            if end > now {
                return true;
            }
            m.finished(voice);
            false
        });
        let mut count = 0;
        while next_event < events && f64::from(next_event) * 0.01 <= now {
            count += 1;
            next_event += 1;
        }
        let requests: Vec<CueRequest> = (count > 0)
            .then_some(CueRequest {
                cue: hit,
                count,
                at: None,
            })
            .into_iter()
            .collect();
        for voice in started(&m.decide(now, &requests, None)) {
            m.playing(voice);
            playing.push((voice, now + 0.12));
            starts.push(now);
        }
    }
    let shortest = (cooldown - frame.min(cooldown)).max(frame);
    for pair in starts.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(
            gap >= shortest - 1e-9,
            "{cooldown_ms} ms a {fps} FPS: avvii a {gap} s, minimo {shortest} s"
        );
    }
    let stats = m.stats();
    assert!(stats.instances == 0 && stats.seats == 0 && stats.headroom == 0);
    starts.len() as f64 / seconds
}

#[test]
fn the_same_events_start_the_same_voices_per_second_at_any_frame_rate() {
    for cooldown_ms in COOLDOWNS_MS {
        let rates: Vec<f64> = FRAME_RATES
            .iter()
            .map(|&fps| starts_per_second(cooldown_ms, fps))
            .collect();
        let (min, max) = rates
            .iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), &r| (lo.min(r), hi.max(r)));
        let spread = (max - min) / min;
        println!(
            "cooldown {cooldown_ms} ms: avvii al secondo a 30, 60 e 144 FPS {rates:?}, \
             scarto {:.1}%",
            spread * 100.0
        );
        // il cooldown è un intervallo medio: con 100 eventi/s gli avvii sono ~1/cooldown
        let expected = (1000.0 / f64::from(cooldown_ms)).min(100.0);
        assert!(min > expected * 0.85, "{cooldown_ms} ms: {rates:?}");
        assert!(
            spread <= 0.10,
            "cooldown {cooldown_ms} ms, avvii al secondo a 30/60/144 FPS: {rates:?}"
        );
    }
}

#[test]
fn the_cooldown_credit_keeps_the_mean_and_bounds_the_shortest_gap() {
    // fotogrammi irregolari da 1 a 20 ms, una richiesta a ogni fotogramma
    let cooldown_ms = 45;
    let (c, mut m) = mixer(
        vec![(
            "hit",
            CueData {
                cooldown_ms,
                max_instances: 64,
                length_s: 0.05,
                ..cue_data()
            },
        )],
        wide(),
    );
    let cooldown = f64::from(cooldown_ms) / 1000.0;
    let mut rng = Pcg32::new(11, 1);
    let (mut now, mut frame_dt) = (0.0, 0.0);
    let mut starts: Vec<(f64, f64)> = Vec::new();
    while now < 30.0 {
        if !started(&m.decide(now, &[req(&c, "hit")], None)).is_empty() {
            starts.push((now, frame_dt));
        }
        frame_dt = f64::from(rng.range_f32(0.001, 0.020));
        now += frame_dt;
    }
    for pair in starts.windows(2) {
        let ((earlier, lag), (later, _)) = (pair[0], pair[1]);
        let shortest = cooldown - lag.min(cooldown);
        assert!(later - earlier >= shortest - 1e-9, "{earlier} → {later}");
        assert!(later > earlier, "due avvii nello stesso fotogramma");
    }
    let mean = (starts.last().unwrap().0 - starts[0].0) / (starts.len() - 1) as f64;
    assert!(
        (mean - cooldown).abs() < cooldown * 0.01,
        "intervallo medio {mean} s"
    );

    // il credito non si accumula nelle pause: dopo un secondo di fotogrammi senza richieste
    // riparte una voce e la seconda aspetta quasi tutto il cooldown
    for _ in 0..100 {
        now += 0.01;
        m.decide(now, &[], None);
    }
    let first = started(&m.decide(now + 0.01, &[req(&c, "hit")], None));
    let again = started(&m.decide(now + 0.02, &[req(&c, "hit")], None));
    assert_eq!((first.len(), again.len()), (1, 0));
}

#[test]
fn below_a_frame_the_limit_is_one_start_per_frame() {
    // 20 ms di cooldown a 30 FPS: un avvio a ogni fotogramma, non di più
    let (c, mut m) = mixer(
        vec![(
            "hit",
            CueData {
                cooldown_ms: 20,
                max_instances: 64,
                ..cue_data()
            },
        )],
        wide(),
    );
    let three = [req(&c, "hit"), req(&c, "hit"), req(&c, "hit")];
    let starts: usize = (1..=30)
        .map(|frame| started(&m.decide(f64::from(frame) / 30.0, &three, None)).len())
        .sum();
    assert_eq!(starts, 30);
}

#[test]
fn the_next_allowed_instant_recovers_only_the_last_frame() {
    // primo avvio: un cooldown pieno
    assert_eq!(next_allowed(None, 1.0, 0.05, 0.02), 1.05);
    // avvio in ritardo di meno di un fotogramma: si recupera (resta il passo di 50 ms)
    assert!((next_allowed(Some(1.05), 1.06, 0.05, 0.02) - 1.10).abs() < 1e-12);
    // dopo una pausa lunga il credito non vale più di un fotogramma
    assert!((next_allowed(Some(1.05), 3.0, 0.05, 0.02) - 3.03).abs() < 1e-12);
    // il fotogramma vale al più un cooldown
    assert!((next_allowed(Some(1.05), 3.0, 0.05, 0.5) - 3.0).abs() < 1e-12);
}

#[test]
fn only_voices_that_never_played_expire() {
    let (c, mut m) = mixer(vec![("hit", cue_data())], wide());
    let voice = started(&m.decide(10.0, &[req(&c, "hit")], None))[0];
    let length = f64::from(cue_data().length_s);
    assert!(m.decide(10.0 + length + 0.49, &[], None).is_empty());
    assert_eq!(
        m.decide(10.0 + length + EXPIRY_GRACE_S, &[], None),
        [Decision::Stop { voice }]
    );
    assert!(m.voices().is_empty());
    assert_eq!(m.stats().expired, 1);
    assert!(!m.playing(voice), "una voce scaduta non c'è più");

    // una voce che suona non scade: finisce solo quando la riproduzione lo dice
    let voice = started(&m.decide(20.0, &[req(&c, "hit")], None))[0];
    assert!(m.playing(voice));
    assert!(m.voices().get(voice).is_some_and(|v| v.playing));
    assert!(m.decide(20.0 + length * 10.0, &[], None).is_empty());
    assert_eq!(m.voices().len(), 1);
    assert!(m.finished(voice));
    assert!(!m.finished(voice));
    assert!(m.voices().is_empty());
    assert_eq!((m.stats().expired, m.stats().finished), (1, 1));
}

#[test]
fn unknown_or_empty_requests_change_nothing() {
    let (c, mut m) = mixer(vec![("hit", cue_data())], wide());
    let unknown = CueRequest {
        cue: crate::audio::CueId(999),
        count: 1,
        at: None,
    };
    let empty = CueRequest {
        count: 0,
        ..req(&c, "hit")
    };
    assert!(m.decide(0.0, &[unknown, empty], None).is_empty());
    assert_eq!(m.stats().unknown, 1);
    assert_eq!(m.stats().requests, 0);
}

/// Varianti di 1000 avvii di un cue con `variants` file (ogni voce finisce subito).
fn variants(seed: u64, variants: usize) -> Vec<usize> {
    let files = (0..variants)
        .map(|i| format!("audio/sfx/test/v{i}.wav"))
        .collect();
    let catalog = catalog_with(vec![(
        "hit",
        CueData {
            files,
            ..cue_data()
        },
    )]);
    let mut m = Mixer::with(&catalog, wide(), seed);
    let hit = [req(&catalog, "hit")];
    (0..1000)
        .map(|frame| match m.decide(f64::from(frame), &hit, None)[..] {
            [Decision::Start { voice, variant, .. }] => {
                m.finished(voice);
                variant
            }
            ref other => panic!("atteso un avvio: {other:?}"),
        })
        .collect()
}

#[test]
fn a_variant_never_plays_twice_in_a_row_and_the_seed_fixes_the_sequence() {
    let picks = variants(AUDIO_SEED, 3);
    assert!(picks.windows(2).all(|w| w[0] != w[1]), "variante ripetuta");
    for variant in 0..3 {
        let n = picks.iter().filter(|&&v| v == variant).count();
        assert!(n > 250, "variante {variant}: {n} su 1000");
    }
    assert_eq!(variants(AUDIO_SEED, 3), picks);
    assert_ne!(variants(AUDIO_SEED + 1, 3), picks);
    assert!(variants(AUDIO_SEED, 1).iter().all(|&v| v == 0));
    let two = variants(AUDIO_SEED, 2);
    assert!(two.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn speed_and_gain_vary_inside_the_catalog_ranges() {
    let (c, mut m) = mixer(
        vec![(
            "hit",
            CueData {
                pitch: (0.9, 1.1),
                gain_jitter_db: 1.5,
                ..cue_data()
            },
        )],
        wide(),
    );
    let (low, high) = (0.5 * db_to_gain(-1.5), 0.5 * db_to_gain(1.5));
    let mut speeds = Vec::new();
    for frame in 0..200 {
        for decision in m.decide(f64::from(frame), &[req(&c, "hit")], None) {
            if let Decision::Start {
                voice, gain, speed, ..
            } = decision
            {
                assert!((0.9..=1.1).contains(&speed), "{speed}");
                assert!(gain >= low * 0.9999 && gain <= high * 1.0001, "{gain}");
                speeds.push(speed);
                m.finished(voice);
            }
        }
    }
    assert_eq!(speeds.len(), 200);
    assert!(
        speeds.windows(2).any(|w| w[0] != w[1]),
        "nessuna variazione"
    );
}

#[test]
fn the_variant_picker_alone() {
    let mut rng = Pcg32::new(1, AUDIO_STREAM);
    let mut one = VariantPicker::default();
    assert!((0..10).all(|_| one.pick(1, &mut rng) == 0));
    let mut two = VariantPicker::default();
    let first = two.pick(2, &mut rng);
    let picks: Vec<usize> = (0..10).map(|_| two.pick(2, &mut rng)).collect();
    let expected: Vec<usize> = (1..=10).map(|i| (first + i) % 2).collect();
    assert_eq!(picks, expected, "con due varianti si alternano");
    // un cue senza file non manda in panic (il catalogo lo vieta comunque)
    assert_eq!(VariantPicker::default().pick(0, &mut rng), 0);
}
