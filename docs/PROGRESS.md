# PROGRESS

Stato corrente, ultimo aggiornamento: **2026-10-22**.

| Fase | Stato |
|---|---|
| S0 — bootstrap e ambiente | **DONE** (gate S0 superato, prove sotto) |
| P0 — core giocabile desktop | da iniziare: prossimo task **P0.1** |
| P1 — web | non iniziata |
| P2 / P3 | non iniziate |

Prossimo task eseguibile: **P0.1** (ostacoli e collisioni nell'arena), vedi `docs/BACKLOG.md`.

---

## S0.1 — Ispezione ambiente (2026-10-22)

| Elemento | Esito |
|---|---|
| OS | Windows 11 Enterprise 10.0.26200 |
| CPU / RAM | Intel Core Ultra 5 238V (8 core), 31,5 GiB |
| GPU | Intel Arc 130V integrata, backend Vulkan (driver 101.8737) |
| Cartella | conteneva solo `HANDOFF_PIXEL_MONSTER_SURVIVOR.md`; non era un repo Git |
| `references/pixel_art_concept.png` | **assente**: si procede senza moodboard |
| Git | 2.54.0 (`user.name` globale presente, email impostata solo nel repo) |
| Python | 3.14.4 + Pillow 12.2.0 |
| Node | presente (nvm4w) |
| Rust | assente → installato rustup 1.29 via winget; toolchain bloccata **1.99.1** |
| MSVC | assente → installati **VS Build Tools 2022 17.14.42** + Windows SDK 10.0.26100 via winget |
| Target WASM | `wasm32-unknown-unknown` installato con la toolchain |
| Trunk / wasm-bindgen / wasm-opt | assenti (servono dalla P1) |
| Codex CLI | assente |
| Disco | ~37 GB liberi all'inizio; `target/` dopo la S0: 10,4 GB |

## Report tranche S0

```text
Task: S0.1–S0.5 - Bootstrap, agenti, prima app Bevy 2D
Owner: orchestrator (sequenziale, ruoli gameplay-engineer / world-generation /
       pixel-art-director / qa-performance nello stesso agente)
Stato: DONE
Assunzioni: default approvati dall'utente (cartella Bonk come radice, arena chiusa,
       mondo 480x270, WebGL2 in P1, sprite generati da script, documenti brevi).
File: AGENTS.md, CLAUDE.md, .claude/agents/*.md (7), docs/agents/README.md, docs/*.md,
       Cargo.toml, Cargo.lock, rust-toolchain.toml, src/** (21 file), tests/headless_sim.rs,
       assets-src/**, assets/**, scripts/{build-assets,validate-assets,check-pixel-grid}.py,
       scripts/check.{ps1,sh}, ASSET_LICENSES.md, README.md, .gitignore
Comandi eseguiti (risultato):
  winget install Rustlang.Rustup                                   OK
  winget install Microsoft.VisualStudio.2022.BuildTools (VCTools)   OK
  rustup toolchain install 1.99.1 (+clippy, rustfmt, wasm32)       OK
  cargo build (prima, con tutte le dipendenze)                     OK, 1384 s (~23 min)
      (il primo tentativo è fallito solo perché lib.rs è stato creato a build avviata)
  cargo build incrementale                                         OK, 27–60 s
  cargo fmt --all -- --check                                       OK (dopo cargo fmt)
  cargo clippy --all-targets -- -D warnings                        OK, 0 warning
  cargo test                                                       OK, 38 test (31 unit + 7 headless)
      due fallimenti reali trovati e corretti durante la tranche:
      - radure di terra al 15% dell'arena → soglia 0.74 → 0.80
      - nemici spinti dentro il player dalla separazione → nuovo sistema keep_out_of_player
  code review (ruolo code-reviewer): CHANGES REQUESTED, 7 rilievi, tutti corretti e
      riverificati (dettaglio in docs/QA_REPORT.md)
  python scripts/build-assets.py / validate-assets.py              OK, 3 PNG, 0 errori
  ./scripts/check.ps1                                              OK (fmt, clippy, test, asset)
  cargo run -- --autoplay --overlay --screenshot ...               OK, exit 0, screenshot salvato
  python scripts/check-pixel-grid.py (1366x768, 1365x767,
      1101x701 con --scale-factor 1.25 e 1.5)                       OK, 100% blocchi uniformi
Comportamento verificato (osservato, non solo compilato):
  - finestra 960x540, mondo a 480x270 scalato 2x, pixel netti (docs/evidence/s0-autoplay-960x540.png)
  - arena procedurale da seed con erba, fiori, radure; 10 gelatine che inseguono e si separano
  - tastiera reale (keybd_event di Windows): D 1 s + W 1 s → player a (76,75), atteso (75,75);
    Esc → pausa con banner e simulazione ferma (docs/evidence/s0-keyboard-pausa.png)
  - finestra 1365x767 (dimensioni dispari): letterbox nero, scala 2x, 100% texel uniformi;
    con la correzione di mezzo pixel disattivata lo stesso controllo scende al 96,1% e fallisce
    (docs/evidence/s0-letterbox-1365x767.png)
  - DPI forzato 1.25 e 1.5: UI ingrandita, canvas sempre 2x con texel perfetti
    (docs/evidence/s0-dpi150-1101x701.png)
  - overlay F3: 60–64 FPS, 15,7–16,5 ms dopo 9–15 s, build dev, 10 nemici, vsync attivo.
    Non è un benchmark: niente release, scenario minimo (vedi docs/QA_REPORT.md)
Rischi o bug noti: vedi "Limiti aperti" in docs/QA_REPORT.md
Prossimo task: P0.1
```
