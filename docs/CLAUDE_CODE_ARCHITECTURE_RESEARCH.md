# Claude Code Architecture Research: Strategic Implications for SCMessenger

**Source:** Comprehensive analysis of the Claude Code source map leak (March 31, 2026)
**Purpose:** Distill actionable architectural patterns for maximizing SCMessenger development velocity
**Last updated:** 2026-04-28

## 1. The Harness Engineering Paradigm

The most profound revelation from the Claude Code leak: **98.4% of the codebase is deterministic infrastructure** — permission gates, context management, tool routing, file parsing, and failure recovery. Only 1.6% is AI decision-making logic or prompt formulation.

**Implication for SCMessenger:** The competitive moat in AI-assisted development has shifted from the LLM to the "harness layer." The existing SCMessenger CLAUDE.md and .claude/ infrastructure should be treated as a mission-critical operating system, not loose configuration. Every rule, constraint, and operational manifest in CLAUDE.md is re-injected into the context prefix on every conversational turn — it must be precise, complete, and never ambiguous.

## 2. The Nine-Stage Agentic Loop

Claude Code's core execution pipeline operates as a `while` loop with nine stages per turn:

1. **Settings resolution** — Load configuration hierarchy
2. **State initialization** — Restore session and environment state
3. **Context assembly** — Gather relevant files, memory, and rules
4-8. **Five sequential pre-model compaction shapers** — Budget reduction, snip, microcompact, context collapse, autocompact
9. **Model call** → **Tool dispatch** → **Permission gating** → **Tool execution** → **Stop condition evaluation**

**Implication for SCMessenger:** Every prompt competes for token budget against system overhead. Prompts must be dense and action-oriented — avoid conversational fluff. The system is designed for parallel tool dispatch; sequential, step-by-step instructions waste both latency budget and context window.

## 3. Multi-Model Orchestration Strategy

Claude Code implements tiered model routing rather than routing everything through a single model. In a single session, three different models may be invoked transparently:

| Tier | Equivalent | Ollama Cloud Model | Use Case |
|------|-----------|-------------------|----------|
| Flagship reasoning | Opus 4.6/4.7 | `glm-5.1:cloud`, `deepseek-v4-pro:cloud`, `kimi-k2-thinking:cloud` | Architecture, deep analysis, complex planning |
| Primary workhorse | Sonnet 3.5/4.0 | `qwen3-coder:480b:cloud`, `qwen3-coder-next:cloud`, `glm-5.1:cloud` | Main coding, feature implementation |
| Fast/lightweight | Haiku 3.5/4.5 | `deepseek-v4-flash:cloud`, `gemini-3-flash-preview:cloud`, `gemma4:31b:cloud` | Triage, lint, CI, title generation, simple queries |

**SCMessenger routing strategy (to be implemented):**
- **Architecture & planning:** `glm-5.1:cloud` or `qwen3-coder:480b:cloud`
- **Rust core implementation:** `glm-5.1:cloud` (proven Rust output) or `qwen3-coder-next:cloud`
- **Crypto/math/protocol validation:** `deepseek-v3.2:cloud` or `deepseek-v4-pro:cloud`
- **Code review / gatekeeping:** `kimi-k2-thinking:cloud` or `kimi-k2.6:cloud`
- **Quick fixes, lint, CI:** `gemini-3-flash-preview:cloud` or `deepseek-v4-flash:cloud`
- **Documentation, tests, bindings:** `gemma4:31b:cloud` or `devstral-2:123b:cloud`
- **Pipeline coordination:** `mistral-large-3:675b:cloud`

## 4. Three-Layer Memory Architecture

Claude Code uses transparent, file-based memory rather than opaque vector databases:

### Layer 1: Static Configuration (CLAUDE.md hierarchy)
Parsed in a four-level priority chain:
1. Managed directives (`/etc/`)
2. User-level (`~/.claude/`)
3. Project-level (`CLAUDE.md`, `.claude/rules/`)
4. Local overrides (`CLAUDE.local.md`, typically gitignored)

