//! Talenti nel menu (T): a sinistra i compagni con il loro livello, al centro l'albero del
//! compagno scelto (ritratto, livello, barra dell'esperienza, punti; una colonna per ramo con i
//! nodi al loro posto, i gradi "r/max" e i collegamenti), a destra il nodo selezionato (gradi,
//! effetti, prossimo grado). Clic o frecce per scegliere un nodo, Invio o clic sul nodo scelto
//! per un grado in più, Tab o clic sulla lista per cambiare compagno, R azzera, T o Esc chiude.
//! Ogni modifica si salva subito nel profilo. Il disegno è in `talents_view`.

use bevy::prelude::*;

use super::menu::MenuChoice;
use super::talents_view::{ViewTarget, spawn_view};
use crate::app::AppState;
use crate::data::GameConfig;
use crate::data::schema::TalentNode;
use crate::profile::Profile;

/// Il pannello dei talenti è aperto (sul compagno di `MenuChoice`) e il nodo selezionato.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct TalentPanel {
    pub open: bool,
    pub selected: Option<String>,
}

pub fn panel_closed(panel: Res<TalentPanel>) -> bool {
    !panel.open
}

#[derive(Component)]
pub(super) struct PanelRoot;

/// Stato di un nodo per un compagno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Al grado massimo.
    Maxed,
    /// Con almeno un grado, ne può prendere altri (ma ora no).
    Learned,
    /// Si può dargli un grado adesso.
    Available,
    Locked,
}

pub fn node_state(
    profile: &Profile,
    config: &GameConfig,
    id: &str,
    node: &TalentNode,
) -> NodeState {
    let rank = profile.rank(config, id, &node.id);
    if rank >= node.max_rank() {
        NodeState::Maxed
    } else if profile.can_learn(config, id, &node.id) {
        NodeState::Available
    } else if rank > 0 {
        NodeState::Learned
    } else {
        NodeState::Locked
    }
}

/// Posto di un nodo nella griglia di tutta la schermata: colonna (due per ramo, il centro a
/// metà) e riga.
fn grid_spot(branch: usize, node: &TalentNode) -> Vec2 {
    let col = if node.col >= 2 { 0.5 } else { node.col as f32 };
    Vec2::new(branch as f32 * 2.0 + col, node.row as f32)
}

/// Il nodo più vicino a `from` nella direzione `dir` (frecce; y verso il basso), se c'è.
pub fn step_selection<'a>(
    nodes: &[(usize, &'a TalentNode)],
    from: &str,
    dir: Vec2,
) -> Option<&'a TalentNode> {
    let (b, here) = nodes.iter().find(|(_, n)| n.id == from)?;
    let at = grid_spot(*b, here);
    nodes
        .iter()
        .filter(|(_, n)| n.id != from)
        .map(|(b, n)| (*n, grid_spot(*b, n) - at))
        .filter(|(_, d)| d.dot(dir) > 0.01)
        // conta di più restare in linea che andare lontano nella direzione
        .min_by(|(_, a), (_, b)| {
            let cost = |d: &Vec2| d.length() + 2.0 * d.dot(dir.perp()).abs();
            cost(a).total_cmp(&cost(b))
        })
        .map(|(n, _)| n)
}

pub struct TalentsMenuPlugin;

