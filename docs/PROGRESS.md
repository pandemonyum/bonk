# PROGRESS

Stato corrente, ultimo aggiornamento: **2026-10-22**.

| Fase | Stato |
|---|---|
| S0 — bootstrap e ambiente | **DONE** (gate S0 superato, prove sotto) |
| P0 — core giocabile desktop | in corso: P0.0–P0.5 DONE |
| P1 — web | non iniziata |
| P2 / P3 | non iniziate |

Prossimo task eseguibile: **P0.6** (timer di 10 minuti, scala di difficoltà, élite, boss con
telegraph, stati MainMenu/GameOver/Victory, restart), vedi `docs/BACKLOG.md`.

## Report tranche P0.5 (2026-10-22)

```text
Task: P0.5 - Secondo e terzo compagno (onda in area, foglie orbitanti), slot da dati
Owner: gameplay-engineer + game-designer (valori) + pixel-art-director (sprite)
Stato: DONE (code review: APPROVE, nessun bloccante; 3 rilievi da correggere e 9 minori
       gestiti, dettaglio in docs/QA_REPORT.md)
File: src/gameplay/{pulse,orbit}.rs, src/presentation/attacks.rs, src/data/validate/attacks.rs
       (+ tests), tests/sim/{patterns,party}.rs, assets-src/sprites/{tide_companion,
       sprout_companion,leaf,droplet}.px (nuovi); src/data/{schema,validate}.rs,
       src/data/validate/progression.rs (+ tests), src/gameplay/{companions,attacks,combat,
       enemies,mod,upgrades}.rs, src/gameplay/upgrades/{modifiers,tests}.rs,
       src/presentation/{sprites,draft,mod}.rs, game-data/*.ron (schema v4), palette.txt,
       tests/sim/{common,combat,progression,upgrades,balance,main}.rs
Comandi eseguiti (risultato):
  ./scripts/check.ps1                       OK: 186 test (121 unit + 65 integrazione), 4 sonde
  controprove (13 regole rotte apposta)     falliscono esattamente i test mirati (QA_REPORT)
  sonda livelli (5 seed)                    invulnerabile: livello finale 33-36 a 10:00
  sonda sopravvivenza con autoplay          1:08-1:22, livello 2-3 (invariata)
  cargo run -- --seed 20 --autoplay --god --time-scale 3/4 --overlay
                                            Frond e Gocci ai livelli 2 e 3; onda e 4 foglie
                                            visibili, livello 9 a 2:41, 60 FPS
Comportamento verificato:
  - onda: un colpo per onda a ogni nemico raggiunto, solo dentro il raggio, rallentamento
    del valore e della durata dei dati, parte solo con un bersaglio, cadenza esatta
  - foglie: equidistanti sul raggio, seguono il player, stesso nemico al massimo ogni intervallo
  - carta "nuovo compagno": posto libero, upgrade già presi, direzione della squadra
  - upgrade successivi su tutti i pattern (gittata = area e orbita, +1 proiettile = +1 foglia)
  - squadra piena: niente carte; restart: solo il compagno iniziale, nessuna foglia o onda
  - stesso seed, stessa run anche con onda e foglie
Problemi trovati e risolti: conteggio delle onde con Added<> dall'esterno dello schedule
  vacuo (riscritto con entità distinte); i rilievi del code review (vedi QA_REPORT).
Rischi noti: la run con autoplay finisce ancora a ~1:10 (sciame al minuto 1); foglie poco
  leggibili sull'erba; scelta del compagno iniziale solo nei dati fino al menu della P0.6.
Prossimo task: P0.6
```

## Report tranche P0.4 (2026-10-22)

