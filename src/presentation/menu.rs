//! Selezione del compagno iniziale (`MenuScreen::Select`, D-066), aperta da "Nuova Partita"
//! sopra l'illustrazione della pagina iniziale, come il mockup dell'utente: a sinistra il titolo
//! e la griglia dei compagni dei dati (`cards`), a destra i dettagli dai dati (`details`:
//! descrizione, abilità iniziale, statistiche dell'attacco, `stats`), in basso Indietro e Inizia
//! Partita (`select`). Al centro niente, solo l'illustrazione (scelta dell'utente). I compagni
//! ancora da sbloccare (`profile::unlocks`) si guardano ma non partono. Tasti in `keys`.
//!
//! Qui stanno le parti comuni al resto del menu: il compagno scelto (`MenuChoice`, che anche i
//! talenti cambiano con Tab), l'avvio controllato della run (`StartRun`), colori ed etichette.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::home::MenuScreen;
use super::talents_menu::{panel_closed, talent_keys};
use crate::app::{AfterLoading, AppState, RunSetup, StartRefused, try_start_run};
use crate::data::GameConfig;
use crate::data::schema::{CompanionData, Element};
use crate::profile::{Profile, proposed_starter};

mod cards;
mod details;
mod keys;
mod select;
mod stats;

pub use keys::{GridStep, grid_step};
pub use stats::{AttackStats, decimal, relative_stats};

/// Colori della palette v1 (`docs/art/STYLE_HANDOFF.md`, `assets-src/palette.txt`).
pub(super) const PANEL: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
pub(super) const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);
/// Alone della carta scelta (`y`).
const GLOW: Color = Color::srgb_u8(0xff, 0xd6, 0x5c);
/// Titoli (`w`) e testi secondari (`4`).
const TITLE: Color = Color::srgb_u8(0xf4, 0xf1, 0xe8);
const SECONDARY: Color = Color::srgb_u8(0xa3, 0xb3, 0xd3);

/// Compagno selezionato, per id: resta giusto anche se una ricarica cambia l'ordine.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(super) struct MenuChoice(pub(super) String);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), reset_choice)
            .add_systems(
                OnEnter(MenuScreen::Select),
                (reset_choice, select::spawn_select).chain(),
            )
            .add_systems(
                Update,
                (
                    // prima dei talenti: l'Esc che chiude il loro pannello non torna anche indietro
                    keys::select_keys.run_if(panel_closed).before(talent_keys),
                    (
                        cards::style_cards,
                        cards::follow_choice,
                        select::style_buttons,
                        details::show_details,
                    )
                        .chain()
                        .after(talent_keys),
                )
                    .run_if(in_state(MenuScreen::Select)),
            );
    }
}

/// Ritratto del compagno nella scelta (`assets/ui/`, uno per id).
pub fn portrait_path(companion: &CompanionData) -> String {
    format!("ui/portrait_{}.png", companion.id)
}

/// Nome dell'elemento a schermo e colore del testo.
pub fn element_label(element: Element) -> (&'static str, Color) {
    match element {
        Element::Fire => ("Fuoco", Color::srgb_u8(0xf0, 0x8a, 0x3c)),
        Element::Water => ("Acqua", Color::srgb_u8(0x6c, 0xc7, 0xc2)),
        Element::Nature => ("Natura", Color::srgb_u8(0xb8, 0xe0, 0x5a)),
        Element::Electric => ("Fulmine", Color::srgb_u8(0xff, 0xd6, 0x5c)),
    }
}

/// Fondo e bordo della pillola dell'elemento (rampe della palette: scuro e base).
fn element_pill(element: Element) -> (Color, Color) {
    match element {
        Element::Fire => (
            Color::srgb_u8(0x6b, 0x2a, 0x1c),
            Color::srgb_u8(0xf2, 0x7a, 0x2e),
        ),
        Element::Water => (
            Color::srgb_u8(0x15, 0x37, 0x4a),
            Color::srgb_u8(0x2f, 0x8f, 0x9d),
        ),
        Element::Nature => (
            Color::srgb_u8(0x3c, 0x65, 0x26),
            Color::srgb_u8(0x74, 0xa8, 0x3c),
        ),
        Element::Electric => (
            Color::srgb_u8(0x74, 0x49, 0x1f),
            Color::srgb_u8(0xf2, 0xc9, 0x4c),
        ),
    }
}

/// All'ingresso nel menu e nella selezione il compagno scelto è l'ultimo usato, se è ancora
/// disponibile, altrimenti il primo disponibile (profilo nuovo: quello `Start` dei dati).
fn reset_choice(
    mut commands: Commands,
    config: Res<GameConfig>,
    setup: Res<RunSetup>,
    profile: Res<Profile>,
) {
    let pick = proposed_starter(&config, &profile, setup.companion.as_deref());
    commands.insert_resource(MenuChoice(pick));
}

/// Avvio della run con il compagno scelto, attraverso il controllo degli sblocchi.
#[derive(SystemParam)]
pub(super) struct StartRun<'w> {
    config: Res<'w, GameConfig>,
    profile: Res<'w, Profile>,
    setup: ResMut<'w, RunSetup>,
    after: ResMut<'w, AfterLoading>,
    next: ResMut<'w, NextState<AppState>>,
}

impl StartRun<'_> {
    /// Parte con il compagno `id` se esiste ed è un compagno iniziale disponibile; altrimenti
    /// lo scrive nel log e si resta nel menu.
    pub(super) fn go(&mut self, id: &str) -> Result<(), StartRefused> {
        let result = try_start_run(
            &self.config,
            &self.profile,
            &mut self.setup,
            &mut self.after,
            &mut self.next,
            id,
        );
        if let Err(why) = &result {
            warn!("partita non avviata: {why}");
        }
        result
    }
}

#[cfg(test)]
mod tests;
