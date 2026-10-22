# QA REPORT

## 2026-10-22 — P0.4 (gemme XP, livelli, draft, upgrade a caldo)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 158 test (105 unitari + 53 integrazione) + 4 sonde ignorate, 10 PNG, 7 sprite citati dai dati |
| Sonda livelli (5 seed, prima carta automatica) | primo level-up con l'autoplay 0:30–0:41; da fermi morte a 0:13 al livello 1 |
| Sonda livelli con player invulnerabile (10 min) | livello 5 a ~2:20, livello 10 a 4:39–5:56, finale 14–25 |
| Sonda sopravvivenza con upgrade (autoplay) | 1:08–1:24, livello 2–3 (senza upgrade 1:06–1:16) |

Controprove (regola rotta apposta → test che falliscono, poi ripristino e verde):

| Regola rotta | Test che falliscono |
|---|---|
| il passo del livello non ferma la simulazione | `the_level_up_step_is_the_last_one_even_with_accelerated_time` |
| `+1 proiettile` ignorato nel ricalcolo | `the_multishot_upgrade_fires_one_more_projectile_per_attack` |
| nessun limite alle gemme a terra | `beyond_the_cap_new_xp_merges_into_gems_on_the_field` |
| `refresh_modifiers` anche in `Loading` (panic del gioco vero) | `the_app_waits_for_the_data_without_panicking_then_starts_the_run` |
| validazione carta per carta / somme che si compensano | `cards_with_the_same_effect_add_up`, `negative_cards_do_not_hide_the_upper_limits` |
| un solo messaggio letto per fotogramma | `extra_choices_in_the_same_frame_are_dropped` |
| carte estratte alla richiesta del draft | `a_pause_on_the_level_up_frame_does_not_change_the_cards` |
| attrazione che non dura | `an_attracted_gem_keeps_chasing_a_player_who_leaves_the_radius` |

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| Player fermo, `--god --time-scale 3`, input reale | draft al livello 2, tasto 2 sceglie la seconda carta, livello 3 dopo | log `upgrade scelto: ember_speed` |
| Mouse reale sulle carte | carta evidenziata al passaggio, clic sulla prima e sulla terza scelgono quelle | `docs/evidence/p04-draft.png` |
| `--autoplay --god --time-scale 4 --overlay` | livello 4 a 2:00, barra XP e conteggio gemme, draft aperto 0,6 s poi scelta automatica | `docs/evidence/p04-hud-xp-autoplay.png` |

Problemi trovati provando il gioco vero (invisibili in headless):
- panic all'avvio: `refresh_modifiers` chiedeva la `GameConfig` in `Loading`; corretto e coperto
  da un test che simula l'attesa asincrona dei dati;
- "più" disegnato come "pi" + quadratino: il font provvisorio non ha le accentate; testi
  riformulati e validazione ASCII (D-033).

### Code review P0.4

