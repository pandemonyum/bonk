//! Schema dei file in `game-data/`. `deny_unknown_fields`: un refuso nel nome di un campo è un
//! errore, non un valore ignorato in silenzio.
//!
//! Versioni: 1 (P0.0) → 2 (P0.3: ondate dello spawn director negli stage, HP e invulnerabilità
//! del player, danno da contatto dei nemici) → 3 (P0.4: XP dei nemici, raggio di raccolta,
//! `progression.ron` e `upgrades.ron`).

use serde::Deserialize;

/// Versione dello schema supportata da questo codice.
pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerFile {
    pub version: u32,
    pub sprite: String,
    pub speed: f32,
    pub footprint_radius: f32,
    pub max_hp: f32,
    /// Secondi di invulnerabilità dopo un colpo subito.
    pub invulnerability_s: f32,
    /// Distanza dai piedi entro cui le gemme XP vengono attirate.
    pub pickup_radius: f32,
}

/// XP per passare dal livello `n` al `n + 1`: `base + per_level × (n − 1)`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XpCurve {
    pub base: f32,
    pub per_level: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GemData {
    pub sprite: String,
    /// Velocità con cui una gemma attirata vola verso il player.
    pub magnet_speed: f32,
    /// Distanza a cui la gemma è raccolta.
    pub collect_distance: f32,
    /// Gemme a terra al massimo: oltre, l'XP di una gemma nuova si somma a una esistente.
    pub max_on_field: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
}

/// Peso relativo di estrazione per rarità nel draft.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RarityWeights {
    pub common: f32,
    pub uncommon: f32,
    pub rare: f32,
}

impl RarityWeights {
    pub fn of(&self, rarity: Rarity) -> f32 {
        match rarity {
            Rarity::Common => self.common,
            Rarity::Uncommon => self.uncommon,
            Rarity::Rare => self.rare,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionFile {
    pub version: u32,
    pub xp_to_level: XpCurve,
    pub gem: GemData,
    /// Carte proposte a ogni livello.
    pub draft_size: u32,
    pub rarity_weights: RarityWeights,
}

/// Effetto di uno stack di upgrade. Le percentuali si sommano tra stack (due stack di +15% =
/// +30% sul valore base dei dati). Gli effetti "Companion" valgono per tutti i compagni.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Effect {
    CompanionDamagePct(f32),
    /// Negativo = attacchi più frequenti.
    CompanionIntervalPct(f32),
    CompanionProjectiles(u32),
    CompanionPierce(u32),
    /// Allunga anche la durata del proiettile, così la portata cresce con la gittata.
    CompanionRangePct(f32),
    ProjectileSpeedPct(f32),
    /// HP massimi in più; cura della stessa quantità.
    MaxHp(f32),
    RegenPerSecond(f32),
    MoveSpeedPct(f32),
    PickupRadiusPct(f32),
    /// Cura immediata, nessun effetto permanente.
    Heal(f32),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradeData {
    pub id: String,
    /// Titolo della carta.
    pub name: String,
    /// Testo della carta: cosa fa uno stack.
    pub description: String,
    pub rarity: Rarity,
    pub max_stack: u32,
    /// Id di upgrade da avere già (almeno uno stack ciascuno) perché questo venga proposto.
    pub requires: Vec<String>,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradesFile {
    pub version: u32,
    pub upgrades: Vec<UpgradeData>,
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
    /// Danno inflitto al player a contatto (una volta per finestra di invulnerabilità).
    pub contact_damage: f32,
    /// XP della gemma lasciata alla morte.
    pub xp: u32,
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

/// Un'ondata dello spawn director: un archetipo che entra in scena da `start_s` con un ritmo
/// che cresce linearmente nel tempo.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaveData {
    /// Id in `enemies.ron`.
    pub enemy: String,
    /// Secondo della run da cui l'ondata è attiva.
    pub start_s: f32,
    /// Nemici al minuto all'inizio dell'ondata.
    pub rate_per_min: f32,
    /// Nemici al minuto in più per ogni minuto trascorso da `start_s`.
    pub rate_growth_per_min: f32,
    /// Nemici di questo archetipo vivi al massimo nello stesso momento.
    pub max_alive: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnPlan {
    /// Anello (min, max) in px attorno al player in cui nascono i nemici; i punti dentro la
    /// vista della camera, fuori dall'arena o dentro un ostacolo vengono scartati.
    pub ring: (f32, f32),
    /// Nemici vivi al massimo in totale.
    pub max_alive: u32,
    /// Nemici della prima ondata creati all'avvio della run.
    pub initial_burst: u32,
    pub waves: Vec<WaveData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageData {
    pub id: String,
    pub display_name: String,
    pub starting_companion: String,
    pub spawns: SpawnPlan,
    pub obstacles: ObstacleParams,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagesFile {
    pub version: u32,
    pub stages: Vec<StageData>,
}
