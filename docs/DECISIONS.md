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

## D-011 Sprite come griglie di testo + Python/Pillow *(integrata da D-065 per concept e illustrazioni)*
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

## D-052 Compagni dal foglio dell'utente, con i suoi nomi *(utente)*
Il primo disegno dei compagni mandato dall'utente riprendeva tratti riconoscibili di Pokémon
(Charmander, Squirtle, Pikachu): non è stato usato, nemmeno ridisegnato con i tratti cambiati
(due tentativi giudicati dall'utente troppo diversi, "sembrano delle palle"). L'utente ha poi
fornito un suo foglio di sei creature in sei pose, che non riprende personaggi precisi
(volpe di foglie, lontra con la bolla, lucertola di fuoco con creste nere, tartaruga di muschio,
fuoco fatuo, gatto con la coda a stella), ne dichiara i diritti e ne chiede la riproduzione
identica: tagliato a risoluzione nativa e ripulito, animato dalle sue pose (fermo di fronte,
cammino di profilo specchiato, di schiena in su e di fronte in giù, attacco dalla posa felice
con gli effetti). Abbinamento: Fiammetto (fuoco) = lucertola, Ondino (acqua) = lontra, Foglin
(natura) = volpe di foglie, Tuonny (elettro) = gatto; tartaruga e fuoco fatuo restano fuori
per ora. Nomi dell'utente al posto di Brasa, Gocci e Frond. Fogli 4×5 da 40×36 (campo `frame` di
`companions.ron`, `presentation::companion_anim`), ritratti del menu a figura intera 40×36,
distanze di inseguimento portate a 26–30 px e partenza delle palle di fuoco a 16 px per i
corpi più grandi. La lucertola tiene la fiamma sulla punta della coda come nel disegno
dell'utente: tratto che insieme al colore richiama ancora un personaggio noto, coperto dalla
sua decisione. Tuonny è disegnato ma entra in gioco con il suo attacco elettrico (tranche dopo).

## D-054 Riepilogo di fine run per compagno *(utente)*
Ogni colpo porta chi l'ha fatto (`Source`, indice del compagno in `companions.ron`, sul compagno
e su proiettili e onde) e passa da `combat::strike_enemy`: danno vero senza l'eccesso oltre gli
HP del nemico, il nemico finito al colpo di grazia, le cure dei talenti al compagno. I risultati
mostrano per compagno danni, quota, sconfitti, cure ed esperienza; le cure di carte e
rigenerazione hanno una riga a parte. Le statistiche stanno in `RunStats` (vettore per indice,
niente `HashMap`, D-047).

