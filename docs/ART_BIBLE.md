# ART BIBLE — v1 (art pass "16-bit")

Direzione: survivor top-down luminoso e amichevole in **pixel art stile 16-bit**, ricca ma
leggibile in massa. Riferimento di stile: l'immagine portata dall'utente il 2026-10-23 (solo
**stile**: densità di dettaglio, rampe di colore, contorni, prato, layout dell'HUD e delle
carte). **Mai** riprodurne i contenuti: niente creature, allenatore col cappellino, edifici,
sfere, nomi o UI di Pokémon o di altri giochi. La moodboard non è nel repository e non va
ritagliata. Eccezione: il player è il personaggio fornito dall'utente, che ne dichiara i
diritti e accetta il rischio di somiglianza (sezione "Player", D-050).

Anteprime di questa versione (composte da `scripts/preview-scene.py`, x3; il foglio del player
x4 in `art-preview-player-sheet.png`):
`docs/evidence/art-preview-scene.png` (gioco + HUD), `-draft.png` (carte), `-select.png`
(scelta del compagno), `-boss.png` (boss, élite, segni di pericolo, barra del boss),
`-tiles.png` (mappa di prova e tileset). Prova nel gioco: `docs/evidence/art-v1-gioco.png`
(finestra 1101×701, scala 2×, `check-pixel-grid.py` 100% di blocchi uniformi; HUD e draft
sono ancora quelli del codice attuale, le icone nuove non sono collegate).

## Regole dello stile v1

1. **Rampe di 3–4 toni per materiale** (riflesso, luce, base, ombra, scuro), luce sempre da
   **nord-ovest**: lato sinistro e alto chiari, destro e basso in ombra.
2. **Contorno scuro con luce selettiva**: gli attori hanno il contorno `k`; dove serve calore
   il contorno è il tono scuro del materiale (fuoco `x`). Le gocce dell'onda restano senza
   contorno (D-037).
3. **Colori saturi e caldi sugli attori, ambiente più spento**: l'erba è un verde medio, gli
   alleati e i nemici sono più saturi e hanno il contorno, così si staccano dal fondo.
4. **Forma prima del colore**: alleati e nemici differiscono per sagoma (lezione P0.2). Nemici
   tondi o a cupola, viola con dettagli rossi; alleati a goccia, bolla, seme, mantello.
5. **Creature tonde ed espressive**: occhi grandi con riflesso, gote, sopracciglia; in un'orda
   di 200 si legge ancora "chi è chi".
6. **Ombra a terra** (`9`, verde oliva scuro) cotta negli sprite di player, compagni, élite,
   boss, alberi e rocce: li "appoggia" al prato e aiuta a ritrovare il player nell'orda. I
   nemici comuni non l'hanno, per non appesantire l'orda; lo sciame vola.
7. Pixel-perfect (D-007): 1 texel = 1 pixel di mondo, niente rotazioni o scale degli sprite,
   larghezza pari; i centrati (proiettili, gocce, foglie, segni) anche altezza pari.

## Griglia e dimensioni

| Elemento | v0 | v1 | Motivo (D-012) |
|---|---|---|---|
| Risoluzione virtuale | 480×270 | = | upscaling intero nearest, letterbox |
| Tile | 16×16 | = | tileset a colonna (array texture), 42 tile |
| Evocatore (player) | 16×21 | **foglio 144×304, fotogrammi 36×38** | 4×8 fotogrammi animati, personaggio dell'utente (D-050); in piedi 22–23×30 visibili più l'ombra, centrato tra le colonne 17 e 18; ingombro e limiti dell'arena sono del codice |
| Compagni (Fiammetto, Ondino, Foglin, Tuonny) | fogli 160×180 da 40×36 (D-052, grandi come il player) | **fogli 112×120, fotogrammi 28×24** | personaggi dell'utente ridisegnati a 2/3 del player (D-060): 20–22 px visibili, al massimo 24 di larghezza |
| Scintilla del fulmine | — | **6×6** | `spark.png`, ancora al centro, senza contorno, ripetuta lungo il fulmine di Tuonny |
| Gelatina, foglio | — | **28×24** (~22×18 visibili), 4×2 | stile del player; salto e sconfitta |
| Sciame, foglio | — | **24×22** (~18×16), 4×2 | vola: ombra piccola sulla riga in basso |
| Corazzato, foglio | — | **32×28** (~26×22), 4×2 | passo e guscio che si spacca |
| Gelatina Gigante, foglio | — | **44×38** (~36×30), 4×4 | passo, sconfitta, carica, scatto |
| Gran Corazzato, foglio | — | **96×80** (~80×66), 4×4 | passo, sconfitta, salto, schianto; ~1/4 dell'altezza dello schermo |
| Proiettile di fuoco | 6×6 | **8×8** | sfera con nucleo bianco, visibile sopra l'orda |
| Foglia orbitante | 6×6 | **8×8** | nervatura e contorno |
| Goccia dell'onda | 4×4 | = | più chiara (`L`/`w`), senza contorno |
| Gemma XP | 6×8 | **8×10** | cristallo sfaccettato con riflesso |
| Moneta | — | **10×10**, foglio 4×1 | oro caldo, tonda e piatta (la gemma XP è blu e a punta); gira su se stessa |
| Forziere, foglio | — | **24×22** (22×18 visibili chiuso), 4×2 | legno scuro e oro; largo quasi come il player (22 contro 23), più basso (18 contro 30) |
| Segno di pericolo | — | **6×6** | anello rosso a terra |
| Roccia | 16×12 | **20×16** | masso tozzo nello stile del player; impronta r=7 a 4 px sopra la base, sulla parte bassa del masso |
| Albero | 32×32 | **40×48** | chioma a ciuffi separati per tono, contorno solo sulla sagoma; impronta r=4 a 4 px sopra la base, sul piede del tronco |

