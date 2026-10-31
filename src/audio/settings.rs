//! Impostazioni dell'audio scelte dal giocatore: un volume a tacche per il generale e per ogni
//! bus, il muto e l'audio con la finestra non attiva (progetto § 3.4 e § 3.10).
//!
//! In AUDIO-1 la risorsa nasce con i predefiniti; in AUDIO-2 si legge e si scrive nel profilo:
//! per questo è serializzabile e ogni campo ha un default, così i profili salvati prima
//! dell'audio si leggono. Il muto di `--mute` (`session_muted`) vale per la sessione e non si
//! serializza: una prova automatica non spegne l'audio del giocatore.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Tacche di ogni cursore: 0 è il silenzio, `STEPS` è 0 dB.
pub const STEPS: u8 = 10;

/// I bus con un cursore proprio. Il volume generale li moltiplica tutti; i suoni
/// dell'interfaccia seguono `Sfx` (con il loro trim nel catalogo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bus {
    Music,
    Sfx,
    Ambience,
}

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Volume generale, da 0 a [`STEPS`] tacche.
    pub master: u8,
    pub music: u8,
    pub sfx: u8,
    pub ambience: u8,
    /// Tutto in silenzio senza perdere i volumi scelti (scelta del giocatore, nel profilo).
    pub muted: bool,
    /// L'audio continua quando la finestra non è attiva (D-070: spento di default).
    pub play_unfocused: bool,
    /// Muto della sola sessione (`--mute`, `?mute`): mai nel profilo.
    #[serde(skip)]
    pub session_muted: bool,
}

impl Default for AudioSettings {
    /// Predefiniti del progetto (§ 3.4): generale 8, musica 7, effetti 8, ambiente 7.
    fn default() -> Self {
        Self {
            master: 8,
            music: 7,
            sfx: 8,
            ambience: 7,
            muted: false,
            play_unfocused: false,
            session_muted: false,
        }
    }
}

/// Guadagno lineare di un cursore: `(tacche / STEPS)²`, così le tacche basse restano usabili
/// (l'orecchio sente il volume in modo logaritmico). Oltre `STEPS` vale come `STEPS`.
pub fn step_gain(step: u8) -> f32 {
    let fraction = f32::from(step.min(STEPS)) / f32::from(STEPS);
    fraction * fraction
}

impl AudioSettings {
    /// Tacche del cursore di un bus.
    pub fn steps(&self, bus: Bus) -> u8 {
        match bus {
            Bus::Music => self.music,
            Bus::Sfx => self.sfx,
            Bus::Ambience => self.ambience,
        }
    }

    /// Muto per scelta del giocatore o per la sessione.
    pub fn is_muted(&self) -> bool {
        self.muted || self.session_muted
    }

    /// Guadagno di un bus voluto dal giocatore: generale × bus, 0 se l'audio è muto.
    pub fn bus_gain(&self, bus: Bus) -> f32 {
        if self.is_muted() {
            return 0.0;
        }
        step_gain(self.master) * step_gain(self.steps(bus))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUSES: [Bus; 3] = [Bus::Music, Bus::Sfx, Bus::Ambience];

    #[test]
    fn the_steps_go_from_silence_to_unity_and_never_go_down() {
        assert_eq!(step_gain(0), 0.0);
        assert_eq!(step_gain(STEPS), 1.0);
        assert_eq!(step_gain(STEPS + 5), 1.0);
        for step in 0..STEPS {
            assert!(step_gain(step) < step_gain(step + 1), "tacca {step}");
        }
        // la curva è quadratica: metà corsa = un quarto del guadagno (circa -12 dB)
        assert!((step_gain(STEPS / 2) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn the_bus_gain_multiplies_master_and_bus_and_mute_silences_everything() {
        let settings = AudioSettings::default();
        for bus in BUSES {
            let expected = step_gain(settings.master) * step_gain(settings.steps(bus));
            assert_eq!(settings.bus_gain(bus), expected, "{bus:?}");
            assert!(settings.bus_gain(bus) > 0.0);
        }
        for muted in [
            AudioSettings {
                muted: true,
                ..default_full()
            },
            AudioSettings {
                session_muted: true,
                ..default_full()
            },
        ] {
            assert!(muted.is_muted());
            for bus in BUSES {
                assert_eq!(muted.bus_gain(bus), 0.0, "{bus:?}");
            }
        }
        assert!(!default_full().is_muted());
        let no_master = AudioSettings {
            master: 0,
            ..default_full()
        };
        assert!(BUSES.iter().all(|&bus| no_master.bus_gain(bus) == 0.0));
        // un bus a zero non spegne gli altri
        let no_music = AudioSettings {
            music: 0,
            ..default_full()
        };
        assert_eq!(no_music.bus_gain(Bus::Music), 0.0);
        assert_eq!(no_music.bus_gain(Bus::Sfx), 1.0);
        assert_eq!(no_music.bus_gain(Bus::Ambience), 1.0);
    }

    fn default_full() -> AudioSettings {
        AudioSettings {
            master: STEPS,
            music: STEPS,
            sfx: STEPS,
            ambience: STEPS,
            ..AudioSettings::default()
        }
    }

    #[test]
    fn the_defaults_are_those_of_the_project() {
        let d = AudioSettings::default();
        assert_eq!((d.master, d.music, d.sfx, d.ambience), (8, 7, 8, 7));
        assert!(!d.muted && !d.play_unfocused);
    }

    #[test]
    fn missing_fields_take_the_default_so_old_profiles_still_load() {
        let empty: AudioSettings = ron::from_str("()").unwrap();
        assert_eq!(empty, AudioSettings::default());
        let partial: AudioSettings = ron::from_str("(music: 3, muted: true)").unwrap();
        assert_eq!(
            partial,
            AudioSettings {
                music: 3,
                muted: true,
                ..AudioSettings::default()
            }
        );
        let text = ron::to_string(&partial).unwrap();
        assert_eq!(ron::from_str::<AudioSettings>(&text).unwrap(), partial);
    }

    #[test]
    fn the_session_mute_never_reaches_the_profile() {
        let session = AudioSettings {
            session_muted: true,
            ..AudioSettings::default()
        };
        let text = ron::to_string(&session).unwrap();
        assert!(!text.contains("session"), "{text}");
        let back: AudioSettings = ron::from_str(&text).unwrap();
        assert!(!back.session_muted && !back.is_muted());
        assert_eq!(back, AudioSettings::default());
        // un profilo scritto a mano con il campo non lo accende
        let by_hand = ron::from_str::<AudioSettings>("(session_muted: true)");
        assert!(by_hand.is_ok_and(|s| !s.session_muted));
    }
}
