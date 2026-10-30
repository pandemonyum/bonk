# Arte con ChatGPT: handoff manuale (D-065)

Unica fonte del dettaglio della procedura. L'unico canale per l'arte fatta con l'IA è la
conversazione ChatGPT dell'utente, a mano. Il pixel-art-director prepara la richiesta e controlla
il file; l'utente genera, sceglie e approva. La skill di Claude Code
`.claude/skills/chatgpt-art-bridge/SKILL.md` rimanda a questo file.

## Quale via

Ogni richiesta grafica dichiara la via e perché.

| Via | Quando | Cosa succede |
|---|---|---|
| `LOCAL_PIXEL` | tile, ritocchi deterministici, palette mapping, icone semplici, tutto quello che la pipeline `.px` fa bene | lavoro in locale, nessuna richiesta |
| `CHATGPT_CONCEPT` | sfondi e illustrazioni complesse, design di compagni o boss nuovi, ritratti elaborati, evoluzioni artistiche, grandi decorazioni UI | richiesta nuova e gate dell'utente |
| `CHATGPT_EDIT` | revisione di un concept già approvato quando serve continuità visiva | modifica a partire dalla sua immagine, non un'immagine nuova slegata |

Il tipo della richiesta (`illustration`, `sprite_concept`, `ui_art`, `edit`) decide la PROMOTE.

## Nomi e cartelle

- **ID** `ART-<AREA>-<NNN>`: area breve in maiuscolo (`HOME`, `COMP`, `BOSS`, `UI`…), numero a
  tre cifre per area. Se esiste una richiesta equivalente si riusa, con una versione nuova.
- **Richiesta** `docs/art/requests/<ID>.md`, dal modello `docs/art/ART_REQUEST_TEMPLATE.md`. È
  nel repository.
- **Inbox** `.art-inbox/<ID>/<ID>_v<N>.png` (v1, v2… per le revisioni), ignorata da git.
- **`.art-inbox/<ID>/ref/`**: solo copie di immagini già approvate, preparate dall'agente quando
  il file del repo non si allega così com'è (uno sprite 28×24 ingrandito ×8 nearest, un formato
  che la chat non accetta).
- **`.art-inbox/<ID>/check/`**: copie di controllo dell'agente (aree sicure disegnate sopra).
- **Ritaglio** `.art-inbox/<ID>/<ID>_v<N>_16x9.png` (o il formato della richiesta), fatto
  all'INGEST.
- Le bozze non entrano mai nel repository: ci entra solo la versione approvata, alla PROMOTE.

Reference ingrandita per uno sprite (crea prima la cartella `ref/`):

```text
python -c "import sys; from PIL import Image; im = Image.open(sys.argv[1]); im.resize((im.width * 8, im.height * 8), Image.Resampling.NEAREST).save(sys.argv[2])" assets/ui/portrait_ember.png .art-inbox/ART-COMP-001/ref/portrait_ember_x8.png
```

## Stati

| Stato | Quando |
|---|---|
| `REQUESTED` | richiesta scritta e prompt pronto; nessuna immagine controllata |
| `GENERATED` | l'utente ha scritto `<ID> pronto` e l'agente ha aperto il file nell'inbox |
| `APPROVED` | l'agente ha mostrato l'esito dei controlli (ed eventuale ritaglio) e l'utente ha confermato esplicitamente quella versione |
| `INTEGRATED` | promossa; build, validazione e prove a schermo passano; licenza registrata |
| `REJECTED` | richiesta chiusa senza asset: l'utente rinuncia o la via diventa `LOCAL_PIXEL` |
| `BLOCKED` | manca qualcosa che l'agente non risolve da solo (file assente o illeggibile, la chat rifiuta, serve una decisione dell'utente o di un altro owner): si scrive cosa manca e chi sblocca |

Una versione scartata non chiude la richiesta: nel registro diventa "scartata", la richiesta
torna `REQUESTED` con il prompt di revisione e si aspetta `v<N+1>`.

Registro delle versioni, in fondo a ogni richiesta:

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|

## 1. PREPARE (agente)

- Sceglie via e tipo, scrive o aggiorna `docs/art/requests/<ID>.md` dal modello.
- Prompt in italiano, un solo blocco, che si regge da solo: ChatGPT non legge percorsi locali né
  vede il gioco. Le regole dell'ART_BIBLE che servono si riscrivono nel prompt, mai "segui
  l'ART_BIBLE".
