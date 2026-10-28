# Handoff art pass v1 "16-bit" → orchestratore

Ramo `claude/art-style-16bit-adda3a`. Stile e regole in `docs/ART_BIBLE.md` v1; anteprime in
`docs/evidence/art-preview-*.png`, prova nel gioco in `docs/evidence/art-v1-gioco.png`.

Cosa c'è già nel ramo e funziona senza altro codice: palette v1, tutti gli sprite ridisegnati
con gli stessi percorsi, tileset da 42 tile con bordi morbidi e varianti (agganciato in
`src/world/tiles.rs` e `src/world/generation.rs`, con test), icone UI in `assets/ui/`.
Cosa resta al codice: i valori sotto (dimensioni cambiate) e il collegamento delle icone UI.

## 1. Dimensioni nuove e valori di gioco da aggiornare

Gli sprite restano ancorati in basso al centro (proiettili, gocce, foglie e segno di pericolo al
centro). Player, compagni, élite, boss, alberi e rocce hanno un'**ombra a terra cotta** nelle
ultime 1–3 righe: la `Position` (piedi) cade al centro dell'ombra, quindi i piedi disegnati
stanno 1–2 px sopra la base.

| Sprite | v0 | v1 |
|---|---|---|
| `sprites/summoner.png` → `sprites/summoner_sheet.png` | 16×21 | 16×24, poi foglio animato del personaggio dell'utente (D-050): 144×304 con fotogrammi 36×38; `summoner.png` è stato rimosso |
| `sprites/ember_companion.png` | 16×17 | 16×20 |
| `sprites/tide_companion.png` | 16×14 | 18×18 |
| `sprites/sprout_companion.png` | 16×17 | 16×20 |
| `sprites/blob_enemy.png` … `boss_enemy.png` (righe sotto) | — | sostituiti dai fogli animati `*_sheet.png` e rimossi (2026-10-28, vedi `docs/ART_BIBLE.md`) |
| `sprites/blob_enemy.png` | 16×13 | 16×14 |
| `sprites/swarm_enemy.png` | 12×8 | 14×11 |
| `sprites/armored_enemy.png` | 20×15 | 20×15 |
| `sprites/elite_enemy.png` | — (segnaposto P0.6) | 26×22 |
| `sprites/boss_enemy.png` | — (segnaposto P0.6) | 56×47 |
| `sprites/telegraph.png` | — (segnaposto P0.6) | 6×6, ancora al centro |
| `sprites/fireball.png` | 6×6 | 8×8 |
| `sprites/leaf.png` | 6×6 | 8×8 |
| `sprites/droplet.png` | 4×4 | 4×4 |
| `sprites/xp_gem.png` | 6×8 | 8×10 |
| `sprites/rock.png` | 16×10 | 16×12, poi 20×16 nello stile del player (impronta invariata) |
| `sprites/tree.png` | 24×26 | 32×32, poi 40×48 nello stile del player (impronta invariata) |

Valori proposti (sono proposte del pixel-art-director: i numeri di bilanciamento restano del
game-designer, il codice del gameplay-engineer e di world-generation):

| File | Campo | Ora | Proposto | Motivo |
|---|---|---|---|---|
| `game-data/attacks.ron` | `ember_bolt` → `radius` | 3.0 | 4.0 | la sfera disegnata è 8×8 |
| `game-data/attacks.ron` | `ember_bolt` → `muzzle_height` | 8.0 | 9.0 | centro del corpo di Brasa (16×20, occhi a 10 px dalla base) |
| `game-data/attacks.ron` | `tide_pulse` → `center_height` | 8.0 | 10.0 | petto dell'evocatore, ora alto 24 |
| `game-data/attacks.ron` | `leaf_orbit` → `center_height` | 8.0 | 10.0 | stesso centro dell'onda |
| `game-data/attacks.ron` | `leaf_orbit` → `radius` | 5.0 | 5.0 | invariato: la foglia 8×8 ha raggio visivo 4, 5 resta generoso |
| `game-data/enemies.ron` | `blob` → `hurtbox` | (7, 6) | (7, 7) | una riga in più, corpo centrato a 7 |
| `game-data/enemies.ron` | `swarm` → `hurtbox` | (5, 4) | (6, 5) | ali larghe 14, corpo a 5 px dalla base |
| `game-data/enemies.ron` | `swarm` → `separation_radius` | 10 | 12 | sagoma più larga, evita che gli sciami si sovrappongano a pacchi |
| `game-data/enemies.ron` | `armored` | — | invariato | stessa misura |
| `game-data/enemies.ron` | élite (P0.6) → `hurtbox` | — | (12, 10) | corpo 26×22 |
| `game-data/enemies.ron` | élite → `footprint_radius` / `contact_distance` / `separation_radius` | — | 9 / 16 / 26 | 9 è il massimo validato (varco 18 px) |
| `game-data/enemies.ron` | boss (P0.6) → `hurtbox` | — | (22, 22) | massa del corpo tra le righe 8 e 44 |
| `game-data/enemies.ron` | boss → `footprint_radius` / `contact_distance` / `separation_radius` | — | 9 / 28 / 44 | impronta al massimo validato; lo sprite è molto più largo dell'impronta (passa tra gli alberi "sopra" le chiome, accettabile per un boss) |
| `game-data/player.ron` | `footprint_radius` | 5 | 5 | larghezza invariata |
| `src/world/obstacles.rs` | `ObstacleKind::Rock.footprint()` | (7.0, 3.0) | (7.0, 4.0) | roccia più alta di 2 px, base con ombra |
| `src/world/obstacles.rs` | `ObstacleKind::Tree.footprint()` | (4.0, 2.0) | (4.0, 4.0) | la base del tronco è 2 px sopra il fondo (ombra) |
| `src/world/mod.rs` | `BODY_HEIGHT` | 21.0 | 24.0 | altezza del nuovo evocatore: la testa resta a filo della siepe nord; il test `whole_sprite_stays_inside_the_hedge` passa a `arena.max.y = 1992`. Con il foglio del player (D-050) il personaggio è alto 30 px più l'ombra (fotogramma 36×38): valore e sporgenza davanti alla siepe sono scelte del codice |

