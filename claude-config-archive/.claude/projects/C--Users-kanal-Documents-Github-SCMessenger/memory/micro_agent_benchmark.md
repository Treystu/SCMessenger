---
name: micro_agent_benchmark
description: API cost and quality benchmarks for micro task wiring via direct ollama run
type: project
---

## Micro Agent Benchmarks (direct `ollama run`, May 2026)

Micro tasks use `task_wire_micro_*.md` format (TARGET/WIRE/VERIFY) with `.claude/scripts/micro_agent.py`.
No Claude Code wrapper — single `ollama run --think=false --hidethinking` call.

| Model | API/task (5hr%) | Quality | Cleanup | Speed |
|---|---|---|---|---|
| deepseek-v4-flash:cloud (140B) | ~2.4% | Perfect | None | ~2 min |
| deepseek-v4-pro:cloud (1.6T) | ~2.5% | Perfect | None | ~3 min |
| gemma4:31b:cloud | ~2.2% | ~50% (misplaced code, private field access) | Yes | ~2 min |
| qwen2.5-coder:7b (local) | 0% | Partial | Yes | ~4 min |

**Why:** Flash and Pro produce identical output for structured micro wiring tasks. The quality gap only matters for complex reasoning (architecture, review). Flash is slightly faster and same price.

**How to apply:** Route micro wiring tasks to `deepseek-v4-flash:cloud`. Reserve `deepseek-v4-pro:cloud` for architecture/review. Avoid `gemma4:31b:cloud` for code edits — it accesses private fields and misplaces code ~50% of the time.