impl Plugin for TalentsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TalentPanel>()
            .add_systems(OnExit(AppState::MainMenu), close_panel)
            .add_systems(
                Update,
                (talent_keys, show_panel)
                    .chain()
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

fn close_panel(mut panel: ResMut<TalentPanel>) {
    panel.open = false;
}

/// Un grado in più al nodo, se si può, e salvataggio.
pub(super) fn learn(profile: &mut Profile, config: &GameConfig, id: &str, node: &str) {
    if profile.learn(config, id, node) {
        info!(
            "talento imparato: {node} ({})",
            profile.rank(config, id, node)
        );
        profile.save();
    }
}

pub(super) fn talent_keys(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut panel: ResMut<TalentPanel>,
    mut choice: ResMut<MenuChoice>,
    mut profile: ResMut<Profile>,
) {
    if !panel.open {
        if keys.just_pressed(KeyCode::KeyT) {
            panel.open = true;
        }
        return;
    }
    if keys.any_just_pressed([KeyCode::KeyT, KeyCode::Escape]) {
        panel.open = false;
        return;
    }
    let ids: Vec<&str> = config.companions.iter().map(|c| c.id.as_str()).collect();
    if keys.just_pressed(KeyCode::Tab) && !ids.is_empty() {
        let count = ids.len();
        let index = ids.iter().position(|id| *id == choice.0).unwrap_or(0);
        let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let next = if back {
            (index + count - 1) % count
        } else {
            (index + 1) % count
        };
        choice.0 = ids[next].to_string();
        panel.selected = None;
    }
    let Some(companion) = config.companion(&choice.0) else {
        return;
    };
    let id = companion.id.clone();
    let nodes: Vec<(usize, &TalentNode)> = companion
        .talents
        .iter()
        .enumerate()
        .flat_map(|(b, branch)| branch.nodes.iter().map(move |n| (b, n)))
        .collect();
    // il nodo selezionato è sempre uno del compagno (il primo, all'apertura)
    let valid = panel
        .selected
        .as_ref()
        .is_some_and(|s| nodes.iter().any(|(_, n)| n.id == *s));
    if !valid {
        let first = nodes.first().map(|(_, n)| n.id.clone());
        if panel.selected != first {
            panel.selected = first;
        }
    }
    let arrows = [
        (KeyCode::ArrowUp, Vec2::NEG_Y),
        (KeyCode::ArrowDown, Vec2::Y),
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::ArrowRight, Vec2::X),
    ];
    for (key, dir) in arrows {
        if keys.just_pressed(key)
            && let Some(from) = panel.selected.clone()
            && let Some(next) = step_selection(&nodes, &from, dir)
        {
            panel.selected = Some(next.id.clone());
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        profile.reset(&id);
        profile.save();
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space])
        && let Some(node) = panel.selected.clone()
    {
        learn(&mut profile, &config, &id, &node);
    }
}

/// Il pannello si ricostruisce quando cambiano apertura, selezione, compagno o profilo.
fn show_panel(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<GameConfig>,
    panel: Res<TalentPanel>,
    choice: Res<MenuChoice>,
    profile: Res<Profile>,
    roots: Query<Entity, With<PanelRoot>>,
) {
    let changed = panel.is_changed() || choice.is_changed() || profile.is_changed();
    if !changed && (roots.is_empty() != panel.open) {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    if !panel.open || config.companion(&choice.0).is_none() {
        return;
    }
    let target = ViewTarget {
        companion: choice.0.clone(),
        selected: panel.selected.clone(),
    };
    spawn_view(&mut commands, &assets, &config, &profile, &target);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_follow_points_ranks_and_requirements() {
        let config = GameConfig::embedded();
        let mut profile = Profile::default();
        let ember = config.companion("ember").unwrap();
        let first = &ember.talents[0].nodes[0];
        let second = &ember.talents[0].nodes[1];
        assert_eq!(
            node_state(&profile, &config, "ember", first),
            NodeState::Locked
        );
        let curve = config.progression.companion_xp;
        profile
            .data
            .companions
            .push(crate::profile::CompanionProgress {
                id: "ember".into(),
                xp: curve.total_for(curve.max_level),
                learned: vec![],
            });
        assert_eq!(
            node_state(&profile, &config, "ember", first),
            NodeState::Available
        );
        assert_eq!(
            node_state(&profile, &config, "ember", second),
            NodeState::Locked
        );
        for _ in 0..first.max_rank() {
            assert!(profile.learn(&config, "ember", &first.id));
        }
        assert_eq!(
            node_state(&profile, &config, "ember", first),
            NodeState::Maxed
        );
    }

    #[test]
    fn arrows_move_to_the_nearest_node_in_that_direction() {
        let config = GameConfig::embedded();
        let ember = config.companion("ember").unwrap();
        let nodes: Vec<(usize, &TalentNode)> = ember
            .talents
            .iter()
            .enumerate()
            .flat_map(|(b, br)| br.nodes.iter().map(move |n| (b, n)))
            .collect();
        let top = &ember.talents[0].nodes[0];
        let below = step_selection(&nodes, &top.id, Vec2::Y).unwrap();
        assert_eq!(below.row, top.row + 1, "giù: la riga sotto");
        assert!(
            step_selection(&nodes, &top.id, Vec2::NEG_Y).is_none(),
            "sopra non c'è nulla"
        );
        let right = step_selection(&nodes, &top.id, Vec2::X).unwrap();
        assert_eq!(right.row, top.row, "a destra: stessa riga");
    }
}
