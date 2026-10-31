//! Audio con la finestra non attiva (progetto § 3.11, D-070): con `play_unfocused = false`, su
//! `WindowFocused(false)` il volume generale scende a zero in [`FOCUS_FADE_S`] secondi di tempo
//! reale e risale nello stesso tempo al ritorno del fuoco. È una dissolvenza della riproduzione
//! ([`FocusFade::level`] moltiplica il volume di ogni voce): non tocca `AudioSettings` né il
//! profilo. Sul desktop il gioco continua a girare senza fuoco (`WinitSettings::game`), quindi la
//! dissolvenza avanza; nel browser la pagina sospende anche l'`AudioContext` quando la scheda è
//! nascosta (`web/index.html`).

use bevy::prelude::*;
use bevy::window::WindowFocused;

use super::settings::AudioSettings;

/// Durata della dissolvenza in uscita e in entrata, in secondi di tempo reale.
pub const FOCUS_FADE_S: f32 = 0.25;

/// Fuoco della finestra e livello della dissolvenza (0 silenzio, 1 pieno).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FocusFade {
    pub focused: bool,
    pub level: f32,
}

impl Default for FocusFade {
    /// Si parte con la finestra attiva: il primo `WindowFocused` dice il contrario se serve.
    fn default() -> Self {
        Self {
            focused: true,
            level: 1.0,
        }
    }
}

/// Il livello voluto: pieno con la finestra attiva o con l'audio fuori dalla finestra acceso.
pub fn focus_target(focused: bool, play_unfocused: bool) -> f32 {
    if focused || play_unfocused { 1.0 } else { 0.0 }
}

/// Un passo lineare da `level` verso `target`, che percorre tutto l'intervallo 0–1 in
/// `duration` secondi; non lo supera mai. Un fotogramma lungo (o `duration` ≤ 0) arriva subito.
pub fn fade_toward(level: f32, target: f32, dt: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        return target;
    }
    let step = dt.max(0.0) / duration;
    if level < target {
        (level + step).min(target)
    } else {
        (level - step).max(target)
    }
}

/// Legge i cambi di fuoco e fa avanzare la dissolvenza con il tempo reale.
pub(super) fn follow_focus(
    mut focus: MessageReader<WindowFocused>,
    settings: Res<AudioSettings>,
    time: Res<Time<Real>>,
    mut fade: ResMut<FocusFade>,
) {
    if let Some(last) = focus.read().last()
        && fade.focused != last.focused
    {
        fade.focused = last.focused;
    }
    let target = focus_target(fade.focused, settings.play_unfocused);
    let level = fade_toward(fade.level, target, time.delta_secs(), FOCUS_FADE_S);
    if level != fade.level {
        fade.level = level;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fade_takes_a_quarter_of_a_second_each_way_and_never_overshoots() {
        let frame = 1.0 / 60.0;
        let mut level = 1.0;
        let mut frames = 0;
        while level > 0.0 {
            level = fade_toward(level, 0.0, frame, FOCUS_FADE_S);
            frames += 1;
            assert!((0.0..=1.0).contains(&level));
        }
        // 0,25 s a 60 FPS: 15 fotogrammi (più uno per l'arrotondamento in virgola mobile)
        assert!((15..=16).contains(&frames), "{frames} fotogrammi");
        assert_eq!(fade_toward(0.0, 1.0, FOCUS_FADE_S / 2.0, FOCUS_FADE_S), 0.5);
        assert_eq!(fade_toward(0.5, 1.0, FOCUS_FADE_S, FOCUS_FADE_S), 1.0);
        // un intoppo lungo arriva subito, un tempo negativo non muove nulla
        assert_eq!(fade_toward(1.0, 0.0, 3.0, FOCUS_FADE_S), 0.0);
        assert_eq!(fade_toward(0.3, 1.0, -1.0, FOCUS_FADE_S), 0.3);
        assert_eq!(fade_toward(0.3, 1.0, 0.0, 0.0), 1.0);
        // già al livello voluto: resta
        assert_eq!(fade_toward(1.0, 1.0, frame, FOCUS_FADE_S), 1.0);
    }

    #[test]
    fn only_an_unfocused_window_without_the_option_goes_silent() {
        assert_eq!(focus_target(true, false), 1.0);
        assert_eq!(focus_target(true, true), 1.0);
        assert_eq!(focus_target(false, true), 1.0);
        assert_eq!(focus_target(false, false), 0.0);
    }
}
