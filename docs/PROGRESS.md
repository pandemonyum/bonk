# PROGRESS

Stato corrente, ultimo aggiornamento: **2026-10-23**.

| Fase | Stato |
|---|---|
| S0 — bootstrap e ambiente | **DONE** (gate S0 superato, prove sotto) |
| P0 — core giocabile desktop | in corso: P0.0–P0.7 DONE, manca la prova umana del gate |
| P1 — web | non iniziata |
| P2 / P3 | non iniziate |

Prossimo task eseguibile: **gate P0** (10 minuti giocabili end-to-end: serve la prova umana
del bilanciamento, vedi `docs/BACKLOG.md`), poi **P1** (web). Art pass v1, font pixel e P0.7
sono integrati in `main`.

## Report tranche P1 "web giocabile" (2026-10-29)

```text
Task: P1.1-P1.5 + P1-SAVE - il gioco nel browser da un dist/ statico (D-061)
Owner: orchestrator (ruolo web-release svolto direttamente: le modifiche toccavano anche src/)
Stato: DONE tranne P1.6 (CI: serve un remote GitHub, da chiedere all'utente)
File nuovi: web/index.html, scripts/build-web.ps1, scripts/build-web.sh, src/web.rs,
  docs/RELEASE.md, .claude/launch.json; cambiati: Cargo.toml (web-sys solo wasm, profilo
  wasm-release), src/lib.rs (canvas, meta), src/options.rs, src/profile.rs, src/world/render.rs
Comandi eseguiti (risultato):
  cargo install wasm-bindgen-cli --version 0.2.129 --locked   OK
  ./scripts/build-web.ps1   OK: wasm grezzo 25,2 MiB, in dist 22,5 MiB, gzip 5,9 MiB; dist 23,3 MiB
  ./scripts/check.ps1       OK (215 unit + 101 integrazione, 193 PNG)
  browser (Chromium del pannello, WebGL2 su ANGLE/D3D11): menu, run con tastiera, draft,
    sconfitta e risultati, profilo nel localStorage, ?autoplay&seed=3, versione nuova senza cache
Problemi trovati e risolti: 404 sui .meta degli asset; terreno nero (TilemapChunk e
  Rgba16Uint su WebGL2: terreno cotto in sprite, uguale al pixel sul desktop); .wasm da 54 MiB
  (nomi delle funzioni); script PowerShell 5.1 (variabile "$want:", stderr di cargo con Stop,
  pagina letta in ANSI)
Rischi noti: un avviso WebGL "texSubImage2D: invalid format" all'avvio di Bevy, senza effetti
  visibili; bande nere quando la finestra non è un multiplo di 480×270 (P1-FILL); wasm-opt non
  installato; provato su un solo browser
Prossimo task: P1.6 (CI) quando c'è un remote; pubblicazione solo con il via dell'utente
```

## Report tranche "riepilogo, squadra per livelli, orde, forzieri, talenti" (2026-10-28)

