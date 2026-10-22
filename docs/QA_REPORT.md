# QA REPORT

## 2026-10-22 — S0

**Macchina:** Windows 11 Enterprise 26200, Intel Core Ultra 5 238V, 31,5 GiB, Intel Arc 130V
integrata (Vulkan). **Build:** profilo `dev` (codice opt-level 1, dipendenze opt-level 3).

### Automatici

| Controllo | Comando | Esito |
|---|---|---|
| Formattazione | `cargo fmt --all -- --check` | OK |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | OK, 0 warning |
| Unit test | `cargo test --lib` | 31/31 OK |
| Simulazione headless | `cargo test --test headless_sim` | 7/7 OK (~1 s) |
| Asset | `python scripts/validate-assets.py` | OK, 3 PNG, 0 errori |
| Griglia pixel | `python scripts/check-pixel-grid.py` | OK 100% a 1366×768, 1365×767, 1101×701 @1.25 e @1.5 |
| Compilazione WASM | `cargo check --lib --target wasm32-unknown-unknown --locked` | OK (eseguito dal code-reviewer, 7 min 27 s; build web completa: P1) |

I test headless coprono: spawn (1 player + 10 nemici); velocità 75 px/s su 60 passi; player
sempre dentro l'arena dopo 60 s di spinta in diagonale; inseguimento (distanza media −75% in
8 s) con nessun nemico dentro il player e coppie di nemici sempre oltre 7 px (metà del raggio
di separazione); nemici mai dentro il player quando è schiacciato contro un muro; stesso seed =
stessa simulazione; pausa che congela la simulazione e ripresa che la fa ripartire.

### Prove sul gioco reale

| Prova | Come | Osservato | Prova |
|---|---|---|---|
| Avvio e rendering | `cargo run -- --autoplay --overlay --screenshot ...` | finestra 960×540, arena, 10 nemici che inseguono, pixel netti, exit 0 | `docs/evidence/s0-autoplay-960x540.png` |
| Tastiera reale | `keybd_event` di Windows: D 1 s, W 1 s, Esc | player (76,75) vs atteso (75,75); pausa con banner, fase `Paused` | `docs/evidence/s0-keyboard-pausa.png` |
| Resize / letterbox | `--window 1365x767` + `check-pixel-grid.py` | scala 2×, bande nere, 100% texel uniformi | `docs/evidence/s0-letterbox-1365x767.png` |
| Controprova mezzo pixel | stessa finestra con correzione disattivata | 96,1% → il controllo fallisce (non vacuo) | log nel report S0 |
| DPI 125% / 150% | `--window 1101x701 --scale-factor 1.25` e `1.5` | testo UI ingrandito (override attivo), canvas 2×, 100% texel uniformi | `docs/evidence/s0-dpi150-1101x701.png` |

Nota: a 960×540 senza bande nere il promemoria comandi copre il canvas e `check-pixel-grid`
misura ~99% anche con scala corretta: il controllo va fatto con finestre letterbox.

### Prestazioni (indicative, NON benchmark)

Overlay F3, build dev, 10 nemici, vsync attivo: 64 FPS / 15,7 ms a 15 s (1366×768), 60 FPS /
16,5 ms a 9 s (960×540). Nei primi 6 s la media è più bassa (49 FPS) per l'avvio. Benchmark
50/200/500 nemici in release: P3.1.

### Code review S0 (ruolo code-reviewer, lettura indipendente)

Verdetto iniziale CHANGES REQUESTED, 7 rilievi, tutti corretti nella stessa tranche:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | `check.ps1` poteva dire OK senza eseguire un passo (comando mancante) | exit code azzerato per passo, `try/catch`, fallback `py` | passo con comando inesistente → exit 1 |
| 2 | Nessun test sulla ripresa dalla pausa | asserzioni di ripresa nel test di pausa | rimuovendo `resume_time` il test fallisce |
| 3 | Ordine indefinito tra `move_player` e `chase_player` | nuovo `SimSet::Player` prima di `Movement` | test di determinismo verdi |
| 4 | Nemici dentro il player contro un muro | respinta dal player come ultima regola | nuovo test `enemies_never_enter_the_player_against_a_wall` |
| 5 | Limiti arena pensati per sprite centrati | `ArenaBounds` con `min`/`max` sui piedi | test `whole_sprite_stays_inside_the_hedge` |
| 6 | Attesa screenshot calcolata da `--exit-after` | si attende fino a `max(exit, screenshot) + 10 s` | revisione del codice |
| 7 | Affermazioni non provate nei documenti (determinismo cross-platform, "non sovrapposti", DPI 125%) | testi corretti; test di separazione rafforzato; DPI provato con `--scale-factor` | sopra |

### Limiti aperti

- DPI provato solo con override (`--scale-factor`), non su un monitor reale al 125%.
- Testato solo con finestra in primo piano; minimizzazione e monitor multipli non provati.
- La pausa su perdita di focus è attiva ma non provata con un vero cambio di finestra.
- Replay identici tra nativo e WASM non garantiti (vedi D-005).
- Build release non ancora compilata né misurata.
- Nessuna prova umana di "sensazione" del movimento: serve un playtest dell'utente.
