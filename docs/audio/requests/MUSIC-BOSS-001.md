# MUSIC-BOSS-001 — Musica del boss

Richiesta di musica con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi compone e
bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `MUSIC-BOSS-001` |
| Tipo | `music_loop` |
| File nel gioco | `assets/audio/music/boss.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per questo file |
| Stato | `REQUESTED` (2026-11-02) |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3b (D-071: AUDIO-3 porta solo la musica della home) |
| Decisioni | D-069, D-070 |
| Dipende da | `MUSIC-RUN-001`: stessa famiglia di suoni, stessa tonica Re |

## Il gioco in breve

*Companion Survivors* è un gioco d'azione 2D in pixel art visto dall'alto:
- si guida un ragazzo in un'arena aperta, mentre piccole creature arrivano a ondate da ogni
  lato, a centinaia;
- i suoi compagni, mostriciattoli di fuoco, acqua, foglie e fulmine, attaccano da soli;
- si raccolgono cristalli d'esperienza e a ogni livello si sceglie un potenziamento tra tre
  carte;
- una partita dura 15 minuti: due orde enormi alle 5:00 e alle 10:00, poi un boss;
- gli effetti sonori sono tanti, brevi e in stile chip.

## 1. Dove suona e cosa succede attorno

- **Entrata:** alle 15:00 le ondate si fermano e arriva il boss, il Gran Corazzato. È un gigante
  corazzato, lento e pesante, molto più grande di tutte le creature viste fino a lì.
  - Suona uno stinger d'arrivo: un colpo breve, su Re.
  - La musica della partita (Re dorico, 120 BPM) lascia il posto a questa in 1,5 s, con una
    dissolvenza incrociata a potenza costante.
  - Il file parte sempre dal suo inizio.
- **Uscita:**
  - se il boss cade: stinger di vittoria (in Re maggiore), poi la schermata dei risultati con
    una dissolvenza di 0,6 s;
  - se cade il giocatore: stinger di sconfitta (in Re minore).
- **Durata:** lo scontro dura da uno a qualche minuto, quindi il loop di 80 s può girare più
  volte.
- **Abbassamento automatico:**
  - in pausa la musica scende di 9 dB;
  - nella scelta dei potenziamenti scende di 6 dB;
  - attacco 0,25 s, rilascio 0,5 s.
- **Sopra suonano** i colpi pesanti del boss, gli avvisi dei suoi attacchi, i colpi dei
  compagni e le raccolte. Quindi:
  - spazio libero sopra i 2 kHz;
  - il basso spinge ma non copre i colpi gravi del boss.

## 2. Carattere e immagini

Il terreno trema. Un gigante corazzato avanza lento e pesante: è l'ultima prova della partita,
dopo quindici minuti di ondate. Si combatte con coraggio, non si fugge.

- **Deve far sentire:** tensione, peso, determinazione. Un tono eroico e scuro.
- **Non deve:** fare paura, essere tragica o da orchestra cinematografica realistica.

## 3. Palette strumentale

Originale, ispirata ai giochi a 16 bit degli anni '90 e adatta al pixel art.

- **La stessa famiglia di suoni della musica della partita:**
  - cassa fatta con un seno che scende rapido di altezza;
  - rullante e charleston di rumore;
  - onda quadra e FM.
- **In più:**
  - basso FM pesante sul Re grave, a ottave;
  - colpi di quadra saturata o di ottoni FM;
  - tappeto di archi campionati a bassa risoluzione;
  - tom gravi tipo timpani e rulli di rullante;
  - melodia nel registro medio-grave, raddoppiata all'ottava;
  - brevi cromatismi per la tensione: Do diesis sulla dominante, Mi bemolle di passaggio.
- **Al massimo 8 voci insieme.**

Da evitare: orchestre realistiche, cori, voci, chitarre distorte moderne, suoni horror (urla,
rumori industriali).

## 4. Dati musicali

| Voce | Valore |
|---|---|
| BPM | 144 |
| Tonalità e modo | Re minore: Re Mi Fa Sol La Si♭ Do, con il Do♯ sull'accordo di La (dominante) |
| Metro | 4/4 |
| Battute | 48 |
| Lunghezza esatta del loop | **3 528 000 campioni** a 44 100 Hz (80,0 s) |

Conti, verificati con Python: 44 100 × 60 / 144 = 18 375 campioni per battito; × 4 battiti =
73 500 per battuta; × 48 battute = **3 528 000 campioni**.

**Struttura suggerita** (facoltativa):

| Battute | Parte | Contenuto |
|---|---|---|
| 1–16 | A | riff del basso e colpi |
| 17–32 | B | melodia |
| 33–40 | C | calo, il basso quasi da solo e la tensione che risale |
| 41–48 | A' | ripresa e giro che riporta alla battuta 1 |

**Rapporti con le altre tracce:**
- **Musica della partita:** Re dorico, 120 BPM. Stessa tonica, una sola nota diversa (Si♭ al
  posto del Si). Il passaggio avviene sotto lo stinger d'arrivo, in 1,5 s.
- **Stinger d'arrivo:** su Re.
- **Stinger di vittoria:** in Re maggiore, interrompe questa traccia.
- **Stinger di sconfitta:** in Re minore, interrompe questa traccia.

## 5. Regole del loop

- Il file contiene **solo il corpo del loop**: niente introduzione, niente finale, niente
  dissolvenze all'inizio o alla fine, nessun silenzio iniziale.
- La lunghezza è esatta: 3 528 000 campioni, tolleranza ±1 campione.
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
- **Peso massimo:** 80 s × 16 000 byte = **1 280 000 byte** (1,22 MiB, 128 kbps medi).
- Niente infrabassi sotto i 30–35 Hz: il basso pesante sta sopra. Compatibile in mono, niente
  trucchi di fase.
- **Master** facoltativo ma consigliato: WAV PCM 24 bit (o 16), 44 100 Hz, stesso contenuto e
  stessa lunghezza.
- **Esportazione**, per esempio con Audacity:
  - progetto a 44 100 Hz;
  - selezione in campioni da 0 per 3 528 000;
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento;
  - poi si riapre il file e si ricontrolla la lunghezza.

## 7. Loudness e picchi

- Circa **−16 LUFS integrati**, ±1 LU, come le altre musiche. Il peso viene
  dall'arrangiamento.
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP.

## 8. Consegna, provenienza e licenze

- **File:** `MUSIC-BOSS-001_v1.ogg`; le revisioni sono v2, v3… Il master si chiama
  `MUSIC-BOSS-001_v1.wav` (o `.flac`).
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/MUSIC-BOSS-001/` e
  scrive `MUSIC-BOSS-001 pronto`.
