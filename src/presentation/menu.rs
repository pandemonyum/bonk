//! Scelta del compagno (schermata "Compagni" del menu, `MenuScreen::Companions`), sopra
//! l'illustrazione della pagina iniziale. Ogni carta mostra lo sprite, l'elemento, il tipo di
//! attacco e due barre (danno al secondo su un nemico, portata) relative agli altri compagni
//! (`cards`). 1–4, A/D o clic per scegliere, Invio o "INIZIA" per partire, Esc o "INDIETRO" per
//! tornare alla pagina iniziale, da cui "Nuova partita" parte con il compagno scelto qui.
//! Titoli e frasi in Pixelify Sans, etichette in Tiny5.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::fonts;
use super::home::MenuScreen;
use super::talents_menu::{panel_closed, talent_keys};
use crate::app::{AfterLoading, AppState, RunSetup, start_run};
use crate::data::GameConfig;
use crate::data::schema::{AttackPattern, CompanionData, Element};

mod cards;

/// Colori della palette v1 (`docs/art/STYLE_HANDOFF.md`).
pub(super) const PANEL: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
const CARD: Color = Color::srgb_u8(0x1b, 0x26, 0x40);
const CARD_HOVER: Color = Color::srgb_u8(0x2b, 0x3a, 0x5e);
pub(super) const GOLD: Color = Color::srgb_u8(0xf2, 0xc9, 0x4c);
const MUTED: Color = Color::srgb_u8(0x5a, 0x6d, 0x99);

/// Compagno selezionato, per id: resta giusto anche se una ricarica cambia l'ordine.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(super) struct MenuChoice(pub(super) String);

/// Carta del compagno con questo id.
#[derive(Component)]
struct CompanionCard(String);

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct BackButton;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), reset_choice)
            .add_systems(OnEnter(MenuScreen::Companions), spawn_menu)
            .add_systems(
                Update,
                (
                    // prima dei talenti: l'Esc che chiude il loro pannello non torna anche indietro
                    choose_with_keys.run_if(panel_closed).before(talent_keys),
                    highlight_choice,
                    hover_cards,
                )
                    .chain()
                    .run_if(in_state(MenuScreen::Companions)),
            );
    }
}

/// Ritratto del compagno nella scelta (`assets/ui/`, uno per id).
pub fn portrait_path(companion: &CompanionData) -> String {
    format!("ui/portrait_{}.png", companion.id)
}

/// Riga descrittiva del pattern di attacco.
pub fn pattern_label(pattern: &AttackPattern) -> &'static str {
    match pattern {
        AttackPattern::Projectile(_) => "Proiettile sul nemico vicino",
        AttackPattern::Pulse(_) => "Onda attorno a te: colpisce e rallenta",
        AttackPattern::Orbit(_) => "Foglie che ti girano attorno",
        AttackPattern::Chain(_) => "Fulmine che salta tra i nemici",
    }
}

pub fn element_label(element: Element) -> (&'static str, Color) {
    match element {
        Element::Fire => ("Fuoco", Color::srgb_u8(0xf0, 0x8a, 0x3c)),
        Element::Water => ("Acqua", Color::srgb_u8(0x6c, 0xc7, 0xc2)),
        Element::Nature => ("Natura", Color::srgb_u8(0xb8, 0xe0, 0x5a)),
        Element::Electric => ("Elettro", Color::srgb_u8(0xff, 0xd6, 0x5c)),
    }
}

/// (danno al secondo su un nemico, portata) di ogni compagno, divisi per il massimo tra tutti:
/// valori da 0 a 1 per le barre del menu.
pub fn relative_stats(config: &GameConfig) -> Vec<(f32, f32)> {
    let raw: Vec<(f32, f32)> = config
        .companions
        .iter()
        .map(|c| {
            let a = config.companion_attack(c);
            (a.damage / a.interval_s, a.range)
        })
        .collect();
    let max_dps = raw.iter().map(|r| r.0).fold(f32::MIN_POSITIVE, f32::max);
    let max_reach = raw.iter().map(|r| r.1).fold(f32::MIN_POSITIVE, f32::max);
    raw.iter()
        .map(|(dps, reach)| (dps / max_dps, reach / max_reach))
        .collect()
}

/// All'ingresso nel menu il compagno scelto è l'ultimo con cui si è giocato (all'avvio quello
/// dello stage).
fn reset_choice(mut commands: Commands, config: Res<GameConfig>, setup: Res<RunSetup>) {
    commands.insert_resource(MenuChoice(setup.starter(&config).id.clone()));
}