## D-055 Compagni sbloccati ai livelli 3 e 9 *(utente)*
Si parte con il solo compagno scelto nel menu; ai livelli di `progression.ron: companion_levels`
il draft propone soltanto i compagni che possono unirsi (negli altri draft le carte "nuovo
compagno" non escono più; sostituisce `companion_card_until_level`). Tre posti: uno dei quattro
resta fuori (scelta dell'utente). L'esempio dell'utente era 6/12/18, il game-designer ha
proposto 4 e 10; con la sonda Fiammetto da solo arrivava al livello 4 verso 2:00, dentro lo
sciame (1/8 vittorie): secondo compagno al 3 e sciame a 2:00, terzo al 9 (prima del muro delle
3:15). L'HUD mostra "compagno al LV n" sotto il livello.

## D-056 Run di 15 minuti con un'orda ogni 5 *(utente)*
15:00 di ondate, poi il boss. A 5:00 e 10:00 un'orda (`stages.ron: run.hordes`): 120 e 165 nemici
in 8 s, in cerchio attorno al player e fuori vista, oltre i limiti delle ondate fino a 320 vivi,
annunciata 4 s prima ("ARRIVA L'ORDA!"); un RNG suo, così gli spawn normali restano quelli di
prima. Crescita dei nemici +14% HP e +9% danno al minuto (a 15:00 ×3,1 e ×2,35). Con la sonda le
sconfitte cadono verso 3:30 e nella prima orda (27/32 vittorie, 84%).

## D-057 Monete, forzieri e reliquie *(utente)*
Per far girare il player per la mappa: alcuni nemici lasciano monete (3% gelatine e sciame,
15% corazzati, 6 sicure dalle élite) che si raccolgono come le gemme; due forzieri stanno a
600–1000 px, fuori vista, e con abbastanza monete (10, +5 a ogni apertura) ci si passa sopra e
si sceglie una reliquia su tre. Le reliquie sono upgrade di rarità `Relic` (mai nel draft dei
livelli, una volta per run, il Banchetto quando sono finite): riusano carte, icone, stack ed
effetti; un effetto nuovo, `CoinGainPct`. La Clessidra è −12% e non −20% come proposto: con 5
Respiro rapido l'onda di Ondino sarebbe durata più del suo intervallo (validazione dei limiti).
Freccia al bordo verso il forziere più vicino col prezzo, dorata se basta.

## D-058 Talenti persistenti dei compagni *(utente)*
Scelta dell'utente: i talenti restano tra una run e l'altra. Profilo RON nella cartella dei
dati dell'utente (`%APPDATA%\PixelMonsterSurvivor\profile.ron` su Windows; sul web per ora solo
in memoria, `localStorage` nella P1), niente account; le run con `--autoplay`, `--god` o
`--time-scale` non salvano. A fine run ogni compagno in squadra guadagna ⌊s in squadra ÷ 15⌋ +
⌊sconfitti ÷ 40⌋ + 20 se vittoria; per salire dal livello n servono 100 + 50 × (n − 1); un punto
per livello dal 2 al 7. Due rami da tre nodi per compagno (`companions.ron: talents`, proposta in
`docs/design/TALENTS_AND_CHESTS.md`): statistiche che si sommano alle carte e sei meccaniche
generiche (esplosione, cono ogni N attacchi, cura per colpito, eco, colpo periodico,
bruciatura); di una meccanica vale l'ultimo nodo preso del ramo. Si imparano e si azzerano
gratis dal menu (T). I compagni leggono i talenti dal profilo quando entrano in squadra. Con
tutti i talenti la sonda vinceva 32/32 con il 40% dei danni: i nodi 3 sono stati tagliati di un
terzo (guardia del game-designer: 97% e 50%). Gli effetti usano lo sprite dell'attacco del
compagno; le icone dei nodi sono del pixel-art-director.

## D-059 Alberi dei talenti grandi e schermata come il riferimento dell'utente *(utente)*
L'utente ha mandato un riferimento della schermata dei talenti e ha scelto gli alberi grandi:
tre rami per compagno (elemento, Supporto, Evoluzione), in ognuno due percorsi da tre nodi, un
nodo di unione e un finale; i nodi hanno gradi ("r/max", un punto per grado) e si aprono quando
i nodi che richiedono hanno almeno un grado (`requires`, colonna e riga nei dati). 24 nodi e 48
punti per compagno, livello massimo 49, curva 20 + 3 × (n − 1): ~3 punti dopo la prima run, metà
albero in ~15, tutto in ~50 (proposta del game-designer in `docs/design/TALENT_TREES_V2.md`, con
il tetto di forza ricavato dalla sonda di D-058). Il Supporto aiuta il player finché il compagno
è in squadra (`Player*`: HP, rigenerazione, velocità, raccolta, monete, danni subiti −%,
invulnerabilità +s, validati entro −20% e 0,15 s). Il profilo salva un id per grado: i salvataggi
vecchi restano leggibili, i nodi vecchi non esistono più e i punti tornano da spendere. Schermata:
compagni a sinistra, albero al centro con le colonne colorate, nodo scelto a destra con gli
effetti e il prossimo grado; frecce, Invio, Tab, R, clic. Del riferimento si è preso solo
l'impianto: i compagni sono i nostri, icone originali del pixel-art-director (96 nodi, 12
intestazioni). Sonda con tutti gli alberi completi: 32/32 vittorie con il 53% dei danni subiti (56% prima del primo taglio del § 4, le Evoluzioni a 0,5% di danno per grado): sopra la guardia del 97%, perché con gli alberi completi il bot non arriva più alla stretta delle 3:30; da rivedere con la prova umana.

## D-060 Compagni più piccoli del player *(utente)*
L'utente li trovava grandi quanto l'evocatore. Il pixel-art-director ha ridisegnato a mano i
quattro fogli a 28×24 (contenuto 20–22 px di altezza, circa 2/3 del player; niente riduzione
automatica, che impastava i volti) dalle pose approvate di D-052; l'utente li ha approvati
sull'anteprima. Nei dati: `frame` 28×24, partenza della palla di fuoco e del fulmine a 11 px dai
piedi (erano 16), compagni a 19–22 px dietro al player (erano 26–30). I ritratti del menu (40×36)
restano quelli grandi.

## D-069 Sistema audio solo nel client, `bevy_audio` con Vorbis e WAV *(utente)*
Progetto approvato dall'utente il 2026-10-31: `docs/design/AUDIO_SYSTEM.md`, con i crate nuovi.
- **Architettura:** `src/audio/` solo in `client_app`; `headless_app` non cambia. Tre livelli:
  - ascolto: osservatori e contatori, in sola lettura;
  - politica: `Mixer` puro, testato senza scheda audio;
  - riproduzione: l'unico punto che userà `bevy_audio`, da AUDIO-2.
