//! Profilo del giocatore salvato sul dispositivo: esperienza e talenti imparati di ogni
//! compagno, tra una run e l'altra. Niente account né rete: un file RON nella cartella dei dati
//! dell'utente, sul web lo stesso testo nel `localStorage` del browser (il percorso fa da
//! chiave). Le run con `--autoplay`, `--god` o `--time-scale` leggono il profilo ma non lo
//! salvano.
//!
//! A fine run (sconfitta o vittoria) ogni compagno in squadra guadagna esperienza (tempo in
//! squadra, nemici finiti, vittoria: `progression.ron: companion_xp`); ogni livello dà un punto
//! da spendere nel suo albero (`companions.ron: talents`) dal menu. I compagni iniziali
//! sbloccati stanno in `unlocked` e le loro regole in `unlocks`: nessuna run ne sblocca.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::app::RunPhase;
use crate::data::GameConfig;
use crate::data::schema::TalentNode;
use crate::data::schema::XpGain;
use crate::gameplay::combat::{RunStats, Source};
use crate::gameplay::companions::Companion;
use crate::gameplay::run::RunClock;
use crate::gameplay::talents::{LearnedTalents, node_rank};

mod unlocks;
pub use unlocks::{UnlockStatus, first_unlocked, is_unlocked, proposed_starter, unlock_status};

pub const PROFILE_VERSION: u32 = 1;
const FILE_NAME: &str = "profile.ron";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompanionProgress {
    pub id: String,
    pub xp: u32,
    #[serde(default)]
    pub learned: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileData {
    pub version: u32,
    #[serde(default)]
    pub companions: Vec<CompanionProgress>,
    /// Schermo intero (Opzioni); sul desktop anche all'avvio.
    #[serde(default)]
    pub fullscreen: bool,
    /// Compagni sbloccati come compagni iniziali da una condizione (`unlocks`, D-066), ordinati e
    /// senza doppioni. Quelli `Start` nei dati non ci sono: lo sono per tutti.
    #[serde(default)]
    pub unlocked: Vec<String>,
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            companions: Vec::new(),
            fullscreen: false,
            unlocked: Vec::new(),
        }
    }
}

/// Il profilo della sessione. `path` è dove si salva; `None`: solo in memoria (prove, run
/// automatiche, web).
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Profile {
    pub data: ProfileData,
    pub path: Option<PathBuf>,
}

/// Cartella dei dati dell'utente per il gioco, se il sistema ne ha una.
pub fn default_path() -> Option<PathBuf> {
    if cfg!(target_arch = "wasm32") {
        // chiave del localStorage (`web::store`)
        return Some(PathBuf::from("PixelMonsterSurvivor").join(FILE_NAME));
    }
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library").join("Application Support"))
    } else {
        env("XDG_DATA_HOME").or_else(|| env("HOME").map(|h| h.join(".local").join("share")))
    }?;
    Some(base.join("PixelMonsterSurvivor").join(FILE_NAME))
}

impl Profile {
    /// Legge il profilo da `path`; un file assente è un profilo nuovo, uno illeggibile viene
    /// messo da parte (`.bak`) invece di essere sovrascritto.
    pub fn open(path: PathBuf) -> Self {
        let data = match read_store(&path) {
            None => ProfileData::default(),
            Some(text) => match ron::from_str::<ProfileData>(&text) {
                Ok(mut data) => {
                    data.unlocked.sort();
                    data.unlocked.dedup();
                    data
                }
                Err(error) => {
                    warn!("profilo illeggibile ({error}): ne comincio uno nuovo");
                    set_aside(&path, &text);
                    ProfileData::default()
                }
            },
        };
        Self {
            data,
            path: Some(path),
        }
    }

    pub fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let text = match ron::ser::to_string_pretty(&self.data, ron::ser::PrettyConfig::default()) {
            Ok(text) => text,
            Err(error) => {
                warn!("profilo non salvato: {error}");
                return;
            }
        };
        if let Err(error) = write_store(path, &text) {
            warn!("profilo non salvato in {}: {error}", path.display());
        }
    }

    pub fn progress(&self, id: &str) -> CompanionProgress {
        self.data
            .companions
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .unwrap_or_else(|| CompanionProgress {
                id: id.to_string(),
                ..default()
            })
    }

    fn entry(&mut self, id: &str) -> &mut CompanionProgress {
        if let Some(k) = self.data.companions.iter().position(|c| c.id == id) {
            return &mut self.data.companions[k];
        }
        self.data.companions.push(CompanionProgress {
            id: id.to_string(),
            ..default()
        });
        self.data.companions.last_mut().expect("appena aggiunto")
    }

    pub fn learned(&self) -> LearnedTalents {
        LearnedTalents(
            self.data
                .companions
                .iter()
                .map(|c| (c.id.clone(), c.learned.clone()))
                .collect(),
        )
    }

    /// Livello del compagno e punti: guadagnati (uno per livello oltre il primo) e spesi.
    pub fn points(&self, config: &GameConfig, id: &str) -> (u32, u32, u32) {
        let progress = self.progress(id);
        let level = config.progression.companion_xp.level(progress.xp);
        let spent = config.companion(id).map_or(0, |c| {
            c.talents
                .iter()
                .flat_map(|b| b.nodes.iter())
                .map(|n| node_rank(&progress.learned, n))
                .sum()
        });
        (level, level - 1, spent)
    }

    /// Grado del nodo `node` del compagno `id` (0 se non imparato).
    pub fn rank(&self, config: &GameConfig, id: &str, node: &str) -> u32 {
        let progress = self.progress(id);
        find_node(config, id, node).map_or(0, |n| node_rank(&progress.learned, n))
    }

    /// Si può dare un grado in più a `node` del compagno `id`: un punto libero, il nodo non al
    /// massimo e i nodi che richiede con almeno un grado.
    pub fn can_learn(&self, config: &GameConfig, id: &str, node: &str) -> bool {
        let (_, earned, spent) = self.points(config, id);
        let progress = self.progress(id);
        let Some(n) = find_node(config, id, node) else {
            return false;
        };
        spent < earned
            && node_rank(&progress.learned, n) < n.max_rank()
            && n.requires.iter().all(|r| progress.learned.contains(r))
    }

    /// Un grado in più (l'id si ripete una volta per grado).
    pub fn learn(&mut self, config: &GameConfig, id: &str, node: &str) -> bool {
        if !self.can_learn(config, id, node) {
            return false;
        }
        self.entry(id).learned.push(node.to_string());
        true
    }

    /// Libera tutti i punti del compagno.
    pub fn reset(&mut self, id: &str) {
        self.entry(id).learned.clear();
    }
}

