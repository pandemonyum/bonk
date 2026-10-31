//! Caricamento dei dati nel client: i file di `game-data/` arrivano dall'AssetServer come
//! testo, poi `GameConfig::parse` li valida tutti insieme. Con la ricarica a caldo attiva
//! (`--watch`, default nelle build di sviluppo) ogni salvataggio ricarica i dati e, se validi,
//! fa ripartire la run.

use std::io;

use bevy::asset::io::{AssetSourceBuilder, Reader};
use bevy::asset::{AssetLoadFailedEvent, AssetLoader, LoadContext};
use bevy::prelude::*;

use super::{DATA_FILES, DataSources, DataStatus, GameConfig};
use crate::app::{AppState, RunSetup};

/// Sorgente degli asset per i dati: `data://player.ron` → `game-data/player.ron`
/// (sul disco accanto ad `assets/`, sul web in `game-data-<versione>/` del sito, `web::versioned_dir`).
pub const DATA_SOURCE: &str = "data";
const DATA_DIR: &str = "game-data";

/// Testo grezzo di un file di dati.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct DataText(pub String);

#[derive(Default, TypePath)]
pub struct DataTextLoader;

impl AssetLoader for DataTextLoader {
    type Asset = DataText;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<DataText, io::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        String::from_utf8(bytes)
            .map(DataText)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Da chiamare prima di `DefaultPlugins`: l'AssetServer legge le sorgenti alla creazione.
pub fn register_data_source(app: &mut App) {
    register_data_source_at(app, &crate::web::versioned_dir(DATA_DIR));
}

/// Come [`register_data_source`] ma da una cartella qualsiasi (test).
pub fn register_data_source_at(app: &mut App, dir: &str) {
    app.register_asset_source(DATA_SOURCE, AssetSourceBuilder::platform_default(dir, None));
}

#[derive(Resource)]
struct DataHandles([Handle<DataText>; DATA_FILES.len()]);

pub struct DataLoaderPlugin;

impl Plugin for DataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DataText>()
            .init_asset_loader::<DataTextLoader>()
            .add_systems(Startup, start_loading)
            .add_systems(Update, update_config);
    }
}

fn start_loading(mut commands: Commands, assets: Res<AssetServer>) {
    let handles = DATA_FILES.map(|file| assets.load(format!("{DATA_SOURCE}://{file}")));
    commands.insert_resource(DataHandles(handles));
}

/// Testi dei file se sono tutti caricati, nell'ordine di `DATA_FILES`.
fn loaded_texts<'a>(
    handles: &DataHandles,
    texts: &'a Assets<DataText>,
) -> Option<[&'a str; DATA_FILES.len()]> {
    let mut out = [""; DATA_FILES.len()];
    for (slot, handle) in out.iter_mut().zip(&handles.0) {
        *slot = texts.get(handle)?.0.as_str();
    }
    Some(out)
}

/// Primo caricamento e ricariche. Si (ri)valida quando un file arriva o cambia:
/// - dati validi al primo caricamento → `GameConfig` e la run può partire;
/// - dati validi e diversi durante il gioco → nuova configurazione, run riavviata;
/// - dati non validi o file illeggibile (es. non UTF-8) → errori a schermo, anche durante la
///   run; l'eventuale configurazione precedente resta in uso e salvando il file corretto si
///   riparte da qui (anche se l'errore era al primo avvio).
///
/// Lo stage scelto (`RunSetup::stage`, D-068) diventa lo stage giocato a ogni caricamento; se
/// non esiste è un errore dei dati come gli altri.
///
/// Bevy segnala con `Added` (non `Modified`) la prima versione riuscita di un file il cui
/// caricamento era fallito: per questo contano entrambi gli eventi.
#[allow(clippy::too_many_arguments)]
fn update_config(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<DataText>>,
    mut failures: MessageReader<AssetLoadFailedEvent<DataText>>,
    handles: Res<DataHandles>,
    texts: Res<Assets<DataText>>,
    current: Option<Res<GameConfig>>,
    mut status: ResMut<DataStatus>,
    setup: Option<Res<RunSetup>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let ours = |id: AssetId<DataText>| handles.0.iter().any(|h| h.id() == id);
    let unreadable: Vec<String> = failures
        .read()
        .filter(|failure| ours(failure.id))
        .map(|failure| {
            format!(
                "{}: impossibile leggere il file ({})",
                failure.path, failure.error
            )
        })
        .collect();
    if !unreadable.is_empty() {
        set_error(&mut status, unreadable);
        return;
    }
    let changed = events.read().any(|event| match event {
        AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::LoadedWithDependencies { id } => ours(*id),
        _ => false,
    });
    let first_load = current.is_none() && *status == DataStatus::Loading;
    if !changed && !first_load {
        return;
    }
    let Some(texts) = loaded_texts(&handles, &texts) else {
        return; // ancora in caricamento
    };
    let chosen = setup.as_ref().and_then(|s| s.stage.clone());
    let parsed =
        GameConfig::parse(&DataSources::from_files(texts)).and_then(|config| match chosen {
            Some(id) => config.with_stage(&id).map_err(|error| vec![error]),
            None => Ok(config),
        });
    match parsed {
        Ok(config) => {
            *status = DataStatus::Ready;
            match current {
                None => {
                    info!("dati di gioco caricati da {DATA_DIR}/");
                    commands.insert_resource(config);
                }
                Some(current) if *current != config => {
                    info!("dati di gioco ricaricati: la run riparte con i nuovi valori");
                    commands.insert_resource(config);
                    // dal menu si torna al menu (`AfterLoading`), dalla run a una run nuova
                    if matches!(*state.get(), AppState::InRun | AppState::MainMenu) {
                        next.set(AppState::Loading);
                    }
                }
                Some(_) => {}
            }
        }
        Err(errors) => set_error(&mut status, errors),
    }
}

fn set_error(status: &mut DataStatus, errors: Vec<String>) {
    let new = DataStatus::Error(errors);
    if *status != new {
        if let DataStatus::Error(list) = &new {
            for error in list {
                error!("dati di gioco: {error}");
            }
        }
        *status = new;
    }
}

/// Caricamento vero dall'AssetServer, senza GPU, da una cartella temporanea con copie dei file.
/// Le ricariche sono forzate con `AssetServer::reload` invece che dal watcher (deterministico).
#[cfg(test)]
mod tests;
