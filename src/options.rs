//! Opzioni da riga di comando. Sul web non ci sono argomenti: valgono i default.

use std::path::PathBuf;
use std::str::FromStr;

use bevy::prelude::Resource;

pub const DEFAULT_SEED: u64 = 0x5EED_2026;

pub const USAGE: &str = "\
Uso: pixel-monster-survivor [opzioni]
  --seed <u64>             seed della run (default 1592598566)
  --autoplay               il personaggio si muove da solo (prove e screenshot)
  --overlay                mostra subito l'overlay di debug (F3)
  --screenshot <file.png>  salva uno screenshot della finestra
  --screenshot-after <s>   secondi prima dello screenshot (default 3)
  --screenshot-burst <n>   n screenshot a 0,1 s l'uno dall'altro (file-0.png, file-1.png, ...)
  --exit-after <s>         chiude il gioco dopo s secondi (attende gli screenshot)
  --window <LxA>           dimensione iniziale della finestra (default 960x540)
  --scale-factor <f>       forza il fattore DPI (es. 1.25 per simulare uno schermo al 125%)
  --god                    il player non subisce danni (prove e screenshot di run lunghe)
  --time-scale <f>         velocità del tempo di gioco (es. 4 = 4 minuti di gioco al minuto)
  --watch / --no-watch     ricarica a caldo di assets/ e game-data/ (default: attiva nella
                           build di sviluppo, spenta in release; non disponibile sul web)
  -h, --help               mostra questo aiuto";

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct LaunchOptions {
    pub seed: u64,
    pub autoplay: bool,
    pub show_overlay: bool,
    pub screenshot: Option<PathBuf>,
    pub screenshot_after_secs: f32,
    /// Numero di screenshot della raffica (1 = uno solo, col percorso così com'è).
    pub screenshot_burst: u32,
    pub exit_after_secs: Option<f32>,
    /// Pixel logici; 960×540 = 2× la risoluzione virtuale 480×270.
    pub window_size: (u32, u32),
    /// `None` = DPI del sistema.
    pub scale_factor: Option<f32>,
    /// Il player non subisce danni.
    pub god: bool,
    /// Moltiplicatore del tempo di gioco (1 = tempo reale).
    pub time_scale: f32,
    /// Ricarica a caldo; `None` = default della build (vedi [`LaunchOptions::watch_files`]).
    pub watch: Option<bool>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            autoplay: false,
            show_overlay: false,
            screenshot: None,
            screenshot_after_secs: 3.0,
            screenshot_burst: 1,
            exit_after_secs: None,
            window_size: (960, 540),
            scale_factor: None,
            god: false,
            time_scale: 1.0,
            watch: None,
        }
    }
}

impl LaunchOptions {
    /// Ricarica a caldo effettiva: esplicita da riga di comando, altrimenti attiva solo nelle
    /// build di sviluppo. Sul web non c'è watcher e il valore viene ignorato.
    pub fn watch_files(&self) -> bool {
        self.watch.unwrap_or(cfg!(debug_assertions))
    }

    /// Legge gli argomenti del processo; in caso di errore stampa l'uso ed esce.
    pub fn from_env() -> Self {
        match Self::parse(std::env::args().skip(1)) {
            Ok(Some(options)) => options,
            Ok(None) => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            Err(message) => {
                eprintln!("{message}\n\n{USAGE}");
                std::process::exit(2);
            }
        }
    }

    /// `Ok(None)` se è stato chiesto l'aiuto.
    pub fn parse<I, S>(args: I) -> Result<Option<Self>, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut options = Self::default();
        let mut args = args.into_iter().map(Into::into);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--seed" => options.seed = value(&mut args, &flag)?,
                "--autoplay" => options.autoplay = true,
                "--overlay" => options.show_overlay = true,
                "--screenshot" => options.screenshot = Some(value(&mut args, &flag)?),
                "--screenshot-after" => options.screenshot_after_secs = value(&mut args, &flag)?,
                "--screenshot-burst" => match value::<u32>(&mut args, &flag)? {
                    n @ 1..=50 => options.screenshot_burst = n,
                    n => return Err(format!("--screenshot-burst: da 1 a 50, trovato {n}")),
                },
                "--exit-after" => options.exit_after_secs = Some(value(&mut args, &flag)?),
                "--window" => {
                    options.window_size = window_size(&value::<String>(&mut args, &flag)?)?
                }
                "--scale-factor" => match value::<f32>(&mut args, &flag)? {
                    f if f > 0.0 && f.is_finite() => options.scale_factor = Some(f),
                    f => return Err(format!("--scale-factor deve essere > 0, trovato {f}")),
                },
                "--god" => options.god = true,
                "--time-scale" => match value::<f32>(&mut args, &flag)? {
                    f if f > 0.0 && f <= 20.0 => options.time_scale = f,
                    f => return Err(format!("--time-scale: da 0 (escluso) a 20, trovato {f}")),
                },
                "--watch" => options.watch = Some(true),
                "--no-watch" => options.watch = Some(false),
                "-h" | "--help" => return Ok(None),
                other => return Err(format!("opzione sconosciuta: {other}")),
            }
        }
        Ok(Some(options))
    }
}

