# AUDIO_BIBLE — regole degli asset audio

Breve, come `docs/ART_BIBLE.md` per la grafica. Progetto del sistema: `docs/design/AUDIO_SYSTEM.md`;
decisioni D-069 (sistema, formati) e D-070 (origine degli asset, proprietario). Proprietario
degli asset audio e di questo file: pixel-art-director. Il codice di `src/audio/` è del
gameplay-engineer.

## 1. Stile sonoro

- **Chip e retro, come il pixel art:** onde semplici (quadra con duty, triangolo, seno, rumore a
  campioni tenuti), inviluppi brevi, nessun riverbero negli effetti.
- **Leggibile nelle orde:** attacco netto, coda corta, pochi suoni lunghi. Il registro acuto
  e i timbri più duri vanno a ciò che conta (player colpito, livello, preavvisi); colpi e
  raccolte restano morbidi, perché suonano centinaia di volte.
- **Famiglie per ruolo** (proposta per AUDIO-5, da provare in gioco):
  - compagni con un timbro per elemento: fuoco con rumore e quadra in discesa, acqua con seno
    o triangolo e vibrato, natura con triangolo morbido, elettrico con quadra stretta e
    rumore;
  - nemici più gravi e più rumorosi;
  - raccolte con toni puri in salita;
  - interfaccia con blip brevi nel registro medio-acuto.
- **Originalità:** suoni nostri, mai imitazioni di effetti o jingle di altri giochi (Pokémon,
  Nintendo e simili). Intervalli e contorni generici, mai una sequenza riconoscibile: per
  esempio niente "nota breve e poi una quarta sopra lunga" su onda quadra (il contorno della
  moneta di un gioco famoso), niente fanfare di vittoria note.

## 2. Formati per bus