```text
Task: U-RECAP, U-SQUAD, U-HORDE, U-CHEST, U-TALENT - richieste dell'utente (D-054..D-058)
Owner: orchestrator (codice e integrazione) + game-designer (proposta di talenti, monete,
       forzieri, livelli: docs/design/TALENTS_AND_CHESTS.md) + pixel-art-director (moneta,
       forziere, icone HUD, 9 reliquie, 24 talenti; compagni piccoli in anteprima)
Stato: DONE (tranne U-SMALL, in attesa del via dell'utente)
File nuovi: src/profile.rs, src/gameplay/{hordes,treasure,talents,talent_effects}.rs,
  src/data/schema/{run,treasure,talents}.rs, src/presentation/{recap,horde_banner,
  treasure_art,chest_ui,talent_art,talents_menu}.rs, tests/sim/{recap,hordes,treasure,
  talents}.rs
Comandi eseguiti (risultato):
  ./scripts/check.ps1   OK (vedi QA_REPORT per i numeri dell'ultimo giro)
  sonda giocatore medio: 27/32 senza talenti (84%); con tutti i talenti 32/32 anche dopo il taglio dei nodi 3, danni subiti 23–127 (~37% di quelli senza talenti): la guardia del 97%/50% resta superata, da rivedere con la prova umana o con l'albero nuovo
  gioco vero: riepilogo con esperienza, orda a 5:00 con annuncio, forziere aperto con le
    reliquie, freccia e monete nell'HUD, esplosione e soffio dei talenti, schermata dei talenti
Problemi trovati e risolti: l'HUD leggeva i dati di gioco prima del caricamento e il gioco si
  chiudeva all'avvio (già nel commit 36993a2; i test non lo vedevano perché hanno i dati
  subito); i compagni nascevano prima che il profilo passasse i talenti alla run (ora li
  leggono dal profilo); la freccia del forziere copriva il draft; "1 monete"; Clessidra −20%
  oltre i limiti dell'onda; frecce ← → non disegnate dai font
Rischi noti: talenti al massimo ancora molto forti per il bot; profilo sul web solo in memoria
  fino alla P1; i compagni sono ancora grandi quanto il player (U-SMALL)
Prossimo task: via dell'utente sui compagni piccoli; prova umana; P1
```

## Report tranche "compagni dell'utente e Tuonny" (2026-10-28)

```text
Task: ART-P4 + P-TUONNY - compagni animati dal foglio dell'utente; Tuonny con fulmine a catena
Owner: orchestrator + pixel-art-director (fogli, ritratti, scintilla) + gameplay-engineer
       (companion_anim, chain) + game-designer (valori del fulmine, sonde)
Stato: DONE (commit dei tre compagni 1fb78a9, poi Tuonny)
File (Tuonny): src/gameplay/chain.rs (nuovo), src/gameplay/{mod,combat,companions,upgrades}.rs,
  src/gameplay/upgrades/{modifiers,rules}.rs, src/data/schema.rs, src/data/schema/attacks.rs,
  src/data/validate/{attacks,progression}.rs, src/presentation/{attacks,abilities,
  companion_anim,icons,menu}.rs, game-data/{attacks,companions,upgrades}.ron,
  tests/sim/{chain,formulas,balance,main}.rs, assets/sprites/spark.png (+ .px),
  docs/{GDD,DECISIONS,BACKLOG,PROGRESS,QA_REPORT,ART_BIBLE}.md, docs/evidence/tuonny-*.png
Comandi eseguiti (risultato):
  ./scripts/check.ps1   OK (fmt, clippy, 174 unit + 86 integrazione, 48 PNG e 2 font)
  sonda giocatore medio con la composizione della squadra: vedi GDD, Misure (D-053)
  gioco vero: menu a quattro carte con il tasto 4, run con Tuonny e fulmini a zigzag
Problemi trovati e risolti: la squadra senza Ondino crollava al minuto 4 (fulmine rafforzato e
  scossa); formula della cadenza: il fulmine istantaneo colpisce anche all'istante 0
Rischi noti: squadra senza Ondino ancora più dura delle altre (voluto, da sentire con una
  persona); compagni grandi quanto il player (D-052)
Prossimo task: prova umana, P1
```

## Report tranche "mappa" (2026-10-28)

