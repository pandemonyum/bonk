# CLAUDE.md

Le regole del progetto sono in [AGENTS.md](AGENTS.md): leggilo per primo, vale anche per Claude Code.

Specifico per Claude Code:

- I ruoli specializzati sono subagent in `.claude/agents/`. La sessione principale è
  l'**orchestrator**: assegna i task ai subagent rispettando l'ownership di `AGENTS.md` e
  integra il risultato. Non lanciare in parallelo due subagent che toccano gli stessi file.
- Terminale: Windows con PowerShell e Git Bash. Rust si trova in `%USERPROFILE%\.cargo\bin`
  (aggiungilo al `PATH` della shell se `cargo` non risulta disponibile).
- Prova visiva: usa le opzioni `--autoplay --screenshot ...` del gioco e apri il PNG prodotto.
  Per il web (dalla P1) è disponibile il server MCP Playwright.
- Arte con ChatGPT (D-065): skill `.claude/skills/chatgpt-art-bridge/` (`/chatgpt-art-bridge`).
  Quando l'utente scrive `<ID> pronto` (es. `ART-HOME-001 pronto`) riprendi da quella richiesta.
- Musica e ambienti (D-070): stesso schema con `docs/audio/AUDIO_HANDOFF_WORKFLOW.md`; per
  `MUSIC-…` o `AMB-… pronto` riprendi da `docs/audio/requests/<ID>.md` con il pixel-art-director.
