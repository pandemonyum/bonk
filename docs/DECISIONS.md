# DECISIONS

Decisioni reversibili prese con default prudenti. Formato: contesto → decisione → motivo.
Le decisioni marcate *(utente)* sono state approvate esplicitamente.

## D-001 Radice del repository = `C:\Workspace\Bonk` *(utente)*
L'handoff mostra `pixel-monster-survivor/` come radice; si usa la cartella esistente per non
creare un livello in più. Il nome del crate resta `pixel-monster-survivor`.

## D-002 Bevy 0.20.0 e Rust 1.99.1 bloccati
Bevy 0.20.0 (8 ottobre 2026) richiede Rust ≥ 1.97.1. Si blocca la stable corrente 1.99.1 in
`rust-toolchain.toml`. Aggiornamento: cambiare `channel`, `cargo update`, rieseguire
`scripts/check.*`, annotare qui.

## D-003 VS Build Tools 2022 invece di 2026 *(utente: MSVC via winget)*
La 2022 (17.14) è la versione più collaudata con `cc`/`link.exe` dalle dipendenze Rust; con la
2026 alcune build script con versioni vecchie di `cc` potrebbero non trovare il toolset.

## D-004 Feature Bevy: solo `2d` + `ui`
`default-features = false`, niente 3D/PBR/glTF: meno codice da compilare. `2d` include già
`webgl2`, `default_font`, `multi_threaded`. Audio aggiunto quando serve (P2.4). Feature del
progetto `dev` rimossa il 2026-10-22: in nativo `bevy/file_watcher` è sempre compilato e la ricarica a caldo si accende a runtime (`--watch`, default in debug), perché una seconda combinazione di feature ricompilava tutta Bevy e raddoppiava `target/` (vedi D-027).

## D-005 RNG proprio (PCG32 + SplitMix64), niente crate `rand`
Sequenze di interi stabili tra versioni di dipendenze e tra nativo e WASM (verificato contro la
sequenza di riferimento ufficiale di PCG32). Garanzia attuale: stesso seed **sulla stessa
piattaforma** = stessa partita (test headless). Tra nativo e WASM le posizioni possono
differire nell'ultimo bit perché `sin`/`cos` usano la libreria matematica del sistema; per
replay identici cross-platform valutare la feature `libm` (backlog).

## D-006 Arena chiusa 256×256 tile (4096×4096 px), 16 chunk da 64×64 *(utente)*
Niente streaming nell'MVP. Bordo di siepe di 2 tile; camera ristretta perché la vista non esca.

## D-007 Pixel-perfect con render-to-texture
Mondo disegnato su texture 480×270, mostrata con scala intera (`floor`) e nearest-neighbor,
bande nere sul resto. Calcolo in pixel **fisici**: verificato al 100% di texel uniformi con
DPI forzato a 1.25 e 1.5 (`--scale-factor`, finestra 1101×701), non ancora su uno schermo
reale al 125%. Con finestre di dimensione dispari il canvas viene spostato di mezzo pixel
fisico (senza: 96,1% di texel uniformi misurati, con: 100%). Sotto 480×270 fisici si riduce
in modo frazionario.

## D-008 Simulazione a passo fisso 60 Hz con `Position`/`PreviousPosition`
La logica non usa `Transform`; il rendering interpola con `overstep_fraction` e arrotonda al
pixel. In pausa si ferma `Time<Virtual>` oltre a bloccare i set con `RunPhase::Playing`.

## D-009 Gameplay separato dalla grafica
Le entità di gioco nascono senza sprite; la presentazione aggancia `Sprite` con observer
`On<Add<T>>`. Così `headless_app()` gira con `MinimalPlugins` e i test non richiedono GPU.

## D-010 Stati: `AppState::InRun` + sottostato `RunPhase { Playing, Paused }`
Le entità della run hanno `DespawnOnExit(AppState::InRun)`: il restart della P0.6 sarà uscire
e rientrare nello stato. `MainMenu`, `GameOver`, `Victory`, `LevelUp` si aggiungono in P0.

## D-011 Sprite come griglie di testo + Python/Pillow
Nessun editor grafico disponibile né approvato; gli agenti modificano file `.px` leggibili.
Palette "Germoglio" originale. Generatori IA di immagini esclusi senza approvazione.

## D-012 Dimensioni sprite 16–24 px per le creature comuni
L'handoff indica "tipicamente 24–32 px". A 480×270 una creatura da 32 px occupa il 12%
dell'altezza e riduce la leggibilità delle orde: comuni 16 px, compagni 16–24, élite 24–32,
boss 48–64. Da confermare nell'art pass (P2) guardando screenshot con 200 nemici.

## D-013 Testo HUD provvisorio a risoluzione nativa con FiraMono (OFL, incluso in Bevy)
Decisione sul font pixel e sull'HUD dentro/fuori dal canvas rimandata a P0.7/P2.

## D-014 Nessun crate dell'ecosistema Bevy nella S0
Tilemap con `TilemapChunk` integrato in Bevy, input e stati integrati. `bevy_ecs_tilemap`,
`leafwing-input-manager` ecc. esistono per 0.20 ma non servono ancora.

## D-015 Eccezioni clippy
`too_many_arguments` e `type_complexity` disattivati in `Cargo.toml [lints.clippy]`: tipici dei
sistemi Bevy (anche Bevy disattiva `type_complexity`). Tutto il resto con `-D warnings`.

