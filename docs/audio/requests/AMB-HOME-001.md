# AMB-HOME-001 — Ambienti della home: portale, cascate e fiume

Richiesta di ambienti con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi lavora sui
suoni e bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `AMB-HOME-001` (due parti: `portal`, `falls`) |
| Tipo | `ambience_loop` |
| File nel gioco | `assets/audio/ambience/home_portal.ogg`, `assets/audio/ambience/home_falls.ogg` |
| Canali ammessi | sintesi dell'utente; registrazioni CC0 (o CC-BY con attribuzione) scaricate dall'utente, con la licenza salvata, e montate da lui; commissione a una persona; servizio IA solo con nome, termini e via esplicito per ciascun file |
| Stato | `REQUESTED` (2026-11-02), per tutte e due le parti |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-4 |
| Decisioni | D-069, D-070 |
| Legata a | `MUSIC-HOME-001`: il ronzio del portale è intonato sul Fa della musica della home |

## Il gioco in breve

*Companion Survivors* è un gioco d'azione 2D in pixel art. Prima della partita c'è una pagina
iniziale con una grande illustrazione:
- un mattino limpido;
- dall'alto di una scarpata si guarda una valle con un fiume, cascate che scendono da scogliere
  chiare e un castello lontano;
- a destra, su un'altura, un'antica porta di pietra con un vortice azzurro a spirale e rune che
  pulsano.

Nel gioco l'illustrazione è animata: il vortice gira, le cascate scorrono, il fiume luccica.
Questi due loop ne danno il suono.

## 1. Dove suonano e cosa succede attorno

- **Dove:** in tutte le schermate del menu (pagina iniziale, scelta del compagno, talenti,
  opzioni), sotto la musica della home (Fa lidio, 80 BPM, circa −16 LUFS).
- **Entrata e uscita:** entrano con una dissolvenza di 2 s. Escono in 1 s quando parte la
  partita, mentre la musica passa a quella della partita in 1,5 s.
- **Volume:** li regola il cursore "Ambiente" delle opzioni. Non dipendono dall'interruttore
  delle animazioni.
- **Due loop separati** che suonano insieme, con lunghezze diverse: 32 s il portale, 27 s le
  cascate. Così la coppia si ripete identica solo ogni 864 s (14 min 24 s).
- **Stereo:** ogni file è già mixato con la posizione che ha nell'illustrazione. Il gioco non
  sposta i suoni.
- **Equilibrio con il resto:**
  - sopra c'è la musica, e nelle opzioni brevi "blip" dell'interfaccia intorno a 1–1,3 kHz;
  - gli ambienti non devono coprire la musica: niente fruscio duro tra 2 e 6 kHz, poco sopra
    gli 8 kHz;
  - niente melodia.
- **Durata dell'ascolto:** si può restare nei menu per molti minuti, quindi le testure devono
  reggere 30 s, e poi molti giri, senza stancare.

### Posizioni nell'illustrazione

Percentuali della larghezza contate da sinistra e dell'altezza contate dall'alto, misurate
sull'illustrazione di 1672 × 941 px. Il pan va da −1 (tutto a sinistra) a +1 (tutto a
destra); quello suggerito è un po' ristretto verso il centro, per l'ascolto in cuffia.

| Elemento | Larghezza | Altezza | Note | Pan suggerito |
|---|---|---|---|---|
| Portale (porta di pietra e vortice) | 79,5–91,2%, centro del vortice 85,3% | 21,8–47,3%, centro 39,4% | in alto, su un'altura lontana | +0,6 |
| Cascata grande | 88,6–93,9% | 63,5–82,3% | la più grande e la più bassa nel quadro, quindi la più vicina | +0,65 |
| Cascata sotto il ponte di pietra | 77,5–80,1% | 57,7–68,1% | media | +0,55 |
| Cascata sotto l'arco | 73,7–77,5% | 60,0–69,2% | media | +0,5 |
| Cascata sotto il castello | 69,1–73,0% | 41,4–47,6% | piccola e lontana | +0,4 |
| Cascata centrale | 67,5–71,4% | 50,6–59,3% | piccola | +0,4 |
| Cascata di sinistra | 59,7–64,0% | 47,3–55,8% | piccola e lontana | +0,25 |
| Fiume e lago | 32,9–74,2%, centro 53,5% | 41,7–77,8% | largo, tranquillo | da −0,35 a +0,5 |
| Parte sinistra (albero, staccionata, villaggio) | 0–33% | — | niente acqua | solo la parte diffusa |

