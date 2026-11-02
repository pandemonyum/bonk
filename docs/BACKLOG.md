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
| ~~P1.1~~ **DONE** | Build `wasm32-unknown-unknown` (profilo `wasm-release`) con `wasm-bindgen-cli` 0.2.129 alla versione del lockfile, senza Trunk (D-061) | orchestrator |
| ~~P1.2~~ **DONE** | `web/index.html`: "Clicca per giocare", avanzamento, errore se WebGL2 assente, errori a schermo | orchestrator |
| ~~P1.3~~ **DONE** | Server HTTP locale + browser: menu, run, input da tastiera, sconfitta, risultati, salvataggio, `?autoplay`; console senza errori del gioco | orchestrator |
| ~~P1.4~~ **DONE** | `scripts/build-web.ps1` e `.sh`, output `dist/` | orchestrator |
| ~~P1.5~~ **DONE** | `.wasm` 22,5 MiB (5,9 gzip), `application/wasm`, cache busting con l'impronta del `.wasm`, asset e dati via HTTP; `docs/RELEASE.md` | orchestrator |
| ~~P1.6~~ **DONE** | CI GitHub Actions: gate di `scripts/check.sh` e build web con `dist/` come artefatto (D-064, `docs/RELEASE.md`) | web-release |

## P2 / P3

Come da handoff §6 (art pass, animazioni, audio; profiling 50/200/500 nemici, bilanciamento,
opzioni, browser, release candidate).

## Grafica del player (2026-10-28, su richiesta dell'utente)

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~ART-P2~~ **DONE** | Nemici e boss animati (movimento, sconfitta, carica/scatto, preparazione/colpo), in proporzione al player (D-051) | pixel-art-director (fogli) + gameplay-engineer (`enemy_anim`, dati) + game-designer (hurtbox, contatto, separazione) | fogli validati e approvati dall'utente; test del contratto sui PNG; logica testata; nel gioco vero orda, élite e boss animati; bilanciamento rimisurato; gate verde |
| ~~ART-P3~~ **DONE** | Mappa nello stesso stile: tileset (stessi 42 indici e ruoli), rocce (20×16), alberi (40×48); tolti gli sprite fissi dei nemici rimasti inutilizzati | pixel-art-director | anteprime e screenshot del gioco vero approvati dall'utente; generatore della mappa e impronte invariati; gate verde |
| ~~ART-P4~~ **DONE** | Compagni animati dal foglio dell'utente (D-052): Fiammetto, Ondino, Foglin; fermo, cammino in 3 direzioni, attacco | pixel-art-director (fogli) + gameplay-engineer (`companion_anim`, menu) | anteprime approvate dall'utente; test del contratto sui PNG; menu e run nel gioco vero; gate verde |
| ~~P-TUONNY~~ **DONE** | Quarto compagno Tuonny (elettro): fulmine a catena con scossa (D-053), carta "Nuovo compagno", menu a quattro carte | game-designer + gameplay-engineer + pixel-art-director (scintilla) | test del pattern e della formula; bilanciamento rimisurato; menu e fulmini nel gioco vero; gate verde |
| ART-P5 | Tartaruga di muschio e fuoco fatuo del foglio dell'utente come compagni futuri (servono attacchi propri) | game-designer + gameplay-engineer | da proporre all'utente |
| ~~ART-P1~~ **DONE** | Player animato: fermo e cammino in 4 direzioni, colpito, caduta; sprite identico a quello fornito dall'utente (D-050) | pixel-art-director (foglio, pipeline) + gameplay-engineer (`player_anim`, risultati dopo la caduta) | foglio 4×8 validato; test del contratto sul PNG; anteprime approvate dall'utente prima dell'integrazione; cammino nelle 4 direzioni e caduta nel gioco vero; gate verde |

## Richieste dell'utente (2026-10-28, sera)

