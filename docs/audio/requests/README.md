# Richieste di musica e ambienti

Indice delle richieste dell'handoff manuale (D-070). La procedura è in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`, il modello in `docs/audio/AUDIO_REQUEST_TEMPLATE.md`.
Ogni richiesta si regge da sola. Questo file tiene insieme quello che le lega: ordine,
dipendenze, tonalità, tempi e peso.

## Indice e ordine consigliato

| Ordine | ID | File nel gioco | Che cosa | Dipende da | Tranche | Stato |
|---|---|---|---|---|---|---|
| 1 | [`MUSIC-RUN-001`](MUSIC-RUN-001.md) | `music/run_meadow.ogg` | partita nella Prateria Verde, e ripiego per ogni stage | — | AUDIO-3b | `REQUESTED` |
| 2 | [`MUSIC-HORDE-001`](MUSIC-HORDE-001.md) | `music/horde.ogg` | orde delle 5:00 e delle 10:00 | RUN: tonalità, tempo, suoni | AUDIO-3b | `REQUESTED` |
| 3 | [`MUSIC-HOME-001`](MUSIC-HOME-001.md) | `music/home.ogg` | pagina iniziale e menu | RUN: stesse sette note | AUDIO-3 | `APPROVED` (v1, 2026-11-02, D-071): promossa in `assets/`, il gioco non la suona ancora |
| — | [`AMB-HOME-001`](AMB-HOME-001.md) | nessuno (erano `ambience/home_portal.ogg`, `ambience/home_falls.ogg`) | portale; cascate e fiume | — | ~~AUDIO-4~~ annullato | `REJECTED` (2026-11-02, D-071): niente ambienti nella home |
| 5 | [`MUSIC-BOSS-001`](MUSIC-BOSS-001.md) | `music/boss.ogg` | boss delle 15:00 | RUN: suoni, tonica | AUDIO-3b | `REQUESTED` |
| 6 | [`MUSIC-STING-001`](MUSIC-STING-001.md) | `music/stinger_boss.ogg`, `stinger_victory.ogg`, `stinger_defeat.ogg` | arrivo del boss, vittoria, sconfitta | RUN e BOSS: tonalità, suoni | AUDIO-3b | `REQUESTED` |

I percorsi sono relativi ad `assets/audio/`.

Perché quest'ordine:
- **1. RUN:** è la musica che si sente di più ed è il riferimento di palette, tonalità e tempo
  per orda, boss e stinger. Il rischio più grosso, il loop Vorbis in Bevy che non deve aprire
  un vuoto al giro (AUDIO_BIBLE § 4), con D-071 si prova prima sulla home, in AUDIO-3.
- **2. HORDE:** subito dopo, perché il passaggio tra partita e orda è il più delicato:
  dissolvenze di 2 s in entrata e in uscita, battiti in fase casuale. Meglio stesso autore e
  stessa sessione della run.
- **3. HOME:** è la prima cosa che si sente. Approvata per prima (D-071), quindi apre AUDIO-3.
- **AMB-HOME:** rifiutata dall'utente (D-071), niente ambienti nella home. Il legame con la home
  (il ronzio del portale sulla sua tonica) non c'è più.
- **5–6. BOSS e STINGER:** per ultimi. Gli stinger devono stare bene sopra le tracce che
  interrompono, quindi conviene averle già.

### Dipendenze con le tranche

- **AUDIO-3** (direttore della musica, gameplay-engineer + pixel-art-director). Aggiunge:
  - le sezioni `music.tracks` e `music.stingers` di `catalog.ron`;
  - il test di decodifica esteso a musica e stinger: campioni, picco, giro, attacco e coda
    (`AUDIO_HANDOFF_WORKFLOW.md`, INGEST).

  Con D-071 è diviso in due:
  - **AUDIO-3** subito, con la sola `home.ogg` (approvata e già promossa): direttore, due deck,
    dissolvenze; gli stati senza traccia sono silenzio;
  - **AUDIO-3b** quando arrivano le tracce della partita (RUN, HORDE, BOSS, STING): ducking,
    stinger, pausa e ripresa della traccia della run.
- ~~**AUDIO-4**~~ (ambienti) **annullato** (D-071): niente sezione `ambience` nel catalogo per
  la home. Il bus `Ambience` resta nel codice; un ambiente nuovo, per esempio di uno stage, si
  fa solo su una nuova richiesta dell'utente.
- Prima della tranche giusta un file approvato si può già promuovere (assets, manifest,
  licenza), ma il gioco non lo suona: la richiesta resta `APPROVED` fino a `INTEGRATED`.

