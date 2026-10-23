# ART BIBLE — v1 (art pass "16-bit")

Direzione: survivor top-down luminoso e amichevole in **pixel art stile 16-bit**, ricca ma
leggibile in massa. Riferimento di stile: l'immagine portata dall'utente il 2026-10-23 (solo
**stile**: densità di dettaglio, rampe di colore, contorni, prato, layout dell'HUD e delle
carte). **Mai** riprodurne i contenuti: niente creature, allenatore col cappellino, edifici,
sfere, nomi o UI di Pokémon o di altri giochi. La moodboard non è nel repository e non va
ritagliata.

Anteprime di questa versione (composte da `scripts/preview-scene.py`, x3):
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
| Evocatore (player) | 16×21 | **16×24** | 3 righe in più: viso con occhi e gote, cintura, stivali e ombra; stessa larghezza (ingombro invariato) |
| Brasa | 16×17 | **16×20** | punta di fiamma più alta e pancia; resta la più alta e stretta dei tre |
| Gocci | 16×14 | **18×18** | la bolla è la più larga dei tre; spazio per il ricciolo della cresta |
| Frond | 16×17 | **16×20** | foglie più grandi con nervatura |
| Gelatina | 16×13 | **16×14** | una riga per la bocca con le zanne |
| Sciame | 12×8 | **14×11** | ali con membrana e corpo tondo leggibili; resta il più piccolo |
| Corazzato | 20×15 | = | ridisegnato a carapace viola |
| Élite | (24–32) | **26×22** | gelatina grande e scura, spine rosse |
| Boss | (48–64) | **56×47** | colosso a cupola, ~1/6 dell'altezza dello schermo |
| Proiettile di fuoco | 6×6 | **8×8** | sfera con nucleo bianco, visibile sopra l'orda |
| Foglia orbitante | 6×6 | **8×8** | nervatura e contorno |
| Goccia dell'onda | 4×4 | = | più chiara (`L`/`w`), senza contorno |
| Gemma XP | 6×8 | **8×10** | cristallo sfaccettato con riflesso |
| Segno di pericolo | — | **6×6** | anello rosso a terra |
| Roccia | 16×10 | **16×12** | quattro toni e ombra; larghezza = impronta (r=7) |
| Albero | 24×26 | **32×32** | chioma a ciuffi; impronta solo sul tronco |

Massimi: comuni ≤ 20 px, compagni e player ≤ 24, alberi ≤ 32, boss ≤ 64 (validato). A
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
| Alleato teal (mantello, Gocci) | `L` `l` `c` `C` `n` |
| Oro (fermagli, Frond) | `y` `g` `G` `h` |
| Fuoco alleato (Brasa) | `y` nucleo, `f` `o` `O` `x` — arancio, mai il rosso dei pericoli |
| Natura alleato (Frond, foglie) | `i` `e` `E` `v` — più chiaro e giallo dell'erba |
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

## Tileset `meadow_tileset.png` (indici in `src/world/tiles.rs`)

Procedurale con seed fisso in `scripts/build-assets.py`, timbri che si avvolgono sui bordi del
tile (nessuna cucitura).

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

## Interfaccia (`assets-src/ui/` → `assets/ui/`)

Layout dell'HUD come nel riferimento (vedi `art-preview-scene.png`): cuori in alto a sinistra,
sotto la barra XP e la fila delle icone delle abilità; timer con orologio centrato in alto;
livello e uccisioni con teschio in alto a destra. Carte del draft: pannello blu notte `1` con
riga di luce `2`, contorno `k` e bordo del colore dell'elemento (fuoco `o`, acqua `c`, natura
`e`, neutro `3`; selezionata oro `g` con alone `y`), icona in alto, titolo, "Lv.", descrizione.
Scelta del compagno: ritratto 32×32, nome, barre ATT/VEL/AREA (fuoco/teal/lime), slot
bloccato col lucchetto. Barra del boss: teschio, barra rossa `r` con luce `X` e ombra `R`.

Telaio delle icone 16×16: contorno `k` con angoli trasparenti, bordo chiaro in alto a sinistra
e scuro in basso a destra, fondo a due toni del tema, simbolo 12×12 al centro. Temi: fuoco
`f O O x`, acqua `l C C n`, natura `e E v N`, neutro `4 3 2 1`, XP `W U U V`.

Le icone vanno disegnate alla **stessa scala intera del mondo** (1 texel UI = 1 texel del
canvas), non a pixel doppi: icone a 2× accanto a sprite a 1× danno pixel di misure diverse.
Elenco e uso previsto in `docs/art/STYLE_HANDOFF.md`.

Font (D-044): **Tiny5** per HUD, numeri, tasti ed etichette, solo a 16 o 24 px (2 o 3 pixel
per pixel del font), senza antialiasing e con ombra `#1b1428` di un pixel del font; **Pixelify
Sans** per titoli (22/33/44 px) e frasi (16 px) di menu, draft e risultati. Lettere accentate e
segni Latin-1 sono disegnati da entrambi. Il carattere 3×5 delle anteprime resta un segnaposto
dello script `preview-scene.py`.

## Pipeline

1. Sprite e icone: file `.px` in `assets-src/sprites/` e `assets-src/ui/` (intestazione `out:`
   e `mirror:`, poi la griglia; con `mirror: true` si scrive solo la metà sinistra). Commento
   in testa: cosa rappresenta, dimensioni, scelte di forma e colore.
2. `python scripts/build-assets.py` → PNG in `assets/` + `assets/manifest.json` (tipo `sprite`,
   `ui` o `tileset`).
3. `python scripts/validate-assets.py`: manifest, dimensioni (sprite ≤ 64, UI ≤ 32), naming,
   solo colori della palette, alpha 0/255, licenza; avvisi se un upgrade o un compagno dei dati
   non ha icona o ritratto.
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

## Da fare

Animazioni idle/movimento 2–4 frame (servono più frame nei `.px` e nel manifest); nomi
definitivi dei compagni con verifica dei marchi; VFX di colpo piccoli; cespugli come ostacolo
(richiede un tipo nuovo in `src/world/obstacles.rs`).
