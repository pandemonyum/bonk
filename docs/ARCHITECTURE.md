# ARCHITECTURE

Rust 2024 + Bevy 0.20.0 (feature `2d`, `ui`). Un solo crate con libreria (`src/lib.rs`) e
binario sottile (`src/main.rs`). Niente workspace finché non serve.

## Due app dalla stessa libreria

| Funzione | Plugin | Uso |
|---|---|---|
| `client_app(options)` | `DefaultPlugins` (nearest) + simulazione + input + render mondo + presentazione + diagnostica | gioco desktop/web |
| `headless_app(seed)` | `MinimalPlugins` + `StatesPlugin` + simulazione; `TimeUpdateStrategy::FixedTimesteps(1)` | test, benchmark, futura simulazione di run complete |

## Moduli

| Modulo | Contenuto | Richiede GPU |
|---|---|---|
| `app` | `AppState` (`Loading`, `MainMenu`, `InRun`), sottostato `RunPhase` (`Playing`, `Paused`, `LevelUp`, `Defeated`, `Victory`), `RunSeed`, `RunSetup` (compagno scelto), `AfterLoading` (menu o run dopo il caricamento), `restart_run`, `back_to_menu`, `SimulationPlugin` | no |
| `data` | schema dei file `game-data/*.ron`, `GameConfig` validata, `DataStatus`, `DataId`, copia incorporata per headless | no |
| `data::loader` | sorgente `data://`, loader di testo, primo caricamento e ricarica a caldo | AssetServer (client) |
| `options` | parsing CLI (`--seed`, `--autoplay`, `--overlay`, `--screenshot*`, `--exit-after`, `--window`, `--scale-factor`) | no |
| `rng` | PCG32, SplitMix64, `hash2` per coordinate | no |
| `gameplay::kinematics` | `Position`, `PreviousPosition`, `body()` | no |
| `gameplay::player` | `Player`, `MoveIntent`, movimento 8 direzioni dentro l'arena | no |
| `gameplay::enemies` | `Enemy`, `enemy_bundle`, inseguimento, separazione (griglia, stile Jacobi), ostacoli, limiti, respinta dal player | no |
| `gameplay::spawning` | spawn director: budget per ondata, limiti per archetipo e totali, punti validi fuori vista; difficoltà per minuto (`Scale`), élite periodiche, boss a fine durata | no |
| `gameplay::spatial` | `EnemyGrid`: griglia 32 px ordinata per cella, ricostruita prima di `Resolve` e di `Combat` | no |
| `gameplay::damage` | danno da contatto al player, `Invulnerability`, `GodMode`, passaggio a `Defeated` | no |
| `gameplay::run` | `RunClock` (tempo di gioco, fermo fuori da `Playing`), conto alla rovescia, vittoria in `PostUpdate` (boss nato e non più in campo, player vivo) | no |
| `gameplay::threats` | `Charger` (carica dell'élite), `Slammer` (colpo ad area del boss), `Telegraph`, `PendingSlam`; preavvisi di almeno `MIN_TELEGRAPH_S` | no |
| `gameplay::companions` | `Companion` (posto e angolo di formazione), `spawn_companion` con l'attacco del suo pattern e gli upgrade correnti, `join_companions` dalle carte `JoinCompanion` | no |
| `gameplay::attacks` | pattern `Projectile`: `AutoAttack`, carica e cadenza (`attack_tick`, riusata dall'onda), scelta del bersaglio, raffiche | no |
| `gameplay::pulse` | pattern `Pulse`: `PulseAttack`, onde (`Pulse`) che si allargano dal player, colpo una volta per onda, `Slowed` | no |
| `gameplay::orbit` | pattern `Orbit`: `OrbitAttack`, foglie (`Leaf`) equidistanti attorno al player, intervallo per nemico | no |
| `gameplay::combat` | `Health`, `Hurtbox`, `Projectile`, `HitFlash`, `RunStats`; volo dei proiettili, colpi, morti | no |
| `gameplay::progression` | `XpGem`, `Attracted`, `PickupRadius`, `Progression` (livello, XP, livelli in attesa); gemme lasciate dai morti, limite con fusione, attrazione e raccolta | no |
| `gameplay::upgrades` | `UpgradeStacks`, `Draft`, `ChooseUpgrade`, `AutoPick`; estrazione pesata, apertura del draft, scelta, ricalcolo a caldo, rigenerazione; `upgrades::modifiers`: `Modifiers` (dati base + stack), `pct_factor` | no |
| `gameplay::input` | tastiera → `MoveIntent`, Esc/P pausa, pausa su perdita focus, 1/2/3 nel draft | input (client) |
| `world` | costanti arena, `ArenaBounds` (`min`/`max` sui piedi), `inside_arena` (limite proiettili); `generation` (seed → tile, pura); `tiles` (indici); spawn degli ostacoli (`WorldSetup`) | no |
| `world::obstacles` | rocce/alberi dal seed (pura), `ObstacleField` con griglia spaziale 32 px, entità `Obstacle` | no |
| `gameplay::collision` | `Footprint`, `push_outside`, respinta dei corpi dagli ostacoli | no |
| `world::render` | terreno in pezzi da 512 px cotti sulla CPU e mostrati come sprite attorno alla camera (D-061) | sì |
| `world::view` | vista minima 480×270 e più grande 640×360, centro camera; condivisa da camera, spawn director e forzieri, che considerano in vista la più grande (D-062) | no |
| `presentation::camera` | canvas 480×270, camera mondo/schermo, scala intera, offset mezzo pixel, follow; `UiScale` dalla scala del canvas (D-045) | sì |
| `presentation::sprites` | observer `On<Add<Enemy/Companion/...>>` → `Sprite`; interpolazione + snap al pixel + y-sort; tinte di colpo e rallentamento | sì |
| `presentation::player_anim` | sprite del player dal foglio 4×8 (`TextureAtlas`): fermo e cammino in 4 direzioni (sinistra specchiata), colpito, caduta; stato in `PlayerAnimation`, logica pura testata | sì |
| `presentation::attacks` | foglie come sprite; anello di gocce figlie dell'onda, al raggio interpolato e arrotondato al pixel | sì |
| `presentation::hud` | promemoria "Esc: pausa e comandi", schermata di pausa con comandi e potenziamenti presi, errori dei dati | sì |
| `presentation::fonts` | `UiFonts` (Tiny5, Pixelify Sans), `Typeface` con hook che assegna font e antialiasing, `hud(size)` / `ui(size)` al posto di `TextFont::from_font_size` | sì |
| `presentation::icons` | `ui_scale` (scala dell'interfaccia dalla scala del canvas e dal DPI), `UI_PX_PER_TEXEL`, `ui_icon`: icone di `assets/ui/` a texel interi | sì |
| `presentation::abilities` | fila delle abilità: un posto per posto della squadra, icona dell'elemento con barra di ricarica, lucchetto sui posti liberi | sì |
| `presentation::health_bar` | barra degli HP 16×2 texel sotto i piedi del player, figlia del suo sprite | sì |
| `presentation::boss_pointer` | teschio al bordo della vista nella direzione del boss fuori schermo | sì |
| `presentation::status` | in alto a sinistra cuori (20 HP l'uno) con HP, barra XP e il contenitore della fila delle abilità; in alto a destra livello e teschio con le uccisioni | sì |
| `presentation::menu` | menu iniziale: carte dei compagni (sprite, elemento, attacco, barre relative), 1/2/3, frecce, clic, Invio | sì |
| `presentation::results` | risultati di sconfitta (dopo la caduta del player) e vittoria: tempo, livello, nemici, danni, squadra, potenziamenti | sì |
| `presentation::boss_bar` | conto alla rovescia al centro in alto, poi nome e barra HP del boss | sì |
| `presentation::telegraphs` | punti rossi in fila o in cerchio, figli del `Telegraph`, che lampeggiano verso lo scadere | sì |
| `presentation::draft` | schermata del draft: carte con rarità, testo e stack, evidenziate al passaggio, clic → `ChooseUpgrade` | sì |
| `diagnostics` | overlay F3, screenshot F12/programmati, autoplay, uscita automatica | sì |

## Flusso di un frame

1. `PreUpdate`/`Update`: input o autoplay scrivono `MoveIntent` (risorsa).
2. `FixedPreUpdate` (60 Hz): `PreviousPosition ← Position`.
3. `FixedUpdate`, in catena e solo in `RunPhase::Playing`: `SimSet::Player` (movimento del
   player con respinta dagli ostacoli e limiti) → `SimSet::Movement` (inseguimento, a velocità
   ridotta se `Slowed`; formazione dei compagni; volo dei proiettili; fronte delle onde; foglie
   create o tolte secondo l'attacco, poi spostate attorno al player) → `SimSet::Resolve`
   (separazione, ostacoli, limiti arena, respinta dal player per ultima; la griglia viene
   ricostruita all'inizio) → `SimSet::Combat` (griglia ricostruita; spari e onde in partenza;
   colpi di proiettili, onde e foglie; gemme; morti; scadenza proiettili, fine tinta e
   rallentamento) →
   `SimSet::Damage` (danno da contatto al player dai nemici vivi, poi colpi ad area allo
   scadere del preavviso, con le stesse regole: `strike_player`) → `SimSet::Pickup` (gemme
   attirate e raccolte, rigenerazione) → `SimSet::Spawn` (spawn director: ondate ed élite fino a
   fine durata, poi il boss). Una morte è contata
   una sola volta e lascia una sola gemma: `drop_xp_gems` gira tra i colpi e `resolve_deaths`,
   che rimuove il morto nello stesso passo; i colpi ignorano chi è a 0 HP. Il passo fisso gira
   solo con `simulation_running` (Playing, player vivo, nessun livello in attesa): il passo
   della morte o del livello nuovo è l'ultimo anche col tempo accelerato.
   `RunPhase::Paused`, `LevelUp` e `Defeated` fermano anche il tempo virtuale.
4. `Update`: con livelli in attesa `open_draft` passa a `LevelUp`, dove `deal_draft` (in
   `OnEnter`) estrae le carte, così una pausa nello stesso fotogramma non spreca estrazioni; la scelta
   (tasti, clic o `AutoPick`) arriva come messaggio `ChooseUpgrade` (vale il primo del
   fotogramma, gli altri sono scartati), aggiunge uno stack, apre il
   draft successivo o torna a `Playing`; `refresh_modifiers` ricalcola attacchi e player dai
   dati base più gli stack (solo in `InRun`: nel client la `GameConfig` arriva dopo l'avvio).
5. `PostUpdate` (prima di `TransformSystems::Propagate`): `SyncTransforms` scrive i
   `Transform` interpolati e arrotondati; la camera mondo segue il player a pixel interi;
   il canvas si adatta alla finestra.

## Rendering pixel-perfect

- `WorldCamera` (order −1, layer 0) disegna su `Image` 480×270 `Bgra8UnormSrgb`, MSAA off.
- Sprite `Canvas` (layer 1) mostra la texture; `ScreenCamera` (layer 1, `IsDefaultUiCamera`)
  la scala di `n = floor(min(Wfis/480, Hfis/270))` con `ortho.scale = scale_factor / n`.
- `ImagePlugin::default_nearest()` per tutte le texture. Ancora degli sprite in basso al centro
  con larghezza pari: i bordi cadono sulla griglia dei texel.
- Profondità: suolo z=0, gemme z = 5 − y·10⁻⁴, attori z = 10 − y·10⁻⁴, proiettili, foglie e
  onde z = 20 − y·10⁻⁴ (`DepthBase`; le gocce dell'onda ereditano la z dell'entità `Pulse`).

## Dati

Tutti i valori di bilanciamento sono in `game-data/` (`player`, `companions`, `attacks`,
`enemies`, `stages`, `progression`, `upgrades`, schema versione 5). Restano nel codice solo le costanti strutturali
(dimensioni dell'arena e dei tile, frequenza della simulazione, densità degli ostacoli).

```text
client:   Startup → load data://*.ron → (tutti caricati) → GameConfig::parse
              ok  → GameConfig + DataStatus::Ready → Loading → InRun
              err → DataStatus::Error (schermata) → si riprova al salvataggio del file
          AssetEvent::Modified (--watch) → parse → ok e diversa → restart della run
headless: GameConfig::embedded() (include_str! degli stessi file) → InRun dal primo update
```

La validazione controlla anche vincoli fisici legati al codice: nessun corpo più largo del
varco minimo tra ostacoli, nessun proiettile abbastanza veloce da attraversare un nemico tra
due passi, anelli di spawn coerenti, riferimenti tra file esistenti. Per gli upgrade gli stessi
limiti valgono con tutti gli stack al massimo, più requisiti non circolari, primo draft mai
vuoto e testi a schermo con i soli caratteri che i font dell'interfaccia disegnano (D-044).

## Test

- Unit test accanto al codice per funzioni pure (direzioni, inseguimento, separazione, RNG,
  generazione, scala canvas, opzioni).
- `tests/sim/` è un solo eseguibile (D-027) con i moduli `movement` (movimento, limiti,
  pausa, restart, attesa dei dati come nel client), `combat` (compagno, danni, morti, cadenza),
  `damage`, `spawning`, `progression` (gemme, livelli, draft, upgrade a caldo), `world`
  (ostacoli, collisioni) e helper in `common`; tutti senza GPU. `headless_app` sceglie da sola
  la prima carta del draft (`AutoPick`); `quiet_config` toglie le XP per misurare i valori base.
- `tests/sim/formulas.rs` (P0.7): formule misurate nella simulazione con i valori dei dati:
  danno per colpo e cadenza dei tre pattern con gli stack, livelli dall'XP raccolta.
- `tests/sim/balance.rs`: sonde `#[ignore]` (colpi, sopravvivenza con e senza upgrade, ritmo
  dei livelli, costo per passo, "giocatore medio" di `tests/sim/bot.rs` che schiva, raccoglie e
  sceglie le carte; `PMS_SEEDS=1,2,3` per scegliere le seed):
  `cargo test --test sim balance -- --ignored --nocapture`.
- `scripts/check-pixel-grid.py` per la verifica nearest-neighbor su screenshot reali.
