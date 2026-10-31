# AGENTS.md — regole condivise per tutti gli agenti

Letto da Claude Code (via `CLAUDE.md`), Codex e qualunque altro agente. La specifica completa
del progetto è [HANDOFF_PIXEL_MONSTER_SURVIVOR.md](HANDOFF_PIXEL_MONSTER_SURVIVOR.md): in caso di
conflitto vale l'ultimo requisito esplicito dell'utente, poi l'handoff, poi questo file.

## Progetto in una riga

Survivor roguelite top-down 2D (orde, attacchi automatici, XP, carte upgrade, boss) con
compagni mostriciattolo **originali** in pixel art. Rust + Bevy 0.20, nativo Windows + web WASM.

## Inizio di ogni sessione

1. Leggi questo file, `docs/PROGRESS.md`, `docs/BACKLOG.md`, `docs/DECISIONS.md`.
2. Prendi il **prossimo task non completato** del backlog secondo le dipendenze.
3. Scrivi owner e criteri di done nel backlog **prima** di modificare file.
4. Lavora, esegui i test pertinenti, correggi.
5. Aggiorna `docs/PROGRESS.md` con il report di tranche (formato sotto).

## Comandi reali

| Scopo | Windows (PowerShell) | Linux/macOS |
|---|---|---|
| Gate qualità (fmt + clippy + test) | `./scripts/check.ps1` | `./scripts/check.sh` |
| Avvio gioco | `cargo run` | `cargo run` |
| Avvio (ricarica a caldo di `assets/` e `game-data/` attiva in debug) | `cargo run` (`-- --no-watch` per spegnerla) | idem |
| Rigenera placeholder pixel art | `python scripts/build-assets.py` | idem |
| Valida asset | `python scripts/validate-assets.py` | idem |
| Prova automatica con screenshot | `cargo run -- --autoplay --screenshot screenshots/s.png --screenshot-after 4 --exit-after 5` | idem |
| Minuti di gioco in pochi secondi | `cargo run -- --autoplay --god --time-scale 4 --overlay` | idem |
| Sonde di bilanciamento e prestazioni | `cargo test --test sim balance -- --ignored --nocapture --test-threads 1` | idem |

Opzioni CLI del gioco: `--seed <u64>`, `--autoplay`, `--overlay`, `--screenshot <file>`,
`--screenshot-after <s>`, `--screenshot-burst <n>`, `--exit-after <s>`, `--window <LxA>`,
`--scale-factor <f>`, `--god`, `--time-scale <f>`, `--watch`/`--no-watch`,
`--menu-screen <home|select|options>`, `--mute` (audio a zero, senza salvare), `--no-audio`
(senza `AudioPlugin` di Bevy) (`cargo run -- --help`; sul web `?mute`, `?no-audio`). All'avvio c'è la pagina
iniziale; Nuova Partita apre la selezione del compagno (frecce, 1–9 o clic, Invio avvia, Esc
torna indietro; `--autoplay` salta il menu).
Tasti: WASD/frecce movimento, Esc/P pausa, F5 ricomincia (dai risultati anche Invio, M menu),
1/2/3 o clic per la carta del draft, F3 overlay debug, F12 screenshot in
`screenshots/`. Verifica nearest-neighbor su
uno screenshot: `python scripts/check-pixel-grid.py <png>` (finestra con bande nere, senza overlay).

## Ownership dei file (nessuna sovrapposizione)

| Owner | File / directory |
|---|---|
| orchestrator | `AGENTS.md`, `CLAUDE.md`, `.claude/agents/`, `docs/agents/`, `docs/PROGRESS.md`, `docs/BACKLOG.md`, `docs/DECISIONS.md`, `Cargo.toml` `[dependencies]`, `Cargo.lock`, `src/main.rs`, `src/lib.rs` |
| game-designer | `docs/GDD.md`, valori in `game-data/*.ron` (lo schema è del gameplay-engineer) |
| gameplay-engineer | `src/app.rs`, `src/rng.rs`, `src/gameplay/`, `src/data/` (schema + loader), `src/presentation/`, `src/audio/` (schema del catalogo audio compreso) |
| world-generation | `src/world/` |
| pixel-art-director | `docs/ART_BIBLE.md`, `docs/art/`, `docs/audio/`, `assets-src/`, `assets/` (anche gli asset audio e i valori di `assets/audio/catalog.ron`, D-070), `scripts/build-assets.py`, `scripts/art_*.py` e `scripts/audio_*.py`, `scripts/validate-assets.py`, `ASSET_LICENSES.md` |
| qa-performance | `docs/QA_REPORT.md`, `src/diagnostics/`, `tests/` (un solo eseguibile: `tests/sim/`), `benches/`, `scripts/check.*`, `scripts/clean-stale.*` |
| web-release | `web/`, `scripts/build-web.*`, `.github/`, `docs/RELEASE.md`, sezioni `[profile.*]` di `Cargo.toml` |
| code-reviewer | nessun file: solo lettura, esito nel report |