/// Testo del profilo: dal file, sul web dal `localStorage` (chiave = percorso).
fn read_store(path: &std::path::Path) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::web::load(&path.to_string_lossy())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(path).ok()
    }
}

fn write_store(path: &std::path::Path, text: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::web::store(&path.to_string_lossy(), text)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        path.parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(path, text))
            .map_err(|e| e.to_string())
    }
}

/// Un profilo rovinato non si perde: resta accanto come `.bak`.
fn set_aside(path: &std::path::Path, text: &str) {
    let backup = path.with_extension("ron.bak");
    #[cfg(target_arch = "wasm32")]
    {
        let _ = crate::web::store(&backup.to_string_lossy(), text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
        let _ = std::fs::rename(path, backup);
    }
}

/// Il nodo `node` dell'albero del compagno `id`.
fn find_node<'a>(config: &'a GameConfig, id: &str, node: &str) -> Option<&'a TalentNode> {
    config
        .companion(id)?
        .talents
        .iter()
        .flat_map(|b| b.nodes.iter())
        .find(|n| n.id == node)
}

/// Quando è entrato in squadra un compagno (secondi della run).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct JoinedAt(pub f32);

/// L'esperienza guadagnata nella run da un compagno, per il riepilogo.
#[derive(Debug, Clone, PartialEq)]
pub struct XpLine {
    pub id: String,
    pub gain: XpGain,
    pub level_before: u32,
    pub level_after: u32,
}

/// Esperienza della run appena finita, compagno per compagno (vuota durante la run).
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct RunXp(pub Vec<XpLine>);

/// Assegna l'esperienza della run ai compagni `members` (id, secondi in squadra, nemici
/// finiti) e restituisce le righe del riepilogo.
pub fn award(
    config: &GameConfig,
    profile: &mut Profile,
    members: &[(String, f32, u32)],
    victory: bool,
) -> Vec<XpLine> {
    let curve = &config.progression.companion_xp;
    members
        .iter()
        .map(|(id, seconds, kills)| {
            let gain = curve.earned(*seconds, *kills, victory);
            let entry = profile.entry(id);
            let level_before = curve.level(entry.xp);
            entry.xp = entry.xp.saturating_add(gain.total());
            XpLine {
                id: id.clone(),
                gain,
                level_before,
                level_after: curve.level(entry.xp),
            }
        })
        .collect()
}

/// Set in cui si assegna l'esperienza: i risultati si costruiscono dopo.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileAward;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Profile>()
            .init_resource::<RunXp>()
            .add_observer(note_join_time)
            .add_systems(
                OnEnter(crate::app::AppState::InRun),
                |mut xp: ResMut<RunXp>| xp.0.clear(),
            )
            .add_systems(OnEnter(RunPhase::Defeated), award_run.in_set(ProfileAward))
            .add_systems(OnEnter(RunPhase::Victory), award_run.in_set(ProfileAward));
    }
}

fn note_join_time(add: On<Add<Companion>>, mut commands: Commands, clock: Option<Res<RunClock>>) {
    let at = clock.map_or(0.0, |c| c.elapsed);
    commands.entity(add.entity).try_insert(JoinedAt(at));
}

fn award_run(
    config: Res<GameConfig>,
    clock: Res<RunClock>,
    stats: Res<RunStats>,
    phase: Res<State<RunPhase>>,
    mut profile: ResMut<Profile>,
    mut xp: ResMut<RunXp>,
    party: Query<(&crate::data::DataId, &Source, &JoinedAt, &Companion)>,
) {
    if !xp.0.is_empty() {
        return; // già assegnata in questa run
    }
    let mut members: Vec<(usize, String, f32, u32)> = party
        .iter()
        .map(|(id, source, joined, companion)| {
            let seconds = (clock.elapsed - joined.0).max(0.0);
            (
                companion.slot,
                id.0.clone(),
                seconds,
                stats.tally(*source).kills,
            )
        })
        .collect();
    members.sort_by_key(|m| m.0);
    let members: Vec<(String, f32, u32)> = members
        .into_iter()
        .map(|(_, id, seconds, kills)| (id, seconds, kills))
        .collect();
    let victory = *phase.get() == RunPhase::Victory;
    xp.0 = award(&config, &mut profile, &members, victory);
    for line in &xp.0 {
        info!(
            "{} +{} esperienza (livello {} → {})",
            line.id,
            line.gain.total(),
            line.level_before,
            line.level_after
        );
    }
    profile.save();
}

#[cfg(test)]
mod tests;
