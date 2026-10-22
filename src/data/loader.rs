//! Caricamento dei dati nel client: i file di `game-data/` arrivano dall'AssetServer come
//! testo, poi `GameConfig::parse` li valida tutti insieme. Con la ricarica a caldo attiva
//! (`--watch`, default nelle build di sviluppo) ogni salvataggio ricarica i dati e, se validi,
//! fa ripartire la run.

use std::io;

use bevy::asset::io::{AssetSourceBuilder, Reader};
use bevy::asset::{AssetLoader, LoadContext, LoadState};
use bevy::prelude::*;

use super::{DATA_FILES, DataSources, DataStatus, GameConfig};
use crate::app::AppState;

/// Sorgente degli asset per i dati: `data://player.ron` → `game-data/player.ron`
/// (sul disco accanto ad `assets/`, sul web nella cartella `game-data/` del sito).
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
    app.register_asset_source(
        DATA_SOURCE,
        AssetSourceBuilder::platform_default(DATA_DIR, None),
    );
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
fn loaded_texts<'a>(handles: &DataHandles, texts: &'a Assets<DataText>) -> Option<[&'a str; 5]> {
    let mut out = [""; 5];
    for (slot, handle) in out.iter_mut().zip(&handles.0) {
        *slot = texts.get(handle)?.0.as_str();
    }
    Some(out)
}

fn sources(texts: [&str; 5]) -> DataSources<'_> {
    let [player, companions, attacks, enemies, stages] = texts;
    DataSources {
        player,
        companions,
        attacks,
        enemies,
        stages,
    }
}

/// Primo caricamento e ricariche. Si (ri)valida quando i file sono appena arrivati o quando
/// uno di essi cambia su disco:
/// - dati validi al primo caricamento → `GameConfig` e la run può partire;
/// - dati validi e diversi durante il gioco → nuova configurazione, run riavviata;
/// - dati non validi → errori a schermo; l'eventuale configurazione precedente resta in uso,
///   e salvando il file corretto si riparte da qui (anche se l'errore era al primo avvio).
fn update_config(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<DataText>>,
    assets: Res<AssetServer>,
    handles: Res<DataHandles>,
    texts: Res<Assets<DataText>>,
    current: Option<Res<GameConfig>>,
    mut status: ResMut<DataStatus>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let changed = events.read().any(|event| match event {
        AssetEvent::Modified { id } => handles.0.iter().any(|h| h.id() == *id),
        _ => false,
    });
    let first_load = current.is_none() && *status == DataStatus::Loading;
    if !changed && !first_load {
        return;
    }
    if first_load {
        let failures: Vec<String> = DATA_FILES
            .iter()
            .zip(&handles.0)
            .filter_map(|(file, handle)| match assets.load_state(handle) {
                LoadState::Failed(err) => {
                    Some(format!("{file}: impossibile leggere il file ({err})"))
                }
                _ => None,
            })
            .collect();
        if !failures.is_empty() {
            set_error(&mut status, failures);
            return;
        }
    }
    let Some(texts) = loaded_texts(&handles, &texts) else {
        return; // ancora in caricamento
    };
    match GameConfig::parse(&sources(texts)) {
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
                    if *state.get() == AppState::InRun {
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
