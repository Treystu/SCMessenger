# SCMessenger MiMo Code provider setup

## Current required backend

SCMessenger_Clean should run MiMo Code through the same OpenRouter backend and model used by the working Claude Code connection. Claude Code uses the Anthropic-compatible OpenRouter endpoint (`https://openrouter.ai/api`); MiMo Code uses the OpenAI-compatible adapter, so its provider URL must be the OpenRouter OpenAI-compatible endpoint (`https://openrouter.ai/api/v1`).

Required environment:

```bash
export OPENROUTER_API_KEY="<your OpenRouter key>"
export ANTHROPIC_BASE_URL="https://openrouter.ai/api"
export ANTHROPIC_AUTH_TOKEN="<same OpenRouter key>"
unset ANTHROPIC_API_KEY
```

Default agent model for this workspace:

```bash
openrouter/nex-agi/nex-n2-pro:free
```

This is the free/ultra-low-cost completion backend ported from Claude Code to MiMo Code for SCMessenger swarm/agentic work.

## Deprecated provider cleanup

MiMo Code should have no Xiaomi MiMo provider, Xiaomi endpoint, or Xiaomi credential configured. The only connector is OpenRouter.

## Verification

After launching MiMo Code from `SCMessenger_Clean`, confirm the resolved config:

```bash
mimo debug config
mimo providers list
mimo models openrouter
```

Expected:

- `provider.openrouter.api` resolves to `https://openrouter.ai/api/v1` for MiMo Code's OpenAI-compatible adapter
- `provider.openrouter.options.headers.Authorization` resolves to `Bearer $OPENROUTER_API_KEY`
- `OPENROUTER_API_KEY` is present
- `openrouter/nex-agi/nex-n2-pro:free` is available
- `ANTHROPIC_API_KEY` is not set

Smoke test:

```bash
mimo run --agent build --model openrouter/nex-agi/nex-n2-pro:free 'Reply with one line: MiMo OpenRouter smoke test OK.'
```