## Coerenza fra le tracce

| Traccia | Richiesta | Tonalità e modo | BPM | Metro | Battute | Campioni a 44 100 Hz | Durata | Rapporto con le altre |
|---|---|---|---|---|---|---|---|---|
| `home` | MUSIC-HOME-001 | Fa lidio (tasti bianchi, Si naturale) | 80 | 4/4 | 24 | 3 175 200 | 72,0 s | Stesse sette note della run. 80 = 2/3 di 120: le terzine di crome della home vanno alla velocità delle crome della run |
| `run_meadow` | MUSIC-RUN-001 | Re dorico (tasti bianchi, Si naturale) | 120 | 4/4 | 48 | 4 233 600 | 96,0 s | Riferimento |
| `horde` | MUSIC-HORDE-001 | Re dorico, senza Si♭ | 120 | 4/4 | 24 | 2 116 800 | 48,0 s | Stessa tonalità e stesso tempo della run. Una battuta dura 2 s, quanto la dissolvenza |
| `boss` | MUSIC-BOSS-001 | Re minore (Si♭; Do♯ sulla dominante) | 144 | 4/4 | 48 | 3 528 000 | 80,0 s | Tonalità vicina: stessa tonica della run, una nota diversa. Arriva sempre dalla run, sotto lo stinger d'arrivo |
| `stinger_boss` | MUSIC-STING-001 | Re, senza terza, con Mi♭ di tensione | 144 (consigliati) | — | — | ≤ 132 300 | 2–3 s | Interrompe la run (Re dorico), apre il boss (Re minore) |
| `stinger_victory` | MUSIC-STING-001 | Re maggiore | libero | — | — | ≤ 220 500 | 3–5 s | Interrompe il boss (Re minore) e lo chiude in maggiore |
| `stinger_defeat` | MUSIC-STING-001 | Re minore | libero, rallentando | — | — | ≤ 176 400 | 2–4 s | Interrompe run, orda o boss: tutti su Re |
| ~~`home_portal`~~ | AMB-HOME-001, `REJECTED` (D-071) | ronzio su Fa (Fa1 43,65 Hz e armoniche) | — | — | — | 1 411 200 | 32,0 s | Ritirato: non è nel gioco |
| ~~`home_falls`~~ | AMB-HOME-001, `REJECTED` (D-071) | rumore, nessuna altezza | — | — | — | 1 190 700 | 27,0 s | Ritirato: non è nel gioco |
| `run_forest` (facoltativa) | nessuna | Re dorico, registro più grave e timbri più scuri | 120 | 4/4 | 48 | 4 233 600 | 96,0 s | Come la run: si incrocia con l'orda, quindi stesse note e stesso tempo |
| `results` (facoltativa) | nessuna | Re, accordi sospesi senza terza | 80 | 4/4 | 12 | 1 587 600 | 36,0 s | Neutra dopo la vittoria (Re maggiore) e dopo la sconfitta (Re minore); 80 BPM come la home, che segue |

Conti, verificati con Python (campioni per battito = 2 646 000 / BPM):

| Traccia | Conto | Campioni |
|---|---|---|
| home | 33 075 × 4 × 24 | 3 175 200 |
| run_meadow e run_forest | 22 050 × 4 × 48 | 4 233 600 |
| horde | 22 050 × 4 × 24 | 2 116 800 |
| boss | 18 375 × 4 × 48 | 3 528 000 |
| results | 33 075 × 4 × 12 | 1 587 600 |
| home_portal (ritirato, D-071) | 32 × 44 100 | 1 411 200 |
| home_falls (ritirato, D-071) | 27 × 44 100 | 1 190 700 |
| stinger (massimi) | 3, 5 e 4 s × 44 100 | 132 300, 220 500, 176 400 |

Le regole di questa tabella:
- **Run e orda:** stessa tonalità, stesso modo, stesso tempo, perché passano dall'una all'altra
  con dissolvenze di 2 s e i battiti non allineati.
- **Boss:** su una tonalità vicina, al massimo un'alterazione di differenza.
- **Home:** in relazione con la run, cioè la stessa collezione di note.
- **Stinger:** sulla tonica della traccia che interrompono.
- **Trasporre** si può, ma tutta la famiglia insieme e dello stesso intervallo, mai una traccia
  sola. Prima si aggiornano questa tabella e le richieste. (Il riaccordo del portale sulla
  tonica della home non serve più: ambienti ritirati, D-071.)