**Critical finding:** These files are NOT ingested once. They are **re-injected into the context prefix on every single turn**, guaranteeing persistent constraint awareness.

### Layer 2: Pointer Index (MEMORY.md)
A dynamic directory of memory references for structural lookups without retaining full file contents in the active buffer.

### Layer 3: In-Context Memory
The active transcript of the current session.

### Skeptical Memory
The system forces the LLM to treat historical recollections as **probabilistic hints** rather than empirical facts. Before mutating based on recalled information, the agent must verify current state against external reality via read tools.

**Implication for SCMessenger:**
- CLAUDE.md must contain **definitive constraints** ("never modify the legacy auth module") not suggestions
- CLAUDE.md must include **operational manifests** (exact build/test/deploy commands)
- MEMORY.md serves as the pointer index for key documentation locations
- Agents must verify before acting — trust observed state over recalled state

## 5. Context Compaction Cascade

When the context window approaches capacity, a five-stage cascade executes (cheapest and least destructive first):

| Stage | Mechanism | Trigger |
|-------|-----------|---------|
| 1. Budget Reduction | Per-tool byte limits (`maxResultSizeChars`) | Always active |
| 2. Snip Compaction | Prune older message pairs | Headless/SDK mode |
| 3. Microcompact | Strip stale tool calls (5+ turns old), replace with boundary marker | Context size threshold |
| 4. Context Collapse | Progressive compression of older segments (`marble_origami`) | Feature-flagged |
| 5. Autocompact | Dense model-generated summary of older history | Configurable token threshold |

**Emergency safeguard:** If API returns `prompt_too_long`, Reactive Compact intercepts the fault mid-request, forces compaction, and auto-retries.

**Implication for SCMessenger:**
- **Avoid triggering compaction** by using `head`, `tail`, bounded `sed` operations instead of full file dumps
- **Prioritize shell tools** (grep, ripgrep, git log/diff) over sequential file reading
- **Phrase prompts for parallel execution** — "read X and Y simultaneously" triggers concurrent path
- A circuit breaker exists because runaway compaction once wasted 250,000 API calls/day — respect token budgets

## 6. Deterministic Tool Ecosystem

Claude Code ships **54 distinct tools**, **27 hook events**, and **4 extension mechanisms**:

| Category | Key Tools | SCMessenger Relevance |
|----------|-----------|----------------------|
| File I/O | Read, Write, Edit, NotebookEdit | Core code editing |
| Search | Glob, Grep, WebSearch, WebFetch | Code exploration, API docs |
| Execution | Bash, MCP, LSP, Skill | Build/test, external services |
| Agent Coordination | Agent, SendMessage, TeamCreate, TaskCreate | Swarm orchestration |

**Parallel execution:** The query engine detects independent operations and dispatches concurrently via two paths:
- `runTools` — classifies as concurrent-safe or exclusive
- `StreamingToolExecutor` — begins execution as tokens stream in

### Bash Security Framework

Over 2,500 lines of defense logic with 25+ validators (regex, shell-quote parsing, tree-sitter AST analysis):
- `validateSafeCommandSubstitution` — subshell inspection
- `validateRedirections` — blocks dangerous output redirects
- `validateObfuscatedFlags` — detects ANSI-C quoting evasion
- Three-parser differential mechanism — routes commands through three independent parsers to catch edge cases

**Implication for SCMessenger:** Git operations require explicit trust dialogs because hooks and config can execute arbitrary code. Always defer destructive operations.

## 7. Multi-Agent Swarms & Worktree Isolation

- `/batch` command decomposes tasks into 5-30 parallel git worktree agents
- Each agent operates in isolated environment with file-based JSON mailboxes
- Protected by locking mechanisms and 10-retry backoff sequences
- Shared prompt cache reduces token costs (workers inherit context prefix from lead)

