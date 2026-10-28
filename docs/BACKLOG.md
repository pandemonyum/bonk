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
| ~~P0.0~~ **DONE** | Loader `game-data/*.ron` via AssetServer (funziona anche su WASM) + validazione all'avvio con messaggio d'errore; spostare in RON i valori oggi costanti (velocità, raggi, n. nemici, attacco) | gameplay-engineer | test di parsing e di validazione (dato errato → errore leggibile); `cargo run` (ricarica a caldo) applica i valori salvati |
| ~~P0.3~~ **DONE** | Spawn director: 3 archetipi nemici da dati, curva per minuto, cap per archetipo, spawn fuori camera e mai sul player; HP, danno al player, morte, hit feedback, despawn; griglia spaziale al posto della separazione O(n²) | gameplay-engineer + game-designer (valori) | test: nessuno spawn dentro la camera/il player, cap rispettato, restart non accumula entità; overlay con conteggi |
| ~~P0.4~~ **DONE** | Gemme XP (drop una sola volta), raccolta per prossimità, livelli, draft di 3 carte con pausa, effetti applicati a caldo | gameplay-engineer + game-designer | test: +1 proiettile cambia i proiettili visibili, +danno cambia il danno effettivo; screenshot del draft |
| ~~P0.5~~ **DONE** | Secondo e terzo companion (onda in area, foglie orbitanti), slot da dati; un compagno aggiunto a metà run deve nascere con gli upgrade già presi (oggi `refresh_modifiers` gira solo quando cambiano gli stack) | gameplay-engineer | test per ciascun pattern di attacco; test: compagno nuovo con i modificatori correnti; screenshot |
| ~~P0.6~~ **DONE** | Timer 10 min, scala difficoltà, élite, boss con telegraph, stati MainMenu/GameOver/Victory, restart | gameplay-engineer | test headless: run completa a seed fisso con autoplay arriva a fine senza panic; restart pulisce lo stato |
| ~~P0.7~~ **DONE** | HUD (HP, compagni, tempo, risorse, abilità, XP) leggibile; test formule e bilanciamento base | gameplay-engineer (HUD in `src/presentation/`) + qa-performance (`tests/sim/`) + game-designer (valori e GDD); orchestrator assegna e integra | interfaccia che scala con il canvas (testi, pannelli e icone con lo stesso pixel a ogni finestra); barra HP sotto il player; barra delle abilità con un posto per compagno, ricarica visibile e posti liberi bloccati; upgrade presi con gli stack; indicatore del boss fuori schermo; test di integrazione che misurano danno al secondo, cadenza con gli upgrade e livelli dall'XP con i valori di `GameConfig`; sonda "giocatore medio" (schiva, raccoglie, sceglie carte sensate) e valori tarati sugli obiettivi del GDD; gate verde; screenshot HUD a 960×540 e a finestra grande |

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

## Grafica del player (2026-10-28, su richiesta dell'utente)

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~ART-P1~~ **DONE** | Player animato: fermo e cammino in 4 direzioni, colpito, caduta; sprite identico a quello fornito dall'utente (D-050) | pixel-art-director (foglio, pipeline) + gameplay-engineer (`player_anim`, risultati dopo la caduta) | foglio 4×8 validato; test del contratto sul PNG; anteprime approvate dall'utente prima dell'integrazione; cammino nelle 4 direzioni e caduta nel gioco vero; gate verde |

## Debito tecnico e idee registrate

- Player (D-050): somiglianza con personaggi Nintendo accettata dall'utente, **da rivalutare
  prima di pubblicare**; personaggio più grande dei compagni (~30 contro ~20 di altezza) e
  copre in parte quello dietro; sporge sulla siepe a nord (~6 texel). Se serve più piccolo:
  ridisegno a ~24 di altezza, non un ridimensionamento (pixel irregolari).

- Radure di terra con bordi a gradini: servono tile di transizione erba/terra (P2, pixel-art-director).
- ~~Testo HUD con FiraMono~~: Tiny5 e Pixelify Sans (D-044); ~~testi a dimensione fissa a
  finestra grande~~: tutta l'interfaccia scala con il canvas (D-045). Le misure
  dell'interfaccia vanno tenute pari (1 texel = 2 unità) e Tiny5 a 16 o 32 px.
- ~~Separazione nemici O(n²)~~: sostituita dalla griglia spaziale in P0.3.
- Il player si perde dentro l'orda a 200+ nemici: contorno/ombra più marcati o un indicatore
  sotto i piedi (P2, pixel-art-director); ~~HP vicino al player~~: barra sotto i piedi (P0.7).
- **Prova umana (prossimo passo del bilanciamento):** la taratura della P0.7 usa il "giocatore
  medio" di `tests/sim/bot.rs` (23/24 vittorie, boss in ~66 s). Serve una run giocata da una
  persona per ogni compagno iniziale, annotando a che minuto si va in difficoltà, se il boss
  dura troppo o poco e se il compagno garantito nei primi draft basta.
- Font in `assets/fonts/` (~0,5 MB, variabili con assi inutilizzati): per il web (P1) valutare un
  sottoinsieme statico Latin-1 (l'OFL lo permette; il nome va cambiato solo se si modifica un
  font con Reserved Font Name, e questi non ne hanno).
- La tinta di colpo e di rallentamento colora anche l'ombra cotta negli sprite v1 (accettato;
  in alternativa l'ombra come sprite figlio).
- Il livello a 10:00 arriva quasi sempre a 36–37 con il giocatore medio: tutti i 1592 nemici
  generati vengono sconfitti e l'XP finisce. Da valutare con la prova umana (curva XP o
  ondate) se gli ultimi minuti devono far crescere ancora.
- L'élite carica anche quando è già a contatto (scatto inutile, innocuo): valutare una
  distanza minima.
- ~~Il boss nasce fuori vista~~: indicatore al bordo dello schermo (P0.7). Resta da valutare un
  avviso "arriva il boss" al centro.
- Foglie di Frond (6×6, lime) poco leggibili sull'erba: valutare contorno più marcato o una
  scia (P2, pixel-art-director).
- Replay identici tra nativo e WASM: valutare la feature `libm` di Bevy/glam per `sin`/`cos`
  deterministici e misurarne il costo (serve solo se si vogliono seed condivisi tra piattaforme).
- Valutare `rust-lld` come linker su Windows per accorciare i link incrementali (misurare prima/dopo).
- Build release non ancora provata (LTO thin + codegen-units 1): misurare tempi in P3.1.
- Su Windows il gioco lanciato fuori da `cargo run` cerca `assets/` accanto all'eseguibile:
  impostare `BEVY_ASSET_ROOT` o copiare `assets/` nel pacchetto (P3.5).
