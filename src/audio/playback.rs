//! Riproduzione (progetto § 3.2, § 3.4, § 3.9, § 3.11): l'unico punto del gioco che crea voci di
//! `bevy_audio` (`AudioPlayer`). In `AudioSet::Apply` esegue le decisioni del mixer
//! ([`VoiceCommands`]):
//! - `Start` con il file pronto: un'entità con `AudioPlayer`, `PlaybackSettings` in modo
//!   `Despawn` (volume e velocità) e [`PlaybackVoice`]. Con il file non pronto il suono si scarta
//!   (un colpo sentito un secondo dopo è peggio di nessun suono) e il mixer libera subito la voce;
//! - `Stop`: despawn dell'entità.
//!
//! Quando nasce il sink la voce suona (`Mixer::playing`, e la prima volta per cue un `info!`): si
//! segna in `AudioSet::Listen`, prima che il mixer decida le scadenze, così un fotogramma lungo
//! subito dopo l'avvio non fa scadere una voce che suona già;
//! quando l'entità sparisce, tolta da Bevy a fine suono o da noi, la voce è finita
//! (`Mixer::finished`). Il volume di ogni voce viva si ricalcola a ogni fotogramma (generale × bus
//! × trim × guadagno della voce × dissolvenza del fuoco) e si riscrive nel sink solo se cambia:
//! `GlobalVolume` vale solo alla nascita del sink (`bevy_audio-0.20.0/src/audio_output.rs:263`).
//!
//! Tutto gira solo se all'avvio c'era `Assets<AudioSource>`, cioè l'`AudioPlugin` di Bevy: con
//! `--no-audio` un `load` di `AudioSource` andrebbe in panic (`bevy_asset-0.20.0/src/server/
//! info.rs:855`). Senza dispositivo Bevy non crea sink (`audio_output.rs:103`): le voci scadono
//! nel mixer, si tolgono con uno `Stop` e un avviso lo dice una volta sola.

use std::collections::BTreeSet;

use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};
use bevy::ecs::component::Mutable;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::catalog::{AudioCatalog, CueBus, CueId};
use super::mixer::{Decision, VoiceId, expiry_time};
use super::{AudioMixer, AudioSet, VoiceCommands};

mod volume;
pub(crate) use volume::Mix;
pub use volume::{VOLUME_EPSILON, needs_volume, voice_volume};

/// Gli handle di tutti i file dei cue, caricati all'avvio (gli effetti sono piccoli, § 3.9).
#[derive(Resource, Debug, Clone, Default)]
pub struct AudioLibrary {
    /// Per [`CueId`], le varianti nell'ordine del catalogo.
    cues: Vec<Vec<Handle<AudioSource>>>,
}

impl AudioLibrary {
    /// Chiede all'`AssetServer` ogni file del catalogo. Solo con `Assets<AudioSource>`.
    pub fn load(catalog: &AudioCatalog, server: &AssetServer) -> Self {
        let cues = catalog
            .cues()
            .iter()
            .map(|cue| {
                cue.data
                    .files
                    .iter()
                    .map(|file| server.load(file.clone()))
                    .collect()
            })
            .collect();
        Self { cues }
    }

    /// Handle già pronti, per cue e variante (prove).
    pub fn from_handles(cues: Vec<Vec<Handle<AudioSource>>>) -> Self {
        Self { cues }
    }

    pub fn handle(&self, cue: CueId, variant: usize) -> Option<&Handle<AudioSource>> {
        self.cues.get(cue.index())?.get(variant)
    }

    /// File in tutto.
    pub fn files(&self) -> usize {
        self.cues.iter().map(Vec::len).sum()
    }
}

/// Una voce viva della riproduzione.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlaybackVoice {
    pub voice: VoiceId,
    pub cue: CueId,
    pub bus: CueBus,
    /// Guadagno base dal mixer: cue × variazione × boost, senza trim né cursori.
    pub gain: f32,
    /// Volume lineare voluto adesso, catena completa (quello del sink, se c'è).
    pub volume: f32,
    /// Il sink è nato: la voce suona.
    pub sounding: bool,
    /// Istante dell'avvio, in secondi di tempo reale.
    pub started_s: f64,
}