```text
Task: P0.4 - Gemme XP, livelli, draft di 3 carte con pausa, upgrade applicati a caldo
Owner: gameplay-engineer + game-designer (valori, testi) + pixel-art-director (gemma, carte)
Stato: DONE (code review: CHANGES REQUESTED, nessun bloccante; 3 rilievi da correggere e 7
       minori corretti, 4 plausibili gestiti, dettaglio in docs/QA_REPORT.md)
File: src/gameplay/{progression,upgrades}.rs, src/gameplay/upgrades/tests.rs,
       src/data/validate/progression.rs, src/presentation/draft.rs, tests/sim/progression.rs,
       src/gameplay/upgrades/modifiers.rs, src/data/validate/progression/tests.rs,
       tests/sim/upgrades.rs, game-data/{progression,upgrades}.ron,
       assets-src/sprites/xp_gem.px (nuovi);
       src/gameplay/{mod,combat,enemies,player,input,damage}.rs, src/data/{mod,schema,loader,
       validate}.rs, src/presentation/{hud,sprites,mod}.rs, src/diagnostics/overlay.rs,
       src/{app,lib}.rs, game-data/*.ron (schema v3), tests/sim/{common,balance,movement}.rs
Comandi eseguiti (risultato):
  ./scripts/check.ps1                       OK: 158 test (105 unit + 53 integrazione), 4 sonde
  controprove (8 regole rotte apposta)      falliscono esattamente i test mirati (QA_REPORT)
  sonda livelli (5 seed, prima carta)       primo level-up con autoplay 0:30-0:41
  sonda livelli invulnerabile (10 min)      Lv5 ~2:20, Lv10 4:39-5:56, finale Lv 14-25
  gioco vero, input reale (keybd/mouse)     draft aperto, tasto 2 e clic su 1a e 3a carta
  cargo run -- --autoplay --god --time-scale 4 --overlay   livello 4 a 2:00, 0,6 s di draft
Comportamento verificato:
  - ogni nemico sconfitto lascia una sola gemma anche se colpito due volte nello stesso passo
  - oltre il limite di gemme a terra l'XP si fonde senza perdite
  - il draft ferma tempo, colpi e orologio; col tempo x4 il passo del livello è l'ultimo
  - +1 proiettile: raffica e proiettili in campo = base + 1; +15% danno: il colpo vero toglie
    il danno nuovo; HP massimi, cura, rigenerazione, velocità e raggio di raccolta misurati
  - restart: livello 1, nessuno stack, attacco ai valori base
Problemi trovati e risolti: panic all'avvio del gioco vero (ricalcolo degli upgrade prima che
  la configurazione esista, invisibile in headless) → condizione su InRun + test che simula
  l'attesa dei dati; lettere accentate non disegnate dal font → testi ASCII e validazione.
Rischi noti: con scelta casuale e autoplay cieco la run dura ~1:10-1:24; font senza accenti.
Prossimo task: P0.5
```

## Report tranche P0.3 (2026-10-22)

```text
Task: P0.3 - Spawn director, 3 archetipi, danni al player, griglia spaziale
Owner: gameplay-engineer + game-designer (valori) + pixel-art-director (sprite) (sequenziale)
Stato: DONE (commit 654770e; code review: 7 rilievi corretti nel commit successivo, dettaglio
       in docs/QA_REPORT.md; dopo le correzioni 122 test)
File: src/gameplay/{spawning,spatial,damage,run}.rs, src/world/view.rs (nuovi),
       src/gameplay/{mod,enemies,combat,player,input}.rs, src/data/{schema,validate,mod,loader}.rs,
       src/presentation/{camera,hud,sprites}.rs, src/diagnostics/overlay.rs, src/{app,lib,options}.rs,
       game-data/*.ron (schema v2), assets-src/sprites/{swarm,armored}_enemy.px,
       tests/sim/{spawning,damage}.rs (nuovi), tests/sim/{common,movement,combat,world,balance}.rs
Comandi eseguiti (risultato):
  ./scripts/check.ps1                       OK: 115 test (84 unit + 31 integrazione), 3 sonde
  sonda costo per passo                     0,45 / 0,49 / 0,62 ms con 50 / 200 / 500 nemici
  sonda sopravvivenza (5 seed)              fermi 0:13, autoplay 1:06-1:16 (dopo taratura)
  cargo run -- --autoplay --god --time-scale 4/8 --overlay   135 e 220 nemici, 59 FPS / 16,9 ms
  sconfitta reale + Invio da tastiera        run nuova, statistiche azzerate
Comportamento verificato:
  - controprova: con il director che ignora la vista i test di spawn falliscono ("nato in vista")
  - ritmo di nascita = integrale esatto delle rette di stages.ron (minuto 1 e 3)
  - nell'angolo nord-est (camera non centrata) gli spawn continuano, tutti validi
  - danno sommato, finestra di invulnerabilità, god mode, sconfitta che ferma orologio e mondo
Problemi trovati e risolti: prima taratura troppo dura (fermi 0:10); tre test con valori di
  gioco scritti a mano (contro AGENTS.md) resi dipendenti dalla configurazione; un test di
  spawn che verificava il director con la sua stessa funzione, riscritto con controlli propri.
Rischi noti: player poco visibile dentro orde da 200+; senza upgrade la run dura ~1 minuto.
Prossimo task: P0.4
```