| ID | Task | Owner | Stato |
|---|---|---|---|
| ~~U-RECAP~~ **DONE** | Riepilogo di fine run: danni, quota, sconfitti, cure, esperienza per compagno (D-054) | orchestrator + gameplay-engineer | test dei crediti con i quattro pattern; gioco vero |
| ~~U-SQUAD~~ **DONE** | Un compagno all'inizio, gli altri ai livelli 3 e 9, tre posti (D-055) | orchestrator + game-designer | test del draft e della run fino alla squadra piena |
| ~~U-HORDE~~ **DONE** | Run da 15 minuti, orde a 5:00 e 10:00 con annuncio (D-056) | orchestrator + game-designer | test dell'orda; sonda 84% |
| ~~U-CHEST~~ **DONE** | Monete, forzieri lontani con freccia, reliquie (D-057) | orchestrator + game-designer + pixel-art-director | test di monete, forzieri e reliquie; gioco vero |
| ~~U-TALENT~~ **DONE** | Talenti persistenti: profilo, esperienza, alberi, sei meccaniche, schermata nel menu (D-058) | orchestrator + game-designer + pixel-art-director | test di profilo, punti e meccaniche; sonda al massimo |
| ~~U-TREE~~ **DONE** | Alberi grandi (3 rami, gradi, unione, finale) e schermata come il riferimento dell'utente (D-059) | game-designer + orchestrator + pixel-art-director | test di gradi, requisiti, effetti di Supporto, schermata; gioco vero |
| ~~U-SMALL~~ **DONE** | Compagni più piccoli (28×24, ~2/3 del player), approvati dall'utente (D-060) | pixel-art-director + orchestrator | gate verde; gioco vero |
| ~~P1-SAVE~~ **DONE** | Profilo nel `localStorage` sul web (D-061) | orchestrator | provato nel browser |
| ~~P1-FILL~~ **DONE** | Più mappa invece delle bande nere, fino a 640×360 pixel di mondo (D-062) | orchestrator | finestre 960×540, 1280×720, 1366×768 e browser; sonda 27/32 come prima |
| U-PLAY | Prova umana: forzieri e orde si sentono? Squadra senza Ondino, talenti al massimo | utente | — |

