//! Solo sul web (`wasm32`): le opzioni di avvio arrivano dall'indirizzo della pagina
//! (`index.html?autoplay&seed=3` come `--autoplay --seed 3`) e il profilo si salva nel
//! `localStorage` del browser invece che su file.

/// Cartella di asset o dati della build: sul web con la versione nel nome (`assets-<versione>`,
/// messa da `scripts/build-web.*` in `PMS_WEB_VERSION` alla compilazione), così il browser non
/// usa dalla cache quelli di una build vecchia; sul desktop la cartella così com'è.
pub fn versioned_dir(base: &str) -> String {
    match option_env!("PMS_WEB_VERSION") {
        Some(version) if cfg!(target_arch = "wasm32") && !version.is_empty() => {
            format!("{base}-{version}")
        }
        _ => base.to_string(),
    }
}

/// Le opzioni dell'indirizzo come argomenti della riga di comando: `?autoplay&seed=3&time-scale=2`
/// → `--autoplay --seed 3 --time-scale 2`. Nomi e valori come in `options::USAGE`.
pub fn query_args(query: &str) -> Vec<String> {
    let mut args = Vec::new();
    for pair in query.trim_start_matches('?').split('&') {
        if pair.is_empty() {
            continue;
        }
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        args.push(format!("--{name}"));
        if !value.is_empty() {
            args.push(value.replace("%20", " "));
        }
    }
    args
}

/// La parte `?…` dell'indirizzo della pagina (vuota fuori dal browser).
pub fn page_query() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Il valore salvato con la chiave `key` nel `localStorage`.
#[cfg(target_arch = "wasm32")]
pub fn load(key: &str) -> Option<String> {
    storage()?.get_item(key).ok()?
}

/// La pagina è a schermo intero (anche se ci è entrata o ne è uscita con i tasti del browser).
#[cfg(target_arch = "wasm32")]
pub fn fullscreen() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.fullscreen_element())
        .is_some()
}

/// Chiede al browser lo schermo intero per il contenitore del canvas (`#stage`, così il canvas
/// lo riempie a piena risoluzione) o di uscirne. Il browser lo concede solo poco dopo un clic o
/// un tasto veri (il gioco lo chiede al fotogramma dopo, che basta); se rifiuta, la promessa
/// fallita va solo nel log: lasciata senza `catch` la pagina la mostrerebbe come errore.
#[cfg(target_arch = "wasm32")]
pub fn set_fullscreen(on: bool) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if !on {
        document.exit_fullscreen();
        return;
    }
    let Some(stage) = document.get_element_by_id("stage") else {
        return;
    };
    // `requestFullscreen` restituisce una promessa, che `web_sys` non espone
    let request = js_sys::Reflect::get(&stage, &"requestFullscreen".into())
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok());
    let Some(request) = request else {
        return;
    };
    match request.call0(&stage) {
        Ok(result) => {
            if let Ok(promise) = result.dyn_into::<js_sys::Promise>() {
                let refused = Closure::<dyn FnMut(wasm_bindgen::JsValue)>::new(|reason| {
                    bevy::log::warn!("schermo intero rifiutato dal browser: {reason:?}");
                });
                let _ = promise.catch(&refused);
                // la promessa si risolve più tardi: la closure deve restare viva (una per richiesta)
                refused.forget();
            }
        }
        Err(e) => bevy::log::warn!("schermo intero rifiutato dal browser: {e:?}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn store(key: &str, value: &str) -> Result<(), String> {
    let storage = storage().ok_or("localStorage non disponibile")?;
    storage
        .set_item(key, value)
        .map_err(|e| format!("localStorage: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_address_becomes_command_line_options() {
        assert!(query_args("").is_empty());
        assert_eq!(
            query_args("?autoplay&seed=3&time-scale=2"),
            ["--autoplay", "--seed", "3", "--time-scale", "2"]
        );
        assert_eq!(query_args("?god&&"), ["--god"]);
    }
}
