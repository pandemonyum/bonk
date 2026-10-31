//! Criterio § 12.1: dopo 10 s di gioco simulato `headless_app` non ha nulla dell'audio. Due
//! controlli, provati prima su un'app che l'audio ce l'ha tutto (non passano a vuoto):
//! - i pezzi per nome ([`pieces`]): risorse e componenti del client e di `bevy_audio`;
//! - le risorse per `TypeId`: tutte quelle che `GameAudioPlugin` aggiunge a un'app minima,
//!   anche quelle che arriveranno dopo AUDIO-2 senza che qualcuno allunghi l'elenco.
//!
//! `AudioSink` e `SpatialAudioSink` non sono nell'elenco: senza dispositivo non si possono creare
//! e Bevy li mette solo su un'entità che ha già `AudioPlayer` (`bevy_audio-0.20.0/src/
//! audio_output.rs`), che l'elenco controlla.

use std::any::TypeId;
use std::collections::BTreeSet;

use bevy::audio::{
    AudioPlayer, AudioSource, DefaultSpatialScale, GlobalVolume, Pitch, PlaybackSettings,
    SpatialListener,
};
use bevy::prelude::*;
use bevy::window::WindowFocused;
use pixel_monster_survivor::audio::{
    AudioCatalog, AudioLibrary, AudioMixer, AudioSettings, AudioStats, CueBus, CueId, CueQueue,
    FocusFade, GameAudioPlugin, PlaybackStats, PlaybackVoice, VoiceCommands, VoiceId,
};
use pixel_monster_survivor::headless_app;

use crate::common::{clock, enemy_positions, run_steps};

/// Un pezzo dell'audio: il nome, come vederlo in un mondo e, per una risorsa, il suo tipo.
struct Piece {
    name: &'static str,
    present: fn(&mut World) -> bool,
    resource: Option<TypeId>,
}

macro_rules! resource {
    ($t:ty) => {
        Piece {
            name: stringify!($t),
            present: |world| world.contains_resource::<$t>(),
            resource: Some(TypeId::of::<$t>()),
        }
    };
}

macro_rules! component {
    ($t:ty) => {
        Piece {
            name: stringify!($t),
            present: |world| {
                let mut query = world.query_filtered::<(), With<$t>>();
                query.iter(world).next().is_some()
            },
            resource: None,
        }
    };
}

/// Tutti i pezzi dell'audio che un'app può avere: quelli di `src/audio/` (`mod.rs`, `focus.rs`,
/// `playback.rs`) e quelli di `bevy_audio` (risorse dell'`AudioPlugin`, asset, voci anche del
/// generatore `Pitch`, ascoltatore).
fn pieces() -> [Piece; 18] {
    [
        resource!(AudioSettings),
        resource!(CueQueue),
        resource!(AudioMixer),
        resource!(VoiceCommands),
        resource!(AudioStats),
        resource!(AudioCatalog),
        resource!(FocusFade),
        resource!(PlaybackStats),
        resource!(AudioLibrary),
        component!(PlaybackVoice),
        resource!(Assets<AudioSource>),
        resource!(Assets<Pitch>),
        resource!(GlobalVolume),
        resource!(DefaultSpatialScale),
        component!(AudioPlayer<AudioSource>),
        component!(AudioPlayer<Pitch>),
        component!(PlaybackSettings),
        component!(SpatialListener),
    ]
}

/// I pezzi presenti in `app`, nell'ordine dell'elenco.
fn footprint(app: &mut App, pieces: &[Piece]) -> Vec<&'static str> {
    let world = app.world_mut();
    pieces
        .iter()
        .filter(|piece| (piece.present)(world))
        .map(|piece| piece.name)
        .collect()
}

/// I tipi di tutte le risorse di `app`.
fn resource_types(app: &App) -> BTreeSet<TypeId> {
    app.world()
        .iter_resources()
        .filter_map(|(_, info, _)| info.type_id())
        .collect()
}

/// Un'app minima con il messaggio della finestra che l'audio legge (`WindowFocused`; nel client
/// lo registra la finestra): dal client differisce solo per l'audio.
fn base() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WindowFocused>();
    app
}

/// La base con tutto l'audio che si può avere senza dispositivo: `GameAudioPlugin` (nucleo,
/// impostazioni, fuoco, riproduzione), la libreria e le risorse dell'`AudioPlugin` di Bevy messe
/// a mano (il plugin vero aprirebbe il dispositivo), una voce per tipo e l'ascoltatore.
fn client() -> App {
    let mut app = base();
    app.add_plugins(GameAudioPlugin { muted: false })
        .insert_resource(AudioLibrary::default())
        .init_resource::<Assets<AudioSource>>()
        .init_resource::<Assets<Pitch>>()
        .init_resource::<GlobalVolume>()
        .init_resource::<DefaultSpatialScale>();
    let world = app.world_mut();
    world.spawn(AudioPlayer::<AudioSource>(default()));
    world.spawn(AudioPlayer::<Pitch>(default()));
    world.spawn(SpatialListener::default());
    world.spawn(PlaybackVoice {
        voice: VoiceId(0),
        cue: CueId(0),
        bus: CueBus::Ui,
        gain: 1.0,
        volume: 1.0,
        sounding: false,
        started_s: 0.0,
    });
    app.update();
    app
}

#[test]
fn the_headless_run_has_nothing_of_the_audio() {
    let pieces = pieces();
    let names: Vec<&str> = pieces.iter().map(|piece| piece.name).collect();

    // i due controlli vedono tutto l'audio di un'app che ce l'ha
    let mut client = client();
    assert_eq!(footprint(&mut client, &pieces), names);
    let mut base = base();
    base.update();
    let audio: BTreeSet<TypeId> = resource_types(&client)
        .difference(&resource_types(&base))
        .copied()
        .collect();
    for piece in &pieces {
        if let Some(resource) = piece.resource {
            assert!(
                audio.contains(&resource),
                "{}: la differenza per tipo non la vede",
                piece.name
            );
        }
    }
    let named = pieces.iter().filter(|p| p.resource.is_some()).count();

    // la run headless, viva dopo 10 s, non ne ha nessuno
    let mut app = headless_app(3);
    run_steps(&mut app, 600);
    assert!(clock(&app) > 9.9, "run ferma a {} s", clock(&app));
    assert!(!enemy_positions(&mut app).is_empty(), "nessun nemico");
    assert_eq!(footprint(&mut app, &pieces), Vec::<&str>::new());
    let leaked: Vec<String> = resource_types(&app)
        .intersection(&audio)
        .map(|resource| {
            pieces
                .iter()
                .find(|piece| piece.resource == Some(*resource))
                .map_or_else(
                    || format!("{resource:?} (non nell'elenco: aggiungila a `pieces`)"),
                    |piece| piece.name.to_string(),
                )
        })
        .collect();
    println!(
        "headless: {} pezzi per nome {names:?}; {} risorse aggiunte dall'audio ({named} \
         nell'elenco), {} nella run headless",
        pieces.len(),
        audio.len(),
        leaked.len()
    );
    assert!(
        leaked.is_empty(),
        "risorse dell'audio nella run headless: {leaked:?}"
    );
}