```text
Task: ART-P3 - mappa nello stile del player e dei nemici
Owner: pixel-art-director (tileset, roccia, albero, pulizia) + orchestrator (verifica, commit)
Stato: DONE
File: scripts/build-assets.py (disegno dei tile; nomi, seed e maschere invariati),
  assets/tiles/meadow_tileset.png, assets-src/sprites/{rock,tree}.px, assets/sprites/
  {rock,tree}.png, assets/manifest.json, scripts/preview-scene.py, ASSET_LICENSES.md,
  docs/{ART_BIBLE,BACKLOG,PROGRESS,QA_REPORT}.md, docs/art/STYLE_HANDOFF.md,
  docs/evidence/{art-preview-*,mappa-gioco}.png; tolti {blob,swarm,armored,elite,boss}_enemy
  (.px e .png)
Comandi eseguiti (risultato): build-assets.py / validate-assets.py OK (43 PNG);
  ./scripts/check.ps1 OK (166 unit + 82 integrazione); gioco vero con seed 3 e 11
Rischi noti: alberi più grandi coprono di più chi passa dietro; compagni ancora nello stile v1
Prossimo task: ART-P4 (compagni) da proporre, prova umana, P1
```

## Report tranche "nemici e boss animati" (2026-10-28)

```text
Task: ART-P2 - nemici e boss animati in proporzione al player
Owner: orchestrator + pixel-art-director (5 fogli, palette, validatore) + gameplay-engineer
       (src/presentation/enemy_anim.rs, schema `frame`) + game-designer (valori)
Stato: DONE (la mappa, ART-P3, segue)
File: src/presentation/enemy_anim.rs (nuovo), src/presentation/{mod,sprites}.rs,
  src/data/schema.rs, src/data/validate/run.rs (+ tests), game-data/enemies.ron,
  assets-src/sprites/{blob,swarm,armored,elite,boss}_sheet.px e enemies.palette.txt,
  assets/sprites/*_sheet.png, assets/manifest.json, scripts/validate-assets.py,
  ASSET_LICENSES.md, docs/{ART_BIBLE,DECISIONS,BACKLOG,PROGRESS,QA_REPORT}.md,
  docs/evidence/{nemici,boss}-animati-gioco.png
Comandi eseguiti (risultato):
  ./scripts/check.ps1   OK (fmt, clippy, 166 unit + 82 integrazione, 48 PNG e 2 font)
  sonda giocatore medio (24 run): contatto alzato per tutti 16/24; contatto dei piccoli come
    prima 24/24, boss 46–81 s, danni 89–416
  gioco vero (autoplay ×10): orda animata con resti, élite, boss che si gira e colpisce
Comportamento verificato: fogli approvati dall'utente prima dell'integrazione; nemici
  specchiati sul verso; resti solo per chi muore
Rischi noti: boss simile a una tartaruga spinosa (Bowser) insieme al player (accettato);
  sprite fissi dei nemici ancora in assets/ finché la ART-P3 non li toglie
Prossimo task: ART-P3 (mappa)
```

## Report tranche "player animato" (2026-10-28)

```text
Task: ART-P1 - player con animazioni (fermo, cammino in 4 direzioni, colpito, caduta), uguale
      allo sprite dell'utente
Owner: orchestrator (integrazione, decisioni con l'utente) + pixel-art-director (foglio,
       pipeline) + gameplay-engineer (src/presentation/player_anim.rs, risultati)
Stato: DONE
Assunzioni: l'utente dichiara i diritti sul foglio che ha fornito e accetta il rischio di
  somiglianza (D-050); i limiti dell'arena restano quelli del corpo 16×24.
Passi: due disegni originali (D-049) respinti dall'utente come troppo diversi; foglio fornito
  dall'utente tagliato a risoluzione nativa (griglia trovata fotogramma per fotogramma), poi,
  su sua richiesta, ridisegnato pulito perché il cammino non si leggeva (scale diverse tra i
  fotogrammi); anteprime approvate prima dell'integrazione.
File: src/presentation/player_anim.rs (nuovo), src/presentation/{mod,sprites,results}.rs,
  src/world/mod.rs (commento), game-data/player.ron, assets-src/sprites/summoner_sheet.px e
  .palette.txt (nuovi), assets/sprites/summoner_sheet.png (nuovo), assets/manifest.json,
  scripts/{build-assets,validate-assets,preview-scene}.py, ASSET_LICENSES.md,
  docs/{ART_BIBLE,DECISIONS,BACKLOG,ARCHITECTURE,PROGRESS,QA_REPORT}.md,
  docs/art/STYLE_HANDOFF.md, docs/evidence/{art-preview-*,player-*}.png; tolti
  assets-src/sprites/summoner.px e assets/sprites/summoner.png
Comandi eseguiti (risultato):
  ./scripts/check.ps1   OK (fmt, clippy, 159 unit + 82 integrazione, 43 PNG e 2 font)
  build-assets.py / validate-assets.py   OK (foglio 144×304, 32 fotogrammi 36×38)
  gioco vero: cammino con WASD reali nelle 4 direzioni (sinistra specchiata), caduta con
    autoplay ×10 e pannello della sconfitta dopo la caduta
Comportamento verificato: direzione dall'asse dominante, passo alternato leggibile, colpito
  quando riparte l'invulnerabilità, caduta a tempo reale e corpo senza tinta rossa a terra
Rischi noti: somiglianza Nintendo (da rivalutare prima di pubblicare); player più grande dei
  compagni; sporge sulla siepe nord
Prossimo task: prova umana (gate P0), poi P1
```