/// Contatori della riproduzione (overlay F3 in AUDIO-5) e cue già sentiti nella sessione.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct PlaybackStats {
    /// Entità di voce create.
    pub spawned: u64,
    /// `Start` scartati perché il file non era pronto.
    pub not_ready: u64,
    /// Sink nati.
    pub sounding: u64,
    /// Voci tolte con uno `Stop` del mixer.
    pub stopped: u64,
    /// Voci col file caricato tolte dal mixer per scadenza: nessun sink entro `length_s` + 0,5 s
    /// (nessun dispositivo). Con la riproduzione accesa è uguale a `MixerStats::expired` (i file
    /// non pronti liberano la voce subito e non scadono); le fermate per furto o `Restart` non ci
    /// sono, anche oltre `length_s`.
    pub silent_expired: u64,
    /// Cue che hanno suonato almeno una volta (un `info!` alla prima).
    pub heard: BTreeSet<CueId>,
    /// Avvisi scritti nel log per le voci mute: al più uno per sessione.
    pub silent_warnings: u32,
}

/// Passi della riproduzione: `Notice` in `AudioSet::Listen` (i sink nati, prima del mixer);
/// `Execute` e poi `Volumes` in `AudioSet::Apply`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaybackStep {
    Notice,
    Execute,
    Volumes,
}

/// La riproduzione gira: c'è la libreria, quindi `Assets<AudioSource>`.
fn backend_on(
    library: Option<Res<AudioLibrary>>,
    sources: Option<Res<Assets<AudioSource>>>,
) -> bool {
    library.is_some() && sources.is_some()
}

/// La riproduzione: si accende solo se all'avvio esiste `Assets<AudioSource>`.
pub struct PlaybackPlugin;

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaybackStats>()
            .configure_sets(
                Update,
                PlaybackStep::Notice
                    .in_set(AudioSet::Listen)
                    .run_if(backend_on),
            )
            .configure_sets(
                Update,
                (PlaybackStep::Execute, PlaybackStep::Volumes)
                    .chain()
                    .in_set(AudioSet::Apply)
                    .run_if(backend_on),
            )
            .add_systems(Startup, load_library)
            .add_systems(
                Update,
                (
                    notice_sinks::<AudioSink>.in_set(PlaybackStep::Notice),
                    execute.in_set(PlaybackStep::Execute),
                    apply_volumes::<AudioSink>.in_set(PlaybackStep::Volumes),
                ),
            )
            .add_observer(voice_gone);
    }
}

/// `Startup`: gli handle di tutti i file, solo con `Assets<AudioSource>` (una libreria già
/// inserita, nelle prove, resta).
fn load_library(
    mut commands: Commands,
    library: Option<Res<AudioLibrary>>,
    server: Option<Res<AssetServer>>,
    sources: Option<Res<Assets<AudioSource>>>,
    catalog: Res<AudioCatalog>,
) {
    if library.is_some() {
        return;
    }
    match (server, sources) {
        (Some(server), Some(_)) => {
            let library = AudioLibrary::load(&catalog, &server);
            info!(
                "audio: {} file di {} cue in caricamento",
                library.files(),
                catalog.cues().len()
            );
            commands.insert_resource(library);
        }
        _ => info!("audio: riproduzione spenta, senza l'AudioPlugin di Bevy (--no-audio)"),
    }
}

/// I file da cui suonare: libreria e asset.
#[derive(SystemParam)]
struct Backend<'w> {
    library: Res<'w, AudioLibrary>,
    sources: Res<'w, Assets<AudioSource>>,
}

impl Backend<'_> {
    /// Il file è pronto: è negli asset, la condizione con cui Bevy crea il sink
    /// (`audio_output.rs:110`).
    fn ready(&self, handle: &Handle<AudioSource>) -> bool {
        self.sources.contains(handle.id())
    }
}

/// Mixer e contatori, scritti insieme.
#[derive(SystemParam)]
struct Book<'w> {
    mixer: ResMut<'w, AudioMixer>,
    stats: ResMut<'w, PlaybackStats>,
}