- **`PROVENANCE.txt`** nella stessa cartella:

  ```text
  [file]
  nome: MUSIC-BOSS-001_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: MUSIC-BOSS-001_v1.wav | nessuno
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
- [ ] granule finale 3 528 000 ±1
- [ ] peso ≤ 1 280 000 byte
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] −16 LUFS ±1
- [ ] estremi vicini allo zero, nessun silenzio né dissolvenza agli estremi
- [ ] continuità del giro
- [ ] stessa famiglia di suoni della musica della partita
- [ ] ascolto dell'utente per almeno tre giri: nessuno stacco, carattere giusto, niente di
      riconoscibile
- [ ] conferma esplicita dell'utente su questa versione

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo, con il comando della procedura
(`... MUSIC-BOSS-001_v1.ogg 3528000`). Picco, loudness, estremi e continuità si misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - comando per l'Ogg con 3 528 000;
  - analisi del master;
  - ascolto in loop dell'utente.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run -- --autoplay --god --time-scale 4`, con la traccia corrente nel log o
    nell'overlay F3: arrivo del boss alle 15:00 (circa 225 s reali) con lo stinger e la
    dissolvenza di 1,5 s, giri senza stacchi, vittoria o sconfitta con il loro stinger;
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
- **Nota in `assets-src/audio/music/boss.md`:** sì / no
- **Voce in `ASSET_LICENSES.md`:** sì / no
