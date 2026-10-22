# HANDOFF OPERATIVO — PROJECT PIXEL MONSTER SURVIVOR

**Data:** 22 ottobre 2026  
**Stato:** specifica iniziale / implementazione da avviare  
**Destinatario:** agente AI locale (Claude Code, Codex, o orchestratore equivalente)  
**Priorità:** trasformare questo handoff in un repository funzionante e in un prototipo giocabile; non fermarsi a produrre altra documentazione.

> **ISTRUZIONE PRINCIPALE PER L'AGENTE LOCALE**  
> Sei il **Lead Engineer + Orchestrator** del progetto. Leggi integralmente questo documento. Ispeziona la cartella corrente e l'ambiente di sviluppo, preserva gli eventuali file esistenti, prepara le convenzioni e gli agenti specializzati, poi realizza **vertical slice P0** e **build web P1** secondo i criteri di accettazione. Procedi autonomamente sulle decisioni reversibili di dettaglio, registrandole; non inventare di aver eseguito test, build o deployment. Chiedi intervento umano solo per accessi, credenziali, costi, licenze, o decisioni creative davvero bloccanti. Se manca un tool, realizza la migliore alternativa funzionante e documenta il blocco. Ogni tranche deve produrre codice testabile, non soltanto piani.

---

## 0. Contesto e obiettivo non negoziabile

L'utente vuole creare, **principalmente usando agenti IA**, un videogioco originale ispirato al **gameplay survival roguelite** di *Megabonk*, *Vampire Survivors* e *Azeroth Survivors*, ma con **identità visiva monster-collector in pixel art** (colorata, simpatica, ispirazione generale ai classici RPG di creature). L'ultima concept image condivisa in chat è un collage pixel-art. Se disponibile nel pacchetto, consultare `references/pixel_art_concept.png` **solo come direzione artistica, non come spritesheet pronto per il gioco**.

**Correzione chiave rispetto alle proposte precedenti:** niente gioco Pokémon tradizionale a turni, niente esplorazione JRPG come loop principale, niente mondo 3D realistico. Il gameplay rimane **survivor top-down 2D a orde, con movimento libero, attacchi automatici, esperienza, livelli, scelte di potenziamenti e boss**. La pixel art sostituisce il precedente stile dark-fantasy 3D.

**Obiettivo del prodotto:** browser desktop in prima battuta, più build nativa PC dallo stesso codice; single-player, senza account né backend per il MVP.

**Obiettivo del processo:** generare/modificare gioco e contenuti con file di testo, comandi CLI, generatori procedurali e codice. Evitare dipendenze da editor visuali e operazioni manuali ripetitive. La verifica umana del divertimento e della resa visiva rimane necessaria.

### Principi di proprietà intellettuale

- Creare **creature, nomi, silhouette, palette, abilità, icone, mappe, UI e lore originali**.
- Non usare Pokémon ufficiali, Poké Ball, sprite Nintendo/Game Freak, loghi, nomi, musiche, UI copiate o asset estratti da giochi esistenti.
- Somiglianza ammessa solo a livello di **genere / direzione artistica generale** (pixel art leggibile e creature simpatiche), non di copie riconoscibili.
- Qualsiasi asset di terze parti deve essere tracciato con URL, autore, licenza e condizioni di ridistribuzione.
- Nome di lavorazione: **Pixel Monster Survivor** (provvisorio; verificare disponibilità del nome prima di pubblicazione commerciale).

---

## 1. Game design: la partita tipo

**Fantasia del giocatore:** un giovane evocatore corre sul campo insieme a compagni mostriciattolo originali che combattono automaticamente contro orde di creature ostili. Sceglie sinergie elementali per sopravvivere fino al boss finale.

**Core loop (MVP):**

