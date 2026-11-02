# MUSIC-HOME-001 — Musica della pagina iniziale e dei menu

Richiesta di musica con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi compone e
bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `MUSIC-HOME-001` |
| Tipo | `music_loop` |
| File nel gioco | `assets/audio/music/home.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per questo file |
| Stato | `INTEGRATED` (2026-11-02, AUDIO-3): v1 in `assets/audio/music/home.ogg`, suona nei menu; l'utente l'ha ascoltata nel gioco ("Va bene") |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3 |
| Decisioni | D-069, D-070, D-071 |
| Legata a | `MUSIC-RUN-001` (stesse sette note). Il legame con `AMB-HOME-001` (ronzio del portale sul Fa di questa traccia) non c'è più: ambienti della home rifiutati dall'utente, `REJECTED` (D-071) |

## Il gioco in breve

*Companion Survivors* è un gioco d'azione 2D in pixel art visto dall'alto:
- si guida un ragazzo in un'arena aperta, mentre piccole creature arrivano a ondate da ogni
  lato, a centinaia;
- i suoi compagni, mostriciattoli di fuoco, acqua, foglie e fulmine, attaccano da soli;
- una partita dura 15 minuti;
- prima della partita c'è una pagina iniziale con una grande illustrazione, da cui si apre il
  menu: nuova partita, scelta del compagno, talenti, opzioni.

## 1. Dove suona e cosa succede attorno

- **Dove:** nella pagina iniziale e in tutte le schermate del menu: scelta del compagno,
  talenti, opzioni. È la prima cosa che si sente: all'avvio del gioco o, nel browser, dopo il
  clic su "Clicca per giocare".
- **Insieme a questa traccia** suonano due loop d'ambiente più bassi:
  - il **ronzio grave del portale magico**, a destra, intonato su Fa (43,65 Hz, con armoniche a
    87, 131 e 175 Hz);
  - le **cascate e il fiume**, soprattutto a destra.

  La musica non deve coprirli né litigarci:
  - registro grave leggero;
  - niente note gravi tenute sotto i 100 Hz diverse da Fa e Do.

  *(Ritirati, D-071: l'utente non vuole ambienti nella home, e questa traccia ci suona da sola.
  Il brief resta com'era quando è stata fatta la v1.)*
- **Interfaccia:** nelle opzioni ogni tacca del volume suona un breve "blip" di onda quadra
  intorno a 1–1,3 kHz. La melodia non deve stare sempre in quella zona con timbri duri.
- **Quanto si sente:** si può restare nei menu a lungo, a scegliere il compagno o leggere i
  talenti, quindi il loop di 72 s gira anche molte volte. Calmo, senza picchi, senza ritornelli
  insistenti.
- **Dissolvenze:**
  - avvio della partita: questa traccia sfuma nella musica della partita in 1,5 s, con una
    dissolvenza incrociata a potenza costante;
  - ritorno al menu dai risultati: 1,5 s;
  - finestra non attiva: il volume scende a zero in 0,25 s e risale al ritorno.
- **Nei menu** non ci sono abbassamenti automatici.

## 2. Carattere e immagini

L'illustrazione della pagina iniziale: un mattino limpido.
- Dall'alto di una scarpata erbosa con margherite si guarda una valle: boschi, un fiume che
  brilla, cascate che scendono da scogliere chiare, un castello dai tetti rossi su una collina
  lontana.
- A destra, su un'altura, c'è un'antica porta di pietra con un vortice azzurro a spirale e rune
  che pulsano piano.
- Un ragazzo e tre piccoli compagni guardano la valle prima di partire.

- **Deve far sentire:** meraviglia tranquilla, attesa, un pizzico di mistero (il portale). Il
  viaggio sta per cominciare.
- **Non deve:** essere triste, epica o una ninna nanna; niente fanfare.

## 3. Palette strumentale

Originale, ispirata ai giochi a 16 bit degli anni '90 e adatta al pixel art:
- **tappeto:** pad caldo di onda quadra o dente di sega filtrati, con attacco lento;
- **arpeggio:** in terzine di crome, in triangolo o con un'arpa campionata a bassa risoluzione;
- **melodia:** rada, in onda quadra morbida al 50% o con un flauto campionato, vibrato lento;
- **motivo del portale:** campanella FM o suono di vetro che torna ogni tanto e si appoggia sul
  Si naturale sopra il Fa, il colore del modo lidio;
- **basso:** leggero, in triangolo, con poche note lunghe su Fa e Do;
- **percussioni:** assenti o lievissime, per esempio uno shaker di rumore molto basso e un tom
  morbido ogni due battute;
- **effetti:** eco e riverbero medi, con la coda ripiegata (sezione 5);
- **al massimo 8 voci insieme.**

Da evitare: supersaw, orchestre realistiche, voci, batteria piena, suoni acuti e duri tenuti a
lungo.

## 4. Dati musicali

| Voce | Valore |
|---|---|
| BPM | 80 |
| Tonalità e modo | Fa lidio: Fa Sol La Si Do Re Mi, solo tasti bianchi. La nota caratteristica è il Si naturale |
| Metro | 4/4 |
| Battute | 24 |
| Lunghezza esatta del loop | **3 175 200 campioni** a 44 100 Hz (72,0 s) |

Conti, verificati con Python: 44 100 × 60 / 80 = 33 075 campioni per battito; × 4 battiti =
132 300 per battuta; × 24 battute = **3 175 200 campioni**.

**Struttura suggerita** (facoltativa):

| Battute | Parte | Contenuto |
|---|---|---|
| 1–8 | A | tappeto, arpeggio e motivo del portale |
| 9–16 | B | entra la melodia |
| 17–24 | A' | la melodia si dirada e torna al motivo del portale, pronta a ripartire |

Progressione di esempio, generica: | Fa | Sol/Fa | Fa | Sol/Fa | Rem | Mim | Fa | Sol |.

**Rapporti con le altre tracce:**
- **Musica della partita:** Re dorico, 120 BPM. Usa le stesse sette note, così la dissolvenza
  di 1,5 s non stona.
  - 80 BPM sono i 2/3 di 120: le terzine di crome di questa traccia vanno alla stessa velocità
    delle crome della partita.
- **Ronzio del portale:** su Fa, la tonica di questa traccia. *(Ritirato, D-071.)*

## 5. Regole del loop

- Il file contiene **solo il corpo del loop**: niente introduzione, niente finale, niente
  dissolvenze all'inizio o alla fine, nessun silenzio iniziale.
- La lunghezza è esatta: 3 175 200 campioni, tolleranza ±1 campione.
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
- **Peso massimo:** 72 s × 16 000 byte = **1 152 000 byte** (1,10 MiB, 128 kbps medi).
- Niente infrabassi sotto i 30–35 Hz. Compatibile in mono, niente trucchi di fase.
- **Master** facoltativo ma consigliato: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza.
- **Esportazione**, per esempio con Audacity:
  - progetto a 44 100 Hz;
  - selezione in campioni da 0 per 3 175 200;
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento;
  - poi si riapre il file e si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Circa **−16 LUFS integrati**, ±1 LU. Il gioco la abbassa da sé se serve: tutte le musiche
  hanno lo stesso livello nel file.
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP.

## 8. Consegna, provenienza e licenze

- **File:** `MUSIC-HOME-001_v1.ogg`; le revisioni sono v2, v3… Il master si chiama
  `MUSIC-HOME-001_v1.wav` (o `.flac`).
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/MUSIC-HOME-001/` e
  scrive `MUSIC-HOME-001 pronto`.
