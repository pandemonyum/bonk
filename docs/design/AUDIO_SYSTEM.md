# Sistema audio — documento di progettazione (proposta, AUDIO-0)

Stato: **proposta da approvare**. Nessun codice, asset, dipendenza o file di configurazione è
stato modificato. Dopo il via dell'utente l'orchestratore registra le decisioni (D-069, D-070),
le righe del backlog e l'ownership in `AGENTS.md`, poi parte AUDIO-1.

Fonti lette: `AGENTS.md`, `CLAUDE.md`, `docs/ARCHITECTURE.md`, `docs/PROGRESS.md`,
`docs/BACKLOG.md`, `docs/DECISIONS.md`, handoff §6 (P2.4, P3.3), il codice di `src/` e i
sorgenti di `bevy_audio` 0.20.0, `rodio` 0.22.2 e `cpal` 0.17.3 nel registro locale di cargo.

---

## 0. In breve

- Un modulo nuovo, `src/audio/`, aggiunto **solo** a `client_app`. La simulazione non lo
  conosce: l'audio osserva i fatti di gioco (componenti aggiunti o rimossi, contatori, stati),
  non scrive nulla nel mondo della simulazione e non usa i suoi RNG.
- Tre livelli separati: **ascolto** (fatti di gioco → richieste di suono, `Cue`), **politica**
  (limiti, cooldown, priorità, varianti, coalescenza: funzioni pure, testate senza scheda audio),
  **riproduzione** (l'unico punto che usa `bevy_audio`).
- Quattro bus con volumi indipendenti: generale, musica, effetti, ambiente. L'interfaccia usa il
  bus degli effetti.
- Musica in loop con un direttore a due "deck" e dissolvenze incrociate tra home, run, orda,
  boss, vittoria e sconfitta. Abbassamento automatico (ducking) in pausa e nel draft.
- Home: due loop ambientali (portale, cascate) con dissolvenza in entrata e in uscita.
- Impostazioni nel profilo già esistente (file su Windows, `localStorage` sul web), con
  `#[serde(default)]`: i profili vecchi si leggono.
- Web: l'`AudioContext` nasce già dopo il clic su "Clicca per giocare". Una piccola protezione
  nella pagina lo riprende al primo tasto e lo sospende quando la scheda è nascosta.
- Licenze: ogni file audio ha una voce nel manifest e in `ASSET_LICENSES.md`, con origine,
  autore, licenza SPDX e approvazione. Il validatore rifiuta i file senza voce.
- Serve il tuo via su: **crate nuovi da scaricare** (`lewton`, `ogg`, `hound`), **da dove
  vengono i suoni** e **chi possiede gli asset audio** (§ 13).

---

## 1. Stato attuale verificato

| Cosa | Stato |
|---|---|
| Feature Bevy | `default-features = false`, `["2d", "ui"]`: **niente audio compilato** (D-004: "Audio aggiunto quando serve (P2.4)") |
| `bevy_audio` 0.20.0, `rodio` 0.22.2, `cpal` 0.17.3 | già nel `Cargo.lock` (dipendenze di `bevy_dev_tools`) e nel registro locale, ma non compilati (`cargo tree -i bevy_audio` vuoto) |
| Decoder dei formati (`lewton` + `ogg` per Vorbis, `hound` per WAV) | **assenti** dal lockfile e dal registro: vanno scaricati |
| Asset audio | nessuno; `assets/` ha `fonts`, `sprites`, `tiles`, `ui` |
| Strumenti locali | niente `ffmpeg`, `oggenc`, `sox`; Python 3.14 senza `numpy`/`soundfile` (c'è il modulo `wave` della libreria standard) |
| CI Linux | installa già `libasound2-dev` (`.github/workflows/ci.yml:38`), che serve a `cpal` su Linux |
| Pagina web | il `.wasm` si scarica e parte solo dopo il clic (`web/index.html:94-102`); `docs/RELEASE.md` lo prevede già per l'audio |
| Profilo | `ProfileData` con campi `#[serde(default)]` (`fullscreen`, `still_home`), salvato su file o nel `localStorage` |
| Opzioni | `presentation::options_menu`: Schermo intero, Animazioni |
| Lavoro in corso nel checkout | MAP-BIOMES ha modifiche non committate in `src/app.rs`, `src/lib.rs`, `src/options.rs`, `src/data/`: l'implementazione dell'audio parte **dopo** che quella tranche è integrata |

---

## 2. API di Bevy 0.20 verificate e vincoli che ne derivano

Tutto letto nei sorgenti della release bloccata (`~/.cargo/registry/src/*/bevy_audio-0.20.0`,
`cpal-0.17.3`, `bevy_ecs-0.20.0`, `bevy_asset-0.20.0`, `bevy_winit-0.20.0`).

| API | Dove | Conseguenza per il progetto |
|---|---|---|
| Feature `audio = ["bevy_audio", "vorbis"]`, `wav`, `symphonia-*` | `bevy-0.20.0/Cargo.toml:2765` | si abilitano `audio` + `wav` |
| Il loader `AudioLoader` si registra solo con `mp3`, `flac`, `wav` o `vorbis` | `bevy_audio/src/lib.rs:97` | `symphonia-vorbis` da solo **non** registra il loader: va usato `vorbis` |
| `AudioPlugin` entra in `DefaultPlugins` con la feature | `bevy_internal/src/default_plugins.rs:80` | il client lo riceve da solo; `headless_app` usa `MinimalPlugins` e resta muto |
| `AudioPlayer<Source>(Handle<Source>)` con `#[require(PlaybackSettings)]` | `audio.rs:251` | una voce è un'entità |
| `PlaybackSettings { mode, volume, speed, paused, muted, spatial, ... }`, `PlaybackMode::{Once, Loop, Despawn, Remove}` | `audio.rs:11`, `audio.rs:35` | one-shot con `Despawn`, musica e ambienti con `Loop`; `speed` cambia altezza e durata (variazione di pitch) |
| `AudioSinkPlayback`: `set_volume(&mut self)`, `pause`, `play`, `stop`, `empty`, `position`, `try_seek` | `sinks.rs:10` | dissolvenze fatte da noi, volume per voce a ogni frame |
| `GlobalVolume` moltiplicato **solo alla creazione** del sink | `audio_output.rs:193`, `:263` | i volumi utente non possono passare da `GlobalVolume`: li applichiamo noi a ogni voce viva |
| `Volume::{Linear, Decibels}`, `fade_towards`, operatori `*` e `/` | `volume.rs:36`, `:240` | conversione dB ↔ lineare già pronta |
| L'uscita si apre in `AudioOutput::default()` con `open_default_sink()`; senza dispositivo solo un `warn!` | `audio_output.rs:22` | nessun panic senza scheda audio, ma… |
| `audio_output_available` è `pub(crate)` | `audio_output.rs:326` | … non sappiamo dall'esterno se c'è un dispositivo: le voci senza sink vanno ripulite da noi |
| `play_queued_audio_system` crea il sink quando l'asset è caricato | `audio_output.rs:84` | un suono chiesto prima del caricamento partirebbe **in ritardo**: lo scartiamo invece |
| Decoder di `AudioSource` con `.build().unwrap()` | `audio_source.rs:101` | un file corrotto o di un formato non abilitato **manda in panic** il gioco alla riproduzione: ogni file va decodificato in un test |
| Web: `cpal` crea l'`AudioContext` alla costruzione dello stream e chiama `resume()` in `play()` | `cpal/src/host/webaudio/mod.rs:239`, `:430` | il contesto nasce durante `init()`, cioè dopo il clic della pagina |
| Web: buffer da 2048 frame, due "worker" con `AudioBufferSourceNode` sul thread principale | `webaudio/mod.rs:59` | ~46 ms di audio per buffer: fotogrammi molto lunghi e molti decoder Vorbis sul thread principale possono gracchiare |
| Osservatori `On<Add<T>>`, `On<Insert<T>>`, `On<Remove<T>>` | `bevy_ecs/src/lifecycle.rs:352-437` | ascolto senza toccare il gameplay (lo stesso schema di `leave_remains` in `enemy_anim.rs`) |
| `AssetServer::is_loaded_with_dependencies` | `bevy_asset/src/server/mod.rs:1163` | controllo "caricato?" prima di avviare una voce |
| `WinitSettings::game()`: fuori fuoco `reactive_low_power(1/60 s)` | `bevy_winit/src/winit_config.rs:18` | sul desktop l'app continua a girare senza fuoco: le dissolvenze funzionano |

---

## 3. Architettura

### 3.1 Principi

1. **Solo client.** `GameAudioPlugin` si aggiunge in `client_app`, mai in `headless_app`. I
   moduli `app`, `gameplay`, `world`, `data` e `profile::unlocks` non importano `crate::audio` né
   `bevy::audio` (lo verifica un test, § 12).
2. **Sola lettura.** L'audio legge componenti, risorse, messaggi e stati. Non inserisce
   componenti sulle entità di gioco, non modifica risorse della simulazione, non consuma
   messaggi che altri leggono (ogni `MessageReader` ha il suo cursore) e non usa i flussi di
   `Pcg32` della simulazione.
3. **Eventi, non fotogrammi.** Un suono nasce da un fatto di gioco: un proiettile creato, un
   nemico sconfitto, un livello nuovo. Gli osservatori scattano nei passi fissi a 60 Hz, quando
   si applicano i comandi. I contatori si confrontano con il valore precedente: la differenza è
   il numero di eventi, non di fotogrammi. Nessun suono dipende da `Update` che gira a vuoto.
4. **Politica pura.** Limiti, priorità, cooldown e varianti sono funzioni senza Bevy audio,
   con un orologio passato come argomento. Si testano a 30, 60 e 144 FPS simulati.
5. **Un solo punto di riproduzione.** Solo `src/audio/playback.rs` crea `AudioPlayer`. Il resto
   del gioco, interfaccia compresa, chiede suoni con `AudioManager` (un `SystemParam`).
6. **Mai un panic per l'audio.** File mancante, non caricato, dispositivo assente, contesto del
   browser sospeso: il suono si salta e resta un contatore diagnostico.

### 3.2 Flusso di un fotogramma

```text
FixedUpdate (60 Hz, simulazione invariata)
  comandi applicati ──► osservatori audio (On<Add<Projectile>>, On<Remove<Enemy>>, ...)
                              │ push
                              ▼
                        CueQueue (risorsa del client, solo dati: cue + quantità + posizione)
Update
  ascolto per differenze (RunStats, Progression, Treasury, horde_alert, stati) ──► CueQueue
  UI (menu, draft, opzioni) ──► AudioManager::play(cue) ──► CueQueue
  AudioSet::Decide   mixer: coalescenza per cue → cooldown → limiti per cue → priorità e
                     furto delle voci → scelta della variante e del pitch → VoiceRequests
  AudioSet::Music    direttore musicale: traccia voluta (funzione pura) → deck A/B, dissolvenze
  AudioSet::Ambience loop della home: entrata, uscita, dissolvenze
  AudioSet::Apply    riproduzione: spawn di AudioPlayer + PlaybackSettings, volume di ogni voce
                     viva = generale × bus × cue × variante × dissolvenza × ducking;
                     pulizia delle voci finite o mai partite
PostUpdate
  bevy_audio: crea i sink delle voci nuove (play_queued_audio_system), despawn di quelle finite
```

Il tempo dell'audio è `Time<Real>`: cooldown e dissolvenze non cambiano con la pausa, con
`--time-scale` o con il tempo virtuale fermo nel draft. Con `--time-scale 4` gli eventi sono
quattro volte più fitti e i limiti li assorbono.

### 3.3 Moduli (`src/audio/`, ciascuno sotto ~400 righe)

| File | Contenuto | `bevy_audio` |
|---|---|---|
| `mod.rs` | `GameAudioPlugin`, `AudioSet` (`Listen`, `Decide`, `Music`, `Ambience`, `Apply`), registrazione dei sotto-plugin | no |
| `settings.rs` | `AudioSettings` (volumi 0–10 per bus, muto, audio fuori fuoco), curva tacche → guadagno, lettura e scrittura nel `Profile` | no |
| `catalog.rs` | schema RON del catalogo (`assets/audio/catalog.ron`, incorporato con `include_str!` come `home_ambient.ron`), validazione con errori leggibili, tabelle di lookup precalcolate (indice del compagno → cue) | no |
| `cues.rs` | `CueId` (indice compatto), `CueRequest { cue, count, at: Option<Vec2> }`, `CueQueue`, `AudioManager` | no |
| `listen.rs` | osservatori e sistemi che traducono i fatti di gioco in richieste (§ 6) | no |
| `mixer.rs` | `VoiceTable`, cooldown, limiti, priorità, furto, coalescenza, `VariantPicker` con il suo `Pcg32` | no |
| `music.rs` | `MusicContext` → `MusicTrack` (funzione pura), `MusicDirector` con due deck, dissolvenze equal-power, ducking | solo per il volume dei deck |
| `ambience.rs` | loop della home, uno per voce del catalogo | solo per il volume |
| `playback.rs` | spawn delle voci, volume per voce, `VoiceReaper`, rilevamento "nessun dispositivo" | sì |
| `tests.rs` | test di catalogo, mixer, musica, impostazioni | no |

`cues`, `mixer`, `catalog`, `settings` e la parte pura di `music` girano anche in un'app con
`MinimalPlugins`: è così che i test di integrazione li usano accanto alla simulazione.

### 3.4 Bus e catena del volume

```text
guadagno della voce = generale × bus(utente) × trim del bus (catalogo) × guadagno del cue
                      × guadagno della variante × boost di coalescenza × dissolvenza × ducking
```

- I `cues` del catalogo passano dal mixer delle voci e sono solo `Sfx` o `Ui`. Musica, stinger e
  ambienti hanno le loro sezioni del catalogo e non occupano i posti degli effetti (D-069).
- Bus: `Music`, `Sfx`, `Ambience`. Il generale li moltiplica tutti. I suoni dell'interfaccia
  hanno un trim proprio nel catalogo ma seguono il cursore "Effetti" (quattro cursori in tutto,
  come chiede il requisito 1).
- Valori utente da 0 a 10 tacche; guadagno = (tacche / 10)². Lo 0 è silenzio vero (il sink
  viene messo in pausa, non solo azzerato), il 10 è 0 dB. Predefiniti: generale 8, musica 7,
  effetti 8, ambiente 7.
- `GlobalVolume` resta a 1. Un cambio delle impostazioni ricalcola il volume di tutte le voci
  vive nello stesso fotogramma (`Changed<AudioSettings>`); il volume di ogni sink si riscrive
  solo se cambia di più di 0,001.
- Margine: la somma dei guadagni delle voci effetto vive ha un tetto (`sfx_headroom`, es. 3,0
  lineare, trim compreso). Una voce nuova non `Critical` che lo supererebbe perde prima il
  boost di coalescenza. Se è ancora oltre, ruba voci di priorità strettamente minore, a parità
  la più vecchia, finché ci sta; altrimenti si scarta. Le `Critical` partono sempre e il loro
  guadagno conta nella somma: il tetto si supera solo per loro (D-069, dopo la review di
  AUDIO-1). `bevy_audio` non ha un limitatore sul bus: è la nostra protezione contro la
  saturazione nelle orde.

### 3.5 Voci: limiti, cooldown, priorità

Valori di partenza, tutti nel catalogo:

| Parametro | Nativo | Web | Significato |
|---|---|---|---|
| `max_voices` | 24 | 16 | voci effetto vive insieme (musica e ambienti a parte: 2 deck + fino a 2 in pausa + 2 loop) |
| `reserved_critical` | 3 | 3 | posti tenuti per la priorità `Critical`: le non `Critical` ne hanno al massimo `max − reserved_critical`, le `Critical` occupano prima la riserva |
| `max_starts_per_frame` | 6 | 4 | voci nuove non `Critical` per fotogramma; le altre del fotogramma si scartano per priorità. Le `Critical` non contano |

Per ogni cue:

| Campo | Esempio | Regola |
|---|---|---|
| `priority` | `Low`, `Normal`, `High`, `Critical` | a posti finiti una richiesta ruba la voce di priorità più bassa (a parità la più vecchia); se nessuna è più bassa, si scarta. Per i posti un avvio ruba al massimo una voce |
| `cooldown_ms` | 45 | intervallo **medio** tra due partenze dello stesso cue, in tempo reale: `next = max(next + cd, now + cd − durata del fotogramma)`. Il ritardo dovuto al fotogramma non si accumula, quindi gli avvii al secondo non dipendono dal frame rate. L'intervallo minimo è il cooldown meno un fotogramma, mai meno di un fotogramma. Sotto un fotogramma a 30 FPS (< 34 ms) il limite vero è un avvio per fotogramma (D-069) |
| `max_instances` | 4 | istanze dello stesso cue vive insieme |
| `on_limit` | `Drop` o `Restart` | a istanze finite: scarta, oppure ferma la più vecchia e riparte |
| `coalesce` | `Boost(max_db: 3.0)` o `Single` | le richieste dello stesso cue nello stesso fotogramma diventano **una** voce; con `Boost` il guadagno sale di 1,5 dB per raddoppio della quantità, fino a `max_db` |
| `cull` | `View` o `Always` | con `View` le richieste con una posizione fuori dalla vista grande (640×360, `world::view`) non suonano |

Classi di priorità proposte:

- `Critical`: player colpito, player a terra, preavviso del boss, arrivo del boss, orda in
  arrivo, livello nuovo, stinger di vittoria e sconfitta.
- `High`: sconfitta dell'élite e del boss, forziere aperto, compagno che si unisce, interfaccia.
- `Normal`: attacchi dei compagni (sparo, onda, fulmine, foglie), effetti dei talenti.
- `Low`: colpo su un nemico, sconfitta dei nemici comuni, gemme e monete raccolte.

Una voce si toglie dalla `VoiceTable` quando il sink è vuoto, oppure dopo la sua durata più
0,5 s se il sink non è mai nato (asset non pronto, dispositivo assente). Se per 2 s nessuna
voce riceve un sink mentre ce ne sono di pronte, il backend si considera senza uscita: un solo
avviso nel log, nessuna voce nuova finché non arriva un sink.

### 3.6 Varianti e pitch

- Ogni cue ha 1–5 file. `VariantPicker` ne sceglie uno a caso **escludendo l'ultimo
  suonato** (con un solo file: sempre quello).
- Variazione di altezza `pitch: (0.95, 1.05)` tramite `PlaybackSettings::speed` (cambia anche la
  durata, va bene per effetti brevi). Variazione di guadagno `gain_jitter_db: 1.5`.
- Scala per le raccolte: `pickup.xp` sale di un semitono per raccolta ravvicinata, fino a +7,
  e torna giù dopo 0,6 s senza raccolte (l'effetto "catena" dei survivor), sempre entro i
  cooldown.
- Il caso viene da un `rng::Pcg32` dell'audio con una seed fissa: non tocca la seed della run e
  i test sono ripetibili.

### 3.7 Musica

**Traccia voluta** (funzione pura, testata con tutte le combinazioni):

```text
AppState::Loading                      → nessun cambio (il restart con F5 non interrompe la musica)
AppState::MainMenu (ogni MenuScreen)   → Home
AppState::InRun:
  RunPhase::Victory                    → stinger "victory", poi Results (o silenzio)
  RunPhase::Defeated                   → stinger "defeat" dopo la caduta, poi Results (o silenzio)
  esiste un'entità Boss                → Boss
  orda annunciata o in corso,
    o finita da meno di horde_hold_s   → Horde
  altrimenti                           → Run(stage)   (traccia per stage con ripiego: pronto per MAP-BIOMES)
```

- `Paused` e `LevelUp` non cambiano traccia: abbassano la musica (ducking: −9 dB in pausa, −6 dB
  nel draft, attacco 0,25 s, rilascio 0,5 s).
- `horde_hold_s` (es. 30 s) è un'isteresi: la musica dell'orda non va avanti e indietro mentre
  l'orda si svuota.
- **Due deck.** La traccia nuova parte a volume zero sul deck libero; quando il suo sink esiste,
  parte la dissolvenza equal-power (in = sen(t·π/2), out = cos(t·π/2)). Durate nel catalogo:
  menu → run 1,5 s, run → orda 2,0 s (l'orda è annunciata 4 s prima, D-056), → boss 1,5 s,
  → risultati 0,6 s, ritorno al menu 1,5 s.
- La traccia uscente si mette in **pausa** (non si ferma) se è Run: tornando dall'orda la run
  riprende da dove era. Al massimo due deck in pausa, poi il più vecchio si ferma.
- Gli stinger sono one-shot sul bus della musica, con la musica abbassata sotto.
- Tracce caricate a richiesta: Home all'avvio, Run quando si apre la selezione, Orda e Boss
  all'inizio della run. Su un caricamento lento la dissolvenza aspetta il sink, non salta.

### 3.8 Ambienti della home (portale e cascate)

- Due loop sul bus dell'ambiente: `home_portal` (ronzio e vortice) e `home_falls` (cascate e
  fiume), già mixati in stereo con la posizione nell'illustrazione, così non serve l'audio
  spaziale.
- `OnEnter(AppState::MainMenu)`: dissolvenza in entrata 2 s; `OnExit`: uscita 1 s, poi
  despawn. Attivi in tutte le schermate del menu (home, selezione, opzioni).
- Facoltativo nel catalogo: un one-shot `home_portal_pulse` ogni 6–12 s a caso, a volume basso.
- Indipendenti dall'interruttore "Animazioni" (che riguarda la grafica): li regola il cursore
  "Ambiente". Domanda aperta § 13.
- Gli stessi meccanismi serviranno per gli ambienti degli stage (vento della prateria, bosco)
  quando MAP-BIOMES li chiederà.

### 3.9 Caricamento e memoria

- All'avvio `AudioLibrary` carica gli handle di tutti gli effetti (sono piccoli). Musica e
  ambienti si caricano a richiesta (§ 3.7).
- Prima di avviare una voce: `is_loaded_with_dependencies`. Se il file non è pronto il suono
  **si scarta** (un colpo sentito un secondo dopo è peggio di nessun suono) e si conta in
  `AudioStats::not_ready`.
- `AudioSource` tiene i byte compressi e decodifica a ogni riproduzione
  (`audio_source.rs:101`): gli effetti in WAV PCM costano solo la copia, la musica in Vorbis si
  decodifica in streaming.

### 3.10 Impostazioni e persistenza

- `ProfileData.audio: AudioPrefs { master: 8, music: 7, sfx: 8, ambience: 7, muted: false,
  play_unfocused: false }`, con `#[serde(default)]` sul campo e su ogni sotto-campo: i profili
  salvati prima si leggono (test come `profiles_saved_before_the_option_still_load`).
- Sul desktop va nel file del profilo, sul web nel `localStorage`, con lo stesso testo RON. Le
  run automatiche (`--autoplay`, `--god`, `--time-scale`) leggono le impostazioni ma non le
  salvano, come oggi.
- Opzioni: righe "Volume generale", "Musica", "Effetti", "Ambiente" con una barra a 10 tacche
  fatta di nodi UI (nessun glifo nuovo, D-044), sinistra e destra per cambiare, clic sulla
  barra; "Audio fuori dalla finestra: Sì/No". Cambiando "Effetti" suona `ui.tick` come
  anteprima. Salvataggio alla chiusura della schermata, non a ogni tacca.
- CLI e indirizzo: `--mute` (`?mute` sul web) per le prove automatiche, senza salvare;
  `--no-audio` non aggiunge l'`AudioPlugin` di Bevy
  (`DefaultPlugins.build().disable::<AudioPlugin>()`); i sistemi di riproduzione girano allora
  solo se esiste `Assets<AudioSource>`.

### 3.11 Windows e WASM

**Windows (WASAPI tramite `cpal`).** Nessuna dipendenza di sistema. Latenza tipica di qualche
decina di ms. Fuori fuoco l'app continua a 60 Hz (`WinitSettings::game`): con
`play_unfocused = false` il generale scende a zero in 0,25 s e i sink si mettono in pausa; al
ritorno del fuoco ripartono. Limite noto: se cambia il dispositivo predefinito (cuffie
staccate), Bevy 0.20 non riapre l'uscita (`AudioOutput` nasce una volta sola). Si documenta, non
si aggira in questa fase.

**Web.**

1. *Politica di autoplay.* I browser lasciano partire un `AudioContext` solo dopo
   un'interazione dell'utente con la pagina. Nella nostra pagina `init()` parte dentro il gestore
   del clic su "Clicca per giocare", quindi il contesto di `cpal` nasce dopo il clic e Chrome,
   Edge e Firefox dovrebbero farlo partire. Va **verificato** nel browser (AUDIO-2).
2. *Protezione nella pagina* (`web/index.html`, web-release), prima dell'`import` del gioco: un
   involucro sul costruttore `AudioContext` tiene l'elenco dei contesti creati. Al primo
   `pointerdown` o `keydown` chiama `resume()` sui contesti sospesi. Su `visibilitychange`
   chiama `suspend()` quando la scheda è nascosta e `resume()` quando torna visibile. Lo stato
   va nella console per le prove. È uno schema noto per l'autoplay di Web Audio; nessun
   cambiamento in Rust.
3. *Scheda nascosta.* `requestAnimationFrame` si ferma e Bevy non aggiorna più. I callback
   `onended` di `cpal`, invece, continuano a chiedere campioni al mixer: senza la protezione la
   musica continuerebbe da sola. La sospensione dalla pagina lo evita.
4. *Thread principale.* Sul web la decodifica e lo scheduling dell'audio girano sul thread del
   gioco (§ 2). Per questo: effetti in WAV (niente decodifica), al massimo 3 flussi Vorbis
   insieme (2 deck + 1 ambiente; il secondo deck solo durante una dissolvenza), 16 voci, e una
   misura dei fotogrammi in un'orda (AUDIO-5).
5. *Dimensioni.* Budget proposto ≤ 8 MiB di audio in `dist/` (musica ≈ 1–1,3 MB per 90 s di
   Vorbis stereo a ~96–112 kbps; effetti WAV mono 16 bit ≈ 0,6–1 MB in tutto). Ogni file è
   molto sotto il limite di 25 MiB. Le cartelle con la versione (`assets-<v>`, D-062) valgono
   anche per l'audio: `build-web.*` copia già tutto `assets/`.
6. *Safari* non è un obiettivo della P1. Se il contesto resta sospeso, lo riprende il primo
   tasto (punto 2). Si documenta in `docs/RELEASE.md`.

### 3.12 Separazione dalla simulazione headless

- `headless_app` non cambia: nessun `AudioPlugin`, nessun `GameAudioPlugin`.
- Gli osservatori audio sono registrati solo da `GameAudioPlugin`. Nel client scattano negli
  stessi passi della simulazione, ma fanno solo un `push` su una risorsa del client.
- Test di garanzia (§ 12): headless senza risorse audio, simulazione identica con e senza
  l'ascolto audio, nessun import dell'audio nei moduli della simulazione.

### 3.13 Licenze e provenienza

- Ogni file in `assets/audio/` ha una voce in `assets/manifest.json` (`kind: "music"`,
  `"ambience"` o `"sfx"`) con `source`, `author`, `license` (SPDX o `original`), `url` se
  esterno, `attribution` se richiesta, `approved` (data e chi). Una riga corrispondente va in una
  sezione "Audio" di `ASSET_LICENSES.md`.
- Licenze ammesse senza chiedere: `original` (fatto nel progetto), `user-owned` (dell'utente,
  come le immagini di D-050/D-052), `CC0-1.0`. Con `CC-BY-4.0` serve l'attribuzione nei
  documenti e, prima della release, in una schermata dei crediti. Escluse: `-NC`, `-ND`,
  licenze "royalty free" senza testo chiaro, estratti da altri giochi, suoni che imitano jingle
  riconoscibili (Pokémon, Nintendo e simili), musica da servizi IA senza diritti chiari (solo con
  il tuo via).
- I sorgenti riproducibili stanno in `assets-src/audio/`: patch degli effetti sintetizzati,
  progetti, note di montaggio e `provenance.ron` per gli esterni (URL, data del download, testo
  della licenza salvato accanto).
- Il validatore rifiuta: file senza voce nel manifest, licenza fuori elenco, formato non
  abilitato, nomi fuori dallo schema `^[a-z0-9_]+\.(wav|ogg)$`.

---

## 4. Struttura delle cartelle

```text
src/audio/                     codice (§ 3.3)
assets/audio/
  catalog.ron                  catalogo: cue, varianti, bus, priorità, cooldown, tracce, dissolvenze
  music/                       home.ogg, run_meadow.ogg, horde.ogg, boss.ogg, results.ogg,
                               stinger_victory.ogg, stinger_defeat.ogg, stinger_boss.ogg
  ambience/                    home_portal.ogg, home_falls.ogg
  sfx/
    companions/                ember_shot_1.wav … spark_chain_3.wav
    enemies/                   hit_1.wav … boss_slam.wav
    player/                    hurt_1.wav, down.wav
    pickups/                   xp_1.wav, coin_1.wav, chest_open.wav
    run/                       level_up.wav, horde_warn.wav, companion_join.wav
    talents/                   blast_1.wav, …
    ui/                        move.wav, confirm.wav, back.wav, refused.wav, tick.wav, …
assets-src/audio/
  sfx/*.sfx.ron                patch testuali degli effetti sintetizzati (se approvato, § 13)
  music/, ambience/            progetti e note di montaggio dei loop (punti di loop, tonalità, BPM)
  provenance.ron               provenienza dei file esterni o dell'utente
  licenses/                    testo delle licenze esterne
docs/audio/
  AUDIO_BIBLE.md               stile sonoro, loudness, formati, regole dei loop
  requests/<ID>.md             richieste per l'handoff manuale (se approvato, § 13)
scripts/
  audio_sfx.py                 sintesi degli effetti dalle patch (solo libreria standard: wave, math)
  validate-assets.py           + controlli audio (header WAV e Ogg, durate, canali, frequenza)
tests/sim/audio.rs             garanzie di separazione e stress delle orde
.audio-inbox/                  ignorata da git: file consegnati dall'utente in attesa di controllo
```

---

## 5. Catalogo iniziale degli audio

Nomi provvisori. Durate, numero di varianti e priorità sono la proposta di partenza. I nomi dei
compagni nei dati: `ember` (Fiammetto), `tide` (Ondino), `sprout` (Foglin), `spark` (Tuonny);
nemici `blob`, `swarm`, `armored`, `elite_blob`, `boss_king`.

### 5.1 Musica (bus Music, Vorbis stereo 44,1 kHz, loop senza stacco)

| ID | Uso | Durata del loop | Carattere |
|---|---|---|---|
| `home` | pagina iniziale, selezione, opzioni, talenti | 60–90 s | calma, misteriosa, legata al portale |
| `run_meadow` | run nella prateria (ripiego per ogni stage) | 90–120 s | avventura, ritmo medio |
| `run_forest` | Bosco Ombroso (MAP-FOREST), facoltativa | 90–120 s | stessa tonalità, più scura |
| `horde` | orde a 5:00 e 10:00 | 45–60 s | stessa tonalità della run, più percussioni |
| `boss` | boss in campo | 60–90 s | tensione, basso marcato |
| `results` | schermata dei risultati, facoltativa | 30–45 s | breve e tranquilla |
| `stinger_boss` | arrivo del boss | 2–3 s | one-shot |
| `stinger_victory` | vittoria | 3–5 s | one-shot originale, nessun jingle riconoscibile |
| `stinger_defeat` | sconfitta | 2–4 s | one-shot |

### 5.2 Ambienti della home (bus Ambience, Vorbis stereo)

| ID | Contenuto | Durata del loop |
|---|---|---|
| `home_portal` | ronzio basso del portale, vortice | 20–40 s |
| `home_falls` | cascate e fiume, più forte a sinistra o a destra come nell'illustrazione | 20–40 s |
| `home_portal_pulse` | facoltativo: impulso delle rune, one-shot raro | 1–2 s |

### 5.3 Effetti di gioco (bus Sfx, WAV mono 16 bit 44,1 kHz, ≤ 1,5 s)

| Cue | Varianti | Priorità | Cooldown | Istanze | Coalescenza |
|---|---|---|---|---|---|
| `shot.ember` (palla di fuoco) | 3 | Normal | 60 ms | 3 | Boost 3 dB (raffiche) |
| `pulse.tide` (onda d'acqua) | 2 | Normal | 120 ms | 2 | Single |
| `orbit.sprout` (foglie create o ricreate) | 2 | Normal | 250 ms | 1 | Single |
| `chain.spark` (fulmine a catena) | 3 | Normal | 80 ms | 2 | Single |
| `talent.blast` (esplosione dei talenti) | 2 | Normal | 80 ms | 3 | Boost 2 dB |
| `enemy.hit` (colpo generico) | 4 | Low | 35 ms | 4 | Boost 3 dB |
| `enemy.die.small` (`blob`, `swarm`) | 4 | Low | 40 ms | 4 | Boost 3 dB |
| `enemy.die.armored` | 3 | Low | 60 ms | 3 | Boost 2 dB |
| `elite.warn` (linea della carica) | 1 | Critical | 300 ms | 2 | Single |
| `elite.die` | 2 | High | 0 | 2 | Single |
| `boss.warn` (cerchio del colpo) | 1 | Critical | 300 ms | 1 | Single |
| `boss.slam` | 2 | High | 200 ms | 1 | Single |
| `boss.die` | 1 | Critical | 0 | 1 | Single |
| `player.hurt` | 3 | Critical | 150 ms | 1 | Single |
| `player.down` | 1 | Critical | 0 | 1 | Single |
| `pickup.xp` (scala di altezza) | 2 | Low | 35 ms | 3 | Single |
| `pickup.coin` | 2 | Low | 50 ms | 2 | Single |
| `chest.open` | 1 | High | 0 | 1 | Single |
| `level.up` | 1 | Critical | 0 | 1 | Single |
| `horde.warn` ("ARRIVA L'ORDA!") | 1 | Critical | 0 | 1 | Single |
| `companion.join` | 1 | High | 0 | 1 | Single |

### 5.4 Interfaccia (bus Sfx con il trim `ui`)

`ui.move` (voce evidenziata), `ui.confirm`, `ui.back`, `ui.refused` (avvio rifiutato, voce
bloccata), `ui.tick` (anteprima del volume), `ui.toggle`, `ui.pause_open`, `ui.pause_close`,
`draft.open`, `draft.hover`, `draft.pick`, `relic.pick`, `talent.learn`, `talent.reset`.
Tutti `High`, 1 variante, cooldown 40 ms, 2 istanze.

Totale di partenza: 8 musiche e stinger, 2–3 ambienti, ~50 file di effetti contando le
varianti.

---

## 6. Eventi da intercettare

Nessuna riga richiede di cambiare `src/gameplay/`. Dove si legge "osservatore", lo schema è
quello già usato dalla presentazione (`leave_remains`, `attach_*`).

| Cue | Fatto di gioco | Aggancio (sola lettura) | Note |
|---|---|---|---|
| `shot.<companion>` | proiettile creato | `On<Add<Projectile>>` + `Source(i)` → id del compagno | i proiettili di una raffica nello stesso passo diventano una voce |
| `pulse.tide` | onda partita | `On<Add<Pulse>>` + `Source` | |
| `chain.spark` | fulmine | `On<Add<ChainBolt>>` | |
| `orbit.sprout` | foglie create | `On<Add<Leaf>>` | molte foglie nello stesso passo = una voce |
| `talent.blast` | esplosione di un talento | `On<Add<Blast>>` | |
| `enemy.hit` | colpi andati a segno | differenza di `RunStats::hits` nel fotogramma | un osservatore su ogni `HitFlash` costerebbe migliaia di chiamate al secondo nelle orde; il contatore basta |
| `enemy.die.*`, `elite.die`, `boss.die` | nemico sconfitto | `On<Remove<Enemy>>` con `Health::is_dead()` e `AppState::InRun`; tipo da `DataId`; `Boss` presente | come `leave_remains`: chi sparisce per restart o fine run non suona; posizione per il `cull` |
| `elite.warn` | preavviso della carica | `On<Add<Telegraph>>` con `TelegraphShape::Line` | |
| `boss.warn` | preavviso del colpo ad area | `On<Add<Telegraph>>` con `TelegraphShape::Ring` | |
| `boss.slam` | colpo ad area eseguito | `On<Remove<Telegraph>>` con `remaining <= 0` e `PendingSlam` sulla stessa entità | se chi l'ha lanciato muore prima il segno sparisce con `remaining > 0`: niente suono |
| `stinger_boss` + musica Boss | boss in campo | `On<Add<Boss>>` | |
| `player.hurt` | player colpito | `On<Insert<HitFlash>>` con `With<Player>` | priorità massima |
| `player.down`, `stinger_defeat` | sconfitta | `OnEnter(RunPhase::Defeated)` | |
| `stinger_victory` | vittoria | `OnEnter(RunPhase::Victory)` | |
| `pickup.xp` | gemme raccolte | differenza di `Progression::total_xp` | la fusione delle gemme non cambia l'XP: non suona |
| `level.up` | livello nuovo | aumento di `Progression::level` | il draft si apre subito dopo (`draft.open`) |
| `pickup.coin` | monete raccolte | differenza di `Treasury::collected` | |
| `chest.open` | forziere aperto | `MessageReader<ChestOpened>` | |
| `horde.warn` | orda annunciata | `horde_alert(plan, RunClock::elapsed)` che diventa `Coming` | la stessa funzione pura che usa il banner |
| `companion.join` | compagno nuovo a metà run | `On<Add<Companion>>` con `RunClock::elapsed > 0` | il compagno iniziale non suona |
| `draft.pick`, `relic.pick` | carta scelta | `MessageReader<ChooseUpgrade>` | |
| `ui.pause_open` / `close` | pausa | `OnEnter` / `OnExit(RunPhase::Paused)` | anche ducking della musica |
| `ui.move` | voce evidenziata | `AudioManager::play` dove `menu_list.rs` cambia `ListSelection` | `ListSelection` è `pub(super)`: l'audio non la interroga, la presentazione chiama il gestore |
| `ui.confirm`, `ui.back`, `ui.refused`, `talent.*`, `ui.toggle` | azioni nei menu | chiamate `AudioManager::play` nei gestori della presentazione | poche righe in `src/presentation/`, assegnate esplicitamente |
| musica e ambienti | stato del gioco | `State<AppState>`, `State<RunPhase>`, `Query<With<Boss>>`, `HordeState`, `horde_alert`, `RunSetup::stage` | funzione pura § 3.7 |

Restart e ricarica a caldo passano da `AppState::Loading`: le voci in corso finiscono da sole,
la musica non cambia, i contatori dell'ascolto (valori precedenti di `RunStats`, `Progression`,
`Treasury`) ripartono da zero in `OnEnter(AppState::InRun)`, così il reset delle risorse non
sembra un evento.

---

## 7. Formato del catalogo (esempio)

`assets/audio/catalog.ron`, incorporato con `include_str!` e validato nei test come
`assets/ui/home_ambient.ron` (D-067). Schema in `src/audio/catalog.rs`, valori del proprietario
degli asset audio. Mappe ordinate (`BTreeMap`), niente `HashMap` (D-047).

```ron
(
    version: 1,
    voices: (max: 24, max_web: 16, reserved_critical: 3, starts_per_frame: 6,
             starts_per_frame_web: 4, sfx_headroom: 3.0),
    trims: (music: 0.8, sfx: 0.8, ambience: 0.7, ui: 0.6),
    music: (
        tracks: {
            "home": (file: "audio/music/home.ogg", gain: 0.8),
            "run_meadow": (file: "audio/music/run_meadow.ogg", gain: 0.75),
            "horde": (file: "audio/music/horde.ogg", gain: 0.8),
            "boss": (file: "audio/music/boss.ogg", gain: 0.85),
        },
        stingers: {
            "boss": (file: "audio/music/stinger_boss.ogg", gain: 0.9),
            "victory": (file: "audio/music/stinger_victory.ogg", gain: 0.9),
            "defeat": (file: "audio/music/stinger_defeat.ogg", gain: 0.9),
        },
        run_by_stage: { "meadow": "run_meadow" },
        run_default: "run_meadow",
        fades: (to_run: 1.5, to_horde: 2.0, to_boss: 1.5, to_results: 0.6, to_home: 1.5),
        horde_hold_s: 30.0,
        duck: (pause_db: -9.0, draft_db: -6.0, attack_s: 0.25, release_s: 0.5),
    ),
    ambience: [
        (id: "home_portal", file: "audio/ambience/home_portal.ogg", gain: 0.6,
         fade_in_s: 2.0, fade_out_s: 1.0),
        (id: "home_falls", file: "audio/ambience/home_falls.ogg", gain: 0.55,
         fade_in_s: 2.0, fade_out_s: 1.0),
    ],
    cues: {
        "shot.ember": (
            bus: Sfx,
            files: ["audio/sfx/companions/ember_shot_1.wav", "audio/sfx/companions/ember_shot_2.wav",
                    "audio/sfx/companions/ember_shot_3.wav"],
            gain: 0.45, gain_jitter_db: 1.5, pitch: (0.95, 1.05),
            priority: Normal, cooldown_ms: 60, max_instances: 3, on_limit: Drop,
            coalesce: Boost(max_db: 3.0), cull: Always,
        ),
        // ...
    },
    // ripieghi: un compagno o un nemico senza cue proprio usa questi (avviso, non errore)
    fallbacks: { "shot": "shot.ember", "enemy.die": "enemy.die.small" },
)
```

La validazione controlla: file esistenti nel manifest, formato giusto per il bus (WAV per gli
effetti, Ogg per musica e ambienti), guadagni in (0, 1], pitch in [0,5; 2], cooldown ≥ 0,
`max_instances` ≥ 1, ogni compagno di `companions.ron` con un cue `shot.*` o il ripiego, ogni
stage con una traccia o il ripiego, nessun cue sconosciuto chiesto dal codice (l'elenco dei cue
del codice è una costante testata contro il catalogo).

---

## 8. Dipendenze e feature (proposta D-069)

```toml
bevy = { version = "0.20.0", default-features = false, features = ["2d", "ui", "audio", "wav"] }
```

| Crate | Versione | Licenza | Stato | Perché |
|---|---|---|---|---|
| `bevy_audio` | 0.20.0 | MIT OR Apache-2.0 | nel lockfile | audio di Bevy |
| `rodio` | 0.22.2 | MIT OR Apache-2.0 | nel lockfile | mixer e decoder |
| `cpal` | 0.17.3 | Apache-2.0 | nel lockfile | uscita: WASAPI su Windows, Web Audio sul web, ALSA su Linux |
| `lewton` | 0.10.x | da confermare sul `Cargo.toml` scaricato (atteso MIT OR Apache-2.0) | **da scaricare** | Vorbis in Rust puro, va anche su WASM |
| `ogg` | 0.8.x (dipendenza di `lewton`) | da confermare (atteso BSD-3-Clause) | **da scaricare** | contenitore Ogg |
| `hound` | 3.5.x | da confermare (atteso Apache-2.0) | **da scaricare** | WAV per gli effetti, senza decodifica |

Perché WAV per gli effetti e Vorbis per musica e ambienti: sul web la decodifica gira sul
thread del gioco e ogni voce ha il suo decoder; il PCM non costa nulla, mentre i brani lunghi in
WAV peserebbero troppo da scaricare. Niente MP3, FLAC o `symphonia`: meno codice.

Costi da misurare in AUDIO-1 e registrare in D-069: dimensione del `.wasm` prima e dopo (stima
+0,5–1,5 MiB), tempo di compilazione, spazio in `target/` (D-027: un solo set di feature per
nativo e web, nessuna feature opzionale "audio" da accendere a parte, che raddoppierebbe la
compilazione di Bevy). Il comando `cargo update` o `cargo fetch` per i tre crate serve la rete:
lo eseguo solo dopo il tuo via.

---

## 9. Ownership proposta

| Owner | Cosa |
|---|---|
| orchestrator | `Cargo.toml` `[dependencies]`, `Cargo.lock`, `src/lib.rs` (`pub mod audio`, plugin in `client_app`, `--no-audio`), D-069/D-070, `AGENTS.md`, `docs/ARCHITECTURE.md`, backlog, progress, `.gitignore` (`.audio-inbox/`) |
| gameplay-engineer | `src/audio/` (tutto il codice); su assegnazione esplicita `src/profile.rs` (`AudioPrefs`), `src/presentation/options_menu.rs` (cursori), le chiamate `AudioManager::play` in `src/presentation/`, `src/options.rs` e `src/web.rs` (`--mute`, `?mute`) |
| proprietario degli asset audio (§ 13, domanda 3) | `assets/audio/` (anche i valori di `catalog.ron`), `assets-src/audio/`, `docs/audio/`, `scripts/audio_*.py`, i controlli audio in `scripts/validate-assets.py`, le voci audio di `assets/manifest.json` e `ASSET_LICENSES.md` |
| web-release | protezione dell'`AudioContext` in `web/index.html`, misura delle dimensioni in `scripts/build-web.*`, `docs/RELEASE.md` (Safari, autoplay) |
| qa-performance | `tests/sim/audio.rs`, righe audio nell'overlay F3 (`src/diagnostics/`: voci vive, scartate, rubate, non pronte, traccia corrente), misura dei fotogrammi in un'orda |
| game-designer | nessun file: niente cambia nel bilanciamento |
| code-reviewer | review di AUDIO-1, AUDIO-2 e AUDIO-5 |

`src/presentation/options_menu.rs`, `src/profile.rs`, `src/options.rs` e `src/lib.rs` sono
toccati anche da MAP-BIOMES, che è in corso: le tranche audio che li toccano partono dopo.

---

## 10. Rischi tecnici

| # | Rischio | Probabilità / impatto | Mitigazione |
|---|---|---|---|
| R1 | Crate nuovi da scaricare (`lewton`, `ogg`, `hound`), licenze da confermare | certa / basso | D-069 con versioni e licenze lette dai `Cargo.toml` scaricati; via dell'utente prima del download |
| R2 | Il browser lascia l'`AudioContext` sospeso | media / alto sul web | contesto creato dopo il clic; protezione nella pagina (§ 3.11); prova in Chrome, Edge, Firefox |
| R3 | Audio sul thread principale nel web: gracchia con fotogrammi lunghi o troppi decoder | media / medio | effetti in WAV, ≤ 3 flussi Vorbis, 16 voci, misura in un'orda; se serve, meno voci sul web |
| R4 | Musica che continua nella scheda nascosta | alta senza protezione / medio | `suspend()` su `visibilitychange` |
| R5 | File corrotto o di un formato non abilitato → panic (`audio_source.rs:101`) | bassa / alto | test che decodifica ogni file del catalogo; il validatore ammette solo `.wav` e `.ogg` |
| R6 | Nessun dispositivo audio (CI, desktop remoto): voci senza sink che si accumulano | media / basso | `VoiceReaper` e backend "senza uscita" (§ 3.5); `--no-audio` |
| R7 | Saturazione nelle orde (300+ nemici, molte morti al secondo) | alta senza limiti / alto | coalescenza, cooldown, istanze, `max_voices`, `sfx_headroom`; test di stress (§ 12) |
| R8 | Stacco al punto di loop in Vorbis | media / medio | loop preparati alla lunghezza esatta, controllo di continuità tra fine e inizio nel test di decodifica, ascolto; in caso, il loop d'ambiente in WAV |
| R9 | Suoni in ritardo se il file non è ancora caricato | media / basso | `is_loaded_with_dependencies`, altrimenti si scarta |
| R10 | Osservatori su percorsi caldi | bassa / medio | i colpi dal contatore `RunStats::hits`; osservatori solo su eventi rari o per passo; misura con la sonda di costo per passo |
| R11 | L'audio cambia la simulazione (RNG, comandi, ordine dei sistemi) | bassa / alto | sola lettura, `Pcg32` proprio, test di trasparenza (§ 12) |
| R12 | Cleanup scambiato per eventi (despawn al restart, reset delle risorse) | media / basso | filtro `Health::is_dead()` e `InRun`, contatori azzerati in `OnEnter(InRun)` |
| R13 | Cambio del dispositivo su Windows: Bevy 0.20 non riapre l'uscita | media / basso | limite documentato; riavviare il gioco |
| R14 | `GlobalVolume` vale solo alla creazione del sink | certa / medio | volumi gestiti per voce (§ 3.4) |
| R15 | Licenze: obblighi di attribuzione, termini dei servizi IA, suoni simili a giochi famosi | media / alto | elenco delle licenze ammesse, validatore, `provenance.ron`, approvazione per ogni file esterno |
| R16 | Conflitti con MAP-BIOMES in corso | alta / medio | tranche audio dopo l'integrazione di MAP-BIOMES; AUDIO-1 non tocca file condivisi tranne `lib.rs` |
| R17 | Peso del download sul web | bassa / medio | budget ≤ 8 MiB, musica caricata a richiesta |

---

## 11. Piano incrementale

Ogni tranche finisce con `./scripts/check.ps1` verde, clippy per `wasm32-unknown-unknown`,
report in `docs/PROGRESS.md` e commit locale.

| ID | Contenuto | Owner | Criteri di done |
|---|---|---|---|
| AUDIO-0 | Questo documento; via dell'utente; D-069 (dipendenze), D-070 (origine degli asset, ownership); righe del backlog; `AGENTS.md` | orchestrator | decisioni registrate; nessun codice |
| AUDIO-1 | Fondamenta senza suoni: feature `audio` + `wav`, `src/audio/` con impostazioni, catalogo (vuoto o con file mancanti saltati), coda dei cue, mixer puro, `GameAudioPlugin` solo nel client, `--mute`, `--no-audio` | orchestrator (dipendenze, `lib.rs`) + gameplay-engineer + qa-performance (test di separazione) | test del mixer (limiti, cooldown, priorità, furto, coalescenza, varianti senza ripetizione, stessa sequenza a 30/60/144 FPS); test di separazione (§ 12); dimensione del `.wasm` prima e dopo in D-069; nessun cambiamento nei test e nelle sonde esistenti |
| AUDIO-2 | Riproduzione e impostazioni: `playback.rs`, volumi per voce, ripulitura, assenza del dispositivo, `AudioPrefs` nel profilo, cursori nelle Opzioni, fuoco della finestra; protezione della pagina web | gameplay-engineer + web-release + code-reviewer | profilo vecchio leggibile; un cambio di volume cambia le voci vive; nessun panic senza dispositivo (`--no-audio` e scheda disattivata); nel browser l'audio parte dopo il clic e si ferma con la scheda nascosta (log della console); 1–2 effetti di prova dal percorso scelto in § 13 |
| AUDIO-3 | Musica: direttore, deck, dissolvenze, ducking, stinger; tracce provvisorie | gameplay-engineer + proprietario degli asset audio | test della funzione pura con ogni stato; nel gioco vero: home → selezione → run → pausa → orda (con `--time-scale`) → boss → vittoria e sconfitta → menu, senza buchi né sovrapposizioni (traccia corrente e volumi dei deck nel log o nell'overlay) |
| AUDIO-4 | Ambienti della home: portale e cascate | gameplay-engineer + proprietario degli asset audio | entrata e uscita senza scatti, cursore "Ambiente" efficace, nessun loop duplicato entrando e uscendo dal menu più volte (test) |
| AUDIO-5 | Effetti di gioco: ascolto (§ 6) e catalogo degli effetti; taratura dei limiti nelle orde | gameplay-engineer + proprietario degli asset audio + qa-performance + code-reviewer | test di stress: orda a 5:00 con `--autoplay --god --time-scale 4`, voci vive sempre ≤ `max_voices`, partenze al secondo entro i limiti, nessun `Critical` perso; costo per passo della sonda invariato; FPS nativo e web nell'orda annotati |
| AUDIO-6 | Interfaccia: menu, selezione, draft, forzieri, talenti, pausa | gameplay-engineer | ogni azione con il suo suono; `ui.refused` per il compagno bloccato; nessun suono doppio per tasto e clic insieme |
| AUDIO-7 | Asset definitivi e licenze: sostituzione dei provvisori, loudness uniforme, `ASSET_LICENSES.md`, crediti per eventuali CC-BY | proprietario degli asset audio + utente (approvazione) | ogni file con licenza e approvazione; validatore e test di decodifica verdi; picchi ≤ −1 dBFS; loudness dei bus entro ±2 dB dal riferimento della `AUDIO_BIBLE` |
| AUDIO-8 | Prova su Windows e web: Chrome, Edge, Firefox; schermo intero, fuoco, scheda nascosta, restart, ritorno al menu | qa-performance + web-release | checklist in `docs/QA_REPORT.md` con esiti veri; limiti noti in `docs/RELEASE.md` |

AUDIO-1 si può fare subito dopo il via e dopo MAP-BIOMES. AUDIO-3 e AUDIO-4 non dipendono l'uno
dall'altro, ma toccano entrambi `src/audio/`: vanno in sequenza (una sola persona sugli stessi
file).

---

## 12. Criteri di accettazione del sistema

**Separazione e determinismo**
1. `headless_app` non contiene `AudioSettings`, `CueQueue`, `Assets<AudioSource>` né entità con
   `AudioPlayer` dopo 10 s di gioco simulato.
2. Stessa seed, 2 minuti di run headless (7200 passi: il test resta sotto ~10 s) con e senza i
   plugin di ascolto e politica: `RunStats`,
   `Progression`, `Treasury`, impronta delle posizioni e `RunClock` identici.
3. Un test sul testo dei sorgenti: `src/app.rs`, `src/gameplay/`, `src/world/`, `src/data/` non
   contengono `crate::audio` né `bevy::audio`; `AudioPlayer` compare solo in
   `src/audio/playback.rs`.
4. Tutti i test, le sonde di bilanciamento e la sonda di costo per passo esistenti danno gli
   stessi risultati di prima.

**Comportamento**
5. Volumi generale, musica, effetti e ambiente cambiano solo il loro bus, anche sulle voci già in
   corso; a 0 il bus è in silenzio; salvati e ripresi all'avvio sul desktop e sul web; un profilo
   di prima dell'audio si legge.
6. La musica segue home → run → orda → boss → vittoria/sconfitta → home con dissolvenze delle
   durate del catalogo, senza silenzi oltre 0,2 s non voluti e mai due tracce di gioco piene
   insieme; il restart con F5 non la interrompe.
7. Il portale e le cascate suonano in loop in tutte le schermate del menu ed escono in
   dissolvenza all'avvio della run; nessun loop duplicato dopo dieci entrate e uscite.
8. Ogni effetto nasce da un evento del § 6. Con la stessa sequenza di eventi a 30, 60 e 144 FPS
   simulati le voci avviate al secondo restano entro il ±10%, per ogni cooldown del catalogo da
   34 ms in su.
9. Nell'orda a 5:00 (stress con `--time-scale 4`): voci effetto vive ≤ `max_voices`, partenze
   non `Critical` per fotogramma ≤ `starts_per_frame`, somma dei guadagni ≤ `sfx_headroom` (oltre
   solo per le voci `Critical`), tutti i suoni `Critical` partiti.
10. Nessun cue suona due volte di fila con la stessa variante, se ne ha più d'una.

**Piattaforme**
11. Windows: nessun panic con la scheda audio disattivata o con `--no-audio`; fuori fuoco
    l'audio si spegne se l'opzione lo dice.
12. Web (Chrome, Edge, Firefox): l'audio parte dopo "Clicca per giocare", la console non ha
    errori dell'audio, la scheda nascosta sospende il suono e al ritorno riprende; audio in
    `dist/` entro il budget.

**Licenze**
13. Ogni file in `assets/audio/` ha manifest, licenza ammessa, origine e approvazione; il
    validatore fallisce se ne manca uno; il test di decodifica passa su ogni file.

---

## 13. Domande per l'utente (bloccano AUDIO-1 o AUDIO-2)

1. **Crate nuovi.** Posso scaricare da crates.io `lewton`, `ogg` e `hound` (feature `audio` e
   `wav` di Bevy)? Senza, Bevy non carica né WAV né Ogg.
2. **Da dove vengono i suoni.** Nessun asset è stato scaricato o generato. Tre strade, anche da
   combinare:
   - **A. Sintesi locale degli effetti (consigliata per gli effetti).** Patch testuali in
     `assets-src/audio/sfx/` e uno script Python con la sola libreria standard che produce i WAV,
     come i `.px` degli sprite. Suoni originali in stile chip, adatti al pixel art,
     riproducibili, licenza `original`. È "generare asset": serve il tuo via.
   - **B. Handoff manuale (consigliata per musica e ambienti).** Come D-065 per l'arte: preparo
     le richieste in `docs/audio/requests/` (durata, loop, tonalità, BPM, carattere, nessun
     riferimento a giochi esistenti); tu componi, commissioni o scegli i file e li metti in
     `.audio-inbox/<ID>/`; io li controllo e li integro solo con la tua approvazione. Qui non
     c'è un encoder Ogg: i file vanno consegnati già in Ogg Vorbis, oppure mi autorizzi a
     installare `ffmpeg` o `oggenc`.
   - **C. Librerie CC0** (pacchetti di effetti e registrazioni di cascate con licenza CC0, da
     verificare file per file al momento del download): serve il via per ogni download.
3. **Chi possiede gli asset audio.** Allargare il ruolo del pixel-art-director (ha già manifest,
   validatore e `ASSET_LICENSES.md`; nella sua definizione l'audio è fuori scope "fino alla
   P2.4"), oppure un agente nuovo `audio-director` in `.claude/agents/`, che però dovrebbe
   dividere con lui manifest, validatore e `ASSET_LICENSES.md`. Consiglio la prima.
4. **Ambienti della home e interruttore "Animazioni".** Proposta: indipendenti (li regola solo
   il cursore "Ambiente"). In alternativa, spegnendo le animazioni si spengono anche i loop.
5. **Audio fuori dalla finestra.** Proposta: spento di default (`play_unfocused = false`), come
   la pausa automatica alla perdita del fuoco.
