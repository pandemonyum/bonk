//! Musica (progetto § 3.7, D-069, D-071): la traccia voluta in ogni momento del gioco
//! ([`wanted`], pura) e il direttore con due deck ([`Decks`], puro) che passa da una traccia
//! all'altra con una dissolvenza equal-power. Qui la parte con Bevy, in `AudioSet::Music`:
//! - legge gli stati (`AppState`, `MenuScreen`, `RunPhase`) e lo stage di `GameConfig` in sola
//!   lettura;
//! - carica una traccia la prima volta che serve ([`MusicLibrary`]: l'handle resta, il ritorno al
//!   menu non la ricarica) e crea i deck con `playback::spawn_deck`, l'unico punto con le voci di
//!   Bevy;
//! - segue la nascita dei sink ([`notice_decks`], in `AudioSet::Listen`) e scrive il livello dei
//!   deck; il volume lo applica [`deck_volumes`] in `AudioSet::Apply`: generale × cursore Musica
//!   × `trims.music` × guadagno della traccia × livello del deck × fuoco della finestra, a ogni
//!   fotogramma, così un cursore cambia subito i deck vivi;
//! - un file che non si carica (assente) diventa silenzio, con un avviso: la traccia uscente non
//!   resta appesa ad aspettare un sink che non nascerà.
//!
//! Gira solo con `Assets<AudioSource>` (`playback::backend_on`): con `--no-audio` nessun `load`.
//! Senza dispositivo i sink non nascono: il deck voluto resta a zero e chi esce, non avendo mai
//! suonato, si toglie subito.
//!
//! Memoria del loop (`rodio-0.22.2/src/source/repeat.rs` e `buffered.rs`): con
//! `PlaybackMode::Loop` Bevy suona `decoder.repeat_infinite()`, cioè un `Buffered` del decoder.
//! Mentre suona il primo giro, i campioni decodificati restano in una catena di blocchi (uno per
//! pacchetto Vorbis, al più 32 768 campioni) che il `Repeat` tiene dalla testa per ripartire: dal
//! secondo giro in poi non si decodifica più, ma tutta la traccia resta in memoria in `f32`
//! (`rodio::Sample`, senza la feature `64bit`). Per `home.ogg`: 3 175 200 frame × 2 canali × 4
//! byte = 25 401 600 byte (24,2 MiB) per deck, più la catena dei blocchi (misurata nel test di
//! decodifica). La memoria si libera quando il deck si toglie (`rodio::Player` fermato al drop).

use std::collections::BTreeSet;

use bevy::audio::AudioSink;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::AudioSet;
use super::catalog::{MusicCatalog, TrackId};
use super::playback::{self, Mix, PlaybackStep};
use crate::app::{AppState, RunPhase};
use crate::data::GameConfig;
use crate::presentation::home::MenuScreen;

mod choice;
mod decks;
mod live;

pub use choice::{MusicContext, Wanted, wanted};
pub use decks::{Deck, DeckEvent, Decks, MAX_DECKS, Slot, fade_in, fade_out};
pub use live::{MusicDeck, MusicLibrary};
pub(crate) use live::{deck_volumes, notice_decks};

/// Contatori del direttore (prove, overlay in AUDIO-5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MusicStats {
    /// Cambi della traccia voluta.
    pub changes: u64,
    /// Deck creati e tolti.
    pub spawned: u64,
    pub despawned: u64,
    /// Sink nati.
    pub sounding: u64,
    /// Cambi verso una traccia che aveva ancora un deck: rientra dal suo livello.
    pub reentries: u64,
}

/// Il direttore: la traccia voluta, i deck e le loro entità.
#[derive(Resource, Debug, Default)]
pub struct MusicDirector {
    /// Quello che il gioco vuole adesso; `None` prima del primo stato con musica.
    pub target: Option<Wanted>,
    pub decks: Decks,
    entities: [Option<Entity>; MAX_DECKS],
    /// Tracce il cui file non si è caricato: al loro posto silenzio.
    failed: BTreeSet<TrackId>,
    pub stats: MusicStats,
}

