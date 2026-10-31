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
  così il canvas ha il fuoco della tastiera e l'audio può partire (sotto);
- senza WebGL2 un messaggio chiaro invece di uno schermo nero;
- errori della pagina o del gioco nel pannello; l'eccezione con cui winit esce da `init()`
  ("Using exceptions for control flow") non è un errore e viene ignorata;
- il canvas `#game` riempie la finestra (`fit_canvas_to_parent`), i tasti vanno al gioco e non
  alla pagina (`prevent_default_event_handling`);
- Opzioni → Schermo intero: il gioco lo chiede per `#stage` (`web::set_fullscreen`), il browser
  lo concede solo dopo un clic o un tasto veri; un rifiuto resta un avviso nel log (la promessa
  ha il suo `catch`, altrimenti il pannello degli errori la mostrerebbe) e il valore mostrato è
  lo stato vero della pagina (D-063).

### Audio nella pagina (AUDIO-2, D-069)

Il gioco suona con `bevy_audio` → `rodio` → `cpal`, che sul web apre un `AudioContext` (Web
Audio) dentro `init()`. Uno script della pagina avvolge il costruttore `AudioContext` (e
`webkitAudioContext`, se c'è) in un `Proxy`: il contesto resta un `AudioContext` vero, la pagina
ne tiene l'elenco e fa seguire il suo stato alla scheda. Nessun cambiamento in Rust.

- **Lo script deve restare prima dell'`import` di `game.js`**: il codice di `wasm-bindgen` legge
  il costruttore globale una sola volta, quando il modulo viene valutato
  (`const lAudioContext = …`); avvolto dopo, il gioco userebbe quello originale senza errori.
- **Autoplay.** I browser fanno partire un `AudioContext` solo dopo un'interazione con la
  pagina. Il contesto nasce dentro `init()`, cioè dopo il clic su "Clicca per giocare": Chrome,
  Edge e Firefox lo avviano.
- **Scheda nascosta = audio sospeso.** Con la scheda nascosta Bevy non aggiorna più
  (`requestAnimationFrame` è fermo), ma cpal chiede i campioni dai callback `ended` dei suoi
  `AudioBufferSourceNode`, che non dipendono dai fotogrammi: senza la pagina la musica
  continuerebbe da sola. Su `visibilitychange` la pagina chiama `suspend()` (nascosta) e
  `resume()` (visibile). Se il contesto diventa `running` con la scheda nascosta (cpal chiama
  `resume()` appena lo crea, e il download può finire con la scheda in secondo piano), la pagina
  lo sospende di nuovo al suo `statechange`. Una finestra senza fuoco ma ancora visibile non
  conta come nascosta: lì decide il gioco.
- **Safari** non è un obiettivo della P1. Se un contesto resta sospeso, lo riprende il primo
  `pointerdown`, `keydown` o `touchend` con la pagina visibile (ascoltati in cattura su
  `window`, prima del gioco).
- **Promesse.** `resume()` e `suspend()` hanno il loro `catch`: un rifiuto è un avviso
  `audio: … rifiutato` nella console, non un errore nel pannello (come lo schermo intero).
- **`?mute`**: il contesto si apre e gira, il gioco tiene il volume a zero per la sessione senza
  salvarlo nel profilo. **`?no-audio`**: niente `AudioPlugin`, nessun `AudioContext` nasce. Si
  combinano con le altre opzioni (`?autoplay&mute`).
- **Per le prove:** righe `console.info` con il prefisso `audio:` e `window.__pmsAudio.contexts`
  (sola lettura), che a ogni lettura restituisce `[{ state, sampleRate, currentTime }]`.

**Prova nel browser** (`python -m http.server 8080 -d dist`, poi `http://localhost:8080/`):

1. Prima del clic: `__pmsAudio.contexts` è `[]`, nessuna riga `audio:`.
2. Clic su "Clicca per giocare". Finito il download, in console (filtro `audio:`):

   ```text
   audio: contesto 0 creato (running, 44100 Hz)
   audio: contesto 0: running
   ```

   In Firefox la prima riga può dire `suspended`: conta che arrivi `running`. Il pannello degli
   errori resta nascosto. `__pmsAudio.contexts` ha un elemento con `state: "running"` e
   `currentTime` che cresce tra due letture.
3. Scheda nascosta (un'altra scheda in primo piano):

   ```text
   audio: contesto 0 suspend() (visibilitychange)
   audio: contesto 0: suspended
   ```

   `currentTime` resta fermo tra due letture a un secondo di distanza.
4. Di nuovo sulla scheda del gioco:

   ```text
   audio: contesto 0 resume() (visibilitychange)
   audio: contesto 0: running
   ```

5. Se il browser automatico non cambia la visibilità cambiando scheda (headless), simularla:

   ```js
   Object.defineProperty(document, "visibilityState", { value: "hidden", configurable: true });
   document.dispatchEvent(new Event("visibilitychange"));   // righe del passo 3
   delete document.visibilityState;                         // torna lo stato vero
   document.dispatchEvent(new Event("visibilitychange"));   // righe del passo 4
   ```

6. `http://localhost:8080/?no-audio`: il gioco parte, nessuna riga `audio:`, `contexts` è `[]`.
   `?mute`: come il passo 2, senza suono.
7. Facoltativo, `instanceof` su un contesto creato dal `Proxy`. Prima del clic, in console:

   ```js
   (() => { const c = new AudioContext(); c.close();
     return [c instanceof AudioContext, Object.getPrototypeOf(c) === AudioContext.prototype]; })()
   ```

   dà `[true, true]`, con le righe `audio: contesto 0 creato (suspended, …)` e
   `audio: contesto 0: closed` (e l'avviso di Chrome sull'autoplay: nessun clic ancora). Poi
   ricaricare la pagina, altrimenti quel contesto resta nell'elenco come numero 0.

## Salvataggi sul web

Il profilo (esperienza e talenti dei compagni, D-058/D-059) sta nel `localStorage` del browser,
chiave `PixelMonsterSurvivor/profile.ron`, con lo stesso testo RON del file desktop. È legato al
browser e al sito (porta compresa: `localhost:8080` e `localhost:8081` sono profili diversi).

## Pubblicazione

Nessuna finora: un hosting statico qualsiasi basta (itch.io, GitHub Pages, Cloudflare Pages,
Netlify), con `.wasm` servito come `application/wasm` e possibilmente compresso (gzip o brotli).
Pubblicare o creare account richiede il via dell'utente (AGENTS.md).

## CI (P1.6, D-064)

Repository: https://github.com/pandemonyum/bonk (pubblico, ramo `main`). Il workflow
`.github/workflows/ci.yml` parte a ogni push su `main`, a ogni pull request e a mano (Actions →
CI → Run workflow), con due job su Ubuntu 24.04:

- **gate**: `scripts/check.sh` (fmt, clippy senza warning, test, validazione asset), con le
  librerie di sistema che Bevy vuole su Linux (`libudev-dev` per gilrs, `libwayland-dev`,
  `libxkbcommon-dev`, `libasound2-dev`);
- **web**: `scripts/build-web.sh` con wasm-bindgen alla versione del `Cargo.lock`
  (`taiki-e/install-action`); `dist/` resta 14 giorni come artefatto
  `companion-survivors-web`, da scaricare dalla pagina della run. Non pubblica niente.

Toolchain dalla `rust-toolchain.toml`, cache di cargo con `Swatinem/rust-cache`. Un push nuovo
sullo stesso ramo ferma la run precedente. Windows non è in CI: il gate lì resta
`scripts/check.ps1` in locale.

## Misure (2026-10-29)

| | Dimensione |
|---|---|
| `.wasm` uscito da cargo (`wasm-release`) | 25,2 MiB |
| `game_bg.wasm` in `dist` (senza sezioni dei nomi) | 22,5 MiB |
| lo stesso compresso gzip (quello che scarica il browser da un server che comprime) | 5,9 MiB |
| `dist/` in tutto | 23,3 MiB |

La prima build, con i nomi delle funzioni e `opt-level = "s"`, pesava 54 MiB.
