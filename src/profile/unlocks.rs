//! Sblocchi dei compagni iniziali (D-066), in un solo posto: la regola `unlock` dei dati
//! (`companions.ron`) e i compagni che il profilo ha ottenuto (`ProfileData::unlocked`). La UI
//! legge solo `UnlockStatus`; l'avvio della run (`app::try_start_run`) rifiuta chi non è
//! disponibile. Oggi nessuna condizione sblocca nulla e nessuna run aggiunge compagni qui: chi si
//! unisce durante la run (D-055) non diventa compagno iniziale.

use super::Profile;
use crate::data::GameConfig;
use crate::data::schema::{CompanionData, Unlock};

/// Si può scegliere come compagno iniziale?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockStatus {
    Available,
    Locked,
}

impl UnlockStatus {
    pub fn is_available(self) -> bool {
        self == Self::Available
    }
}

/// Stato del compagno `id` per questo profilo; `None` se l'id non c'è nei dati.
pub fn unlock_status(config: &GameConfig, profile: &Profile, id: &str) -> Option<UnlockStatus> {
    let companion = config.companion(id)?;
    let open = match companion.unlock {
        Unlock::Start => true,
        Unlock::Locked => profile.data.unlocked.iter().any(|u| u == id),
    };
    Some(if open {
        UnlockStatus::Available
    } else {
        UnlockStatus::Locked
    })
}

/// Il compagno `id` esiste nei dati e si può scegliere.
pub fn is_unlocked(config: &GameConfig, profile: &Profile, id: &str) -> bool {
    unlock_status(config, profile, id).is_some_and(UnlockStatus::is_available)
}

/// Il primo compagno disponibile nell'ordine dei dati (la validazione garantisce che ci sia un
/// compagno `Start`).
pub fn first_unlocked<'a>(config: &'a GameConfig, profile: &Profile) -> Option<&'a CompanionData> {
    config
        .companions
        .iter()
        .find(|c| is_unlocked(config, profile, &c.id))
}

/// Il compagno proposto entrando nel menu e nella selezione: l'ultimo usato (`last`) se è ancora
/// disponibile, altrimenti il primo disponibile, in ultimo quello dello stage.
pub fn proposed_starter(config: &GameConfig, profile: &Profile, last: Option<&str>) -> String {
    last.filter(|id| is_unlocked(config, profile, id))
        .map(str::to_string)
        .or_else(|| first_unlocked(config, profile).map(|c| c.id.clone()))
        .unwrap_or_else(|| config.stage.starting_companion.clone())
}

impl Profile {
    /// Sblocca `id` come compagno iniziale (per le condizioni future); `unlocked` resta ordinato
    /// e senza doppioni. Non salva: lo fa chi chiama, come per i talenti. `true` se è nuovo.
    pub fn grant(&mut self, id: &str) -> bool {
        match self.data.unlocked.binary_search_by(|u| u.as_str().cmp(id)) {
            Ok(_) => false,
            Err(at) => {
                self.data.unlocked.insert(at, id.to_string());
                true
            }
        }
    }
}
