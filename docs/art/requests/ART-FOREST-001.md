# ART-FOREST-001 — Bosco Ombroso: terreno, decorazioni e ostacoli definitivi

Richiesta ChatGPT (D-065) per gli asset definitivi del Bosco Ombroso (D-068), procedura in
`docs/art/CHATGPT_ART_WORKFLOW.md`. Oggi il bosco gira con asset provvisori procedurali
(MAP-FOREST, sezione "Bosco Ombroso" dell'ART_BIBLE): questa richiesta serve a sostituirli con
un disegno approvato dall'utente, **con gli stessi nomi, ordini e misure**, senza toccare codice
e dati.

| Campo | Valore |
|---|---|
| ID | `ART-FOREST-001` |
| Tipo | `sprite_concept` (le tre parti): ogni immagine è una reference da ridisegnare o quantizzare nella griglia del progetto, mai un PNG del gioco così com'è |
| Via | `CHATGPT_CONCEPT`, perché è il design nuovo di un bioma intero (materiali, bordi, oggetti) che deve avere un'identità sua e restare coerente; la pipeline `.px` lo ha fatto solo in versione provvisoria |
| Stato | `REQUESTED` (parte A). Le parti B e C si lanciano dopo l'approvazione di A |
| Owner | pixel-art-director (richiesta, controllo, ridisegno, promozione) + utente (generazione e approvazione in ChatGPT) |
| Decisioni | D-065, D-068 |

## Tre parti, una immagine ciascuna

Una sola immagine con 41 tile, 8 decorazioni e 4 oggetti verrebbe male: il modello mescola le
scale, perde la griglia, e terreno (opaco) e oggetti (trasparenti) chiedono fondi diversi. La
richiesta è divisa in tre parti, come ART-HOME-002 (parti A e B), con un prompt e un'immagine
ciascuna:

| Parte | Contenuto | Fondo | File di ritorno | Quando |
|---|---|---|---|---|
| A | terreno: 11 tile di base, i due modelli dei bordi (radura e muro), una prova d'insieme | opaco | `.art-inbox/ART-FOREST-001/ART-FOREST-001_A_v1.png` | subito |
| B | le 8 decorazioni calpestabili | trasparente (o magenta pieno) | `.art-inbox/ART-FOREST-001/ART-FOREST-001_B_v1.png` | dopo l'approvazione di A, nella stessa chat |
| C | i 4 ostacoli: abete, ceppo, tronco caduto, colonna spezzata | trasparente (o magenta pieno) | `.art-inbox/ART-FOREST-001/ART-FOREST-001_C_v1.png` | dopo l'approvazione di A, nella stessa chat |

B e C allegano la versione approvata di A: stessi colori, stessa luce, stessa misura dei pixel.
Se una parte non viene, si rifà solo quella (`_v2`), non le altre.

## Obiettivo

Il bosco è il secondo dei sei biomi richiesti dall'utente (D-068), sempre top-down come la
Prateria. Il codice disegna la mappa in tre livelli e legge tutto **per nome** da
`game-data/stages.ron` (biome `forest`):

1. **terreno**, tileset opaco a colonna di 41 tile 16×16 (`assets/tiles/forest_tileset.png`);
2. **decorazioni** calpestabili, tileset 16×16 con trasparenza cotto sopra il terreno, mai sul
   muro né sotto un ostacolo (`assets/tiles/forest_decor.png`);
3. **ostacoli** con collisione, sprite con l'ancora in basso al centro (`assets/sprites/*.png`).

Sopra il terreno il gioco disegna player, compagni, orde di nemici viola, gemme XP blu, monete
d'oro, segni di pericolo rossi: il bosco deve restare più spento di tutti loro.

## File

- **Di ritorno:** le tre immagini della tabella sopra (v2, v3… per le revisioni).
- **Di destinazione** (dopo `APPROVED`, ridisegno nella pipeline, stessi nomi di oggi):
  - A → texture e timbri del terreno in `scripts/art_forest.py`, o gli 11 tile di base
    ridisegnati come blocchi `@ <nome>` in un `assets-src/biomes/forest_tiles.px` letto da
    `art_forest.py`; i 30 bordi li compone la pipeline dai tile approvati con lo stile di bordo
    dell'immagine (vedi "PROMOTE") → `assets/tiles/forest_tileset.png`;
  - B → `assets-src/biomes/forest_decor.px` → `assets/tiles/forest_decor.png`;
  - C → `assets-src/sprites/forest_tree.px`, `stump.px`, `log.px`, `ruin_pillar.px` →
    `assets/sprites/*.png`.
- **Codice e dati che li usano:** `game-data/stages.ron`, `src/world/` (non si cambiano: nomi,
  ordine, misure e impronte restano quelli del contratto qui sotto).

## Contratto da rispettare (uguale agli asset provvisori)

**Terreno**, ordine = indice: `moss_a`, `moss_b`, `moss_leaves`, `moss_roots`, `clearing`,
`thicket`, `moss_c`, `moss_twigs`, `clearing_b`, `leaves_a`, `leaves_b`, poi `clearing_<lati>` ×15
e `thicket_<lati>` ×15 (maschera 1..15, suffissi `n e s w`, bit acceso se il vicino da quel lato
non è dello stesso materiale; nord = riga 0 del tile). Il muschio entra nei bordi di 2–4 px con
profili che valgono 2 px agli estremi.

**Decorazioni**, ordine = indice: `fern`, `mushrooms_tan`, `mushrooms_white`, `stones_moss`,
`ruin_slab`, `ruin_rubble`, `twigs`, `flowers_shade`. Alpha solo 0/255, almeno 1 px trasparente
su ogni lato del tile.

**Ostacoli** (ancora in basso al centro, base a 1–2 px dal fondo, ombra a terra cotta, impronta =
cerchio di raggio r centrato 4 px sopra il fondo):

| Sprite | Misura | r |
|---|---|---|
| `sprites/forest_tree.png` | 40×48 | 4, sul piede del tronco |
| `sprites/stump.png` | 20×16 | 6 |
| `sprites/log.png` | 24×16 | 7 |
| `sprites/ruin_pillar.png` | 16×28 | 6 |

## Formato e vincoli tecnici

ChatGPT non disegna a 16×16 pixel veri: si chiede pixel art **ingrandita di un fattore intero
noto**, su una griglia esatta, così all'INGEST si riporta alla misura nativa contando i blocchi.

| Parte | Tela | Griglia | Scala | Misura nativa |
|---|---|---|---|---|
| A | 1024×1024, opaca | 8×8 celle da 128×128 px, senza spazi né linee | 1 pixel dell'arte = 8×8 px | ogni cella è un tile 16×16 |
| B | 1024×1024, trasparente | 4×4 celle da 256×256 px; usate le prime due righe | 1 pixel dell'arte = 16×16 px | ogni cella è un tile 16×16 |
| C | 1536×1024, trasparente | oggetti in fila, basi sulla stessa riga (y ≈ 900 px), almeno 64 px fra uno e l'altro | 1 pixel dell'arte = 12×12 px | abete 40×48 (480×576 px), colonna 16×28 (192×336), ceppo 20×16 (240×192), tronco 24×16 (288×192) |

- **Alpha:** A opaca (attesa 255..255); B e C con fondo trasparente. Se la chat non dà la
  trasparenza, fondo magenta pieno #FF00FF, uniforme, senza ombre né sfumature: diventa
  trasparente all'INGEST. In ogni caso nel gioco l'alpha è solo 0 o 255, misurata.
- **Palette:** quella del bosco, `assets-src/biomes/forest.palette.txt` (26 colori, scritti nel
  prompt). All'INGEST ogni pixel dell'arte va al colore più vicino; quello che non torna si
  ridisegna.
- **Resa:** nearest-neighbor a scala intera, come tutto il mondo di gioco (480×270 ingrandito).
- **Allineamento:** i tile sono di 16 pixel dell'arte; i bordi del terreno si raccordano fra
  tile diversi; le decorazioni non toccano il bordo del tile.

## Stile (sintesi autonoma dall'ART_BIBLE, già scritta nei prompt)

- Pixel art 16-bit, vista dall'alto come il gameplay (top-down, gli oggetti alti mostrano la
  cima e un po' del lato sud); niente isometrico, niente prospettiva, niente panorama dipinto.
- Pixel netti, colori pieni, niente antialiasing, sfocature o rumore.
- Luce da nord-ovest; rampe di 3–4 toni per materiale.
- Ambiente più scuro e spento degli attori; contorno quasi nero `#1b1428` solo su muro e
  ostacoli, mai sui dettagli a terra; ombra a terra verde oliva `#3d5235`.
- Nell'ambiente niente viola, rosso, blu, azzurro e verde acqua (colori di ruolo di nemici,
  pericoli, gemme XP e alleati).