1. Selezione del compagno iniziale tra 3 archetipi (fuoco, acqua, natura); terzo slot/ulteriori archetipi estendibili.
2. Il giocatore muove l'evocatore con WASD/frecce; camera top-down che segue il personaggio.
3. Compaiono orde; i compagni eseguono automaticamente attacchi con cadenza, targeting e pattern definiti dai dati.
4. I nemici sconfitti rilasciano gemme XP; raccogliendole si sale di livello.
5. Ogni level-up mette in pausa e offre 3 carte casuali: nuove abilità, upgrade, attributi o sinergie.
6. Aumentano numero, velocità e varietà dei nemici; compaiono élite e un boss a fine run.
7. La run termina con vittoria o morte; schermata risultati con durata, nemici sconfitti e build usata.

**Durata iniziale della run:** 10 minuti come configurazione MVP; contenuto bilanciato in iterazioni successive. 

### Meccaniche incluse nell'MVP

- Player 8 direzioni, movimento continuo e collisione semplice con ostacoli.
- Camera che segue il player; zoom/risoluzione gestiti con resa pixel-perfect.
- 1 mappa «Prateria dei Germogli» composta da tile e dettagli procedurali a seed.
- 3 archetipi di compagno **originali** (nomi di lavoro: Brasa/fuoco, Gocci/ acqua, Frond/ natura; l'Art Director deve proporre nomi migliori, verificabili e originali).
- 3 attacchi base automatici: proiettile mirato, onda in area, foglie orbitanti. Sono parametrici e riutilizzabili.
- 3 tipi di nemico + 1 élite + 1 boss originali; IA base inseguimento, separazione per evitare sovrapposizione.
- Drop XP, raccolta per prossimità, level-up, carte upgrade.
- Barre HP / XP, timer, contatori, indicatore delle abilità, pausa e game over.
- Restart senza chiudere l'applicazione; config dati esterna.
- Debug overlay opzionale con FPS, numero nemici, proiettili e collision check.

### Fuori scope nell'MVP

Multiplayer; collezionismo persistente complesso; breeding/evoluzioni; account; classifiche online; monetizzazione; mondo aperto; marketplace; crafting; gacha; mobile touch; storia lunga; live-service; fisica complessa; motore custom da zero. Non aggiungerli senza approvazione: prima va dimostrato il loop survivor.

---

## 2. Direzione artistica / pixel art

**Inquadratura:** top-down o prospettiva 3/4 molto leggera; leggibilità comparabile a *Vampire Survivors* ma più luminosa e amichevole; mai turn-based battle view. Il centro di ogni screenshot in-game deve mostrare il player circondato da molteplici nemici e attacchi automatici, con gemme XP e HUD essenziale.

**Tono:** avventura fantasy vivace; prati, foreste, cascate, caverne di cristallo, rovine vulcaniche. Creature con silhouette forti, occhi/forme espressivi ma design originali.

**Pixel art:**

- Risoluzione virtuale iniziale 480×270 (16:9), con upscaling nearest-neighbor e letterboxing quando necessario; valutare 320×180 se il detail level migliora. Non interpolare le texture con filtro lineare.
- Griglia tile iniziale 16×16 px; sprite personaggio/creature in scala coerente (tipicamente 24–32 px di larghezza, boss più grande); evitare mischiare scale arbitrarie.
- Sprite sheets e texture atlas; palette ridotta e coerente. Preferire hit effects piccoli e leggibili.
- Distinguere chiaramente: alleati, nemici, gemme XP, pericoli e proiettili.
- Schermo 16:9, HUD compatto: HP e compagni in alto a sinistra, tempo in alto, risorse in alto a destra, abilità in basso, XP in fondo.
- Level-up a 3 card, con finestra centrata e mondo oscurato/pausato.

**Ordine di realizzazione asset:** primitive/placeholder leggibili -> sprite originali generati per via procedurale o da artista -> animazioni -> VFX -> polish. Non bloccare la programmazione per ottenere illustrazioni perfette. La concept image è moodboard, NON un asset importabile (non ritagliarla come sorgente di sprite o UI: contiene incoerenze e testi generativi).

**Pipeline da automatizzare:** `assets-src/` (file sorgenti, prompt e metadati), `assets/` (sprite PNG/sheets/tiles/fonts/audio ottimizzati), manifest JSON/TOML per sprite, attribuzioni `ASSET_LICENSES.md`, script di validazione risoluzioni/naming, generazione atlas quando utile. Nessuna dipendenza obbligatoria da Photoshop, Aseprite GUI, Unreal o Unity.

---

## 3. Architettura tecnica scelta

| Ambito | Decisione | Note |
|---|---|---|
| Motore | **Rust + Bevy 0.20** | Pinned a versione `0.20.x` compatibile; aggiornamenti deliberati |
| Rendering | **Bevy 2D sprite/tilemap** | Nessun 3D richiesto |
| Web | **WebAssembly** | Browser moderno, build e test reali |
| Backend grafico web | **WebGL2 come baseline di compatibilità**, WebGPU opzionale | Non presumere WebGPU universale; test Chrome/Edge/Firefox |
| PC | Build Rust nativa Windows | Development & optional release |
| Dati gioco | RON/JSON/TOML, schema versionato | Tutte le statistiche configurabili senza ricompilare se ragionevole |
| Struttura | ECS di Bevy + moduli/plugins | Separare logica, dati, rendering e UI |
| Simulazione | fixed timestep dove opportuno | Preferibilmente 60Hz per logica |
| Random | Seed esplicito | Riproducibilità dei test |
| Hosting | Cloudflare Workers Static Assets o equivalente | Solo dopo build web verificata |
| CI | GitHub Actions | fmt, clippy, test, build native, build web |
| Automazione | Claude Code + Codex | File `AGENTS.md` condiviso e definizioni agente per tool |

**Attenzione:** Bevy 0.20 è uscita l'8 ottobre 2026; leggere esempi e API **della release effettiva bloccata**, non snippet vecchi. Le feature di build web/WebGL2/WebGPU vanno verificate nell'ambiente reale, non supposte. Le limitazioni di hosting possono rendere necessario distribuire un `.wasm` grande tramite servizio di storage/CDN esterno: verificare dimensioni reali (Cloudflare Workers Static Assets: limite per singolo file 25 MiB alla data del documento).

**Target browser:** tastiera + mouse su desktop; non assumere supporto mobile nel MVP. Gestire pagina HTTPS e lifecycle browser, perdita focus, scaling, asset loading e eventuali messaggi d'errore.

### Moduli raccomandati

- `app`: bootstrap, stati `Boot`, `MainMenu`, `Loading`, `Playing`, `LevelUp`, `Paused`, `GameOver`, `Victory`.
- `gameplay/player`: input, movimento, HP, invulnerabilità temporanea.
- `gameplay/companions`: modelli dati, slot, AI follow/orbit, cast automatico.
- `gameplay/combat`: targeting, hitbox, danni, knockback, status e critici opzionali.
- `gameplay/enemies`: spawn director, tipi, movimenti, death/despawn.
- `gameplay/progression`: XP, levelling, upgrade pool, draft 3 carte, applicazione effetti.
- `gameplay/run`: timer, difficoltà, obiettivi, boss, risultati, restart.
- `world`: chunk/tilemap, seed, ostacoli, ambienti, spawn zone.
- `presentation`: sprite animation, VFX, camera, UI, font, audio.
- `data`: loader, schema/validator e parametri bilanciamento.
- `diagnostics`: log, overlays e benchmark.

**Linee guida performance:** object pooling per proiettili/VFX se misurabilmente utile; spatial hash/grid per collisioni e query di prossimità; evitare confronti O(N×M) non necessari; non ricostruire atlas a ogni frame; aggiornamenti diradati della decision logic di orde lontane; limite anti-spawn spiral e garbage di entities. Separare simulazione e rendering per i benchmark headless ove fattibile.

**Soglie iniziali da verificare, NON promesse:** almeno 200 nemici attivi con target 60 FPS su desktop di sviluppo misurato in release; profilo stress a 500 nemici documentando FPS e frame-time; target web ottimizzato dopo la prima build. Il numero finale di nemici dipenderà da dispositivo, VFX e browser.

---

## 4. Predisporre un repository agent-friendly

Se la directory è vuota, inizializza un repository Git Rust/Bevy; se non è vuota, **non sovrascrivere** codice/progetti senza verificarne lo stato. Preferire percorsi brevi e portable (Windows + Linux), niente credenziali nel repo.

### Struttura attesa (adattabile con ADR motivata)

```text
pixel-monster-survivor/
├── AGENTS.md                       # regole condivise lette da Codex e altri agenti
├── CLAUDE.md                       # entrypoint per Claude Code, rimanda ad AGENTS.md
├── .claude/
│   └── agents/
│       ├── game-designer.md
│       ├── gameplay-engineer.md
│       ├── world-generation.md
│       ├── pixel-art-director.md
│       ├── qa-performance.md
│       ├── web-release.md
│       └── code-reviewer.md
├── .github/
│   ├── copilot-instructions.md
│   └── workflows/ci.yml
├── docs/
│   ├── GDD.md
│   ├── ARCHITECTURE.md
│   ├── ART_BIBLE.md
│   ├── BACKLOG.md
│   ├── DECISIONS.md
│   ├── PROGRESS.md
│   ├── QA_REPORT.md
│   ├── RELEASE.md
│   └── agents/                      # ruoli/tool-agnostic e convenzioni handoff
├── crates/                          # opzionale; solo quando il progetto cresce
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── data/
│   ├── gameplay/
│   ├── world/
│   ├── presentation/
│   └── diagnostics/
├── assets-src/
├── assets/
│   ├── sprites/
│   ├── tiles/
│   ├── ui/
│   ├── audio/
│   └── fonts/
├── game-data/
│   ├── companions.ron
│   ├── enemies.ron
│   ├── attacks.ron
│   ├── upgrades.ron
│   └── stages.ron
├── references/
│   └── pixel_art_concept.png       # moodboard, NON asset di produzione
├── scripts/
│   ├── check.sh
│   ├── check.ps1
│   ├── build-web.sh
│   ├── build-web.ps1
│   └── validate-assets.py
├── web/
│   └── index.html
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── ASSET_LICENSES.md
└── README.md
```

L'obiettivo **non** è creare directory vuote per impressionare. Ogni file deve essere utile e quanto prima popolato da codice eseguibile. Evitare architetture premature (microservizi, event bus distribuiti, plugin complessi inutili).

---

## 5. Agenti specializzati da installare

L'orchestratore deve creare **file operativi reali** per i ruoli sotto, non un elenco fittizio. Sfruttare le convenzioni installate nello strumento locale: `.claude/agents/*.md` per subagent Claude Code **se supportati dalla versione presente**; `AGENTS.md` come istruzioni condivise per Codex; per Codex, definire ruoli in `docs/agents/` e usare delega/subagent **solo se effettivamente disponibili**. Se multi-agent non è abilitato, usare esecuzione sequenziale con lo stesso contratto e dichiararlo. Nessun agente deve dichiarare di possedere strumenti non installati.

### A. `orchestrator` — responsabile di delivery (agente principale)
- Scompone task per dipendenze, assegna ownership, coordina merge e test.
- Aggiorna BACKLOG, PROGRESS, DECISIONS e stato build.
- Fa partire un MVP reale prima di rifinire l'intero GDD.
- Integra modifiche, gestisce conflitti e impedisce agenti contemporanei sugli stessi file.
- Segnala blocchi verificabili e richiede approvazione per spese / account / deploy pubblico.

### B. `game-designer` — game loop e bilanciamento
- Produce GDD pratico, parametri modificabili, curve difficoltà, attacchi, sinergie.
- Scrive test case di bilanciamento e obiettivi chiari: giocatore non invincibile, progressione percepibile, build con scelte significative.
- **Non** implementa gameplay in Rust salvo task specifico e autorizzato.

### C. `gameplay-engineer` — ECS / combat / progression
- Implementa plugin e componenti Bevy, input, combattimento, nemici, abilità, level-up, save della run se richiesto.
- Unit test delle formule pure; integrazione con dati esterni.
- Verifica compatibilità API Bevy 0.20 invece di generare codice inventato.

### D. `world-generation` — mappe e ambientazione
- Procedural world seedable, tile placement, ostacoli, streaming solo se serve, visibilità/occlusioni, placement leggibile.
- Mantiene la mappa attraversabile; test di spawn e zone sicure; nessuna mappa editata manualmente come prerequisito.

### E. `pixel-art-director` — stile e pipeline asset
- Mantiene ART_BIBLE, palette, sprite dimensions, silhouette e coerenza visiva.
- Produce asset originali via strumenti realmente disponibili o placeholder procedurali, con manifest/licenze.
- Valida che applichino nearest-neighbor e che le animazioni non sfumino i pixel.
- Evita esplicitamente riproduzioni di personaggi/asset Pokémon.

### F. `qa-performance` — test e analisi prestazioni
- Scrive test automatizzati, esegue fmt/clippy/test, smoke desktop/web, benchmark reproducibili.
- Non dichiara "60 FPS" senza device, build, scenario, durata e misure.
- Diagnostica regressioni, asset mancanti, crash e loop infinito.

### G. `web-release` — build WASM, CI/CD e hosting
- Configura build, pagina HTML, asset packaging, MIME, cache, manifest, CI.
- Produce artefatto statico locale e script ripetibile.
- **Non** pubblica o configura servizi a pagamento senza approvazione esplicita dell'utente.

### H. `code-reviewer` — qualità e integrazione
- Review di API, ownership ECS, test e sicurezza; controlla duplicazioni, test vacui e file non usati.
- Può rifiutare una tranche che non si avvia o rompe i requisiti.

### Template obbligatorio per ogni definizione agente

Ogni file ruolo deve specificare:

1. Missione, scope e fuori scope.
2. Input richiesti e artefatti/output attesi.
3. File/directory di ownership (senza confini incompatibili con altri agenti).
4. Tool disponibili e fallback se assenti.
5. Criteri di completamento e test da eseguire.
6. Protocollo di handoff all'orchestratore con modifiche, test, problemi e rischi.
7. Divieti: non inventare test riusciti, non installare a caso pacchetti non verificati, non distruggere file esistenti, non esporre secret.

**Contratto di consegna di ogni task:** ID, obiettivo, assunzioni, file toccati, comandi eseguiti con esito, screenshot/log se esistono, limiti aperti, prossimo task dipendente.

---

## 6. Sequenza dei lavori — task che l'agente deve ESEGUIRE

**Non eseguire tutto in un'unica modifica monolitica.** Lavorare a tranche verificabili, commit atomici quando possibile e un gate di validazione prima di ciascuna tranche successiva.

### PHASE S0 — bootstrap e ambiente

**S0.1** Ispeziona filesystem, Git, `rustc`, `cargo`, `rustup`, target WASM, terminale/OS e tool AI locali; scrivi risultati in `docs/PROGRESS.md`. Se già esiste un progetto, rispetta il suo layout.  
**S0.2** Crea/aggiorna `AGENTS.md`, `CLAUDE.md`, i ruoli agent, cartelle docs, `.gitignore`, README con comandi veri.  
**S0.3** Imposta una versione specifica di Rust stabile che supporti Bevy 0.20, toolchain e lockfile; documenta comando di aggiornamento.  
**S0.4** Crea applicazione Bevy 2D minimale che mostra schermata di test, sprites/tiles provvisori, movement input e nearest-neighbor.  
**S0.5** Esegui `cargo fmt --all -- --check`, `cargo check`, `cargo test` (dove possibile) e `cargo run`. Scrivi gli esiti effettivi.

**Gate S0:** repository avviabile localmente; immagine 2D con movimento visibile e pixel non sfumati, setup agenti e doc non vuoti.

### PHASE P0 — core playable desktop (PRIORITÀ ASSOLUTA)

**P0.1** Player controllabile (WASD/frecce), camera follow, mappa tile 2D e ostacoli.  
**P0.2** Un companion originale segue il player; attacco automatico colpisce il nemico più vicino nel raggio.  
**P0.3** Spawn director genera almeno 3 archetipi di nemici configurati; inseguimento, danni player, morte, hit feedback, despawn.  
**P0.4** Gemme XP da nemici, pickup, incremento XP/level; 3 carte upgrade, selezione e applicazione a caldo.  
**P0.5** Secondo e terzo companion e attacchi distinti; gestione slot basata su dati.  
**P0.6** Timer run 10 min, scala difficoltà, élite e un boss con telegraph chiaro; game over/victory + restart.  
**P0.7** HUD leggibile e test formule pure/bilanciamento base; debug overlay.

**Gate P0:** 10 minuti giocabili end-to-end sul PC di sviluppo senza crash, con personaggio, compagni, orde, XP, upgrade, boss e restart. **Non confondere codice che compila con gioco verificato:** registrare prova manuale o automazione di input, screenshot e comportamento realmente osservato.

### PHASE P1 — web giocabile (non aspettare il polishing)

**P1.1** Build `wasm32-unknown-unknown` e packaging coerente per Bevy 0.20; scegliere e documentare il bundler (`Trunk`, `wasm-bindgen` o altro) dopo verifica di compatibilità reale.  
**P1.2** HTML loader, file statici, fallback compatibilità WebGL2 e handling errori (schermata se browser non supportato).  
**P1.3** Testa con server HTTP locale (non `file://`), DevTools senza errori fatali, input e gameplay.  
**P1.4** Script `scripts/build-web.ps1` (Windows) e `.sh`, output `dist/` pronto al deploy.  
**P1.5** Controlla asset paths, WASM size, cache invalidation, MIME (`application/wasm`) e dimensioni file hosting.  
**P1.6** Configura CI che generi un artefatto web scaricabile; deployment automatico solo dopo permesso e credenziali fornite.

**Gate P1:** aprendo `http://localhost:...` in almeno un browser desktop supportato si gioca la stessa vertical slice, con evidenza screenshot / test e log.

### PHASE P2 — art pass originale e UX

**P2.1** Redigi ART_BIBLE con palette, reference e dimensioni; crea 3 compagni, 3 nemici, boss, evocatore, tile, UI originali.  
**P2.2** Sostituisci asset provvisori mantenendo gameplay e collisioni invariati.  
**P2.3** Aggiungi 2–4 frame per idle/movement dei principali sprite, animazioni di attacco, VFX e indicatori di pericolo.  
**P2.4** Aggiungi breve soundtrack/SFX solo con risorse originali o licenze adatte e volume regolabile.  
**P2.5** Screenshot comparison con concept come **moodboard**: deve essere survivor, non una schermata RPG.

**Gate P2:** pixel art coerente, personaggi riconoscibili anche in massa, HUD leggibile a 100% e nessun asset con licenza non documentata.

### PHASE P3 — qualità, progressione e release candidate

**P3.1** Profiling e ottimizzazione delle orde; benchmark con 50/200/500 nemici e report misurati.  
**P3.2** Balance tuning con run seeds riproducibili; 2–3 build elementali veramente distinte.  
**P3.3** Opzioni base: audio, fullscreen/windowed per native, volume, pixel scale, pause.  
**P3.4** Test web Chrome/Edge e almeno un altro browser desktop compatibile o documentazione della limitazione.  
**P3.5** Packaging release, licenze, changelog e deployment staging previa approvazione.

**Gate P3:** release candidate installabile/eseguibile e accessibile sul sito di staging approvato, documentando problemi residui.

---

## 7. Specifiche tecniche che devono rimanere data-driven

Esempio di struttura **illustrativa**, lo schema definitivo può essere RON/JSON/TOML con validazione all'avvio:

```json
{
  "id": "companion_leaf_01",
  "display_name": "Nome originale da definire",
  "element": "nature",
  "sprite": "sprites/companions/leaf_01.png",
  "base_stats": {"hp": 100, "attack": 12, "attack_interval_s": 1.1, "speed": 110},
  "starting_attack": "leaf_orbit",
  "tags": ["orbit", "nature", "starter"]
}
```

**Upgrade spec:** ID, rarità, prerequisiti, max stack, formula e descrizione UI. Esempi: proiettili +1, danno +15%, intervallo attacco -10%, area +20%, pickup radius +25%, nuovo slot companion. `upgrade_id` deve attivare modifiche runtime testabili, non soltanto cambiare il testo della card.

**Curve spawner:** seed, difficoltà per minuto, cap per enemy archetype, zone valid spawn fuori camera, warning per elites, eventuale adattamento difficulty; niente spawn dentro il player.  
**Pianificazione contenuti:** 3 companion starters, 3 classi enemy, 1 élite, 1 boss, ~10 upgrade realmente funzionanti nella prima versione con contenuti veri (non mock).

---

## 8. Test, QA, criteri di qualità

### Comandi / controlli minimi

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings` (oppure dichiarare eccezioni motivate)
- `cargo test`
- `cargo run` e test interattivo nativo con screenshot
- build web riproducibile (`scripts/build-web.*`)
- smoke test via server HTTP locale e browser con eventuali screenshot/console log
- profiler o overlay con frame-time + active counts per performance

In CI, evitare la dipendenza obbligatoria dal display grafico per gli unit test: isolare formule, RNG/seed, targeting, curve XP, spawner e upgrade dalle risorse GPU. Non bloccare i test headless per window/renderer.

### Test di accettazione P0

- Player si muove in 8 direzioni e resta dentro map bounds dove previsti.
- Companion segue e colpisce un nemico vicino senza input aggiuntivo.
- I danni riducono HP; nemico a HP=0 muore e rilascia XP una sola volta.
- Salita di livello pausa correttamente la simulazione; scegliendo una card viene applicato l'effetto.
- Upgrade + projectile deve cambiare numero proiettili visibili; upgrade damage deve cambiare numero danno effettivo.
- Spawn director non crea entità illimitate nonostante un restart; un restart ripulisce lo stato della run.
- Una run può finire in morte o vittoria, senza panic.
- Sprite mantiene pixelation nearest-neighbor anche con resize.

### Anti-pattern da evitare

- Eseguire `cargo check` e dichiarare «MVP completato» senza aver visto l'app.
- Generare dieci documenti e zero gameplay.
- Inventare nomi di API Bevy, dipendenze o flag senza verificarli.
- Bloccare tutta la pipeline per asset AI non disponibili.
- Aggiungere backend, Kafka, database, account o servizi cloud inutili.
- Script distruttivi, deploy pubblico o costi non approvati.
- Un solo file Rust gigantesco / ECS systems con migliaia di righe.
- Rendering di 1000 sprite non ottimizzati per inseguire un numero arbitrario.
- Copiare risorse Nintendo/Game Freak o usare trademark come identità del prodotto.

---

## 9. Contratto operativo per il lavoro automatico

1. All'inizio di ogni sessione leggi: `AGENTS.md`, `docs/PROGRESS.md`, `docs/BACKLOG.md`, `docs/DECISIONS.md` se presenti.
2. Identifica il **prossimo task non completato** secondo dipendenze.
3. Nomina un owner e scrivi criteri di done prima di modificare i file.
4. Svolgi il lavoro, esegui i test più pertinenti e correggi problemi autonomamente.
5. Prima di dichiarare «done», fornisci la prova: comandi realmente eseguiti, output sintetici, app/screenshots se pertinenti.
6. Aggiorna `docs/PROGRESS.md` con completato, in corso, blocchi e prossimo task eseguibile.
7. Crea commit chiari solo se repository/utente consentono; non push su remote né release pubbliche non approvate.
8. Se un nuovo requisito entra in conflitto con questo handoff, privilegia l'ultimo requisito esplicito dell'utente e aggiorna la documentazione.

**Decisioni bloccanti:** account hosting, installazione tool con licenza/costi, credenziali, generazione con API pagate, scelta definitiva del nome commerciale, deploy pubblico, utilizzo di asset esterni con licenza ambigua. Per tutto il resto, procedi con default prudenti.

**Report ideale a fine tranche:**

```text
Task: P0.2 - Companion auto attack
Stato: DONE / PARTIAL / BLOCKED
File modificati: ...
Comandi eseguiti (risultato): ...
Comportamento verificato: ...
Screenshot/log: ...
Rischi o bug noti: ...
Prossimo task: P0.3
```

---

## 10. La primissima esecuzione da fare dopo la lettura

Per evitare «analysis paralysis», l'agente locale **deve iniziare oggi dalla configurazione e dal prototipo**, non da una presentazione.

**Ordine esatto:**

1. `pwd`, `git status` (se repo), `rustc --version`, `cargo --version`, verifica Rust toolchain e necessità di prerequisiti locali (su Windows MSVC/C++ Build Tools).
2. Crea l'impianto di agenti (almeno `AGENTS.md`, `CLAUDE.md`, `.claude/agents/`, `docs/agents/`) e documenta esplicitamente cosa il tool AI supporta davvero.
3. Bootstrap Bevy **2D** versione 0.20, minimo progetto nativo.
4. Fai apparire una schermata top-down pixel-art placeholder, un evocatore quadrato/sprite originale, movimento 8 direzioni, camera follow e 10 nemici placeholder che inseguono il player.
5. Lancia `cargo fmt`, `cargo check`, test e `cargo run` se disponibile un display; annota problemi e log reali.
6. Solo dopo passa al companion e agli attacchi automatici: questo è il primo elemento ludico differenziante.

**Definizione di "primo risultato utile":** un'applicazione **avviabile**, non solo documentazione, nella quale con WASD il giocatore si muove nella mappa e i nemici lo inseguono. La tranche immediatamente successiva aggiunge 1 companion che attacca automaticamente.

---

## 11. Riferimenti tecnici da consultare (fonti ufficiali)

- Release Bevy 0.20 (8 ottobre 2026): https://bevy.org/news/bevy-0-20/
- Bevy Quick Start: https://bevy.org/learn/quick-start/getting-started/
- Bevy Setup / ottimizzazione compilazione: https://bevy.org/learn/quick-start/getting-started/setup/
- Bevy rendering 2D e WebGPU/WebGL2 examples: https://bevy.org/examples-webgpu/ e https://bevy.org/examples/2d-rendering/sprite-sheet/
- Bevy ECS fixed timestep: https://github.com/bevyengine/bevy/blob/main/examples/ecs/fixed_timestep.rs
- Cloudflare Workers Static Assets: https://developers.cloudflare.com/workers/static-assets/
- Cloudflare Workers limits: https://developers.cloudflare.com/workers/platform/limits/

> Gli snippet in documentazione web possono non essere aggiornati all'ultima release; verificare sempre i numeri di versione e l'effettiva compilazione con la versione fissata nel repo.

---

## 12. Prompt di avvio — da incollare nell'agente locale

```text
Leggi integralmente HANDOFF_PIXEL_MONSTER_SURVIVOR.md e assumi il ruolo di
Lead Engineer/Orchestrator. Esegui la Phase S0 e poi la P0 in task piccoli,
realizzando concretamente un prototipo Rust + Bevy 0.20 2D pixel art.

Per prima cosa:
- ispeziona questa cartella e lo stato del toolchain, senza sovrascrivere;
- configura AGENTS.md, CLAUDE.md e agenti specializzati per gli strumenti
  effettivamente installati, con compiti, file ownership e criteri di done;
- prepara docs/PROGRESS.md, docs/BACKLOG.md, docs/DECISIONS.md;
- crea ed esegui un primo Bevy 2D nativo con movimento player, mappa e
  10 nemici che inseguono; poi implementa companion e attacco automatico;
- documenta comandi realmente eseguiti, errori e risultati;
- non fermarti a pianificare o a scrivere agenti: produci codice eseguibile.

Vietato inserire asset Pokémon, dipendenze cloud inutili, deploy pubblico,
acquisti o API a pagamento senza esplicita approvazione.
Se una fase è bloccata dall'ambiente, salva un report riproducibile e
prosegui con le parti realizzabili senza dichiarare successi inesistenti.
```

**Handoff end.**
