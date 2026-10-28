//! Profilo del giocatore salvato sul dispositivo: esperienza e talenti imparati di ogni
//! compagno, tra una run e l'altra. Niente account né rete: un file RON nella cartella dei dati
//! dell'utente (sul web, per ora, solo in memoria: la P1 lo porterà nel `localStorage`). Le run
//! con `--autoplay`, `--god` o `--time-scale` leggono il profilo ma non lo salvano.
//!
//! A fine run (sconfitta o vittoria) ogni compagno in squadra guadagna esperienza (tempo in
//! squadra, nemici finiti, vittoria: `progression.ron: companion_xp`); ogni livello dà un punto
//! da spendere nel suo albero (`companions.ron: talents`) dal menu.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::app::RunPhase;
use crate::data::GameConfig;
use crate::data::schema::XpGain;
use crate::gameplay::combat::{RunStats, Source};
use crate::gameplay::companions::Companion;
use crate::gameplay::run::RunClock;
use crate::gameplay::talents::{LearnedTalents, active_nodes};

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
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            companions: Vec::new(),
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
        return None;
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
        let data = match std::fs::read_to_string(&path) {
            Err(_) => ProfileData::default(),
            Ok(text) => match ron::from_str::<ProfileData>(&text) {
                Ok(data) => data,
                Err(error) => {
                    warn!("profilo illeggibile ({error}): ne comincio uno nuovo");
                    let _ = std::fs::rename(&path, path.with_extension("ron.bak"));
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
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(path, text));
        if let Err(error) = written {
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
            active_nodes(&c.talents, &progress.learned).len() as u32
        });
        (level, level - 1, spent)
    }

    /// Si può imparare `node` del compagno `id`: un punto libero e il nodo precedente del suo
    /// ramo già imparato.
    pub fn can_learn(&self, config: &GameConfig, id: &str, node: &str) -> bool {
        let (_, earned, spent) = self.points(config, id);
        let progress = self.progress(id);
        let Some(companion) = config.companion(id) else {
            return false;
        };
        spent < earned
            && !progress.learned.iter().any(|n| n == node)
            && companion.talents.iter().any(|branch| {
                branch
                    .nodes
                    .iter()
                    .position(|n| n.id == node)
                    .is_some_and(|k| {
                        branch.nodes[..k]
                            .iter()
                            .all(|before| progress.learned.contains(&before.id))
                    })
            })
    }

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
mod tests {
    use super::*;

    #[test]
    fn learning_needs_points_and_the_node_before() {
        let config = GameConfig::embedded();
        let ember = config.companion("ember").unwrap();
        let (a1, a2, b1) = (
            ember.talents[0].nodes[0].id.clone(),
            ember.talents[0].nodes[1].id.clone(),
            ember.talents[1].nodes[0].id.clone(),
        );
        let mut profile = Profile::default();
        assert_eq!(profile.points(&config, "ember"), (1, 0, 0));
        assert!(
            !profile.learn(&config, "ember", &a1),
            "nessun punto a livello 1"
        );
        // due livelli: due punti
        let curve = config.progression.companion_xp;
        profile.entry("ember").xp = curve.total_for(3);
        assert!(
            !profile.can_learn(&config, "ember", &a2),
            "serve il nodo prima"
        );
        assert!(profile.learn(&config, "ember", &a1));
        assert!(!profile.learn(&config, "ember", &a1), "già imparato");
        assert!(profile.learn(&config, "ember", &b1));
        assert!(!profile.can_learn(&config, "ember", &a2), "punti finiti");
        assert_eq!(profile.points(&config, "ember"), (3, 2, 2));
        let learned = profile.learned();
        assert_eq!(learned.of("ember"), [a1.clone(), b1.clone()]);
        profile.reset("ember");
        assert_eq!(profile.points(&config, "ember"), (3, 2, 0));
    }

    #[test]
    fn a_run_awards_time_kills_and_victory() {
        let config = GameConfig::embedded();
        let mut profile = Profile::default();
        let lines = award(
            &config,
            &mut profile,
            &[("ember".into(), 960.0, 1000), ("tide".into(), 30.0, 0)],
            true,
        );
        let curve = config.progression.companion_xp;
        assert_eq!(lines[0].gain, curve.earned(960.0, 1000, true));
        assert_eq!(profile.progress("ember").xp, lines[0].gain.total());
        assert_eq!(lines[0].level_before, 1);
        assert_eq!(lines[0].level_after, curve.level(lines[0].gain.total()));
        assert_eq!(
            profile.progress("tide").xp,
            curve.earned(30.0, 0, true).total()
        );
    }

    #[test]
    fn the_profile_round_trips_through_its_file() {
        let dir = std::env::temp_dir().join(format!("pms-profile-{}", std::process::id()));
        let path = dir.join(FILE_NAME);
        let mut profile = Profile::open(path.clone());
        assert_eq!(
            profile.data,
            ProfileData::default(),
            "file assente: profilo nuovo"
        );
        profile.entry("spark").xp = 321;
        profile.entry("spark").learned.push("spark_a1".into());
        profile.save();
        let again = Profile::open(path.clone());
        assert_eq!(again.data, profile.data);
        // un file rovinato non si perde: resta come .bak
        std::fs::write(&path, "(version: 1, companions: [rotto").unwrap();
        let broken = Profile::open(path.clone());
        assert_eq!(broken.data, ProfileData::default());
        assert!(path.with_extension("ron.bak").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