Sugli schermi 4:3 il gioco taglia i bordi dell'illustrazione (resta visibile il 16,2–91,3%
della larghezza): il mix segue comunque il formato 16:9, dove si vede tutto.

Livelli del pan a potenza costante (L = cos((p+1)·π/4), R = sin((p+1)·π/4)), calcolati: il
canale sinistro rispetto al destro è a
- −2,8 dB con p = +0,2;
- −5,0 dB con +0,35;
- −6,7 dB con +0,45;
- −8,7 dB con +0,55;
- −9,8 dB con +0,6;
- −11,0 dB con +0,65;
- −14,0 dB con +0,75.

## 2. Parte `portal` — il portale

**Carattere e immagini:** un'antica porta di pietra in cui gira un vortice azzurro. Si sente un
ronzio caldo e profondo, e dell'aria che gira dentro il vortice e respira lentamente. Magico,
calmo, un po' misterioso; lontano, sull'altura.
- **Non deve:** sembrare elettrico (niente ronzio a 50 o 60 Hz, niente trasformatori, niente
  scariche), fare paura o essere minaccioso.

**Contenuto:**
- **Ronzio basso intonato su Fa**, la tonica della musica della home:
  - fondamentale Fa1, 43,65 Hz;
  - armoniche Fa2 87,31 Hz, Do3 130,81 Hz, Fa3 174,61 Hz, che lo rendono udibile anche sulle
    casse piccole dei portatili;
  - nessun'altra nota.
- **Vortice che respira:**
  - rumore filtrato passa-banda che sale e scende lentamente, per esempio fra 300 e 1200 Hz;
  - un respiro ogni 3,5–5 s, irregolare;
  - gira piano nello stereo fra +0,45 e +0,75, mai nella metà sinistra.
- **Luccichio:** facoltativo, appena percettibile, acuto e fermo, su Fa o Do. Nessuna melodia,
  nessuna sequenza di note.
- **Posizione:** il centro a +0,6, con il canale sinistro circa 10 dB sotto il destro.
  - RMS del canale destro 6–10 dB sopra il sinistro.
  - Morbido e senza transitori, perché è lontano.

**Per chi sintetizza:** il loop riesce perfetto per costruzione se ogni componente compie un
numero intero di cicli in 32 s.
- Frequenze multiple di 1/32 Hz, calcolate:
  - Fa1 = 1397/32 = 43,65625 Hz (+0,11 cent);
  - Fa2 = 2794/32 = 87,3125 Hz;
  - Do3 = 4186/32 = 130,8125 Hz;
  - Fa3 = 5588/32 = 174,625 Hz.
- Modulazioni con un numero intero di cicli in 32 s: per esempio un respiro a 0,25 Hz (8 cicli)
  più uno a 3/32 Hz (3 cicli), che insieme suonano irregolari.

| Voce | Valore |
|---|---|
| Lunghezza del loop | **1 411 200 campioni** a 44 100 Hz (32,0 s): 32 × 44 100 = 1 411 200 |
| Tolleranza | ±22 050 campioni (0,5 s), se un montaggio lo rende necessario; meglio esatta |
| Loudness | circa **−24 LUFS integrati**, ±1,5 LU. È più basso delle cascate perché la misura pesa poco i bassi e il ronzio è soprattutto grave |
| Peso massimo | 32 s × 16 000 byte = **512 000 byte** |

## 3. Parte `falls` — cascate e fiume

**Carattere e immagini:** cascate a più salti che scendono da scogliere chiare e un fiume largo
che scorre tranquillo verso un lago. Fresco, continuo, luminoso, senza fragore minaccioso.

**Mix:**
- **Cascata grande, vicina, a destra:** pan +0,65, il canale sinistro circa 11 dB sotto. È la
  più forte, un rombo pieno ma morbido.
- **Cascate medie e piccole**, tra il 59,7% e l'80,1% della larghezza:
  - pan da +0,25 a +0,55;
  - 4–9 dB sotto la grande;
  - più scure, con meno acuti, perché sono più lontane.
- **Fiume e lago:** sciabordio e scorrere morbido, aperto da −0,35 a +0,5, 8–10 dB sotto la
  cascata grande.