## Report tranche "code review P0" (2026-10-22)

```text
Task: correzioni del code review su P0.2, P0.1, P0.0
Owner: code-reviewer (lettura) → gameplay-engineer / world-generation (correzioni)
Stato: DONE
File: src/gameplay/{combat,player}.rs, src/data/{loader,schema,validate}.rs,
       src/world/{mod,obstacles}.rs, game-data/{attacks,stages}.ron, tests/sim/{combat,world}.rs,
       docs/*
Comandi eseguiti (risultato):
  ./scripts/check.ps1                       OK: 92 test (70 unit + 22 integrazione), 1 ignorato
  controprove con i bug reintrodotti         i due test nuovi falliscono (vedi QA_REPORT)
  cargo run -- --autoplay ...                OK, dati caricati, exit 0
Comportamento verificato: 7 rilievi corretti, dettaglio in docs/QA_REPORT.md
Rischi noti: compilazione WASM da riverificare in P1.
Prossimo task: P0.3
```

## Report tranche P0.0 + riduzione spazio su disco (2026-10-22)

```text
Task: P0.0 - Dati di gioco esterni in game-data/ ; D-027 - spazio su disco
Owner: gameplay-engineer + game-designer (valori) + qa-performance (sequenziale)
Stato: DONE
Assunzioni: game-data/ resta in radice (sorgente di asset data://); valori invariati
       rispetto alle costanti della P0.2/P0.1; ricarica a caldo attiva di default in debug.
File: src/data/{mod,schema,validate,loader}.rs (nuovi), game-data/*.ron (5, nuovi),
       src/{app,lib,options}.rs, src/gameplay/{player,companions,enemies,combat,collision,input}.rs,
       src/presentation/{sprites,hud}.rs, src/world/obstacles.rs, tests/sim/* (unificati),
       scripts/{validate-assets.py,clean-stale.ps1,clean-stale.sh}, Cargo.toml, docs/*
Comandi eseguiti (risultato):
  cargo clean                              rimossi 9423 file, 22,3 GiB
  rustup toolchain uninstall stable        OK (default 1.99.1)
  cargo build / test --no-run / clippy     OK: 837 s; target/ 2,62 → 2,85 → 3,31 GB
  ./scripts/check.ps1                      OK: 79 test (58 unit + 21 integrazione), 1 ignorato
  cargo run (dati validi / non validi / modifica a caldo)   OK, vedi QA_REPORT
Comportamento verificato:
  - i 5 file RON vengono caricati dall'AssetServer; tutti i valori di bilanciamento escono
    dal codice; i test leggono i valori attesi dalla configurazione
  - errori di sintassi (file:riga:colonna), campi sconosciuti, versioni, numeri fuori range,
    riferimenti tra file, vincoli fisici (corpo vs varco ostacoli, proiettile vs hurtbox)
  - schermata d'errore in gioco; ricarica a caldo con restart della run; F5 = restart pulito
    (test: stesso censimento di entità di una run nuova, anche partendo dalla pausa)
Problemi trovati e risolti: errore di link LNK1318 transitorio (contesa sui .pdb), sparito al
  secondo tentativo e non più possibile con i simboli ridotti; nella prima stesura del loader
  correggere un file dopo un errore al primo avvio non aveva effetto (unificato in un sistema).
Rischi noti: compilazione WASM da riverificare in P1; ricarica degli sprite non provata.
Prossimo task: P0.3
```

## Report tranche P0.1 (2026-10-22)