## Report tranche P0.7 (2026-10-23)

```text
Task: P0.7 - HUD leggibile, test delle formule, bilanciamento base
Owner: orchestrator (integrazione) + gameplay-engineer (HUD) + qa-performance (test, sonde) +
       game-designer (valori, GDD); revisione: code-reviewer (sola lettura)
Stato: DONE (la prova umana resta per il gate P0)
Assunzioni: il "giocatore medio" delle sonde è un riferimento, non un giocatore vero; i valori
  sono tarati su di lui con margine e vanno riverificati da una persona.
File modificati: src/presentation/{abilities,health_bar,boss_pointer}.rs (nuovi),
  src/presentation/{icons,camera,status,hud,menu,boss_bar,results,fonts,mod}.rs,
  src/gameplay/pulse.rs (determinismo), src/gameplay/upgrades.rs, src/gameplay/upgrades/{rules,
  tests}.rs, src/data/schema.rs, src/data/validate/progression.rs (+ tests),
  game-data/{progression,upgrades,stages,enemies}.ron, tests/sim/{formulas,bot}.rs (nuovi),
  tests/sim/{balance,patterns,main}.rs, docs/{GDD,DECISIONS,BACKLOG,ARCHITECTURE,PROGRESS,
  QA_REPORT}.md, docs/evidence/p07-*.png
Comandi eseguiti (risultato):
  ./scripts/check.ps1   OK (fmt, clippy, 153 unit + 82 integrazione, 7 sonde ignorate,
                        43 PNG e 2 font)
  test di determinismo nuovo: FALLISCE con il codice vecchio (HashMap), passa con il nuovo
  sonda giocatore medio, 24 run per ogni passo (vedi GDD, Misure): 22/24 → 24/24 → 23/24
  print_survival_with_upgrades: fermi 0:18–0:19, autoplay 2:39–3:11
  gioco vero: HUD, pausa, draft, boss e vittoria a 960×540 e 1440×810; restart con F5
Comportamento verificato: interfaccia intera ×1,5 a 1440×810 con lo stesso pixel (Tiny5 a
  6 px per pixel del font misurato sullo screenshot); fila delle abilità con ricarica e
  lucchetti; barra HP sotto il player; indicatore del boss fuori schermo; cuori e abilità
  presenti dopo F5 a HP pieni; carta "Nuovo compagno" nel draft del livello 3
Problemi trovati e risolti: run non riproducibili con la stessa seed (HashMap nell'onda,
  D-047); revisione: Tiny5 a 24 px e bordi dispari a mezzo pixel con il canvas ×3, barra di
  ricarica mai piena, indicatore del boss sotto la fascia dell'HUD; primo bot che fuggiva
  dritto e falsava le misure
Rischi noti: bilanciamento tarato su un bot; livello finale quasi sempre al massimo (tutti i
  nemici sconfitti); Pixelify Sans a 16 px leggermente morbido
Prossimo task: prova umana (gate P0), poi P1
```