- **Cambiare tempo** si può solo con un BPM che dà campioni interi per battito (2 646 000 / BPM
  intero; elenco nella procedura), e con i campioni ricalcolati. Run e orda devono restare allo
  stesso tempo.

## Budget del peso

`docs/design/AUDIO_SYSTEM.md` § 3.11 fissa al massimo 8 MiB di audio in `dist/` (8 388 608
byte). `build-web` copia `assets/` così com'è, quindi conta la somma dei file in
`assets/audio/`.

Il tetto di ogni file è la durata × 16 000 byte (128 kbps medi). È la qualità Vorbis 3 (112 kbps
nominali) con circa il 14% di margine per il bitrate variabile: i suoni a onda quadra, ricchi di
armoniche, tendono a stare sopra il nominale. Conti fatti con Python:

| File | Durata | Nominale q3 | Tetto |
|---|---|---|---|
| `home.ogg` | 72 s | 1 008 000 | 1 152 000 (1,10 MiB) |
| `run_meadow.ogg` | 96 s | 1 344 000 | 1 536 000 (1,46 MiB) |
| `horde.ogg` | 48 s | 672 000 | 768 000 (0,73 MiB) |
| `boss.ogg` | 80 s | 1 120 000 | 1 280 000 (1,22 MiB) |
| `stinger_boss.ogg` | ≤ 3 s | 42 000 | 48 000 |
| `stinger_victory.ogg` | ≤ 5 s | 70 000 | 80 000 |
| `stinger_defeat.ogg` | ≤ 4 s | 56 000 | 64 000 |
| **Richieste, in tutto** | 308 s | 4 312 000 (4,11 MiB) | **4 928 000 (4,70 MiB)** |
| Riserva per gli effetti WAV (§ 3.11: 0,6–1 MB; oggi 13 030 byte per due file) | — | — | 1 000 000 (0,95 MiB) |
| **Totale al tetto** | — | — | **5 928 000 (5,65 MiB)**: margine 2 460 608 byte (2,35 MiB) |

Senza gli ambienti della home (`home_portal.ogg`, 512 000 byte, e `home_falls.ogg`, 432 000,
rifiutati: D-071) il totale al tetto scende di 944 000 byte. Prima era 6 872 000 (6,55 MiB).

Pesi misurati:
- `home.ogg` v1, promossa il 2026-11-02: 465 651 byte, il 40% del tetto (51,7 kbps medi). Con
  il suo peso vero al posto del tetto il totale fa 5 241 651 byte (5,00 MiB);
- oggi in `assets/audio/` ci sono 480 707 byte: `home.ogg`, i due effetti (13 030) e
  `catalog.ron` (2 026). È il 5,7% degli 8 MiB.

`home.ogg` è un file di sintesi, meno denso di una musica suonata: i pesi delle altre tracce non
si stimano da questo.

Le tracce facoltative, sul totale al tetto:
- `results` a q3 (36 s, tetto 576 000 byte, 0,55 MiB) porta il totale a 6 504 000 byte
  (6,20 MiB): ci sta.
- `run_forest` a q3 (96 s, tetto 1 536 000 byte, 1,46 MiB) lo porta a 7 464 000 byte
  (7,12 MiB): ci sta. Con gli ambienti non ci stava.
- Tutte e due a q3 fanno 8 040 000 byte (7,67 MiB): ci stanno, con un margine di 348 608 byte
  (0,33 MiB). A qualità 2 (circa 96 kbps, tetto 13 000 byte/s) `run_forest` scende a 1 248 000
  byte e il margine sale a 636 608.

Il validatore controlla, dalla prima promozione: lo `sha256`, i campioni e i byte di ogni Ogg
uguali al manifest, il peso entro durata × 16 000 byte e la somma dei file di `assets/audio/`
entro gli 8 MiB (`AUDIO_HANDOFF_WORKFLOW.md`, PROMOTE, punto 7).

## Facoltativi, senza richiesta

Da `docs/design/AUDIO_SYSTEM.md` § 5. Una richiesta si scrive solo se l'utente li vuole:
- **`run_forest`:** Bosco Ombroso. Fino ad allora il bosco usa `run_meadow`, che è il ripiego di
  ogni stage. Dati nella tabella sopra; senza gli ambienti ci sta anche a qualità 3 (budget
  sopra).
- **`results`:** schermata dei risultati. Senza questa traccia, dopo lo stinger c'è silenzio.
  Dati nella tabella sopra.
- ~~**`home_portal_pulse`**~~: un impulso raro delle rune del portale, 1–2 s, ogni 6–12 s.
  Ritirato con gli ambienti della home (D-071).