- Una sola immagine per richiesta, mai uno spritesheet o un foglio animato.
- Reference: percorso esatto nel repo, cosa mostra, cosa non copiare. Niente logo né screenshot
  con la UI, se la richiesta non riguarda proprio quelli.
- Nei divieti i franchise si descrivono senza nominarli.
- Nel report: il blocco del prompt, l'elenco delle immagini da allegare, il nome del file da
  salvare.

Il blocco è delimitato esattamente così:

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
...
===== FINE PROMPT =====
```

## 2. HUMAN GATE (utente)

L'agente si ferma solo su ciò che dipende dall'immagine e scrive:

> Incolla il prompt in ChatGPT, allega le reference indicate, genera e approva l'immagine; salva
> il PNG in `.art-inbox/<ID>/` e poi scrivimi `<ID> pronto`

L'utente:
- incolla il prompt e allega le reference (per `CHATGPT_EDIT` nella chat del concept, se c'è);
- genera, sceglie, scarica e salva come `.art-inbox/<ID>/<ID>_v<N>.png` (un altro formato si
  salva così com'è, con lo stesso nome);
- scrive `<ID> pronto`.

Lo stato resta `REQUESTED`. L'agente non resta in attesa e non dice che l'immagine esiste. La
scelta fatta in ChatGPT non è ancora `APPROVED`.

## 3. INGEST (agente)

Alla ripresa apre davvero i file in `.art-inbox/<ID>/`. Nessun file: resta `REQUESTED` e lo dice.
Con il file: `GENERATED`.

| Controllo | Come |
|---|---|
| formato e nome | PNG (un altro formato si annota); nome `<ID>_v<N>` |
| misura e proporzioni | quelle della richiesta; illustrazioni ≤ 2048 px per lato |
| modalità colore | `RGB` o `RGBA`; profilo ICC sRGB o assente (un altro, per esempio Display P3, si converte in sRGB alla PROMOTE e si annota) |
| alpha | misurata: estremi e pixel a 0 e fra 1 e 254. Mai dire "trasparente" né "opaca" senza la misura |
| bordi | niente cornici, bande, vignettature, firme negli angoli |
| composizione e aree sicure | le percentuali della richiesta, su una copia in `check/` con le aree disegnate |
| testo, logo, firme, watermark | a vista sull'immagine ingrandita: cielo, angoli, insegne, oggetti |
| divieti e reference | quelli della richiesta: personaggi fedeli, niente franchise, niente creature in più |

Misura, modo, profilo, alpha e proporzione (provato in Git Bash e PowerShell 5.1; accetta
cartelle e file):

```text
python -c "
import io, sys
from pathlib import Path
from PIL import Image, ImageCms
paths = [Path(a) for a in sys.argv[1:]]
files = [f for p in paths for f in (sorted(p.iterdir()) if p.is_dir() else [p])]
for f in [f for f in files if f.suffix.lower() in ('.png', '.webp', '.jpg', '.jpeg')]:
    with Image.open(f) as im:
        w, h = im.size
        alpha = im.convert('RGBA').getchannel('A')
        lo, hi = alpha.getextrema()
        hist = alpha.histogram()
        icc = im.info.get('icc_profile')
        icc = ImageCms.getProfileDescription(ImageCms.ImageCmsProfile(io.BytesIO(icc))).strip() if icc else 'nessuno'
        print(f'{f.name}: {im.format} {w}x{h} {im.mode}, ICC {icc}')
        print(f'  alpha {lo}..{hi} (a 0: {hist[0]} px, fra 1 e 254: {sum(hist[1:255])} px)')
        print(f'  proporzione {w / h:.4f} (16:9 = 1.7778, 3:2 = 1.5000, 4:3 = 1.3333)')
" .art-inbox/<ID>
```

Ritaglio a 16:9, se la proporzione è diversa e la richiesta lo prevede. Senza ridimensionare;
centrale per default, il terzo argomento sposta la finestra (0 in alto o a sinistra, 1 in basso
o a destra):

```text
python -c "
import sys
from PIL import Image
src, dst = sys.argv[1], sys.argv[2]
pos = float(sys.argv[3]) if len(sys.argv) > 3 else 0.5
with Image.open(src) as im:
    w, h = im.size
    cw, ch = min(w, round(h * 16 / 9)), min(h, round(w * 9 / 16))
    x, y = round((w - cw) * pos), round((h - ch) * pos)
    im.crop((x, y, x + cw, y + ch)).save(dst, icc_profile=im.info.get('icc_profile'))