## Report tranche "font pixel" (2026-10-23)

```text
Task: scaricare Tiny5 e Pixelify Sans (OFL), verificare le accentate, licenze e decisione,
      usarli in HUD, menu, draft e risultati, togliere il vincolo ASCII
Owner: orchestrator (approvazione utente per il download)
Stato: DONE
File: assets/fonts/{tiny5,pixelify-sans}/ (font + OFL.txt), src/presentation/fonts.rs (nuovo),
      src/presentation/{mod,status,boss_bar,hud,draft,menu,results}.rs,
      src/data/{mod,validate}.rs, src/data/validate/{progression,run}/tests.rs,
      game-data/upgrades.ron, Cargo.toml (skrifa solo per i test), scripts/validate-assets.py,
      ASSET_LICENSES.md, docs/{DECISIONS,ART_BIBLE,ARCHITECTURE,BACKLOG,QA_REPORT}.md,
      docs/art/STYLE_HANDOFF.md, docs/evidence/fonts-*.png
Comandi eseguiti (risultato):
  download da github.com/google/fonts (ofl/tiny5, ofl/pixelifysans) + controllo del cmap e
    delle dimensioni nitide con uno script Pillow: tutte le accentate presenti
  ./scripts/check.ps1   OK (fmt, clippy, 145 unit + 79 integrazione, 43 PNG e 2 font)
  gioco vero: menu e draft con tastiera e mouse reali, pausa, sconfitta, run x10 fino a boss e
    vittoria (seed 5, 11:40, livello 37, 1592 sconfitti)
Comportamento verificato: Tiny5 a 16/24 px con pixel 2×2/3×3 esatti (controllato pixel per
  pixel sullo screenshot); "Velocità", "è salva!", "×3", "1–3" disegnati; carte del draft
  con nome su due righe e descrizione su tre dentro il bordo
Problemi trovati e risolti: terza carta del draft che usciva dal bordo con il font nuovo
  (carte 212×252); HP in grigio-azzurro poco leggibili sull'erba (ombra di un pixel del font).
  La "f" di Pixelify Sans ha la traversa lunga: è il disegno del font, non una legatura.
Rischi noti: testi dell'HUD a dimensione fissa anche a finestra grande (le icone scalano);
  Pixelify Sans a 16 px leggermente morbido (antialiasing).
Prossimo task: P0.7
```

## Report tranche "art pass v1 nel gioco" (2026-10-23)

```text
Task: unione del ramo claude/art-style-16bit-adda3a, valori di STYLE_HANDOFF, icone UI
Owner: orchestrator (unione) + pixel-art-director (asset, sessione parallela) +
       gameplay-engineer (presentazione) + game-designer (valori)
Stato: DONE (commit di unione 940574f + commit di integrazione)
File: unione: assets-src/, assets/, scripts/{build-assets,validate-assets,preview-scene}.py,
       src/world/{tiles,generation}.rs, docs/ART_BIBLE.md, docs/art/STYLE_HANDOFF.md,
       ASSET_LICENSES.md; integrazione: src/presentation/{icons,status}.rs (nuovi),
       src/presentation/{hud,boss_bar,draft,menu,camera,mod}.rs, src/diagnostics/overlay.rs,
       src/world/{obstacles,mod}.rs, game-data/{attacks,enemies}.ron,
       assets-src/ui/upgrade_join_ember.px (icona mancante)
Comandi eseguiti (risultato):
  git merge --no-ff --no-commit (5 conflitti risolti) + build-assets/validate-assets: 43 PNG OK
  cargo test dopo l'unione e dopo i valori: 143 unit + 79 integrazione verdi
  ./scripts/check.ps1                       OK (fmt, clippy, test, asset)
  gioco vero: menu, draft con tastiera reale, run x10 fino alla vittoria con boss e preavviso
Comportamento verificato: ritratti ed elementi nel menu; cuori (5 → 6 → 9 con gli HP), barra
  XP, icone della squadra, timer con orologio, livello e uccisioni con teschio; icone sulle carte;
  boss v1 con barra, teschio e cerchio rosso; vittoria a 11:43 (seed 5), stessi livello e
  sconfitti della run headless
Problemi trovati e risolti: icona per la carta "Nuovo compagno: Brasa" assente (aggiunta);
  overlay di debug sopra livello e uccisioni (spostato); nome del boss non coerente con lo
  sprite (Gran Corazzato).
Rischi noti: font ancora FiraMono (proposte OFL da approvare); tinta di colpo anche sull'ombra.
Prossimo task: P0.7
```

