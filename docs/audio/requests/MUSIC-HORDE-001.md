# MUSIC-HORDE-001 — Musica delle orde

Richiesta di musica con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi compone e
bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `MUSIC-HORDE-001` |
| Tipo | `music_loop` |
| File nel gioco | `assets/audio/music/horde.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per questo file |
| Stato | `REQUESTED` (2026-11-02) |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3 |
| Decisioni | D-069, D-070 |
| Dipende da | `MUSIC-RUN-001`: stessa tonalità, stesso tempo, stessi suoni. Meglio stesso autore e stessa sessione |

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

- **Quando:** due volte a partita, alle 5:00 e alle 10:00.
- **Entrata:**
  - 4 s prima compare l'avviso "orda in arrivo", con un suono acuto;
  - parte questa musica, con una dissolvenza incrociata di 2 s dalla musica della partita
    (Re dorico, 120 BPM), che intanto va in pausa;
  - poi per 8 s nascono centinaia di creature tutt'intorno al giocatore, fino a 320 a schermo.
- **Durata:** la musica resta finché l'orda non è finita, più 30 s; di solito sono 40 s–1 min.
  Il loop di 48 s si sente circa una volta, a volte una e mezza.
- **Uscita:** sfuma in 2 s e la musica della partita riprende da dove era.
- **Il file parte sempre dal suo inizio:** la battuta 1 è quella che annuncia l'orda.
- **Una battuta a 120 BPM dura 2 s, quanto la dissolvenza.** La musica della partita è in un
  punto qualunque, quindi per una battuta i due brani si sovrappongono con i battiti non
  allineati. Suggerimento:
  - battuta 1: un colpo pieno sul primo tempo (piatto e cassa), poi un riempimento che cresce
    (rullo, tom), senza il charleston pieno;
  - groove pieno dalla battuta 2.

  La battuta 1 deve funzionare anche come ripartenza dopo la 24.
- **Abbassamento automatico:**
  - durante un'orda si sale spesso di livello: nella scelta dei potenziamenti la musica scende
    di 6 dB;
  - in pausa scende di 9 dB;
  - attacco 0,25 s, rilascio 0,5 s.
- **Sopra suonano** effetti fittissimi: centinaia di colpi e di creature sconfitte, raccolte
  acute, avvisi. Quindi:
  - spazio libero sopra i 2 kHz;
  - le percussioni spingono, ma senza piatti continui fortissimi.

## 2. Carattere e immagini

Il cielo sembra stringersi. Un'ondata enorme arriva da ogni lato, centinaia di creature in pochi
secondi. Si resiste in mezzo alla massa, mentre i compagni colpiscono senza sosta.

- **Deve far sentire:** urgenza, compattezza, energia, determinazione. È la stessa voce della
  musica della partita, messa sotto pressione.
- **Non deve:** fare paura, essere caotica, epica o orchestrale.

## 3. Palette strumentale

Originale, ispirata ai giochi a 16 bit degli anni '90 e adatta al pixel art.

- **Gli stessi suoni della musica della partita**, se si può con le stesse impostazioni,
  perché è la stessa avventura:
  - melodia in onda quadra stretta (duty 12,5–25%);
  - contromelodia in triangolo o FM leggero;
  - basso in triangolo o FM;
  - arpeggi di quadra;
  - cassa fatta con un seno che scende rapido di altezza;
  - rullante e charleston di rumore.
- **In più:**
  - charleston in semicrome, tom, cassa più fitta;
  - basso in semicrome o a ottave;
  - accordi di quinta senza terza, in FM tipo ottoni o in quadra saturata;
  - frasi brevi e incisive al posto della melodia lunga.
- **Al massimo 8 voci insieme**, come nella traccia della partita.

Da evitare: supersaw, orchestre realistiche, chitarre distorte moderne, voci, cambi bruschi di
timbro rispetto alla musica della partita.

## 4. Dati musicali

| Voce | Valore |
|---|---|
| BPM | 120, come la musica della partita |
| Tonalità e modo | Re dorico: Re Mi Fa Sol La Si Do, solo tasti bianchi. **Niente Si bemolle**: durante la dissolvenza stonerebbe contro il Si naturale della partita |
| Metro | 4/4 |
| Battute | 24 |
| Lunghezza esatta del loop | **2 116 800 campioni** a 44 100 Hz (48,0 s) |

Conti, verificati con Python: 44 100 × 60 / 120 = 22 050 campioni per battito; × 4 battiti =
88 200 per battuta; × 24 battute = **2 116 800 campioni**.

**Struttura suggerita** (facoltativa):

| Battute | Contenuto |
|---|---|
| 1 | lancio: colpo sul primo tempo, riempimento che cresce |
| 2–8 | groove pieno, riff |
| 9–16 | riff in alto e risposta del basso |
| 17–24 | culmine e giro che riporta alla battuta 1 |

Si possono riprendere l'armonia o un frammento ritmico della musica della partita, non la sua
melodia intera.

**Rapporti con le altre tracce:**
- **Musica della partita:** Re dorico, 120 BPM. Dissolvenza incrociata di 2 s in entrata e in
  uscita, con i battiti in fase casuale.
- **Boss:** Re minore, 144 BPM. Arriva sempre dopo la musica della partita, mai dopo questa.
- **Stinger di sconfitta:** su Re, e può interrompere questa traccia.

## 5. Regole del loop

- Il file contiene **solo il corpo del loop**: niente introduzione, niente finale, niente
  dissolvenze all'inizio o alla fine, nessun silenzio iniziale.
- La lunghezza è esatta: 2 116 800 campioni, tolleranza ±1 campione.
- La coda del riverbero e degli echi si riporta all'inizio, non si taglia. Due modi:
  - si esporta il brano due volte di seguito e si tiene esattamente il secondo giro;
  - oppure si somma all'inizio la coda che supera la battuta 24.
- L'ultimo campione deve portare al primo come due campioni consecutivi qualunque: stessa fase,
  nessun gradino.
- Il primo e l'ultimo campione stanno vicini allo zero (|x| ≤ 0,01, cioè −40 dBFS) in tutti e
  due i canali.
  - Per arrivarci si **ruota** il loop di pochi campioni fino a un passaggio per lo zero. La
    lunghezza non cambia. Mai con una dissolvenza.
  - Serve perché un eventuale vuoto al giro diventi un silenzio brevissimo e non un clic.

## 6. Formato ed esportazione

- Ogg Vorbis, stereo, 44 100 Hz, qualità 3 (circa 112 kbps nominali, a bitrate variabile).
- **Peso massimo:** 48 s × 16 000 byte = **768 000 byte** (0,73 MiB, 128 kbps medi).
- Niente infrabassi sotto i 30–35 Hz. Compatibile in mono, niente trucchi di fase.
- **Master** facoltativo ma consigliato: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza.
- **Esportazione**, per esempio con Audacity:
  - progetto a 44 100 Hz;
  - selezione in campioni da 0 per 2 116 800;
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento;
  - poi si riapre il file e si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Circa **−16 LUFS integrati**, ±1 LU, come la musica della partita. L'intensità viene
  dall'arrangiamento, non dal volume del file.
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP.

## 8. Consegna, provenienza e licenze

- **File:** `MUSIC-HORDE-001_v1.ogg`; le revisioni sono v2, v3… Il master si chiama
  `MUSIC-HORDE-001_v1.wav` (o `.flac`).
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/MUSIC-HORDE-001/`
  e scrive `MUSIC-HORDE-001 pronto`.