## Reference da allegare

| REF | File | Parte | Cosa mostra | Cosa NON copiare |
|---|---|---|---|---|
| REF-1 | `.art-inbox/ART-FOREST-001/ref/REF-1_prato_mappa_x3.png` (864×768, RGB; ritaglio senza testo di `docs/evidence/art-preview-tiles.png`, mappa del prato ×3) | A, B | lo stile approvato dei tile del gioco: densità di dettaglio, misura dei pixel, bordi morbidi fra due terreni, siepe che fa da muro con il contorno scuro e l'ombra portata | i colori del prato, la terra marrone, i fiori rosa e gialli, la disposizione della mappa |
| REF-2 | `.art-inbox/ART-FOREST-001/ref/REF-2_albero_roccia_prato_x8.png` (544×384, trasparente; `assets/sprites/tree.png` e `rock.png` ×8) | C | lo stile approvato degli ostacoli: contorno scuro di 1 pixel, luce da in alto a sinistra, ombra a terra, misura dei pixel | la chioma tonda dell'albero (l'abete deve avere un'altra sagoma), il grigio-azzurro della roccia |
| REF-A | la versione approvata della parte A (`ART-FOREST-001_A_v<N>.png`) | B, C | colori, luce e pixel del bosco approvati | la griglia dei tile |

