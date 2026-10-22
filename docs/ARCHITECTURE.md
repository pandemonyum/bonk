# ARCHITECTURE

Rust 2024 + Bevy 0.20.0 (feature `2d`, `ui`). Un solo crate con libreria (`src/lib.rs`) e
binario sottile (`src/main.rs`). Niente workspace finché non serve.

## Due app dalla stessa libreria

| Funzione | Plugin | Uso |
|---|---|---|
| `client_app(options)` | `DefaultPlugins` (nearest) + simulazione + input + render mondo + presentazione + diagnostica | gioco desktop/web |
| `headless_app(seed)` | `MinimalPlugins` + `StatesPlugin` + simulazione; `TimeUpdateStrategy::FixedTimesteps(1)` | test, benchmark, futura simulazione di run complete |

## Moduli

| Modulo | Contenuto | Richiede GPU |
|---|---|---|
| `app` | `AppState` (`InRun`), sottostato `RunPhase` (`Playing`, `Paused`), `RunSeed`, `SimulationPlugin` | no |
| `options` | parsing CLI (`--seed`, `--autoplay`, `--overlay`, `--screenshot*`, `--exit-after`, `--window`, `--scale-factor`) | no |
| `rng` | PCG32, SplitMix64, `hash2` per coordinate | no |
| `gameplay::kinematics` | `Position`, `PreviousPosition`, `body()` | no |
| `gameplay::player` | `Player`, `MoveIntent`, movimento 8 direzioni dentro l'arena | no |
| `gameplay::enemies` | ondata iniziale, inseguimento, separazione, respinta dal player | no |
| `gameplay::input` | tastiera → `MoveIntent`, Esc/P pausa, pausa su perdita focus | input (client) |
| `world` | costanti arena, `ArenaBounds` (`min`/`max` sui piedi); `generation` (seed → tile, pura); `tiles` (indici) | no |
| `world::render` | `TilemapChunk` per chunk | sì |
| `presentation::camera` | canvas 480×270, camera mondo/schermo, scala intera, offset mezzo pixel, follow | sì |
| `presentation::sprites` | observer `On<Add<Player/Enemy>>` → `Sprite`; interpolazione + snap al pixel + y-sort | sì |
| `presentation::hud` | promemoria comandi, banner pausa | sì |
| `diagnostics` | overlay F3, screenshot F12/programmati, autoplay, uscita automatica | sì |

## Flusso di un frame

1. `PreUpdate`/`Update`: input o autoplay scrivono `MoveIntent` (risorsa).
2. `FixedPreUpdate` (60 Hz): `PreviousPosition ← Position`.
3. `FixedUpdate`, in catena e solo in `RunPhase::Playing`: `SimSet::Player` (movimento del
   player) → `SimSet::Movement` (inseguimento) → `SimSet::Resolve` (separazione, limiti arena,
   respinta dal player per ultima).
4. `PostUpdate` (prima di `TransformSystems::Propagate`): `SyncTransforms` scrive i
   `Transform` interpolati e arrotondati; la camera mondo segue il player a pixel interi;
   il canvas si adatta alla finestra.

## Rendering pixel-perfect

- `WorldCamera` (order −1, layer 0) disegna su `Image` 480×270 `Bgra8UnormSrgb`, MSAA off.
- Sprite `Canvas` (layer 1) mostra la texture; `ScreenCamera` (layer 1, `IsDefaultUiCamera`)
  la scala di `n = floor(min(Wfis/480, Hfis/270))` con `ortho.scale = scale_factor / n`.
- `ImagePlugin::default_nearest()` per tutte le texture. Ancora degli sprite in basso al centro
  con larghezza pari: i bordi cadono sulla griglia dei texel.
- Profondità: suolo z=0, attori z = 10 − y·10⁻⁴.

## Dati

S0: valori come costanti con unità nei moduli (`PLAYER_SPEED`, `ENEMY_SPEED`, ...).
P0.0: spostamento in `game-data/*.ron` caricati con l'AssetServer (compatibile WASM) e validati.

## Test

- Unit test accanto al codice per funzioni pure (direzioni, inseguimento, separazione, RNG,
  generazione, scala canvas, opzioni).
- `tests/headless_sim.rs` per i sistemi ECS completi senza GPU.
- `scripts/check-pixel-grid.py` per la verifica nearest-neighbor su screenshot reali.