**Implication for SCMessenger:** For large refactoring tasks (e.g., migrating the transport layer, updating UniFFI bindings across platforms), use the `/batch` swarm pattern.

## 8. Prompt Engineering Evolution

The transition from Opus 4.6 (Fennec) to Opus 4.7 represents the **"death of the warm assistant"**:

- **Old paradigm:** Friendly brainstorming partner, inferring intent from ambiguity, using emojis
- **New paradigm:** "Grumpy senior contractor" — direct, opinionated, executes exact specifications without inferring unstated requirements
- **Thinking Frequency Tuning:** Calibrates cognitive pacing based on task complexity
- **Retrieval-only directive:** Sub-agents must not solve from general knowledge when no verifiable data exists in project files

**Implication for SCMessenger:**
- Write prompts as **technical specifications**, not collaborative discussions
- Assume the agent will execute literally — ambiguity is a liability
- Use the `effort` parameter (`high`/`xhigh`) for complex logic, not temperature/top_p

## 9. KAIROS Daemon & Dream Mode (Research Preview)

Unreleased subsystems reveal Anthropic's strategic direction toward persistent autonomous daemons:

### KAIROS
- Always-on background agent with periodic `<tick>` prompts
- 15-second blocking budget constraint
- Exclusive tools: `SendUserFile`, `PushNotification`, `SubscribePR`

### Dream Mode (autoDream)
- Triggers after 24 hours of inactivity (requires 5+ recent sessions)
- Merges observations, prunes contradictions, consolidates learnings
- Rewrites MEMORY.md pointer index autonomously

**Implication for SCMessenger:** The scheduled tasks skill (`anthropic-skills:schedule`) and consolidate-memory skill (`anthropic-skills:consolidate-memory`) provide analogs to these patterns today. These should be configured for SCMessenger's autonomous operation.

## 10. Security & Supply Chain

### Compaction Poisoning
Attackers embedding logic bombs in repository config files can trick the agent during autocompact to elevate malicious instructions into permanent trusted memory markers (`ContextCollapseCommitEntry`). This would embed a persistent backdoor surviving session restarts.

### Anti-Distillation Poisoning
The system detects competitor scraping behavior and injects fabricated tool definitions and logic errors into API responses.

### SCMessenger Hardening Priorities
1. Audit CLAUDE.md and .claude/rules/ for compaction-safe instruction patterns
2. Never embed secrets or API keys in any config file the agent can read
3. Use the adversarial review pattern before merging critical modules (crypto, transport, relay)
4. Verify git operations require explicit trust dialogs
5. Keep ollama.com/api/tags accessible for model availability verification

## 11. Strategic Implementation Playbook for SCMessenger

### 11.1 Architecting the Persistent Context

- **Definitive constraints:** "never modify the legacy auth module," "prefer WebSockets over SSE for message delivery"
- **Operational manifests:** Exact build/test/deploy commands in CLAUDE.md
- **No ambiguity:** Every instruction is a rule, not a suggestion

### 11.2 Exploiting Parallel Execution

- Phrase prompts to trigger parallel path: "read user schema AND message schema simultaneously"
- Use `/batch` for macro-refactoring (ORM migration, transport layer updates)

### 11.3 Maximizing Tool Efficacy

- Prefer `grep`/`ripgrep` over sequential file reads
- Use `git log`/`git diff` directly instead of asking the agent to summarize
- Limit output with `head`/`tail`/bounded `sed` to avoid compaction triggers

### 11.4 Enforcing Quality via Adversarial Review

- Invoke adversarial review for crypto, notification queue, database locking
- Model acts as security auditor — probes for race conditions, null checks, edge cases
- Isolate reviewer in purely destructive, critical capacity

### 11.5 Autonomous Swarm Completion

- Delegate high-volume deterministic work (tests, bindings, docs) to mid/small-tier models
- Reserve flagship models for architectural decisions and protocol correctness
- Escalate only philosophical-level decisions to the human operator
- Use scheduled tasks for periodic model availability checks and memory consolidation
