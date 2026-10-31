# HOME_AMBIENT_ART_REQUEST — arte per le animazioni ambientali della home

Richieste ChatGPT (D-065) per le animazioni della pagina iniziale (D-067), procedura in
`docs/art/CHATGPT_ART_WORKFLOW.md`. Il gioco anima la home con nodi UI e uno shader che ridisegnano
i pixel dell'illustrazione dentro maschere ricavate da `scripts/art_home_ambient.py`
(`src/presentation/home_ambient/`).

## Cosa NON serve chiedere

Portale e acqua sono fatti con i pixel che l'illustrazione ha già, quindi non servono immagini
nuove:

| Effetto | Come è fatto | Arte nuova |
|---|---|---|
| Vortice del portale che gira e scende verso il centro | rotazione del vortice dipinto attorno all'occhio della spirale, più bracci di luce procedurali | no: la parte dietro le rovine è ricostruita dallo script lungo le orbite di rotazione (`ui/home_ambient/portal_vortex.png`) |
| Rune che pulsano, luce ciano, particelle | luce aggiunta sui pixel delle rune e attorno all'apertura; particelle di 1–3 texel calcolate dallo shader | no |
| Cascate, schiuma, increspature | l'acqua dipinta scorre in giù dentro la sua maschera (due letture sfalsate, senza salti), strisce chiare per colonna, schiuma che ribolle, cerchi nella pozza | no |
| Riflessi del fiume | trattini di luce e un ondeggiare di mezzo texel lontano dalle rive | no |
| Fiamma di Fiammetto | guizzi verso l'alto dei pixel della fiamma | no |

Personaggi e compagni non si animano: servirebbero sprite coerenti e un'approvazione artistica.

## Proposta opzionale: ART-HOME-002, nuvole che scorrono (P3)

| Campo | Valore |
|---|---|
| ID | `ART-HOME-002` |
| Tipo | `edit` (A) + `ui_art` (B) |
| Via | `CHATGPT_EDIT` per A (stessa scena), `CHATGPT_CONCEPT` per B (sprite nuovi nello stile della scena) |
| Stato | `PROPOSED`: non richiesta, in attesa del via dell'utente |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (generazione e approvazione) + gameplay-engineer (effetto `Clouds` nel layout) |
| Decisioni | D-065, D-067 |

### Perché serve arte

Le nuvole sono dipinte nel cielo e davanti alle montagne. Per farle scorrere bisogna vedere
cosa c'è dietro: serve il cielo senza nuvole (A) e le nuvole separate (B). Spostare i pixel
dipinti senza il fondo pulito lascerebbe buchi o strisciate.

### File