- **`PROVENANCE.txt`** nella stessa cartella:

  ```text
  [file]
  nome: MUSIC-HOME-001_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: MUSIC-HOME-001_v1.wav | nessuno
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

Spuntate solo le caselle verificate in modo definitivo. Tra parentesi l'esito della v1 dove
manca ancora qualcosa: il master dice com'era il file prima della codifica, il controllo
definitivo si fa sull'Ogg decodificato (AUDIO-3).

- [ ] nome, `PROVENANCE.txt` completo, licenza ammessa, licenze delle fonti salvate (v1: nome
      giusto, sezione `[file]` completa, `licenza: user-owned`, nessuna fonte di terzi
      dichiarata. Canale confermato dall'utente il 2026-11-02 (D-071): sintesi da codice Python
      scritto con ChatGPT. **Piano ChatGPT e data dei termini non indicati:** da completare prima
      di una pubblicazione)
- [x] Ogg Vorbis valido: un solo flusso, BOS ed EOS, pagine consecutive, CRC giusti
- [x] 2 canali, 44 100 Hz
- [x] granule finale 3 175 200 ±1
- [x] peso ≤ 1 152 000 byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5) (v1: master −5,64 dBTP, OK; Ogg non misurato)
- [ ] −16 LUFS ±1 (v1: master −15,95 LUFS, OK; Ogg non misurato)
- [ ] estremi vicini allo zero, nessun silenzio né dissolvenza agli estremi (v1, sul master:
      primo e ultimo campione OK; ultimi 50 ms NO, **accettato dall'utente** il 2026-11-02,
      D-071; Ogg non misurato)
- [ ] continuità del giro (v1: master OK; Ogg non misurato)
- [ ] registro grave leggero e senza note gravi tenute fuori da Fa e Do (v1, euristica sul
      master: sotto i 100 Hz suona solo Fa2. Il ronzio del portale, che lo motivava, è ritirato:
      D-071)
- [ ] ascolto dell'utente per almeno tre giri: nessuno stacco, carattere giusto, niente di
      riconoscibile (l'approvazione non dice quanti giri ha ascoltato: si conferma
      all'ascolto nel gioco, per `INTEGRATED`)
- [x] conferma esplicita dell'utente su questa versione (2026-11-02, frase nel registro)

Tutti i controlli dell'arrivo, sull'Ogg e sul master, si fanno con
`python -I scripts/audio_inbox.py MUSIC-HOME-001 --save`. Per il solo Ogg basta anche il comando
della procedura (`... MUSIC-HOME-001_v1.ogg 3175200`). Picco, loudness, estremi e continuità si
misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - `python -I scripts/audio_inbox.py MUSIC-HOME-001 --save`: Ogg con 3 175 200 e analisi del
    master;
  - ascolto in loop dell'utente.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run`, senza `--autoplay`, fermi nella home per almeno tre giri (circa 4 minuti), poi
    selezione, opzioni (con i blip delle tacche) e avvio della partita (dissolvenza di 1,5 s);
  - ~~con AUDIO-4 anche insieme al portale e alle cascate~~ (ritirato, D-071: AUDIO-4
    annullato);
  - ascolto dell'utente nel gioco.

