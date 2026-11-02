# MUSIC-STING-001 — Tre stinger: arrivo del boss, vittoria, sconfitta

Richiesta di musica con l'handoff manuale (D-070). Procedura in
`docs/audio/AUDIO_HANDOFF_WORKFLOW.md`. Le sezioni 1–8 sono il brief da girare a chi compone e
bastano da sole; le altre servono al progetto.

| Campo | Valore |
|---|---|
| ID | `MUSIC-STING-001` (tre parti: `boss`, `victory`, `defeat`) |
| Tipo | `stinger` (one-shot, non loop) |
| File nel gioco | `assets/audio/music/stinger_boss.ogg`, `stinger_victory.ogg`, `stinger_defeat.ogg` |
| Canali ammessi | composizione dell'utente, commissione a una persona, file CC0 o CC-BY scelti dall'utente; servizio IA solo con nome, termini e via esplicito per ciascun file |
| Stato | `REQUESTED` (2026-11-02), per tutte e tre le parti |
| Owner | pixel-art-director (richiesta, controllo, promozione) + utente (consegna e approvazione) |
| Tranche che lo usa | AUDIO-3b (D-071: AUDIO-3 porta solo la musica della home) |
| Decisioni | D-069, D-070 |
| Dipende da | `MUSIC-RUN-001` e `MUSIC-BOSS-001`: tonalità e suoni delle tracce che gli stinger interrompono. Meglio stesso autore |

## Il gioco in breve

*Companion Survivors* è un gioco d'azione 2D in pixel art visto dall'alto:
- si guida un ragazzo in un'arena aperta, mentre piccole creature arrivano a ondate da ogni
  lato, a centinaia;
- i suoi compagni, mostriciattoli di fuoco, acqua, foglie e fulmine, attaccano da soli;
- una partita dura 15 minuti: due orde enormi, poi un boss;
- tutte le musiche della partita hanno la tonica Re:
  - partita e orde in Re dorico, 120 BPM;
  - boss in Re minore, 144 BPM.

## 1. Dove suonano e cosa succede attorno

- Uno **stinger** è una frase breve che suona una volta sola, sopra la musica, nei momenti
  chiave. Non è un loop.
- Mentre suona, il gioco abbassa la musica sotto. Poi la musica cambia, con le dissolvenze
  indicate per ogni stinger.
- Lo stinger parte all'istante dell'evento, quindi l'attacco deve essere immediato.
- Gli effetti sonori continuano sopra: colpi, raccolte, avvisi.

| Parte | Quando | Che cosa interrompe | Che cosa viene dopo |
|---|---|---|---|
| `boss` | alle 15:00, all'arrivo del boss | la musica della partita (Re dorico, 120 BPM) | la musica del boss (Re minore, 144 BPM), con una dissolvenza incrociata di 1,5 s |
| `victory` | quando il boss cade | la musica del boss (Re minore, 144 BPM) | la schermata dei risultati, con una dissolvenza di 0,6 s |
| `defeat` | quando il giocatore cade, in qualunque momento | la musica della partita, dell'orda (Re dorico, 120 BPM) o del boss (Re minore, 144 BPM) | la schermata dei risultati, con una dissolvenza di 0,6 s |

## 2. Carattere e immagini

- **`boss`:** un colpo che fa alzare la testa. Qualcosa di enorme è arrivato: un gigante
  corazzato. Minaccia e peso, ma niente horror.
- **`victory`:** il gigante cade. Sollievo e luce, una chiusura breve, luminosa e calda.
  Contenta, non trionfalistica.
- **`defeat`:** si cade. Una frase breve che scende e si spegne con dolcezza, un po'
  malinconica. Né ridicola né drammatica.

## 3. Palette strumentale

Gli stessi suoni della musica della partita e del boss, originali, ispirati ai giochi a 16 bit
degli anni '90:
- onda quadra (duty 12,5–50%), triangolo, FM semplice (campane, ottoni), rumore;
- cassa fatta con un seno che scende rapido di altezza;
- piatti di rumore corti;
- al massimo 8 voci insieme.

Per parte:
- **`boss`:**
  - colpo grave immediato: cassa, tom e rumore insieme;
  - accordo di Re senza terza (Re–La–Re), con un Mi bemolle sopra come tensione, in ottoni FM o
    quadra saturata;
  - coda breve.
- **`victory`:**
  - salita per gradi o arpeggio di Re maggiore in quadra o campana FM;
  - accordo finale tenuto che si spegne, con un piatto morbido.