- **Regole dell'audio:**
  - la simulazione non lo importa;
  - l'audio non scrive nella simulazione e ha un suo `Pcg32` (stream `0xA0D1`);
  - tempo `Time<Real>`;
  - volumi per voce gestiti da noi, perché `GlobalVolume` vale solo alla nascita del sink
    (`bevy_audio-0.20.0/src/audio_output.rs:193`).
- **Dipendenze:** feature `audio` (`bevy_audio` + `vorbis`) e `wav` di Bevy 0.20, uguali in
  nativo e sul web (nessuna feature del progetto, D-027).
  - Già nel lockfile: `bevy_audio` 0.20.0 (MIT OR Apache-2.0), `rodio` 0.22.2 (MIT OR
    Apache-2.0), `cpal` 0.17.3 (Apache-2.0).
  - Nuovi, licenze lette dai `Cargo.toml` scaricati: `lewton` 0.10.2 (MIT OR Apache-2.0),
    `ogg` 0.8.0 (BSD-3-Clause), `hound` 3.5.1 (Apache-2.0).
  - Con `ogg` la nota di copyright va riportata nei pacchetti distribuiti (P3.5).
  - Non `symphonia-vorbis`: da solo non registra il loader (`bevy_audio/src/lib.rs:97`).
  - Effetti in WAV, perché sul web la decodifica gira sul thread del gioco; musica e ambienti in
    Ogg Vorbis, perché pesano meno da scaricare.
  - Costo misurato (build web da `6cba164` senza audio, poi con AUDIO-1): `.wasm` grezzo 25,8 →
    26,4 MiB; in `dist` 23,0 → 23,5 MiB (24 068 639 → 24 622 601 byte, +541 KiB); gzip 6,1 →
    6,2 MiB; `dist` 24,8 → 25,3 MiB. Build `wasm-release` 5 min 35 s → 7 min 15 s, non a
    parità di cache. Con la riproduzione e i primi due effetti (AUDIO-2): 23,6 MiB in `dist`
    (24 706 904 byte), gzip 6,3 MiB, `dist` 25,4 MiB.
- **Regole delle voci (dopo la review di AUDIO-1):**
  - Cooldown medio, non minimo: `next = max(next + cd, now + cd − frame_dt)`, con `frame_dt`
    limitato a `[0, cd]`. Con il cooldown rigido 35 ms davano 15, 20 e 22 avvii al secondo a 30,
    60 e 144 FPS; ora 28,5, 28,6 e 28,0. Lo scarto è al massimo 2,1% da 35 a 300 ms.
  - Prezzo del cooldown medio: l'intervallo minimo è il cooldown meno un fotogramma, mai meno di
    un fotogramma; dopo un intoppo più lungo del cooldown ci possono essere due avvii in
    fotogrammi consecutivi. Sotto i 34 ms il limite vero è un avvio per fotogramma a 30 FPS.
  - Margine `sfx_headroom`: una non-`Critical` perde il boost, poi ruba voci di priorità
    minore, altrimenti si scarta. Le `Critical` partono sempre, tengono il boost e contano nella
    somma.
  - Riserva `Critical` e al massimo un furto per i posti a ogni avvio; le `Critical` non
    consumano il budget del fotogramma.
  - Nel catalogo i `cues` sono solo `Sfx` e `Ui`: musica, stinger e ambienti avranno sezioni
    proprie.
  - Si toglie per scadenza solo una voce che non ha mai suonato (`Mixer::playing`).
- **Opzioni:** `--mute` vale solo per la sessione (`session_muted`, mai nel profilo).
  `--no-audio` toglie l'`AudioPlugin` di Bevy: da AUDIO-2 nessun `load` di `AudioSource`
  senza `Assets<AudioSource>` (andrebbe in panic, `bevy_asset-0.20.0/src/server/info.rs:855`).
- **Rischi noti:**
  - un file corrotto va in panic alla riproduzione (`audio_source.rs:101`): servirà un test
    di decodifica;
  - senza dispositivo nessun sink nasce e le voci si ripuliscono da sole;
  - sul web l'audio gira sul thread principale (buffer da 2048 frame);
  - se cambia il dispositivo predefinito, Bevy non riapre l'uscita.

## D-070 Asset audio: origine, proprietario, impostazioni predefinite *(utente)*
Approvato dall'utente il 2026-10-31 con il progetto `docs/design/AUDIO_SYSTEM.md` (§ 13), con i
consigli del documento:
- **Effetti:** sintetizzati nel progetto da patch testuali in `assets-src/audio/sfx/` con uno
  script Python della sola libreria standard, come i `.px` degli sprite: suoni originali,
  riproducibili, licenza `original`. Il primo arriva in AUDIO-2.