print(f'{w}x{h} -> {cw}x{ch} ({cw / ch:.4f}), angolo in alto a sinistra ({x}, {y})')
" .art-inbox/<ID>/<ID>_v<N>.png .art-inbox/<ID>/<ID>_v<N>_16x9.png
```

Poi l'agente mostra all'utente l'esito dei controlli, il ritaglio e la copia di `check/`, e
chiede la conferma esplicita di quella versione ("approvo `<ID>` v<N>" o simile). Solo allora
`APPROVED`, con data e frase nel registro.

**Revisione**, se qualcosa non va: un prompt con le sole correzioni, non tutto da capo, da usare
nella stessa chat se possibile, allegando l'ultima versione come reference. Va nella richiesta
(sezione "Revisioni") con il nome del file `v<N+1>`; lo stato torna `REQUESTED`.

## 4. PROMOTE (agente, solo dopo `APPROVED`)

| Tipo | Dove va |
|---|---|
| `illustration` | `assets-src/illustrations/<nome>.png` (o `.webp`), al posto dell'eventuale sorgente con lo stesso nome (due sorgenti per lo stesso nome fermano il build), più `<nome>.license.txt` |
| `sprite_concept` | l'immagine resta una reference in `.art-inbox/<ID>/`, fuori dal repo come i fogli originali di D-050/D-052; lo sprite si ridisegna nella griglia `.px`, con la provenienza in `license:`. Frame, celle, ancore, palette, alpha 0/255 e coerenza dei fotogrammi sono della pipeline locale |
| `ui_art` | `.px` se deve stare alla scala dei texel (D-045: icone, telai, bordi); altrimenti come `illustration` |
| `edit` | come il tipo dell'asset di partenza |

`<nome>.license.txt`: la prima riga non vuota che non inizia con `#` diventa la `license` del
manifest (in inglese, come le altre); un file vuoto ferma il build. Per esempio:

```text
# Provenienza: docs/art/requests/<ID>.md e ASSET_LICENSES.md
generated by the user with ChatGPT (<ID> v<N>, approved by the user on <AAAA-MM-GG>), rights declared by the user (D-065)
```

Mai saltare manifest (lo scrive il build), `ASSET_LICENSES.md` e validatori. Uno sprite nato da
un concept non è pronto per il gioco finché non passa i controlli.

## 5. VERIFY (agente)

- `python scripts/build-assets.py` e `python scripts/validate-assets.py`, output vero nel
  report.
- Le prove a schermo del piano di verifica della richiesta, aprendo i PNG; per sprite e icone
  anche `python scripts/check-pixel-grid.py <png>`.
- Stato `INTEGRATED` nel registro e riga in `ASSET_LICENSES.md` (sezione "Asset da richieste
  ChatGPT").
- Prova in `docs/evidence/`: nomi minuscoli senza spazi, poche immagini; le anteprime di lavoro
  restano in `screenshots/`, ignorata da git.
- Report di tranche di `AGENTS.md` all'orchestratore, che aggiorna `docs/PROGRESS.md` e fa il
  commit solo se l'utente lo chiede.

Dopo `INTEGRATED` una modifica è una versione nuova della stessa richiesta (`CHATGPT_EDIT`), o
una richiesta nuova se cambia l'obiettivo.

## Divieti

- Niente API di OpenAI, chiavi, Codex, MCP remoti, automazione o lettura della chat, accesso
  all'account, servizi a pagamento. L'agente non genera né scarica immagini.
- Nessuna riproduzione di Pokémon, Poké Ball o asset di altri giochi; niente elementi
  riconoscibili di franchise altrui; nessun asset di terzi senza URL, autore e licenza tracciati.
- Non fingere di aver usato ChatGPT; non inventare file, immagini, screenshot o verifiche.
- Non spacciare una grafica procedurale per il risultato di una richiesta complessa.
- Non segnare `APPROVED` senza la conferma dell'utente; non chiamare trasparente un PNG senza
  aver misurato l'alpha.
- Non chiedere a ChatGPT spritesheet o fogli animati pronti: una sola immagine per richiesta.
- Le bozze non entrano nel repository; nessun segreto.