Modificare un file di un altro owner richiede che l'orchestratore lo assegni esplicitamente
nel task. Due agenti non lavorano mai contemporaneamente sugli stessi file.

## Regole tecniche

- **Bevy 0.20.x bloccata.** Verifica le API su sorgenti/esempi della release 0.20.0
  (`~/.cargo/registry/src/*/bevy_*-0.20.*`, repo bevy al tag `v0.20.0`), mai su snippet vecchi.
  Esempi di API cambiate rispetto a versioni precedenti: `On<Add<T>>`, `MessageReader`,
  `FontSize::Px`, `asset_server.load_builder()`.
- Logica di gioco in `FixedUpdate` a 60 Hz su `Position`/`PreviousPosition`; il rendering
  interpola. La logica non dipende da `Sprite`/GPU: i test girano con `MinimalPlugins`.
- Gameplay e grafica separati: la presentazione aggancia gli sprite con observer `On<Add<T>>`.
- Random solo da `rng::Pcg32` con seed esplicito; mai `rand::thread_rng` nella simulazione.
- Valori di bilanciamento solo in `game-data/*.ron`, mai come costanti nel codice; un campo
  nuovo va aggiunto allo schema (`src/data/schema.rs`) e alla validazione
  (`src/data/validate.rs`) con un test. I test leggono i valori attesi da `GameConfig`.
- Mondo renderizzato a 480×270 su texture, upscaling intero nearest-neighbor, letterbox.
- Niente dipendenze nuove senza voce in `docs/DECISIONS.md` (motivo, versione, licenza).
- Arte con ChatGPT solo con l'handoff manuale di D-065: il pixel-art-director prepara la
  richiesta in `docs/art/requests/`, l'utente genera nella sua chat e salva in `.art-inbox/`
  (ignorata da git), poi controllo e promozione nella pipeline solo con la sua approvazione.
  Procedura in `docs/art/CHATGPT_ART_WORKFLOW.md`.
- Audio solo nel client (D-069): `src/audio/` osserva la simulazione e non le scrive; `app`,
  `gameplay`, `world` e `data` non lo importano; solo `src/audio/` crea voci (`AudioPlayer`).
  Progetto in `docs/design/AUDIO_SYSTEM.md`.
- Moduli piccoli: nessun file Rust oltre ~400 righe senza motivo scritto.
- Clippy con `-D warnings`; eccezioni ammesse solo in `Cargo.toml [lints]` con motivo.

## Divieti

- Non dichiarare test, build o run riusciti senza averli eseguiti: riporta l'output vero.
- Non usare asset, nomi, loghi o UI di Pokémon/Nintendo/Game Freak o di altri giochi.
- Non installare pacchetti non verificati; non usare API a pagamento o generatori IA di asset
  senza approvazione dell'utente (approvato solo l'handoff manuale con ChatGPT, D-065).
- Non cancellare o sovrascrivere file esistenti senza averli letti; niente script distruttivi.
- Non esporre segreti; niente credenziali nel repository.
- Niente push su remote, deploy pubblici, account o servizi a pagamento senza approvazione.
- Niente backend, database, account o servizi cloud nel MVP.

## Report di fine tranche (obbligatorio)

```text
Task: <ID> - <titolo>
Owner: <agente>
Stato: DONE / PARTIAL / BLOCKED
Assunzioni: ...
File modificati: ...
Comandi eseguiti (risultato): ...
Comportamento verificato: ...
Screenshot/log: ...
Rischi o bug noti: ...
Prossimo task: ...
```
