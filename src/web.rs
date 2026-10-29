//! Solo sul web (`wasm32`): le opzioni di avvio arrivano dall'indirizzo della pagina
//! (`index.html?autoplay&seed=3` come `--autoplay --seed 3`) e il profilo si salva nel
//! `localStorage` del browser invece che su file.

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