- **`defeat`:**
  - linea che scende e rallenta, in triangolo o quadra morbida;
  - ultimo accordo grave di Re minore, o una quinta vuota, che si spegne.

Da evitare in tutte: orchestre realistiche, cori, voci, effetti comici.

**Divieti di originalità:**
- nessuna fanfara di vittoria riconoscibile: niente note veloci ribattute seguite da una nota
  lunga, niente formule che sembrano un jingle famoso;
- per la sconfitta niente "trombone che scende" da cartone;
- se una frase suona "già sentita", si cambia.

## 4. Dati musicali

| Parte | Tonalità | Tempo | Durata, code comprese | Massimo in campioni a 44 100 Hz |
|---|---|---|---|---|
| `boss` | Re (accordo senza terza, Mi♭ di tensione) | 144 BPM consigliati, quelli del boss | 2–3 s, consigliati 2,5 | **132 300** (3,0 s) |
| `victory` | Re maggiore (chiude il Re minore del boss) | libero; 144 BPM per proseguire il boss | 3–5 s, consigliati 4 | **220 500** (5,0 s) |
| `defeat` | Re minore | libero, rallentando (per esempio da 120 a circa 70 BPM) | 2–4 s, consigliati 3 | **176 400** (4,0 s) |

Conti: massimo = secondi × 44 100, cioè 3,0 × 44 100 = 132 300; 5,0 × 44 100 = 220 500;
4,0 × 44 100 = 176 400. Gli stinger non sono loop: la durata può essere qualunque entro il
massimo, non serve una lunghezza esatta.

Esempio per `boss` a 144 BPM: due tempi di colpi (2 × 18 375 = 36 750 campioni, 0,83 s), poi
l'accordo e la coda, in tutto circa 2,5 s.

## 5. Regole del one-shot

- **Attacco immediato:**
  - nessun silenzio all'inizio;
  - il primo campione vicino allo zero (|x| ≤ 0,01, cioè −40 dBFS) in tutti e due i canali;
  - il suono oltre −40 dBFS entro 10 ms.
- **Coda breve che si chiude da sola:**
  - riverbero ed eco si spengono naturalmente entro la durata massima;
  - gli ultimi 10 ms stanno sotto −60 dBFS, l'ultimo campione vicino allo zero;
  - niente coda troncata, niente dissolvenza artificiale lunga.
- **Non sono loop:** nessuna regola di giro.

## 6. Formato ed esportazione

- Ogg Vorbis, stereo, 44 100 Hz, qualità 3 (circa 112 kbps nominali, a bitrate variabile).
- **Peso massimo:** durata massima × 16 000 byte (128 kbps medi):
  - `boss` **48 000 byte**;
  - `victory` **80 000 byte**;
  - `defeat` **64 000 byte**.
- Niente infrabassi sotto i 30–35 Hz. Compatibili in mono.
- **Master** facoltativi ma consigliati: WAV PCM 24 bit (o 16), 44 100 Hz.
- **Esportazione**, per esempio con Audacity:
  - un file per stinger;
  - formato Ogg Vorbis, qualità 3;
  - taglio automatico del silenzio spento: l'attacco va curato a mano.

## 7. Loudness e picchi

- Circa **−16 LUFS integrati** sull'intero file, ±1,5 LU. Suonano sopra la musica abbassata, che
  è allo stesso livello nel file.
- **True peak ≤ −1 dBTP** sull'Ogg; sul master ≤ −1,5 dBTP. Un colpo grave e brusco tende a
  superare il picco: va limitato senza schiacciarlo.

## 8. Consegna, provenienza e licenze

- **File:**
  - `MUSIC-STING-001_boss_v1.ogg`;
  - `MUSIC-STING-001_victory_v1.ogg`;
  - `MUSIC-STING-001_defeat_v1.ogg`.

  Ogni parte ha le sue revisioni (v2, v3…). I master hanno lo stesso nome con `.wav` (o
  `.flac`). Si possono consegnare anche una parte per volta.
- **Dove:** chi compone consegna all'utente; l'utente salva in `.audio-inbox/MUSIC-STING-001/`
  e scrive `MUSIC-STING-001 pronto`.
