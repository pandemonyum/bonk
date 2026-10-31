//! Selezione del compagno iniziale: formule delle statistiche e della griglia, e il flusso nel
//! menu senza finestra (`home::tests::menu_app`) con profili in memoria, mai quello dell'utente.

use super::cards::scroll_to_row;
use super::details::fit_icon;
use super::*;
use crate::presentation::home::tests::{go, menu_app, named, screen, tap};
use crate::profile::is_unlocked;

#[test]
fn stat_bars_are_relative_to_the_strongest_companion() {
    let config = GameConfig::embedded();
    let stats = relative_stats(&config);
    assert_eq!(stats.len(), config.companions.len());
    for row in &stats {
        assert!(row.iter().all(|v| (0.0..=1.0).contains(v)), "{row:?}");
    }
    for k in 0..3 {
        assert!(
            stats.iter().any(|s| s[k] == 1.0),
            "il più forte ha la barra piena ({k})"
        );
    }
}

/// I numeri dei dettagli vengono dall'attacco dei dati, non dall'interfaccia.
#[test]
fn starting_stats_come_from_the_attack_data() {
    let config = GameConfig::embedded();
    for companion in &config.companions {
        let attack = config.companion_attack(companion);
        let stats = AttackStats::of(attack);
        assert_eq!(stats.dps, attack.damage / attack.interval_s);
        assert_eq!(stats.interval_s, attack.interval_s);
        assert_eq!(stats.range, attack.range);
        assert!((stats.rate() * attack.interval_s - 1.0).abs() < 1e-6);
        let rows = super::stats::stat_rows(&config, companion);
        assert_eq!(rows[0].value, format!("{}/s", decimal(stats.dps)));
        assert_eq!(
            rows[1].value,
            format!("ogni {} s", decimal(attack.interval_s))
        );
        assert_eq!(rows[2].value, format!("{} px", decimal(attack.range)));
    }
}

#[test]
fn numbers_use_one_decimal_with_a_comma() {
    assert_eq!(decimal(12.0 / 1.1), "10,9");
    assert_eq!(decimal(4.0), "4");
    assert_eq!(decimal(150.0), "150");
    assert_eq!(decimal(0.6), "0,6");
    assert_eq!(decimal(9.96), "10");
}

#[test]
fn electric_is_called_lightning() {
    assert_eq!(element_label(Element::Electric).0, "Fulmine");
}

#[test]
fn arrows_move_in_the_grid() {
    use GridStep::*;
    // 4 carte su 2 colonne: 0 1 / 2 3
    assert_eq!(grid_step(0, 4, 2, Right), 1);
    assert_eq!(grid_step(1, 4, 2, Right), 2, "destra scorre la lista");
    assert_eq!(grid_step(0, 4, 2, Left), 3, "in giro");
    assert_eq!(grid_step(0, 4, 2, Down), 2);
    assert_eq!(grid_step(3, 4, 2, Up), 1);
    assert_eq!(grid_step(1, 4, 2, Up), 1, "sopra non c'è nulla");
    assert_eq!(grid_step(2, 4, 2, Down), 2, "sotto non c'è nulla");
    // 3 carte: dalla 1 in giù non c'è la carta 3
    assert_eq!(grid_step(1, 3, 2, Down), 1);
    assert_eq!(grid_step(0, 0, 2, Right), 0);
}

#[test]
fn the_grid_scrolls_to_keep_the_chosen_row_in_view() {
    // righe da 160 con 10 di spazio, finestra di due righe (330)
    assert_eq!(scroll_to_row(0.0, 1, 160.0, 10.0, 330.0), 0.0);
    assert_eq!(scroll_to_row(0.0, 2, 160.0, 10.0, 330.0), 170.0);
    assert_eq!(scroll_to_row(170.0, 0, 160.0, 10.0, 330.0), 0.0);
    assert_eq!(scroll_to_row(170.0, 2, 160.0, 10.0, 330.0), 170.0);
}

#[test]
fn ability_sprites_grow_by_whole_texels() {
    // sprite dei colpi in uno spazio di 16 texel, al più 3 volte
    assert_eq!(fit_icon(Vec2::new(8.0, 8.0), 16.0), Vec2::new(16.0, 16.0));
    assert_eq!(fit_icon(Vec2::new(6.0, 6.0), 16.0), Vec2::new(12.0, 12.0));
    assert_eq!(fit_icon(Vec2::new(4.0, 4.0), 16.0), Vec2::new(12.0, 12.0));
    assert_eq!(fit_icon(Vec2::new(40.0, 8.0), 16.0), Vec2::new(40.0, 8.0));
}

