//! Le curve equal-power e i due deck senza Bevy (progetto § 3.7): potenza costante, attesa del
//! sink, rientro, terza traccia, al più due deck e nessun loop doppio.

use crate::audio::catalog::TrackId;
use crate::audio::music::*;

const FRAME: f32 = 1.0 / 60.0;

#[test]
fn the_curves_are_equal_power() {
    assert_eq!((fade_in(0.0), fade_in(1.0)), (0.0, 1.0));
    assert_eq!((fade_out(0.0), fade_out(1.0)), (1.0, 0.0));
    assert_eq!((fade_in(-1.0), fade_in(2.0)), (0.0, 1.0), "fuori da [0, 1]");
    let mut last = (0.0, 1.0);
    for step in 0..=1000 {
        let t = step as f32 / 1000.0;
        let (rise, fall) = (fade_in(t), fade_out(t));
        let angle = t * std::f32::consts::FRAC_PI_2;
        assert!((rise - angle.sin()).abs() < 1e-6, "{t}");
        assert!((fall - angle.cos()).abs() < 1e-6, "{t}");
        assert!((rise * rise + fall * fall - 1.0).abs() < 1e-5, "{t}");
        assert!(rise >= last.0 && fall <= last.1, "{t}");
        last = (rise, fall);
    }
    // a metà, −3 dB per lato (non −6 dB come una dissolvenza lineare)
    assert!((fade_in(0.5) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
}

const HOME: TrackId = TrackId(0);
const RUN: TrackId = TrackId(1);
const OTHER: TrackId = TrackId(2);

/// Un deck pieno che suona già.
fn playing(track: TrackId) -> Decks {
    let mut decks = Decks::default();
    decks.advance(Some(track), 0.0, FRAME);
    decks.set_sounding(Slot::A);
    decks.advance(Some(track), 0.0, FRAME);
    assert_eq!(decks.get(Slot::A).unwrap().level(), 1.0);
    decks
}

fn level(decks: &Decks, track: TrackId) -> f32 {
    decks.find(track).map_or(0.0, Deck::level)
}

#[test]
fn a_crossfade_keeps_the_power_and_ends_with_one_deck() {
    let mut decks = playing(HOME);
    let events = decks.advance(Some(RUN), 1.5, FRAME);
    assert_eq!(
        events,
        [DeckEvent::Spawn {
            slot: Slot::B,
            track: RUN
        }]
    );
    decks.set_sounding(Slot::B);
    let mut frames = 0;
    let mut seen = Vec::new();
    while decks.len() > 1 || level(&decks, RUN) < 1.0 {
        seen.extend(decks.advance(Some(RUN), 1.5, FRAME));
        frames += 1;
        let (fall, rise) = (level(&decks, HOME), level(&decks, RUN));
        assert!(
            (fall * fall + rise * rise - 1.0).abs() < 1e-4,
            "{frames}: {fall} {rise}"
        );
        assert!(frames < 200, "la dissolvenza non finisce");
    }
    // 1,5 s a 60 FPS: 90 fotogrammi (uno in più per l'arrotondamento)
    assert!((90..=91).contains(&frames), "{frames}");
    assert!(seen.contains(&DeckEvent::Full {
        slot: Slot::B,
        track: RUN
    }));
    assert!(seen.contains(&DeckEvent::Despawn {
        slot: Slot::A,
        track: HOME
    }));
    assert_eq!(decks.iter().map(|d| d.track).collect::<Vec<_>>(), [RUN]);
}

#[test]
fn the_crossfade_waits_for_the_sink_and_does_not_skip() {
    let mut decks = playing(HOME);
    decks.advance(Some(RUN), 1.5, FRAME);
    // caricamento lento: cinque secondi senza sink, niente si muove
    for _ in 0..300 {
        assert!(decks.advance(Some(RUN), 1.5, FRAME).is_empty());
        assert!(decks.waiting(Some(RUN)));
        assert_eq!((level(&decks, HOME), level(&decks, RUN)), (1.0, 0.0));
    }
    decks.set_sounding(Slot::B);
    decks.advance(Some(RUN), 1.5, FRAME);
    let first = level(&decks, RUN);
    // il primo passo della curva, non un salto
    assert!((first - fade_in(FRAME / 1.5)).abs() < 1e-6, "{first}");
    assert!(level(&decks, HOME) < 1.0);
    // verso il silenzio non si aspetta nessuno
    let mut silent = playing(HOME);
    silent.advance(None, 1.5, FRAME);
    assert!(level(&silent, HOME) < 1.0 && !silent.waiting(None));
}

#[test]
fn a_deck_that_comes_back_reenters_from_its_level_without_a_new_deck() {
    let mut decks = playing(HOME);
    for _ in 0..45 {
        decks.advance(None, 1.5, FRAME);
    }
    let low = level(&decks, HOME);
    assert!(low > 0.5 && low < 0.8, "{low}");
    let events = decks.advance(Some(HOME), 1.5, FRAME);
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(decks.len(), 1);
    let back = level(&decks, HOME);
    assert!(back > low && back - low < 0.05, "{low} → {back}");
    // tra due tracce: chi entrava torna a uscire dal suo livello, chi usciva rientra
    let mut both = playing(HOME);
    both.advance(Some(RUN), 1.5, FRAME);
    both.set_sounding(Slot::B);
    for _ in 0..30 {
        both.advance(Some(RUN), 1.5, FRAME);
    }
    let (home, run) = (level(&both, HOME), level(&both, RUN));
    assert!(both.advance(Some(HOME), 1.5, FRAME).is_empty());
    assert!(level(&both, HOME) > home && level(&both, RUN) < run);
    assert_eq!(both.len(), 2);
}

#[test]
fn a_third_track_waits_for_a_free_deck() {
    let mut decks = playing(HOME);
    decks.advance(Some(RUN), 1.5, FRAME);
    decks.set_sounding(Slot::B);
    for _ in 0..60 {
        decks.advance(Some(RUN), 1.5, FRAME);
    }
    assert!(level(&decks, HOME) < level(&decks, RUN));
    let mut spawned = None;
    for frame in 0..200 {
        assert!(decks.len() <= MAX_DECKS);
        for event in decks.advance(Some(OTHER), 1.5, FRAME) {
            if let DeckEvent::Spawn { slot, track } = event {
                assert_eq!(track, OTHER);
                spawned = Some((frame, slot));
            }
        }
        if spawned.is_some() {
            break;
        }
    }
    // la casa era più bassa: arriva a zero per prima e lascia il suo posto
    let (frame, slot) = spawned.expect("la terza traccia non trova posto");
    assert_eq!(slot, Slot::A);
    assert!(frame > 0, "un deck è stato tolto prima di arrivare a zero");
    assert!(decks.find(HOME).is_none() && decks.find(RUN).is_some());
}

#[test]
fn a_zero_fade_switches_at_once_and_a_deck_never_heard_leaves_at_once() {
    let mut decks = playing(HOME);
    let events = decks.advance(None, 0.0, FRAME);
    assert_eq!(
        events,
        [DeckEvent::Despawn {
            slot: Slot::A,
            track: HOME
        }]
    );
    // senza dispositivo il sink non nasce: chi non ha mai suonato esce subito
    let mut deaf = Decks::default();
    deaf.advance(Some(HOME), 1.5, FRAME);
    let events = deaf.advance(None, 1.5, FRAME);
    assert!(deaf.is_empty(), "{events:?}");
}

/// Un generatore congruenziale per le sequenze di prova (ripetibili).
fn next(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 33
}

#[test]
fn ten_trips_to_the_menu_never_make_a_third_deck_nor_a_double_loop() {
    let tracks = [Some(HOME), Some(RUN), None, Some(OTHER)];
    for seed in 0..50u64 {
        let mut seed = seed;
        let mut decks = Decks::default();
        // dieci entrate e uscite dal menu, con durate da un fotogramma a tre secondi
        for trip in 0..20 {
            let target = if trip % 2 == 0 {
                Some(HOME)
            } else {
                tracks[(next(&mut seed) % 4) as usize]
            };
            let frames = [1, 2, 3, 7, 45, 89, 91, 180][(next(&mut seed) % 8) as usize];
            for _ in 0..frames {
                let events = decks.advance(target, 1.5, FRAME);
                for event in events {
                    if let DeckEvent::Spawn { slot, .. } = event
                        && !next(&mut seed).is_multiple_of(3)
                    {
                        decks.set_sounding(slot); // il sink nasce subito, o un po' dopo
                    }
                }
                if next(&mut seed).is_multiple_of(5) {
                    for slot in Slot::ALL {
                        decks.set_sounding(slot);
                    }
                }
                assert!(decks.len() <= MAX_DECKS, "{decks:?}");
                let mut seen: Vec<TrackId> = decks.iter().map(|d| d.track).collect();
                seen.sort();
                seen.dedup();
                assert_eq!(seen.len(), decks.len(), "loop doppio: {decks:?}");
            }
        }
        // fermi nel menu: resta solo la home, a pieno
        for slot in Slot::ALL {
            decks.set_sounding(slot);
        }
        for _ in 0..400 {
            decks.advance(Some(HOME), 1.5, FRAME);
            for slot in Slot::ALL {
                decks.set_sounding(slot);
            }
        }
        assert_eq!(decks.len(), 1, "{decks:?}");
        assert_eq!(level(&decks, HOME), 1.0);
    }
}
