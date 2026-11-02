# MUSIC-RUN-001 — Musica della partita (Prateria Verde, e ripiego per ogni ambientazione)

Richiesta di musica con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi compone e
bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `MUSIC-RUN-001` |
| Tipo | `music_loop` |
| File nel gioco | `assets/audio/music/run_meadow.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per questo file |
| Stato | `REQUESTED` (2026-11-02) |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3 |
| Decisioni | D-069, D-070 |

## Il gioco in breve

*Companion Survivors* è un gioco d'azione 2D in pixel art visto dall'alto:
- si guida un ragazzo in un'arena aperta, mentre piccole creature arrivano a ondate da ogni
  lato, a centinaia;
- i suoi compagni, mostriciattoli di fuoco, acqua, foglie e fulmine, attaccano da soli;
- si raccolgono cristalli d'esperienza e a ogni livello si sceglie un potenziamento tra tre
  carte;
- una partita dura 15 minuti: due orde enormi alle 5:00 e alle 10:00, poi un boss;
- gli effetti sonori sono tanti, brevi e in stile chip: colpi, raccolte con note pure che
  salgono, avvisi acuti.

## 1. Dove suona e cosa succede attorno

- **Dove:** durante la partita nella Prateria Verde e in ogni ambientazione che non ha ancora
  una musica sua. Oggi è anche il Bosco Ombroso; più avanti canyon, ghiacci, vulcano e rovine.
  Deve quindi stare bene "all'aperto" in generale, non solo in un prato.
- **Entrata:** quando si avvia la partita, prende il posto della musica del menu con una
  dissolvenza incrociata di 1,5 s. Il file parte sempre dal suo inizio.
- **Quanto si sente:** quasi tutta la partita, tranne le orde e il boss. Tra un'orda e l'altra
  passano circa 5 minuti di fila, quindi il loop di 96 s gira tre volte e più. Deve reggere senza
  stancare.
- **Orde, alle 5:00 e alle 10:00:**
  - l'avviso arriva 4 s prima; la musica passa a quella dell'orda con una dissolvenza incrociata
    di 2 s;
  - questa traccia non si ferma: va in pausa;
  - finita l'orda, dopo circa 40–60 s, riprende con un'altra dissolvenza di 2 s **da dove si era
    fermata**, cioè a metà di una battuta qualunque.
- **Boss, alle 15:00:** suona uno stinger d'arrivo e questa traccia lascia il posto alla musica
  del boss in 1,5 s.
- **Sconfitta:** se il giocatore cade, suona uno stinger di sconfitta con la musica abbassata
  sotto, poi c'è la schermata dei risultati, con una dissolvenza di 0,6 s.
- **Abbassamento automatico:**
  - in pausa la musica scende di 9 dB;
  - nella scelta dei potenziamenti scende di 6 dB. All'inizio della partita la scelta arriva
    spesso, anche ogni 20–30 s;
  - attacco 0,25 s, rilascio 0,5 s.

  Anche più bassa deve restare chiara: a −9 dB basso e melodia si riconoscono ancora.
- **Ricominciare** la partita non interrompe la musica.
- **Sopra suonano** moltissimi effetti brevi: colpi, attacchi dei compagni, creature sconfitte,
  raccolte di cristalli (note pure che salgono di semitono in semitono, nel registro acuto),
  avvisi. Quindi:
  - niente melodie lunghe e insistenti sopra i 2 kHz;
  - melodia nel registro medio, circa 300–1500 Hz;
  - basso chiaro ma non rimbombante;
  - densità media.

## 2. Carattere e immagini

Una prateria aperta sotto il sole pieno. L'erba alta ondeggia nel vento, ci sono qualche roccia e
qualche albero solitario. Si cammina senza mai fermarsi e si combatte con allegria contro sciami
che non finiscono, mentre i compagni colpiscono da soli.

- **Deve far sentire:** avventura, coraggio allegro, energia costante, ottimismo con un filo di
  tensione. Il modo dorico dà luce con un velo di nostalgia.
- **Non deve:** essere epica e orchestrale, da cartone per bambini, malinconica o frenetica.

## 3. Palette strumentale

Originale, ispirata ai giochi a 16 bit degli anni '90 e adatta al pixel art:
- **melodia:** onda quadra con duty 12,5–25%, leggero vibrato ritardato, brevi glissati;
- **contromelodia:** triangolo, oppure un FM leggero (campanella di legno, piano elettrico FM);
- **basso:** triangolo o basso FM "slap" in crome;
- **accordi:** arpeggi veloci di onda quadra al 50%, oppure un pizzicato campionato a bassa
  risoluzione;
- **percussioni:**
  - cassa fatta con un seno che scende rapido di altezza;
  - rullante di rumore;
  - charleston di rumore corto in crome;
  - qualche tom;
- **effetti:** eco a croma puntata con poco feedback, riverbero breve;
- **al massimo 8 voci insieme:** è un vincolo di stile, e tiene chiaro il mix nelle orde.

Da evitare:
- supersaw moderni, orchestre realistiche, chitarre distorte moderne;
- voci;
- batteria acustica con una stanza grande.

**Tieni i suoni di questa traccia:** la musica dell'orda, quella del boss e gli stinger li
riusano, perché sono la stessa avventura.

## 4. Dati musicali

| Voce | Valore |
|---|---|
| BPM | 120 |
| Tonalità e modo | Re dorico: Re Mi Fa Sol La Si Do, solo tasti bianchi. La nota caratteristica è il Si naturale |
| Metro | 4/4 |
| Battute | 48 |
| Lunghezza esatta del loop | **4 233 600 campioni** a 44 100 Hz (96,0 s) |

Conti, verificati con Python: 44 100 × 60 / 120 = 22 050 campioni per battito; × 4 battiti =
88 200 per battuta; × 48 battute = **4 233 600 campioni**.

**Struttura suggerita** (facoltativa):

| Battute | Parte | Contenuto |
|---|---|---|
| 1–16 | A | tema |
| 17–32 | B | sviluppo, con la tensione su Sol e Do |
| 33–40 | C | respiro, con meno strumenti |
| 41–48 | A' | ripresa con una variazione e un giro armonico che riporta alla battuta 1 |

Progressione di esempio, generica: | Rem | Sol | Rem | Sol | Fa | Do | Lam | Lam |.

**Rapporti con le altre tracce:**
- **Orda:** Re dorico, 120 BPM, stessi suoni. Si incrocia con questa per 2 s, cioè una battuta
  esatta, in entrata e in uscita, con i battiti non allineati: deve stare insieme senza
  stonare.
- **Musica del menu:** Fa lidio, 80 BPM. Usa le stesse sette note, così la dissolvenza di 1,5 s
  non stona.
- **Boss:** Re minore, 144 BPM. Stessa tonica, una nota diversa (Si bemolle).
- **Stinger** d'arrivo del boss e di sconfitta: su Re.

## 5. Regole del loop

- Il file contiene **solo il corpo del loop**: niente introduzione, niente finale, niente
  dissolvenze all'inizio o alla fine, nessun silenzio iniziale.
- La lunghezza è esatta: 4 233 600 campioni, tolleranza ±1 campione.
- La coda del riverbero e degli echi si riporta all'inizio, non si taglia. Due modi:
  - si esporta il brano due volte di seguito e si tiene esattamente il secondo giro;
  - oppure si somma all'inizio la coda che supera la battuta 48.
- L'ultimo campione deve portare al primo come due campioni consecutivi qualunque: stessa fase,
  nessun gradino.
- Il primo e l'ultimo campione stanno vicini allo zero (|x| ≤ 0,01, cioè −40 dBFS) in tutti e
  due i canali.
  - Per arrivarci si **ruota** il loop di pochi campioni fino a un passaggio per lo zero. La
    lunghezza non cambia. Mai con una dissolvenza.
  - Serve perché un eventuale vuoto al giro diventi un silenzio brevissimo e non un clic.

## 6. Formato ed esportazione

- Ogg Vorbis, stereo, 44 100 Hz, qualità 3 (circa 112 kbps nominali, a bitrate variabile).
- **Peso massimo:** 96 s × 16 000 byte = **1 536 000 byte** (1,46 MiB, 128 kbps medi).
- Niente infrabassi sotto i 30–35 Hz. Compatibile in mono, niente trucchi di fase.
- **Master** facoltativo ma consigliato: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza.
- **Esportazione**, per esempio con Audacity:
  - progetto a 44 100 Hz;
  - selezione in campioni da 0 per 4 233 600;
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento;
  - poi si riapre il file e si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Circa **−16 LUFS integrati**, ±1 LU.
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP, perché la codifica alza un po' i
  picchi.
- Compressione moderata: la musica scorre sotto molti effetti e deve restare morbida.

## 8. Consegna, provenienza e licenze

- **File:** `MUSIC-RUN-001_v1.ogg`; le revisioni sono v2, v3… Il master si chiama
  `MUSIC-RUN-001_v1.wav` (o `.flac`).
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/MUSIC-RUN-001/` e
  scrive `MUSIC-RUN-001 pronto`.