impl MusicDirector {
    /// L'entità del deck `slot`, se è vivo.
    pub fn entity(&self, slot: Slot) -> Option<Entity> {
        self.entities[slot.index()]
    }
}

/// Il direttore e i suoi sistemi; tutto spento senza `Assets<AudioSource>`.
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicDirector>()
            .init_resource::<MusicLibrary>()
            .add_systems(
                Update,
                (
                    notice_decks::<AudioSink>.in_set(PlaybackStep::Notice),
                    direct.in_set(AudioSet::Music).run_if(playback::backend_on),
                    deck_volumes::<AudioSink>.in_set(PlaybackStep::Volumes),
                ),
            );
    }
}

/// Stati e stage, in sola lettura; senza stati (prove con `MinimalPlugins`) nessun cambio.
#[derive(SystemParam)]
struct MusicInputs<'w> {
    app: Option<Res<'w, State<AppState>>>,
    menu: Option<Res<'w, State<MenuScreen>>>,
    phase: Option<Res<'w, State<RunPhase>>>,
    config: Option<Res<'w, GameConfig>>,
}

impl MusicInputs<'_> {
    fn context(&self) -> Option<MusicContext<'_>> {
        Some(MusicContext {
            app: *self.app.as_ref()?.get(),
            menu: self.menu.as_ref().map(|state| *state.get()),
            phase: self.phase.as_ref().map(|state| *state.get()),
            stage: self.config.as_ref().map(|config| config.stage.id.as_str()),
        })
    }
}

/// Nome di una traccia voluta per il log; `None` fuori: nessuna scelta ancora (avvio).
fn shown(music: &MusicCatalog, track: Option<Option<TrackId>>) -> &str {
    match track {
        None => "avvio",
        Some(None) => "silenzio",
        Some(Some(track)) => music.name(track),
    }
}