Verdetto CHANGES REQUESTED, nessun bloccante:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Validazione carta per carta: due carte di cadenza valide da sole portavano l'intervallo a 0 (60 raffiche/s); positive e negative si compensavano | somme separate per segno, fattore minimo > 0 e limiti fisici sul massimo; HP massimi cumulati (D-034) | 2 test nuovi, falliscono con la vecchia logica |
| 2 | Overflow di `pierce`/`projectiles` (`pierce` "infinito" + carta = panic in dev); raffiche illimitate | somme saturanti; massimo 32 proiettili per raffica | test di validazione |
| 3 | `draft_size` oltre 3 senza tasto né spazio a schermo | validato da 1 a 3 | test di validazione |
| 4 | Due scelte nello stesso fotogramma: la seconda finiva nel draft successivo | vale la prima, le altre scartate | test nuovo con controprova |
| 5 | XP dei nemici senza limite (overflow, ciclo da 1e9 livelli) | massimo 1000 XP per nemico, somme saturanti | test di validazione |
| 6 | Test della gemma attirata vacuo | il player esce dal raggio dopo l'aggancio | controprova |
| 7 | Valori di gioco nei test (slot 1, 2 XP, 60 px, livello 3 dopo 90 s) | slot, distanze e XP ricavati dai dati; livelli iniziali dati da una gemma | gate verde |
| 8 | `tests/sim/progression.rs` oltre 400 righe; gemma assente da `ASSET_LICENSES.md` | diviso in `progression.rs` e `upgrades.rs`; voce aggiunta; `Modifiers` in `upgrades/modifiers.rs` | tutti i file sotto 400 righe |
| 9 | `MaxHp(inf)` accettato; compagno più lento del player al massimo; prerequisito con peso 0; a capo vietato | valori finiti; `follow.max_speed` ≥ velocità massima; requisiti mai proposti segnalati; a capo ammesso | test di validazione |
| 10 | Test mancanti | morte e livello nello stesso passo, restart dal draft, draft vuoto, stesso seed con livelli e carte | 4 test nuovi + confronto esteso |
| P1 | Pausa nel fotogramma del livello: carte estratte di nuovo alla ripresa | estrazione in `OnEnter(LevelUp)` | test nuovo con controprova |
| P2 | Ricarica dei dati con il draft aperto: indici vecchi sulla lista nuova | `get` invece dell'indice diretto | codice |
| P3 | `Modifiers` della run precedente nei primi passi | azzerati in `reset_upgrades` | codice |
| P4 | Compagni aggiunti a metà run senza modificatori | annotato nella P0.5 del backlog | — |

## 2026-10-22 — P0.3 (spawn director, danni al player, griglia spaziale)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 122 test (85 unitari + 34 integrazione) + 3 sonde ignorate, 9 PNG, 6 sprite citati dai dati |
| Sonda costo per passo (build dev, headless) | 0,45 ms (50 nemici) / 0,49 ms (200) / 0,62 ms (500) |
| Sonda sopravvivenza senza upgrade (5 seed) | fermi 0:13; autoplay 1:06–1:16 |

Controprove: un director che ignora la vista fa fallire i test di spawn ("nato in vista");
la vecchia logica della sconfitta fa fallire `a_pause_on_the_killing_frame_does_not_cancel_the_defeat`
e `no_simulation_step_runs_after_the_killing_one`.

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| 3 min di gioco a ×4 (`--god`) | 135 nemici dei 3 archetipi, 59 FPS / 16,8 ms | `docs/evidence/p03-orda-135-nemici.png` |
| ~7 min di gioco a ×8 (`--god`) | 220 nemici (tetto), 59 FPS / 16,9 ms, 8 passi di simulazione per fotogramma | `docs/evidence/p03-220-nemici-59fps.png` |
| Run vera fino alla sconfitta | 01:06, schermata con tempo e sconfitti, mondo fermo | `docs/evidence/p03-sconfitta.png` |
| Invio da tastiera reale da sconfitti | run nuova, statistiche azzerate | `docs/evidence/p03-riavvio-con-invio.png` |

### Code review P0.3

