//! Schema dei file in `game-data/` (versione 1). `deny_unknown_fields`: un refuso nel nome di
//! un campo è un errore, non un valore ignorato in silenzio.

use serde::Deserialize;

/// Versione dello schema supportata da questo codice.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerFile {
    pub version: u32,
    pub sprite: String,
    pub speed: f32,
    pub footprint_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Element {
    Fire,
    Water,
    Nature,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowData {
    pub distance: f32,
    pub max_speed: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionData {
    pub id: String,
    pub display_name: String,
    pub element: Element,
    pub sprite: String,
    pub attack: String,
    pub follow: FollowData,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionsFile {
    pub version: u32,
    pub companions: Vec<CompanionData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackData {
    pub id: String,
    pub damage: f32,
    pub interval_s: f32,
    pub range: f32,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    /// Secondi di volo prima che un proiettile mancato sparisca (portata = velocità × durata).
    pub projectile_lifetime_s: f32,
    pub projectile_sprite: String,
    pub projectiles: u32,
    pub spread_deg: f32,
    pub pierce: u32,
    pub muzzle_height: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttacksFile {
    pub version: u32,
    pub attacks: Vec<AttackData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HurtboxData {
    pub radius: f32,
    pub center_offset: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyData {
    pub id: String,
    pub display_name: String,
    pub sprite: String,
    pub max_hp: f32,
    pub speed: f32,
    pub hurtbox: HurtboxData,
    pub footprint_radius: f32,
    pub contact_distance: f32,
    pub separation_radius: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemiesFile {
    pub version: u32,
    pub enemies: Vec<EnemyData>,
}

/// Ostacoli dello stage. La distanza minima tra ostacoli (attraversabilità) resta nel codice.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleParams {
    /// Probabilità che un tile candidato ospiti un ostacolo, prima dello scarto per vicinanza.
    pub density: f32,
    /// Raggio libero attorno al punto di partenza del player.
    pub safe_radius: f32,
    /// Probabilità che un ostacolo sull'erba sia un albero invece di una roccia.
    pub tree_chance: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageData {
    pub id: String,
    pub display_name: String,
    pub starting_companion: String,
    pub enemy: String,
    pub initial_enemies: u32,
    pub population_cap: u32,
    pub initial_ring: (f32, f32),
    pub respawn_ring: (f32, f32),
    pub obstacles: ObstacleParams,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagesFile {
    pub version: u32,
    pub stages: Vec<StageData>,
}
