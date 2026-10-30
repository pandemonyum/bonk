# Ruoli degli agenti — indice tool-agnostic

Le definizioni dei ruoli sono **una sola copia**: i file in `.claude/agents/`. Sono Markdown
semplice (l'intestazione YAML serve solo a Claude Code) e valgono per qualunque strumento.

| Ruolo | Definizione | Quando |
|---|---|---|
| orchestrator | questo file, sezione sotto | sempre: sessione principale |
| game-designer | [.claude/agents/game-designer.md](../../.claude/agents/game-designer.md) | GDD, valori, test di bilanciamento |
| gameplay-engineer | [.claude/agents/gameplay-engineer.md](../../.claude/agents/gameplay-engineer.md) | codice ECS, dati, presentazione |
| world-generation | [.claude/agents/world-generation.md](../../.claude/agents/world-generation.md) | arena procedurale |
| pixel-art-director | [.claude/agents/pixel-art-director.md](../../.claude/agents/pixel-art-director.md) | ART_BIBLE, sprite, licenze |
| qa-performance | [.claude/agents/qa-performance.md](../../.claude/agents/qa-performance.md) | test, smoke, benchmark |
| web-release | [.claude/agents/web-release.md](../../.claude/agents/web-release.md) | WASM, dist, CI |
| code-reviewer | [.claude/agents/code-reviewer.md](../../.claude/agents/code-reviewer.md) | review prima dell'integrazione |

## Cosa supportano davvero gli strumenti (verificato il 2026-10-22)

| Strumento | Stato su questa macchina | Modalità |
|---|---|---|
| Claude Code (app desktop) | installato | subagent da `.claude/agents/*.md`; l'orchestratore delega con il tool Agent |
| Codex CLI | **non installato** | quando presente: legge `AGENTS.md`; esegue i ruoli in sequenza leggendo il file del ruolo |
| GitHub Copilot | non configurato | nessun file dedicato finché non viene usato |
| ChatGPT (account dell'utente) | nessuna integrazione, per scelta (D-065) | solo handoff manuale per l'arte: l'utente incolla il prompt e salva il PNG in `.art-inbox/` (`docs/art/CHATGPT_ART_WORKFLOW.md`) |

Se uno strumento non supporta subagent, l'esecuzione è **sequenziale**: lo stesso agente
assume un ruolo alla volta, rispetta l'ownership di quel ruolo e produce il suo report.

## orchestrator — responsabile della consegna

- Scompone i task per dipendenze in `docs/BACKLOG.md`, assegna owner e criteri di done.
- Mantiene `docs/PROGRESS.md`, `docs/DECISIONS.md`, `docs/BACKLOG.md` e lo stato build.
- Delega ai ruoli; non lancia in parallelo ruoli che toccano gli stessi file.
- Integra, esegue `scripts/check.*`, fa passare la tranche dal code-reviewer quando è
  sostanziale, poi commit locale atomico. Niente push o deploy senza approvazione.
- Chiede all'utente solo: accessi, credenziali, costi, licenze, nome commerciale, deploy
  pubblico, asset esterni con licenza ambigua. Per il resto decide con default prudenti e li
  registra in `docs/DECISIONS.md`.

## Contratto di consegna di ogni task

ID, obiettivo, assunzioni, file toccati, comandi eseguiti con esito, screenshot/log se
esistono, limiti aperti, prossimo task dipendente. Formato in `AGENTS.md`.
