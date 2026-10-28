//! Schema dei file in `game-data/`. `deny_unknown_fields`: un refuso nel nome di un campo è un
//! errore, non un valore ignorato in silenzio.
//!
//! Versioni: 1 (P0.0) → 2 (P0.3: ondate dello spawn director negli stage, HP e invulnerabilità
//! del player, danno da contatto dei nemici) → 3 (P0.4: XP dei nemici, raggio di raccolta,
//! `progression.ron` e `upgrades.ron`) → 4 (P0.5: attacchi con `pattern` proiettile / onda /
//! orbita, slot dei compagni nel player, carte che aggiungono un compagno) → 5 (P0.6: durata
//! della run, difficoltà per minuto, élite e boss negli stage, cariche e colpi ad area
//! telegrafati nei nemici).

use serde::Deserialize;

mod attacks;
mod run;
mod talents;
mod treasure;
pub use attacks::*;
pub use run::*;
pub use talents::*;
pub use treasure::*;

/// Versione dello schema supportata da questo codice.
pub const SCHEMA_VERSION: u32 = 5;

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
    /// Solo nei forzieri, mai nel draft dei livelli.
    Relic,
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
            Rarity::Relic => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionFile {
    pub version: u32,
    pub xp_to_level: XpCurve,
    pub gem: GemData,
    pub coin: CoinData,
    pub companion_xp: CompanionXp,
    /// Carte proposte a ogni livello.
    pub draft_size: u32,
    pub rarity_weights: RarityWeights,
    /// Livelli del player a cui si sceglie un compagno nuovo: il draft di quel livello propone
    /// solo carte "nuovo compagno" e negli altri non ne escono. Uno per ogni posto oltre al
    /// primo, in ordine crescente.
    #[serde(default)]
    pub companion_levels: Vec<u32>,
}

impl ProgressionFile {
    pub fn is_companion_level(&self, level: u32) -> bool {
        self.companion_levels.contains(&level)
    }

    /// Il prossimo livello dei compagni dopo `level`, se c'è.
    pub fn next_companion_level(&self, level: u32) -> Option<u32> {
        self.companion_levels.iter().copied().find(|&l| l > level)
    }
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
    /// Valore delle monete raccolte in più (100 = doppio).
    CoinGainPct(f32),
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
    Electric,
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
    /// Fotogramma del foglio animato in `sprite` (4 colonne; righe: fermo, cammino, attacco).
    /// Assente: immagine fissa.
    #[serde(default)]
    pub frame: Option<SheetFrame>,
    /// Due rami di talenti, sbloccati con i livelli del compagno tra una run e l'altra.
    #[serde(default)]
    pub talents: Vec<TalentBranch>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionsFile {
    pub version: u32,
    pub companions: Vec<CompanionData>,
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
    /// Carica telegrafata (élite): si ferma, mostra la traiettoria, poi scatta.
    #[serde(default)]
    pub charge: Option<ChargeData>,
    /// Colpo ad area telegrafato (boss): segna un cerchio dove sta il player, poi colpisce.
    #[serde(default)]
    pub slam: Option<SlamData>,
    /// Monete lasciate alla morte (nessuna se manca).
    #[serde(default)]
    pub coins: CoinDrop,
    /// Fotogramma del foglio animato in `sprite` (4 colonne; righe: movimento, sconfitta e, per
    /// chi carica o colpisce ad area, preparazione e attacco). Assente: immagine fissa.
    #[serde(default)]
    pub frame: Option<SheetFrame>,
}

/// Misure di un fotogramma in texel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetFrame {
    pub w: u32,
    pub h: u32,
}

/// Carica: ogni `every_s` secondi, se il player è entro `range`, il nemico si ferma per
/// `windup_s` mostrando la direzione, poi corre a `speed` per `duration_s`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargeData {
    pub every_s: f32,
    pub windup_s: f32,
    pub speed: f32,
    pub duration_s: f32,
    pub range: f32,
}

/// Colpo ad area: ogni `every_s` secondi, se il player è entro `range`, il cerchio di raggio
/// `radius` attorno al player resta segnato per `windup_s`, poi chi ci sta dentro prende
/// `damage`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlamData {
    pub every_s: f32,
    pub windup_s: f32,
    pub radius: f32,
    pub damage: f32,
    pub range: f32,
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
    /// Compagno proposto per primo nel menu e usato senza menu (headless, autoplay).
    pub starting_companion: String,
    pub spawns: SpawnPlan,
    pub run: RunPlan,
    pub chests: ChestPlan,
    pub obstacles: ObstacleParams,
}

/// Arco della run: difficoltà che cresce, élite periodiche, boss alla fine.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPlan {
    /// Secondi di gioco prima del boss: a quel punto le ondate si fermano e arriva il boss;
    /// sconfiggerlo è la vittoria.
    pub duration_s: f32,
    /// HP in più per minuto dei nemici di ondate ed élite, sul valore base (0,15 = +15%).
    pub hp_growth_per_min: f32,
    /// Danno da contatto in più per minuto, come sopra.
    pub damage_growth_per_min: f32,
    pub elites: ElitePlan,
    /// Masse di nemici a istanti fissi, oltre alle ondate continue.
    #[serde(default)]
    pub hordes: HordePlan,
    /// Id del boss in `enemies.ron`.
    pub boss: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElitePlan {
    /// Id dell'élite in `enemies.ron`.
    pub enemy: String,
    pub first_s: f32,
    pub every_s: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagesFile {
    pub version: u32,
    pub stages: Vec<StageData>,
}