- **Musica e ambienti:** handoff manuale come per l'arte (D-065): richiesta in
  `docs/audio/requests/<ID>.md`, file dell'utente in `.audio-inbox/<ID>/` (ignorata da git),
  integrazione solo dopo l'approvazione esplicita. Si consegnano in Ogg Vorbis: qui non c'è un
  encoder e installarne uno (`ffmpeg`, `oggenc`) richiede un via a parte.
- **Librerie CC0:** solo con il via per ogni download, con la licenza salvata accanto.
- **Licenze ammesse:** `original`, `user-owned`, `CC0-1.0`; `CC-BY-4.0` con attribuzione nei
  documenti e nei crediti. Escluse `-NC`, `-ND`, licenze senza testo chiaro, estratti di altri
  giochi, imitazioni di jingle riconoscibili, musica da servizi IA senza diritti chiari.
- **Proprietario:** il pixel-art-director diventa responsabile anche degli asset audio
  (`assets/audio/` con i valori di `catalog.ron`, `assets-src/audio/`, `docs/audio/`,
  `scripts/audio_*.py`, controlli audio del validatore, voci audio del manifest e di
  `ASSET_LICENSES.md`): ha già manifest, validatore e licenze. Il codice di `src/audio/` è del
  gameplay-engineer.
- **Predefiniti:** i loop della home non dipendono dall'interruttore "Animazioni" (li regola il
  cursore "Ambiente"); audio spento quando la finestra non è attiva. *(Loop della home ritirati:
  D-071.)*

## D-071 Musica della home approvata, niente ambienti nella home *(utente)*
Decisione dell'utente del 2026-11-02, dopo i controlli delle prime consegne:
"Apporvo soltanto la musica del menu mentre i suoni home portale e cascate non le voglio che ci
siano in generale".
- **`MUSIC-HOME-001` v1 approvata**, compreso il NO del controllo degli estremi: gli ultimi 50 ms
  sono 19,8 dB sotto il secondo prima, come la fine di ogni battuta. Va in
  `assets/audio/music/home.ogg`.
- **Canale:** suono sintetizzato da codice Python scritto con ChatGPT. Vale come via esplicito
  dell'utente per questo file. Il piano ChatGPT e la data dei termini non sono stati indicati:
  vanno aggiunti alla provenienza prima di una pubblicazione.
- **Niente ambienti nella home.**
  - Il punto 6 della richiesta iniziale (portale e cascate) è ritirato.
  - `AMB-HOME-001` passa a `REJECTED`; AUDIO-4 è annullato.
  - I file consegnati restano solo nell'inbox dell'utente, fuori dal repository.
- **Bus `Ambience`:** resta nel codice e nel profilo. Un ambiente nuovo, per esempio di uno
  stage, si fa solo su una nuova richiesta dell'utente.
- **Web:** senza ambienti i flussi Vorbis sono al massimo 2, i due deck della musica durante una
  dissolvenza. Il limite di 3 del § 3.11 di `docs/design/AUDIO_SYSTEM.md` è rispettato.
- **AUDIO-3 diviso in due:**
  - AUDIO-3 subito, con la sola traccia della home: direttore, due deck, dissolvenze. Gli stati
    senza traccia sono silenzio;
  - AUDIO-3b quando arrivano le tracce della partita: ducking, stinger, pausa della traccia
    della run.
- **Memoria del loop (misurata in AUDIO-3):** in loop Bevy usa `repeat_infinite()`
  (`bevy_audio-0.20.0/src/audio_output.rs:235`), che in rodio 0.22.2 diventa un `Buffered`.
  - Tiene tutti i campioni decodificati in `f32`: per `home.ogg` sono 24,2 MiB per deck, liberati
    quando il deck si toglie.
  - Sul web la memoria wasm non torna indietro. Con le tracce di 90–120 s una dissolvenza può
    arrivare a 60–80 MiB.
  - Da decidere in AUDIO-3b, per esempio con un `Decodable` che ridecodifica in loop senza
    buffer.

## D-068 Mappe a più biomi: tre livelli, un solo generatore, stage scelto a parte *(utente)*
Richiesta dell'utente del 2026-10-31: sei ambientazioni (Prateria Verde, Bosco Ombroso, Canyon
Arido, Cime Glaciali, Vulcano Infernale, Rovine Arcane), sempre nella visuale top-down del
gameplay; prima la struttura e il Bosco Ombroso.

