# BACKLOG

Regola: prima di iniziare un task, scrivere owner e criteri di done qui. Un task è DONE solo
con prove (comandi eseguiti, screenshot o log) nel report in `docs/PROGRESS.md`.

## S0 — bootstrap (DONE 2026-10-22)

- [x] S0.1 Ispezione ambiente — orchestrator
- [x] S0.2 AGENTS.md, CLAUDE.md, ruoli, docs, .gitignore, README — orchestrator
- [x] S0.3 Toolchain 1.99.1 bloccata, Cargo.lock, Bevy 0.20.0 — orchestrator
- [x] S0.4 App Bevy 2D: arena, evocatore, 8 direzioni, camera, 10 nemici, nearest — gameplay-engineer
- [x] S0.5 fmt, clippy, test, run con screenshot — qa-performance

## P0 — core giocabile desktop (priorità assoluta)

Ordine di esecuzione deciso dall'utente il 2026-10-22: **P0.2 → P0.1 → P0.0**, poi P0.3 e
seguenti (handoff §10: il companion che attacca è il primo elemento ludico differenziante).

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~P0.2~~ **DONE** | Primo companion (fuoco) che segue il player e attacca in automatico il nemico più vicino nel raggio con un proiettile mirato; HP dei nemici, danno, morte una sola volta, hit feedback; popolazione mantenuta a 10 nemici finché non arriva lo spawn director | gameplay-engineer + pixel-art-director (sprite) | test: targeting nemico più vicino, cooldown, danno, una sola morte per nemico, nessun input richiesto, popolazione mai oltre 10; screenshot con proiettili |
| ~~P0.1~~ **DONE** | Ostacoli procedurali (rocce/alberi) con collisione semplice per player e nemici; zona centrale libera | world-generation + gameplay-engineer | test: centro libero, flood fill raggiunge i bordi, player non attraversa ostacoli; screenshot |
| P0.0 | Loader `game-data/*.ron` via AssetServer (funziona anche su WASM) + validazione all'avvio con messaggio d'errore; spostare in RON i valori oggi costanti (velocità, raggi, n. nemici, attacco) | gameplay-engineer | test di parsing e di validazione (dato errato → errore leggibile); `cargo run --features dev` ricarica i valori |
| P0.3 | Spawn director: 3 archetipi nemici da dati, curva per minuto, cap per archetipo, spawn fuori camera e mai sul player; HP, danno al player, morte, hit feedback, despawn; griglia spaziale al posto della separazione O(n²) | gameplay-engineer + game-designer (valori) | test: nessuno spawn dentro la camera/il player, cap rispettato, restart non accumula entità; overlay con conteggi |
| P0.4 | Gemme XP (drop una sola volta), raccolta per prossimità, livelli, draft di 3 carte con pausa, effetti applicati a caldo | gameplay-engineer + game-designer | test: +1 proiettile cambia i proiettili visibili, +danno cambia il danno effettivo; screenshot del draft |
| P0.5 | Secondo e terzo companion (onda in area, foglie orbitanti), slot da dati | gameplay-engineer | test per ciascun pattern di attacco; screenshot |
| P0.6 | Timer 10 min, scala difficoltà, élite, boss con telegraph, stati MainMenu/GameOver/Victory, restart | gameplay-engineer | test headless: run completa a seed fisso con autoplay arriva a fine senza panic; restart pulisce lo stato |
| P0.7 | HUD (HP, compagni, tempo, risorse, abilità, XP) leggibile; test formule e bilanciamento base | gameplay-engineer + qa-performance | screenshot HUD; test formule XP/danno/cooldown |

Gate P0: 10 minuti giocabili end-to-end senza crash, con prova umana o autoplay + screenshot.

## P1 — web giocabile

| ID | Task | Owner |
|---|---|---|
| P1.1 | Build `wasm32-unknown-unknown` (profilo `wasm-release`), scelta bundler dopo verifica reale (`wasm-bindgen-cli` alla versione del lockfile, oppure Trunk) | web-release |
| P1.2 | `web/index.html`: loader, errore se WebGL2 assente, "clicca per giocare" (autoplay audio dei browser) | web-release |
| P1.3 | Server HTTP locale + Playwright: screenshot, console senza errori fatali, input | web-release + qa-performance |
| P1.4 | `scripts/build-web.ps1` e `.sh`, output `dist/` | web-release |
| P1.5 | Dimensioni `.wasm` (< 25 MiB), MIME, cache busting, percorsi asset; `docs/RELEASE.md` | web-release |
| P1.6 | CI GitHub Actions (fmt, clippy, test, build nativa, artefatto web) — serve un remote GitHub: chiedere all'utente | web-release |

## P2 / P3

Come da handoff §6 (art pass, animazioni, audio; profiling 50/200/500 nemici, bilanciamento,
opzioni, browser, release candidate).

## Debito tecnico e idee registrate

- Radure di terra con bordi a gradini: servono tile di transizione erba/terra (P2, pixel-art-director).
- Testo HUD/overlay con FiraMono a risoluzione nativa: sostituire con font pixel OFL o
  bitmap originale e decidere se l'HUD va nel canvas 480×270 (P0.7/P2).
- Separazione nemici O(n²): sostituire con griglia spaziale in P0.3 (prima dei 200 nemici).
- Replay identici tra nativo e WASM: valutare la feature `libm` di Bevy/glam per `sin`/`cos`
  deterministici e misurarne il costo (serve solo se si vogliono seed condivisi tra piattaforme).
- Valutare `rust-lld` come linker su Windows per accorciare i link incrementali (misurare prima/dopo).
- Build release non ancora provata (LTO thin + codegen-units 1): misurare tempi in P3.1.
- Su Windows il gioco lanciato fuori da `cargo run` cerca `assets/` accanto all'eseguibile:
  impostare `BEVY_ASSET_ROOT` o copiare `assets/` nel pacchetto (P3.5).
