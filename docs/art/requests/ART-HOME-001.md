# ART-HOME-001 — Sfondo della home nativo 16:9, senza logo né menu

Richiesta ChatGPT (D-065), procedura in `docs/art/CHATGPT_ART_WORKFLOW.md`. Dry run del workflow:
nessuna immagine esiste ancora.

| Campo | Valore |
|---|---|
| ID | `ART-HOME-001` |
| Tipo | `illustration` |
| Via | `CHATGPT_EDIT`, perché serve continuità con il concept approvato di D-063: stessi personaggi, stessa scena e stesso stile; cambiano solo il formato e le scritte dipinte |
| Stato | `REQUESTED` (in attesa dell'utente) |
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
| REF-1 (obbligatoria) | `assets-src/illustrations/home_background.webp` (1448×1086, 4:3, RGB, sRGB) | il concept approvato di D-063: scena, quattro personaggi, stile | il logo "Companion Survivors", il pannello del menu con voci e icone, "v1.0.0", il formato 4:3 |

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

## Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| — | nessun file in `.art-inbox/ART-HOME-001/` (solo la copia di REF-1 in `ref/`) | 2026-10-30 | — | — |

## Revisioni

Nessuna.

## Provenienza e licenza (da compilare dopo l'import)

- **Versione promossa e file:** …
- **Generata il / approvata il:** … / … (frase dell'utente: "…")
- **Strumento:** ChatGPT, account dell'utente.
- **Reference usate:** REF-1 `assets-src/illustrations/home_background.webp` (o la copia PNG in
  `ref/`).
- **Cambiato nel progetto:** … (ritaglio, profilo colore, codifica del build)
- **Diritti:** dichiarati dall'utente secondo i termini di OpenAI che ha accettato (D-065). Ritrae
  il player e tre compagni dell'utente: vale il rischio di somiglianza accettato in D-050.
- **Riga del manifest** (`home_background.license.txt`), proposta:
  `generated by the user with ChatGPT from the approved concept of D-063 (ART-HOME-001 v<N>, approved by the user on <AAAA-MM-GG>), rights declared by the user (D-065)`
- **Voce in `ASSET_LICENSES.md`:** no (da aggiungere alla PROMOTE).