- **Biomi nei dati:** `stages.ron` ha `biomes` oltre a `stages` (schema 7). Un bioma è
  l'aspetto della mappa in tre livelli separati: *terreno* (tileset opaco a colonna, tile di
  base con pesi, macchie a rumore con i bordi a maschera, muro di bordo, varianti solo
  grafiche, gruppi a macchie), *decorazioni calpestabili* (tileset con alpha 0/255 e regole
  con densità, terreno e macchie) e *ostacoli* (tipi con sprite, raggio dell'impronta e pesi
  sul terreno di base e sulle macchie). Uno stage sceglie il bioma per id e tiene densità e
  zona libera: stage diversi possono combinare lo stesso bioma con ostacoli, nemici, orde e
  boss diversi.
- **Nomi, non indici:** i tile si nominano come nel manifest; `world::biome::Biome::compile` li
  traduce in indici (i 15 bordi di un materiale si chiamano `<nome>_<lati>` come in
  `scripts/art_tileset.py`) e i pesi in soglie del tiro. `compile` è anche la validazione dei
  biomi; un test confronta nomi, misure e sprite di ogni bioma con `assets/manifest.json`.
- **Un generatore e un renderer:** `generation` e `obstacles` ricevono il bioma compilato, il
  terreno si cuoce nei pezzi da 512 px come prima (D-061) e le decorazioni si cuociono sopra il
  terreno, tranne sotto un ostacolo: nessuna entità in più, nessun costo per fotogramma. Lo
  sprite di un ostacolo viene dal bioma (`Biome` è una risorsa inserita con gli ostacoli).
- **Prateria invariata:** stessi sali, stesso rumore, soglie ricavate dai pesi in f64 (0,42 +
  0,42 + 0,09 dà esattamente le soglie di prima). Due test con impronte calcolate prima della
  modifica (tile di tutta l'arena e ostacoli, seed 1, 7 e quello di default) lo garantiscono.
- **Regole degli ostacoli uguali per tutti:** impronta circolare centrata nel tile (altezza
  fissa nel codice, non nei dati: un'altezza per tipo avrebbe spostato i centri e ridotto i
  varchi), raggio validato tra 4 e 7 px, mai due ostacoli adiacenti, due tile dal muro, zona
  libera attorno alla partenza. I test di varchi, raggiungibilità, zone di esclusione e griglia
  girano su ogni stage. Arena, limiti e prestazioni non cambiano.
- **Scelta dello stage:** `--stage <id>` (sul web `?stage=`) va in `RunSetup::stage`, che il
  loader dei dati applica a ogni caricamento (anche alla ricarica a caldo); uno stage che non
  esiste è un errore dei dati a schermo. È indipendente dal compagno: la scelta del compagno
  resta in `RunSetup::companion` e `starting_companion` dello stage è solo il default senza
  menu. Ogni stage è validato come stage giocato.
- **Bosco Ombroso (primo incremento):** stessi nemici, run e forzieri della prateria, ostacoli
  un terzo più fitti (densità 0,016: alberi, ceppi, tronchi, rocce, colonne spezzate), radure
  più larghe, foglie a macchie, funghi, felci e lastre di rovine come decorazioni. Asset
  provvisori procedurali nella pipeline; quelli definitivi con ChatGPT (ART-FOREST-001, D-065).
- **Dopo il bosco verificato, non prima:** selezione della mappa nel menu (la struttura c'è:
  `RunSetup::stage` + `GameConfig::with_stage`), effetti ambientali e pericoli del bioma (nuovi
  campi del bioma), gruppi di nemici e boss per stage (già nei campi dello stage).

## D-067 Home animata con i pixel dell'illustrazione *(utente)*
Richiesta dell'utente del 2026-10-30: la home diventa una scena viva. Stessa illustrazione,
stessa composizione, animazioni ambientali delicate, niente video al posto del fondo.

- **Come:** ogni effetto è un nodo UI figlio dell'illustrazione, messo sul suo rettangolo in
  percentuale dell'immagine. Il nodo ha un `UiMaterial` con uno shader WESL
  (`src/presentation/home_ambient/`) che ridisegna solo i pixel della sua maschera, a partire
  dai pixel dell'illustrazione stessa. Dove la maschera è zero, resta il fondo.
- **Allineamento:** lo shader legge l'illustrazione dal rettangolo in cui il fondo la disegna
  davvero (`NodeImageMode::Auto` la centra nel nodo arrotondato), con lo stesso campionatore.
  Disegnata così, l'illustrazione coincide col fondo al pixel a 1024×768, 1280×720, 1366×768 e
  1920×1080.
- **Maschere e layout dalla pipeline:** `scripts/art_home_ambient.py` li ricava
  dall'illustrazione: le zone sono scritte a mano in pixel, i pixel giusti li sceglie il colore.
  Escono `assets/ui/home_ambient/*.png` (tipi `mask` e `plate` del manifest) e
  `assets/ui/home_ambient.ron`, che il gioco incorpora e valida nei test. Nessun pixel dipinto
  nuovo. Il vortice dietro le rovine è ricostruito lungo le orbite di rotazione
  (`portal_vortex.png`), così il vortice fermo e quello che gira non si vedono mai insieme. Il
  vortice da far girare ha i colori del fondo del gioco (a 256 colori): fermo coincide al texel.
  Il texel di bordo, scuro sul profilo delle rovine, resta quello fermo del fondo.
- **Effetti:**
  - portale: vortice che gira e scende verso l'occhio (P1), bracci di luce, rune che pulsano,
    alone, particelle;
  - sei cascate: acqua che scende con due letture sfalsate, strisce, schiuma, cerchi;
  - fiume: riflessi e ondeggiare (P2);
  - fiamma di Fiammetto (P3);
  - lucciole sopra foglie ed erba (P3): stelline di pixel 5×5 che girano piano, si accendono e si
    spengono. Stanno a sinistra fra staccionata e fiori, sotto il pannello opaco e sopra le
    scritte in basso, e in basso a destra fra i ciuffi d'erba.
  Le fasi del tempo sono periodiche: nessun salto nel ciclo. Ogni effetto entra con una
  dissolvenza di 0,8 s.
- **Interruttori:**
  - Opzioni → Animazioni, salvato nel profilo (`still_home`, anche sul web);
  - `--ambient <nomi|none>` per accendere un effetto alla volta;
  - un effetto compare solo con le sue maschere caricate e l'illustrazione della misura del
    layout. Altrimenti resta il fondo fermo.
- **Costo:** 12 PNG (11 maschere e il vortice, ≈ 120 KiB nel build), ≈ 2 MiB di texture,
  11 nodi. Tempo reale
  (`Time<Real>`), solo in `AppState::MainMenu`. Nessuna dipendenza nuova.
- **Non animati:** player e compagni (servono sprite e approvazione). Nuvole e fronde servono
  livelli separati: proposta ART-HOME-002 in `docs/art/requests/HOME_AMBIENT_ART_REQUEST.md`,
  non richiesta.
- **Se l'illustrazione cambia:** si riscrivono le zone dello script e si rigenera. Fino ad allora
  le animazioni restano spente da sole.

## D-066 Selezione del compagno iniziale e compagni da sbloccare *(utente)*
Richiesta dell'utente del 2026-10-30, con un mockup fatto con ChatGPT (solo reference: la
schermata è fatta di nodi UI veri). Il flusso diventa Nuova Partita → selezione → Inizia
Partita → run; Indietro (o Esc) torna alla pagina iniziale senza run. La selezione prende il
posto della schermata "Compagni" (D-040): la voce "Compagni" e la riga "Parti con …" della
pagina iniziale (D-063) spariscono, perché ripeterebbero la stessa scelta. Il gioco non ha
"Continua" (D-063), quindi non c'è nulla da mantenere.