- **`PROVENANCE.txt`** nella stessa cartella, con una sezione `[file]` per ogni stinger:

  ```text
  [file]
  nome: MUSIC-STING-001_boss_v1.ogg
  autore: ...
  strumento: ...
  licenza: user-owned | CC0-1.0 | CC-BY-4.0
  url: ... (oppure -)
  data: AAAA-MM-GG
  attribuzione: ... (oppure -)
  termini: ... (commissione o servizio IA, oppure -)
  master: MUSIC-STING-001_boss_v1.wav | nessuno
  note: ...
  ```

  Ogni campione di altri usato dentro va in una sezione `[fonte]`: descrizione, autore, licenza,
  URL, data, file della licenza salvata in `licenses/`.
- **Licenze ammesse:**
  - `user-owned`: composti dall'utente, oppure commissionati con un accordo scritto che permette
    uso nel gioco (nativo e web, anche a pagamento), modifiche e distribuzione senza scadenza;
  - `CC0-1.0`;
  - `CC-BY-4.0`, con il testo dell'attribuzione;
  - un servizio IA solo se l'utente dichiara nome, piano e termini che gli danno i diritti
    sull'output, e dà un via esplicito per ciascun file.
- **Escluse:**
  - licenze con `-NC` o `-ND`;
  - "royalty free" senza un testo chiaro;
  - estratti di altri giochi, film o canzoni;
  - imitazioni di jingle riconoscibili;
  - musica da servizi IA senza diritti chiari.

## 9. Controlli dell'agente e criteri di accettazione

Per ogni parte:
- [ ] nome, sezione in `PROVENANCE.txt`, licenza ammessa, licenze delle fonti salvate
- [ ] Ogg Vorbis valido: un solo flusso, BOS ed EOS, pagine consecutive, CRC giusti
- [ ] 2 canali, 44 100 Hz
- [ ] durata entro il massimo e dentro la fascia: `boss` 2–3 s, `victory` 3–5 s, `defeat`
      2–4 s
- [ ] peso entro il massimo
- [ ] picco ≤ −1 dBTP (master ≤ −1,5)
- [ ] −16 LUFS ±1,5
- [ ] attacco: primo campione |x| ≤ 0,01 e oltre −40 dBFS entro 10 ms
- [ ] coda: ultimi 10 ms sotto −60 dBFS
- [ ] tonalità della tabella, suoni della stessa famiglia della musica della partita e del boss
- [ ] ascolto dell'utente, anche sopra la traccia che interrompe: niente di riconoscibile,
      nessuna stonatura
- [ ] conferma esplicita dell'utente su quella parte

Ogg, header, pagine, CRC, granule e peso si misurano all'arrivo, con il comando della procedura.
Si passano il centro della fascia e metà della sua ampiezza come tolleranza, così il comando
controlla proprio la fascia:

| Parte | Comando | Fascia controllata |
|---|---|---|
| `boss` | `... MUSIC-STING-001_boss_v1.ogg 110250 22050` | 88 200–132 300 campioni (2–3 s) |
| `victory` | `... MUSIC-STING-001_victory_v1.ogg 176400 44100` | 132 300–220 500 campioni (3–5 s) |
| `defeat` | `... MUSIC-STING-001_defeat_v1.ogg 132300 44100` | 88 200–176 400 campioni (2–4 s) |

Picco, loudness, attacco e coda si misurano:
- sul master WAV, se c'è;
- sull'Ogg decodificato, nel test di decodifica esteso in AUDIO-3.

Quello che non si misura si scrive "non misurato".

## 10. Piano di verifica

- **INGEST:**
  - comando per l'Ogg su ogni parte;
  - analisi dei master;
  - ascolto dell'utente, anche sopra la traccia che lo stinger interrompe.
- **VERIFY:**
  - `python -I scripts/validate-assets.py`;
  - `cargo test --lib audio`;
  - `cargo run -- --autoplay --god --time-scale 4`: arrivo del boss alle 15:00 (circa 225 s
    reali);
  - vittoria e sconfitta in una run vera (senza `--god` per la sconfitta);
  - in ogni caso, musica abbassata sotto lo stinger e passaggio alla traccia successiva;
  - ascolto dell'utente nel gioco.

## Registro delle versioni

| Parte | Versione | File | Data | Controlli | Decisione dell'utente |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

## Revisioni

(Solo le correzioni, per parte, con i punti in secondi, e il nome del file `v<N+1>`.)

## Provenienza e licenza (dopo la promozione)

- **Versioni promosse e file:** …
- **Consegnate il / approvate il:** … / … (frase dell'utente: "…")
- **Canale e autore:** …
- **Licenza (`license` del manifest):** …
- **Attribuzione:** … / non serve
- **Note in `assets-src/audio/music/stinger_*.md`:** sì / no
- **Voci in `ASSET_LICENSES.md`:** sì / no
