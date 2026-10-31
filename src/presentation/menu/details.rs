//! Pannello dei dettagli del compagno guardato: nome, pillola dell'elemento, descrizione
//! (`companions.ron`), "Abilità iniziale" con lo sprite, il nome e la frase dell'attacco
//! (`attacks.ron`) e "Statistiche iniziali" (`stats`). Un compagno da sbloccare mostra anche il
//! lucchetto. Si ricostruisce quando cambia la scelta.

use bevy::prelude::*;

use super::stats::stat_rows;
use super::{GOLD, MenuChoice, SECONDARY, TITLE, element_label, element_pill};
use crate::data::GameConfig;
use crate::data::schema::CompanionData;
use crate::presentation::fonts;
use crate::presentation::icons::{UI_PX_PER_TEXEL, element_icon_path, ui_icon};
use crate::presentation::menu_list::{TEXT, spawn_panel};
use crate::profile::{Profile, is_unlocked};

/// Larghezza del pannello (unità dell'interfaccia).
const DETAILS_W: f32 = 300.0;
/// Lato del riquadro dell'abilità e spazio per lo sprite dentro, in texel. Lo sprite cresce al
/// più di `MAX_ZOOM` volte: i colpi piccoli (la goccia è 4×4) diventerebbero quadrati pieni.
const ABILITY_TEXELS: f32 = 24.0;
const ABILITY_SPRITE: f32 = 16.0;
const MAX_ZOOM: f32 = 3.0;

/// Il contenuto del pannello, da ricostruire.
#[derive(Component)]
pub(super) struct DetailsBody;

/// Lo sprite dell'attacco: si ingrandisce di un numero intero di volte quando l'immagine è
/// caricata (le misure dei colpi cambiano da attacco ad attacco).
#[derive(Component)]
pub(super) struct AttackIcon;

pub(super) fn spawn_details(commands: &mut Commands, assets: &AssetServer) -> Entity {
    let panel = spawn_panel(
        commands,
        assets,
        Node {
            width: px(DETAILS_W),
            flex_shrink: 0.0,
            // in basso, sopra Inizia Partita, come nel riferimento
            align_self: AlignSelf::FlexEnd,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            border: UiRect::all(px(4)),
            ..default()
        },
    );
    let body = commands
        .spawn((
            Name::new("Dettagli"),
            DetailsBody,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .id();
    commands.entity(panel).add_child(body);
    panel
}

fn text(value: impl Into<String>, font: impl Bundle, color: Color) -> impl Bundle {
    (Text::new(value.into()), font, TextColor(color))
}

/// Ricostruisce i dettagli quando cambiano la scelta o il profilo (e al primo fotogramma), e
/// ingrandisce lo sprite dell'attacco appena caricato.
pub(super) fn show_details(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    profile: Res<Profile>,
    choice: Res<MenuChoice>,
    bodies: Query<Entity, With<DetailsBody>>,
    fresh: Query<(), Added<DetailsBody>>,
    mut icons: Query<(&ImageNode, &mut Node), With<AttackIcon>>,
    images: Res<Assets<Image>>,
) {
    for (image, mut node) in &mut icons {
        if let Some(size) = images.get(&image.image).map(Image::size_f32) {
            let wanted = fit_icon(size, ABILITY_SPRITE) * UI_PX_PER_TEXEL;
            if node.width != px(wanted.x) || node.height != px(wanted.y) {
                node.width = px(wanted.x);
                node.height = px(wanted.y);
            }
        }
    }
    if !(choice.is_changed() || profile.is_changed() || !fresh.is_empty()) {
        return;
    }
    let Some(companion) = config.companion(&choice.0) else {
        return;
    };
    let open = is_unlocked(&config, &profile, &companion.id);
    for body in &bodies {
        commands.entity(body).despawn_children();
        let parts = [
            header(&mut commands, &assets, companion),
            commands
                .spawn(text(companion.description.clone(), fonts::ui(16.0), TEXT))
                .id(),
            section(&mut commands, "Abilità iniziale"),
            ability(&mut commands, &assets, &config, companion),
            section(&mut commands, "Statistiche iniziali"),
            stats(&mut commands, &assets, &config, companion),
        ];
        commands.entity(body).add_children(&parts);
        if !open {
            let lock = locked_line(&mut commands, &assets);
            commands.entity(body).insert_children(1, &[lock]);
        }
    }
}

/// La misura in texel di un'immagine `size` ingrandita del massimo numero intero di volte che
/// ci sta in `side` texel, da 1 a `MAX_ZOOM`.
pub fn fit_icon(size: Vec2, side: f32) -> Vec2 {
    let longest = size.max_element().max(1.0);
    size * (side / longest).floor().clamp(1.0, MAX_ZOOM)
}

/// Nome grande e pillola dell'elemento con l'icona.
fn header(commands: &mut Commands, assets: &AssetServer, companion: &CompanionData) -> Entity {
    let (label, color) = element_label(companion.element);
    let (fill, edge) = element_pill(companion.element);
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(6),
            row_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            row.spawn(text(companion.display_name.clone(), fonts::ui(33.0), TITLE));
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(4),
                    padding: UiRect::right(px(8)),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BackgroundColor(fill),
                BorderColor::all(edge),
            ))
            .with_children(|pill| {
                pill.spawn(ui_icon(
                    assets,
                    element_icon_path(companion.element),
                    16.0,
                    16.0,
                ));
                pill.spawn(text(label, fonts::hud(16.0), color));
            });
        })
        .id()
}