fn value<T: FromStr>(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<T, String> {
    let raw = args
        .next()
        .ok_or_else(|| format!("{flag} richiede un valore"))?;
    raw.parse()
        .map_err(|_| format!("{flag}: valore non valido '{raw}'"))
}

fn window_size(raw: &str) -> Result<(u32, u32), String> {
    let invalid = || format!("--window: atteso LxA (es. 1366x768), trovato '{raw}'");
    let (w, h) = raw.split_once(['x', 'X']).ok_or_else(invalid)?;
    match (w.parse(), h.parse()) {
        (Ok(w), Ok(h)) if w > 0 && h > 0 => Ok((w, h)),
        _ => Err(invalid()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<LaunchOptions>, String> {
        LaunchOptions::parse(args.iter().copied())
    }

    #[test]
    fn no_args_gives_defaults() {
        assert_eq!(parse(&[]).unwrap(), Some(LaunchOptions::default()));
    }

    #[test]
    fn parses_all_flags() {
        let o = parse(&[
            "--seed",
            "42",
            "--autoplay",
            "--overlay",
            "--screenshot",
            "out/s.png",
            "--screenshot-after",
            "1.5",
            "--exit-after",
            "4",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(o.seed, 42);
        assert!(o.autoplay && o.show_overlay);
        assert_eq!(o.screenshot, Some(PathBuf::from("out/s.png")));
        assert_eq!(o.screenshot_after_secs, 1.5);
        assert_eq!(o.exit_after_secs, Some(4.0));
    }

    #[test]
    fn parses_window_size() {
        let o = parse(&["--window", "1366x768"]).unwrap().unwrap();
        assert_eq!(o.window_size, (1366, 768));
        assert!(parse(&["--window", "1366"]).is_err());
        assert!(parse(&["--window", "0x768"]).is_err());
        assert!(parse(&["--window", "axb"]).is_err());
    }

    #[test]
    fn parses_scale_factor() {
        let o = parse(&["--scale-factor", "1.25"]).unwrap().unwrap();
        assert_eq!(o.scale_factor, Some(1.25));
        assert!(parse(&["--scale-factor", "0"]).is_err());
        assert!(parse(&["--scale-factor", "-1"]).is_err());
    }

    #[test]
    fn watch_defaults_to_the_build_kind_and_can_be_forced() {
        let default = parse(&[]).unwrap().unwrap();
        assert_eq!(default.watch, None);
        assert_eq!(default.watch_files(), cfg!(debug_assertions));
        assert!(parse(&["--watch"]).unwrap().unwrap().watch_files());
        assert!(!parse(&["--no-watch"]).unwrap().unwrap().watch_files());
    }

    #[test]
    fn parses_god_mode_and_time_scale() {
        let o = parse(&["--god", "--time-scale", "4"]).unwrap().unwrap();
        assert!(o.god);
        assert_eq!(o.time_scale, 4.0);
        assert_eq!(parse(&[]).unwrap().unwrap().time_scale, 1.0);
        assert!(parse(&["--time-scale", "0"]).is_err());
        assert!(parse(&["--time-scale", "50"]).is_err());
    }

    #[test]
    fn parses_screenshot_burst_within_limits() {
        let o = parse(&["--screenshot-burst", "8"]).unwrap().unwrap();
        assert_eq!(o.screenshot_burst, 8);
        assert!(parse(&["--screenshot-burst", "0"]).is_err());
        assert!(parse(&["--screenshot-burst", "51"]).is_err());
    }

    #[test]
    fn help_returns_none() {
        assert_eq!(parse(&["--help"]).unwrap(), None);
    }

    #[test]
    fn rejects_unknown_flag_missing_and_invalid_values() {
        assert!(parse(&["--fly"]).is_err());
        assert!(parse(&["--seed"]).is_err());
        assert!(parse(&["--seed", "abc"]).is_err());
    }

    #[test]
    fn usage_mentions_default_seed() {
        assert!(USAGE.contains(&DEFAULT_SEED.to_string()));
    }
}