```text
Task: P0.1 - Ostacoli procedurali e collisioni
Owner: world-generation + gameplay-engineer + pixel-art-director (sequenziale)
Stato: DONE
File: src/world/obstacles.rs, src/gameplay/collision.rs (nuovi), src/world/mod.rs,
       src/gameplay/{mod,player,enemies}.rs, src/presentation/sprites.rs,
       tests/world_sim.rs, assets-src/sprites/{rock,tree}.px, assets/**, docs/*
Comandi eseguiti (risultato):
  ./scripts/check.ps1                     OK: 65 test (46 unit + 8 + 7 + 4), 7 PNG
  cargo run -- --autoplay --overlay ... (seed default e seed 7)   OK, exit 0
Comportamento verificato:
  - ~650 ostacoli per seed (test: 400-900), stesso seed = stessi ostacoli, zona di partenza
    libera (128 px), varchi sempre ≥ 12 px, flood fill: tutti i tile liberi raggiungibili
  - griglia spaziale: stessi risultati della ricerca esaustiva su 3 punti di prova
  - player: mai dentro un'impronta, scivola attorno se arriva di lato, si ferma sul bordo
    se punta dritto al centro
  - nemici: entro il limite documentato (al più CONTACT_DISTANCE di penetrazione, D-021)
  - 58-62 FPS con ~1150 entità in build dev (overlay, non benchmark)
  - screenshot: docs/evidence/p01-ostacoli-alberi.png, p01-ostacoli-radura.png
Problemi trovati e risolti: albero inizialmente più basso dell'evocatore → ridisegnato 24×26.
Rischi noti: un nemico allineato esattamente player-ostacolo può fermarsi dietro l'ostacolo
  (nessun pathfinding, D-022); i proiettili attraversano gli ostacoli per scelta.
Prossimo task: P0.0
```

## Report tranche P0.2 (2026-10-22)

```text
Task: P0.2 - Primo compagno con attacco automatico
Owner: gameplay-engineer + pixel-art-director (sequenziale, stesso agente)
Stato: DONE
Assunzioni: nemici con HP e morte già in P0.2 (senza, l'attacco non ha effetto visibile);
       popolazione mantenuta a 10 con rimpiazzi fuori schermo finché non c'è lo spawn director.
File: src/gameplay/{combat,companions}.rs (nuovi), src/gameplay/{mod,enemies}.rs,
       src/presentation/{mod,sprites}.rs, src/diagnostics/{overlay,automation}.rs,
       src/options.rs, tests/{combat_sim,balance_probe}.rs, tests/common/mod.rs,
       tests/headless_sim.rs, assets-src/{palette.txt,sprites/ember_companion.px,
       sprites/fireball.px}, assets/**, docs/*, ASSET_LICENSES.md
Comandi eseguiti (risultato):
  ./scripts/check.ps1                     OK: fmt, clippy -D warnings, 55 test
                                          (40 unit + 8 combattimento + 7 movimento), 5 PNG
  cargo test --test balance_probe -- --ignored --nocapture
                                          OK: 54-55 colpi/min, 100% a segno, 14-18 sconfitti/min
  cargo run -- --autoplay --overlay --screenshot ... --screenshot-burst 8
                                          OK, exit 0, 8 screenshot
Comportamento verificato:
  - il compagno segue dietro al player (in corsa < 33 px, fermo a 18±1 px) e spara da solo
  - cadenza esatta: 10 colpi in 600 passi con intervallo 1,1 s (66 passi)
  - danno 12 → HP 30→18 e tinta di colpo; un nemico colpito due volte nello stesso passo
    muore e viene contato una sola volta; popolazione mai sopra 10
  - +2 proiettili per attacco → 3 proiettili in volo (base per l'upgrade della P0.4)
  - screenshot con proiettile appena sparato: docs/evidence/p02-compagno-proiettile.png
Problemi trovati e risolti: prima sagoma del compagno identica alla gelatina (ridisegnato);
  ipotesi "molti colpi a vuoto" smentita dalla sonda (100% a segno), niente mira predittiva.
Rischi noti: rimpiazzi visibili se il player è vicino al bordo (zone di spawn in P0.3);
  collisioni proiettile×nemico O(n×m) da sostituire con griglia spaziale in P0.3.
Prossimo task: P0.1
```

---

## S0.1 — Ispezione ambiente (2026-10-22)

| Elemento | Esito |
|---|---|
| OS | Windows 11 Enterprise 10.0.26200 |
| CPU / RAM | Intel Core Ultra 5 238V (8 core), 31,5 GiB |
| GPU | Intel Arc 130V integrata, backend Vulkan (driver 101.8737) |
| Cartella | conteneva solo `HANDOFF_PIXEL_MONSTER_SURVIVOR.md`; non era un repo Git |
| `references/pixel_art_concept.png` | **assente**: si procede senza moodboard |
| Git | 2.54.0 (`user.name` globale presente, email impostata solo nel repo) |
| Python | 3.14.4 + Pillow 12.2.0 |
| Node | presente (nvm4w) |
| Rust | assente → installato rustup 1.29 via winget; toolchain bloccata **1.99.1** |
| MSVC | assente → installati **VS Build Tools 2022 17.14.42** + Windows SDK 10.0.26100 via winget |
| Target WASM | `wasm32-unknown-unknown` installato con la toolchain |
| Trunk / wasm-bindgen / wasm-opt | assenti (servono dalla P1) |
| Codex CLI | assente |
| Disco | ~37 GB liberi all'inizio; `target/` dopo la S0: 10,4 GB |