Verdetto CHANGES REQUESTED, 7 rilievi, tutti corretti:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Una pausa (Esc o perdita di focus) nel fotogramma del colpo mortale cancellava la sconfitta: player a 0 HP immortale; col tempo accelerato la simulazione continuava dopo la morte | `simulation_running` (Playing e player vivo) per tutti i sistemi del passo fisso; `declare_defeat` in `PostUpdate` dagli HP | 2 test nuovi, falliscono con la vecchia logica |
| 2 | Da sconfitti gli sprite tremavano di 1 px (interpolazione con tempo che scorre) | tempo virtuale fermo anche in `Defeated` | asserzione nel test di sconfitta e di restart |
| 3 | Due ondate dello stesso nemico superavano il limite nello stesso passo | conteggio dei nati nel passo condiviso tra ondate | test `two_waves_of_the_same_enemy_share_its_cap` |
| 4 | L'ondata iniziale ignorava limite e inizio della prima ondata | validazione: `initial_burst` ≤ `waves[0].max_alive` e `waves[0].start_s` = 0 | test di validazione |
| 5 | Test legati ai valori dei dati (id, indici, HP fissi) | id, indici e numeri ricavati da `GameConfig`; nemici dei test di danno creati dal test | gate verde con gli stessi dati; non dipendono più dai numeri |
| 6 | Anello di spawn tutto fuori dall'arena accettato (run vuota senza errori) | validazione dell'anello rispetto al raggio utile dell'arena | test di validazione |
| 7 | `combat.rs` oltre 400 righe | attacchi spostati in `gameplay/attacks.rs`; test di validazione in `data/validate/tests.rs` | tutti i file sotto 400 righe |

## 2026-10-22 — P0.2, P0.1, P0.0 e riduzione dello spazio su disco

Stessa macchina della S0. Build `dev` con il nuovo profilo (dipendenze senza simboli di debug).

### Automatici

| Controllo | Comando | Esito |
|---|---|---|
| Gate completo | `./scripts/check.ps1` | OK: fmt, clippy `-D warnings`, test, asset |
| Unit test | `cargo test --lib` | 58/58 OK |
| Integrazione headless | `cargo test --test sim` | 21/21 OK, 1 ignorato (sonda di bilanciamento) |
| Asset + sprite citati dai dati | `python scripts/validate-assets.py` | OK, 7 PNG, 4 sprite citati in `game-data/` |
| Sonda bilanciamento | `cargo test --test sim balance -- --ignored --nocapture` | 54–55 colpi/min, 100% a segno, 14–18 sconfitti/min |