`MAX_FOOTPRINT_RADIUS` (7) e `MIN_FOOTPRINT_RADIUS` (4) non cambiano, quindi il varco minimo
di 18 px resta valido.

Nota sulla tinta di colpo (`HIT_TINT`, `SLOW_TINT`): moltiplica tutto lo sprite, quindi anche
l'ombra cotta si arrossa o si azzurra per 0,12 s. Accettato; se disturba, l'ombra può diventare
uno sprite figlio separato (servirebbe codice in `src/presentation/sprites.rs`).

## 2. Icone UI (`assets/ui/`, tipo `ui` nel manifest)

Disegnarle alla **stessa scala intera del canvas** (1 texel = 1 pixel del mondo 480×270), con
sampler nearest: se l'HUD resta a risoluzione nativa (D-013), l'`ImageNode` va dimensionato a
`lato × scala_del_canvas`. Posizioni di riferimento in pixel del canvas (da
`scripts/preview-scene.py`, funzione `hud`).

| File | Misura | Uso previsto |
|---|---|---|
| `ui/heart_full.png`, `ui/heart_half.png`, `ui/heart_empty.png` | 10×9 | HP in alto a sinistra: un cuore ogni 20 HP (100 HP = 5 cuori), a (6, 5) con passo 11 |
| `ui/icon_fire.png`, `ui/icon_water.png`, `ui/icon_nature.png` | 16×16 | fila delle abilità sotto la barra XP, a (6, 24) con passo 18, una per compagno in squadra; anche nell'angolo delle carte della scelta del compagno |
| `ui/clock.png` | 10×10 | accanto al timer della run, centrato in alto (y = 4) |
| `ui/skull.png` | 10×10 | contatore uccisioni in alto a destra (y = 13); anche a sinistra della barra del boss |
| `ui/lock.png` | 12×14 | slot bloccati: compagno o mappa non ancora disponibili |
| `ui/upgrade_<id>.png` | 16×16 | icona in alto al centro della carta del draft, una per ogni id di `upgrades.ron`: `ember_damage`, `ember_rate`, `ember_multishot`, `ember_pierce`, `ember_range`, `ember_speed`, `vitality`, `regen`, `swift`, `magnet`, `snack`, `join_tide`, `join_sprout` |
| `ui/portrait_ember.png`, `ui/portrait_tide.png`, `ui/portrait_sprout.png` | 32×32 | scelta del compagno: ritratto in alto nella carta, sfondo trasparente |

`validate-assets.py` avvisa (senza fallire) se un upgrade o un compagno dei dati non ha icona o
ritratto: un id nuovo ricorda di chiedere l'icona.

Colori per i nodi UI di Bevy (dalla palette, `Color::srgb_u8`):

| Elemento | Colore |
|---|---|
| Pannello delle carte e della scelta | fondo `1` #1b2640, riga di luce in alto `2` #2b3a5e, contorno `k` #1b1428 |
| Bordo carta per elemento | fuoco `o` #f27a2e, acqua `c` #2f8f9d, natura `e` #b8e05a, neutro `3` #5a6d99 |
| Carta selezionata / sotto il mouse | bordo oro `g` #f2c94c con alone esterno `y` #ffd65c |
| Velo sopra il gioco durante il draft | `0` #0f1626 a ~60% di opacità |
| Barra XP | fondo `0`, riempimento `u` #4fa4e0, riga di luce `W` #c8eeff, ombra `U` #2f6fb0 |
| Barra del boss | fondo `0`, riempimento `r` #e2574c, luce `X` #ff9a7a, ombra `R` #a8323a |
| Barre ATT/VEL/AREA | fuoco `f` #ffaa45, teal `l` #6cc7c2, lime `e` #b8e05a |
| Testi | titoli `w` #f4f1e8, secondari `4` #a3b3d3, livello e titolo del draft oro `g` |

