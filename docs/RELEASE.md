# Build web e rilascio (P1)

Il gioco gira nel browser come WebAssembly (`wasm32-unknown-unknown`), disegnato con WebGL2
(D-018). È tutto statico: il browser scarica pagina, `.wasm`, asset e dati una volta e il gioco
gira sul dispositivo di chi gioca; il server non fa altro (nessun backend, D-018/AGENTS.md).

## Strumenti

| Strumento | Versione | Come |
|---|---|---|
| Rust | 1.99.1 (`rust-toolchain.toml`) con il target `wasm32-unknown-unknown` | `rustup target add wasm32-unknown-unknown` |
| wasm-bindgen-cli | **identica a `wasm-bindgen` nel `Cargo.lock`** (oggi 0.2.129): gli script si fermano se non coincide | `cargo install wasm-bindgen-cli --version 0.2.129 --locked` |
| wasm-opt (binaryen) | facoltativo: se c'è, `-Os` sul `.wasm` | pacchetto `binaryen` del sistema |
| Python 3 | per la prova locale (`.wasm` servito come `application/wasm`) | — |

Niente Trunk né bundler JavaScript: `wasm-bindgen --target web` produce un modulo ES che la
pagina carica da sola.

## Build

```powershell
./scripts/build-web.ps1
```

```bash
./scripts/build-web.sh
```

1. `cargo build --profile wasm-release --target wasm32-unknown-unknown` (profilo in
   `Cargo.toml`: `opt-level = "s"`, LTO completo, senza simboli di debug);
2. `wasm-bindgen --target web` in `dist/` (`game.js` e `game_bg.wasm`);
3. `wasm-opt -Os` se installato;
4. `web/index.html` in `dist/index.html` con `__VERSION__` = le prime 12 cifre dello SHA-256 del
   `.wasm` (cache busting di `game.js` e `game_bg.wasm`: una build nuova non viene presa dalla
   cache del browser);
5. copia di `assets/` e `game-data/` in `assets-<v>/` e `game-data-<v>/`, con `<v>` le prime
   10 cifre dello SHA-256 del loro contenuto: lo script la passa al gioco alla compilazione
   (`PMS_WEB_VERSION`), così il browser non usa dalla cache asset o dati di una build vecchia
   (D-062);
6. dimensioni (`.wasm` grezzo, in `dist`, gzip) e controllo che nessun file superi 25 MiB.

`dist/` è generata e ignorata da git; lo script la ricrea da zero.

## Prova locale

```bash
python -m http.server 8080 -d dist
```

Poi `http://localhost:8080` (mai `file://`: il browser non carica moduli e `.wasm` da file).
Le opzioni della riga di comando passano dall'indirizzo: `?autoplay&seed=3&time-scale=2&god`
(vedi `src/web.rs`). In Claude Code: configurazione `web` di `.claude/launch.json`.

## La pagina (`web/index.html`)

- "Clicca per giocare": il `.wasm` si scarica e parte solo dopo il clic (avanzamento in MB),
  così il canvas ha il fuoco della tastiera (e l'audio, quando ci sarà, può partire);
- senza WebGL2 un messaggio chiaro invece di uno schermo nero;
- errori della pagina o del gioco nel pannello; l'eccezione con cui winit esce da `init()`
  ("Using exceptions for control flow") non è un errore e viene ignorata;
- il canvas `#game` riempie la finestra (`fit_canvas_to_parent`), i tasti vanno al gioco e non
  alla pagina (`prevent_default_event_handling`);
- Opzioni → Schermo intero: il gioco lo chiede per `#stage` (`web::set_fullscreen`), il browser
  lo concede solo dopo un clic o un tasto veri; un rifiuto resta un avviso nel log (la promessa
  ha il suo `catch`, altrimenti il pannello degli errori la mostrerebbe) e il valore mostrato è
  lo stato vero della pagina (D-063).

## Salvataggi sul web

Il profilo (esperienza e talenti dei compagni, D-058/D-059) sta nel `localStorage` del browser,
chiave `PixelMonsterSurvivor/profile.ron`, con lo stesso testo RON del file desktop. È legato al
browser e al sito (porta compresa: `localhost:8080` e `localhost:8081` sono profili diversi).

## Pubblicazione

Nessuna finora: un hosting statico qualsiasi basta (itch.io, GitHub Pages, Cloudflare Pages,
Netlify), con `.wasm` servito come `application/wasm` e possibilmente compresso (gzip o brotli).
Pubblicare, creare account o collegare un repository remoto richiede il via dell'utente
(AGENTS.md); la CI (P1.6) aspetta un remote GitHub.

## Misure (2026-10-29)

| | Dimensione |
|---|---|
| `.wasm` uscito da cargo (`wasm-release`) | 25,2 MiB |
| `game_bg.wasm` in `dist` (senza sezioni dei nomi) | 22,5 MiB |
| lo stesso compresso gzip (quello che scarica il browser da un server che comprime) | 5,9 MiB |
| `dist/` in tutto | 23,3 MiB |

La prima build, con i nomi delle funzioni e `opt-level = "s"`, pesava 54 MiB.
