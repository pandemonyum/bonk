//! Fila delle abilità sotto la barra XP (handoff dell'art pass, §2): un posto per ogni posto
//! della squadra (`player.ron: companion_slots`), nell'ordine dei posti. Un compagno mostra
//! l'icona del suo elemento e sotto la barra di ricarica del suo attacco (piena = pronto; le
//! foglie girano sempre, quindi la loro resta piena); un posto libero mostra il lucchetto, per
//! ricordare che un compagno può ancora arrivare dal draft.

use bevy::prelude::*;

use super::icons::{UI_PX_PER_TEXEL, element_icon_path, ui_icon};
use crate::app::AppState;
use crate::data::schema::Element;
use crate::data::{DataId, GameConfig};
use crate::gameplay::attacks::AutoAttack;
use crate::gameplay::chain::ChainAttack;
use crate::gameplay::companions::Companion;
use crate::gameplay::pulse::PulseAttack;

/// Contenitore della fila, creato dentro il pannello di `status`.
#[derive(Component)]
pub struct AbilityRow;

/// Riempimento della barra di ricarica del posto `0`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct CooldownFill(usize);

/// Lato del riquadro di un posto e larghezza della barra, in texel del canvas.
const SLOT_TEXELS: f32 = 16.0;
const BAR_TEXELS: f32 = 2.0;
const BAR_BACK: Color = Color::srgb_u8(0x0f, 0x16, 0x26);
const LOCKED: Color = Color::srgba(1.0, 1.0, 1.0, 0.55);

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (build_row, show_cooldowns)
                .chain()
                .run_if(in_state(AppState::InRun)),
        );
    }
}

/// Frazione di ricarica di un attacco: vicina a 0 appena sparato, 1 quando spara al passo
/// fisso successivo. Un attacco pronto senza bersaglio resta a `interval - step`
/// (`attacks::attack_tick`), quindi la carica vista dall'HUD è quella del passo dopo.
pub fn readiness(charge: f32, interval: f32, step: f32) -> f32 {
    if interval > 0.0 {
        // il margine assorbe l'arrotondamento di (interval - step) + step
        ((charge + step) / interval + 1e-4).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

/// Colore della barra per elemento (handoff: barre ATT/VEL/AREA).
fn bar_color(element: Element) -> Color {
    match element {
        Element::Fire => Color::srgb_u8(0xff, 0xaa, 0x45),
        Element::Water => Color::srgb_u8(0x6c, 0xc7, 0xc2),
        Element::Nature => Color::srgb_u8(0xb8, 0xe0, 0x5a),
        Element::Electric => Color::srgb_u8(0xff, 0xd6, 0x5c),
    }
}

/// Chi occupa ogni posto della squadra (id del compagno), per tutti i posti dei dati.
fn occupants(slots: usize, companions: &Query<(&DataId, &Companion)>) -> Vec<Option<String>> {
    let mut taken = vec![None; slots];
    for (id, companion) in companions {
        if let Some(slot) = taken.get_mut(companion.slot) {
            *slot = Some(id.0.clone());
        }
    }
    taken
}

/// Ricostruisce la fila quando cambia la squadra (o il numero di posti dopo una ricarica) e
/// quando la fila stessa è nuova (run ricominciata).
fn build_row(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    companions: Query<(&DataId, &Companion)>,
    row: Single<Entity, With<AbilityRow>>,
    mut shown: Local<Option<(Entity, Vec<Option<String>>)>>,
) {
    let wanted = (
        *row,
        occupants(config.player.companion_slots.len(), &companions),
    );
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    commands.entity(*row).despawn_related::<Children>();
    for (slot, occupant) in wanted.1.iter().enumerate() {
        let data = occupant.as_deref().and_then(|id| config.companion(id));
        let cell = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(UI_PX_PER_TEXEL),
                ..default()
            })
            .id();
        let frame = commands
            .spawn(Node {
                width: px(SLOT_TEXELS * UI_PX_PER_TEXEL),
                height: px(SLOT_TEXELS * UI_PX_PER_TEXEL),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .id();
        let icon = match data {
            Some(companion) => {
                let path = element_icon_path(companion.element);
                commands.spawn(ui_icon(&assets, path, 16.0, 16.0)).id()
            }
            None => commands
                .spawn((
                    ImageNode {
                        color: LOCKED,
                        ..ImageNode::new(assets.load("ui/lock.png"))
                    },
                    Node {
                        width: px(12.0 * UI_PX_PER_TEXEL),
                        height: px(14.0 * UI_PX_PER_TEXEL),
                        ..default()
                    },
                ))
                .id(),
        };
        commands.entity(frame).add_child(icon);
        let bar = commands
            .spawn((
                Node {
                    width: px(SLOT_TEXELS * UI_PX_PER_TEXEL),
                    height: px(BAR_TEXELS * UI_PX_PER_TEXEL),
                    ..default()
                },
                BackgroundColor(if data.is_some() {
                    BAR_BACK
                } else {
                    Color::NONE
                }),
            ))
            .id();
        if let Some(companion) = data {
            let fill = commands
                .spawn((
                    CooldownFill(slot),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(bar_color(companion.element)),
                ))
                .id();
            commands.entity(bar).add_child(fill);
        }
        commands.entity(cell).add_children(&[frame, bar]);
        commands.entity(*row).add_child(cell);
    }
    *shown = Some(wanted);
}

fn show_cooldowns(
    fixed: Res<Time<Fixed>>,
    companions: Query<(
        &Companion,
        Option<&AutoAttack>,
        Option<&PulseAttack>,
        Option<&ChainAttack>,
    )>,
    mut fills: Query<(&CooldownFill, &mut Node)>,
) {
    let step = fixed.timestep().as_secs_f32();
    for (fill, mut node) in &mut fills {
        let ready = companions.iter().find(|(c, ..)| c.slot == fill.0).map_or(
            1.0,
            |(_, shots, wave, bolt)| match (shots, wave, bolt) {
                (Some(a), _, _) => readiness(a.charge, a.interval, step),
                (None, Some(w), _) => readiness(w.charge, w.interval, step),
                (None, None, Some(c)) => readiness(c.charge, c.interval, step),
                // foglie: sempre in azione
                (None, None, None) => 1.0,
            },
        );
        // a texel interi, come le barre disegnate negli sprite
        let texels = (ready * SLOT_TEXELS).floor();
        let width = px(texels * UI_PX_PER_TEXEL);
        if node.width != width {
            node.width = width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gameplay::attacks::attack_tick;

    const STEP: f32 = 1.0 / 60.0;

    #[test]
    fn readiness_goes_from_empty_to_full() {
        assert!(readiness(0.0, 1.1, STEP) < 0.02);
        assert!((readiness(0.55 - STEP, 1.1, STEP) - 0.5).abs() < 1e-3);
        assert_eq!(readiness(5.0, 1.1, STEP), 1.0);
        assert_eq!(readiness(0.3, 0.0, STEP), 1.0);
    }

    /// Un attacco pronto che aspetta un bersaglio mostra la barra piena, per gli intervalli
    /// dei dati e per quelli accorciati dagli upgrade.
    #[test]
    fn a_ready_attack_waiting_for_a_target_reads_full() {
        for interval in [1.1, 2.0, 0.935, 0.77, 0.5] {
            let mut charge = 0.0;
            for _ in 0..600 {
                charge = attack_tick(charge, interval, STEP, false).0;
            }
            let texels = (readiness(charge, interval, STEP) * SLOT_TEXELS).floor();
            assert_eq!(texels, SLOT_TEXELS, "intervallo {interval}");
        }
    }
}