Per le carte servono sei colori e un contorno: si fanno con nodi `BackgroundColor` +
`BorderColor`, niente immagini a 9 fette.

Sprite per la P0.6: `sprites/elite_enemy.png` e `sprites/boss_enemy.png` con ancora in basso al
centro come gli altri nemici; `sprites/telegraph.png` va agganciato al centro (`Sprite` con
ancora di default) e, come le gocce dell'onda, si usa in tanti a cerchio sul bordo dell'area.
Al merge con i segnaposto della P0.6 vince la versione di questo ramo.

## 3. Tileset (già collegato in questo ramo)

`TILE_COUNT` 7 → 42. `chunk_tiles` usa la nuova `visual_tile_at`, che aggiunge bordi
terra/erba e siepe/erba (maschera dei 4 lati) e varianti d'erba con fiori a macchie;
`tile_at` è invariata, quindi ostacoli, arena e gameplay non cambiano. Test nuovi in
`generation.rs`: indici validi e materiale logico conservato, orientamento dei bordi della
terra, bordi della siepe solo verso l'arena, presenza delle varianti.

## 4. Font pixel da proporre all'utente (D-013, D-033) — adottati Tiny5 e Pixelify Sans (D-044)

Aggiornamento 2026-10-23: l'utente ha approvato la proposta; i due font sono in
`assets/fonts/` con la loro licenza, verificati sul cmap e nel gioco (dettagli in D-044).

Tutti SIL Open Font License 1.1. Prima di adottarne uno: aprire il file e controllare che
`à è é ì ò ù` siano disegnate, copiare `OFL.txt` accanto al font in `assets/`, aggiungere la riga
in `ASSET_LICENSES.md` e una decisione in `docs/DECISIONS.md` (poi togliere il vincolo ASCII).

| Font | Autore | Licenza | Fonte | Accentate | Note |
|---|---|---|---|---|---|
| **Pixelify Sans** | Stefie Justprince | OFL 1.1 | Google Fonts; [fontsource.org/fonts/pixelify-sans](https://fontsource.org/fonts/pixelify-sans/about) | sottoinsieme `latin-ext` pubblicato (Fontsource) | contorni vettoriali disegnati su griglia di pixel: nitido solo a multipli della sua griglia; il più leggibile per titoli e descrizioni delle carte |
| **Tiny5** | Stefan Schmidt | OFL 1.1 | [fonts.google.com/specimen/Tiny5](https://fonts.google.com/specimen/Tiny5/about) | insiemi Google "Latin Core" e "Latin Plus" (lingue dell'Europa occidentale) | alto 5 px, larghezza variabile: entra nel canvas 480×270 a 1× per numeri ed etichette dell'HUD |
| **Press Start 2P** | Cody "CodeMan38" Boisclair | OFL 1.1 (nome riservato "Press Start 2P Regular") | [fontlibrary.org/font/press-start-2p](https://fontlibrary.org/font/press-start-2p) | dichiarato paneuropeo (italiano tra le lingue elencate da 1001fonts); non verificato glifo per glifo | 8×8 arcade, molto riconoscibile; lettere larghe, adatto a titoli e numeri, troppo largo per le descrizioni |

Proposta: Tiny5 per l'HUD dentro il canvas e Pixelify Sans per draft, menu e game over.

## 5. Verifiche eseguite in questo ramo (2026-10-23)

- `python scripts/build-assets.py`: 42 asset generati. `python scripts/validate-assets.py`: OK,
  42 PNG, 11 sprite citati in `game-data/`, 0 errori, nessun avviso.
- `cargo test`: 125 unit test (20 in `world::`, 4 nuovi) e 65 di simulazione passati, 0
  falliti, 4 ignorati (le sonde `--ignored`). `rustfmt --check` sui due file toccati: OK.
  Clippy non eseguito in questo ramo (disco): va nel gate dopo il merge.
- Gioco vero: `cargo run -- --seed 20 --autoplay --god --time-scale 4 --window 1101x701
  --no-watch --screenshot ... --screenshot-burst 8`, senza `--overlay` per non coprire il
  canvas. Sprite e tileset nuovi caricati e disegnati; `check-pixel-grid.py` 100% sugli scatti
  senza draft, 95,5% su quelli col draft aperto (il testo è sopra il canvas, falso negativo
  noto). Prova: `docs/evidence/art-v1-gioco.png`.
- Non verificato nel gioco: orda grande con i nuovi sprite (solo nell'anteprima), élite, boss e
  segni di pericolo (servono i dati della P0.6), icone UI (non ancora collegate).