Massimi: comuni ≤ 20 px, fotogramma dei compagni 28×24, fotogramma del player 36×38, alberi ≤ 32, boss ≤ 64
(validato; i fogli animati fotogramma per fotogramma, fino a 96 px per fotogramma). A
480×270 una creatura comune resta sotto il 6% dell'altezza dello schermo: le orde restano
leggibili. Verificato sull'anteprima con 75 nemici; lo screenshot del gioco (seed 20, 1:45 di
gioco) ha un'orda piccola: la prova con 200 nemici nel gioco vero resta da fare.

I valori di gioco che dipendono dalle dimensioni (hurtbox, altezza di sparo e del centro
dell'onda, impronte degli ostacoli) sono proposti in `docs/art/STYLE_HANDOFF.md`.

## Palette "Germoglio" v1 (`assets-src/palette.txt`, 56 colori)

I 31 caratteri della v0 mantengono il ruolo (alcuni ritoccati di tono); i nuovi completano le
rampe. Ogni rampa va dal riflesso allo scuro.

| Ruolo | Rampa |
|---|---|
| Contorno e neutri | `k` contorno, `K` ombra neutra, `w` bianco, `9` ombra a terra |
| Pelle | `s` `S` |
| Alleato teal (icone e carte dell'acqua) | `L` `l` `c` `C` `n` |
| Oro (carta selezionata, icone) | `y` `g` `G` `h` |
| Fuoco alleato (icone e carte del fuoco, palla di fuoco) | `y` nucleo, `f` `o` `O` `x` — arancio, mai il rosso dei pericoli |
| Natura alleato (icone e carte della natura, foglie orbitanti) | `i` `e` `E` `v` — più chiaro e giallo dell'erba |
| Nemici | `Q` `q` `p` `P` `z` viola |
| Pericoli e danni | `X` `r` `R` `j` rosso (anche cuori HP e barra del boss nella UI) |
| Gemme XP | `W` `u` `U` `V` blu |
| Erba e siepe | `b` `a` `A` `B` `N` |
| Terra e legno | `7` `d` `D` `8` |
| Pietra e osso | `6` `t` `T` `5` |
| Fiori | `y` giallo, `m` `M` rosa, `w` bianco (niente fiori viola, rossi o blu: sono colori di ruolo) |
| Interfaccia | `0` `1` `2` blu notte dei pannelli, `3` `4` bordi neutri |

Feedback di colpo: lo sprite viene moltiplicato per (1, 0.45, 0.45) per 0,12 s (codice di
presentazione); con l'ombra cotta anche l'ombra si arrossa per quel tempo, effetto accettato.

## Player: foglio animato `sprites/summoner_sheet.png` (D-050)

Il personaggio è quello del foglio di animazioni generato e fornito dall'utente, che ne
dichiara i diritti e ne ha chiesto la riproduzione identica. Il rischio di somiglianza con
personaggi Nintendo è accettato dall'utente (D-050). Il personaggio ha cappello rosso con
stemma bianco, capelli neri a ciuffi, occhi grandi, maniche corte rosse, salopette blu con
bretelle, guanti e pugni grandi, scarpe marroni.

Sorgente: `assets-src/sprites/summoner_sheet.px`, con la **palette propria**
`summoner_sheet.palette.txt`:
- 24 colori dal foglio dell'utente, dove ogni gruppo di sfumature quasi identiche del
  generatore è unito nel suo colore più usato;
- l'ombra a terra `9` del progetto.

Il problema del foglio originale era la misura: ogni fotogramma era disegnato a una scala un
po' diversa (le schiene e i profili fino al 17% più grandi del fronte) e le gambe del cammino
non si alternavano in modo chiaro. Regole del ridisegno:

- **Un modello per direzione.** Il fermo 0 di fronte, di schiena e di profilo è il master; ogni
  altro fotogramma della stessa direzione ne deriva, e testa, busto e braccia non cambiano
  mai misura né forma. I tre master hanno la stessa altezza (cappello dalla riga 7, suole
  sulla riga 36). Schiena e profilo sono stati ridotti togliendo righe uniformi (cupola del
  cappello, capelli), senza ricampionare i dettagli, e ripuliti: contorno unico `k`, pixel
  isolati del generatore tolti, viso e occhi intatti.
- **Ombra fissa.** L'ombra a terra è quella del master in tutti i fotogrammi in piedi, più
  l'impronta della scarpa alzata; non si sposta con i piedi.
- **Pose dell'utente.**
  - Colpito 1–2 e sconfitto 0–3 sono i suoi fotogrammi, ripuliti. Erano già alla scala del
    master, tranne il colpito 1, ridotto all'altezza del master.
  - Colpito 0 e 3 derivano dal master.

**Contratto con il codice** (`presentation::player_anim`):

| Riga | Animazione | Fotogrammi |
|---|---|---|
| 0 | fermo giù (di fronte) | 0 master, 1–2 respiro (testa e spalle giù di 1 px, braccia e gambe ferme), 3 battito di ciglia |
| 1 | fermo su (di schiena) | come la riga 0; il 3 è uguale allo 0 |
| 2 | fermo a destra | come la riga 0, battito di ciglia sul 3; la sinistra è questa riga specchiata dal gioco |
| 3 | cammino giù | 0 contatto: piede sinistro a terra spostato di 1 px verso il centro, destro alzato di 3 px, pugno destro alzato di 2 px, braccio sinistro indietro di 1 px; 1 passaggio: piedi sotto il corpo, corpo su di 1 px (gambe una riga più lunghe); 2 contatto opposto; 3 passaggio |
| 4 | cammino su | come la riga 3 di schiena |
| 5 | cammino a destra | 0 contatto (passo lungo, gamba vicina avanti, braccio vicino indietro), 1 passaggio (gambe incrociate, gamba lontana piegata dietro, corpo su di 1 px), 2 contatto opposto (braccio vicino avanti), 3 passaggio (gamba vicina piegata dietro) |
| 6 | colpito | 0 spavento (pugni su, bocca aperta), 1 contraccolpo di tre quarti, 2 si raggomitola, 3 di nuovo in piedi |
| 7 | sconfitto | 0 stordito, 1 si accascia, 2 carponi, 3 a terra a pancia in giù (resta a schermo) |

Regole del foglio:
- **Misure:** fotogrammi 36×38, 4 colonne, 8 righe (144×304).
- **Base:** riga 37 = linea di terra con l'ombra, suole e corpo a terra sulla riga 36; ogni
  fotogramma tocca la riga di base (controllato da `validate-assets.py`).
- **Centro:** ancora in basso al centro, corpo centrato tra le colonne 17 e 18 (piedi del
  fronte e della schiena sulle colonne 11–24/25; il master del fronte è largo 23, quindi
  mezzo pixel di scarto è del disegno). Il cappello parte dalla riga 7, dalla 6 nei passaggi.
- **Colori e alpha:** solo i colori della palette propria (controllati da
  `validate-assets.py`, che legge `palette` dal manifest), alpha 0 o 255.
- **Tempi del codice:** fermo 3 fotogrammi/s, cammino 8/s, colpito 0,32 s, caduta 0,7 s. Il
  colpito finisce in piedi perché subito dopo il gioco riprende fermo o cammino.

Formato del sorgente (sezione "Pipeline"):
- intestazione con `frame: 36x38`, `columns: 4`, `rows: <nomi>`, `anchor: bottom_center`,
  `palette: summoner_sheet.palette.txt` e `license:`;
- poi un blocco `@ <riga> <colonna>` di 38 righe per fotogramma;
- `|` a metà riga (dopo la colonna 17) è la guida del centro, `#` è un commento.

Anteprime di controllo (non nel repository): `screenshots/anteprima-v2-animazioni.gif`,
`-foglio.png`, `-confronto.png`. Nel repository c'è `docs/evidence/art-preview-player-sheet.png`.

## Nemici e boss: fogli animati

Un foglio per nemico (`assets-src/sprites/<nome>_sheet.px`), nello stile del player:
- contorno quasi nero `k` (060102) e rampe di 5 toni con luce da nord-ovest;
- forme tonde e tozze, viola con dettagli rossi;
- palette condivisa `enemies.palette.txt`.

Sono disegnati verso **destra** (il gioco specchia per la sinistra). In ogni fotogramma l'ombra
è sulla riga in basso e il corpo è centrato tra le due colonne di mezzo. È lo stesso modello in
ogni fotogramma: le deformazioni sono solo lo schiacciamento voluto delle gelatine. Riga 0
cammino o volo (4 fotogrammi), riga 1 sconfitta, che finisce con un resto (pozza, macchia,
guscio rotto) che il gioco fa sparire in dissolvenza.

| Foglio | Fotogramma | Righe |
|---|---|---|
| `blob_sheet.png` | 28×24 | move (salto: carica, stacco, in aria), death |
| `swarm_sheet.png` | 24×22 | move (battito d'ali, corpo su e giù di 1 px), death |
| `armored_sheet.png` | 32×28 | move (zampe alternate), death |
| `elite_sheet.png` | 44×38 | move, death, charge (si comprime, alone rosso, pronta: il 3 resta), dash (allungata, linee di velocità nel fotogramma) |
| `boss_sheet.png` | 96×80 | move (passo pesante), death, windup (salto: il 3 resta fino all'impatto), slam (schianto, polvere nel fotogramma, ripresa) |

Disegnati con forme ombreggiate in uno script usa e getta fuori dal repository e rifiniti
nella griglia, come gli sprite grandi della v1. I vecchi sprite singoli dei nemici
(`blob_enemy.png` e gli altri) sono stati rimossi quando `game-data/enemies.ron` è passato ai fogli.

## Compagni: fogli animati (D-052)

I quattro compagni sono i personaggi del foglio fornito dall'utente, che ne dichiara i diritti e
ne ha chiesto la riproduzione identica (D-052, stesso trattamento del player).

Origine:
- Il foglio originale (6 creature × 6 pose: di fronte, di profilo a destra, di schiena, di
  profilo a sinistra, felice con gli effetti, sdraiata) è stato tagliato a risoluzione nativa e
  ripulito. I colori quasi identici del generatore sono uniti in 36 colori dell'utente
  (`assets-src/sprites/companions.palette.txt`).
- Abbinamento: `ember` Fiammetto = lucertola di fuoco, `tide` Ondino = lontra delle bolle,
  `sprout` Foglin = volpe delle foglie, `spark` Tuonny = gatto con la coda a stella. La
  tartaruga e lo spirito del foglio non sono usati e non sono nel repository.

**Contratto** (`sprites/<id>_sheet.png`):
- fotogrammi 40×36, 4 colonne, 5 righe (160×180);
- piedi e ombra sulla riga 35 (il taglio era alto 35: una riga vuota in cima);
- centro tra le colonne 19 e 20 (guida `|`), ancora in basso al centro.

| Riga | Nome | Fotogrammi |
|---|---|---|
| 0 | `idle` | 0 master (posa di fronte dell'utente), 1–2 respiro (testa e spalle giù di 1 px), 3 battito di ciglia |
| 1 | `move_right` | master = posa di profilo a destra (il gioco specchia per la sinistra): 0 passo aperto (gruppi di zampe a ±2 px), 1 passaggio (corpo su di 1 px, piede dietro alzato di 2, coda o bolla su di 1), 2 piedi incrociati, 3 passaggio (piede davanti alzato, coda giù) |
| 2 | `attack` | 0 carica dalla posa di fronte (giù di 1 px), 1 posa felice con la metà degli effetti più vicina al corpo, 2 posa felice con tutti gli effetti, 3 di nuovo di fronte |
| 3 | `move_up` | master = posa di schiena: piedi alternati alzati di 2 px, passaggi con il corpo su di 1 px |
| 4 | `move_down` | come la riga 3 con la posa di fronte |

Regole:
- **Master:** le pose dell'utente restano identiche (solo pulizia), e ogni altro fotogramma ne
  deriva alla stessa scala.
- **Ombra:** quella a terra viene rimessa dal master dopo ogni spostamento dei piedi.
- **Fiammetto:** le fiamme sono attaccate al corpo, quindi lancio ed effetto pieno differiscono
  per poche scintille e il passaggio dalla carica al lancio fa da colpo.

Altri file:
- **Ritratti** `ui/portrait_<id>.png`: 40×36, la posa di fronte senza l'ombra a terra (figura
  intera, come le carte dell'utente). Il limite delle icone UI nel validatore è 40.
- **Carte** `ui/upgrade_join_<id>.png`: 16×16, cornice del tema dell'elemento con una testina del
  nuovo disegno, disegnata nel progetto con la palette del progetto.
- **Scintilla** `sprites/spark.png`: 6×6, VFX del fulmine a catena di Tuonny. Nucleo bianco `w`,
  bordo giallo `y` e azzurro `W`, senza contorno come le gocce dell'onda.

Scala: alla scala del foglio dell'utente i compagni sono alti ~30 px visibili, quanto il player
(~30) e più dei nemici comuni. È voluto (stessa risoluzione del suo foglio); se in gioco
coprono troppo, la scelta è del codice (distanza dei posti in `player.ron`), non del disegno.

### Versione piccola (in gioco dal 2026-10-28, D-060)

Richiesta dell'utente: compagni a circa 2/3 dell'altezza del player. Approvata dall'utente
sull'anteprima; i fogli `assets-src/sprites/<id>_sheet.px` sono ora quelli da 28×24 (i 40×36
restano nella storia di git). Prova nel gioco: `docs/evidence/compagni-piccoli-gioco.png`.

File:
- Anteprime ×4 in `screenshots/` (fuori dal repository):
  - `anteprima-compagni-piccoli-squadra.png`: la squadra dietro al player;
  - `anteprima-compagni-attuali-squadra.png`: la stessa scena con i fogli attuali;
  - `anteprima-compagni-piccoli-<id>.png`: i 20 fotogrammi di ciascuno.

**Contratto:** fotogrammi 28×24, stesse 4 colonne e 5 righe (112×120), stesso ordine delle
righe; piedi e ombra sulla riga 23, centro tra le colonne 13 e 14 (guida `|`).

**Come sono fatti:**
- I quattro master (di fronte, profilo, schiena, felice) sono ridisegnati a mano dalle pose
  approvate, con contorno di 1 px rifatto.
- Una riduzione automatica (colore dominante per area) dava facce sporche e scintille nere:
  scartata.
- Gli altri fotogrammi derivano dai master come nei fogli grandi, con passi di 1 px invece
  di 2: piede alzato di 1, gruppi di zampe a ±1 nel passo aperto, terzo fotogramma del
  profilo in appoggio.

Misure (contenuto senza l'ombra, master di fronte):

| Compagno | Di fronte | Profilo | Schiena | Felice |
|---|---|---|---|---|
| Fiammetto | 24×22 | 21×21 | 20×22 | 23×22 |
| Ondino | 23×20 | 20×20 | 22×20 | 24×22 |
| Foglin | 24×22 | 21×21 | 21×22 | 23×22 |
| Tuonny | 23×20 | 15×22 | 22×21 | 24×22 |

Il player è 23×30, quindi i master stanno tra 2/3 (20) e 3/4 (22). Nei fotogrammi di
passaggio il corpo sale di 1 px (fino a 23 di altezza) e nel respiro scende di 1 (fino a 19).
Gli effetti della posa felice (bolle, scintille) restano dentro 24×22.

## Tileset `meadow_tileset.png` (indici in `src/world/tiles.rs`)

Procedurale con seed fisso in `scripts/art_tileset.py` (chiamato da `scripts/build-assets.py`),
timbri che si avvolgono sui bordi del tile (nessuna cucitura). Ridisegnato nello stile del
player con gli **stessi 42 indici e ruoli** (`TILE_NAMES` e `src/world/` invariati):
- **Erba:** ciuffi tozzi a tre e due fili (punte `b`, base `A`), zolle in ombra e macchie di
  sole tonde.
- **Fiori:** più grandi, con un bordo verde scuro e l'ombra a terra.
- **Terra:** crepe, granelli chiari e sassolini.
- **Siepe:** cespugli tondi a due file con bordo `N`. Dove la siepe confina con l'erba il
  contorno è il quasi nero `k` degli attori, con un bordo di luce dentro a nord e a ovest:
  la siepe è il muro dell'arena e si legge come un ostacolo.

L'ambiente resta più spento degli attori (regola 3): il contorno `k` sta solo sulla siepe, mai
sui dettagli del prato.

| Indici | Tile |
|---|---|
| 0–6 | `grass_a`, `grass_b`, `grass_tuft`, `flowers_white`, `flowers_pink`, `dirt`, `hedge` (v0, ridisegnati) |
| 7–11 | `grass_c` (macchie di sole), `grass_clover`, `flowers_yellow`, `grass_pebbles`, `grass_tall` |
| 12–26 | bordi terra/erba `dirt_<lati>`, indice = 12 + maschera − 1 |
| 27–41 | bordi siepe/erba `hedge_<lati>`, indice = 27 + maschera − 1 |

Maschera: N=1, E=2, S=4, O=8, bit acceso se il vicino è di un altro materiale (nord = riga in
alto del tile = y + 1 nell'arena). Nei bordi l'erba entra di 2–4 px con un profilo irregolare
che vale 2 px agli estremi, così due bordi affiancati si raccordano; angoli esterni arrotondati.
Terra: parete in ombra (`8`, `D`) sotto l'erba a nord e a ovest, punte chiare dell'erba verso
sud. Siepe: contorno scuro dei cespugli e ombra portata sull'erba a sud e a est.

Le varianti sono **solo grafiche** (`visual_tile_at` in `src/world/generation.rs`): `tile_at`,
gli ostacoli e il gameplay vedono sempre erba, terra o siepe. Niente acqua né scogliere: non
cambiano il passaggio. I fiori crescono a macchie (rumore a celle di 6 tile).

## Monete e forzieri (palette del progetto)

Anteprima ×4 su erba e su terra, accanto al player e alla gemma XP:
`screenshots/anteprima-monete-forziere.png`.

**Moneta** `sprites/coin.png`: foglio 4×1 da 10×10, riga `spin`, senza ancora (pickup centrata
come la gemma XP).
- Fotogrammi: 0 faccia, 1 tre quarti (bordo visibile a destra), 2 di taglio, 3 tre quarti
  dall'altro lato (bordo a sinistra, in luce).
- Colori: oro `y g G h` con riflesso `w` in alto a sinistra e contorno `k`; un seme inciso al
  centro.
- Lettura: tonda, piatta e calda, mentre la gemma XP è un cristallo blu a punta. Non si
  confondono nemmeno a 1×.

**Forziere** `sprites/chest_sheet.png`: foglio 4×2 da 24×22, ancora in basso al centro.
- Ingombro: largo 22 px (x 1–22), alto 18 px chiuso (righe 3–20) più l'ombra a terra sulla
  riga 21.
- Riga `idle`, chiuso con il luccichio che gira: 0 fermo, 1 riflesso sulla cinghia sinistra,
  2 stella sulla serratura, 3 stellina oltre l'angolo del coperchio.
- Riga `open`: 0 coperchio su di 1 px con la luce nella fessura, 1 coperchio alzato e
  inclinato, 2 aperto all'indietro (si vede l'interno), 3 aperto con il bagliore (raggi `w`/`y`,
  interno illuminato, scintille ai lati).
- Materiali:
  - legno scuro (`D`, `8`) sul fronte, coperchio più chiaro (`7`, `d`) per la luce da
    nord-ovest;
  - cinghie verticali, bordo del coperchio, fascia in basso e serratura d'oro;
  - contorno `k`.
- Lettura: sul sentiero il legno ha la stessa rampa della terra, quindi lo stacco è affidato
  al contorno e all'oro. Sull'anteprima si legge su tutti e due i fondi.

## Interfaccia (`assets-src/ui/` → `assets/ui/`)

Layout dell'HUD come nel riferimento (vedi `art-preview-scene.png`): cuori in alto a sinistra,
sotto la barra XP e la fila delle icone delle abilità; timer con orologio centrato in alto;
livello e uccisioni con teschio in alto a destra. Carte del draft: pannello blu notte `1` con
riga di luce `2`, contorno `k` e bordo del colore dell'elemento (fuoco `o`, acqua `c`, natura
`e`, neutro `3`; selezionata oro `g` con alone `y`), icona in alto, titolo, "Lv.", descrizione.
Scelta del compagno: ritratto 40×36 (figura intera), nome, barre ATT/VEL/AREA (fuoco/teal/lime), slot
bloccato col lucchetto. Barra del boss: teschio, barra rossa `r` con luce `X` e ombra `R`.

Telaio delle icone 16×16: contorno `k` con angoli trasparenti, bordo chiaro in alto a sinistra
e scuro in basso a destra, fondo a due toni del tema, simbolo 12×12 al centro. Temi: fuoco
`f O O x`, acqua `l C C n`, natura `e E v N`, neutro `4 3 2 1`, XP `W U U V`.

**Carte reliquia** (`ui/upgrade_relic_<id>.png`): stesso telaio, tema reliquia viola. Il
draft le mostra con il bordo viola `#c785fa` e l'etichetta "Reliquia"; per distinguerle
dalle carte normali anche senza bordo:
- **Tema:** luce `q`, fondo `P` sopra `z`, bordo scuro `p`.
- **Borchia-gemma** in alto a destra, sul bordo di luce: `Q`, `w`, `Q`. Non tocca mai il
  simbolo.
- **Simboli:** oggetti disegnati con i materiali propri (pietra, oro, legno, corteccia, coccio,
  foglie), mai i simboli delle carte normali.

Simboli:
| Reliquia | Simbolo |
|---|---|
| `relic_courage` | pietra scura con una runa a fiamma d'oro e punti di luce attorno |
| `relic_hourglass` | clessidra d'oro con la sabbia che scende |
| `relic_nest` | nido intrecciato con tre scintille (fiamma, stella, scintilla azzurra) |
| `relic_spyglass` | cannocchiale di corteccia a tre sezioni con anelli d'oro e lente azzurra |
| `relic_oak` | cuore di legno con le venature e una fogliolina |
| `relic_dew` | goccia teal su una foglia |
| `relic_sandals` | sandalo con le cinghie, ala bianca al tallone e linee di vento |
| `relic_piggy` | salvadanaio di coccio di profilo con una moneta nella fessura |
| `relic_feast` | cesto con manico, mela, fungo e foglie |

Confronti:
- Il cuore di quercia è di legno (la vitalità è rossa).
- Il sandalo non è la piuma di "Passo leggero".
- Il cesto non è il frutto della merenda.

Il viola resta il colore dei nemici nel mondo: nell'interfaccia lo usano solo i telai delle
reliquie, mai gli sprite in gioco.

Anteprima ×4: `screenshots/anteprima-reliquie.png`. Nella prima riga le 9 reliquie; sotto
cuore di quercia, sandalo e cesto ci sono vitalità, passo leggero e merenda.

**Icone dei talenti** (`ui/talent_<id>.png`, 16×16, schermata dei talenti nel menu):
- **Telaio:** quello delle carte, nel tema dell'elemento del compagno, come in
  `ui/icon_<elemento>.png`: Fiammetto fuoco, Ondino acqua, Foglin natura, Tuonny elettro.
- **Ramo A:** il fondo a due toni dell'elemento (chiaro sopra):
  - fuoco `O` sopra `x`;
  - acqua `C` sopra `n`;
  - natura `v` sopra `N`;
  - elettro `G`.
- **Ramo B:** il tono scuro dell'elemento su tutto il fondo:
  - fuoco `x`, acqua `n`, natura `N`;
  - elettro `h`, con il bordo scuro `G`.

  Le due righe di un albero si distinguono a colpo d'occhio, anche con i nodi bloccati
  scuriti dal codice (nessuna immagine in più per i bloccati).
- **Simboli:** originali e di due famiglie, che seguono i rami:
  - A cambia l'attacco che il compagno ha già (esplosioni, onde, foglie in orbita, catena);
  - B è l'abilità nuova (soffio, cura e getto, radici, fulmine dal cielo).

  Il gradino 2 e il 3 dello stesso ramo si leggono come "più grande" o "più forte" del
  gradino 1.
- **Nemici:** quando il simbolo ha un bersaglio è una gelatina viola (`q p P`), come nella
  carta Fuoco perforante.

| Talento | Nome | Simbolo |
|---|---|---|
| `ember_a1` | Brace che scoppia | piccola esplosione a otto punte con il nucleo bianco |
| `ember_a2` | Scoppio largo | esplosione grande che riempie la carta |
| `ember_a3` | Brace che cova | nemico viola con tre fiammelle sulla testa |
| `ember_b1` | Soffio rovente | fiamma di traverso che esce verso destra (il fuoco di icon_fire girato) |
| `ember_b2` | Fiato lungo | fiamma lunga e stretta con le linee di velocita' |
| `ember_b3` | Soffio ardente | fiamma di traverso col cuore bianco incandescente e un piu' |
| `tide_a1` | Onda di ritorno | due onde, quella dietro piu' scura |
| `tide_a2` | Marea lenta | nemico viola mezzo sommerso con le gocce in testa |
| `tide_a3` | Marea alta | onda alta su tutta la carta con le frecce ai lati |
| `tide_b1` | Acqua che ristora | cuore sopra una piccola onda |
| `tide_b2` | Getto dal profondo | getto d'acqua che sale dalla pozza e solleva un nemico |
| `tide_b3` | Sorgente viva | fonte di pietra con lo zampillo e un cuore |
| `sprout_a1` | Linfa urticante | foglia che gocciola linfa d'ambra |
| `sprout_a2` | Quarta foglia | quattro foglie in croce attorno a un seme |
| `sprout_a3` | Rovo largo | grande orbita chiara con due foglie e il seme al centro |
| `sprout_b1` | Radici che afferrano | tre radici che escono da terra come dita |
| `sprout_b2` | Radici fitte | cinque radici a ventaglio che spuntano da terra |
| `sprout_b3` | Radici spinose | due radici grosse con le spine chiare |
| `spark_a1` | Eco del lampo | due fulmini, l'eco dietro e piu' scura |
| `spark_a2` | Catena salda | due anelli di catena intrecciati |
| `spark_a3` | Salto in piu' | fulmine che salta fra tre nemici, con un piu' |
| `spark_b1` | Lampo dal cielo | nuvola bianca con il fulmine che scende |
| `spark_b2` | Cielo carico | nuvola scura di temporale con due fulmini |
| `spark_b3` | Lampo grande | fulmine grosso che colpisce terra con lo scoppio |

Anteprima ×4: `screenshots/anteprima-talenti.png`. Un blocco per albero con il ritratto del
compagno, ramo A sopra e ramo B sotto.

**Icone dei talenti v2** (`docs/design/TALENT_TREES_V2.md`, sezioni 5 e 6): per compagno 3 rami
da 8 nodi più l'icona d'intestazione del ramo.
- **File:** `ui/talent_<compagno>_<el|su|ev>_<a|b|c><n>.png` (96 nodi) e
  `ui/branch_<compagno>_<el|su|ev>.png` (12 intestazioni), tutti 16×16.
- **Le 24 icone v1:** `talent_<compagno>_<a|b><n>` restano in `assets/` finché
  l'orchestratore non decide.

Fondo per ramo:

| Ramo | Luce | Fondo | Bordo scuro | Note |
|---|---|---|---|---|
| Elemento | come `icon_<elemento>` | come `icon_<elemento>` | come `icon_<elemento>` | fuoco `f O/x O`, acqua `l C/n C`, natura `e v/N E`, elettro `y G h` |
| Supporto (ambra) | `g` | `h` sopra `8` | `G` | marrone ambra con il bordo d'oro: l'oro chiaro su fondo ocra è già di Tuonny (elettro) |
| Evoluzione (blu) | `W` | `U` sopra `V` | `U` | il blu delle gemme XP, distinto dal teal di Ondino |

Tipi di nodo (stesso telaio delle carte):
- **Percorso** (righe 0–2): bordo chiaro in alto a sinistra e scuro in basso a destra.
- **Unione** (`c1`): bordo chiaro del ramo su tutti e quattro i lati, si legge più forte.
- **Finale** (`c2`): bordo d'oro (`y` in alto a sinistra, `g` in basso a destra) e una borchia
  bianca e oro sull'angolo in alto a destra, che esce di un pixel dal contorno.
- **Intestazione:** distintivo tondo (cerchio con contorno `k`, anello di luce del tema e
  fondo a due toni), così non si confonde con un nodo.
- **Nodi bloccati:** scuriti dal codice, nessuna immagine in più.

Simboli:
- **Fonte:** l'idea della colonna «Icona» del documento, disegnata nella griglia. Gli
  elementi comuni sono timbri condivisi: cuore, moneta, gemma XP, goccia, fiamma,
  fulmine, stella, foglia e le testine dei compagni di `upgrade_join_<id>`.
- **Riusati dalla v1** dove il documento chiede la stessa idea:

  | Nodo v2 | Simbolo v1 |
  |---|---|
  | `tide_ev_a2` | due onde |
  | `tide_su_c2` | fonte con il cuore |
  | `sprout_el_a3` | orbita larga |
  | `sprout_el_b1` | radici |
  | `sprout_el_c2` | radici spinose |
  | `sprout_ev_c2` | quattro foglie |
  | `spark_el_a1` | eco del lampo |
  | `spark_el_b1` | lampo dal cielo |
  | `spark_ev_a1` | catena |
- **Nemici** (bersagli, piedi, sagome avvolte): la gelatina viola, come nelle altre carte.
- **Compagni «più grandi»** (`*_ev_c1`): la testina del compagno con una freccia bianca in su.
- **Originalità:** nessun simbolo preso da altri giochi. L'immagine di riferimento
  dell'utente è servita solo per la disposizione della schermata.

Anteprima ×4: `screenshots/anteprima-talenti-v2.png`, disposta come la schermata:
- per compagno il ritratto e 3 colonne (elemento, supporto, evoluzione);
- in ogni colonna l'intestazione, poi i due percorsi su 3 righe, l'unione e il finale.

Icone del bottino:
- **`ui/coin.png` 16×16:** contatore delle monete nell'HUD ed etichetta del prezzo dei forzieri.
  È la stessa moneta vista di faccia, con il bordo rialzato in luce in alto a sinistra, il
  solco interno e un anello inciso.
- **`ui/icon_chest.png` 16×16:** indicatore del forziere più vicino fuori schermo. È nello stile
  del riquadro del boss (`src/presentation/boss_pointer.rs`: contorno `k` di 1 px e fondo
  pieno), ma il riquadro è già nel PNG:
  - fondo blu notte `1` dei pannelli invece del rosso pericolo `j`;
  - un forziere chiuso 10×9 al posto del teschio.

  Usato così com'è, il codice non deve disegnare bordo né fondo.

Le icone vanno disegnate alla **stessa scala intera del mondo** (1 texel UI = 1 texel del
canvas), non a pixel doppi: icone a 2× accanto a sprite a 1× danno pixel di misure diverse.
Elenco e uso previsto in `docs/art/STYLE_HANDOFF.md`.

L'interfaccia intera è disegnata per 960×540 e scala con il canvas (D-045): 1 texel = 2 unità
dell'interfaccia, quindi misure, bordi e spazi vanno tenuti pari (un bordo dispari diventa
mezzo pixel con il canvas ×3).

Font (D-044): **Tiny5** per HUD, numeri, tasti ed etichette, solo a 16 o 32 px (1 o 2 texel per
pixel del font a ogni scala), senza antialiasing e con ombra `#1b1428` di un pixel del font; **Pixelify
Sans** per titoli (22/33/44 px) e frasi (16 px) di menu, draft e risultati. Lettere accentate e
segni Latin-1 sono disegnati da entrambi. Il carattere 3×5 delle anteprime resta un segnaposto
dello script `preview-scene.py`.

## Home: logo, illustrazione, bottoni (D-063)

Nome mostrato: **Companion Survivors** (solo il nome a schermo). Riferimento: l'immagine della
home portata dall'utente il 2026-10-30, solo per disposizione e colori. Anteprima ×3/×4:
`docs/evidence/home_art_preview.png`.

**Logo** `ui/logo.png` (222×90), generato da `build_logo` in `scripts/art_logo.py` (chiamato da `scripts/build-assets.py`):
- **Disposizione:** due righe. "Survivors" sotto, spostata di 18 px a destra. È mostrato a
  texel nativi (1 pixel del logo = 1 texel), filtro nearest.
- **Lettere:** quelle di Pixelify Sans Regular, portate sulla griglia del font (un pixel del
  font = 93 unità). Niente antialiasing: il build si ferma se una cella è coperta a metà.
  - Ogni pixel del font diventa 3×5 pixel, poi la lettera cresce di 3 a destra e 2 in basso
    (aste e tratti di 6–7 px).
  - La Bold del font variabile non sta sulla griglia, per questo si ingrassa la Regular.
  - La C ha la bocca aperta (`LOGO_GLYPH_EDITS`): in Pixelify è chiusa tranne un pixel.
- **Colori** (`assets-src/logo/logo.palette.txt`), per lettera dall'alto al basso:

  | Riga | Riflesso (1 px in alto) | Sfumatura a quattro fasce | Ombra interna (1 px in basso) | Contorno 2 px | Ombra portata (2 px sotto, 1 a destra) |
  |---|---|---|---|---|---|
  | Companion | `1` fff4c8 | `2` ffe07a, `3` ffb547, `4` f7892f, `5` e45a2a (giallo pallido → arancio → rosso-arancio) | `6` a83a1c | `7` 4a2414 marrone scuro | `8` 22100a |
  | Survivors | `A` eafcff | `B` a6eeff, `C` 66cdf8, `D` 3a9fec, `F` 2a6fd2 (ciano pallido → celeste → blu) | `H` 1d4a9e | `J` 131a48 blu notte | `K` 090d26 |

  Il contorno ha gli angoli arrotondati e si fonde tra le lettere: la riga si legge come un
  blocco unico sopra qualunque zona dell'illustrazione.
- **Decorazioni** (`assets-src/logo/logo_parts.px`, a mano):
  - **Fiamma:** prende il posto del punto della "i" di Companion; è la fiamma di
    `ui/icon_fire` con il contorno `x`.
  - **Stella:** a quattro punte d'oro al centro della "o" di Survivors. Facce a girandola,
    centro `w`, contorno `k`.
  - **Foglie:** tre a sinistra di "Companion" e tre, specchiate, alla fine di "Survivors". Due
    verdi (`e` 9fd651 luce sopra, `E` 4f9a3a sotto) e il contorno `N` 173d26. Stanno dietro
    alle scritte.
- **Originalità:** lettering originale, nessun logo di altri giochi preso a modello.

**Illustrazioni** (`assets-src/illustrations/<nome>.png|.webp` → `assets/ui/<nome>.png`):
- **Origine:** solo immagini fornite dall'utente, che ne dichiara i diritti. Mai generate o
  scaricate dal progetto.
- **Regole:** non seguono la palette né la griglia, quindi sono l'unica eccezione alla
  pixel art del progetto.
- **Build:**
  - nessun ritaglio né ridimensionamento: il "cover" sulla finestra è del codice;
  - 256 colori con dithering Floyd–Steinberg (palette MAXCOVERAGE presa da una copia a 1/3)
    solo se, mediata su 9×9 pixel, la differenza resta sotto 2/255 in media e 32/255 al
    massimo. Altrimenti, o con la trasparenza, il PNG resta pieno;
  - la sorgente va in sRGB (il profilo ICC non viene convertito).
- **Limiti del validatore:** 2048 px per lato e 4 MiB.
- `home_background` di oggi è provvisoria (ha logo e menu dipinti): basta sostituire il file
  con lo stesso nome.

**Bottoni della home** (`ui/menu_<azione>.png`, 16×16, palette del progetto):
- **Stile:** quello delle icone dell'HUD (simbolo con contorno `k`, luce da nord-ovest, senza
  telaio).
- **Toni:** chiari, grigio pietra e bianco (`w 6 t T`), la testa in grigio-azzurro
  (`w 4 3`), così si leggono sul bottone blu notte. Solo la spada di `menu_new_run` è d'oro:
  è l'azione principale, come il cursore.

| File | Simbolo |
|---|---|
| `menu_new_run` | spada di traverso, lama d'oro chiaro, guardia d'oro, impugnatura di cuoio |
| `menu_companions` | testa tonda di creaturina con orecchie a punta e occhi 2×3 (nessun compagno né creatura di altri giochi) |
| `menu_talents` | libro aperto, pagina sinistra in luce, righe di testo |
| `menu_options` | ingranaggio a otto denti col foro tondo |
| `menu_quit` | porta socchiusa nel telaio, spiraglio scuro a destra |

Altri pezzi del pannello:
- **Cursore** `ui/menu_arrow.png` (7×11): triangolo d'oro con la punta sulla riga 5.
- **Angolo** `ui/frame_corner.png` (10×10, alto a sinistra, specchiato dal codice):
  - borchia quadrata d'oro e bronzo 6×6 con il rivetto;
  - bracci con il pieno sulla riga 2 e sulla colonna 2: una linea di bordo del pannello
    sulla seconda riga e colonna di texel si raccorda ai bracci.

## Pipeline

1. Sprite e icone: file `.px` in `assets-src/sprites/` e `assets-src/ui/` (intestazione `out:`
   e `mirror:`, poi la griglia; con `mirror: true` si scrive solo la metà sinistra). Commento
   in testa: cosa rappresenta, dimensioni, scelte di forma e colore. Fogli animati: `frame:`,
   `columns:`, `rows:` (e `anchor:`) nell'intestazione e un blocco `@ <riga> <colonna>` per
   fotogramma; il build li affianca in un solo PNG.
2. `python scripts/build-assets.py` → PNG in `assets/` + `assets/manifest.json` (tipo `sprite`,
   `ui`, `tileset`, `logo` o `illustration`; per i fogli anche `frame`, `frames`, `columns`,
   `rows`, `anchor`; per il logo `palette` e `font`; per le illustrazioni `encoding`,
   `indexed-256` o `rgb`). Due sorgenti con la stessa uscita fermano il build. Il codice sta
   in `scripts/build-assets.py` (sorgenti `.px`, fogli animati, manifest) e nei moduli
   accanto: `art_common.py` (percorsi, palette, griglie), `art_tileset.py`, `art_logo.py`,
   `art_illustration.py`.
3. `python scripts/validate-assets.py`: manifest, dimensioni (sprite ≤ 64, UI ≤ 40, logo ≤
   256×128, illustrazioni ≤ 2048 px e ≤ 4 MiB senza controllo di palette; i fogli
   fotogramma per fotogramma, misura multipla intera del fotogramma, nessun fotogramma vuoto,
   riga di base toccata con l'ancora in basso), naming, solo colori della palette, alpha
   0/255, licenza; avvisi se un upgrade o un compagno dei dati non ha icona o ritratto.
4. Anteprima senza compilare: `python scripts/preview-scene.py` → `docs/evidence/art-preview-*`.
5. Verifica a schermo nel gioco: screenshot + `python scripts/check-pixel-grid.py`.

Strumenti: nessun editor grafico né generatore IA (D-011). Gli sprite grandi (boss, albero,
ritratti) sono stati abbozzati con primitive geometriche ombreggiate (ellissi e poligoni con
la rampa del materiale, script usa e getta fuori dal repository) e poi rifiniti a mano nella
griglia `.px`, che resta l'unica sorgente.

## Lezioni

- P0.2: il primo compagno aveva la sagoma della gelatina e si distingueva solo per colore;
  ridisegnato a goccia di fiamma. Regola: forma, non solo colore.
- v1: il corazzato con il guscio di pietra grigia sembrava la roccia ostacolo; ora ha un
  carapace viola. Regola: i nemici non usano i materiali dell'ambiente come colore principale.
- v1: la cresta di Gocci con un buco al centro si leggeva come un manico; ricciolo pieno con
  l'interno in ombra.
- v1: alberi con ciuffi separati da linee scure regolari sembravano un guscio di tartaruga;
  meglio separare i ciuffi con un bordo di luce in alto e ombra tonale sotto.
- v1: fiori in ogni tile fanno rumore; meglio ciuffi di tre fiori raccolti a macchie.
- Mappa nello stile del player: sassolini con il contorno quasi nero sembravano buchi o macchie
  nere e facevano concorrenza agli attori; i dettagli del prato usano i toni dell'ambiente
  (`A`, `8`) come bordo. La regola delle chiome (niente linee scure fra i ciuffi) vale anche
  per l'albero da 40×48.
- Player (D-050): un foglio generato con le pose giuste ma la scala che cambia da un
  fotogramma all'altro fa "pompare" il personaggio e nasconde il passo. Regola: un master per
  direzione, ogni fotogramma ne deriva, e le tre direzioni hanno la stessa altezza.
- Player: a 1x il cammino di fronte si legge solo se i piedi sono chiaramente diversi nel
  contatto (uno a terra, l'altro su di 3 px) e il pugno opposto sale; 1–2 px non bastano.
- Player: l'ombra a terra va rimessa dal master dopo ogni spostamento dei piedi, altrimenti
  restano pixel d'ombra isolati o l'ombra segue il piede.
- Compagni a 2/3: ridurre il disegno con un filtro (anche a colore dominante, mai sfumato)
  sporca occhi, bocche e scintille e lascia il contorno a tratti. A 28×24 i master vanno
  ridisegnati a mano: occhi 2×3 con il riflesso in alto a sinistra, contorno nuovo sulla
  sagoma, effetti piccoli senza contorno.

## Da fare

Nomi definitivi dei compagni con verifica dei marchi; VFX di colpo piccoli; cespugli come ostacolo
(richiede un tipo nuovo in `src/world/obstacles.rs`). Player: verifica nel gioco vero
(screenshot e `check-pixel-grid.py`) delle pose a terra con il letterbox.