fn locked_line(commands: &mut Commands, assets: &AssetServer) -> Entity {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn(ui_icon(assets, "ui/lock.png", 12.0, 14.0));
            row.spawn(text("Da sbloccare", fonts::ui(16.0), GOLD));
        })
        .id()
}

/// Titolo d'oro di una sezione con la riga che arriva al bordo.
fn section(commands: &mut Commands, title: &str) -> Entity {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6),
            margin: UiRect::top(px(2)),
            ..default()
        })
        .with_children(|row| {
            row.spawn(text(title.to_string(), fonts::ui(16.0), GOLD));
            row.spawn((
                Node {
                    flex_grow: 1.0,
                    height: px(UI_PX_PER_TEXEL),
                    ..default()
                },
                BackgroundColor(GOLD.with_alpha(0.35)),
            ));
        })
        .id()
}

/// Riquadro con lo sprite dell'attacco, nome e frase dell'abilità.
fn ability(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    companion: &CompanionData,
) -> Entity {
    let attack = config.companion_attack(companion);
    let (fill, edge) = element_pill(companion.element);
    let side = ABILITY_TEXELS * UI_PX_PER_TEXEL;
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(side),
                    height: px(side),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(UI_PX_PER_TEXEL)),
                    ..default()
                },
                BackgroundColor(fill),
                BorderColor::all(edge),
            ))
            .with_child((
                AttackIcon,
                ImageNode::new(assets.load(attack.sprite().to_string())),
                Node {
                    width: px(0),
                    height: px(0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_shrink: 1.0,
                row_gap: px(2),
                ..default()
            })
            .with_children(|words| {
                words.spawn(text(attack.display_name.clone(), fonts::ui(22.0), TITLE));
                words.spawn(text(attack.description.clone(), fonts::ui(16.0), SECONDARY));
            });
        })
        .id()
}

/// Tre righe: icona, nome e valore, barra relativa al compagno più forte.
fn stats(
    commands: &mut Commands,
    assets: &AssetServer,
    config: &GameConfig,
    companion: &CompanionData,
) -> Entity {
    let list = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .id();
    for row in stat_rows(config, companion) {
        let (path, w, h) = row.icon;
        let line = commands
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            })
            .with_children(|line| {
                line.spawn(Node {
                    width: px(32),
                    height: px(32),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_child(ui_icon(assets, path, w, h));
                line.spawn(Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                })
                .with_children(|column| {
                    column
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|words| {
                            words.spawn(text(row.label, fonts::ui(16.0), TEXT));
                            words.spawn(text(row.value.clone(), fonts::hud(16.0), TITLE));
                        });
                    column
                        .spawn((
                            Node {
                                height: px(6),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        ))
                        .with_child((
                            Node {
                                width: percent(100.0 * row.fill.clamp(0.0, 1.0)),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(row.color),
                        ));
                });
            })
            .id();
        commands.entity(list).add_child(line);
    }
    list
}
