# Licenze degli asset

Ogni asset distribuito con il gioco deve comparire qui: origine, autore, licenza, condizioni.
`python scripts/validate-assets.py` verifica che ogni PNG in `assets/` abbia una licenza nel
manifest `assets/manifest.json`.

## Asset originali del progetto

| Asset | Origine | Licenza |
|---|---|---|
| `assets/sprites/fireball.png` | `assets-src/sprites/fireball.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/rock.png` (20×16) | `assets-src/sprites/rock.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano (forme ombreggiate rifinite nella griglia) |
| `assets/sprites/tree.png` (40×48) | `assets-src/sprites/tree.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano (forme ombreggiate rifinite nella griglia) |
| `assets/sprites/xp_gem.png` | `assets-src/sprites/xp_gem.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/leaf.png` | `assets-src/sprites/leaf.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/droplet.png` | `assets-src/sprites/droplet.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/spark.png` (scintilla del fulmine di Tuonny, 6×6) | `assets-src/sprites/spark.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/telegraph.png` | `assets-src/sprites/telegraph.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/blob_sheet.png` (Gelatina, foglio 4×2 da 28×24) | `assets-src/sprites/blob_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/swarm_sheet.png` (Sciame, 4×2 da 24×22) | `assets-src/sprites/swarm_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/armored_sheet.png` (Corazzato, 4×2 da 32×28) | `assets-src/sprites/armored_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/elite_sheet.png` (Gelatina Gigante, 4×4 da 44×38) | `assets-src/sprites/elite_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/boss_sheet.png` (Gran Corazzato, 4×4 da 96×80) | `assets-src/sprites/boss_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/ui/icon_fire.png` | `assets-src/ui/icon_fire.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/icon_water.png` | `assets-src/ui/icon_water.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/icon_nature.png` | `assets-src/ui/icon_nature.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_full.png` | `assets-src/ui/heart_full.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_half.png` | `assets-src/ui/heart_half.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_empty.png` | `assets-src/ui/heart_empty.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/skull.png` | `assets-src/ui/skull.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/clock.png` | `assets-src/ui/clock.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/lock.png` | `assets-src/ui/lock.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_damage.png` | `assets-src/ui/upgrade_ember_damage.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_rate.png` | `assets-src/ui/upgrade_ember_rate.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_multishot.png` | `assets-src/ui/upgrade_ember_multishot.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_pierce.png` | `assets-src/ui/upgrade_ember_pierce.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_range.png` | `assets-src/ui/upgrade_ember_range.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_speed.png` | `assets-src/ui/upgrade_ember_speed.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_vitality.png` | `assets-src/ui/upgrade_vitality.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_regen.png` | `assets-src/ui/upgrade_regen.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_swift.png` | `assets-src/ui/upgrade_swift.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_magnet.png` | `assets-src/ui/upgrade_magnet.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_snack.png` | `assets-src/ui/upgrade_snack.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_tide.png` | `assets-src/ui/upgrade_join_tide.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_ember.png` | `assets-src/ui/upgrade_join_ember.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_sprout.png` | `assets-src/ui/upgrade_join_sprout.px` → `scripts/build-assets.py` | originale, proprietà del progetto (testina dei compagni di D-052 disegnata nel progetto) |
| `assets/ui/upgrade_join_spark.png` | `assets-src/ui/upgrade_join_spark.px` → `scripts/build-assets.py` | originale, proprietà del progetto (testina dei compagni di D-052 disegnata nel progetto) |
| `assets/ui/icon_electric.png` | `assets-src/ui/icon_electric.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/tiles/meadow_tileset.png` | generato da `scripts/build-assets.py` (seed 20261022, 42 tile dalla v1, ridisegnati nello stile del player il 2026-10-28) | originale, proprietà del progetto |
| Palette "Germoglio" v1 (`assets-src/palette.txt`, 56 colori) | definita per il progetto | originale, proprietà del progetto |
| Palette dei nemici (`assets-src/sprites/enemies.palette.txt`, 18 colori) | definita per il progetto: viola e rossi della palette "Germoglio", contorno come il player | originale, proprietà del progetto |

## Terze parti distribuite in `assets/`