- **`PROVENANCE.txt`** nella stessa cartella:

  ```text
  [file]
  nome: MUSIC-RUN-001_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: MUSIC-RUN-001_v1.wav | nessuno
  note: ...
  ```

  Ogni campione o loop di altri usato dentro va in una sezione `[fonte]`: descrizione, autore,
  licenza, URL, data, file della licenza salvata in `licenses/`.
- **Licenze ammesse:**
  - `user-owned`: composta dall'utente, oppure commissionata con un accordo scritto che permette
    uso nel gioco (nativo e web, anche a pagamento), modifiche e distribuzione senza scadenza;
  - `CC0-1.0`;
  - `CC-BY-4.0`, con il testo dell'attribuzione;
  - musica da un servizio IA solo se l'utente dichiara nome, piano e termini che gli danno i
    diritti sull'output, e dà un via esplicito per questo file.
- **Escluse:**
  - licenze con `-NC` o `-ND`;
  - "royalty free" senza un testo chiaro;
  - estratti, remix o cover di altri giochi, film o canzoni;
  - imitazioni di temi riconoscibili;
  - musica da servizi IA senza diritti chiari.

## 9. Controlli dell'agente e criteri di accettazione

- [ ] nome, `PROVENANCE.txt` completo, licenza ammessa, licenze delle fonti salvate
- [ ] Ogg Vorbis valido: un solo flusso, BOS ed EOS, pagine consecutive, CRC giusti
- [ ] 2 canali, 44 100 Hz
- [ ] granule finale 4 233 600 ±1
- [ ] peso ≤ 1 536 000 byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] −16 LUFS ±1
- [ ] estremi vicini allo zero, nessun silenzio né dissolvenza agli estremi
- [ ] continuità del giro
- [ ] ascolto dell'utente per almeno tre giri: nessuno stacco, carattere giusto, niente di
      riconoscibile
- [ ] conferma esplicita dell'utente su questa versione

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo, con il comando della procedura
(`... MUSIC-RUN-001_v1.ogg 4233600`). Picco, loudness, estremi e continuità si misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - comando per l'Ogg con 4 233 600;
  - analisi del master;
  - ascolto in loop dell'utente.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run -- --autoplay --god --time-scale 4`, con la traccia corrente nel log o
    nell'overlay F3: partenza dalla home, giri senza stacchi, pausa e draft abbassati, passaggio
    all'orda delle 5:00 (circa 75 s reali) e ritorno con la run ripresa da dove era;
  - `--stage forest` per il ripiego;
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
- **Nota in `assets-src/audio/music/run_meadow.md`:** sì / no
- **Voce in `ASSET_LICENSES.md`:** sì / no