- **`PROVENANCE.txt`** nella stessa cartella:

  ```text
  [file]
  nome: MUSIC-HORDE-001_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: MUSIC-HORDE-001_v1.wav | nessuno
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
- [ ] granule finale 2 116 800 ±1
- [ ] peso ≤ 768 000 byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] −16 LUFS ±1
- [ ] estremi vicini allo zero, nessun silenzio né dissolvenza agli estremi
- [ ] continuità del giro
- [ ] stessi tempo, tonalità e famiglia di suoni della musica della partita; nessun Si bemolle
- [ ] ascolto dell'utente in loop e incrociato con la musica della partita (due tracce in
      Audacity, 2 s di sovrapposizione): nessuno stacco, nessuna stonatura, niente di
      riconoscibile
- [ ] conferma esplicita dell'utente su questa versione

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo, con il comando della procedura
(`... MUSIC-HORDE-001_v1.ogg 2116800`). Picco, loudness, estremi e continuità si misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - comando per l'Ogg con 2 116 800;
  - analisi del master;
  - ascolto in loop dell'utente, anche incrociato con la musica della partita.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run -- --autoplay --god --time-scale 4`, con la traccia corrente nel log o
    nell'overlay F3: entrata all'avviso delle 5:00 (circa 75 s reali), dissolvenza di 2 s,
    ducking nel draft, uscita dopo l'orda e ripresa della partita da dove era; lo stesso alle
    10:00;
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
- **Nota in `assets-src/audio/music/horde.md`:** sì / no
- **Voce in `ASSET_LICENSES.md`:** sì / no