Controprove (il test fallisce quando il comportamento si rompe): ripresa dalla pausa senza
`resume_time`; sprite mancante citato in `enemies.ron`; dati con velocità negativa e
riferimento inesistente (schermata d'errore con entrambi i messaggi).

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| Compagno che attacca (P0.2) | proiettile in partenza dal compagno; tinta di colpo visibile su una gelatina | `docs/evidence/p02-compagno-proiettile.png`, `p00-dati-caricati-e-colpo.png` |
| Ostacoli (P0.1) | rocce e alberi sparsi, chioma che copre per profondità, orde che li aggirano; 58–62 FPS con ~1150 entità | `docs/evidence/p01-ostacoli-alberi.png`, `p01-ostacoli-radura.png` |
| Dati da `game-data/` (P0.0) | log "dati di gioco caricati da game-data/" e run avviata | `docs/evidence/p00-dati-caricati-e-colpo.png` |
| Dati non validi | schermata con "enemies.ron: blob.speed deve essere >= 0, trovato -3" e "stages.ron: meadow.enemy = \"dragon\": non esiste in enemies.ron" | `docs/evidence/p00-errori-dati.png` |
| Ricarica a caldo | `stages.ron` modificato a gioco avviato (10 → 20 nemici): log "dati di gioco ricaricati", la run riparte con 20 nemici e statistiche azzerate | `docs/evidence/p00-ricarica-a-caldo.png` |

### Spazio su disco (D-027)

| Momento | `target/` | Disco C: libero |
|---|---|---|
| Prima | 21,7 GB (11 GB di copie non più usate, .pdb da ~0,75 GB per eseguibile) | 6,5 GB |
| Dopo `cargo clean` + nuova configurazione: solo `cargo build` | 2,62 GB (build completa 14 min invece di 23) | |
| + test compilati (un solo eseguibile d'integrazione) | 2,85 GB | |
| + clippy | 3,31 GB | 25,6 GB |

Rimossa anche la toolchain `stable` duplicata (1,3 GB); default rustup = 1.99.1.

### Code review P0.2/P0.1/P0.0 (ruolo code-reviewer, lettura indipendente)

Verdetto iniziale CHANGES REQUESTED, 7 rilievi, tutti corretti. Dopo le correzioni: 92 test
(70 unitari + 22 d'integrazione, 1 sonda ignorata), gate verde, gioco avviato con i dati nuovi.

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Lungo la siepe nord i colpi partivano fuori dal rettangolo dei piedi e sparivano al primo passo: il compagno non colpiva | limite dei proiettili = intera arena (`world::inside_arena`) | test `shots_fired_along_the_north_hedge_still_hit`; col vecchio limite il nemico resta a 30 HP (fallisce) |
| 2 | Ricarica a caldo: dopo un primo caricamento fallito la versione corretta arriva come `Added`, ignorata; un file illeggibile durante la run non mostrava errori | conta `Added`/`Modified`/`LoadedWithDependencies`; legge `AssetLoadFailedEvent` | 5 test del loader con AssetServer vero; reintroducendo il bug `fixing_an_unreadable_file_after_a_failed_first_load_starts_the_game` fallisce |
| 3 | Test della griglia spaziale vacuo (sonde senza ostacoli vicini) | sonde su ostacoli veri ai bordi e al centro, con asserzione "non vuoto" | test aggiornato |
| 4 | `pierce` enorme in overflow; corpi veloci attraversano i tronchi | `saturating_add`; validazione velocità ≤ (4 + raggio) px per passo | test di validazione (600 px/s rifiutati, 540 accettati) |
| 5 | Commento falso: il clamp poteva riportare un corpo dentro una roccia della riga più a nord | margine di 2 tile dalla siepe | test `obstacle_exclusion_zones_stay_inside_the_walkable_area` su 10 seed |
| 6 | Valori di bilanciamento nel codice (`LIFETIME_MARGIN`, densità ostacoli, zona libera, quota alberi) | `projectile_lifetime_s` in `attacks.ron`, `obstacles` in `stages.ron`, validati | test di validazione e `parameters_change_the_field` |
| 7 | Cadenza arrotondata per eccesso (0,935 s → 0,95 s) | carica che conserva il resto (`attack_tick`) | test: 66 passi esatti per 1,1 s, media 56,1 passi per 0,935 s, nessuna raffica dopo attesa |

### Limiti aperti

- Compilazione WASM non riverificata dopo la P0.0 (loader dei dati, watcher solo nativo):
  la verifica completa è il primo passo della P1.
- Ricarica a caldo provata sui dati; sugli sprite (`assets/`) è attiva ma non provata.
- I rimpiazzi dei nemici possono comparire in vista vicino ai bordi dell'arena (P0.3).

## 2026-10-22 — S0

**Macchina:** Windows 11 Enterprise 26200, Intel Core Ultra 5 238V, 31,5 GiB, Intel Arc 130V
integrata (Vulkan). **Build:** profilo `dev` (codice opt-level 1, dipendenze opt-level 3).

### Automatici

| Controllo | Comando | Esito |
|---|---|---|
| Formattazione | `cargo fmt --all -- --check` | OK |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | OK, 0 warning |
| Unit test | `cargo test --lib` | 31/31 OK |
| Simulazione headless | `cargo test --test headless_sim` | 7/7 OK (~1 s) |
| Asset | `python scripts/validate-assets.py` | OK, 3 PNG, 0 errori |
| Griglia pixel | `python scripts/check-pixel-grid.py` | OK 100% a 1366×768, 1365×767, 1101×701 @1.25 e @1.5 |
| Compilazione WASM | `cargo check --lib --target wasm32-unknown-unknown --locked` | OK (eseguito dal code-reviewer, 7 min 27 s; build web completa: P1) |

I test headless coprono: spawn (1 player + 10 nemici); velocità 75 px/s su 60 passi; player
sempre dentro l'arena dopo 60 s di spinta in diagonale; inseguimento (distanza media −75% in
8 s) con nessun nemico dentro il player e coppie di nemici sempre oltre 7 px (metà del raggio
di separazione); nemici mai dentro il player quando è schiacciato contro un muro; stesso seed =
stessa simulazione; pausa che congela la simulazione e ripresa che la fa ripartire.

### Prove sul gioco reale

| Prova | Come | Osservato | Prova |
|---|---|---|---|
| Avvio e rendering | `cargo run -- --autoplay --overlay --screenshot ...` | finestra 960×540, arena, 10 nemici che inseguono, pixel netti, exit 0 | `docs/evidence/s0-autoplay-960x540.png` |
| Tastiera reale | `keybd_event` di Windows: D 1 s, W 1 s, Esc | player (76,75) vs atteso (75,75); pausa con banner, fase `Paused` | `docs/evidence/s0-keyboard-pausa.png` |
| Resize / letterbox | `--window 1365x767` + `check-pixel-grid.py` | scala 2×, bande nere, 100% texel uniformi | `docs/evidence/s0-letterbox-1365x767.png` |
| Controprova mezzo pixel | stessa finestra con correzione disattivata | 96,1% → il controllo fallisce (non vacuo) | log nel report S0 |
| DPI 125% / 150% | `--window 1101x701 --scale-factor 1.25` e `1.5` | testo UI ingrandito (override attivo), canvas 2×, 100% texel uniformi | `docs/evidence/s0-dpi150-1101x701.png` |

Nota: a 960×540 senza bande nere il promemoria comandi copre il canvas e `check-pixel-grid`
misura ~99% anche con scala corretta: il controllo va fatto con finestre letterbox.

### Prestazioni (indicative, NON benchmark)

Overlay F3, build dev, 10 nemici, vsync attivo: 64 FPS / 15,7 ms a 15 s (1366×768), 60 FPS /
16,5 ms a 9 s (960×540). Nei primi 6 s la media è più bassa (49 FPS) per l'avvio. Benchmark
50/200/500 nemici in release: P3.1.

### Code review S0 (ruolo code-reviewer, lettura indipendente)

Verdetto iniziale CHANGES REQUESTED, 7 rilievi, tutti corretti nella stessa tranche:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | `check.ps1` poteva dire OK senza eseguire un passo (comando mancante) | exit code azzerato per passo, `try/catch`, fallback `py` | passo con comando inesistente → exit 1 |
| 2 | Nessun test sulla ripresa dalla pausa | asserzioni di ripresa nel test di pausa | rimuovendo `resume_time` il test fallisce |
| 3 | Ordine indefinito tra `move_player` e `chase_player` | nuovo `SimSet::Player` prima di `Movement` | test di determinismo verdi |
| 4 | Nemici dentro il player contro un muro | respinta dal player come ultima regola | nuovo test `enemies_never_enter_the_player_against_a_wall` |
| 5 | Limiti arena pensati per sprite centrati | `ArenaBounds` con `min`/`max` sui piedi | test `whole_sprite_stays_inside_the_hedge` |
| 6 | Attesa screenshot calcolata da `--exit-after` | si attende fino a `max(exit, screenshot) + 10 s` | revisione del codice |
| 7 | Affermazioni non provate nei documenti (determinismo cross-platform, "non sovrapposti", DPI 125%) | testi corretti; test di separazione rafforzato; DPI provato con `--scale-factor` | sopra |

### Limiti aperti

- DPI provato solo con override (`--scale-factor`), non su un monitor reale al 125%.
- Testato solo con finestra in primo piano; minimizzazione e monitor multipli non provati.
- La pausa su perdita di focus è attiva ma non provata con un vero cambio di finestra.
- Replay identici tra nativo e WASM non garantiti (vedi D-005).
- Build release non ancora compilata né misurata.
- Nessuna prova umana di "sensazione" del movimento: serve un playtest dell'utente.