- **Sinistra** (albero, staccionata, villaggio): niente acqua, solo la parte diffusa del suono.
- **Risultato:**
  - RMS del canale destro 3–6 dB sopra il sinistro;
  - compatibile in mono, senza trucchi di fase.

**Da evitare:** uccelli, voci, tonfi o schizzi singoli riconoscibili, gocce isolate, raffiche
di vento. Ogni evento che si distingue fa sentire il giro dei 27 s.

| Voce | Valore |
|---|---|
| Lunghezza del loop | **1 190 700 campioni** a 44 100 Hz (27,0 s): 27 × 44 100 = 1 190 700 |
| Tolleranza | ±22 050 campioni (0,5 s); la lunghezza deve restare diversa da quella del portale e non un suo multiplo |
| Loudness | circa **−22 LUFS integrati**, ±1,5 LU |
| Peso massimo | 27 s × 16 000 byte = **432 000 byte** |

## 4. Da dove possono venire i suoni

- **Registrazioni CC0**, o CC-BY con l'attribuzione, di cascate e fiumi, scelte e scaricate
  dall'utente:
  - per ogni file salvare la pagina o il testo della licenza in `licenses/` e annotare URL,
    autore e data;
  - il montaggio: ricampionare a 44 100 Hz prima di tagliare il loop, poi posizionare nello
    stereo, equalizzare e chiudere il loop;
  - parole per la ricerca: "waterfall", "river", "stream", "cascata", "fiume", "ruscello",
    "ambience", "CC0".
- **Sintesi:**
  - rumore rosa o marrone filtrato, a strati, con modulazioni lente;
  - gorgoglii fatti di brevi grani di rumore filtrato;
  - per il portale toni puri e rumore filtrato.
- **Strade escluse:**
  - registrazioni senza licenza chiara, "royalty free" senza testo, licenze `-NC` o `-ND`;
  - suoni presi da altri giochi o film;
  - suoni da servizi IA senza diritti chiari o senza il via per il file.

## 5. Regole del loop

- Il file contiene **solo il loop**: nessuna dissolvenza all'inizio o alla fine, nessun
  silenzio iniziale.
- L'incrocio che chiude il loop sta **dentro** il file. Si prende un materiale più lungo del
  loop e se ne incrociano fine e inizio sovrapponendoli, così che l'ultimo campione porti al
  primo come due campioni consecutivi qualunque.
- Il primo e l'ultimo campione stanno vicini allo zero (|x| ≤ 0,01, cioè −40 dBFS) in tutti e
  due i canali. Per arrivarci si **ruota** il loop di pochi campioni (la lunghezza non cambia),
  mai con una dissolvenza.
- Testura uniforme, con variazioni lente; nessun evento riconoscibile (sezioni 2 e 3).

## 6. Formato ed esportazione

- Ogg Vorbis, stereo, 44 100 Hz, qualità 3 (circa 112 kbps nominali, a bitrate variabile).
- Pesi massimi: portale **512 000 byte**, cascate **432 000 byte** (durata × 16 000 byte, cioè
  128 kbps medi).
- Niente infrabassi sotto i 30–35 Hz: il Fa1 a 43,65 Hz sta sopra. Compatibili in mono.
- **Master** facoltativi ma consigliati: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza.
- **Esportazione**, per esempio con Audacity:
  - progetto a 44 100 Hz;
  - selezione in campioni da 0 per 1 411 200 (portale) o 1 190 700 (cascate);
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento;
  - poi si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Più bassi della musica, di 6–8 dB: cascate circa **−22 LUFS**, portale circa **−24 LUFS**
  (±1,5 LU).
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP. A questi livelli resta molto
  margine.
- Il gioco regola l'equilibrio fra i due loop e la musica con i suoi volumi: nei file conta
  solo restare nelle fasce.

## 8. Consegna, provenienza e licenze

- **File:**
  - `AMB-HOME-001_portal_v1.ogg`;
  - `AMB-HOME-001_falls_v1.ogg`.

  Ogni parte ha le sue revisioni (v2, v3…). I master hanno lo stesso nome con `.wav` (o
  `.flac`). Si possono consegnare anche una parte per volta.
- **Dove:** l'utente salva in `.audio-inbox/AMB-HOME-001/` e scrive `AMB-HOME-001 pronto`.
  Le licenze delle registrazioni vanno in `.audio-inbox/AMB-HOME-001/licenses/`.