// --- Il flusso nel menu ---

fn state(app: &App) -> AppState {
    *app.world().resource::<State<AppState>>().get()
}

fn choice(app: &App) -> String {
    app.world().resource::<MenuChoice>().0.clone()
}

fn open_selection(profile: Profile) -> App {
    let mut app = menu_app(profile);
    go(&mut app, MenuScreen::Select);
    app
}

/// Il primo compagno bloccato dei dati per un profilo nuovo.
fn locked_companion(config: &GameConfig) -> (usize, String) {
    let profile = Profile::default();
    config
        .companions
        .iter()
        .enumerate()
        .find(|(_, c)| !is_unlocked(config, &profile, &c.id))
        .map(|(i, c)| (i, c.id.clone()))
        .expect("i dati hanno un compagno da sbloccare")
}

const DIGITS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// Profilo nuovo: si entra con Fiammetto, l'unico disponibile, e si parte con lui.
#[test]
fn a_new_profile_starts_with_fiammetto() {
    let mut app = open_selection(Profile::default());
    assert_eq!(choice(&app), "ember");
    tap(&mut app, KeyCode::Enter);
    app.update();
    assert_eq!(state(&app), AppState::InRun);
    let setup = app.world().resource::<RunSetup>();
    assert_eq!(setup.companion.as_deref(), Some("ember"));
    assert_eq!(
        app.world().resource::<AfterLoading>().0,
        AppState::InRun,
        "F5 e Invio dai risultati ripartono senza menu"
    );
    assert_eq!(named(&mut app, "Selezione"), 0, "la schermata è chiusa");
}

/// Un compagno bloccato si guarda ma non parte, né da tastiera né chiamando l'avvio.
#[test]
fn a_locked_companion_never_starts() {
    let config = GameConfig::embedded();
    let (slot, locked) = locked_companion(&config);
    let mut app = open_selection(Profile::default());
    tap(&mut app, DIGITS[slot]);
    assert_eq!(choice(&app), locked, "si può guardare");
    for key in [KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter] {
        tap(&mut app, key);
        app.update();
        assert_eq!(state(&app), AppState::MainMenu, "{key:?}");
        assert_eq!(screen(&app), MenuScreen::Select);
        assert_eq!(app.world().resource::<RunSetup>(), &RunSetup::default());
    }
    // la stessa logica del pulsante, chiamata direttamente
    let result = app
        .world_mut()
        .run_system_cached_with(
            |id: In<String>, mut start: StartRun| start.go(&id),
            locked.clone(),
        )
        .unwrap();
    assert_eq!(result, Err(StartRefused::Locked(locked)));
    app.update();
    assert_eq!(state(&app), AppState::MainMenu);
    assert_eq!(app.world().resource::<RunSetup>(), &RunSetup::default());
    // Inizia Partita non risponde ai clic
    assert_eq!(disabled_start_buttons(&mut app), 1);
}

/// Quante "Inizia Partita" spente ci sono a schermo.
fn disabled_start_buttons(app: &mut App) -> usize {
    let world = app.world_mut();
    world
        .query_filtered::<(), (
            With<super::select::StartButton>,
            With<bevy::ui::InteractionDisabled>,
        )>()
        .iter(world)
        .count()
}

/// Inizia Partita si spegne guardando un bloccato e si riaccende tornando a Fiammetto.
#[test]
fn the_start_button_follows_the_chosen_companion() {
    let config = GameConfig::embedded();
    let (slot, _) = locked_companion(&config);
    let ember = config.companion_index("ember").expect("Fiammetto nei dati");
    let mut app = open_selection(Profile::default());
    app.update();
    assert_eq!(
        disabled_start_buttons(&mut app),
        0,
        "con Fiammetto è acceso"
    );
    tap(&mut app, DIGITS[slot]);
    app.update();
    assert_eq!(
        disabled_start_buttons(&mut app),
        1,
        "con un bloccato è spento"
    );
    tap(&mut app, DIGITS[ember]);
    app.update();
    assert_eq!(choice(&app), "ember");
    assert_eq!(disabled_start_buttons(&mut app), 0, "di nuovo acceso");
}

