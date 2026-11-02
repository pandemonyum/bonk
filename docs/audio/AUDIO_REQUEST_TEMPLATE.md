# <ID> — <titolo breve>

Modello delle richieste di musica e ambienti (D-070). Si copia in `docs/audio/requests/<ID>.md` e
si compila ogni voce. La procedura è in `docs/audio/AUDIO_HANDOFF_WORKFLOW.md`.

La richiesta **si regge da sola**: chi compone, chi riceve la commissione o chi sceglie un file
deve trovarci tutto. Quindi:
- le regole tecniche si scrivono per intero, senza rimandare ad altri documenti;
- tonalità e tempi delle tracce vicine si riportano.

Le sezioni 1–8 sono il brief da girare a chi fa il lavoro; le altre servono al progetto. Nessun
nome di giochi, franchise, compositori o brani da imitare.

| Campo | Valore |
|---|---|
| ID | `MUSIC-<AREA>-<NNN>` oppure `AMB-<AREA>-<NNN>` |
| Tipo | `music_loop` / `stinger` / `ambience_loop` |
| File nel gioco | `assets/audio/music/<nome>.ogg` oppure `assets/audio/ambience/<nome>.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per questo file |
| Stato | `REQUESTED` |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3 (musica, stinger) / AUDIO-4 (ambienti) |
| Decisioni | D-069, D-070 |

## Il gioco in breve

Due o tre righe che spiegano il gioco senza nominare altri titoli: gioco d'azione 2D in pixel art
visto dall'alto, ondate di piccole creature, compagni che attaccano da soli, molti effetti sonori
brevi e acuti.

## 1. Dove suona e cosa succede attorno

- Quando parte e quando finisce. Le dissolvenze le fa il gioco: incrociate a potenza costante,
  con le durate.
- Abbassamento automatico: −9 dB in pausa, −6 dB nella scelta dei potenziamenti, attacco
  0,25 s, rilascio 0,5 s.
- Pause e riprese, per esempio la run che riparte da dove era dopo l'orda.
- Quanto a lungo si ascolta di fila e quante volte gira il loop.
- Che cosa suona sopra: effetti brevi, registri occupati, altri loop.

## 2. Carattere e immagini

Immagini a parole: luogo, luce, movimento, stato d'animo. Che cosa deve far sentire, che cosa
no.

## 3. Palette strumentale

Originale, ispirata ai giochi a 16 bit degli anni '90 e adatta al pixel art:
- campioni brevi a bassa risoluzione;
- FM semplice;
- onde quadre, triangolari e rumore;
- poche voci insieme.

Che cosa evitare: per esempio supersaw moderni, orchestre cinematografiche realistiche, voci con
parole, batterie acustiche con grandi stanze.

## 4. Dati musicali

| Voce | Valore |
|---|---|
| BPM | … (tra quelli che danno campioni interi per battito: 2 646 000 / BPM intero) |
| Tonalità e modo | … |
| Metro | … |
| Battute | … |
| Lunghezza esatta del loop | … campioni a 44 100 Hz (… s) |
| Struttura suggerita | … |

Conti, verificati con Python: 44 100 × 60 / BPM = … campioni per battito; × … battiti = …
per battuta; × … battute = **… campioni**.

**Rapporti con le altre tracce:** tonalità e BPM delle tracce che si incrociano con questa, e
perché devono andare d'accordo.

## 5. Regole del loop (oppure del one-shot)

Per un loop:
- il file contiene solo il corpo del loop: niente introduzione, niente finale, niente
  dissolvenze all'inizio o alla fine, nessun silenzio iniziale;
- la lunghezza è quella esatta in campioni, con una tolleranza di ±1 campione;
- la coda del riverbero e degli echi si riporta all'inizio, non si taglia. Due modi:
  - si esporta il brano due volte di seguito e si tiene esattamente il secondo giro;
  - oppure si somma all'inizio la coda che supera la fine;
- l'ultimo campione deve portare al primo come due campioni consecutivi qualunque: stessa
  fase, nessun gradino;
- il primo e l'ultimo campione stanno vicini allo zero (|x| ≤ 0,01, cioè −40 dBFS) in tutti e
  due i canali. Per arrivarci si **ruota** il loop di pochi campioni fino a un passaggio per lo
  zero (la lunghezza non cambia), mai con una dissolvenza. Serve perché un eventuale vuoto al
  giro diventi un silenzio brevissimo e non un clic.

Per uno stinger:
- attacco immediato: il primo campione vicino allo zero, il suono oltre −40 dBFS entro 10 ms;
- coda breve che si chiude da sola, sotto −60 dBFS negli ultimi 10 ms;
- durata massima comprese le code;
- non è un loop.

Per un ambiente:
- l'incrocio per chiudere il loop sta **dentro** il file: si sovrappongono e si incrociano la
  fine e l'inizio di un materiale più lungo. Agli estremi non c'è nessuna dissolvenza;
- nessun evento riconoscibile (un tonfo, un verso) che faccia sentire il giro.

## 6. Formato ed esportazione

- Ogg Vorbis, stereo, 44 100 Hz, qualità 3 (circa 112 kbps nominali, a bitrate variabile).
- Peso massimo: durata × 16 000 byte (128 kbps medi) = … byte.
- Niente infrabassi sotto i 30–35 Hz; compatibile in mono, niente trucchi di fase.
- Master facoltativo ma consigliato: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza dell'Ogg.
- Esportazione con Audacity, per esempio:
  - selezione in campioni con la lunghezza esatta;
  - qualità 3;
  - taglio automatico del silenzio spento;
  - dopo l'esportazione si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Musica e stinger: circa −16 LUFS integrati (±1 LU; stinger ±1,5).
- Ambienti più bassi: circa −22 LUFS (±1,5); un ronzio grave −24 LUFS.
- True peak ≤ −1 dBTP sull'Ogg; sul master ≤ −1,5 dBTP, perché la codifica alza un po' i
  picchi.
- Volumi relativi tra le tracce nel gioco, non nel file.

## 8. Consegna, provenienza e licenze

- **Nome del file:** `<ID>_v1.ogg` (per più file `<ID>_<parte>_v1.ogg`); le revisioni v2, v3…
  Il master ha lo stesso nome, `.wav` o `.flac`.
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/<ID>/` e scrive
  `<ID> pronto`.