La schermata ha la griglia dei compagni a sinistra e i dettagli a destra. Al centro non c'è
nulla, si vede solo lo sfondo: l'anteprima animata del mockup è stata provata e l'utente l'ha
fatta togliere ("è orrendo quel fiammetto che stai mettendo al centro").

Sblocchi:
- ogni compagno ha nei dati una regola `unlock`: `Start`, disponibile su un profilo nuovo, o
  `Locked`, non ancora ottenibile. Le condizioni future saranno regole nuove, valutate in un
  solo posto; la UI legge solo lo stato. Per ora solo Fiammetto è `Start`, e niente missioni,
  costi o obiettivi;
- il profilo salva in `unlocked` i compagni sbloccati da una condizione, oggi nessuno. Un
  profilo vecchio si legge senza perdere esperienza, talenti e opzioni, e ha anche lui solo
  Fiammetto: niente diritti acquisiti per chi ha già giocato con gli altri;
- l'avvio della run rifiuta un compagno bloccato anche se la UI lo lasciasse passare;
- i compagni che si uniscono durante la run (D-055) non diventano compagni iniziali;
- il compagno iniziale dello stage deve essere `Start` (validazione dei dati).

I testi vengono dai file di gioco: la descrizione del compagno da `companions.ron`, il nome e la
descrizione dell'attacco da `attacks.ron`. Le statistiche vengono dall'attacco: danno, cadenza,
portata. I compagni non hanno punti salute, quindi la "Salute" del mockup non c'è. L'elemento
Electric si chiama "Fulmine" a schermo. Nessun asset nuovo: ritratti, fogli animati, icone e
lucchetto ci sono già.

