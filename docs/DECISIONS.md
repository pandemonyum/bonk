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
Decisione sul font pixel e sull'HUD dentro/fuori dal canvas rimandata a P0.7/P2. Font: superata
da D-044 (FiraMono resta solo nell'overlay di debug); l'HUD resta a risoluzione nativa.

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

## D-028 Spawn director a budget con rette per ondata
Ogni ondata ha ritmo = base + crescita × minuti (due numeri leggibili dal designer) e un budget
che matura a ogni passo; il budget non si accumula oltre 1 quando si è al limite o manca un punto
valido, così non partono raffiche. Punti scartati se in vista (con 32 px di margine), fuori
dall'arena o in un ostacolo: si usa la stessa geometria della camera (`world::view`), quindi
anche negli angoli, dove la camera non è centrata sul player, nessun nemico compare a schermo.
Due ondate con lo stesso archetipo condividono il conteggio dei vivi.

## D-029 Danno da contatto sommato per finestra di invulnerabilità
Alternative valutate: solo il nemico più forte (la folla non conta), somma con rendimenti
decrescenti (più regole da spiegare). Scelta: somma semplice, con valori bassi (3/2/6) e 0,6 s
di finestra, tarati con la sonda di sopravvivenza. Stare fermi in mezzo all'orda è letale, come
nel genere; muoversi è la difesa. La sconfitta è `RunPhase::Defeated` (minima); risultati
completi e vittoria arrivano con la P0.6. Dopo il code review P0.3 la sconfitta si decide
dagli HP in `PostUpdate` (vince su una pausa chiesta nello stesso fotogramma) e tutto il passo
fisso gira solo con il player vivo (`simulation_running`): il passo della morte è l'ultimo
anche col tempo accelerato.

## D-030 Griglia spaziale ordinata per cella, ricostruita due volte per passo
Ordinamento stabile per indice di cella + indice d'inizio per cella: niente hash, allocazioni
riutilizzate, celle di una riga contigue, ordine deterministico. Ricostruita prima della
separazione e prima del combattimento (posizioni aggiornate per mira e colpi): O(n) ciascuna,
misurato 0,45–0,62 ms per passo completo con 50–500 nemici in build dev. Separazione in stile
Jacobi (tutte le spinte calcolate sulla stessa fotografia) per non dipendere dall'ordine di visita.

## D-031 XP intera, gemme attirate per sempre, limite con fusione
XP e soglie sono interi (la curva `base + per_level × (n − 1)` si arrotonda per eccesso): l'HUD
mostra numeri esatti e i test confrontano senza tolleranze. Una gemma entrata nel raggio di
raccolta resta attirata finché non arriva, a una velocità che la validazione vuole più alta di
quella massima del player (con tutti gli upgrade). Oltre `max_on_field` l'XP di un nemico va alla
gemma più vicina, già a terra o nata nello stesso passo: nessuna XP persa, al massimo 300
entità. Le gemme nascono nella catena del combattimento tra i colpi e la rimozione dei morti,
quindi una per nemico; la raccolta gira in `SimSet::Pickup`, dopo i danni e prima dello spawn.

## D-032 Draft come fase della run, upgrade ricalcolati dai dati base
`RunPhase::LevelUp` ferma il tempo virtuale come la pausa; `simulation_running` richiede anche
nessun livello in attesa, così il passo del livello nuovo è l'ultimo anche col tempo accelerato
(stessa regola della sconfitta, D-029). Carte estratte senza ripetizione, pesate per rarità, con
uno stream RNG dedicato (2): stesso seed e stesse scelte = stesso draft. A ogni scelta attacchi e
player si ricalcolano da dati base + stack (nessun errore che si accumula; restart e ricarica dei
dati ripartono puliti); la carica degli attacchi resta, la durata dei proiettili segue gittata e
velocità perché la portata resti almeno la gittata. Gli stack sono indicizzati come
`GameConfig::upgrades`: la configurazione non cambia durante una run (una ricarica la fa
ripartire). Le carte si estraggono entrando in `LevelUp` (`OnEnter`), non quando il draft viene
chiesto: se nello stesso fotogramma vince una pausa nessuna estrazione va persa (rilievo del code
review). Vale la prima scelta del fotogramma, le altre sono scartate. Scelta automatica della
prima carta (`AutoPick`): subito in headless, perché test e
simulazioni lunghe non si fermino al primo livello (i test del draft la tolgono); dopo 0,6 s
reali con `--autoplay`, così il draft si vede. Senza upgrade disponibili il livello passa senza
draft.

## D-033 Testi delle carte solo in ASCII finché non c'è un font *(superata da D-044)*
Il FiraMono ridotto incluso in Bevy (D-013) non ha le lettere accentate: nella prima prova del
draft "più" usciva come "pi" seguito da un quadratino. I testi di `upgrades.ron` sono stati
riformulati e la validazione rifiuta caratteri non ASCII con un messaggio che lo spiega. Da
togliere quando in P0.7/P2 si sceglie un font (pixel OFL o bitmap originale) con le accentate.

## D-034 Limiti degli upgrade sommati per segno, tetti strutturali
La validazione somma le carte con lo stesso effetto, separando quelle che tolgono (il valore più
basso raggiungibile deve restare positivo: niente intervallo 0 con 60 raffiche al secondo) da
quelle che aggiungono (il valore più alto deve rispettare i limiti fisici). Sommare tutto
insieme faceva compensare una carta +50% con una −30%. Tetti nel codice, controllati sui dati:
3 carte per draft (tasti 1-3, larghezza dello schermo), 32 proiettili per raffica, 1000 XP per
nemico; perforazione e XP usano somme saturanti (`pierce` enorme resta "infinito"). I compagni
devono stare dietro al player anche alla sua velocità massima.

## D-035 Attacchi con campi comuni e un `pattern` per tipo
`attacks.ron` (schema 4) ha `damage`, `interval_s` e `range` per tutti gli attacchi, più il
`pattern`: `Projectile`, `Pulse` (onda), `Orbit` (foglie), ognuno con i suoi parametri. Così le
carte "Companion" valgono per tutti senza casi speciali: la gittata è anche il raggio dell'onda
e dell'orbita, "+1 proiettile" è anche "+1 foglia", la cadenza delle foglie è l'intervallo tra
due colpi sullo stesso nemico. Onda e foglie sono centrate sul player (come nel GDD), non sul
compagno: si leggono meglio e proteggono dove serve. L'onda è un disco pieno che si allarga
(chi è dentro il fronte è raggiunto, niente tunnel) e colpisce una volta per onda; parte solo
con un nemico nel raggio, con la stessa carica dei proiettili. Il rallentamento non si somma:
vale il più forte e il più lungo, anche tra due onde nello stesso passo. Un'onda deve finire
prima della successiva (validato con gli upgrade al massimo). Per le foglie la validazione
controlla che il passo lungo l'orbita, anche con l'orbita più larga, non salti oltre un nemico;
l'intervallo tra due colpi sullo stesso nemico si arrotonda al passo più vicino. Restano vietati
foglie ferme (`angular_speed_deg: 0`) e congelamento (`slow: 0`): sarebbero pattern diversi.

## D-036 Squadra: posti nel player, compagni dalle carte del draft
`player.ron: companion_slots` è la lista dei posti (il primo è del compagno iniziale), ognuno con
l'angolo nella formazione dietro al player (±55°: tre compagni distinti). Gli altri compagni
arrivano con le carte `JoinCompanion` ("nuovo slot compagno" dell'handoff), proposte solo con
un posto libero e se il compagno non è già in squadra (`PartyView`, calcolata dal compagno
iniziale e dalle carte prese); il posto è il primo libero nell'ordine di scelta. Il compagno
nuovo nasce con gli upgrade già presi (rilievo P4 del code review P0.4) e con la direzione di
marcia della squadra, quindi già dietro al player. La scelta del compagno iniziale resta
in `stages.ron` finché il menu della P0.6 non la propone al giocatore.

## D-037 Onda disegnata con gocce figlie, niente scale
L'anello dell'onda non è uno sprite scalato (vietato dalla regola pixel-perfect): è fatto di
8–40 gocce 4×4, figlie dell'entità `Pulse`, spostate ogni fotogramma sul raggio interpolato e
arrotondato al pixel. Il numero di gocce dipende dal raggio finale (una ogni ~9 px). Le gocce
non hanno contorno per non scurire l'orda che attraversano.

## D-038 Arco della run: boss allo scadere del timer, vittoria quando cade
Il timer di 10 minuti (`stages.ron: run.duration_s`) porta al boss: a quel punto ondate ed élite
si fermano e nasce il boss (Gran Corazzato), fuori vista; batterlo è la vittoria (stato `Victory`, deciso in
`PostUpdate` come la sconfitta, che vince se player e boss cadono nello stesso passo). La
difficoltà cresce in modo lineare per minuto su HP e danno dei nemici di ondate ed élite, applicata
alla nascita (chi è già in campo non cambia); il boss non è scalato. Élite a intervalli fissi, non
casuali: si sa che arrivano. Il passo in cui cade il boss è l'ultimo anche col tempo accelerato
(`simulation_running`, come per la morte), quindi l'esito non dipende dagli FPS.

## D-039 Attacchi telegrafati: preavviso minimo, stesse regole del contatto
Carica e colpo ad area mostrano il segno per almeno 0,5 s (validato), con lampeggio sempre più
rapido. Il colpo cade dove stava il player al preavviso, non dove va: schivare è uscire dal
cerchio, e la validazione vuole che dal centro si esca in tempo alla velocità base. Il danno
passa da `strike_player`, come il contatto: invulnerabilità, modalità invulnerabile e morte
valgono uguali. Ogni segno ricorda chi l'ha lanciato e sparisce senza effetto se muore (anche
nello stesso passo): il colpo di un boss appena sconfitto non arriva. La carica non supera in un
passo la distanza di contatto (validato), per non saltare oltre il player.

## D-040 Menu e scelta del compagno, ritorno senza menu
`AppState::MainMenu` mostra i compagni dei dati; la scelta va in `RunSetup` e decide il compagno
iniziale (`RunSetup::starter`, che ripiega su quello dello stage se la ricarica lo toglie) e la
squadra del draft (`PartyView`). `AfterLoading` dice dove andare dopo il caricamento: menu
all'avvio del client e con M dai risultati, run diretta in headless, con `--autoplay`, con F5 e
con Invio dai risultati. Ogni compagno ha la sua carta "nuovo compagno" (anche Brasa): quella del
compagno scelto non esce. L'HUD della run nasce entrando in `InRun` e sparisce uscendo.

## D-042 Carte proposte solo se servono alla squadra
Con la scelta del compagno iniziale si poteva partire senza nessuno che spara e vedersi proporre
perforazione e velocità dei proiettili. `PartyView` dice se in squadra c'è chi spara
(`has_projectiles`) e chi ha proiettili o foglie (`has_counted`): perforazione e velocità
richiedono il primo, "+1 proiettile" il secondo. Le altre carte valgono per tutti.

## D-041 La scelta automatica preferisce i compagni
`AutoPick` (autoplay, simulazioni) sceglie una carta "nuovo compagno" se c'è, altrimenti la
prima: la squadra è il potenziamento più grande e ogni giocatore la prende. Prima sceglieva la
prima carta e le sonde misuravano soprattutto la sfortuna del draft.

## D-043 Art pass v1 nel gioco: icone alla scala del canvas, valori dagli sprite *(utente)*
Il ramo dell'art pass è unito in `main` (conflitti: vince la versione grafica degli sprite di
élite, boss e segno di pericolo; licenze unite; manifest rigenerato). Le icone di `assets/ui/`
sono disegnate a texel del canvas: l'HUD resta a risoluzione nativa (D-013) e ogni icona prende
`lato × PixelScale`, cioè la scala intera del canvas divisa per il DPI della finestra, così
occupa gli stessi pixel fisici di uno sprite del mondo e resta nitida. Cuori al posto della
riga di testo degli HP (uno ogni 20 HP, l'ultimo vale il resto), barra XP sotto i cuori, icone
degli elementi della squadra, orologio, teschio, icone delle carte, ritratti nel menu; colori
dei pannelli dalla palette v1. I valori di gioco cambiati dalle dimensioni nuove (hurtbox,
altezze di sparo e del centro di onda e foglie, impronte di roccia e albero, ingombro del
corpo) sono quelli proposti in `docs/art/STYLE_HANDOFF.md`. Il boss diventa "Gran Corazzato":
lo sprite è un colosso a cupola della famiglia del corazzato, non una gelatina. Il font resta
FiraMono finché l'utente non sceglie tra le proposte OFL dell'handoff (testi ancora ASCII).

## D-044 Font pixel OFL: Tiny5 per l'HUD, Pixelify Sans per menu, draft e risultati *(utente)*
I due font proposti nell'handoff dell'art pass, scaricati dal repository ufficiale di Google
Fonts (`ofl/tiny5`, `ofl/pixelifysans`) con il loro `OFL.txt`, stanno in `assets/fonts/<font>/`;
file rinominati, contenuto intatto, nessun Reserved Font Name. Verificati sul cmap: tutto
l'ASCII e tutto il Latin-1 tranne il trattino morbido (e la barra spezzata in Pixelify), più
‘ ’ “ ” – — … €. Tiny5 ha 8 pixel di disegno per em: a 16 e 24 px ogni suo pixel è 2×2 e 3×3,
quindi è usato solo a quelle dimensioni, senza antialiasing e con un'ombra di un pixel del font
(leggibile sopra l'erba). Pixelify Sans ha circa 11 pixel per em e contorni non tutti sulla
griglia: resta con l'antialiasing, a 22/33/44 px per titoli e nomi e a 16 px per le frasi.
Tiny5 per numeri, tasti ed etichette brevi (anche dentro menu, carte e risultati), Pixelify
Sans per titoli e frasi. Un testo sceglie il font con `fonts::hud(size)` o `fonts::ui(size)`:
l'hook del componente `Typeface` mette handle e antialiasing appena il testo nasce. Il vincolo
ASCII (D-033) diventa "solo caratteri che i due font disegnano" (`data::ui_drawable`), con un
test che lo confronta con il cmap dei file; le carte del draft passano a 212×252 px perché
nome su due righe e descrizione su tre entrino. Dipendenza nuova solo per i test: `skrifa` 0.44
(MIT OR Apache-2.0, il lettore di font di Google usato da parley), come dev-dependency alla
stessa versione e con le stesse feature che Bevy compila già: nessun crate nuovo nel lockfile.

## D-045 Tutta l'interfaccia alla scala intera del canvas (`UiScale`)
Sostituisce `PixelScale` della D-043, che ingrandiva solo le icone: a finestra grande i testi e
i pannelli restavano piccoli accanto a icone doppie. La camera imposta `UiScale` = scala intera
del canvas / fattore DPI / 2, quindi l'interfaccia si disegna una volta per 960×540 (2 unità per
texel del canvas) e a ogni finestra testi, pannelli e icone crescono insieme al mondo con lo
stesso pixel e le icone restano a texel interi. Due regole nuove (rilievi della revisione):
Tiny5 solo a 16 o 32 px, perché i 24 px della D-044 davano pixel del font da 4,5 pixel con il
canvas ×3; misure, bordi e spazi dell'interfaccia sempre pari (1 texel = 2 unità). L'HUD resta
fuori dal canvas (D-013), solo scalato come lui. Sotto la scala 1 (finestre più piccole del
canvas) l'interfaccia non si riduce oltre. Il layout in pixel (carte, pannelli) non cambia.

## D-046 HUD della P0.7: abilità, HP sotto il player, boss fuori schermo, comandi in pausa
Seguendo l'handoff dell'art pass, la fila sotto la barra XP diventa la fila delle abilità: un
posto per ogni posto della squadra dei dati, icona dell'elemento con la barra di ricarica
dell'attacco a texel interi (le foglie girano sempre: barra piena), lucchetto sui posti liberi.
Sotto i piedi del player una barra degli HP di 16×2 texel nel mondo, sempre visibile. Il boss
fuori vista ha un indicatore al bordo (teschio su riquadro rosso) nella sua direzione. La riga
fissa dei comandi in basso diventa "Esc: pausa e comandi": l'elenco completo e i potenziamenti
presi sono nella schermata di pausa, una riga breve nel menu. Cuori e fila delle abilità si
ricostruiscono anche quando la loro fila è nuova: prima lo stato ricordato era solo la lista dei
cuori, quindi una run ricominciata con gli stessi HP poteva restare con la fila nuova vuota.

## D-047 Niente `HashMap` nell'ordine degli effetti della simulazione
Le prime sonde del "giocatore medio" davano risultati diversi con la stessa seed. Causa: l'onda
di Gocci raccoglieva i rallentamenti del passo in una `HashMap` e li inseriva nell'ordine della
mappa, che in Rust è casuale a ogni istanza; l'ordine degli inserimenti decide l'ordine dei
nemici nelle tabelle dell'ECS, quindi bersagli a pari distanza e separazioni cambiavano. Ora i
rallentamenti si scrivono nell'ordine dei colpi (un `Vec`). Regola: nella simulazione nessuna
iterazione su `HashMap`/`HashSet` che produca comandi o effetti; il test
`the_whole_party_in_a_crowd_is_deterministic` (tre compagni in un'orda fitta, 90 s) fallisce con
il codice vecchio e passa con quello nuovo.

## D-048 Bilanciamento base della P0.7: giocatore medio come riferimento, compagno garantito
Le sonde di prima misuravano due estremi (autoplay cieco che muore a ~2 minuti, bot che scappa
e non cresce). Il riferimento diventa il "giocatore medio" di `tests/sim/bot.rs`: gira attorno
ai nemici, esce dai segni di pericolo, raccoglie e sceglie carte sensate, su 8 seed per ognuno
dei tre compagni iniziali. Cambi ai dati, ciascuno con la misura che lo motiva (GDD, Misure):
nei draft dei livelli 2 e 3 c'è sempre una carta "nuovo compagno" se un posto è libero
(campo nuovo `progression.ron: companion_card_until_level`, 0 = nessuna garanzia, validato:
serve almeno una carta compagno), rigenerazione 0,5 → 0,3 HP/s per stack, danno dei nemici
+8% → +12% al minuto, boss 3800 → 5500 HP. Risultato: 23/24 vittorie, boss in 50–95 s. Il
bot reagisce meglio di una persona: la taratura successiva si fa dopo una prova umana, non
spingendo oltre sul bot.

## D-049 Player nuovo con cappello, animato da un foglio di 32 fotogrammi *(utente; disegno superato da D-050)*
L'utente ha mandato un foglio di animazioni come riferimento e chiesto un personaggio "uguale".
Il personaggio del riferimento (cappello rosso con stemma bianco, capelli neri, maglia rossa,
salopette blu) ha i tratti distintivi di personaggi Nintendo e la provenienza del foglio è
ignota: niente copia né ricalco dei pixel. Prima proposta (giacca e borsa dei semi) giudicata
dall'utente troppo diversa; scelta finale dell'utente: disegnato da zero con proporzioni
(~17×26, testa col cappello circa metà altezza e più larga del corpo), taglio dell'abito
(cappellino a cupola con visiera, maglietta a maniche corte, salopette con bretelle e due
bottoni), dettaglio e pose del riferimento, ma colori e stemma propri: cappello teal con un
germoglio d'oro, capelli castani, maglietta arancione, salopette verde oliva, bottoni di legno,
scarpe marroni. Il taglio "cappellino + salopette" resta l'archetipo che si associa a Mario
anche con colori, stemma e capelli diversi e senza baffi: rischio di somiglianza segnalato
all'utente, da rivalutare prima di una pubblicazione. Animazioni scelte dall'utente: fermo e
cammino nelle quattro direzioni, colpito e
sconfitto (niente salto o accovacciato: il gioco è top-down). Foglio `sprites/summoner_sheet.png`
4×8 fotogrammi da 20×28 (la sinistra è la destra specchiata), contratto in
`presentation::player_anim` e in `docs/ART_BIBLE.md`, verificato da un test sul PNG. Direzione
dall'asse dominante dello spostamento, passo più svelto con gli upgrade di movimento, colpito
quando l'invulnerabilità riparte, caduta a tempo reale (a run persa il tempo di gioco è fermo)
e pannello della sconfitta dopo la caduta (1,2 s; l'ultimo fotogramma è a faccia in giù, come
nel riferimento; a terra niente tinta di colpo, perché a run persa il lampo non scadrebbe più).
I limiti dell'arena restano quelli del
corpo 16×24: contro la siepe nord il cappello sporge di 2–3 texel, davanti alla siepe.

## D-050 Il player è lo sprite fornito dall'utente, riprodotto identico *(utente)*
Le due versioni originali della D-049 (giacca, poi stesso taglio con colori propri) non
bastavano all'utente, che vuole il personaggio identico alla sua immagine. L'utente ha generato
un foglio 4×8 nel formato richiesto, dichiara di averne i diritti e ha chiesto la riproduzione
identica; il rischio di somiglianza con personaggi Nintendo (cappello rosso con stemma bianco,
capelli neri, maglia rossa, salopette blu) gli è stato spiegato e lo accetta: questa decisione
supera, per il player, la regola "stile sì, copie no" della P0 ed è da rivalutare prima di
qualunque pubblicazione. Il foglio arrivato in chat (webp ingrandito circa ×5 e compresso, con
"pixel" di misure diverse tra un fotogramma e l'altro) è stato tagliato a risoluzione nativa
seguendo la griglia vera di ogni fotogramma, poi ridisegnato pulito per un cammino leggibile:
un modello per direzione (testa e corpo uguali in tutti i fotogrammi, anche tra fronte, retro
e profilo), passo in 4 tempi con il piede alzato di 3 px e la braccia opposte, colpito e caduta
con le pose dell'utente alla stessa scala. Fotogrammi 36×38 (personaggio largo ~23 e alto ~30,
più grande dei compagni), palette propria del foglio (`summoner_sheet.palette.txt`, 24 colori
più l'ombra del gioco), validata a parte. I limiti dell'arena restano quelli del corpo 16×24:
contro la siepe il player sporge di ~6 texel a nord e 3–4 ai lati, disegnato davanti.

## D-051 Nemici e boss animati, in proporzione al player *(utente)*
Scelte dell'utente: disegni a mano del pixel-art-director (originali, stessa identità di prima)
e misure in proporzione al player di D-050. Ogni nemico ha un foglio di 4 colonne (campo
facoltativo `frame` in `enemies.ron`; senza, resta un'immagine fissa): riga 0 movimento, 1
sconfitta, e per élite e boss 2 preparazione e 3 attacco. Fotogrammi: gelatina 28×24, sciame
24×22, corazzato 32×28, gelatina gigante 44×38, Gran Corazzato 96×80 (palette comune
`enemies.palette.txt`). `presentation::enemy_anim`: ciclo di movimento specchiato sul verso,
carica e scatto dal `Charger`, preparazione del boss finché c'è il suo `PendingSlam` e colpo per
0,4 s dopo, resti animati dove un nemico è morto (entità di sola grafica, caduta 0,4 s e
dissolvenza 0,3 s; non per chi sparisce al restart). Valori nei dati misurati sui corpi dei
fogli: hurtbox e separazione più grandi (es. gelatina raggio 7 → 9, separazione 14 → 18; boss
22 → 30 e 44 → 60), impronte invariate (piedi, max 9). Lezione: alzare anche la distanza di
contatto dei nemici piccoli faceva colpire da troppo lontano (24 run del giocatore medio: da
23 a 16 vittorie, morti fra 4:17 e 7:24); tornati ai valori di prima (gli sprite si toccano già)
e tenuti più grandi solo per élite (20) e boss (36): 24/24 vittorie, boss in 46–81 s. Il boss
a cupola con spine, testa e coda ricorda una tartaruga spinosa: insieme al player di D-050
richiama Bowser; segnalato all'utente, accettato.

## D-018 WebGL2 come unico backend web in P1 *(utente)*
In Bevy 0.20 la feature `webgpu` sostituisce `webgl2` nella stessa build: WebGPU eventuale
come seconda build separata dopo la P1.