Le due copie in `ref/` sono state preparate il 2026-10-31 (nearest, nessun ritocco) da asset
dello stile v1 già nel gioco; REF-1 è il ritaglio della mappa di `art-preview-tiles.png`, senza
la colonna di testo (ChatGPT tende a ricopiare le scritte).

**Da non allegare né usare:**
- i concept delle sei mappe o qualunque illustrazione panoramica del bosco: sono vedute
  dipinte, non tile. Possono guidare l'atmosfera solo a parole, mai come texture;
- una moodboard o un'immagine con tanti elementi non si ritaglia automaticamente presumendo che
  sia un tileset: si usa solo un'immagine fatta da questa richiesta, controllata all'INGEST;
- screenshot del gioco con HUD, personaggi o nemici (`docs/evidence/art-preview-forest.png`
  compresa): il bosco provvisorio non è approvato e non è un modello.

## Da preservare / da evitare

- **Preservare:** la leggibilità del gioco attuale (fondo spento, attori saturi); muro che si
  legge come muro (contorno scuro e ombra portata, come la siepe del prato); radure più chiare
  del muschio; abete con una sagoma diversa dall'albero tondo del prato; pietra delle rovine
  grigio caldo d'oliva.
- **Evitare:** vista isometrica o in prospettiva, panorami, orizzonte, luci volumetriche e
  nebbie dipinte; antialiasing e sfumature; funghi rossi a pois bianchi o di qualunque colore
  di ruolo; pietra grigio-viola o grigio-blu; testo, numeri, firme, cornici, linee di griglia;
  creature, personaggi, oggetti o simboli di giochi esistenti.

## Prompt A — terreno

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego un'immagine (REF-1): è una mappa del mio videogioco 2D in pixel art, una prateria vista dall'alto. Prendila solo come modello di stile: misura dei pixel, quantità di dettaglio, bordi morbidi fra due terreni, siepe scura che fa da muro. Non copiarne i colori, la terra marrone, i fiori né la disposizione.

Crea UNA sola immagine: il foglio dei tile del terreno di un nuovo livello, il "Bosco Ombroso", un sottobosco di muschio con radure al sole e un muro di cespugli fitti.

FORMATO (molto importante)
- Immagine quadrata 1024×1024 pixel, piena, senza trasparenza.
- È una griglia di 8×8 celle quadrate da 128×128 pixel, senza spazi fra le celle, senza linee di griglia, senza cornici.
- Ogni cella è un tile di 16×16 pixel dell'arte. Ogni pixel dell'arte è un quadrato pieno di 8×8 pixel dell'immagine, di un solo colore, allineato alla griglia che parte dall'angolo in alto a sinistra. Tutti i pixel dell'arte hanno la stessa misura in tutta l'immagine.