| Asset | Autore | Licenza | Fonte | Note |
|---|---|---|---|---|
| `assets/fonts/tiny5/Tiny5.ttf` (variabile, istanza di default) | Stefan Schmidt, "The Tiny5 Project Authors" | SIL Open Font License 1.1, testo in `assets/fonts/tiny5/OFL.txt` | Google Fonts, `github.com/google/fonts` `ofl/tiny5/Tiny5[BLED,JITT,ROND,wdth,wght].ttf` (upstream `github.com/Gissio/font_tiny5`), scaricato il 2026-10-23, SHA-256 `588626f2…d8fa` | file rinominato, contenuto non modificato; HUD (D-044) |
| `assets/fonts/pixelify-sans/PixelifySans.ttf` (variabile, istanza di default) | Stefie Justprince, "The Pixelify Sans Project Authors" | SIL Open Font License 1.1, testo in `assets/fonts/pixelify-sans/OFL.txt` | Google Fonts, `github.com/google/fonts` `ofl/pixelifysans/PixelifySans[wght].ttf` (upstream `github.com/eifetx/Pixelify-Sans`), scaricato il 2026-10-23, SHA-256 `9ba86cd0…e8a5` | file rinominato, contenuto non modificato; menu, draft, risultati (D-044) |

Nessuno dei due dichiara un Reserved Font Name. L'OFL permette di distribuirli con il gioco
anche a pagamento, purché il file di licenza li accompagni e il font non sia venduto da solo.

## Terze parti incluse nel binario

| Asset | Autore | Licenza | Note |
|---|---|---|---|
| Font FiraMono (subset), incorporato da Bevy (`bevy_text`, feature `default_font`) | Mozilla / Carrois Apostrophe | SIL Open Font License 1.1 | resta solo nell'overlay di debug (F3); l'interfaccia usa Tiny5 e Pixelify Sans (D-044) |

Motore e librerie Rust: Bevy e dipendenze sono MIT/Apache-2.0; l'elenco completo si genera
con `cargo tree` (da affiancare a un report licenze prima della release, P3.5).

Stile v1 (2026-10-23): tutti gli sprite e le icone sopra sono disegnati a mano come griglie
`.px` (i più grandi abbozzati con forme geometriche e rifiniti a mano), senza generatori IA né
asset scaricati. L'immagine di riferimento dell'utente ha guidato solo lo stile: nessun pixel,
forma, personaggio o nome ne è stato copiato. Le anteprime `docs/evidence/art-preview-*.png`
sono composte da `scripts/preview-scene.py` con questi stessi asset (non vanno nel gioco).

## Asset forniti dall'utente

| Asset | Origine | Diritti e licenza |
|---|---|---|
| `assets/sprites/ember_sheet.png` (Fiammetto, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/ember_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_ember.png` (Fiammetto, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_ember.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/tide_sheet.png` (Ondino, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/tide_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_tide.png` (Ondino, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_tide.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/sprout_sheet.png` (Foglin, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/sprout_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_sprout.png` (Foglin, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_sprout.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/spark_sheet.png` (Tuonny, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/spark_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_spark.png` (Tuonny, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_spark.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| Palette `assets-src/sprites/companions.palette.txt` (36 colori) | colori del foglio dell'utente, sfumature quasi uguali unite | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/summoner_sheet.png` (player, foglio 4×8 da 36×38) | foglio di animazioni generato e fornito dall'utente, tagliato a risoluzione nativa, poi ridisegnato e pulito nel progetto: `assets-src/sprites/summoner_sheet.px` con la palette propria `assets-src/sprites/summoner_sheet.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica; il progetto l'ha ridisegnato e ripulito. Rischio di somiglianza con personaggi Nintendo accettato dall'utente (D-050). Nel manifest: `user-provided (rights declared by the user, D-050), redrawn and cleaned in assets-src` |

Cosa è stato fatto sul foglio dell'utente (2026-10-28, D-050):
- **Personaggio:** design, colori, proporzioni e dettagli restano quelli dell'utente.
- **Colori:** le sfumature quasi identiche del generatore sono unite in 24 colori tutti presi
  dal suo foglio, più l'ombra a terra del progetto.
- **Master:** il fermo 0 di fronte, di schiena e di profilo è il master. Schiena e profilo sono
  portati all'altezza del fronte togliendo righe uniformi, senza ricampionare i dettagli.
- **Fotogrammi:** fermo, cammino e colpito 0 e 3 derivano dai master. Colpito 1–2 e sconfitto
  0–3 sono le pose dell'utente ripulite (il colpito 1 ridotto all'altezza del master).

Il foglio originale e il ritaglio restano fuori dal repository.

## Asset vietati

Nessuno sprite, suono, font, logo o UI estratto da Pokémon o da altri giochi. La concept
image in `references/` (se presente) è solo moodboard e non va ritagliata.