## D-016 Identità Git locale
`user.email` impostata solo nel repository (`git config user.email`), non globalmente.
Commit locali atomici per tranche; nessun remote, nessun push.

## D-017 Documentazione essenziale
Non creati finché non hanno contenuto reale: `docs/RELEASE.md` e `.github/workflows/ci.yml`
(P1), `.github/copilot-instructions.md` (Copilot non in uso).

## D-019 Ordine della simulazione: `SimSet::Player` → `Movement` → `Resolve`
Il player si muove per primo, chi lo insegue legge sempre la posizione del passo corrente:
l'ordine non dipende da quello di registrazione dei sistemi (rilievo del code review S0).

## D-020 Limiti dell'arena come rettangolo `min`/`max` sui piedi
La `Position` è il punto dei piedi (ancora in basso al centro): il limite nord sottrae
l'altezza del corpo, quello sud solo un piccolo margine. Ingombri costanti in S0, da dati in P0.1.

## D-021 La respinta dal player è l'ultima regola dei nemici
Ordine: separazione → limiti arena → respinta dal player. Contro un muro un nemico può sforare
i limiti di al massimo `CONTACT_DISTANCE`, ma non entra mai nel player (test dedicato).

## D-022 Ostacoli sparsi con distanza minima invece di pathfinding
Rocce e alberi su tile non adiacenti (nemmeno in diagonale): i varchi restano ≥ 18 px, più
larghi di qualunque corpo, quindi l'arena è sempre attraversabile e i nemici possono inseguire
in linea retta scivolando attorno agli ostacoli. Niente pathfinding finché il level design non
richiede muri o corridoi. Due tile di margine dalla siepe, così la zona di respinta di ogni
ostacolo sta dentro i limiti dei piedi (dopo il code review P0). Densità, zona libera e quota
di alberi sono in `stages.ron`; distanza minima e margine restano nel codice perché sono la
garanzia di attraversabilità. La validazione dei dati impedisce corpi più larghi del varco e
velocità che salterebbero oltre un tronco in un passo. Collisione sull'impronta a terra (cerchio ai piedi); i proiettili
passano sopra gli ostacoli (lettura del combattimento prima del realismo); il compagno, che
fluttua, li ignora.

## D-023 Dati in `game-data/` via sorgente di asset `data://`
I file RON restano nella cartella `game-data/` in radice (come da handoff, ownership del
game-designer) e l'AssetServer li legge da una sorgente registrata `data` →
`AssetSourceBuilder::platform_default("game-data")`: da disco in nativo, via HTTP sul web
(il `dist/` della P1 dovrà includere `game-data/`). Loader proprio che restituisce il testo;
parsing e validazione insieme in `GameConfig::parse`, così gli errori sono tutti in un elenco
con file, voce e campo. `deny_unknown_fields` contro i refusi. Dipendenze aggiunte: `serde`
1 e `ron` 0.12, già presenti nel lockfile tramite Bevy (MIT/Apache-2.0).

## D-024 Restart e ricarica passano da `AppState::Loading`
Restart (F5) = `InRun → Loading → InRun`: le entità della run hanno `DespawnOnExit(InRun)`,
le risorse della run si ricreano in `OnEnter(InRun)`. La ricarica a caldo dei dati
(`--watch`, attiva di default nelle build di sviluppo) usa lo stesso percorso: dati validi e diversi → la run riparte con i valori
nuovi; dati non validi → errori a schermo e la configurazione precedente resta in uso.

## D-025 Headless con i dati incorporati a compile time
`headless_app()` usa `include_str!` degli stessi file di `game-data/`: i test girano senza
AssetServer e provano sempre i dati veri; un file non valido fa fallire il test
`the_real_data_files_are_valid`. `headless_app_with(seed, config)` per i test che cambiano valori.

## D-026 Sprite di attori e proiettili dai dati
Le entità portano un `DataId` e la presentazione ricava lo sprite dal percorso nei dati;
`validate-assets.py` verifica che ogni sprite citato in `game-data/` esista nel manifest.
Gli sprite degli ostacoli restano nel codice del mondo finché non c'è un `world.ron`.

## D-027 Spazio su disco di `target/` *(utente: ridurre lo spazio)*
Misurato il 2026-10-22: 21,7 GB, di cui ~11 GB copie di compilazione non più usate (nuovi hash a
ogni cambio di dipendenze o feature, più una build `--features dev` interrotta) e .pdb da
~0,75 GB per ogni eseguibile. Contromisure: dipendenze senza simboli di debug e nostro codice
con soli numeri di riga (`[profile.dev]`), un solo eseguibile per i test d'integrazione
(`tests/sim/`), un solo set di feature (watcher sempre compilato in nativo, acceso a runtime),
toolchain `stable` duplicata rimossa (default rustup = 1.99.1), `scripts/clean-stale.*` per
ripulire le copie vecchie del nostro crate. Costo: backtrace senza dettagli dentro Bevy.

## D-018 WebGL2 come unico backend web in P1 *(utente)*
In Bevy 0.20 la feature `webgpu` sostituisce `webgl2` nella stessa build: WebGPU eventuale
come seconda build separata dopo la P1.