COSA C'È NELLE CELLE (righe dall'alto, colonne da sinistra)
Riga 1: 1) muschio; 2) muschio con un'altra disposizione dei ciuffi; 3) muschio con due foglie cadute; 4) muschio con una radice che affiora e rientra nel terreno senza toccare i bordi del tile; 5) radura: erba al sole; 6) muro di sottobosco fitto; 7) muschio con chiazze di luce filtrata dalle chiome; 8) muschio con rametti e aghi di pino caduti.
Riga 2: 1) radura con trifoglio e fiorellini bianchi; 2) muschio con foglie cadute color ocra; 3) muschio con foglie cadute marroni; celle 4–8) cinque tile di muschio diversi messi uno accanto all'altro, che si raccordano senza cuciture.
Righe 3–6, colonne 1–4: il modello dei bordi della radura. Le celle formano quattro forme di radura separate da sottili canali di muschio: un'isola quadrata di 3×3 tile (righe 3–5, colonne 1–3); una striscia verticale larga un tile e lunga tre (colonna 4, righe 3–5); una striscia orizzontale alta un tile e lunga tre (riga 6, colonne 1–3); un tile di radura isolato (riga 6, colonna 4). In ogni tile il muschio entra di 2–4 pixel dell'arte dai lati che non continuano nella stessa forma, con un profilo morbido e irregolare che vale sempre 2 pixel agli angoli del tile; dove la radura tocca il muschio l'erba passa in ombra e il muschio prende un po' di luce.
Righe 3–6, colonne 5–8: lo stesso modello per il muro di sottobosco: isola 3×3, striscia verticale, striscia orizzontale, cespuglio isolato, separati da canali di muschio. Dove il muro tocca il muschio ha un contorno quasi nero di 1 pixel dell'arte, dentro il contorno una riga di luce in alto e a sinistra, e sul muschio sotto e a destra un'ombra portata scura di 1–2 pixel dell'arte.
Righe 7–8: prova d'insieme. Riga 7: muschio con un piccolo tappeto di foglie e l'inizio di una radura. Riga 8: il muro di sottobosco lungo tutta la riga, con il bordo verso l'alto che confina con il muschio della riga 7.

