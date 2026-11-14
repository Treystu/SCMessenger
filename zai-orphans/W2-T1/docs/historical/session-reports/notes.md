# Scratch Notes

## Migration Notes (2026-06-13)
- Migrated from Claude Code → MiMo Code
- Claude Code config preserved at ~/.claude/settings.json
- MiMo Code global config at ~/.config/mimocode/mimocode.json
- MiMo Code should use the same OpenRouter backend as Claude Code:
  - `ANTHROPIC_BASE_URL=https://openrouter.ai/api`
  - `OPENROUTER_API_KEY` present
  - default model `openrouter/nex-agi/nex-n2-pro:free`
- Direct Xiaomi MiMo credentials are optional and are not the SCMessenger_Clean default.
- mimo binary at /Users/scmessenger/.hermes/node/bin/mimo (added to .zshrc PATH)

## Workspace Architecture
- SCMessenger_Clean = production workspace (gatekeeper approved only)
- SCMessenger = working workspace (consolidation/pruning)
- MiMo Code workspace tied to SCMessenger_Clean — deleting it removes everything

## Orchestration Policy (2026-06-17)
- Kimi (this session) is the primary planner, verifier, and implementer.
- OpenRouter free-tier models via MiMo Code are used only for easy/mechanical sub-tasks.
- Paid OpenRouter models (`openrouter/fusion`, `claude-opus-4.8`) are not used without explicit approval.
