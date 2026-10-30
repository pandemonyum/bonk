# ART-HOME-001 — Sfondo della home nativo 16:9, senza logo né menu

Richiesta ChatGPT (D-065), procedura in `docs/art/CHATGPT_ART_WORKFLOW.md`. La v1 è arrivata il
2026-10-30, è stata controllata all'INGEST, approvata dall'utente lo stesso giorno, promossa e
verificata nel gioco (esiti sotto). Chiusa: una modifica futura è una v2 (`CHATGPT_EDIT`).

| Campo | Valore |
|---|---|
| ID | `ART-HOME-001` |
| Tipo | `illustration` |
| Via | `CHATGPT_EDIT`, perché serve continuità con il concept approvato di D-063: stessi personaggi, stessa scena e stesso stile; cambiano solo il formato e le scritte dipinte |
| Stato | `INTEGRATED`: v1 approvata dall'utente il 2026-10-30 ("Approvo v1"), promossa e verificata il 2026-10-30 (build, validatore e home a 1280×720, 1920×1080, 1024×768) |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (generazione e approvazione in ChatGPT) + gameplay-engineer (`FOCUS` della home, solo se serve) |
| Decisioni | D-063, D-065 |

## Obiettivo

Sfondo della pagina iniziale, nativo 16:9, senza logo, titolo, menu, bottoni, numero di versione
né altro testo. Logo (`ui/logo.png`) e pannello del menu restano layer separati del codice
(`src/presentation/home.rs`, da non modificare). Il concept di oggi è provvisorio perché ha
dipinti sopra il logo "Companion Survivors", il pannello del menu (Nuova Partita, Continua,
Compagni, Talenti, Opzioni, Esci) e "v1.0.0".

## File

- **Di ritorno:** `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png` (v2, v3 per le revisioni).
- **Di destinazione:** `assets-src/illustrations/home_background.png` al posto del `.webp` (che
  resta nella storia di git), più `assets-src/illustrations/home_background.license.txt`.
- **Uscita del build:** `assets/ui/home_background.png` (`kind` `illustration`).
- **Codice:** la home copre la finestra senza deformare con `FOCUS` (0.65, 0.75) e legge la
  misura dall'immagine. Un ritocco di `FOCUS` è del gameplay-engineer, fuori da questa richiesta.

## Formato e vincoli tecnici

- **Uscita:** una sola immagine orizzontale 16:9, idealmente 1920×1080 o la risoluzione più alta
  disponibile; lato lungo fra 1448 px (quanto il concept di oggi) e 2048 px (validatore). PNG del
  build ≤ 4 MiB (oggi il concept pesa 2,41 MiB in RGB e 0,80 MiB a 256 colori).
- **Formato diverso da 16:9** (per esempio 3:2): tutto l'essenziale nella fascia centrale 16:9,
  con circa l'8% di cielo e prato tagliabile sopra e sotto. Il ritaglio si fa all'INGEST e si
  mostra all'utente prima dell'approvazione. Niente stiramento, niente ingrandimento.
- **Alpha:** opaca, la trasparenza non serve (misurata all'INGEST: atteso 255..255).
- **Colore:** sRGB (il build non converte i profili). Palette libera: è un'illustrazione.
- **Resa:** filtro lineare e "cover" del codice, non nearest-neighbor.
- **Aree coperte dalla UI**, in % dell'immagine 16:9 (UI = scala del canvas / 2, scala =
  floor(min(L/480, A/270)); logo 444×180 unità UI con padding (28, 12), pannello largo 300
  unità, x 44–344):

  | Finestra | UI | Logo | Pannello | Parte visibile dell'immagine |
  |---|---|---|---|---|
  | 1280×720 | ×1 | x 2–37%, y 2–27% | x 3–27%, y ~27–72% | tutta |
  | 1920×1080 | ×2 | x 3–49%, y 2–36% | x 5–36%, y ~37–95% | tutta |
  | 1024×768 (4:3) | ×1 | fino a x ≈ 51%, y 0–25% | fino a x ≈ 41%, y ~27–67% | x ≈ 16–91% |

- **Composizione richiesta** (in % del fotogramma 16:9):
  - gruppo, da sinistra Foglin, player, Ondino, Fiammetto, dentro x 50–88%; teste non sopra
    y ≈ 50%; piedi a y ≈ 85–88%; player ≈ 28–32% dell'altezza, alla stessa scala del concept
    (dove il gruppo sta in x ≈ 44–94% del 4:3, cappello a y ≈ 56%, piedi a y ≈ 86%);
  - fascia x 0–45%: niente personaggi né punti di interesse, paesaggio e fronde più calmi e
    meno contrastati, nessun pannello dipinto;
  - in alto a sinistra (x 0–52%, y 0–36%) niente punti di interesse: è l'area del logo a
    1920×1080 e a 1024×768 (aggiunta del pixel-art-director ai dati misurati);
  - castello, portale e cascate dentro x 40–90%, y 5–55%; castello e portale oltre x 50%;
  - nulla di essenziale nel 16% a sinistra né nel 9% a destra (tagliati a 4:3).