MATERIALI
- Muschio (il pavimento quasi ovunque): verde scuro e freddo, tendente al verde-bluastro ma mai azzurro. Cuscinetti tondi di muschio a basso contrasto, qualche puntino più chiaro, dettagli piccoli (al massimo 4–5 pixel dell'arte). Deve essere un fondo calmo: sopra ci corrono decine di personaggi e nemici.
- Radura: erba verde medio, al sole, chiaramente più chiara del muschio, con ciuffetti tozzi a tre fili e qualche macchia di sole.
- Muro di sottobosco: cespugli di rovo tondi e fitti, più scuri del muschio, ognuno con la cima chiara in alto a sinistra; fra un cespuglio e l'altro buio profondo e qualche ramo marrone. Si deve capire subito che non si attraversa.
- Foglie cadute: foglioline di 2–3 pixel dell'arte, ocra spento e marrone, sparse, più fitte al centro del tile e rade verso i bordi; l'ultimo pixel di ogni lato resta muschio.
- Ogni tile delle righe 1 e 2 si deve poter ripetere: messo accanto a sé stesso in ogni direzione non mostra cuciture.

STILE
- Pixel art in stile 16-bit per un gioco visto dall'alto (top-down): il terreno è visto da sopra, piatto. Niente vista isometrica, niente prospettiva, niente orizzonte, niente paesaggio o panorama dipinto.
- Pixel netti e quadrati, colori pieni. Niente antialiasing, sfumature morbide, sfocature, rumore, texture fotografiche o effetto pittura.
- Luce sempre da in alto a sinistra: lati in alto e a sinistra più chiari, in basso e a destra in ombra. Ogni materiale ha 3–4 toni.
- È lo sfondo di un gioco con personaggi dai colori vivaci e nemici viola: il bosco deve essere più scuro e meno saturo di loro.
- Niente viola, lilla, rosso, rosa acceso, blu, azzurro né verde acqua: in questo gioco sono i colori dei nemici, dei pericoli, dei cristalli dell'esperienza e degli alleati.
- Usa il più possibile solo questi colori: muschio #6b9455 #4f7d4f #3d6647 #2e5040 #213c33 #162a25; erba della radura #84c25c #559c47 #3c7f3f #2a6238; terra, legno e foglie #e0b27a #c08b5a #8f603d #5a3a26 #b8862e #74491f; contorno #1b1428; bianco #f4f1e8; giallo dei fiori #ffd65c; ombra a terra #3d5235; pietra #c4bd9a #958f6e #676650 #434639; crema #e8dcc0 #b5a582.

DIVIETI
- Niente testo, lettere, numeri, firme, watermark, etichette, cornici o linee di griglia.
- Niente personaggi, animali, creature, oggetti alti, alberi, rocce o funghi in questa immagine: solo terreno e muro.
- Niente elementi riconoscibili di giochi, cartoni o marchi esistenti.

Consegna una sola immagine, senza varianti affiancate né collage.
===== FINE PROMPT =====
```

Allegare: REF-1. Salvare come `.art-inbox/ART-FOREST-001/ART-FOREST-001_A_v1.png`.

## Prompt B — decorazioni (dopo l'approvazione di A)

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego due immagini. REF-A è il foglio dei tile del terreno del "Bosco Ombroso" che abbiamo appena approvato: usa gli stessi colori, la stessa luce e lo stesso stile di pixel. REF-1 è una mappa del mio gioco, una prateria: prendila solo come modello di quantità di dettaglio, non dei colori.

Crea UNA sola immagine: 8 decorazioni piccole che stanno per terra nel bosco, separate, su fondo trasparente.

FORMATO (molto importante)
- Immagine quadrata 1024×1024 pixel con fondo trasparente. Se non puoi fare la trasparenza, usa un fondo magenta pieno #FF00FF, uniforme, senza ombre né sfumature.
- È una griglia di 4×4 celle da 256×256 pixel, senza linee di griglia né cornici. Usa solo le prime due righe; le ultime due restano vuote (trasparenti).
- Ogni cella è un tile di 16×16 pixel dell'arte. Ogni pixel dell'arte è un quadrato pieno di 16×16 pixel dell'immagine, di un solo colore, allineato alla griglia che parte dall'angolo in alto a sinistra.
- Ogni decorazione sta dentro la sua cella con almeno un pixel dell'arte (16 pixel dell'immagine) vuoto su ogni lato.
- Bordi netti: nessun pixel semitrasparente, nessun alone.

LE 8 DECORAZIONI (riga 1 da sinistra, poi riga 2 da sinistra)
1) una felce bassa a ventaglio, verde, appoggiata a terra;
2) tre funghetti color nocciola con il gambo crema;
3) tre funghetti crema con l'orlo del cappello nocciola chiaro;
4) tre sassi piatti di pietra grigio caldo con il muschio sopra;
5) una lastra di pavimento di pietra antica, spezzata, piatta a terra, scura e consumata, con il muschio in una crepa e una scheggia staccata;
6) quattro piccole macerie di pietra antica sparse;
7) tre rametti secchi incrociati;
8) un cespo basso di fiorellini bianchi con il cuore giallo, da sottobosco.

COME DEVONO ESSERE
- Sono dettagli piatti sul terreno che i personaggi calpestano: bassi, piccoli, niente parti alte.
- Senza contorno scuro: usano solo i toni dell'ambiente. Sotto le cose rialzate un'ombra a terra di 1 pixel dell'arte, verde oliva scuro #3d5235.
- Si devono leggere sia sul muschio scuro sia sull'erba chiara della radura, ma restare più spente dei personaggi del gioco.
- Funghi solo nocciola, crema o arancio spento: mai rossi, mai a pois bianchi, mai viola o blu.
- Pietra grigio caldo con un velo d'oliva: mai grigio-azzurro, mai grigio-viola.

STILE
- Pixel art in stile 16-bit per un gioco visto dall'alto (top-down). Niente vista isometrica, niente prospettiva.
- Pixel netti e quadrati, colori pieni. Niente antialiasing, sfumature morbide, sfocature o effetto pittura.
- Luce sempre da in alto a sinistra: lati in alto e a sinistra più chiari, in basso e a destra in ombra. 3–4 toni per materiale.
- Niente viola, lilla, rosso, rosa acceso, blu, azzurro né verde acqua.
- Usa il più possibile solo questi colori: muschio e felce #6b9455 #4f7d4f #3d6647 #2e5040 #213c33; erba #84c25c #559c47 #3c7f3f; legno e funghi #e0b27a #c08b5a #8f603d #5a3a26; ocra #b8862e #74491f; crema #e8dcc0 #b5a582 #f4f1e8; giallo #ffd65c; pietra #c4bd9a #958f6e #676650 #434639; ombra a terra #3d5235.

DIVIETI
- Niente testo, lettere, numeri, firme, watermark, etichette, cornici o linee di griglia.
- Niente terreno o sfondo dietro le decorazioni: solo le 8 decorazioni sul fondo trasparente.
- Niente personaggi, animali, creature né elementi riconoscibili di giochi, cartoni o marchi esistenti.

Consegna una sola immagine, senza varianti affiancate né collage.
===== FINE PROMPT =====
```

Allegare: REF-A (la `ART-FOREST-001_A_v<N>.png` approvata) e REF-1. Salvare come
`.art-inbox/ART-FOREST-001/ART-FOREST-001_B_v1.png`.

## Prompt C — ostacoli (dopo l'approvazione di A)

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego due immagini. REF-A è il foglio dei tile del terreno del "Bosco Ombroso" che abbiamo appena approvato: usa gli stessi colori, la stessa luce e lo stesso stile di pixel. REF-2 mostra un albero e una roccia del mio gioco: prendili come modello di contorno, ombreggiatura, ombra a terra e misura dei pixel. Non copiare la chioma tonda dell'albero né il grigio-azzurro della roccia.

Crea UNA sola immagine: 4 ostacoli del bosco, separati, su fondo trasparente.

FORMATO (molto importante)
- Immagine orizzontale 1536×1024 pixel con fondo trasparente. Se non puoi fare la trasparenza, usa un fondo magenta pieno #FF00FF, uniforme, senza ombre né sfumature.
- Ogni pixel dell'arte è un quadrato pieno di 12×12 pixel dell'immagine, di un solo colore, e tutti i pixel dell'arte hanno la stessa misura.
- I quattro oggetti stanno in fila da sinistra a destra, con almeno 64 pixel vuoti fra uno e l'altro e dai bordi; le loro basi poggiano tutte sulla stessa riga orizzontale, a circa 900 pixel dall'alto.
- Misure esatte in pixel dell'arte (larghezza × altezza, ombra a terra compresa), quindi in pixel dell'immagine: abete 40×48 (480×576); colonna spezzata 16×28 (192×336); ceppo 20×16 (240×192); tronco caduto 24×16 (288×192).
- Bordi netti: nessun pixel semitrasparente, nessun alone.

I 4 OSTACOLI (da sinistra)
1) Abete del bosco: sagoma a punta con quattro palchi di rami sovrapposti, le punte dei rami che pendono sul bordo di ogni palco; verde scuro e freddo, più scuro dell'erba; lato sinistro in luce, destro in ombra; sotto ogni palco l'ombra del palco sopra, senza linee scure fra i rami. In basso si vede un tronco corto e largo circa 8 pixel dell'arte, con le radici: è l'unica parte che blocca i personaggi, che passano dietro la chioma.
2) Colonna di pietra antica spezzata: fusto scanalato largo circa 10 pixel dell'arte su un basamento largo 14, rotta in cima con la faccia della rottura in luce, una crepa, un po' di muschio sulla rottura e sul basamento.
3) Ceppo d'albero tagliato: faccia del taglio in alto con gli anelli, corteccia, quattro radici che si allargano a terra, un ciuffo di muschio.
4) Tronco caduto corto, steso in orizzontale: a sinistra la faccia del taglio con gli anelli, corteccia con le scanalature lungo il tronco, cuscini di muschio sopra, l'estremità destra arrotondata.