fn spawn_menu(
    mut commands: Commands,
    config: Res<GameConfig>,
    assets: Res<AssetServer>,
    choice: Res<MenuChoice>,
) {
    let stats = relative_stats(&config);
    let root = commands
        .spawn((
            Name::new("MainMenu"),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            // l'illustrazione della pagina iniziale si intravede sotto
            BackgroundColor(PANEL.with_alpha(0.86)),
            DespawnOnExit(MenuScreen::Companions),
            children![
                (Text::new("COMPAGNI"), fonts::ui(44.0), TextColor(GOLD),),
                (
                    Text::new("Scegli con chi parti"),
                    fonts::ui(22.0),
                    TextColor(Color::WHITE),
                ),
            ],
        ))
        .id();
    let row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(16),
            ..default()
        })
        .id();
    commands.entity(root).add_child(row);
    for (i, companion) in config.companions.iter().enumerate() {
        let attack = config.companion_attack(companion);
        let card = cards::spawn_card(
            &mut commands,
            &assets,
            i,
            companion,
            pattern_label(&attack.pattern),
            stats[i],
            companion.id == choice.0,
        );
        commands.entity(row).add_child(card);
    }
    let stage = &config.stage;
    let goal = format!(
        "Mappa: {}  ·  sopravvivi {} e sconfiggi il boss",
        stage.display_name,
        crate::gameplay::run::mm_ss(stage.run.duration_s)
    );
    let footer = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                margin: UiRect::top(px(6)),
                ..default()
            },
            children![(
                Text::new(goal),
                fonts::ui(16.0),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            )],
        ))
        .id();
    let back = commands
        .spawn((
            BackButton,
            footer_button("INDIETRO  (Esc)", MUTED, Color::WHITE),
        ))
        .observe(
            |_: On<Activate>, mut screen: ResMut<NextState<MenuScreen>>| {
                screen.set(MenuScreen::Home)
            },
        )
        .id();
    let start = commands
        .spawn((StartButton, footer_button("INIZIA  (Invio)", GOLD, GOLD)))
        .observe(
            |_: On<Activate>, choice: Res<MenuChoice>, mut start: StartRun| start.go(&choice.0),
        )
        .id();
    let buttons = commands
        .spawn(Node {
            column_gap: px(16),
            ..default()
        })
        .add_children(&[back, start])
        .id();
    let hint = commands
        .spawn((
            Text::new(
                "1–4, A/D o clic per scegliere · T talenti · in partita WASD o frecce, Esc pausa",
            ),
            fonts::hud(16.0),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        ))
        .id();
    commands.entity(footer).add_children(&[buttons, hint]);
    commands.entity(root).add_child(footer);
}

/// Pulsante in fondo alla scelta: bordo `edge`, testo `color`.
fn footer_button(label: &str, edge: Color, color: Color) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Node {
            padding: UiRect::axes(px(26), px(8)),
            border: UiRect::all(px(2)),
            ..default()
        },
        BorderColor::all(edge),
        BackgroundColor(CARD),
        children![(
            Text::new(label.to_string()),
            fonts::ui(22.0),
            TextColor(color)
        )],
    )
}

/// Avvio della run con il compagno scelto.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct StartRun<'w> {
    config: Res<'w, GameConfig>,
    setup: ResMut<'w, RunSetup>,
    after: ResMut<'w, AfterLoading>,
    next: ResMut<'w, NextState<AppState>>,
}

impl StartRun<'_> {
    /// Parte con il compagno `id`, se esiste ancora nei dati.
    pub(super) fn go(&mut self, id: &str) {
        if self.config.companion(id).is_some() {
            start_run(&mut self.setup, &mut self.after, &mut self.next, id);
        }
    }
}

fn choose_with_keys(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut choice: ResMut<MenuChoice>,
    mut start: StartRun,
    mut screen: ResMut<NextState<MenuScreen>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        screen.set(MenuScreen::Home);
        return;
    }
    const KEYS: [[KeyCode; 2]; 4] = [
        [KeyCode::Digit1, KeyCode::Numpad1],
        [KeyCode::Digit2, KeyCode::Numpad2],
        [KeyCode::Digit3, KeyCode::Numpad3],
        [KeyCode::Digit4, KeyCode::Numpad4],
    ];
    let ids: Vec<&str> = config.companions.iter().map(|c| c.id.as_str()).collect();
    let mut index = ids.iter().position(|id| *id == choice.0).unwrap_or(0);
    if let Some(slot) = KEYS.iter().position(|pair| keys.any_just_pressed(*pair))
        && slot < ids.len()
    {
        index = slot;
    }
    let count = ids.len().max(1);
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        index = (index + count - 1) % count;
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        index = (index + 1) % count;
    }
    if let Some(id) = ids.get(index)
        && choice.0 != *id
    {
        choice.0 = id.to_string();
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) {
        start.go(&choice.0);
    }
}

fn highlight_choice(choice: Res<MenuChoice>, mut cards: Query<(&CompanionCard, &mut BorderColor)>) {
    if !choice.is_changed() {
        return;
    }
    for (card, mut border) in &mut cards {
        *border = BorderColor::all(if card.0 == choice.0 { GOLD } else { MUTED });
    }
}

fn hover_cards(
    mut cards: Query<
        (&Hovered, &mut BackgroundColor),
        (
            Or<(With<CompanionCard>, With<StartButton>, With<BackButton>)>,
            Changed<Hovered>,
        ),
    >,
) {
    for (hovered, mut fill) in &mut cards {
        fill.0 = if hovered.get() { CARD_HOVER } else { CARD };
    }
}

#[cfg(test)]
mod tests;
