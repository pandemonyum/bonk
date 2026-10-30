# ART-<AREA>-<NNN> — <titolo breve>

Modello delle richieste ChatGPT (D-065). Copialo in `docs/art/requests/<ID>.md` e compila ogni
voce; procedura in `docs/art/CHATGPT_ART_WORKFLOW.md`.

| Campo | Valore |
|---|---|
| ID | `ART-<AREA>-<NNN>` |
| Tipo | `illustration` / `sprite_concept` / `ui_art` / `edit` |
| Via | `CHATGPT_CONCEPT` / `CHATGPT_EDIT`, perché: … |
| Stato | `REQUESTED` |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (generazione e approvazione) |
| Decisioni | D-065, … |

## Obiettivo

A cosa serve nel gioco o nella UI, dove si vede, cosa il codice ci disegna sopra.

## File

- **Di ritorno:** `.art-inbox/<ID>/<ID>_v1.png` (v2, v3… per le revisioni).
- **Di destinazione proposto:** `assets-src/…` (sorgente) → `assets/…` (uscita del build).
- **Codice che lo usa:** … (non si cambia nella richiesta; un ritocco è dell'owner del codice).

## Formato e vincoli tecnici

- **Dimensioni e proporzioni target:** … (minimo e massimo; validatore: illustrazioni ≤ 2048 px
  per lato e ≤ 4 MiB dopo il build, sprite e UI secondo l'ART_BIBLE).
- **Alpha:** opaca / trasparente (per gli sprite solo 0 o 255, sempre misurata).
- **Palette:** libera (illustrazioni) / "Germoglio" o palette propria (sprite e UI ridisegnati).
- **Resa:** nearest-neighbor a scala intera (sprite, UI) o filtro lineare con "cover"
  (illustrazioni).
- **Canvas, safe area, ancore:** cosa copre la UI e a che misure di finestra, cosa può essere
  tagliato, ancora e riga di base per gli sprite.

## Stile (sintesi autonoma dall'ART_BIBLE)

Le regole che servono, scritte per intero: il prompt non può rimandare all'ART_BIBLE.

## Reference da allegare

| REF | File nel repo | Cosa mostra | Cosa NON copiare |
|---|---|---|---|
| REF-1 | `…` | … | … |

Da non allegare: …

## Da preservare / da evitare

- **Preservare:** …
- **Evitare:** …

## Prompt

```text
===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====
(in italiano, un solo blocco autosufficiente: cosa fare, reference allegate e cosa non copiarne,
contenuto, composizione in parole semplici e percentuali, stile, divieti, una sola immagine)
===== FINE PROMPT =====
```

Gate: "Incolla il prompt in ChatGPT, allega le reference indicate, genera e approva l'immagine;
salva il PNG in `.art-inbox/<ID>/` e poi scrivimi `<ID> pronto`".

## Criteri di approvazione

- [ ] formato, misura e proporzioni
- [ ] alpha misurata come richiesto
- [ ] niente testo, logo, firme, watermark, cornici, UI
- [ ] contenuto fedele alle reference
- [ ] composizione e aree sicure
- [ ] nessun elemento di franchise altrui, nessuna creatura non richiesta
- [ ] stile coerente con l'ART_BIBLE e con le reference
- [ ] conferma esplicita dell'utente su questa versione

## Piano di verifica

- **INGEST:** comando di controllo del workflow, ritaglio se previsto, copia con le aree in
  `.art-inbox/<ID>/check/`.
- **VERIFY:** `python scripts/build-assets.py`, `python scripts/validate-assets.py`, comandi del
  gioco e cosa guardare negli screenshot, prova da mettere in `docs/evidence/`.

## Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| — | — | — | — | — |

## Revisioni

(prompt con le sole correzioni, fra gli stessi delimitatori, e nome del file `v<N+1>`)

## Provenienza e licenza (da compilare dopo l'import)

- **Versione promossa e file:** …
- **Generata il / approvata il:** … / … (frase dell'utente: "…")
- **Strumento:** ChatGPT, account dell'utente.
- **Reference usate:** …
- **Cambiato nel progetto:** … (ritaglio, profilo colore, codifica del build, ridisegno `.px`)
- **Diritti:** dichiarati dall'utente secondo i termini di OpenAI che ha accettato (D-065).
- **Riga del manifest** (`<nome>.license.txt` o `license:` del `.px`): …
- **Voce in `ASSET_LICENSES.md`:** sì / no