## Report tranche P0.6 (2026-10-23)

```text
Task: P0.6 - Timer 10 min, difficoltà, élite e boss telegrafati, menu, vittoria/sconfitta, restart
Owner: gameplay-engineer + game-designer (valori) + qa-performance (test), orchestrator
       (lib.rs); segnaposto grafici in attesa dell'art pass parallelo
Stato: DONE (code review: CHANGES REQUESTED, nessun bloccante; 3 rilievi da correggere e 8
       minori corretti, dettaglio in docs/QA_REPORT.md)
File: src/gameplay/threats.rs, src/gameplay/upgrades/rules.rs, src/presentation/{menu,results,
       telegraphs,boss_bar}.rs (+ menu/tests.rs), src/data/schema/attacks.rs,
       src/data/validate/run.rs (+ tests), src/data/validate/progression/pattern_tests.rs,
       tests/sim/{run,threats}.rs, assets-src/sprites/{elite_enemy,boss_enemy,telegraph}.px
       (nuovi); src/{app,lib}.rs, src/data/{schema,loader,validate}.rs, src/gameplay/{mod,
       spawning,enemies,run,damage,input,companions,upgrades}.rs, src/presentation/{hud,mod}.rs,
       game-data/*.ron (schema v5), tests/sim/{common,balance,main,party,progression}.rs
Comandi eseguiti (risultato):
  ./scripts/check.ps1                       OK: 139 unit + 79 integrazione, 6 sonde ignorate
  run completa headless (seed 5, autoplay invulnerabile)   boss a 10:00, vittoria a 11:49,
                                            combattimento 109 s; restart = run nuova (censimento)
  controprove (5 regole rotte apposta)      falliscono esattamente i test mirati (QA_REPORT)
  sonde: sopravvivenza autoplay 1:37-2:53; bot che schiva 13 min ma senza vittoria
  gioco vero: menu con tastiera reale (2 + Invio = Gocci), run x10 fino al boss, cerchio
                                            rosso, barra HP, schermata di vittoria
Comportamento verificato:
  - a 10:00 ondate ed élite si fermano, il boss nasce una volta con i valori dei dati
  - boss a 0 HP = vittoria; player e boss insieme = sconfitta; il passo è l'ultimo anche x4
  - nemici più forti con i minuti, élite in orario, carica con linea e scatto della distanza
  - colpo del boss: chi resta prende il danno, chi esce o è invulnerabile no; sparisce se il
    boss cade; nessun preavviso dopo il restart; stesso seed stessa run
  - menu: compagno scelto in squadra, la sua carta non esce; F5/Invio senza menu, M al menu
Problemi trovati e risolti: lo scatto dell'élite misurato a contatto (test sistemato);
  `Added<T>` dall'esterno dello schedule non vede le entità nuove; i rilievi del code review.
Rischi noti: bilanciamento da tarare con un giocatore vero; sprite di élite/boss segnaposto;
  la carica può partire anche a contatto; il boss nasce fuori vista senza indicatore.
Prossimo task: P0.7
```

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
