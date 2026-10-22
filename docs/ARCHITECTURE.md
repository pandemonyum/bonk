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
| `gameplay::enemies` | ondata iniziale, inseguimento, separazione, respinta dal player, rimpiazzo (`SpawnRng` unico per la run) | no |
| `gameplay::companions` | `Companion`, scia dietro al player, attacco del compagno di fuoco | no |
| `gameplay::combat` | `Health`, `Hurtbox`, `AutoAttack`, `Projectile`, `HitFlash`, `RunStats`; targeting, colpi, morti | no |
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
   player) → `SimSet::Movement` (inseguimento, scia dei compagni, volo dei proiettili) →
   `SimSet::Resolve` (separazione, limiti arena, respinta dal player per ultima) →
   `SimSet::Combat` (spari, colpi, morti, scadenza proiettili, fine tinta) → `SimSet::Spawn`
   (rimpiazzo nemici). Una morte è contata una sola volta: il despawn è applicato prima del
   passo successivo e i colpi ignorano chi è a 0 HP.
4. `PostUpdate` (prima di `TransformSystems::Propagate`): `SyncTransforms` scrive i
   `Transform` interpolati e arrotondati; la camera mondo segue il player a pixel interi;
   il canvas si adatta alla finestra.

## Rendering pixel-perfect

- `WorldCamera` (order −1, layer 0) disegna su `Image` 480×270 `Bgra8UnormSrgb`, MSAA off.
- Sprite `Canvas` (layer 1) mostra la texture; `ScreenCamera` (layer 1, `IsDefaultUiCamera`)
  la scala di `n = floor(min(Wfis/480, Hfis/270))` con `ortho.scale = scale_factor / n`.
- `ImagePlugin::default_nearest()` per tutte le texture. Ancora degli sprite in basso al centro
  con larghezza pari: i bordi cadono sulla griglia dei texel.
- Profondità: suolo z=0, attori z = 10 − y·10⁻⁴, proiettili z = 20 − y·10⁻⁴ (`DepthBase`).

## Dati

S0: valori come costanti con unità nei moduli (`PLAYER_SPEED`, `ENEMY_SPEED`, ...).
P0.0: spostamento in `game-data/*.ron` caricati con l'AssetServer (compatibile WASM) e validati.

## Test

- Unit test accanto al codice per funzioni pure (direzioni, inseguimento, separazione, RNG,
  generazione, scala canvas, opzioni).
- `tests/headless_sim.rs` (movimento, limiti, pausa) e `tests/combat_sim.rs` (compagno,
  danni, morti, cadenza, popolazione) per i sistemi ECS completi senza GPU; helper in
  `tests/common/`.
- `tests/balance_probe.rs`: sonda di bilanciamento `#[ignore]`, stampa statistiche di 60 s.
- `scripts/check-pixel-grid.py` per la verifica nearest-neighbor su screenshot reali.
