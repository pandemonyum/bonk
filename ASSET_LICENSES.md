# Licenze degli asset

Ogni asset distribuito con il gioco deve comparire qui: origine, autore, licenza, condizioni.
`python scripts/validate-assets.py` verifica che ogni PNG in `assets/` abbia una licenza nel
manifest `assets/manifest.json`.

## Asset originali del progetto

| Asset | Origine | Licenza |
|---|---|---|
| `assets/sprites/summoner.png` | `assets-src/sprites/summoner.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/blob_enemy.png` | `assets-src/sprites/blob_enemy.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/ember_companion.png` | `assets-src/sprites/ember_companion.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/fireball.png` | `assets-src/sprites/fireball.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/rock.png` | `assets-src/sprites/rock.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/tree.png` | `assets-src/sprites/tree.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/tiles/meadow_tileset.png` | generato da `scripts/build-assets.py` (seed 20261022) | originale, proprietà del progetto |
| Palette "Germoglio" (`assets-src/palette.txt`) | definita per il progetto | originale, proprietà del progetto |

## Terze parti incluse nel binario

| Asset | Autore | Licenza | Note |
|---|---|---|---|
| Font FiraMono (subset), incorporato da Bevy (`bevy_text`, feature `default_font`) | Mozilla / Carrois Apostrophe | SIL Open Font License 1.1 | usato solo per HUD/overlay provvisori; da sostituire con un font pixel (P2) |

Motore e librerie Rust: Bevy e dipendenze sono MIT/Apache-2.0; l'elenco completo si genera
con `cargo tree` (da affiancare a un report licenze prima della release, P3.5).

## Asset vietati

Nessuno sprite, suono, font, logo o UI estratto da Pokémon o da altri giochi. La concept
image in `references/` (se presente) è solo moodboard e non va ritagliata.
