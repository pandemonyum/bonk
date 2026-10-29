//! Forzieri nell'interfaccia: sopra ogni forziere a schermo il prezzo ("15 monete", dorato) o
//! quanto manca ("Mancano 6", grigio); fuori schermo l'icona del forziere più vicino al bordo
//! della vista, nella sua direzione e col prezzo, come l'indicatore del boss. Il player deve
//! sapere dove andare e se gli conviene.

use bevy::prelude::*;

use super::boss_pointer::{edge_point, inner_rect};
use super::camera::{VisibleArea, visible_rect};
use super::fonts;
use super::icons::{UI_PX_PER_TEXEL, ui_icon};
use crate::app::{AppState, RunPhase};
use crate::data::GameConfig;
use crate::gameplay::kinematics::Position;
use crate::gameplay::player::Player;
use crate::gameplay::treasure::{Chest, Treasury};

const GOLD: Color = Color::srgb(0.95, 0.78, 0.3);
const GREY: Color = Color::srgb(0.62, 0.62, 0.68);
/// Lato dell'icona del forziere in texel (il riquadro è disegnato nel PNG).
const ICON_TEXELS: f32 = 16.0;
/// Larghezza delle scritte (unità dell'interfaccia), centrate sul loro punto.
const LABEL_W: f32 = 200.0;
/// Il prezzo sta sopra il coperchio, in texel dai piedi del forziere.
const LABEL_ABOVE: f32 = 30.0;
/// Sopra l'HUD come l'indicatore del boss.
const UI_Z: i32 = 5;

#[derive(Component)]
struct ChestPointer;

#[derive(Component)]
struct PointerPrice;

/// Scritta sopra un forziere a schermo (una per forziere, ricreate quando cambiano).
#[derive(Component)]
struct ChestLabel(usize);

#[derive(Component)]
struct UiOrigin;

/// La scritta del prezzo: quanto costa se le monete bastano, se no quante ne mancano.
pub fn price_text(price: u32, coins: f32) -> (String, bool) {
    let have = (coins + 1e-3).floor() as u32;
    if have >= price {
        let unit = if price == 1 { "moneta" } else { "monete" };
        (format!("{price} {unit}"), true)
    } else {
        (format!("Mancano {}", price - have), false)
    }
}

pub struct ChestUiPlugin;

impl Plugin for ChestUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InRun), spawn_origin)
            .add_systems(
                Update,
                (point_at_chest, label_chests).run_if(in_state(AppState::InRun)),
            )
            .add_systems(OnExit(RunPhase::Playing), hide_chest_ui)
            .add_systems(OnEnter(RunPhase::Playing), show_chest_ui);
    }
}

/// Draft, pausa e risultati coprono il campo: le scritte dei forzieri spariscono con lui.
fn hide_chest_ui(mut origin: Query<&mut Visibility, With<UiOrigin>>) {
    for mut visibility in &mut origin {
        *visibility = Visibility::Hidden;
    }
}

fn show_chest_ui(mut origin: Query<&mut Visibility, With<UiOrigin>>) {
    for mut visibility in &mut origin {
        *visibility = Visibility::Inherited;
    }
}

/// Come per il boss: un nodo a dimensione zero al centro della finestra fa da origine.
fn spawn_origin(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Name::new("ChestUiOrigin"),
        UiOrigin,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            ..default()
        },
        GlobalZIndex(UI_Z),
        DespawnOnExit(AppState::InRun),
        children![(
            ChestPointer,
            Node {
                position_type: PositionType::Absolute,
                width: px(LABEL_W),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(2),
                ..default()
            },
            Visibility::Hidden,
            children![
                ui_icon(&assets, "ui/icon_chest.png", ICON_TEXELS, ICON_TEXELS),
                (
                    PointerPrice,
                    Text::new(""),
                    fonts::hud(16.0),
                    TextColor(GOLD)
                ),
            ],
        )],
    ));
}

/// Posizione nell'interfaccia (dal centro, y in basso) di un punto in texel dal centro della
/// vista (y in alto), con la scritta larga `LABEL_W` centrata.
fn ui_at(spot: Vec2) -> (Val, Val) {
    (
        px(spot.x.round() * UI_PX_PER_TEXEL - LABEL_W / 2.0),
        px(-spot.y.round() * UI_PX_PER_TEXEL),
    )
}