/// Sceglie la traccia, fa avanzare le dissolvenze, crea e toglie i deck.
fn direct(
    mut commands: Commands,
    inputs: MusicInputs,
    mix: Mix,
    mut director: ResMut<MusicDirector>,
    mut library: ResMut<MusicLibrary>,
    server: Option<Res<AssetServer>>,
    mut decks: Query<&mut MusicDeck>,
    time: Res<Time<Real>>,
) {
    let music = &mix.catalog.music;
    let director = &mut *director;
    // i sink nati; un deck la cui entità è sparita esce dall'elenco (un deck creato nel
    // fotogramma prima c'è già: i comandi si applicano entro la fine di `Update`)
    for slot in Slot::ALL {
        let Some(entity) = director.entities[slot.index()] else {
            continue;
        };
        match decks.get(entity) {
            Ok(deck) if deck.sounding => director.decks.set_sounding(slot),
            Ok(_) => {}
            Err(_) => {
                warn!("musica: il deck {} è sparito", slot.name());
                director.decks.forget(slot);
                director.entities[slot.index()] = None;
            }
        }
    }
    // un file che non si carica: silenzio al suo posto
    if let Some(server) = &server {
        let broken: Vec<Deck> = director
            .decks
            .iter()
            .filter(|deck| !deck.sounding)
            .filter(|deck| {
                library
                    .handle(deck.track)
                    .is_some_and(|handle| server.load_state(handle.id()).is_failed())
            })
            .copied()
            .collect();
        for deck in broken {
            let file = music.track(deck.track).map_or("?", |t| t.file.as_str());
            warn!(
                "musica: {file} non si carica, al posto di {} silenzio",
                music.name(deck.track)
            );
            director.failed.insert(deck.track);
        }
    }
    let failed = |track: Option<TrackId>| track.is_some_and(|t| director.failed.contains(&t));
    if let Some(context) = inputs.context()
        && let Some(mut wanted) = wanted(&context, music)
    {
        if failed(wanted.track) {
            wanted.track = None;
        }
        let before = director.target.map(|target| target.track);
        director.target = Some(wanted);
        if before != Some(wanted.track) {
            note_change(director, music, before, wanted);
        }
    }
    // in `Loading` la scelta resta, ma un file fallito nel frattempo diventa silenzio
    let Some(mut target) = director.target else {
        return;
    };
    if target.track.is_some_and(|t| director.failed.contains(&t)) {
        target.track = None;
        director.target = Some(target);
    }
    let events = director
        .decks
        .advance(target.track, target.fade_s, time.delta_secs());
    for event in events {
        match event {
            DeckEvent::Spawn { slot, track } => {
                let handle = library.get_or_load(track, music, server.as_deref());
                let (Some(handle), Some(data)) = (handle, music.track(track)) else {
                    warn!("musica: {} senza AssetServer, silenzio", music.name(track));
                    director.failed.insert(track);
                    director.decks.forget(slot);
                    continue;
                };
                let deck = MusicDeck {
                    slot,
                    track,
                    gain: data.gain,
                    level: 0.0,
                    volume: 0.0,
                    sounding: false,
                    spawned_s: time.elapsed_secs_f64(),
                };
                let entity = playback::spawn_deck(&mut commands, handle, deck);
                director.entities[slot.index()] = Some(entity);
                director.stats.spawned += 1;
                info!(
                    "musica: deck {} prende {} ({}), a zero finché non nasce il sink",
                    slot.name(),
                    data.name,
                    data.file
                );
            }
            DeckEvent::Full { slot, track } => {
                let gain = music.track(track).map_or(0.0, |t| t.gain);
                info!(
                    "musica: deck {} {} a pieno (livello 1, volume {:.4}, fuoco della finestra \
                     {:.2})",
                    slot.name(),
                    music.name(track),
                    mix.music(gain),
                    mix.focus()
                );
            }
            DeckEvent::Despawn { slot, track } => {
                if let Some(entity) = director.entities[slot.index()].take() {
                    commands.entity(entity).try_despawn();
                }
                director.stats.despawned += 1;
                info!(
                    "musica: deck {} {} arrivato a zero, tolto ({} deck vivi)",
                    slot.name(),
                    music.name(track),
                    director.decks.len()
                );
            }
        }
    }
    for deck in director.decks.iter() {
        if let Some(entity) = director.entities[deck.slot.index()]
            && let Ok(mut live) = decks.get_mut(entity)
        {
            let level = deck.level();
            if live.level != level {
                live.level = level;
            }
        }
    }
}

/// Un cambio della traccia voluta, nel log: da dove a dove, con quale dissolvenza e quali deck.
fn note_change(
    director: &mut MusicDirector,
    music: &MusicCatalog,
    before: Option<Option<TrackId>>,
    wanted: Wanted,
) {
    director.stats.changes += 1;
    let incoming = match wanted.track.map(|track| director.decks.find(track)) {
        Some(Some(deck)) => {
            director.stats.reentries += 1;
            format!(
                "deck {} rientra dal livello {:.2}",
                deck.slot.name(),
                deck.level()
            )
        }
        Some(None) => "deck nuovo".to_string(),
        None => "nessun deck".to_string(),
    };
    let outgoing: Vec<String> = director
        .decks
        .iter()
        .filter(|deck| Some(deck.track) != wanted.track)
        .map(|deck| {
            format!(
                "deck {} {} esce dal livello {:.2}",
                deck.slot.name(),
                music.name(deck.track),
                deck.level()
            )
        })
        .collect();
    let outgoing = if outgoing.is_empty() {
        String::new()
    } else {
        format!("; {}", outgoing.join(", "))
    };
    info!(
        "musica: {} → {} (dissolvenza {} s; {incoming}{outgoing})",
        shown(music, before),
        shown(music, Some(wanted.track)),
        wanted.fade_s
    );
}