| Uso | Formato | Canali | Frequenza | Durata | Cartella |
|---|---|---|---|---|---|
| Effetti, bus `Sfx` e `Ui` (i `cues` del catalogo) | WAV PCM 16 bit, header canonico | mono | 44 100 Hz | ≤ 1,5 s | `assets/audio/sfx/<gruppo>/` |
| Musica e stinger (AUDIO-3) | Ogg Vorbis | stereo | 44 100 Hz | loop 45–120 s, stinger 2–5 s | `assets/audio/music/` |
| Ambienti (quelli della home e AUDIO-4 ritirati, D-071: un ambiente nuovo solo su richiesta dell'utente) | Ogg Vorbis | stereo | 44 100 Hz | loop 20–40 s | `assets/audio/ambience/` |

- **Effetti in WAV:** sul web la decodifica gira sul thread del gioco (D-069).
- **Musica e ambienti in Ogg:** pesano meno da scaricare. Qui non c'è un encoder, quindi li
  consegna l'utente con l'handoff manuale (D-070): richiesta in `docs/audio/requests/<ID>.md`,
  file in `.audio-inbox/<ID>/`, integrazione solo dopo la sua approvazione esplicita.
  - Procedura: `docs/audio/AUDIO_HANDOFF_WORKFLOW.md`.
  - Indice, tonalità, tempi e budget: `docs/audio/requests/README.md`.
- **Qualità e peso dei file Ogg:**
  - Vorbis qualità 3 (circa 112 kbps nominali, a bitrate variabile);
  - tetto per file: durata × 16 000 byte (128 kbps medi);
  - tutto l'audio in `dist/` entro 8 MiB (`docs/design/AUDIO_SYSTEM.md` § 3.11).

## 3. Loudness e picchi

- **Effetti:**
  - picco di campione a −3 dBFS, normalizzato dalla sintesi, e mai oltre −1 dBFS (lo
    controlla il validatore);
  - `audio_sfx.py` stima anche il picco tra i campioni (interpolazione 4×) e si ferma se
    supera −1 dBFS;
  - il margine copre quel picco e le voci sommate nel mix, che il catalogo limita con
    `sfx_headroom` e il boost di coalescenza.
- **Volumi relativi nel catalogo, non nei file:** tutti i file al picco di riferimento; il
  peso di un suono si regola con il `gain` del cue (0,3–0,8 di partenza) e i trim dei bus.
- **Niente click:**
  - ogni effetto inizia e finisce a zero, con una dissolvenza a coseno di almeno 1 ms
    (predefinite 2 ms all'inizio e 5 ms alla fine);
  - il silenzio finale si taglia sotto −60 dB dal picco;
  - il quadrato è a media nulla, così l'inviluppo non dà gradini di continua.
- **Musica e ambienti:**
  - musica: −16 LUFS integrati, ±1 LU;
  - stinger: −16 LUFS integrati sull'intero file, ±1,5 LU. Suonano sopra la musica abbassata;
  - ambienti 6–8 dB sotto la musica: cascate −22 LUFS, portale −24 LUFS, ±1,5 LU. La misura
    pesa poco i bassi, e il ronzio del portale è soprattutto grave (portale e cascate della
    home ritirati, D-071: i livelli restano come riferimento per un ambiente nuovo);
  - picco reale ≤ −1 dBTP sull'Ogg; sul master ≤ −1,5 dBTP, perché la codifica Vorbis alza un
    po' i picchi;
  - niente infrabassi sotto i 30–35 Hz; compatibili in mono.
- **Come si misurano** (`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`, INGEST):
  - all'arrivo, sul master WAV, con la libreria standard: picco, stima tra i campioni, loudness
    BS.1770;
  - dopo la promozione, sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3 (AUDIO-4
    annullato, D-071): picco di campione;
  - il validatore legge l'header Vorbis e il granule finale, non decodifica;
  - senza master WAV, quei valori restano "non misurati" fino al test.

## 4. Regole dei loop (musica e ambienti)

- **Loop senza stacco:** l'ultimo campione si raccorda al primo (stessa fase, nessun gradino),
  e il file non ha silenzio né dissolvenze all'inizio o alla fine. Le dissolvenze le fa il
  gioco (`fades` e `fade_in_s` del catalogo).
- **Code:** la coda del riverbero si ripiega all'inizio del loop, non si taglia. Si esporta il
  brano due volte di seguito e si tiene il secondo giro, oppure si somma all'inizio la coda che
  supera la fine.
- **Estremi:** il primo e l'ultimo campione vicini allo zero (|x| ≤ 0,01, cioè −40 dBFS) in
  tutti e due i canali.
  - Ci si arriva ruotando il loop di pochi campioni, senza cambiare la lunghezza, mai con una
    dissolvenza.
  - Così un eventuale vuoto al giro diventa un silenzio brevissimo e non un clic.
- **Durata:** in battute intere, con la lunghezza esatta in campioni e una tolleranza di ±1
  campione.
  - I BPM devono dare campioni interi per battito: 2 646 000 / BPM intero.
  - BPM, tonalità e punto di loop vanno in una nota in `assets-src/audio/music/<id>.md` (o
    `ambience/`).
- **Ambienti:**
  - l'incrocio che chiude il loop sta dentro il file, mai agli estremi;
  - tolleranza ±0,5 s dalla lunghezza della richiesta;
  - due loop che suonano insieme hanno lunghezze diverse e non multiple (per la home erano 32 e
    27 s; ritirati, D-071);
  - nessun evento riconoscibile che faccia sentire il giro.
- **Tonalità** (tabella in `docs/audio/requests/README.md`):
  - run e orda nella stessa tonalità, modo e tempo, perché si incrociano per 2 s con i battiti
    non allineati;
  - boss su una tonalità vicina, al massimo un'alterazione di differenza;
  - home con la stessa collezione di note della run;
  - stinger sulla tonica della traccia che interrompono;
  - ~~il ronzio del portale sulla tonica della home~~ (ritirato, D-071).
- **Stinger:**
  - one-shot con la loro coda naturale, entro 5 s;
  - attacco immediato: primo campione vicino allo zero, oltre −40 dBFS entro 10 ms;
  - ultimi 10 ms sotto −60 dBFS.
- **Da verificare in AUDIO-3:** che la riproduzione in loop di Bevy 0.20 con Vorbis non apra
  un vuoto al giro. Non è ancora provato.

## 5. Nomi

- **File:** `^[a-z0-9_]+\.(wav|ogg)$`, cartelle in minuscolo; le varianti `_1`, `_2`, …
  (`ember_shot_1.wav`).
- **Patch:** `assets-src/audio/sfx/<gruppo>_<nome>.json` scrive
  `assets/audio/sfx/<gruppo>/<nome>.wav` (campo `out`), per esempio `ui_tick.json` scrive
  `ui/tick.wav`.
- **Cue:** parole minuscole separate da un punto, come nel catalogo (`ui.tick`, `shot.ember`,
  `enemy.die.small`).

## 6. Pipeline degli effetti

1. Scrivi o modifica la patch in `assets-src/audio/sfx/` (JSON, nessun parser esterno).
   - Il formato è in testa a `scripts/audio_sfx.py`.
   - Contenuto: strati sine, square con `duty`, triangle e noise con `seed`; inviluppo ADSR
     con `curve`; `sweep` di frequenza; `vibrato`; `lowpass_hz`.
   - Un campo sconosciuto è un errore.
2. `python -I scripts/audio_sfx.py` genera i WAV e le voci `kind: "sfx"` del manifest.
   - Ogni voce contiene percorso, canali, frequenza, bit, campioni, durata, picco, `sha256`,
     cue, patch, licenza e `approved`.
   - È deterministico: un file uguale non si riscrive e la stessa patch dà lo stesso file al
     byte. Il rumore usa solo `random.Random(seed).random()`, stabile tra le versioni di
     Python.
   - `--check` non scrive nulla e fallisce se qualcosa cambierebbe.
3. Solo dopo che il WAV esiste, aggiungi il cue in `assets/audio/catalog.ron`.
4. Aggiungi una riga nella sezione "Audio" di `ASSET_LICENSES.md`.
5. `python -I scripts/validate-assets.py`, poi `cargo test --lib audio` (il test del catalogo
   reale).

Il validatore rifiuta:

- in `assets/audio/` file diversi da `catalog.ron`, `.wav` e `.ogg`;
- nomi fuori schema;
- file senza voce audio nel manifest, o voci senza file;
- licenze fuori elenco (`original`, `user-owned`, `CC0-1.0`, `CC-BY-4.0` con `attribution`);
- file senza riga in `ASSET_LICENSES.md`;
- effetti non PCM 16 bit mono 44 100 Hz, oltre 1,5 s o con il picco oltre −1 dBFS, o diversi
  dalla voce del manifest (durata, `sha256`);
- Ogg senza header Vorbis, con una pagina senza `OggS` o troncata (i CRC li controlla
  `scripts/audio_inbox.py`);
- Ogg diversi dalla voce del manifest: canali, frequenza, campioni (granule finale), durata,
  byte, `sha256`; voci senza `samples`, `bytes` o `sha256`, o senza la nota in `assets-src/`;
- Ogg oltre il tetto del peso (durata × 16 000 byte);
- `assets/audio/` oltre gli 8 MiB in tutto;
- percorsi `"audio/…wav|ogg"` di `catalog.ron` assenti dal manifest.

Un file audio con la sua voce che il catalogo non cita è solo un avviso: un file approvato può
stare in `assets/` prima della sua sezione del catalogo, e intanto il gioco non lo suona.

Altre regole:

- **Manifest condiviso:** `build-assets.py` riscrive le voci delle immagini e conserva in coda
  quelle audio, quindi i due script si lanciano in qualunque ordine.
- **Approvazione:** `approved` resta `null` finché l'utente non ha ascoltato il suono nel
  gioco. Con il suo via si scrive a mano data e chi (per esempio
  `"2026-11-02, utente"`), e `audio_sfx.py` lo conserva finché lo `sha256` non cambia.
  - Vale anche per musica e ambienti consegnati: la conferma della versione (stato `APPROVED`
    della richiesta) permette la promozione; `approved` nel manifest si scrive dopo l'ascolto
    nel gioco (`INTEGRATED`).

## 7. Effetti attuali

| Cue | File | Durata | Com'è fatto |
|---|---|---|---|
| `ui.tick` | `audio/sfx/ui/tick.wav` | 37,9 ms | quadra al 25% da 1320 a 990 Hz in 30 ms, triangolo all'ottava sotto, clic di rumore di 5 ms, passa-basso a 7 kHz |
| `ui.toggle` | `audio/sfx/ui/toggle.wav` | 108,8 ms | due note brevi Do6 e Mi6 (quadra al 12,5% con un pizzico di altezza, triangolo all'ottava sotto, clic di rumore), passa-basso a 7,5 kHz |

Nel catalogo tutti e due sono `Ui`, `High`, cooldown 40 ms, 2 istanze, senza variazione di
guadagno né di altezza (la tacca è l'anteprima del volume). Sono provvisori, non ancora
ascoltati dall'utente.
