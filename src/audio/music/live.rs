//! I deck vivi come entità: il componente [`MusicDeck`], gli handle delle tracce
//! ([`MusicLibrary`]), la nascita del sink e il volume a ogni fotogramma. Generici sul sink per le
//! prove, che non hanno un dispositivo.

use std::collections::BTreeMap;

use bevy::audio::{AudioSinkPlayback, AudioSource, Volume};
use bevy::ecs::component::Mutable;
use bevy::prelude::*;

use super::{MusicDirector, Slot};
use crate::audio::catalog::{MusicCatalog, TrackId};
use crate::audio::playback::Mix;

/// Un deck vivo della musica, sull'entità creata da `playback::spawn_deck`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MusicDeck {
    pub slot: Slot,
    pub track: TrackId,
    /// Guadagno della traccia, dal catalogo.
    pub gain: f32,
    /// Livello della dissolvenza, 0–1 (lo scrive il direttore).
    pub level: f32,
    /// Volume lineare voluto adesso, catena completa (quello del sink, se c'è).
    pub volume: f32,
    /// Il sink è nato.
    pub sounding: bool,
    /// Istante della creazione, in secondi di tempo reale.
    pub spawned_s: f64,
}

/// Gli handle delle tracce già chieste: si caricano la prima volta che servono e restano.
#[derive(Resource, Debug, Clone, Default)]
pub struct MusicLibrary {
    handles: BTreeMap<TrackId, Handle<AudioSource>>,
}

impl MusicLibrary {
    /// Handle già pronti (prove): nessun caricamento per queste tracce.
    pub fn from_handles(handles: impl IntoIterator<Item = (TrackId, Handle<AudioSource>)>) -> Self {
        Self {
            handles: handles.into_iter().collect(),
        }
    }

    pub fn handle(&self, track: TrackId) -> Option<&Handle<AudioSource>> {
        self.handles.get(&track)
    }

    /// Tracce chieste finora.
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// L'handle di `track`, chiesto all'`AssetServer` la prima volta.
    pub(super) fn get_or_load(
        &mut self,
        track: TrackId,
        music: &MusicCatalog,
        server: Option<&AssetServer>,
    ) -> Option<Handle<AudioSource>> {
        if let Some(handle) = self.handles.get(&track) {
            return Some(handle.clone());
        }
        let file = &music.track(track)?.file;
        let handle: Handle<AudioSource> = server?.load(file.clone());
        info!("musica: carico {file}");
        self.handles.insert(track, handle.clone());
        Some(handle)
    }
}

/// Il sink `S` di un deck è nato: il deck suona e la dissolvenza può partire. In
/// `AudioSet::Listen`; generico per le prove, che non hanno un dispositivo.
pub(crate) fn notice_decks<S: Component>(
    mut decks: Query<&mut MusicDeck, Added<S>>,
    mut director: ResMut<MusicDirector>,
    mix: Mix,
    time: Res<Time<Real>>,
) {
    for mut deck in &mut decks {
        if deck.sounding {
            continue;
        }
        deck.sounding = true;
        director.stats.sounding += 1;
        let fade = director.target.map_or(0.0, |target| target.fade_s);
        info!(
            "musica: deck {} {}: sink nato dopo {:.2} s, dissolvenza equal-power di {fade} s",
            deck.slot.name(),
            mix.catalog.music.name(deck.track),
            time.elapsed_secs_f64() - deck.spawned_s
        );
    }
}

/// Il volume di ogni deck vivo, riscritto nel sink `S` a ogni cambio: i deck sono al più due e
/// una dissolvenza deve arrivare esatta al suo livello (le voci degli effetti usano invece la
/// soglia di [`needs_volume`](super::playback::needs_volume)).
pub(crate) fn deck_volumes<S: Component<Mutability = Mutable> + AudioSinkPlayback>(
    mut decks: Query<(&mut MusicDeck, Option<&mut S>)>,
    mix: Mix,
) {
    for (mut deck, sink) in &mut decks {
        let volume = mix.music(deck.gain * deck.level);
        if deck.volume != volume {
            deck.volume = volume;
        }
        if let Some(mut sink) = sink
            && sink.volume().to_linear() != volume
        {
            sink.set_volume(Volume::Linear(volume));
        }
    }
}
