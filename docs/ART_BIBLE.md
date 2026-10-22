# ART BIBLE — v0 (placeholder S0)

Direzione: survivor top-down luminoso e amichevole, pixel art leggibile in massa. Creature
originali con silhouette forti. **Mai** riprodurre Pokémon, Poké Ball o asset di altri giochi.
Moodboard `references/pixel_art_concept.png`: non presente al 2026-10-22.

## Griglia e scale

| Elemento | Dimensione | Note |
|---|---|---|
| Risoluzione virtuale | 480×270 | upscaling intero nearest, letterbox |
| Tile | 16×16 | tileset a colonna (array texture) |
| Evocatore (player) | 16×21 | ancora in basso al centro |
| Nemici comuni | gelatina 16×13, sciame 12×8, corazzato 20×15 | la taglia dice il ruolo: piccolo = veloce e fragile, grande = lento e resistente; silhouette diverse (tonda, alata, a cupola) |
| Compagni | 16×17 (fuoco) | silhouette verticale/appuntita: mai la stessa forma dei nemici |
| Proiettili | 6×6 | ancora al centro, nucleo chiaro, disegnati sopra gli attori |
| Gemma XP | 6×8 | rombo blu `u/U` con riflesso `w`; a terra, sotto gli attori |
| Roccia | 16×10 | luce in alto a sinistra; impronta r=7 |
| Albero | 24×26 | più alto dell'evocatore; chioma scura `A/B` per staccarsi dall'erba; impronta solo tronco r=4 |
| Élite | 24–32 px | |
| Boss | 48–64 px | massimo validato da `validate-assets.py` |

Regola: 1 texel = 1 pixel di mondo, niente rotazioni o scale non intere degli sprite;
larghezza pari per gli sprite ancorati al centro.

## Palette "Germoglio" (`assets-src/palette.txt`)

Colori per ruolo, per distinguere a colpo d'occhio:

| Ruolo | Colori |
|---|---|
| Contorno | `k` #1b1428 (tutti gli attori hanno contorno scuro) |
| Alleati (player, compagni) | teal `c/C/l` #2f8f9d #1f5a6b #6cc7c2, oro `g/G`; fuoco alleato `o/O` #f08a3c #b85a2a |
| Nemici | viola `p/P/q` #8e5bc8 #5b3a8c #c79bf0 |
| Pericoli e danni | rosso `r/R` #e2574c #a8323a |
| Gemme XP | blu `u/U` #4fa4e0 #2f6fb0 |
| Ambiente | erba `a/A/b` #5fae4e #3f8a43 #8fd16a, siepe `B` #2c6b3a, terra `d/D`, pietra `t/T`, fiori `y/m/w` |

L'ambiente resta su verdi/terra a contrasto medio; gli attori hanno contorno e colori più
saturi per staccarsi dal fondo. Il fuoco dei compagni è arancio, distinto dal rosso che resta
riservato a pericoli e danni. Feedback di colpo: il nemico viene moltiplicato per
(1, 0.45, 0.45) per 0,12 s.

Osservato in P0.3 con 220 nemici (`docs/evidence/p03-220-nemici-59fps.png`): l'orda si
stratifica da sola, sciami al centro e corazzati sul bordo, e le tre sagome restano
distinguibili anche in massa. Da migliorare in P2: il player si perde dentro l'orda (serve un
contorno o un'ombra più marcata) e le gelatine ammassate formano una macchia uniforme.

Interfaccia del draft (P0.4, `docs/evidence/p04-draft.png`): carte scure semitrasparenti con
bordo del colore della rarità (comune grigio chiaro, non comune verde, rara oro), titolo in
blu XP, carta più chiara al passaggio del mouse. Testi solo ASCII finché il font provvisorio
non ha le accentate (D-033).

Lezione P0.2: la prima versione del compagno aveva la stessa sagoma della gelatina e si
distingueva solo per il colore; ridisegnato a goccia di fiamma. Regola: alleati e nemici
devono differire per **forma**, non solo per colore.

## Pipeline

1. Sprite: file `assets-src/sprites/<nome>.px` (intestazione `out:` e `mirror:`, poi griglia;
   con `mirror: true` si scrive solo la metà sinistra).
2. `python scripts/build-assets.py` → PNG in `assets/` + `assets/manifest.json`.
3. `python scripts/validate-assets.py`: manifest, dimensioni, naming, solo colori palette,
   alpha 0/255, licenza.
4. Verifica a schermo: screenshot + `python scripts/check-pixel-grid.py`.

## Tileset `meadow_tileset.png` (indici in `src/world/tiles.rs`)

0 `grass_a`, 1 `grass_b`, 2 `grass_tuft`, 3 `flowers_white`, 4 `flowers_pink`, 5 `dirt`,
6 `hedge`.

## Da fare (P2)

Nomi originali dei tre compagni (lavorazione: Brasa/fuoco, Gocci/acqua, Frond/natura) con
verifica che non siano marchi noti; animazioni idle/movimento 2–4 frame; tile di transizione
erba/terra; font pixel; VFX di colpo piccoli e leggibili.
