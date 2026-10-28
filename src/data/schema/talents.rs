//! Talenti dei compagni (`companions.ron: talents`) ed esperienza che li sblocca
//! (`progression.ron: companion_xp`). Ogni compagno ha i suoi rami (colonne della schermata);
//! un nodo ha uno o più gradi, ognuno costa un punto, e si apre quando i nodi che richiede hanno
//! almeno un grado. Si guadagna un punto per livello del compagno; i livelli si salvano sul
//! dispositivo tra una run e l'altra.

use serde::Deserialize;

/// Colore e senso di un ramo: l'attacco del compagno, l'aiuto al player, il compagno stesso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub enum BranchTheme {
    #[default]
    Element,
    Support,
    Evolution,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TalentBranch {
    pub name: String,
    #[serde(default)]
    pub theme: BranchTheme,
    pub nodes: Vec<TalentNode>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TalentNode {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Posto nella colonna del ramo: colonna 0 o 1 per i due percorsi, 2 al centro; riga
    /// dall'alto.
    #[serde(default)]
    pub col: u32,
    #[serde(default)]
    pub row: u32,
    /// Nodi (dello stesso ramo) che devono avere almeno un grado.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Effetti di ogni grado, in ordine.
    pub ranks: Vec<Vec<TalentEffect>>,
}

impl TalentNode {
    pub fn max_rank(&self) -> u32 {
        self.ranks.len() as u32
    }
}

/// Cosa fa un grado di un nodo. Le statistiche si sommano (come le carte, e grado dopo grado);
/// per le meccaniche nuove vale l'ultimo grado preso che le nomina, nell'ordine dei dati (il
/// grado 2 "allarga" l'esplosione del grado 1). I danni in % partono dal danno dell'attacco già
/// modificato da carte e talenti; raggi, portate, ritmi e ritardi sono fissi. Gli effetti
/// `Player*` aiutano il player finché il compagno è in squadra.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum TalentEffect {
    DamagePct(f32),
    /// Negativo = attacchi più frequenti.
    IntervalPct(f32),
    /// Gittata, raggio dell'onda o dell'orbita.
    RangePct(f32),
    /// Proiettili, foglie o salti in più.
    ExtraCount(u32),
    /// Frazione di velocità che resta a chi è rallentato (sostituisce quella dell'attacco).
    SlowFactor(f32),
    /// Secondi di rallentamento (sostituisce quelli dell'attacco).
    SlowSecs(f32),
    /// Danno di un salto rispetto al precedente (sostituisce quello del fulmine).
    ChainFalloff(f32),
    /// Velocità angolare delle foglie in più.
    AngularSpeedPct(f32),
    /// Il proiettile esplode sul colpito e ferisce gli altri nemici entro `radius`.
    Explode {
        radius: f32,
        damage_pct: f32,
    },
    /// Ogni `every` attacchi anche un cono verso il bersaglio.
    Cone {
        every: u32,
        half_angle_deg: f32,
        range: f32,
        damage_pct: f32,
    },
    /// Ogni nemico colpito cura il player, fino a `cap` HP per attacco.
    HealPerHit {
        hp: f32,
        cap: f32,
    },
    /// L'attacco riparte dopo `delay_s` con il danno in %.
    Echo {
        delay_s: f32,
        damage_pct: f32,
    },
    /// Ogni `every_s` un colpo ad area sul nemico più vicino entro `range`.
    Strike {
        every_s: f32,
        range: f32,
        radius: f32,
        damage_pct: f32,
    },
    /// Chi è colpito brucia: `dps_pct` del colpo al secondo per `duration_s` (si rinnova, non
    /// si somma).
    Burn {
        dps_pct: f32,
        duration_s: f32,
    },
    /// HP massimi del player in più (e cura della stessa quantità).
    PlayerMaxHp(f32),
    /// HP al secondo per il player.
    PlayerRegen(f32),
    PlayerSpeedPct(f32),
    PlayerPickupPct(f32),
    PlayerCoinPct(f32),
    /// Danni subiti dal player in % (negativo = meno danni), per ogni colpo.
    PlayerDamageTakenPct(f32),
    /// Secondi di invulnerabilità in più dopo un colpo.
    PlayerInvulnSecs(f32),
}

/// Esperienza dei compagni a fine run e livelli.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionXp {
    /// Un punto ogni tanti secondi in squadra.
    pub seconds_per_point: f32,
    /// Un punto ogni tanti nemici finiti.
    pub kills_per_point: u32,
    pub victory: u32,
    /// Per salire dal livello n al n + 1: `first_level + per_level × (n − 1)`.
    pub first_level: u32,
    pub per_level: u32,
    pub max_level: u32,
}

impl CompanionXp {
    /// Esperienza della run: tempo in squadra, nemici finiti, vittoria.
    pub fn earned(&self, seconds: f32, kills: u32, victory: bool) -> XpGain {
        XpGain {
            time: (seconds.max(0.0) / self.seconds_per_point.max(f32::EPSILON)).floor() as u32,
            kills: kills / self.kills_per_point.max(1),
            victory: if victory { self.victory } else { 0 },
        }
    }

    /// Livello con `xp` di esperienza totale (da 1 a `max_level`).
    pub fn level(&self, xp: u32) -> u32 {
        let mut level = 1;
        let mut needed = 0u32;
        while level < self.max_level {
            needed = needed.saturating_add(self.to_next(level));
            if xp < needed {
                break;
            }
            level += 1;
        }
        level
    }

    /// Esperienza per salire dal livello `level` al successivo.
    pub fn to_next(&self, level: u32) -> u32 {
        self.first_level
            .saturating_add(self.per_level.saturating_mul(level.saturating_sub(1)))
    }

    /// Esperienza totale per arrivare al livello `level`.
    pub fn total_for(&self, level: u32) -> u32 {
        (1..level).map(|l| self.to_next(l)).sum()
    }
}

/// L'esperienza di una run, divisa per voce (per il riepilogo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XpGain {
    pub time: u32,
    pub kills: u32,
    pub victory: u32,
}

impl XpGain {
    pub fn total(&self) -> u32 {
        self.time + self.kills + self.victory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XP: CompanionXp = CompanionXp {
        seconds_per_point: 15.0,
        kills_per_point: 40,
        victory: 20,
        first_level: 100,
        per_level: 50,
        max_level: 7,
    };

    #[test]
    fn a_run_earns_time_kills_and_victory() {
        let gain = XP.earned(16.0 * 60.0, 1000, true);
        assert_eq!(
            gain,
            XpGain {
                time: 64,
                kills: 25,
                victory: 20
            }
        );
        assert_eq!(gain.total(), 109);
        assert_eq!(XP.earned(9.0 * 60.0, 450, false).total(), 47);
        assert_eq!(XP.earned(-3.0, 0, false).total(), 0);
    }

    #[test]
    fn levels_follow_the_curve_and_stop_at_the_max() {
        assert_eq!(XP.level(0), 1);
        assert_eq!(XP.level(99), 1);
        assert_eq!(XP.level(100), 2);
        assert_eq!(XP.level(249), 2);
        assert_eq!(XP.level(250), 3);
        assert_eq!(XP.total_for(7), 1350);
        assert_eq!(XP.level(1350), 7);
        assert_eq!(XP.level(99_999), 7);
    }
}
