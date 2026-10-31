//! La riproduzione nel tempo, con il fixture di `tests/playback.rs` (file caricato, nessun sink
//! vero): un fotogramma lunghissimo subito dopo la nascita del sink non fa scadere una voce che
//! suona (i sink si segnano in `AudioSet::Listen`, prima del mixer); senza dispositivo le voci
//! scadono, l'entità sparisce, la voce lascia il mixer e l'avviso esce una volta sola.

use std::time::Duration;

use bevy::audio::AudioPlayer;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::cue_data;
use super::playback::{FakeSink, app, ask, mixer_voices, voices};
use crate::audio::mixer::EXPIRY_GRACE_S;
use crate::audio::*;

/// Un fotogramma più lungo della scadenza di una voce mai suonata del cue di prova.
fn past_expiry() -> Duration {
    Duration::from_secs_f64(f64::from(cue_data().length_s) + EXPIRY_GRACE_S + 0.25)
}

/// Un fotogramma che dura `dt` di tempo reale.
fn frame(app: &mut App, dt: Duration) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
    app.update();
}

#[test]
fn a_long_frame_right_after_the_sink_is_born_does_not_expire_the_voice() {
    let (mut app, _) = app(true);
    ask(&mut app);
    app.update();
    let (entity, voice) = voices(&mut app)[0];
    // Bevy crea il sink nel `PostUpdate` dell'avvio; il fotogramma dopo dura il doppio della
    // scadenza (schermo intero, shader, scheda del browser nascosta e riaperta)
    app.world_mut().entity_mut(entity).insert(FakeSink {
        volume: voice.volume,
        writes: 0,
    });
    frame(&mut app, past_expiry() * 2);
    let live = voices(&mut app);
    assert_eq!(live.len(), 1, "la voce che suona è stata tolta");
    assert!(live[0].1.sounding);
    let mixer = &app.world().resource::<AudioMixer>().0;
    assert!(mixer.voices().get(voice.voice).is_some_and(|v| v.playing));
    assert_eq!(mixer.stats().expired, 0);
    assert_eq!(mixer_voices(&app), 1);
    let stats = app.world().resource::<PlaybackStats>();
    assert_eq!(
        (stats.stopped, stats.silent_expired, stats.silent_warnings),
        (0, 0, 0)
    );
}

#[test]
fn without_a_device_the_voices_expire_leave_the_mixer_and_warn_only_once() {
    let (mut app, _) = app(true);
    // una richiesta a ogni fotogramma, ogni fotogramma oltre la scadenza: ognuno toglie la voce
    // del fotogramma prima (mai un sink) e ne avvia una nuova
    let requests = 10;
    for _ in 0..requests {
        ask(&mut app);
        frame(&mut app, past_expiry());
    }
    let stats = app.world().resource::<PlaybackStats>().clone();
    assert_eq!(stats.spawned, requests);
    assert_eq!(stats.silent_expired, requests - 1);
    assert_eq!(stats.silent_warnings, 1, "un avviso solo");
    assert_eq!(voices(&mut app).len(), 1);
    // senza richieste scade anche l'ultima
    frame(&mut app, past_expiry());
    let world = app.world_mut();
    assert_eq!(world.query::<&AudioPlayer>().iter(world).count(), 0);
    assert!(voices(&mut app).is_empty());
    assert_eq!(mixer_voices(&app), 0);
    let mixer = app.world().resource::<AudioMixer>().0.stats();
    assert_eq!(mixer.expired, requests);
    assert_eq!(
        mixer.finished, 0,
        "le voci scadute erano già fuori dal mixer"
    );
    let stats = app.world().resource::<PlaybackStats>();
    assert_eq!((stats.silent_expired, stats.silent_warnings), (requests, 1));
    assert_eq!(
        stats.silent_expired, mixer.expired,
        "solo le scadenze del mixer"
    );
    assert_eq!(stats.sounding, 0);
}

/// Una voce mai suonata fermata da un `Restart` dopo `length_s` ma prima del suo termine non è
/// scaduta: `silent_expired` conta solo le scadenze del mixer, e nessun avviso.
#[test]
fn a_restart_after_the_length_is_not_an_expiry() {
    let (mut app, _) = app(true);
    ask(&mut app);
    app.update();
    let length = Duration::from_secs_f64(f64::from(cue_data().length_s) + EXPIRY_GRACE_S / 2.0);
    ask(&mut app);
    frame(&mut app, length);
    let mixer = app.world().resource::<AudioMixer>().0.stats();
    assert_eq!((mixer.restarted, mixer.expired), (1, 0));
    let stats = app.world().resource::<PlaybackStats>();
    assert_eq!(stats.stopped, 1);
    assert_eq!((stats.silent_expired, stats.silent_warnings), (0, 0));
    assert_eq!(stats.silent_expired, mixer.expired);
}