## D-065 Arte con ChatGPT solo con handoff manuale *(utente)*
L'utente ha autorizzato il 2026-10-30 un solo canale per l'arte fatta con l'IA: la sua
conversazione ChatGPT, a mano. Niente API di OpenAI, Codex, MCP remoti, automazione o lettura
della chat, accesso all'account, generatori a pagamento. Per ogni richiesta grafica il
pixel-art-director sceglie una via:
- `LOCAL_PIXEL`: tile, icone, ritocchi, palette. Pipeline `.px`, come sempre.
- `CHATGPT_CONCEPT`: illustrazioni e sfondi, design di compagni o boss nuovi, ritratti
  elaborati, grandi decorazioni dell'interfaccia.
- `CHATGPT_EDIT`: modifica di un concept già approvato, a partire dalla sua immagine.

Per le due vie ChatGPT scrive `docs/art/requests/<ID>.md` con un prompt che si regge da solo.
L'utente lo incolla, allega le reference indicate, genera, sceglie e salva il PNG in
`.art-inbox/<ID>/`, ignorata da git. Poi l'agente controlla il file vero e lo porta nella
pipeline solo dopo un'approvazione esplicita dell'utente. Procedura:
`docs/art/CHATGPT_ART_WORKFLOW.md`.

Integra D-011 solo per i concept e le illustrazioni approvate. Il resto non cambia:
- sprite, tile e icone hanno come sorgente solo la griglia `.px`; un'immagine di ChatGPT per
  uno sprite è una reference da ridisegnare, mai un foglio animato pronto;
- palette, pixel-perfect, font, manifest, validatori e licenze restano quelli di prima;
- niente elementi riconoscibili di franchise altrui.

Provenienza: richiesta, versione scelta, data e approvazione vanno nella richiesta e in
`ASSET_LICENSES.md`. I diritti sull'immagine li dichiara l'utente, che usa ChatGPT con il suo
account e i termini di OpenAI che ha accettato (come le sue immagini di D-050, D-052, D-063).