- Il paesaggio si allarga: non si stira né si taglia il 4:3. Allargando a sinistra di circa
  0,3 volte l'altezza, il gruppo del concept cade già in x ≈ 50–87%.

## Stile (sintesi dall'ART_BIBLE)

Pixel art 16-bit luminosa e amichevole, pixel netti. Personaggi con colori saturi e caldi e
contorno scuro; ambiente un po' più tenue ma luminoso, così gli attori si staccano dal fondo.
Luce diurna, forme leggibili. Nessun elemento di franchise altrui. Le illustrazioni non seguono
palette né griglia del progetto (ART_BIBLE, "Illustrazioni"), ma restano nello stile del concept.

## Reference da allegare

| REF | File nel repo | Cosa mostra | Cosa NON copiare |
|---|---|---|---|
| REF-1 (obbligatoria) | `assets-src/illustrations/home_background.webp` (1448×1086, 4:3, RGB, sRGB; dalla PROMOTE solo nella storia di git, commit `0ef0ad2`) | il concept approvato di D-063: scena, quattro personaggi, stile | il logo "Companion Survivors", il pannello del menu con voci e icone, "v1.0.0", il formato 4:3 |

Se la chat non accetta il `.webp`, allega la copia già pronta
`.art-inbox/ART-HOME-001/ref/REF-1_home_background.png`: PNG senza perdita, stessi pixel (verificato
con `ImageChops.difference`), stesso profilo sRGB. È stata fatta così:

```text
python -c "from PIL import Image; im = Image.open('assets-src/illustrations/home_background.webp'); im.save('.art-inbox/ART-HOME-001/ref/REF-1_home_background.png', icc_profile=im.info.get('icc_profile'))"
```

Da non allegare: `assets/ui/logo.png`, screenshot del gioco o della home con la UI (per esempio
`docs/evidence/home_page_provvisoria.png`), fogli degli sprite e ritratti, immagini di Tuonny.

## Da preservare / da evitare

- **Preservare:**
  - a destra un sentiero d'erba sul ciglio di una scarpata, con margherite bianche;
  - il player di spalle: cappello rosso con visiera e fascetta bianca dietro, capelli neri a
    ciuffi, maglietta rossa a maniche corte, zaino di cuoio marrone con fibbia sulla schiena
    (non è un oggetto in mano), salopette/jeans blu con bretelle, guanti marrone scuro, stivali
    marroni;
  - a sinistra del player Foglin: volpe arancione, petto crema, occhi verdi, grandi foglie
    verdi come orecchie e coda, seduta, girata verso il player;
  - a destra Ondino: lontra blu, muso e pancia crema, bocca aperta felice, bolle azzurre;
  - all'estrema destra Fiammetto: lucertola rosso-arancio, cresta a punte grigio antracite su
    testa e schiena, pancia crema, fiamma sulla punta della coda, guarda il gruppo;
  - la valle: boschi, sentiero di terra a curve con un ponte di legno e casette, lago e fiume,
    scogliere con cascate e un ponte di pietra ad arco, castello dai tetti rossi su una collina
    lontana, porta di pietra in rovina con portale azzurro a spirale e rune luminose,
    montagne, nuvole bianche nel cielo azzurro;
  - a sinistra la cornice: grande tronco con le fronde, staccionata di legno in basso.
- **Evitare:**
  - logo, titolo, menu, bottoni, numero di versione, qualunque testo o pannello dipinto;
  - mantello, armi o oggetti in mano al player;
  - Tuonny (il quarto compagno non è nel concept) e qualunque creatura nuova;
  - elementi di franchise: creature note, sfere bicolori rosso-bianche, loghi o UI di altri
    giochi;
  - personaggi rimpiccioliti, deformati o tagliati; il 4:3 stirato.

## Prompt

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego un'immagine (REF-1): è il concept approvato della pagina iniziale del mio videogioco, un'illustrazione in pixel art. Partendo da questa immagine crea UNA sola immagine: la stessa scena come sfondo pulito, in formato orizzontale 16:9.

COSA CAMBIA
1. Formato: da 4:3 a 16:9 orizzontale, idealmente 1920×1080 o la risoluzione più alta che puoi. Allarga il paesaggio ai lati (più valle, bosco, scogliere e cielo): non stirare, non schiacciare e non ritagliare la scena di partenza.
2. Nella reference sono dipinti sopra la scena un logo con una scritta in alto a sinistra, un pannello scuro del menu con voci e icone sulla sinistra e un numero di versione in basso a sinistra. Toglili tutti e non riprodurli in nessuna forma, nemmeno sfocati o semitrasparenti: al loro posto continua il paesaggio.