/// Un compagno sbloccato dal profilo (condizione futura) parte come Fiammetto.
#[test]
fn a_granted_companion_starts() {
    let config = GameConfig::embedded();
    let (slot, locked) = locked_companion(&config);
    let mut profile = Profile::default();
    profile.grant(&locked);
    let mut app = open_selection(profile);
    tap(&mut app, DIGITS[slot]);
    tap(&mut app, KeyCode::Enter);
    app.update();
    assert_eq!(state(&app), AppState::InRun);
    assert_eq!(
        app.world().resource::<RunSetup>().companion.as_deref(),
        Some(locked.as_str())
    );
}

/// Indietro (Esc o Backspace) torna alla pagina iniziale senza run.
#[test]
fn back_returns_home_without_a_run() {
    for key in [KeyCode::Escape, KeyCode::Backspace] {
        let mut app = open_selection(Profile::default());
        tap(&mut app, key);
        assert_eq!(screen(&app), MenuScreen::Home, "{key:?}");
        assert_eq!(state(&app), AppState::MainMenu);
        assert_eq!(app.world().resource::<RunSetup>(), &RunSetup::default());
        assert_eq!(named(&mut app, "Selezione"), 0);
        assert_eq!(named(&mut app, "Home"), 1);
    }
}

/// Frecce e cifre muovono la scelta nella griglia dei dati.
#[test]
fn keys_move_the_choice_in_the_grid() {
    let config = GameConfig::embedded();
    let ids: Vec<String> = config.companions.iter().map(|c| c.id.clone()).collect();
    let mut app = open_selection(Profile::default());
    tap(&mut app, KeyCode::ArrowRight);
    assert_eq!(choice(&app), ids[1]);
    tap(&mut app, KeyCode::KeyS);
    assert_eq!(
        choice(&app),
        ids[grid_step(1, ids.len(), 2, GridStep::Down)]
    );
    tap(&mut app, KeyCode::ArrowUp);
    assert_eq!(choice(&app), ids[1]);
    tap(&mut app, KeyCode::KeyA);
    assert_eq!(choice(&app), ids[0]);
    tap(&mut app, DIGITS[ids.len() - 1]);
    assert_eq!(choice(&app), ids[ids.len() - 1]);
    // una cifra oltre i compagni non cambia nulla
    if ids.len() < DIGITS.len() {
        tap(&mut app, DIGITS[ids.len()]);
        assert_eq!(choice(&app), ids[ids.len() - 1]);
    }
}

/// Aperta e chiusa più volte, la schermata non lascia doppioni: una radice, una carta per
/// compagno e un pannello dei dettagli.
#[test]
fn opening_the_selection_again_leaves_no_duplicates() {
    let config = GameConfig::embedded();
    let mut app = menu_app(Profile::default());
    for _ in 0..3 {
        go(&mut app, MenuScreen::Select);
        go(&mut app, MenuScreen::Home);
    }
    go(&mut app, MenuScreen::Select);
    // un cambio di scelta ricostruisce i dettagli al posto di quelli di prima
    tap(&mut app, KeyCode::ArrowRight);
    assert_eq!(named(&mut app, "Selezione"), 1);
    assert_eq!(named(&mut app, "Griglia dei compagni"), 1);
    assert_eq!(named(&mut app, "Dettagli"), 1);
    for companion in &config.companions {
        assert_eq!(named(&mut app, &format!("Carta {}", companion.id)), 1);
    }
    let world = app.world_mut();
    let headers = world
        .query::<&Text>()
        .iter(world)
        .filter(|t| t.0 == "Abilità iniziale")
        .count();
    assert_eq!(headers, 1, "un solo contenuto dei dettagli");
}

/// Entrando si torna all'ultimo compagno usato se è disponibile, altrimenti al primo
/// disponibile: un compagno bloccato guardato con i talenti (Tab) non resta scelto.
#[test]
fn entering_proposes_the_last_available_companion() {
    let config = GameConfig::embedded();
    let (_, locked) = locked_companion(&config);
    let mut app = menu_app(Profile::default());
    app.world_mut().resource_mut::<RunSetup>().companion = Some(locked.clone());
    app.world_mut().resource_mut::<MenuChoice>().0 = locked;
    go(&mut app, MenuScreen::Select);
    assert_eq!(choice(&app), "ember");
}