/// Esegue le decisioni del fotogramma.
fn execute(
    mut commands: Commands,
    decisions: Res<VoiceCommands>,
    backend: Backend,
    voices: Query<(Entity, &PlaybackVoice)>,
    mut book: Book,
    mix: Mix,
    time: Res<Time<Real>>,
) {
    let now = time.elapsed_secs_f64();
    let mut spawned: Vec<(VoiceId, Entity)> = Vec::new();
    for decision in &decisions.0 {
        match *decision {
            Decision::Start {
                voice,
                cue,
                variant,
                gain,
                speed,
            } => {
                let ready = backend
                    .library
                    .handle(cue, variant)
                    .filter(|handle| backend.ready(handle));
                let (Some(handle), Some(data)) = (ready, mix.catalog.cue(cue)) else {
                    book.mixer.0.finished(voice);
                    book.stats.not_ready += 1;
                    continue;
                };
                let bus = data.data.bus;
                let volume = mix.volume(bus, gain);
                let entity = commands
                    .spawn((
                        AudioPlayer::new(handle.clone()),
                        PlaybackSettings {
                            volume: Volume::Linear(volume),
                            speed,
                            ..PlaybackSettings::DESPAWN
                        },
                        PlaybackVoice {
                            voice,
                            cue,
                            bus,
                            gain,
                            volume,
                            sounding: false,
                            started_s: now,
                        },
                    ))
                    .id();
                spawned.push((voice, entity));
                book.stats.spawned += 1;
            }
            Decision::Stop { voice } => {
                let live = voices
                    .iter()
                    .find(|(_, playback)| playback.voice == voice)
                    .map(|(entity, playback)| (entity, Some(*playback)));
                let fresh = || {
                    spawned
                        .iter()
                        .find(|(id, _)| *id == voice)
                        .map(|&(_, entity)| (entity, None))
                };
                let Some((entity, playback)) = live.or_else(fresh) else {
                    continue;
                };
                if let Some(playback) = playback {
                    note_silent(&playback, now, &mix.catalog, &mut book.stats);
                }
                commands.entity(entity).try_despawn();
                book.stats.stopped += 1;
            }
        }
    }
}

/// Uno `Stop` per scadenza: la voce non ha mai suonato e il suo termine è passato. Il mixer toglie
/// le voci scadute prima di ogni furto o `Restart`, con lo stesso orologio e lo stesso termine
/// ([`expiry_time`]), quindi una voce oltre il termine è ferma per scadenza e una prima no. Un
/// avviso solo, poi i contatori.
fn note_silent(
    playback: &PlaybackVoice,
    now: f64,
    catalog: &AudioCatalog,
    stats: &mut PlaybackStats,
) {
    let Some(cue) = catalog.cue(playback.cue) else {
        return;
    };
    if playback.sounding || now < expiry_time(playback.started_s, cue.data.length_s) {
        return; // fermata per un furto o un `Restart`, non scaduta
    }
    stats.silent_expired += 1;
    if stats.silent_warnings == 0 {
        stats.silent_warnings = 1;
        warn!(
            "audio: {} non ha suonato (file caricato, nessun sink: manca il dispositivo audio?); \
             non lo ripeto",
            cue.name
        );
    }
}

/// Il sink `S` è nato: la voce suona. Gira in `AudioSet::Listen`, prima delle scadenze del mixer.
/// Generico per le prove, che non hanno un dispositivo.
pub(crate) fn notice_sinks<S: Component>(
    mut voices: Query<&mut PlaybackVoice, Added<S>>,
    mut mixer: ResMut<AudioMixer>,
    catalog: Res<AudioCatalog>,
    mut stats: ResMut<PlaybackStats>,
) {
    for mut playback in &mut voices {
        if playback.sounding {
            continue;
        }
        playback.sounding = true;
        mixer.0.playing(playback.voice);
        stats.sounding += 1;
        if stats.heard.insert(playback.cue) {
            let name = catalog
                .cue(playback.cue)
                .map_or("?", |cue| cue.name.as_str());
            info!("audio: primo suono di {name}");
        }
    }
}

/// Il volume di ogni voce viva, riscritto nel sink `S` solo se cambia ([`needs_volume`]).
pub(crate) fn apply_volumes<S: Component<Mutability = Mutable> + AudioSinkPlayback>(
    mut voices: Query<(&mut PlaybackVoice, Option<&mut S>)>,
    mix: Mix,
) {
    for (mut playback, sink) in &mut voices {
        let volume = mix.volume(playback.bus, playback.gain);
        if playback.volume != volume {
            playback.volume = volume;
        }
        if let Some(mut sink) = sink
            && needs_volume(sink.volume().to_linear(), volume)
        {
            sink.set_volume(Volume::Linear(volume));
        }
    }
}

/// L'entità di una voce sparisce (Bevy a fine suono, o uno `Stop`): la voce è finita. Una voce
/// già tolta dal mixer (furto, scadenza, file non pronto) non c'è più e va bene.
fn voice_gone(
    remove: On<Remove<PlaybackVoice>>,
    voices: Query<&PlaybackVoice>,
    mixer: Option<ResMut<AudioMixer>>,
) {
    if let (Ok(playback), Some(mut mixer)) = (voices.get(remove.entity), mixer) {
        mixer.0.finished(playback.voice);
    }
}