## Selezione del compagno iniziale (2026-10-30, D-066)

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~U-SELECT~~ **DONE** | Nuova Partita → selezione del compagno iniziale → Inizia Partita; griglia dai dati con i bloccati oscurati e il lucchetto, niente al centro, dettagli dai dati, sblocchi persistenti (solo Fiammetto su un profilo nuovo), avvio rifiutato per un bloccato | gameplay-engineer (`src/presentation/`, `src/app.rs`, `src/profile.rs`, `src/data/`, `src/options.rs`, campi di testo e `unlock` in `game-data/companions.ron` e `attacks.ron`, assegnati dall'orchestratore) + code-reviewer + orchestrator (decisione, backlog, progress, build web) | test: profilo nuovo, selezione iniziale, bloccato non avviabile, avvio con Fiammetto, Indietro, persistenza, profilo vecchio, nessuna run che sblocca, schermata aperta e chiusa più volte senza duplicati; `./scripts/check.ps1` e clippy wasm verdi; build web; screenshot a 1280×720, 1920×1080 e 1024×768 |

## Home animata (2026-10-30, D-067)

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~HOME-AMBIENT~~ **DONE** | Home animata dai pixel dell'illustrazione: portale (P1), cascate e fiume (P2), fiamma di Fiammetto e lucciole (P3); opzione Animazioni; spegnimento per effetto con `--ambient` | orchestrator (codice in `src/presentation/home_ambient/`, righe in `home.rs`, `mod.rs`, `options_menu.rs`, `profile.rs`, `options.rs`, dopo U-SELECT) + pixel-art-director (`scripts/art_home_ambient.py`, maschere, manifest, validatore, ART_BIBLE, ASSET_LICENSES, richiesta) | allineamento al pixel a 1024×768, 1280×720, 1366×768 e 1920×1080; loop senza salti e nessuna entità duplicata (test); fondo fermo senza maschere o con l'interruttore spento; WebGL2 nel browser; `./scripts/check.ps1` verde; screenshot di due istanti e GIF |
| ART-HOME-002 | Nuvole che scorrono: cielo pulito e nuvole separate da ChatGPT (`docs/art/requests/HOME_AMBIENT_ART_REQUEST.md`) | pixel-art-director + utente | proposta: serve il via dell'utente |

## Arte con ChatGPT, handoff manuale (2026-10-30, D-065)

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~ART-BRIDGE~~ **DONE** | Workflow manuale con ChatGPT: decisione, procedura, template delle richieste, `.art-inbox/` ignorata, istruzioni del pixel-art-director e skill di Claude Code; dry run con ART-HOME-001 | orchestrator (D-065, `AGENTS.md`, `CLAUDE.md`, `.claude/`, `.gitignore`, backlog e progress) + pixel-art-director (`docs/art/`, `docs/ART_BIBLE.md`, `ASSET_LICENSES.md`, `scripts/art_illustration.py`) | richiesta ART-HOME-001 con prompt autosufficiente, reference da allegare e percorso di ritorno; `.art-inbox/` ignorata e `assets-src/` no; build e validazione degli asset passano senza cambiare `assets/`; nessuna immagine generata dal progetto |
| ~~ART-HOME-001~~ **DONE** | Illustrazione della home nativa 16:9 senza logo, menu né testo, dal concept approvato di D-063 | pixel-art-director (richiesta, controllo, promozione) + utente (generazione e approvazione in ChatGPT) + gameplay-engineer (`FOCUS` della home, se serve) | PNG approvato dall'utente in `.art-inbox/ART-HOME-001/`; promosso come `assets-src/illustrations/home_background.*` con licenza e provenienza; build e validazione; home nel gioco vero a 1280×720, 1920×1080 e 1024×768 senza player o compagni tagliati né coperti dalla UI. Fatto il 2026-10-30: v1 approvata dall'utente, 1672×941, `FOCUS` invariato |

## Mappe a più biomi (2026-10-31, D-068)

Richiesta dell'utente: sei ambientazioni (Prateria Verde, Bosco Ombroso, Canyon Arido, Cime
Glaciali, Vulcano Infernale, Rovine Arcane), sempre top-down 2D come il gameplay attuale. Prima
la struttura e il Bosco Ombroso; mappe selezionabili, effetti ambientali, pericoli e boss
dedicati solo dopo che il bosco è verificato.

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~MAP-BIOMES~~ **DONE** | Generazione del mondo configurabile per stage: biomi in `stages.ron` con tre livelli (terreno a tileset con bordi, decorazioni calpestabili cotte nel terreno, ostacoli con collisione e sprite dai dati); un solo generatore e un solo renderer; stage scelto con `--stage` indipendente dal compagno; prateria identica a prima | orchestrator (codice in `src/world/`, `src/data/`, `src/presentation/sprites.rs`, `src/app.rs`, `src/options.rs`, `src/lib.rs`, `src/gameplay/{collision,spawning}.rs` (solo test), `tests/sim/{world,run}.rs`; schema e valori in `game-data/*.ron`, assegnati dall'orchestratore) + code-reviewer | prateria identica al pixel e agli ostacoli di prima (test con impronte calcolate prima della modifica); stessa seed = stessa mappa per ogni bioma; ogni bioma carica i suoi tileset e sprite (test sul manifest); regole degli ostacoli (zona libera, varchi, raggiungibilità, margini) per ogni stage; validazione dei biomi con errori leggibili; `./scripts/check.ps1` verde |
| ~~MAP-FOREST~~ **DONE** (asset provvisori) | Stage Bosco Ombroso: terreno verde scuro, radure, alberi, ceppi, tronchi, funghi, piccole rovine; asset provvisori procedurali nella pipeline esistente | pixel-art-director (tileset, decorazioni, sprite degli ostacoli, manifest, validatore, ART_BIBLE) + orchestrator (stage in `stages.ron`) | asset validati (dimensioni, alpha 0/255, palette, allineamento a 16 px); run vera con `--stage forest`: nemici, compagno, XP, collisioni; screenshot del gioco vero senza overlay; sonda di bilanciamento sul bosco |
| ART-FOREST-001 (REQUESTED, parte A; B e C dopo A) | Richiesta ChatGPT per tileset e oggetti definitivi del bosco (D-065) | pixel-art-director + utente | richiesta in `docs/art/requests/` con dimensioni native, bordi ripetibili, oggetti separati con trasparenza; nessuna immagine generata dal progetto |

## Sistema audio (2026-10-31, D-069, D-070)

Richiesta dell'utente (P2.4 e P3.3 dell'handoff): musica, effetti e ambienti con volumi
indipendenti, separati dalla simulazione. Progetto approvato dall'utente:
`docs/design/AUDIO_SYSTEM.md`. Ogni tranche: `./scripts/check.ps1` verde, clippy
`wasm32-unknown-unknown`, report in `docs/PROGRESS.md`, commit locale.