## Registro delle versioni

| Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|
| v1 | `MUSIC-HOME-001_v1.ogg`, 465 651 byte, sha256 `92392ab03c59…`; master `MUSIC-HOME-001_v1.wav` (PCM 24 bit) | 2026-11-02 | **Ogg:** contenitore OK (73 pagine, 1 flusso, CRC giusti, BOS ed EOS); Vorbis 2 canali 44 100 Hz OK; granule 3 175 200, scarto 0, OK; peso 465 651 ≤ 1 152 000 OK (51,7 kbps medi, nominale 112); encoder OK (tag `ENCODER=Lavc61.19.101 libvorbis`; il fornitore è `Lavf61.7.103`, la firma del muxer di FFmpeg); picco, loudness, estremi e giro non misurati. **Master:** 3 175 200 frame, uguali all'Ogg, OK; true peak −5,64 dBTP OK; −15,95 LUFS OK; estremi **NO**: primo e ultimo campione ≤ 0,0016 OK, ma ultimi 50 ms −19,8 dB (L) e −16,8 dB (R) sotto il secondo prima (soglia −6); le 23 stanghette interne finiscono tutte così (da −17,0 a −20,9 dB in L), quindi è il modo in cui finisce ogni battuta e non una dissolvenza al giro: da giudicare all'ascolto; continuità OK (salto L 0,0020 e R 0,0006, contro un 99,9° percentile di 0,0235 e 0,0222); stereo R − L −0,97 dB, correlazione +0,948; note gravi tenute fuori da Fa e Do: nessuna (euristica: sotto i 100 Hz solo Fa2, al massimo −31 dBFS; banda fino a 100 Hz −17,8 dB sul totale). **Provenienza:** sintesi da codice scritto con ChatGPT; canale, piano e termini da confermare dall'utente. Uscita completa: `.audio-inbox/MUSIC-HOME-001/check/v1.txt` | **approvata** il 2026-11-02 (D-071), dopo aver ricevuto l'esito dei controlli, compreso il NO degli estremi, e la richiesta di confermare canale, piano e termini. Frase dell'utente: "Apporvo soltanto la musica del menu mentre i suoni home portale e cascate non le voglio che ci siano in generale". **NO degli estremi accettato dall'utente.** Canale: sintesi da codice Python scritto con ChatGPT; l'approvazione vale come via esplicito per questo file. Piano ChatGPT e data dei termini non indicati: da completare prima di una pubblicazione. Promossa lo stesso giorno in `assets/audio/music/home.ogg` (sha256 uguale). `INTEGRATED` solo quando suona nel gioco e l'utente l'ha sentita; ascoltata nel gioco e confermata il 2026-11-02 ("Va bene"): `INTEGRATED` |

## Revisioni

(Solo le correzioni, con i punti in battute o secondi, e il nome del file `v<N+1>`.)

## Provenienza e licenza (dopo la promozione)

- **Versione promossa e file:** v1, `MUSIC-HOME-001_v1.ogg` → `assets/audio/music/home.ogg`,
  copiato byte per byte il 2026-11-02. 465 651 byte, sha256
  `92392ab03c59abe3a30e8cd62e37d457708057182068680e854ed890e6bb92ba`, uguale nei due file. Il
  master WAV resta all'utente, fuori dal repository.
- **Consegnata il / approvata il:** 2026-11-02 / 2026-11-02 (frase dell'utente: "Apporvo
  soltanto la musica del menu mentre i suoni home portale e cascate non le voglio che ci siano in
  generale"). NO degli estremi accettato.
- **Canale e autore:** suono sintetizzato da codice Python scritto con ChatGPT su richiesta
  dell'utente (D-071). Autore dichiarato in `PROVENANCE.txt`: "ChatGPT
  (composizione e script originale su richiesta dell'utente)". Nessuna fonte di terzi
  dichiarata. **Piano ChatGPT e data dei termini non indicati dall'utente:** da completare prima
  di una pubblicazione.
- **Licenza (`license` del manifest):** `user-owned (generated by the user with ChatGPT, terms
  not indicated)`; `approved: null` fino a `INTEGRATED`.
- **Attribuzione:** non serve.
- **Nota in `assets-src/audio/music/home.md`:** sì, con `PROVENANCE.txt` copiato per intero.
- **Voce in `ASSET_LICENSES.md`:** sì, sezione "Audio".
- **Catalogo:** la sezione `music` di `assets/audio/catalog.ron` arriva con AUDIO-3
  (gameplay-engineer). Fino ad allora il validatore dà l'avviso "non citato in
  assets/audio/catalog.ron".