COME DEVONO ESSERE
- Ogni oggetto ha un contorno quasi nero #1b1428 di 1 pixel dell'arte sulla sagoma esterna e un'ombra a terra piatta verde oliva scuro #3d5235 alla base, larga quanto l'oggetto.
- La base dell'oggetto sta 1–2 pixel dell'arte sopra il fondo del suo riquadro; l'oggetto è centrato in orizzontale nel suo riquadro.
- Pietra grigio caldo con un velo d'oliva: mai grigio-azzurro, mai grigio-viola. Legno marrone caldo.
- Devono distinguersi subito dai personaggi e dai nemici viola del gioco: niente occhi, facce o forme di creatura.

STILE
- Pixel art in stile 16-bit per un gioco visto dall'alto (top-down): si vede la cima degli oggetti e un po' del lato verso chi guarda, come nei giochi di ruolo 16-bit. Niente vista isometrica, niente prospettiva, niente panorama.
- Pixel netti e quadrati, colori pieni. Niente antialiasing, sfumature morbide, sfocature o effetto pittura.
- Luce sempre da in alto a sinistra: lati in alto e a sinistra più chiari, in basso e a destra in ombra. 3–4 toni per materiale.
- Niente viola, lilla, rosso, rosa acceso, blu, azzurro né verde acqua.
- Usa il più possibile solo questi colori: chioma e muschio #6b9455 #4f7d4f #3d6647 #2e5040 #213c33 #162a25; legno #e0b27a #c08b5a #8f603d #5a3a26; pietra #c4bd9a #958f6e #676650 #434639; contorno #1b1428; ombra a terra #3d5235.

