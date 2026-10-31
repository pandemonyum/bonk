//! Richieste di suono (progetto § 3.2): chi vuole un suono mette un [`CueRequest`] nella
//! [`CueQueue`]; il mixer le svuota una volta per fotogramma in `AudioSet::Decide`. Il resto del
//! gioco (interfaccia compresa) passa da [`AudioManager`], che senza l'audio (headless, test
//! della presentazione) non fa nulla.

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::log::warn;
use bevy::math::Vec2;
use bevy::prelude::{Res, ResMut, Resource};

use super::catalog::{AudioCatalog, CueId};

/// Un fatto di gioco che vuole un suono: `count` eventi dello stesso cue (es. tre proiettili
/// della stessa raffica), con la posizione nel mondo se il cue si scarta fuori vista.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CueRequest {
    pub cue: CueId,
    pub count: u32,
    pub at: Option<Vec2>,
}

/// Richieste del fotogramma, solo dati: nessun suono parte da qui.
#[derive(Resource, Debug, Default)]
pub struct CueQueue {
    requests: Vec<CueRequest>,
    /// Nomi chiesti che il catalogo non conosce (ognuno avvisa una volta sola).
    unknown: BTreeSet<String>,
    /// Richieste con un nome sconosciuto, in tutto.
    unknown_requests: u64,
}

impl CueQueue {
    pub fn push(&mut self, request: CueRequest) {
        self.requests.push(request);
    }

    /// Le richieste del fotogramma; la coda resta vuota.
    pub fn take(&mut self) -> Vec<CueRequest> {
        std::mem::take(&mut self.requests)
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Nomi chiesti che il catalogo non conosce, in ordine.
    pub fn unknown(&self) -> impl Iterator<Item = &str> {
        self.unknown.iter().map(String::as_str)
    }

    pub fn unknown_requests(&self) -> u64 {
        self.unknown_requests
    }

    /// Un nome sconosciuto: avviso nel log la prima volta, poi solo il contatore.
    fn unknown_name(&mut self, name: &str) {
        self.unknown_requests += 1;
        if !self.unknown.contains(name) {
            warn!("audio: il cue \"{name}\" non esiste in assets/audio/catalog.ron");
            self.unknown.insert(name.to_string());
        }
    }
}

/// Il modo di chiedere un suono da un sistema: `audio.play("ui.confirm")`. Senza le risorse
/// dell'audio (app senza `AudioCorePlugin`) ogni chiamata non fa nulla; un nome sconosciuto dà
/// un avviso, mai un panic.
#[derive(SystemParam)]
pub struct AudioManager<'w> {
    queue: Option<ResMut<'w, CueQueue>>,
    catalog: Option<Res<'w, AudioCatalog>>,
}

impl AudioManager<'_> {
    /// C'è l'audio (le risorse esistono).
    pub fn enabled(&self) -> bool {
        self.queue.is_some() && self.catalog.is_some()
    }

    /// Id di un cue o del suo ripiego, per chi chiede spesso lo stesso suono.
    pub fn id(&self, name: &str) -> Option<CueId> {
        self.catalog.as_ref()?.id(name)
    }

    /// Un suono senza posizione. `true` se la richiesta è in coda (il mixer può ancora
    /// scartarla).
    pub fn play(&mut self, name: &str) -> bool {
        self.request(name, 1, None)
    }

    /// Un suono in un punto del mondo (i cue con `cull: View` fuori vista si scartano).
    pub fn play_at(&mut self, name: &str, at: Vec2) -> bool {
        self.request(name, 1, Some(at))
    }

    /// `count` eventi dello stesso suono nel fotogramma (0 eventi: nulla in coda, `false`).
    pub fn request(&mut self, name: &str, count: u32, at: Option<Vec2>) -> bool {
        let (Some(queue), Some(catalog)) = (self.queue.as_mut(), self.catalog.as_ref()) else {
            return false;
        };
        if count == 0 {
            return false;
        }
        match catalog.id(name) {
            Some(cue) => {
                queue.push(CueRequest { cue, count, at });
                true
            }
            None => {
                queue.unknown_name(name);
                false
            }
        }
    }

    /// Come [`AudioManager::request`] con un id già cercato.
    pub fn request_id(&mut self, cue: CueId, count: u32, at: Option<Vec2>) -> bool {
        let Some(queue) = self.queue.as_mut() else {
            return false;
        };
        if count == 0 {
            return false;
        }
        queue.push(CueRequest { cue, count, at });
        true
    }
}