COSA RESTA IDENTICO
I quattro personaggi della reference, con lo stesso design, gli stessi colori, le stesse proporzioni, la stessa posa e la stessa scala (il ragazzo è alto circa il 30% dell'immagine). Non rimpicciolirli e non deformarli.
- Il ragazzo, visto di spalle: cappello rosso con visiera e fascetta bianca dietro, capelli neri a ciuffi, maglietta rossa a maniche corte, zaino di cuoio marrone con la fibbia portato sulla schiena, salopette di jeans blu con le bretelle, guanti marrone scuro, stivali marroni. Niente mantello, niente armi, niente oggetti in mano.
- Alla sua sinistra la volpe arancione con il petto crema e gli occhi verdi, con grandi foglie verdi al posto delle orecchie e della coda; è seduta e girata verso il ragazzo.
- Alla sua destra la lontra blu con muso e pancia crema, la bocca aperta e felice, bolle azzurre attorno.
- All'estrema destra la lucertola rosso-arancio con la cresta a punte grigio antracite su testa e schiena, la pancia crema e una fiamma sulla punta della coda; guarda il gruppo.
Il luogo: il gruppo sta su un sentiero d'erba sul ciglio di una scarpata, con margherite bianche, e guarda una valle con boschi, un sentiero di terra a curve con un ponte di legno e casette, un lago e un fiume, scogliere con cascate e un ponte di pietra ad arco, un castello dai tetti rossi su una collina lontana, una porta di pietra in rovina con un portale azzurro a spirale e rune luminose, montagne e nuvole bianche nel cielo azzurro. A sinistra fanno da cornice un grande tronco con le sue fronde e, in basso, una staccionata di legno.

COMPOSIZIONE (percentuali della larghezza contate da sinistra, dell'altezza contate dall'alto)
- Il gruppo, da sinistra volpe, ragazzo, lontra, lucertola, sta tutto fra il 50% e l'88% della larghezza. Nessuna testa sopra la metà dell'altezza; i piedi poggiano fra l'85% e l'88% dell'altezza. Tutti interi, nessuno tagliato o coperto.
- Fascia sinistra, dal bordo al 45% della larghezza: sopra il gioco ci mette il titolo e il menu. Lì niente personaggi e niente punti di interesse, solo paesaggio e fronde più calmi, meno contrastati e meno dettagliati. Non disegnare pannelli, riquadri o spazi vuoti per il testo: solo paesaggio.
- In alto a sinistra, fino al 52% della larghezza e fino al 36% dell'altezza, niente punti di interesse: lì va il titolo.
- Castello, portale e cascate stanno fra il 40% e il 90% della larghezza e fra il 5% e il 55% dell'altezza; castello e portale a destra della metà.
- Niente di importante nel 16% a sinistra né nel 9% a destra: su alcuni schermi quei bordi vengono tagliati.
- Se non puoi fare un 16:9 esatto, fai un orizzontale un po' più alto (per esempio 3:2) e lascia sopra e sotto circa l'8% di solo cielo e prato, che verrà tagliato: le percentuali qui sopra valgono per la fascia centrale 16:9.

STILE
- Pixel art 16-bit luminosa e amichevole, come la reference: pixel netti e leggibili, niente sfocature, niente effetto pittura, foto o 3D.
- Personaggi con colori saturi e caldi e un contorno scuro; ambiente un po' più tenue ma luminoso, così i personaggi si staccano dal fondo.
- Luce del giorno, cielo azzurro, forme semplici e leggibili.
- Immagine piena, senza trasparenza.

DIVIETI
- Niente testo, lettere, numeri, logo, firme, watermark; niente cornici, bottoni, pannelli, icone o interfaccia di gioco.
- Niente elementi di giochi, cartoni o marchi esistenti: nessuna creatura conosciuta, nessuna sfera bicolore rossa e bianca, nessun logo o interfaccia di altri giochi.
- Nessuna creatura o persona in più oltre ai quattro personaggi descritti, nemmeno altre creature viste prima in questa conversazione.
- Al ragazzo niente mantello, armi o oggetti in mano.

Consegna una sola immagine, senza varianti affiancate né collage.
===== FINE PROMPT =====
```

Allegare: REF-1 (`assets-src/illustrations/home_background.webp`, o la copia PNG in `ref/`).
Salvare come: `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png`. Se c'è ancora la chat in cui è nato
il concept, usare quella.

Gate: "Incolla il prompt in ChatGPT, allega le reference indicate, genera e approva l'immagine;
salva il PNG in `.art-inbox/ART-HOME-001/` e poi scrivimi `ART-HOME-001 pronto`".

## Criteri di approvazione

- [ ] Una sola immagine orizzontale, 16:9 (dopo l'eventuale ritaglio mostrato all'utente), lato
      lungo fra 1448 e 2048 px; oltre 2048 una riduzione si concorda con l'utente e si annota.
- [ ] Alpha misurata 255..255 (o modo `RGB`); profilo sRGB o assente.
- [ ] Niente testo, lettere, numeri, logo, firme, watermark, cornici, bottoni, pannelli o HUD,
      angoli e cielo compresi; nessuna traccia del logo, del menu o di "v1.0.0" della reference.
- [ ] I quattro personaggi fedeli a REF-1 nei dettagli elencati sopra, e solo loro: niente
      mantello, armi o oggetti in mano al player, niente Tuonny né altre creature.
- [ ] Gruppo dentro x 50–88%, teste non sopra y 50%, piedi a y 85–88%, player 28–32%
      dell'altezza; tutti interi.
- [ ] Fascia x 0–45% calma, senza personaggi, punti di interesse o pannelli dipinti; niente punti
      di interesse in x 0–52%, y 0–36%.
- [ ] Castello, portale e cascate dentro x 40–90%, y 5–55%; castello e portale oltre x 50%.
- [ ] Niente di essenziale in x 0–16% e x 91–100%.
- [ ] Nessun elemento di franchise (creature note, sfere bicolori rosso-bianche, loghi o UI di
      altri giochi).
- [ ] Stile coerente con il concept: pixel art 16-bit, pixel netti, personaggi saturi con contorno
      scuro, ambiente più tenue ma luminoso, luce diurna.
- [ ] PNG del build ≤ 4 MiB (validatore, al VERIFY).
- [ ] Conferma esplicita dell'utente su questa versione (e sul ritaglio, se c'è).

Esito sulla v1 (le caselle restano come modello): criterio per criterio nella tabella dell'INGEST;
i due NO (fascia sinistra, landmark fuori riquadro) e i due DUBBIO (runa a "M", bordo destro a 4:3)
sono stati accettati dall'utente approvando la v1; PNG del build 0,84 MiB (VERIFY); conferma
esplicita "Approvo v1" del 2026-10-30, senza ritaglio.

## Piano di verifica

**INGEST**
1. Comando di controllo del workflow su `.art-inbox/ART-HOME-001`.
2. Se non è 16:9: ritaglio centrale con il comando del workflow in
   `ART-HOME-001_v<N>_16x9.png`; un'altra posizione (più prato o più cielo) va scritta nel
   registro.
3. Copia in `.art-inbox/ART-HOME-001/check/` con disegnati il rettangolo del gruppo, la fascia
   x 45%, il riquadro x 52% / y 36%, il rettangolo di castello, portale e cascate, i tagli del
   4:3 (x 16% e 91%) e le aree di logo e pannello della tabella. Mostrata all'utente con l'esito.

**VERIFY** (dopo `APPROVED` e PROMOTE)

```text
python scripts/build-assets.py
python scripts/validate-assets.py
cargo run -- --window 1280x720 --scale-factor 1 --screenshot screenshots/art-home-001-1280x720.png --screenshot-after 4 --exit-after 5
cargo run -- --window 1920x1080 --scale-factor 1 --screenshot screenshots/art-home-001-1920x1080.png --screenshot-after 4 --exit-after 5
cargo run -- --window 1024x768 --scale-factor 1 --screenshot screenshots/art-home-001-1024x768.png --screenshot-after 4 --exit-after 5
```

- Senza `--autoplay`, che salta il menu. `--scale-factor 1` tiene la finestra in pixel fisici,
  così canvas e UI hanno le scale della tabella anche su uno schermo al 125%. Si riporta la
  misura vera di ogni screenshot: una finestra più grande dello schermo può essere ridotta dal
  sistema.
- Negli screenshot: player e compagni interi e non coperti da logo o pannello; logo e pannello
  leggibili sopra la fascia sinistra; a 1024×768 nulla di essenziale tagliato.
- Manifest: `ui/home_background.png` con la misura nuova, `source`
  `assets-src/illustrations/home_background.png` e `license` dalla prima riga del
  `.license.txt`; `git status` con il `.webp` tolto e `.png` e `.license.txt` aggiunti.
- Prove: `docs/evidence/home_page_16x9_1280x720.png`, `_1920x1080`, `_1024x768` (in `.jpg`
  se pesano troppo, come le prove web).
- Documenti: in ART_BIBLE ("Illustrazioni") e in `ASSET_LICENSES.md` la home non è più
  provvisoria; la riga di D-063 passa alla sezione delle richieste ChatGPT.
- Se a 1024×768 il gruppo risulta tagliato con il `FOCUS` attuale: segnalazione
  all'orchestratore per il gameplay-engineer, non un ritocco dell'immagine.

## Esito INGEST v1 (2026-10-30)

File: `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png`, 3.093.663 byte. Comando di controllo del
workflow sulla cartella:

```text
ART-HOME-001_v1.png: PNG 1672x941 RGB, ICC nessuno
  alpha 255..255 (a 0: 0 px, fra 1 e 254: 0 px)
  proporzione 1.7768 (16:9 = 1.7778, 3:2 = 1.5000, 4:3 = 1.3333)
```

- **Ritaglio: nessuno.** È 16:9 al pixel: per 1672 px di larghezza il 16:9 esatto è alto 940,5
  px; il comando di ritaglio darebbe 1672×940 (1,7787), lontano dal 16:9 quanto 941 ma con una
  riga in meno. Al "cover" la differenza è 0,6 px a 1920×1080.
- **Bordi:** nessuna cornice né banda (salto fra le 2 righe/colonne esterne e quelle interne ≤ 3,3
  livelli su 255, varianza alta come il resto). In basso a sinistra e a destra fogliame in primo
  piano sfocato, come in REF-1.
- **Peso del build, stimato in memoria** con la stessa logica di `scripts/art_illustration.py`
  (nessun file scritto): RGB 2,80 MiB, 256 colori 0,84 MiB (errore medio 0,90, max 18), quindi
  `indexed-256`. Il dato vero è del VERIFY.

Misure, in % del fotogramma 1672×941 (righello sugli ingrandimenti):

| Elemento | Pixel | % |
|---|---|---|
| gruppo, estremi x | 829 (coda-foglia di Foglin) – 1480 (fiamma di Fiammetto) | x 49,6–88,5 |
| testa più alta (cappello) | y 528 | y 56,1 (orecchie-foglia di Foglin y 65,8) |
| piedi | player 801–811, Ondino 810, Fiammetto 827, Foglin seduto 785 | y 85,1–86,2 / 86,1 / 87,9 / 83,4 (in REF-1 Foglin è a 83,3) |
| altezza del player | 528–811 = 283 px | 30,1 |
| castello | x 1023–1200, y 192–323 | x 61,2–71,8, y 20,4–34,3 |
| portale (arco di pietra) | x 1309–1530, y 190–432; rovine accanto fino a x 1546 | x 78,3–91,5, y 20,2–45,9; rovine fino a 92,5; spirale x 82,8–87,6 |
| cascate | 1: x 1010–1050, y 452–510; 2: x 1140–1205, y 397–542; 3 (sotto il ponte di pietra): x 1252–1330, y 550–630; 4 (grande a destra): x 1498–1537, y 600–733 | 1: x 60,4–62,8, y 48,0–54,2; 2: x 68,2–72,1, y 42,2–57,6; 3: x 74,9–79,5, y 58,4–67,0; 4: x 89,6–91,9, y 63,8–77,9 |
| runa a forma di "M" sulla chiave dell'arco | x 1413–1428, y 292–305 | x 84,5–85,4, y 31,0–32,4 (c'è già in REF-1) |

Dettaglio e contrasto (media di `FIND_EDGES`, deviazione standard della luminanza, saturazione
media HSV, 0–255): fascia x 0–45% 27,8 / 64,1 / 161; x 45–100% 28,7 / 58,0 / 140; area del logo
x 0–52% y 0–36% 17,6 / 63,4 / 159 contro 19,2 / 48,1 / 126 a destra; area del pannello a
1920×1080 34,7 / 60,3 / 160, quanto l'area del gruppo (35,1 / 57,4 / 152).

Copie di controllo (ignorate da git) in `.art-inbox/ART-HOME-001/check/`:
`ART-HOME-001_v1_check.png` (aree del piano con legenda e gruppo misurato),
`ART-HOME-001_v1_home_1920.png` (cover a 1920×1080, logo ×4 a (56, 24), pannello x 88–688 y
396–1030), `ART-HOME-001_v1_home_1024.png` (cover con `FOCUS` (0.65, 0.75): immagine 1364,6×768 in
(−221,4, 0), visibile x 16,2–91,3%; logo ×2 a (28, 12), pannello x 44–344 y 198–515). Pannello
simulato #1b2640 al 92% con bordo #9a743c; niente scritte del pannello, versione e suggerimento.

| Criterio | Esito | Dato |
|---|---|---|
| Una immagine 16:9, lato lungo 1448–2048 | OK | 1672×941, 1,7768, una sola scena, niente collage; nessun ritaglio (sopra) |
| Alpha e profilo | OK | `RGB`, alpha 255..255 misurata, ICC assente |
| Niente testo, logo, firme, watermark, cornici, pannelli; nessuna traccia di logo, menu, "v1.0.0" | DUBBIO | angoli, cielo, area del vecchio logo, del menu e di "v1.0.0" puliti; ma fra le rune del portale c'è un segno che si legge come "M" (e due che ricordano lettere), già presenti in REF-1 |
| Quattro personaggi fedeli, solo loro; niente mantello, armi, oggetti in mano | OK | dettagli dell'elenco tutti presenti; mani vuote nei guanti; l'oggetto grigio al fianco destro del player è anche in REF-1; nessuna creatura o persona in più (villaggio, cielo, bordo destro, primo piano controllati) |
| Gruppo x 50–88%, teste non sopra y 50%, piedi y 85–88%, player 28–32%, tutti interi | OK (al limite) | x 49,6–88,5% (7 e 9 px fuori, nessuna area della UI li copre); testa più alta y 56,1%; piedi 85,1–87,9%, Foglin seduto 83,4% come in REF-1; player 30,1%; tutti interi |
| Fascia x 0–45% calma, senza personaggi, punti di interesse o pannelli; niente punti di interesse in x 0–52% y 0–36% | NO | niente personaggi né pannelli; ma la fascia non è più calma del resto (bordi 27,8 contro 28,7, contrasto 64,1 contro 58,0, saturazione 161 contro 140): tronco contrastato, villaggio col campanile rosso a x 22–29% y 53–64%, ponte di legno a x 36–48%. In x 0–52% y 0–36% solo fronde, cielo e montagne lontane |
| Castello, portale, cascate in x 40–90%, y 5–55%; castello e portale oltre x 50% | NO (parziale) | castello OK; portale fino a x 91,5% (rovine 92,5%); cascate 1 e 2 dentro (la 2 scende a y 57,6%), 3 e 4 sotto y 55% e la 4 fino a x 91,9%, come in REF-1 dove le cascate stavano a y ≈ 52–71% |
| Niente di essenziale in x 0–16% e 91–100% | DUBBIO | a sinistra solo tronco, fiori, sassi; a destra l'ultimo bordo dell'arco (91,3–91,5%), le rovine e metà della cascata 4: a 1024×768 l'arco arriva al bordo dello schermo, spirale intera |
| Nessun elemento di franchise | OK | nessuna sfera bicolore, logo o UI di altri giochi; l'impostazione della scena è quella del concept di D-063 (rischio di somiglianza di D-050), la v1 non aggiunge nulla |
| Stile coerente con il concept | OK | stessa resa di REF-1 (pseudo pixel art, contorni scuri, luce diurna, sfocatura del primo piano agli angoli bassi); ambiente non più tenue dei personaggi, come in REF-1 |
| PNG del build ≤ 4 MiB | OK (stima) | 0,84 MiB a 256 colori, 2,80 MiB in RGB; da confermare al VERIFY |
| Conferma esplicita dell'utente | in attesa | — |

Anteprime simulate: a 1920×1080 e a 1024×768 gruppo intero e scoperto, logo leggibile sulle
fronde grazie al suo contorno, pannello sopra villaggio e staccionata, castello e portale visibili.
I due NO non si vedono come difetti nelle anteprime; contano se si vuole la fascia sinistra davvero
calma e i landmark tutti dentro il riquadro.

## PROMOTE v1 (2026-10-30)

- `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png` copiato byte per byte in
  `assets-src/illustrations/home_background.png` (`cmp` senza differenze; SHA-256
  `f44fd9abb7ee5d0f7400210297ef5edaedc9e51121030dd35b0b2334213798e7` per entrambi, 3.093.663
  byte). Nessun ritocco e nessun ritaglio. Nessun profilo ICC né chunk `sRGB`/`gAMA`: sRGB per
  convenzione, come vuole il build.
- `assets-src/illustrations/home_background.webp` tolto dalla cartella (resta nella storia di git,
  commit `0ef0ad2`): due sorgenti con lo stesso nome fermerebbero il build.
- `assets-src/illustrations/home_background.license.txt`: tre righe di commento (provenienza,
  copia, hash) e la riga della licenza del manifest.

## Esito VERIFY v1 (2026-10-30)

`python scripts/build-assets.py` (uscita 0):

```text
  home_background.png: RGB 2.80 MiB, 256 colori 0.84 MiB (errore medio 0.90, max 18) -> indexed-256
  ...
  ui/home_background.png           1672x941
OK: 202 asset generati in assets/
```

`python scripts/validate-assets.py` (uscita 0):

```text
OK: 202 PNG e 2 font controllati, 17 sprite citati in game-data/, 0 errori
```

Voce del manifest: `ui/home_background.png`, `kind` `illustration`, `size` [1672, 941], `frames`
1, `encoding` `indexed-256`, `source` `assets-src/illustrations/home_background.png`, `license`
`generated by the user with ChatGPT from the approved concept of D-063 (ART-HOME-001 v1, approved
by the user on 2026-10-30), rights declared by the user (D-065)`. PNG del build: modo `P`, 256
colori, 875.983 byte (0,84 MiB, sotto i 4 MiB del validatore).

`git status` dopo la PROMOTE: ` D assets-src/illustrations/home_background.webp`,
`?? assets-src/illustrations/home_background.png`, `?? assets-src/illustrations/home_background.license.txt`,
` M assets/manifest.json`, ` M assets/ui/home_background.png`.

Gioco nativo, home senza `--autoplay`, `--scale-factor 1 --no-watch`, screenshot dopo 4 s (uscita 0
per tutte e tre). Misure vere degli screenshot uguali alle finestre chieste: nessuna riduzione del
sistema. Posizioni a schermo calcolate con `cover()` e `FOCUS` (0.65, 0.75) del codice, poi
controllate sugli screenshot:

| Finestra | Screenshot | Immagine a schermo | Parte visibile | Gruppo a schermo (coda di Foglin – fiamma di Fiammetto; cappello) | Logo / pannello (UI) |
|---|---|---|---|---|---|
| 1280×720 | 1280×720 | 1280,0×720,4 in (0, −0,3) | tutta | x 635–1133; y 404 | logo x 28–472 y 12–192, pannello x 44–344 |
| 1920×1080 | 1920×1080 | 1920,0×1080,6 in (0, −0,4) | tutta | x 952–1700; y 606 | logo x 56–944 y 24–384, pannello x 88–688 |
| 1024×768 | 1024×768 | 1364,6×768,0 in (−221,4, 0) | x 16,2–91,3% | x 455–987; y 431 | logo x 28–472 y 12–192, pannello x 44–344 |

- **Player e compagni:** interi e scoperti a tutte e tre le misure. Il margine più stretto è a
  1920×1080, fra il bordo destro del logo (x 944, che finisce a y 384) e la coda di Foglin (x 952,
  orecchie-foglia a y ≈ 711): nessuna sovrapposizione. A 1024×768 la fiamma di Fiammetto sta a
  37 px dal bordo destro.
- **Logo e pannello:** leggibili. Il logo sta sulle fronde e sul cielo, il pannello sopra il
  villaggio e la staccionata. Dietro non si vede più nulla di dipinto: niente logo né menu doppi
  come nella prova provvisoria `docs/evidence/home_page_provvisoria.png`.
- **Angoli bassi:** "v0.1.0" a sinistra si legge sul fogliame scuro. "W/S o frecce · Invio · T
  talenti" a destra si legge grazie all'ombra del font; ha meno contrasto nella parte sinistra,
  sopra le rocce illuminate (controllato su ingrandimenti ×2–×3).
- **1024×768:** nulla di essenziale tagliato. Gruppo intero, castello intero, spirale del portale
  intera (bordo destro a x 974). Restano fuori, come mostrato all'INGEST e accettato con la v1,
  gli ultimi ~3 px dell'arco di pietra (x 1027), le rovine accanto (x 1040) e metà della cascata
  4. Il `FOCUS` non va ritoccato.
- **Nearest-neighbor:** non si applica all'illustrazione, resa con filtro lineare come da
  richiesta. Il logo resta a texel nativi, come prima.
- Nel log di ogni avvio c'è `WARN bevy_render::texture::gpu_image: No previous asset to copy from
  for image ... size: (640, 360, 1)`: è la texture del canvas, non l'illustrazione.

Prove (JPEG qualità 85, 4:2:0, perché i PNG pesano 1,7–3,6 MB):
`docs/evidence/home_page_16x9_1280x720.jpg` (359.590 byte),
`docs/evidence/home_page_16x9_1920x1080.jpg` (595.433 byte),
`docs/evidence/home_page_16x9_1024x768.jpg` (303.238 byte). Prima: `docs/evidence/home_page_provvisoria.png`.
Screenshot di lavoro in `screenshots/art-home-001-<misura>.png` (ignorata da git).

## Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| v1 | `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png` (3.093.663 byte); copie in `check/` | 2026-10-30 | PNG 1672×941 `RGB`, ICC assente, alpha 255..255, 1,7768 (16:9 al pixel, nessun ritaglio); niente testo, logo, menu, firme; personaggi fedeli, nessuna creatura in più; gruppo x 49,6–88,5%, testa y 56,1%, piedi y 83,4–87,9%, player 30,1%; NO fascia sinistra non più calma; NO portale fino a x 91,5% e cascate 3–4 sotto y 55%; DUBBIO runa a "M"; build stimato 0,84 MiB | **approvata** il 2026-10-30, dopo l'esito dell'INGEST e le anteprime di `check/`: "Approvo v1" (messaggio dell'utente nella sessione con l'orchestratore). Accetta i due NO e i DUBBIO; la v2 delle "Revisioni" non serve. Promossa e `INTEGRATED` il 2026-10-30 (sopra) |

## Revisioni

### v2 (preparata il 2026-10-30, da usare solo se l'utente non approva la v1; non usata, la v1 è approvata)

Correzioni: fascia sinistra più calma, rune senza lettere, portale e cascata grande dentro il 90%
della larghezza. L'altezza delle cascate 3 e 4 (sotto y 55%) non si corregge: è quella di REF-1,
non sta sotto la UI e spostarla vorrebbe dire ridisegnare le scogliere, con il rischio di perdere
i personaggi. Da usare nella stessa chat della v1, se possibile.

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego un'immagine: è l'ultima versione dello sfondo della pagina iniziale del mio videogioco, un'illustrazione in pixel art in formato orizzontale 16:9. Va quasi bene. Modificala partendo proprio da questa immagine, senza rifarla da capo: stessa scena, stesso formato 16:9, almeno la stessa risoluzione, stessa inquadratura e stesso stile. Cambia solo questi tre punti.

1. Fascia sinistra, dal bordo sinistro fin quasi a metà della larghezza: nel gioco sopra ci vanno il titolo e il menu, quindi deve essere più calma. Meno dettagli e meno contrasto, colori un po' più chiari e meno saturi, come velati da una leggera foschia: il grande tronco e le fronde in alto a sinistra con meno foglie singole e luci meno forti; il villaggio con il campanile rosso più tenue e meno dettagliato; la staccionata, i sassi e i fiori in basso a sinistra meno contrastati. Restano gli stessi elementi. Non aggiungere pannelli, riquadri, cornici o spazi vuoti.
2. Le rune luminose della porta di pietra non devono sembrare lettere o numeri: oggi il simbolo in cima all'arco, sopra il portale, sembra una "M" e altri sembrano lettere. Sostituiscili con segni astratti (linee, punti, piccoli rombi, spirali), sempre azzurri e luminosi.
3. Sposta un po' verso sinistra la porta di pietra con il portale e le rovine accanto, e la cascata grande vicino al bordo destro, così che finiscano prima del 90% della larghezza contata da sinistra (oggi arrivano a circa il 92%). A destra la valle si allarga di conseguenza con alberi, scogliera e cielo.

TUTTO IL RESTO IDENTICO
- I quattro personaggi esattamente come sono, nella stessa posizione e alla stessa scala: il ragazzo di spalle con cappello rosso, zaino marrone e salopette blu; la volpe arancione con le foglie al posto di orecchie e coda; la lontra blu con le bolle; la lucertola rosso-arancio con la cresta grigia e la fiamma sulla coda. Non spostarli, non ridisegnarli, non cambiarne colori, pose o proporzioni.
- Castello, cielo, nuvole, montagne, lago, fiume, ponti, casette e il sentiero d'erba con le margherite: invariati.
- Pixel art 16-bit luminosa, pixel netti, luce del giorno; immagine piena, senza trasparenza.

DIVIETI
- Niente testo, lettere, numeri, logo, firme, watermark; niente cornici, bottoni, pannelli, icone o interfaccia di gioco.
- Niente elementi di giochi, cartoni o marchi esistenti: nessuna creatura conosciuta, nessuna sfera bicolore rossa e bianca, nessun logo o interfaccia di altri giochi.
- Nessuna creatura o persona in più oltre ai quattro personaggi; al ragazzo niente mantello, armi o oggetti in mano.

Consegna una sola immagine, senza varianti affiancate né collage.
===== FINE PROMPT =====
```

Allegare: `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png` (la v1, come reference della modifica).
Salvare come: `.art-inbox/ART-HOME-001/ART-HOME-001_v2.png`. Se si sceglie questa via, la v1 diventa
"scartata" nel registro e la richiesta torna `REQUESTED`.

## Provenienza e licenza

- **Versione promossa e file:** v1, `.art-inbox/ART-HOME-001/ART-HOME-001_v1.png` →
  `assets-src/illustrations/home_background.png` (copia byte per byte, SHA-256
  `f44fd9ab…3798e7`) → `assets/ui/home_background.png` (build).
- **Generata il / approvata il:** 2026-10-30 (file salvato dall'utente nell'inbox) / 2026-10-30
  (frase dell'utente: "Approvo v1", scritta nella sessione con l'orchestratore).
- **Strumento:** ChatGPT, account dell'utente.
- **Reference usate:** REF-1 `assets-src/illustrations/home_background.webp`, il concept di D-063
  (o la copia PNG senza perdita in `ref/`; quale delle due sia stata allegata non è registrato).
- **Cambiato nel progetto:** nessun ritocco del contenuto e nessun ritaglio; nessun profilo ICC,
  quindi sRGB per convenzione (nessuna conversione); il build la codifica a 256 colori con
  dithering Floyd–Steinberg (`indexed-256`, 0,84 MiB contro 2,80 MiB in RGB).
- **Diritti:** dichiarati dall'utente secondo i termini di OpenAI che ha accettato (D-065). Ritrae
  il player e tre compagni dell'utente: vale il rischio di somiglianza accettato in D-050.
- **Riga del manifest** (`home_background.license.txt`, letta dal build):
  `generated by the user with ChatGPT from the approved concept of D-063 (ART-HOME-001 v1, approved by the user on 2026-10-30), rights declared by the user (D-065)`
- **Voce in `ASSET_LICENSES.md`:** sì, sezione "Asset da richieste ChatGPT (D-065)"; la tabella di
  D-063 descrive ora il concept come reference non più distribuita.