DIVIETI
- Niente testo, lettere, numeri, firme, watermark, etichette, cornici o linee di griglia.
- Niente terreno o sfondo dietro gli oggetti: solo i 4 oggetti con la loro ombra a terra sul fondo trasparente.
- Niente personaggi, animali, creature né elementi riconoscibili di giochi, cartoni o marchi esistenti.

Consegna una sola immagine, senza varianti affiancate né collage.
===== FINE PROMPT =====
```

Allegare: REF-A (la `ART-FOREST-001_A_v<N>.png` approvata) e REF-2. Salvare come
`.art-inbox/ART-FOREST-001/ART-FOREST-001_C_v1.png`.

Gate (parte A, oggi): "Incolla il prompt A in ChatGPT, allega le reference indicate, genera e
approva l'immagine; salva il PNG in `.art-inbox/ART-FOREST-001/` e poi scrivimi
`ART-FOREST-001 pronto`". Per B e C lo stesso, dopo l'approvazione di A.

## Criteri di approvazione

Per ogni parte:
- [ ] una sola immagine nel formato della tabella; griglia e scala rispettate (comando di
      controllo sotto: blocchi uniformi alla scala attesa);
- [ ] alpha misurata: A 255..255; B e C con fondo a 0 (o magenta pieno) e nessun pixel fra 1 e
      254 dopo la conversione;
- [ ] colori vicini alla palette del bosco (distanza media misurata), niente viola, rosso, blu,
      azzurro, verde acqua;
- [ ] vista top-down come il gioco; niente isometrico, prospettiva o panorama;
- [ ] niente testo, firme, watermark, cornici, linee di griglia; nessun elemento di franchise,
      nessuna creatura;
- [ ] conferma esplicita dell'utente su quella versione.

In più:
- **A:** gli 11 tile di base riconoscibili nelle celle giuste; tile delle righe 1–2 ripetibili
  senza cuciture (prova all'INGEST: 3×3 copie affiancate); i due modelli dei bordi con le
  quattro forme separate dal muschio; muro che si legge come muro (contorno, luce, ombra
  portata); muschio più scuro della radura e calmo.
- **B:** le 8 decorazioni nell'ordine, piatte, senza contorno scuro, 1 pixel dell'arte libero
  sui lati; leggibili su muschio e radura (prova all'INGEST sopra i tile di A).
- **C:** misure in pixel dell'arte entro ±1 da quelle richieste (si correggono nel ridisegno);
  basi allineate; contorno di 1 pixel e ombra a terra; l'abete con una sagoma diversa
  dall'albero tondo del prato; la parte che tocca terra coincide con l'impronta (tronco
  dell'abete ~8 px, ceppo ~12, tronco caduto i 14 px centrali, basamento della colonna ~12).

## Piano di verifica

**INGEST** (per ogni file che arriva; nessun file = la parte resta `REQUESTED`)

1. Comando di controllo del workflow (misura, modo, profilo, alpha) sulla cartella
   `.art-inbox/ART-FOREST-001`.
2. Griglia e palette: blocchi uniformi alla scala attesa e distanza dei colori dalla palette del
   bosco (scala 8 per A, 16 per B, 12 per C; provato in Git Bash):

   ```text
   python -c "
   import sys
   from PIL import Image
   f, s = sys.argv[1], int(sys.argv[2])
   pal = [int(l.split()[1], 16) for l in open('assets-src/biomes/forest.palette.txt', encoding='utf-8') if l.strip() and not l.startswith('#')]
   pal = [((c >> 16) & 255, (c >> 8) & 255, c & 255) for c in pal]
   im = Image.open(f).convert('RGBA'); w, h = im.size; px = im.load()
   ok = tot = 0; err = []
   for by in range(0, h - s + 1, s):
       for bx in range(0, w - s + 1, s):
           ref = px[bx + s // 2, by + s // 2]
           if ref[3] == 0:
               continue
           tot += 1
           ok += all(max(abs(a - b) for a, b in zip(px[bx + dx, by + dy], ref)) <= 24 for dy in range(s) for dx in range(s))
           err.append(min(sum((a - b) ** 2 for a, b in zip(ref[:3], p)) ** 0.5 for p in pal))
   print(f'{f}: {w}x{h}, griglia {w / s:g}x{h / s:g} pixel dell arte da {s}x{s}')
   print(f'  blocchi pieni uniformi (tolleranza 24): {ok / max(tot, 1):.1%} di {tot}')
   print(f'  distanza dal colore piu vicino della palette del bosco: media {sum(err) / max(len(err), 1):.1f}, massima {max(err, default=0):.1f} (su 441)')
   " .art-inbox/ART-FOREST-001/ART-FOREST-001_A_v1.png 8
   ```

   Se la griglia non torna (pixel di misure diverse, sfasati o sfumati), l'immagine resta una
   reference da ridisegnare a mano: si scrive nel registro, non si forza la quantizzazione.
3. B e C con fondo magenta: copia con il magenta (distanza < 60 da #FF00FF) portato ad alpha 0
   e il resto ad alpha 255, in `.art-inbox/ART-FOREST-001/check/`; si misura l'alone rosa.
4. Copie di controllo in `check/`: riduzione alla misura nativa (colore centrale di ogni
   blocco, mappato alla palette del bosco), ingrandita ×4; per A le 3×3 ripetizioni dei tile
   delle righe 1–2; per B le decorazioni sopra il muschio e la radura di A; per C gli oggetti
   accanto al player alla scala del gioco.
5. Esito all'utente con le copie di `check/`, poi la sua conferma esplicita: solo allora
   `APPROVED` per quella parte.

**PROMOTE** (solo dopo `APPROVED`, una parte alla volta)

- **A:** gli 11 tile di base dalla riduzione quantizzata, ripuliti nella griglia (pixel isolati,
  colori fuori rampa), diventano la sorgente del terreno; i 30 bordi `clearing_<lati>` e
  `thicket_<lati>` li compone la pipeline (`make_forest_edge` in `scripts/art_forest.py`)
  dalle texture approvate, con profili, luce, contorno e ombra presi dai due modelli
  dell'immagine: così ogni maschera si raccorda al pixel con le altre, cosa che un'immagine
  generata non garantisce. Se l'utente preferisce i bordi disegnati nell'immagine, si
  ridisegnano tutti e 15 per materiale nella griglia e si verificano i raccordi.
- **B:** le 8 decorazioni ridotte, quantizzate e ripulite nei blocchi di
  `assets-src/biomes/forest_decor.px`, alpha 0/255, 1 px libero sui lati.
- **C:** i 4 sprite ridisegnati nei `.px` alle misure del contratto (la riduzione è solo la base
  del ridisegno), ancora, base, ombra `9` e impronte come oggi; `license:` con la provenienza.
- Colori che non stanno nella palette del bosco: si portano al più vicino; se ne serve uno
  nuovo, si aggiunge in coda alla palette e all'ART_BIBLE.

**VERIFY**

```text
python scripts/build-assets.py
python scripts/validate-assets.py
python scripts/preview-scene.py forest
cargo run -- --stage forest --autoplay --screenshot screenshots/art-forest-001.png --screenshot-after 4 --exit-after 5
python scripts/check-pixel-grid.py screenshots/art-forest-001.png
```

- Validatore: tileset 16×656 opaco e 16×128 con alpha 0/255 e bordo libero, sprite alle misure
  del contratto, solo colori della palette del bosco.
- Anteprima `docs/evidence/art-preview-forest.png` e screenshot del gioco (dell'orchestratore,
  che compila): attori, gemme e monete leggibili sul muschio e sulle radure; muro chiaro come
  muro; ostacoli distinti dai nemici; nessun pixel sfumato.
- Prova in `docs/evidence/` e righe in `ASSET_LICENSES.md` (sezione "Asset da richieste
  ChatGPT"), stato `INTEGRATED` per parte.

## Registro delle versioni

| Parte | Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|---|
| A | — | — | — | — | — |
| B | — | — | — | — | — |
| C | — | — | — | — | — |

## Revisioni

(prompt con le sole correzioni, fra gli stessi delimitatori, da usare nella stessa chat
allegando l'ultima versione; nome del file `ART-FOREST-001_<parte>_v<N+1>.png`)

## Provenienza e licenza (da compilare dopo l'import)

- **Versione promossa e file:** …
- **Generata il / approvata il:** … / … (frase dell'utente: "…")
- **Strumento:** ChatGPT, account dell'utente.
- **Reference usate:** REF-1, REF-2 (copie in `.art-inbox/ART-FOREST-001/ref/`), REF-A.
- **Cambiato nel progetto:** riduzione alla misura nativa, quantizzazione nella palette del
  bosco, ridisegno nella griglia `.px`, bordi composti dalla pipeline.
- **Diritti:** dichiarati dall'utente secondo i termini di OpenAI che ha accettato (D-065).
- **Riga del manifest** (`license:` dei `.px` e licenza dei tileset): …
- **Voce in `ASSET_LICENSES.md`:** sì / no