## Report tranche S0

```text
Task: S0.1–S0.5 - Bootstrap, agenti, prima app Bevy 2D
Owner: orchestrator (sequenziale, ruoli gameplay-engineer / world-generation /
       pixel-art-director / qa-performance nello stesso agente)
Stato: DONE
Assunzioni: default approvati dall'utente (cartella Bonk come radice, arena chiusa,
       mondo 480x270, WebGL2 in P1, sprite generati da script, documenti brevi).
File: AGENTS.md, CLAUDE.md, .claude/agents/*.md (7), docs/agents/README.md, docs/*.md,
       Cargo.toml, Cargo.lock, rust-toolchain.toml, src/** (21 file), tests/headless_sim.rs,
       assets-src/**, assets/**, scripts/{build-assets,validate-assets,check-pixel-grid}.py,
       scripts/check.{ps1,sh}, ASSET_LICENSES.md, README.md, .gitignore
Comandi eseguiti (risultato):
  winget install Rustlang.Rustup                                   OK
  winget install Microsoft.VisualStudio.2022.BuildTools (VCTools)   OK
  rustup toolchain install 1.99.1 (+clippy, rustfmt, wasm32)       OK
  cargo build (prima, con tutte le dipendenze)                     OK, 1384 s (~23 min)
      (il primo tentativo è fallito solo perché lib.rs è stato creato a build avviata)
  cargo build incrementale                                         OK, 27–60 s
  cargo fmt --all -- --check                                       OK (dopo cargo fmt)
  cargo clippy --all-targets -- -D warnings                        OK, 0 warning
  cargo test                                                       OK, 38 test (31 unit + 7 headless)
      due fallimenti reali trovati e corretti durante la tranche:
      - radure di terra al 15% dell'arena → soglia 0.74 → 0.80
      - nemici spinti dentro il player dalla separazione → nuovo sistema keep_out_of_player
  code review (ruolo code-reviewer): CHANGES REQUESTED, 7 rilievi, tutti corretti e
      riverificati (dettaglio in docs/QA_REPORT.md)
  python scripts/build-assets.py / validate-assets.py              OK, 3 PNG, 0 errori
  ./scripts/check.ps1                                              OK (fmt, clippy, test, asset)
  cargo run -- --autoplay --overlay --screenshot ...               OK, exit 0, screenshot salvato
  python scripts/check-pixel-grid.py (1366x768, 1365x767,
      1101x701 con --scale-factor 1.25 e 1.5)                       OK, 100% blocchi uniformi
Comportamento verificato (osservato, non solo compilato):
  - finestra 960x540, mondo a 480x270 scalato 2x, pixel netti (docs/evidence/s0-autoplay-960x540.png)
  - arena procedurale da seed con erba, fiori, radure; 10 gelatine che inseguono e si separano
  - tastiera reale (keybd_event di Windows): D 1 s + W 1 s → player a (76,75), atteso (75,75);
    Esc → pausa con banner e simulazione ferma (docs/evidence/s0-keyboard-pausa.png)
  - finestra 1365x767 (dimensioni dispari): letterbox nero, scala 2x, 100% texel uniformi;
    con la correzione di mezzo pixel disattivata lo stesso controllo scende al 96,1% e fallisce
    (docs/evidence/s0-letterbox-1365x767.png)
  - DPI forzato 1.25 e 1.5: UI ingrandita, canvas sempre 2x con texel perfetti
    (docs/evidence/s0-dpi150-1101x701.png)
  - overlay F3: 60–64 FPS, 15,7–16,5 ms dopo 9–15 s, build dev, 10 nemici, vsync attivo.
    Non è un benchmark: niente release, scenario minimo (vedi docs/QA_REPORT.md)
Rischi o bug noti: vedi "Limiti aperti" in docs/QA_REPORT.md
Prossimo task: P0.1
```
