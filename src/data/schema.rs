//! Schema dei file in `game-data/`. `deny_unknown_fields`: un refuso nel nome di un campo è un
//! errore, non un valore ignorato in silenzio.
//!
//! Versioni: 1 (P0.0) → 2 (P0.3: ondate dello spawn director negli stage, HP e invulnerabilità
//! del player, danno da contatto dei nemici) → 3 (P0.4: XP dei nemici, raggio di raccolta,
//! `progression.ron` e `upgrades.ron`) → 4 (P0.5: attacchi con `pattern` proiettile / onda /
//! orbita, slot dei compagni nel player, carte che aggiungono un compagno).

use serde::Deserialize;

/// Versione dello schema supportata da questo codice.
pub const SCHEMA_VERSION: u32 = 4;

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
    /// Posti per i compagni, il primo è del compagno iniziale. Ognuno ha la sua posizione nella
    /// formazione dietro al player.
    pub companion_slots: Vec<SlotData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotData {
    /// Gradi di rotazione rispetto a "dritto dietro al player" (positivi = verso sinistra).
    pub angle_deg: f32,
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
/// +30% sul valore base dei dati). Gli effetti "Companion" valgono per tutti i compagni e
/// per tutti i pattern di attacco, dove hanno senso.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Effect {
    CompanionDamagePct(f32),
    /// Negativo = attacchi più frequenti (per le foglie: colpi più ravvicinati sullo stesso nemico).
    CompanionIntervalPct(f32),
    /// Proiettili per raffica, o foglie in orbita.
    CompanionProjectiles(u32),
    CompanionPierce(u32),
    /// Gittata dei proiettili (ne allunga anche la durata), raggio dell'onda, raggio dell'orbita.
    CompanionRangePct(f32),
    ProjectileSpeedPct(f32),
    /// HP massimi in più; cura della stessa quantità.
    MaxHp(f32),
    RegenPerSecond(f32),
    MoveSpeedPct(f32),
    PickupRadiusPct(f32),
    /// Cura immediata, nessun effetto permanente.
    Heal(f32),
    /// Il compagno con questo id (in `companions.ron`) si unisce alla squadra nel primo posto
    /// libero. La carta è proposta solo se c'è un posto libero.
    JoinCompanion(String),
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

/// Un attacco automatico. Danno, intervallo e portata sono comuni a tutti i pattern, così gli
/// upgrade "Companion" valgono per tutti.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackData {
    pub id: String,
    pub damage: f32,
    /// Proiettile e onda: secondi tra due attacchi. Orbita: secondi prima che le foglie possano
    /// colpire di nuovo lo stesso nemico.
    pub interval_s: f32,
    /// Proiettile: distanza massima del bersaglio. Onda: raggio finale. Orbita: raggio dell'orbita.
    pub range: f32,
    pub pattern: AttackPattern,
}

impl AttackData {
    pub fn projectile(&self) -> Option<&ProjectileData> {
        match &self.pattern {
            AttackPattern::Projectile(p) => Some(p),
            _ => None,
        }
    }

    pub fn pulse(&self) -> Option<&PulseData> {
        match &self.pattern {
            AttackPattern::Pulse(p) => Some(p),
            _ => None,
        }
    }

    pub fn orbit(&self) -> Option<&OrbitData> {
        match &self.pattern {
            AttackPattern::Orbit(o) => Some(o),
            _ => None,
        }
    }

    /// Sprite dei colpi (proiettile, goccia dell'onda, foglia).
    pub fn sprite(&self) -> &str {
        match &self.pattern {
            AttackPattern::Projectile(p) => &p.sprite,
            AttackPattern::Pulse(p) => &p.sprite,
            AttackPattern::Orbit(o) => &o.sprite,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum AttackPattern {
    /// Proiettili mirati sul nemico più vicino entro `range`.
    Projectile(ProjectileData),
    /// Onda che si allarga attorno al player fino a `range`, colpisce e rallenta.
    Pulse(PulseData),
    /// Foglie che girano attorno al player a distanza `range`.
    Orbit(OrbitData),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectileData {
    /// px/s.
    pub speed: f32,
    pub radius: f32,
    /// Secondi di volo prima che un proiettile mancato sparisca (portata = velocità × durata).
    pub lifetime_s: f32,
    pub sprite: String,
    /// Proiettili per attacco, a ventaglio.
    pub count: u32,
    pub spread_deg: f32,
    /// Nemici extra attraversati da ogni proiettile.
    pub pierce: u32,
    /// Altezza di partenza del colpo sopra i piedi di chi attacca.
    pub muzzle_height: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PulseData {
    /// px/s a cui il fronte dell'onda si allarga.
    pub speed: f32,
    /// Frazione della velocità che resta ai nemici colpiti (0,6 = rallentati del 40%).
    pub slow: f32,
    pub slow_s: f32,
    /// Altezza del centro dell'onda sopra i piedi del player.
    pub center_height: f32,
    /// Sprite delle gocce che disegnano il fronte.
    pub sprite: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbitData {
    /// Foglie in orbita, equidistanti.
    pub count: u32,
    /// Gradi al secondo (positivi = senso antiorario).
    pub angular_speed_deg: f32,
    /// Raggio colpibile di una foglia.
    pub radius: f32,
    /// Altezza del centro dell'orbita sopra i piedi del player.
    pub center_height: f32,
    pub sprite: String,
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
