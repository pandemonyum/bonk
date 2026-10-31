//! Impostazioni dell'audio scelte dal giocatore: un volume a tacche per il generale e per ogni
//! bus, il muto e l'audio con la finestra non attiva (progetto § 3.4 e § 3.10).
//!
//! Stanno nel profilo (`ProfileData::audio`): per questo sono serializzabili e ogni campo ha un
//! default, così i profili salvati prima dell'audio si leggono. All'avvio [`apply_profile`] le
//! copia nella risorsa; le Opzioni le cambiano e le salvano con [`AudioSettings::store`]. Il muto
//! di `--mute` (`session_muted`) vale per la sessione e non si serializza: una prova automatica
//! non spegne l'audio del giocatore, e resta anche quando il profilo sostituisce la risorsa.

use bevy::prelude::{Res, ResMut, Resource};
use serde::{Deserialize, Serialize};

use crate::profile::{Profile, ProfileData};

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

    /// Tacche di un cursore delle Opzioni.
    pub fn slider(&self, slider: Slider) -> u8 {
        match slider {
            Slider::Master => self.master,
            Slider::Bus(bus) => self.steps(bus),
        }
    }

    /// Porta un cursore a `steps` tacche (al massimo [`STEPS`]); `true` se è cambiato.
    pub fn set_slider(&mut self, slider: Slider, steps: u8) -> bool {
        let steps = steps.min(STEPS);
        let field = match slider {
            Slider::Master => &mut self.master,
            Slider::Bus(Bus::Music) => &mut self.music,
            Slider::Bus(Bus::Sfx) => &mut self.sfx,
            Slider::Bus(Bus::Ambience) => &mut self.ambience,
        };
        let changed = *field != steps;
        *field = steps;
        changed
    }

    /// Le impostazioni come vanno nel profilo: senza il muto della sessione.
    pub fn saved(&self) -> Self {
        Self {
            session_muted: false,
            ..self.clone()
        }
    }

    /// Le impostazioni di un profilo, con il muto della sessione di `self` (`--mute`).
    pub fn with_profile(&self, data: &ProfileData) -> Self {
        Self {
            session_muted: self.session_muted,
            ..data.audio.clone()
        }
    }

    /// Le scrive nel profilo e lo salva (se ha un percorso): il muto della sessione non ci va.
    pub fn store(&self, profile: &mut Profile) {
        profile.data.audio = self.saved();
        profile.save();
    }
}

/// I quattro cursori delle Opzioni: il volume generale e un bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slider {
    Master,
    Bus(Bus),
}

impl Slider {
    /// Nell'ordine delle Opzioni.
    pub const ALL: [Slider; 4] = [
        Slider::Master,
        Slider::Bus(Bus::Music),
        Slider::Bus(Bus::Sfx),
        Slider::Bus(Bus::Ambience),
    ];
}

/// `Startup`: le impostazioni del profilo del dispositivo, tenendo il muto della sessione.
/// Senza profilo (prove) restano i predefiniti.
pub fn apply_profile(profile: Option<Res<Profile>>, mut settings: ResMut<AudioSettings>) {
    let Some(profile) = profile else {
        return;
    };
    let wanted = settings.with_profile(&profile.data);
    if *settings != wanted {
        *settings = wanted;
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

    #[test]
    fn every_slider_moves_its_own_field_up_to_the_last_step() {
        for (k, slider) in Slider::ALL.into_iter().enumerate() {
            let mut settings = AudioSettings::default();
            let before = settings.clone();
            assert!(settings.set_slider(slider, 3));
            assert_eq!(settings.slider(slider), 3);
            assert!(
                !settings.set_slider(slider, 3),
                "stesso valore: nessun cambio"
            );
            assert!(settings.set_slider(slider, STEPS + 4));
            assert_eq!(settings.slider(slider), STEPS, "oltre l'ultima tacca");
            for (j, other) in Slider::ALL.into_iter().enumerate() {
                if j != k {
                    assert_eq!(settings.slider(other), before.slider(other), "{slider:?}");
                }
            }
        }
        let settings = AudioSettings::default();
        assert_eq!(settings.slider(Slider::Master), settings.master);
        assert_eq!(settings.slider(Slider::Bus(Bus::Music)), settings.music);
        assert_eq!(settings.slider(Slider::Bus(Bus::Sfx)), settings.sfx);
        assert_eq!(
            settings.slider(Slider::Bus(Bus::Ambience)),
            settings.ambience
        );
    }

    #[test]
    fn the_profile_replaces_the_settings_but_not_the_session_mute() {
        let mut data = ProfileData::default();
        data.audio.music = 2;
        data.audio.play_unfocused = true;
        let session = AudioSettings {
            session_muted: true,
            ..AudioSettings::default()
        };
        let loaded = session.with_profile(&data);
        assert_eq!(loaded.music, 2);
        assert!(loaded.play_unfocused && loaded.session_muted && loaded.is_muted());
        // e il muto della sessione non torna nel profilo
        let mut profile = Profile::default();
        loaded.store(&mut profile);
        assert_eq!(profile.data.audio, data.audio);
        assert!(!profile.data.audio.session_muted);
        assert_eq!(loaded.saved(), data.audio);
    }
}