| ID | Task | Owner | Criteri di done |
|---|---|---|---|
| ~~AUDIO-0~~ **DONE** | Progetto approvato; decisioni D-069 (dipendenze) e D-070 (origine degli asset, proprietario, impostazioni predefinite); backlog; ownership in `AGENTS.md` | orchestrator | decisioni registrate; nessun codice |
| ~~AUDIO-1~~ **DONE** | Fondamenta senza suoni: feature `audio` + `wav` di Bevy; `src/audio/` con impostazioni (tacche → guadagno), catalogo incorporato e validato, coda dei cue con `AudioManager`, mixer puro (limiti, cooldown, priorità, furto, coalescenza, varianti, margine); `AudioCorePlugin` (va anche con `MinimalPlugins`) e `GameAudioPlugin` solo in `client_app`; `--mute`, `--no-audio` (`?mute` sul web) | orchestrator (`Cargo.toml`, `Cargo.lock`, `src/lib.rs`, D-069) + gameplay-engineer (`src/audio/`, `assets/audio/catalog.ron` iniziale senza file, `src/options.rs`, `src/web.rs`, assegnati dall'orchestratore) + qa-performance (`tests/sim/audio.rs`) + code-reviewer | test del mixer: limiti, cooldown, priorità e furto, coalescenza, varianti senza ripetizione, stessa sequenza di eventi a 30/60/144 FPS entro il ±10%; test di separazione: headless senza risorse audio, simulazione identica con e senza `AudioCorePlugin`, moduli della simulazione senza import dell'audio; `.wasm` prima e dopo in D-069; test e sonde esistenti invariati |
| ~~AUDIO-2~~ **DONE** | Riproduzione e impostazioni: voci, volume per voce, ripulitura, nessun dispositivo; `AudioPrefs` nel profilo, cursori nelle Opzioni, fuoco della finestra; protezione dell'`AudioContext` nella pagina; 1–2 effetti di prova sintetizzati (`ui.tick`, `ui.toggle`, usati nelle Opzioni) | gameplay-engineer (`src/audio/`, su assegnazione `src/profile.rs`, `src/presentation/options_menu.rs`, `docs/ARCHITECTURE.md`) + pixel-art-director (`scripts/audio_sfx.py`, `assets-src/audio/`, `assets/audio/` con i valori di `catalog.ron`, manifest, validatore, `ASSET_LICENSES.md`, `docs/audio/AUDIO_BIBLE.md`) + web-release (`web/index.html`, `docs/RELEASE.md`) + qa-performance (`tests/sim/audio*`) + code-reviewer; orchestrator `.gitignore` (`.audio-inbox/`), prova nel browser | profilo vecchio leggibile; il volume cambia le voci vive; nessun panic senza dispositivo; con `--no-audio` nessun `load` di `AudioSource` (senza `AudioPlugin` andrebbe in panic: `bevy_asset-0.20.0/src/server/info.rs:855`); `Mixer::playing` alla nascita del sink, `finished` quando si svuota; `--mute` mai salvato nel profilo (e riapplicato se il profilo sostituisce `AudioSettings`); nel browser l'audio parte dopo il clic e si sospende con la scheda nascosta; `tests/sim/audio.rs` (qa-performance): `AudioPlayer` ammesso solo in `src/audio/playback.rs`, nella simulazione vietati anche i tipi audio della prelude di Bevy (`AudioSource`, `AudioSink`, `PlaybackSettings`, `GlobalVolume`, `SpatialListener`), una terza run di prova che dimostra che la trasparenza vede una scrittura, file diviso sotto ~400 righe |
| ~~AUDIO-REQ~~ **DONE** (richieste `REQUESTED`, aspettano i file dell'utente) | Procedura dell'handoff manuale audio (D-070) e richieste autosufficienti per musica (home, run, orda, boss, stinger) e ambienti della home (portale, cascate) | pixel-art-director (`docs/audio/`) + orchestrator (`AGENTS.md`, `CLAUDE.md`, backlog, progress) | procedura con ID, stati, inbox `.audio-inbox/`, provenienza e licenze ammesse; una richiesta per consegna con BPM, tonalità, lunghezza esatta del loop in campioni, regole del loop, formato, loudness, controlli all'arrivo; tonalità e tempi coerenti tra le tracce; budget del peso; nessun file audio prodotto dal progetto |
| ~~AUDIO-3~~ **DONE** (2026-11-02; `MUSIC-HOME-001` v1 integrata e ascoltata dall'utente, D-071); la prova nel browser è rimandata alla prossima build web | Musica della home: promozione di `home.ogg`; sezione `music` del catalogo (tracce, dissolvenze, fuoco della finestra); funzione pura della traccia voluta per ogni stato (§ 3.7), con silenzio dove manca la traccia; direttore con due deck e dissolvenza equal-power che aspetta il sink; test di decodifica esteso alla musica | pixel-art-director (promozione: `assets/audio/music/`, manifest, `ASSET_LICENSES.md`, nota in `assets-src/audio/music/`, richieste in `docs/audio/`) → gameplay-engineer (`src/audio/`, sezione `music` di `assets/audio/catalog.ron` su assegnazione, `docs/ARCHITECTURE.md`) + web-release (`web/index.html`) + code-reviewer | `sha256` di `home.ogg` uguale all'Ogg approvato; funzione della traccia testata in ogni stato (menu → home; partita, orda, boss, vittoria, sconfitta → silenzio finché manca la traccia; `Loading` non cambia nulla, F5 non interrompe); nessun loop duplicato dopo dieci entrate e uscite dal menu (test); al massimo 2 deck vivi; decodifica: 2 canali, 44 100 Hz, campioni 3 175 200 ±1, picco di campione ≤ −1 dBFS, giro del loop in Bevy senza vuoto né gradino; dissolvenze nel catalogo, compresa quella del fuoco della finestra (oggi `FOCUS_FADE_S` in `src/audio/focus.rs`); traccia e volumi dei deck nel log; nella pagina web `follow()` anche alla creazione del contesto (oggi aspetta il primo `statechange`); gioco vero: home → selezione → opzioni → partita (dissolvenza a silenzio in 1,5 s) → menu (ritorno in 1,5 s); ascolto dell'utente nel gioco |
| AUDIO-3b (servono `MUSIC-RUN-001` e le altre tracce della partita) | Musica della partita: tracce di run, orda, boss e risultati, ducking in pausa e nel draft, stinger, pausa e ripresa della traccia della run, isteresi dell'orda | gameplay-engineer + pixel-art-director | gioco vero home → run → orda → boss → risultati → menu senza buchi né sovrapposizioni; stinger con attacco e coda nel test di decodifica |
| ~~AUDIO-4~~ **ANNULLATO** (utente, 2026-11-02, D-071) | Ambienti della home: portale e cascate. L'utente non li vuole; `AMB-HOME-001` `REJECTED` | — | — |
| AUDIO-5 | Effetti di gioco dagli eventi (osservatori e contatori, nessuna modifica a `src/gameplay/`); taratura dei limiti nelle orde | gameplay-engineer + pixel-art-director + qa-performance + code-reviewer | stress nell'orda a 5:00: voci ≤ limiti, nessun `Critical` perso; costo per passo invariato; FPS nativo e web annotati; validazione: le `Critical` solo `Single` (o con un boost basso), perché saltano il margine; test della regola "al massimo 65 536 cue" |
| AUDIO-6 | Suoni dell'interfaccia: menu, selezione, draft, forzieri, talenti, pausa | gameplay-engineer | ogni azione con il suo suono, nessun doppio per tasto e clic |
| AUDIO-7 | Asset definitivi, loudness, licenze, crediti | pixel-art-director + utente | ogni file con licenza e approvazione; validatore e test di decodifica verdi; picchi ≤ −1 dBFS |
| AUDIO-8 | Prova su Windows e web (Chrome, Edge, Firefox) | qa-performance + web-release | checklist con esiti veri in `docs/QA_REPORT.md`; limiti noti in `docs/RELEASE.md` |

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
- Legatura "fi" di Pixelify Sans: "finestra" si leggeva "Anestra" (AUDIO-2). Le voci delle liste
  ora usano `fonts::ui_unligated` (`liga` a 0); gli altri testi con "fi", "fl" o "ff" (draft,
  risultati, talenti) hanno ancora la legatura: decidere per tutta l'interfaccia (D-044).
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