- **`PROVENANCE.txt`** nella stessa cartella: una sezione `[file]` per ogni loop e una sezione
  `[fonte]` per ogni registrazione usata.

  ```text
  [file]
  nome: AMB-HOME-001_falls_v1.ogg
  autore: utente (montaggio)
  strumento: ...
  licenza: CC0-1.0 | CC-BY-4.0 | user-owned
  url: -
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: -
  master: AMB-HOME-001_falls_v1.wav | nessuno
  note: fonti montate, filtri, punto dell'incrocio

  [fonte]
  per: AMB-HOME-001_falls_v1.ogg
  descrizione: ...
  autore: ...
  licenza: CC0-1.0
  url: ...
  data: AAAA-MM-GG (download)
  licenza salvata: licenses/....
  ```

- **Licenze ammesse:**
  - `user-owned`: sintesi dell'utente, o commissione con un accordo scritto (uso nel gioco
    nativo e web anche a pagamento, modifiche, distribuzione senza scadenza);
  - `CC0-1.0`;
  - `CC-BY-4.0`, con l'attribuzione;
  - un servizio IA solo con nome, piano, termini dichiarati e via per il file.

  In un montaggio vale la licenza più restrittiva delle fonti.
- **Escluse:**
  - licenze con `-NC` o `-ND`;
  - "royalty free" senza un testo chiaro;
  - suoni di altri giochi o film;
  - servizi IA senza diritti chiari.

## 9. Controlli dell'agente e criteri di accettazione

Per ogni parte:
- [ ] nome, sezioni `[file]` e `[fonte]` in `PROVENANCE.txt`, licenza ammessa, licenze salvate
      in `licenses/`
- [ ] Ogg Vorbis valido: un solo flusso, BOS ed EOS, pagine consecutive, CRC giusti
- [ ] 2 canali, 44 100 Hz
- [ ] lunghezza: portale 1 411 200 ±22 050, cascate 1 190 700 ±22 050; le due lunghezze sono
      diverse e nessuna è multipla dell'altra
- [ ] peso: portale ≤ 512 000 byte, cascate ≤ 432 000 byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] loudness: portale −24 LUFS ±1,5, cascate −22 LUFS ±1,5
- [ ] estremi vicini allo zero, nessun silenzio né dissolvenza agli estremi
- [ ] continuità del giro
- [ ] stereo: RMS destro sopra il sinistro di 6–10 dB (portale) e di 3–6 dB (cascate);
      correlazione positiva
- [ ] niente melodia; portale intonato su Fa, senza ronzio elettrico; nessun evento
      riconoscibile
- [ ] ascolto dell'utente in loop, i due insieme e sotto la musica della home se c'è già:
      nessuno stacco, posizione credibile rispetto all'illustrazione
- [ ] conferma esplicita dell'utente su quella parte

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo, con il comando della procedura:
- `... AMB-HOME-001_portal_v1.ogg 1411200 22050`;
- `... AMB-HOME-001_falls_v1.ogg 1190700 22050`.

Picco, loudness, estremi, continuità e stereo si misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-4.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - comando per l'Ogg su ogni parte;
  - analisi dei master (con il bilanciamento destra/sinistra);
  - ascolto in loop dell'utente, i due insieme.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run`, senza `--autoplay`: entrata in 2 s nel menu, cursore "Ambiente", passaggio
    alla selezione e alle opzioni senza interruzioni, uscita in 1 s all'avvio della partita;
    dieci entrate e uscite dal menu senza loop doppi (AUDIO-4);
  - ascolto dell'utente nel gioco, guardando l'illustrazione.

## Registro delle versioni

| Parte | Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

## Revisioni

(Solo le correzioni, per parte, con i punti in secondi, e il nome del file `v<N+1>`.)

## Provenienza e licenza (dopo la promozione)

- **Versioni promosse e file:** …
- **Consegnate il / approvate il:** … / … (frase dell'utente: "…")
- **Canale, autore e fonti:** …
- **Licenza (`license` del manifest):** …
- **Attribuzione:** … / non serve
- **Note in `assets-src/audio/ambience/home_portal.md` e `home_falls.md`:** sì / no
- **Licenze delle fonti in `assets-src/audio/licenses/`:** sì / no / non servono
- **Voci in `ASSET_LICENSES.md`:** sì / no