Primo uso: ART-HOME-001, lo sfondo della home nativo 16:9 senza logo né menu (1672×941, v1
approvata dall'utente il 2026-10-30). Ha preso il posto del concept 4:3 di D-063, che resta
nella storia di git.

## D-064 Repository su GitHub e CI con GitHub Actions *(utente)*
L'utente ha chiesto il push su https://github.com/pandemonyum/bonk (pubblico) e poi la CI. La
CI gira su Linux (runner gratuiti e più veloci di quelli Windows per un repository pubblico) con
lo stesso gate di `scripts/check.sh` e la build web, che lascia `dist/` come artefatto: nessuna
pubblicazione automatica, che resta da decidere con l'utente (GitHub Pages, itch.io). Le
versioni di toolchain e wasm-bindgen vengono da `rust-toolchain.toml` e `Cargo.lock`, così la
CI non va aggiornata a mano quando cambiano. I commit portano l'email dell'autore, visibile nel
repository pubblico: segnalato all'utente.

## D-063 Pagina iniziale come l'illustrazione dell'utente; il gioco si chiama Companion Survivors *(utente)*
L'utente ha mandato un'illustrazione della pagina iniziale (paesaggio con il suo player di
spalle e tre dei suoi compagni, logo in alto a sinistra, pannello scuro con le voci) e poi la
stessa senza logo e menu da usare come sfondo. È sua, come i fogli di D-050/D-052: sta in
`assets-src/illustrations/` e non segue palette né griglia dei pixel (filtro lineare; copre la
finestra senza deformarsi e il taglio tiene player e compagni). Logo, icone, freccia e fregi
del pannello sono originali, disegnati nella pipeline. Il menu diventa una pagina iniziale:
Nuova partita (con il compagno scelto, scritto in fondo al pannello: lo cambiano anche i
talenti con Tab), Compagni (la scelta di prima, con Indietro), Talenti, Opzioni ed Esci (non nel
browser). Lo schermo intero delle Opzioni sul desktop si salva nel profilo e vale dall'avvio,
tranne nelle run automatiche (screenshot, uscita a tempo, profilo in sola lettura); nel browser
vale per la visita, lo chiede il gioco alla pagina (solo dopo un clic o un tasto veri) e il
valore mostrato è lo stato vero della pagina, che ne esce anche da sola con Esc. Niente "Continua"
finché non si può salvare una run a metà (scelta dell'utente). Nome mostrato: Companion
Survivors (logo, finestra, pagina web, documenti); restano il nome del crate e
dell'eseguibile, la cartella del profilo e la chiave del `localStorage`, così i progressi già
salvati non si perdono.

## D-062 Più mappa invece delle bande nere *(utente)*
Con una finestra che non è un multiplo esatto di 480×270 il canvas resta alla stessa scala
intera ma cresce fino a coprirla, fino a 640×360 pixel di mondo (1280×720 è tutta piena;
oltre il limite restano bande sottili). Per la simulazione "in vista" è sempre il rettangolo
più grande, con ogni finestra: spawn, orde e forzieri non nascono mai sotto gli occhi e una
seed dà la stessa run su ogni schermo; la camera si ferma ai bordi dell'arena con la stessa
misura. Gli indicatori al bordo (boss, forzieri) usano la parte davvero a schermo. L'anello di
spawn passa da (290, 360) a (260, 400) (con (300, 420) i nemici arrivavano tardi e la sonda
vinceva il 94%), più una fascia di riserva di 160 px oltre l'anello, per gli spawn e per le
orde: negli angoli e ai lati la vista grande copriva parte dei punti (l'angolo diventava un
riparo, le orde nascevano all'87%). Sonda dopo: 27/32 (84%) e danni medi 173, come prima (27/32,
177). Sul web asset e dati stanno
in cartelle con la versione del loro contenuto (`assets-<v>`, `game-data-<v>`, scelta dagli
script e passata al gioco in `PMS_WEB_VERSION`): nella prova il browser aveva preso dalla cache
un `stages.ron` vecchio e il gioco si fermava sull'errore dei dati.

## D-061 Build web: wasm-bindgen, pagina propria, terreno cotto in sprite
P1 chiesta dall'utente. Niente Trunk: `wasm-bindgen-cli` alla versione del `Cargo.lock`
(0.2.129, gli script si fermano se non coincide) e una pagina scritta a mano
(`web/index.html`: "Clicca per giocare", avanzamento del download, messaggio senza WebGL2,
errori a schermo). Il gioco parte nel canvas `#game` che riempie la finestra, i tasti vanno al
gioco; le opzioni di avvio passano dall'indirizzo (`?autoplay&seed=3`); il profilo sta nel
`localStorage` (stesso RON del file). Due problemi trovati solo nel browser: le richieste dei
`.meta` di ogni asset (404: `AssetMetaCheck::Never`) e il terreno nero, perché il
`TilemapChunk` di Bevy usa una texture di indici `Rgba16Uint` che WebGL2 rifiuta; il terreno è
ora cotto sulla CPU in pezzi da 512 px dal tileset e mostrato come sprite, solo attorno alla
camera, uguale su desktop (stesso disegno, verificato al pixel) e sul web. Dimensioni: il primo
`.wasm` pesava 54 MiB (20 di nomi delle funzioni): con `strip = "symbols"`, `opt-level = "z"`
e senza le sezioni dei nomi 22,5 MiB in `dist`, 5,9 MiB compressi, sotto il limite di 25 MiB per
file. `wasm-opt` non è installato (gli script lo usano se c'è). La CI aspetta un remote GitHub.

## D-053 Tuonny e il fulmine a catena
Quarto compagno (elemento nuovo `Electric`, gatto con la coda a stella del foglio dell'utente,
D-052) con un pattern nuovo `Chain`: colpisce il nemico vivo più vicino entro `range`, poi salta
al più vicino non ancora colpito entro `jump_range` dall'ultimo, fino a `jumps` volte, con il
danno × `falloff` a ogni salto; istantaneo (nessun proiettile in volo), lascia per 0,15 s un
`ChainBolt` che la presentazione disegna come scintille a zigzag; ogni colpito prende una scossa
(`slow`, `slow_s`, la stessa regola di combinazione dell'onda). Gli upgrade di danno, cadenza e
portata valgono anche qui (la portata allunga anche i salti) e "+1 proiettile" dà un salto in
più. Con quattro compagni e tre posti la squadra non li ha più tutti: la sonda ha mostrato che
quella senza Ondino (Foglin + Fiammetto + Tuonny) perdeva 8 run su 9 verso il minuto 4 con la
prima taratura (10 danni ogni 1,6 s, 3 salti da 60 px, nessuna scossa); con 12 ogni 1,3 s, 5
salti da 70 px e la scossa (60% per 0,8 s) vince 6 run su 10 e perde più tardi (minuti 6–9),
mentre ogni squadra con Ondino vince. Menu a quattro carte (tasti 1–4), carta "Nuovo compagno:
Tuonny".

## D-018 WebGL2 come unico backend web in P1 *(utente)*
In Bevy 0.20 la feature `webgpu` sostituisce `webgl2` nella stessa build: WebGPU eventuale
come seconda build separata dopo la P1.
