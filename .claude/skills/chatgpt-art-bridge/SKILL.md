---
name: chatgpt-art-bridge
description: Handoff manuale con ChatGPT per l'arte di Bonk / Companion Survivors (D-065). Usala quando serve un'illustrazione, uno sfondo, il concept di un compagno o di un boss, un ritratto elaborato o una grande decorazione UI, e quando l'utente scrive "<ID> pronto" (es. "ART-HOME-001 pronto") o approva o respinge una versione. Argomento: l'ID della richiesta oppure la descrizione dell'asset.
---

# ChatGPT art bridge

Procedura completa e unica: `docs/art/CHATGPT_ART_WORKFLOW.md`. Protocollo del ruolo:
`.claude/agents/pixel-art-director.md`, sezione 5. Decisione: D-065 in `docs/DECISIONS.md`.
Questa skill dice solo come la sessione principale (orchestrator) guida il giro.

Non esiste nessuna integrazione con ChatGPT: niente API, chiavi, Codex, MCP remoti,
automazione della chat. L'immagine la fa l'utente nella sua chat. Mai dire che un'immagine è
stata generata, salvata o approvata se non l'hai vista nel file o letta dall'utente.

## Richiesta nuova (argomento: descrizione dell'asset)

1. Controlla in `docs/art/requests/` che non ci sia già una richiesta equivalente: se c'è,
   riusala.
2. Delega al subagent `pixel-art-director` il passo PREPARE: sceglie la via (`LOCAL_PIXEL`,
   `CHATGPT_CONCEPT`, `CHATGPT_EDIT`) e, se è ChatGPT, scrive `docs/art/requests/<ID>.md` dal
   template `docs/art/ART_REQUEST_TEMPLATE.md`. Con `LOCAL_PIXEL` il lavoro continua nella
   pipeline `.px` e questa skill finisce qui.
3. Il report del subagent non arriva all'utente: riporta tu, per intero e senza riassumerlo, il
   blocco fra `===== INIZIO PROMPT DA INCOLLARE IN CHATGPT =====` e `===== FINE PROMPT =====`,
   le reference da allegare (percorsi nel repo) e il nome del file da salvare.
4. Gate: "Incolla il prompt in ChatGPT, allega le reference indicate, genera e approva
   l'immagine; salva il PNG in `.art-inbox/<ID>/` e poi scrivimi `<ID> pronto`". Chiudi il
   turno: niente attese in background né controlli a intervalli. Il lavoro di codice che non
   dipende dall'immagine può andare avanti.

## Ripresa (argomento: `<ID>`, o il messaggio `<ID> pronto`)

1. Leggi `docs/art/requests/<ID>.md` e guarda davvero `.art-inbox/<ID>/`. Se il file non c'è,
   dillo e fermati.
2. Delega al `pixel-art-director` il passo INGEST: controlli del workflow, stato `GENERATED`.
   Mostra all'utente l'esito vero dei controlli (ed eventuale ritaglio). Se qualcosa non va,
   riporta per intero il prompt di revisione e torna al gate.
3. Chiedi all'utente di confermare la versione esatta (`<ID>_v<N>.png`). Solo con il suo sì
   esplicito la richiesta passa ad `APPROVED`.
4. PROMOTE e VERIFY dal `pixel-art-director`: sorgente in `assets-src/`, provenienza,
   `python scripts/build-assets.py`, `python scripts/validate-assets.py`, prove a schermo della
   richiesta. Se serve codice (per esempio il `FOCUS` della home) assegnalo al suo owner.
5. Aggiorna `docs/PROGRESS.md` e lo stato in `docs/BACKLOG.md`. Commit solo se l'utente lo
   chiede.
