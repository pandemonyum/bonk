//! La traccia voluta in ogni momento del gioco (progetto § 3.7): una funzione pura degli stati,
//! letti in sola lettura dalla simulazione e dal menu, e della sezione `music` del catalogo.
//!
//! - `AppState::Loading`: nessun cambio (il restart con F5 e la ricarica dei dati non
//!   interrompono la musica);
//! - `AppState::MainMenu`, ogni schermata: la traccia della home;
//! - in partita (`Playing`, `Paused`, `LevelUp`): la traccia dello stage, o il ripiego;
//! - vittoria e sconfitta (schermata dei risultati): silenzio, finché non c'è la traccia.
//!
//! Una traccia che il catalogo non ha è silenzio.

use crate::app::{AppState, RunPhase};
use crate::audio::catalog::{MusicCatalog, TrackId};
use crate::presentation::home::MenuScreen;

/// Il momento del gioco, per la musica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicContext<'a> {
    pub app: AppState,
    /// La schermata del menu, in `MainMenu`: oggi tutte suonano la home.
    pub menu: Option<MenuScreen>,
    /// La fase della run, in `InRun`.
    pub phase: Option<RunPhase>,
    /// Lo stage giocato (`GameConfig::stage`), se c'è una configurazione.
    pub stage: Option<&'a str>,
}

/// Cosa vuole la musica: una traccia (`None`: silenzio) e la dissolvenza per arrivarci.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wanted {
    pub track: Option<TrackId>,
    pub fade_s: f32,
}

/// La traccia voluta in `context`; `None` se la musica non deve cambiare.
pub fn wanted(context: &MusicContext, music: &MusicCatalog) -> Option<Wanted> {
    match context.app {
        AppState::Loading => None,
        // home, selezione del compagno, opzioni (e il pannello dei talenti, sopra la home)
        AppState::MainMenu => Some(Wanted {
            track: music.home(),
            fade_s: music.fades.to_home,
        }),
        AppState::InRun => {
            let track = match context.phase.unwrap_or_default() {
                // pausa e draft non cambiano traccia. AUDIO-3b: qui il ducking, il boss in campo
                // (Boss) e l'orda annunciata o in corso con l'isteresi (Horde)
                RunPhase::Playing | RunPhase::Paused | RunPhase::LevelUp => {
                    music.run(context.stage)
                }
                // AUDIO-3b: stinger di vittoria e sconfitta, poi la traccia dei risultati
                RunPhase::Victory | RunPhase::Defeated => None,
            };
            Some(Wanted {
                track,
                fade_s: music.fades.to_run,
            })
        }
    }
}