- **Di ritorno:** `.art-inbox/ART-HOME-002/ART-HOME-002_A_v1.png` e `ART-HOME-002_B_v1.png`.
- **Di destinazione proposto:** `assets-src/illustrations/home_sky_clean.png` (A, solo la fascia
  del cielo viene usata, ritagliata e allineata all'INGEST) e `assets-src/illustrations/
  home_clouds.png` (B), con i loro `.license.txt`.
- **Codice che li userebbe:** un tipo `Clouds` in `src/presentation/home_ambient/` (non ancora
  scritto: si scrive solo se le immagini vengono approvate).

### Formato e vincoli tecnici

- **A, cielo pulito:** la stessa illustrazione 16:9 alla stessa misura (1672×941, o il 16:9 più
  grande che ChatGPT dà, poi riportato a 1672×941 all'INGEST). Si usa solo la fascia del cielo:
  da y 0 a circa y 260 (in pixel dell'illustrazione), da x 150 a 1672. Il resto può cambiare:
  non si usa.
- **B, nuvole:** 6 nuvole separate, ognuna larga 120–260 px, su fondo piatto magenta #FF00FF,
  distanti almeno 40 px l'una dall'altra e dal bordo. Il magenta diventa trasparente all'INGEST
  (alpha 0/255, nessun alone).
- **Resa:** filtro lineare con "cover", come l'illustrazione: stesse dimensioni dei pixel e stessi
  contorni delle nuvole di oggi.
- **Safe area:** a sinistra il logo e il pannello coprono x 0–360 circa a 1280×720; le nuvole che
  passano lì dietro non devono attirare l'occhio.

### Reference da allegare

| REF | File nel repo | Cosa mostra | Cosa NON copiare |
|---|---|---|---|
| REF-1 | `assets-src/illustrations/home_background.png` | la scena approvata (ART-HOME-001 v1): cielo, montagne, nuvole, stile | personaggi e compagni non vanno ridisegnati |

### Prompt A (cielo pulito)

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego un'illustrazione pixel art 16:9 di un paesaggio fantasy (REF-1). Modificala così:
togli tutte le nuvole bianche dal cielo e da davanti alle montagne, e al loro posto dipingi il
cielo azzurro e le montagne che stavano dietro, come se la giornata fosse limpida. Le cime, i
pendii e i colori delle montagne devono continuare in modo naturale dove prima c'erano le nuvole.
Non cambiare nient'altro: stessa inquadratura, stessa misura, stessa luce, stesso stile pixel art
16-bit a pixel ben visibili, stessi colori. Non aggiungere testo, logo, firme, cornici, interfaccia
o creature. Una sola immagine 16:9.
===== FINE PROMPT =====
```

### Prompt B (nuvole separate)

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
Ti allego un'illustrazione pixel art 16:9 di un paesaggio fantasy (REF-1). Dipingi 6 nuvole
cumulo separate nello stesso identico stile delle nuvole di questa illustrazione: pixel art 16-bit
a pixel ben visibili, bianche con ombre azzurro-lavanda, contorni morbidi a gradini, luce da
sinistra in alto. Ognuna larga fra 120 e 260 pixel, forme diverse fra loro (due lunghe e basse,
due tondeggianti, due piccole). Mettile su uno sfondo piatto di un solo colore magenta puro
#FF00FF, senza sfumature né ombre sul fondo, distanti almeno 40 pixel l'una dall'altra e dai
bordi. Nessun cielo, nessuna montagna, nessun testo, logo, firma o cornice. Una sola immagine.
===== FINE PROMPT =====
```

Gate: "Incolla i prompt in ChatGPT, allega REF-1 a entrambi, genera e approva le immagini; salva i
PNG in `.art-inbox/ART-HOME-002/` e poi scrivimi `ART-HOME-002 pronto`".

### Criteri di approvazione

- [ ] A: cielo e montagne senza nuvole, senza ridisegnare inquadratura, luce e stile
- [ ] A: le cime ricostruite sono credibili (nessuna montagna nuova, nessun taglio netto)
- [ ] B: 6 nuvole separate su magenta piatto, misure richieste, nessun alone rosa sui bordi
- [ ] B: stile identico alle nuvole della home (pixel, colori, ombre)
- [ ] niente testo, logo, firme, watermark, cornici, UI, creature
- [ ] conferma esplicita dell'utente su ogni versione

### Piano di verifica

- **INGEST:** misura e proporzioni; per A allineamento con l'illustrazione (differenza fuori dalla
  fascia del cielo ignorata, dentro solo dove c'erano nuvole); per B chiave magenta → alpha 0/255,
  ritaglio di ogni nuvola, controllo dei bordi su fondo scuro e chiaro.
- **VERIFY:** `python scripts/build-assets.py`, `python scripts/validate-assets.py`, home a
  1280×720, 1920×1080 e 1024×768 con una raffica di screenshot (`--screenshot-burst`) e una GIF in
  `docs/evidence/`.

### Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| — | — | — | — | — |

## Valutati e non chiesti

- **Fronde che oscillano:** l'albero grande è in alto a sinistra, dietro logo e pannello. Farlo
  muovere distrarrebbe dal menu, e uno spostamento dei pixel dipinti senza un livello separato
  sembrerebbe gelatina. Si riprende solo con un livello delle fronde fatto apposta.
- **Lucciole vicino alla vegetazione:** fatte con lo shader, senza arte (D-067): stelline di
  pixel sopra foglie ed erba, maschere `motes_left` e `motes_right`.