- **`PROVENANCE.txt`**, nella stessa cartella:

  ```text
  [file]
  nome: <ID>_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: <ID>_v1.wav | nessuno
  note: ...
  ```

  Per un montaggio si aggiunge una sezione `[fonte]` per ogni materiale (descrizione, autore,
  licenza, URL, data, file della licenza salvata in `licenses/`).
- **Licenze ammesse:**
  - `user-owned`: composto dall'utente, o commissionato con un accordo scritto che permette
    uso nel gioco, nativo e web, anche a pagamento, modifiche e distribuzione senza scadenza;
  - `CC0-1.0`;
  - `CC-BY-4.0`, con il testo dell'attribuzione;
  - un servizio IA solo se l'utente dichiara nome, piano e termini che gli danno i diritti
    sull'output, e dà un via esplicito per questo file.
- **Escluse:**
  - licenze con `-NC` o `-ND`;
  - "royalty free" senza un testo chiaro;
  - estratti, remix o cover di altri giochi, film o canzoni;
  - imitazioni di jingle o temi riconoscibili;
  - musica da servizi IA senza diritti chiari.

## 9. Controlli dell'agente e criteri di accettazione

- [ ] nome, `PROVENANCE.txt` completo, licenza ammessa, licenze delle fonti salvate
- [ ] Ogg Vorbis valido: un solo flusso, BOS ed EOS, pagine consecutive, CRC giusti
- [ ] 2 canali, 44 100 Hz
- [ ] lunghezza: … campioni ±1 (ambienti ±22 050; stinger entro il massimo)
- [ ] peso ≤ … byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] loudness nella fascia
- [ ] estremi vicini allo zero; nessun silenzio né dissolvenza agli estremi (stinger: attacco e
      coda)
- [ ] continuità del giro
- [ ] (ambienti) bilanciamento stereo della richiesta, compatibile in mono
- [ ] ascolto dell'utente in loop: nessuno stacco; carattere giusto; niente di riconoscibile
- [ ] conferma esplicita dell'utente su questa versione

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo con la libreria standard.
Picco, loudness, estremi e continuità si misurano:
- sul master WAV all'arrivo;
- sull'Ogg decodificato, nel test di decodifica dopo la promozione.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:** comando per l'Ogg della procedura con i campioni attesi; analisi del master se
  c'è; ascolto in loop dell'utente.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio` (test di decodifica esteso in AUDIO-3 e AUDIO-4);
  - prova nel gioco nello stato giusto, con la traccia corrente nel log o nell'overlay F3;
  - ascolto dell'utente nel gioco.

## Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| — | — | — | — | — |

## Revisioni

(Solo le correzioni, con i punti in battute o secondi, e il nome del file `v<N+1>`.)

## Provenienza e licenza (dopo la promozione)

- **Versione promossa e file:** …
- **Consegnata il / approvata il:** … / … (frase dell'utente: "…")
- **Canale e autore:** …
- **Licenza (`license` del manifest):** …
- **Attribuzione:** … / non serve
- **Nota in `assets-src/audio/…/<nome>.md`:** sì / no
- **Voce in `ASSET_LICENSES.md`:** sì / no
