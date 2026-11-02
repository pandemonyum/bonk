//! Le voci vive della riproduzione, senza `bevy_audio` oltre al sink (nessuna voce nasce qui):
//! la nascita del sink, il volume a ogni fotogramma, la fine della voce e le scadenze senza sink.
//! Generici sul sink dove serve, per le prove senza dispositivo.

use bevy::audio::{AudioSinkPlayback, Volume};
use bevy::ecs::component::Mutable;
use bevy::prelude::*;

use super::volume::{Mix, needs_volume};
use super::{PlaybackStats, PlaybackVoice};
use crate::audio::AudioMixer;
use crate::audio::catalog::AudioCatalog;
use crate::audio::mixer::expiry_time;

/// Uno `Stop` per scadenza: la voce non ha mai suonato e il suo termine è passato. Il mixer toglie
/// le voci scadute prima di ogni furto o `Restart`, con lo stesso orologio e lo stesso termine
/// ([`expiry_time`]), quindi una voce oltre il termine è ferma per scadenza e una prima no. Un
/// avviso solo, poi i contatori.
pub(super) fn note_silent(
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
pub(super) fn voice_gone(
    remove: On<Remove<PlaybackVoice>>,
    voices: Query<&PlaybackVoice>,
    mixer: Option<ResMut<AudioMixer>>,
) {
    if let (Ok(playback), Some(mut mixer)) = (voices.get(remove.entity), mixer) {
        mixer.0.finished(playback.voice);
    }
}
