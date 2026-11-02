# Richieste di musica e ambienti

Indice delle richieste dell'handoff manuale (D-070). La procedura è in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`, il modello in `docs/audio/AUDIO_REQUEST_TEMPLATE.md`.
Ogni richiesta si regge da sola. Questo file tiene insieme quello che le lega: ordine,
dipendenze, tonalità, tempi e peso.

## Indice e ordine consigliato

| Ordine | ID | File nel gioco | Che cosa | Dipende da | Tranche | Stato |
|---|---|---|---|---|---|---|
| 1 | [`MUSIC-RUN-001`](MUSIC-RUN-001.md) | `music/run_meadow.ogg` | partita nella Prateria Verde, e ripiego per ogni stage | — | AUDIO-3 | `REQUESTED` |
| 2 | [`MUSIC-HORDE-001`](MUSIC-HORDE-001.md) | `music/horde.ogg` | orde delle 5:00 e delle 10:00 | RUN: tonalità, tempo, suoni | AUDIO-3 | `REQUESTED` |
| 3 | [`MUSIC-HOME-001`](MUSIC-HOME-001.md) | `music/home.ogg` | pagina iniziale e menu | RUN: stesse sette note | AUDIO-3 | `REQUESTED` |
| 4 | [`AMB-HOME-001`](AMB-HOME-001.md) | `ambience/home_portal.ogg`, `ambience/home_falls.ogg` | portale; cascate e fiume | HOME: il portale ronza sul suo Fa | AUDIO-4 | `REQUESTED` |
| 5 | [`MUSIC-BOSS-001`](MUSIC-BOSS-001.md) | `music/boss.ogg` | boss delle 15:00 | RUN: suoni, tonica | AUDIO-3 | `REQUESTED` |
| 6 | [`MUSIC-STING-001`](MUSIC-STING-001.md) | `music/stinger_boss.ogg`, `stinger_victory.ogg`, `stinger_defeat.ogg` | arrivo del boss, vittoria, sconfitta | RUN e BOSS: tonalità, suoni | AUDIO-3 | `REQUESTED` |

I percorsi sono relativi ad `assets/audio/`.

Perché quest'ordine:
- **1. RUN:** è la musica che si sente di più ed è il riferimento di palette, tonalità e tempo
  per orda, boss e stinger. Basta lei per provare in AUDIO-3 il rischio più grosso: il loop
  Vorbis in Bevy, che non deve aprire un vuoto al giro (AUDIO_BIBLE § 4).
- **2. HORDE:** subito dopo, perché il passaggio tra partita e orda è il più delicato:
  dissolvenze di 2 s in entrata e in uscita, battiti in fase casuale. Meglio stesso autore e
  stessa sessione della run.
- **3. HOME:** è la prima cosa che si sente, e il ronzio del portale prende la sua tonica.
- **4. AMB-HOME:** va su un'altra strada (registrazioni o sintesi), quindi può procedere in
  parallelo a tutte le altre. Dipende dalla home solo per l'altezza del ronzio: se la home
  cambia tonalità, il portale si riaccorda.
- **5–6. BOSS e STINGER:** per ultimi. Gli stinger devono stare bene sopra le tracce che
  interrompono, quindi conviene averle già.

### Dipendenze con le tranche

- **AUDIO-3** (direttore della musica, gameplay-engineer + pixel-art-director). Aggiunge:
  - le sezioni `music.tracks` e `music.stingers` di `catalog.ron`;
  - il test di decodifica esteso a musica e stinger: campioni, picco, giro, attacco e coda
    (`AUDIO_HANDOFF_WORKFLOW.md`, INGEST).

  Il backlog chiede "tracce provvisorie", ma qui non c'è un encoder Ogg e il catalogo vuole Ogg
  per la musica. Il progetto da solo non può fare tracce provvisorie: servono una prima
  consegna dell'utente (anche grezza, una v1 di RUN), oppure il via per `oggenc` o `ffmpeg`.
  È una decisione dell'utente.
- **AUDIO-4** (ambienti, gameplay-engineer + pixel-art-director). Aggiunge la sezione `ambience`
  del catalogo e la stessa estensione del test.
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
| `home_portal` | AMB-HOME-001 | ronzio su Fa (Fa1 43,65 Hz e armoniche) | — | — | — | 1 411 200 | 32,0 s | Intonato sulla tonica della home |
| `home_falls` | AMB-HOME-001 | rumore, nessuna altezza | — | — | — | 1 190 700 | 27,0 s | 27 e 32 s si ripetono insieme solo ogni 864 s |
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
| home_portal | 32 × 44 100 | 1 411 200 |
| home_falls | 27 × 44 100 | 1 190 700 |
| stinger (massimi) | 3, 5 e 4 s × 44 100 | 132 300, 220 500, 176 400 |

Le regole di questa tabella:
- **Run e orda:** stessa tonalità, stesso modo, stesso tempo, perché passano dall'una all'altra
  con dissolvenze di 2 s e i battiti non allineati.
- **Boss:** su una tonalità vicina, al massimo un'alterazione di differenza.
- **Home:** in relazione con la run, cioè la stessa collezione di note.
- **Stinger:** sulla tonica della traccia che interrompono.
- **Trasporre** si può, ma tutta la famiglia insieme e dello stesso intervallo, mai una traccia
  sola. Prima si aggiornano questa tabella e le richieste; il portale si riaccorda sulla
  tonica della home.
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
| `home_portal.ogg` | 32 s | 448 000 | 512 000 (0,49 MiB) |
| `home_falls.ogg` | 27 s | 378 000 | 432 000 (0,41 MiB) |
| **Richieste, in tutto** | 367 s | 5 138 000 (4,90 MiB) | **5 872 000 (5,60 MiB)** |
| Riserva per gli effetti WAV (§ 3.11: 0,6–1 MB; oggi 13 030 byte per due file) | — | — | 1 000 000 (0,95 MiB) |
| **Totale al tetto** | — | — | **6 872 000 (6,55 MiB)**: margine 1 516 608 byte (1,45 MiB) |

Le tracce facoltative:
- `results` a q3 (36 s, tetto 576 000 byte, 0,55 MiB) porta il totale a 7,10 MiB: ci sta.
- `run_forest` a q3 (96 s, tetto 1,46 MiB) porta il totale a 8,01 MiB: non ci sta. Ci sta a
  qualità 2 (circa 96 kbps, tetto 13 000 byte/s = 1,19 MiB): totale 7,74 MiB.
- Tutte e due insieme a q2 fanno 8,19 MiB: non ci stanno al tetto. Si decide sui pesi misurati
  dei file già consegnati: al nominale le richieste fanno 4,90 MiB, e allora ci starebbero.

Il validatore controllerà il totale alla prima promozione (`AUDIO_HANDOFF_WORKFLOW.md`, PROMOTE,
punto 7).

## Facoltativi, senza richiesta

Da `docs/design/AUDIO_SYSTEM.md` § 5. Una richiesta si scrive solo se l'utente li vuole:
- **`run_forest`:** Bosco Ombroso. Fino ad allora il bosco usa `run_meadow`, che è il ripiego di
  ogni stage. Dati nella tabella sopra; peso solo a qualità 2, salvo margine misurato.
- **`results`:** schermata dei risultati. Senza questa traccia, dopo lo stinger c'è silenzio.
  Dati nella tabella sopra.
- **`home_portal_pulse`:** un impulso raro delle rune del portale, 1–2 s, ogni 6–12 s. È un
  one-shot breve: si può sintetizzare nel progetto come gli effetti (D-070), senza handoff. Si
  decide in AUDIO-4.