#[allow(clippy::type_complexity)]
fn point_at_chest(
    config: Res<GameConfig>,
    area: Res<VisibleArea>,
    treasury: Res<Treasury>,
    player: Query<&Position, With<Player>>,
    chests: Query<&Position, With<Chest>>,
    pointer: Single<(&mut Node, &mut Visibility), With<ChestPointer>>,
    mut price: Single<(&mut Text, &mut TextColor), With<PointerPrice>>,
    mut icon: Query<&mut ImageNode>,
    children: Query<&Children, With<ChestPointer>>,
) {
    let (mut node, mut visibility) = pointer.into_inner();
    let Ok(me) = player.single() else {
        return;
    };
    let view = visible_rect(me.0, &area);
    let on_screen = chests
        .iter()
        .any(|c| view.contains(c.0 + Vec2::Y * (LABEL_ABOVE / 2.0)));
    let nearest = chests.iter().min_by(|a, b| {
        a.0.distance_squared(me.0)
            .total_cmp(&b.0.distance_squared(me.0))
    });
    let (low, high) = inner_rect(view.half_size());
    // il punto è il centro dell'icona, con il prezzo sotto: in basso serve più margine
    let spot = match nearest {
        Some(chest) if !on_screen => edge_point(
            chest.0 - view.center(),
            view.half_size(),
            low + Vec2::Y * 10.0,
            high,
        ),
        _ => None,
    };
    let Some(spot) = spot else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let (left, top) = ui_at(spot + Vec2::Y * (ICON_TEXELS / 2.0));
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    let (line, afford) = price_text(treasury.price(&config.stage.chests), treasury.coins);
    let color = if afford { GOLD } else { GREY };
    let (text, text_color) = &mut *price;
    if text.0 != line {
        text.0 = line;
    }
    text_color.0 = color;
    // l'icona grigia quando non basta: lo si capisce anche senza leggere
    for kids in &children {
        for kid in kids.iter() {
            if let Ok(mut image) = icon.get_mut(kid) {
                image.color = if afford { Color::WHITE } else { GREY };
            }
        }
    }
    if *visibility != Visibility::Inherited {
        *visibility = Visibility::Inherited;
    }
}

/// Le scritte sopra i forzieri a schermo, ricreate a ogni fotogramma solo se cambia il numero
/// dei forzieri visibili; spostate ogni fotogramma.
fn label_chests(
    mut commands: Commands,
    config: Res<GameConfig>,
    area: Res<VisibleArea>,
    treasury: Res<Treasury>,
    player: Query<&Position, With<Player>>,
    chests: Query<&Position, With<Chest>>,
    origin: Single<Entity, With<UiOrigin>>,
    mut labels: Query<(Entity, &ChestLabel, &mut Node, &mut Text, &mut TextColor)>,
) {
    let Ok(me) = player.single() else {
        return;
    };
    let view = visible_rect(me.0, &area);
    let visible: Vec<Vec2> = chests
        .iter()
        .map(|c| c.0)
        .filter(|at| view.contains(*at + Vec2::Y * (LABEL_ABOVE / 2.0)))
        .collect();
    if labels.iter().len() != visible.len() {
        for (entity, ..) in &labels {
            commands.entity(entity).despawn();
        }
        for k in 0..visible.len() {
            let label = commands
                .spawn((
                    ChestLabel(k),
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(LABEL_W),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Text::new(""),
                    fonts::hud(16.0),
                    TextColor(GOLD),
                    TextLayout::justify(Justify::Center),
                ))
                .id();
            commands.entity(*origin).add_child(label);
        }
        return;
    }
    let (line, afford) = price_text(treasury.price(&config.stage.chests), treasury.coins);
    let color = if afford { GOLD } else { GREY };
    for (_, label, mut node, mut text, mut text_color) in &mut labels {
        let Some(at) = visible.get(label.0) else {
            continue;
        };
        let spot = *at - view.center() + Vec2::Y * LABEL_ABOVE;
        let (left, top) = ui_at(spot);
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
        if text.0 != line {
            text.0 = line.clone();
        }
        text_color.0 = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_label_says_the_price_or_what_is_missing() {
        assert_eq!(price_text(10, 12.0), ("10 monete".to_string(), true));
        assert_eq!(price_text(10, 10.0), ("10 monete".to_string(), true));
        assert_eq!(price_text(1, 3.0), ("1 moneta".to_string(), true));
        assert_eq!(price_text(15, 9.0), ("Mancano 6".to_string(), false));
        // con il Salvadanaio le monete hanno decimali: si contano per difetto
        assert_eq!(price_text(15, 14.6), ("Mancano 1".to_string(), false));
    }
}
