//! Schema di `attacks.ron`: campi comuni e parametri di ogni pattern (proiettile, onda,
//! orbita).

use serde::Deserialize;

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
