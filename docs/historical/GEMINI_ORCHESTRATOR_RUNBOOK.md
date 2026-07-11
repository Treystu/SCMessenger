# Gemini 3.5 Flash Orchestrator Runbook

Status: Active
Last updated: 2026-07-10

This document defines the operational procedures, configuration, process slot management, model routing, and verification steps for the SCMessenger Gemini 3.5 Flash Orchestrator. 

[INFO] This orchestrator uses Gemini 3.5 Flash to coordinate and dispatch tasks, Qwen/ollama-cloud models to execute them, and the Windows host to compile, verify, and commit.

---

## 1. Slash Command & CLI Invocation

The orchestrator can be invoked via the `/gemini-orchestrator` slash command or directly from the terminal.

### Slash Command Usage
```
/gemini-orchestrator [task_file | domain_filter | dry-run]
```

### Command Arguments
- **No arguments**: Scans `HANDOFF/todo/_QUEUE.md` and processes the next available non-`[DEVICE]` task.
- **`dry-run`**: Validates the next task, outputs the task details and path, and prepares prompts in the `tmp/gemini-orchestrator/` folder without initiating a worker dispatch.
- **Specific Task File**: Claims and executes a specific task (e.g., `/gemini-orchestrator HANDOFF/todo/P1_ANDROID_mDNS_Self_Loopback_Discovery.md`).
- **Domain Filter**: Filters the queue to a specific domain (e.g., `/gemini-orchestrator rust` or `/gemini-orchestrator android`).

### Direct CLI Invocation
For manual invocation inside Git Bash on the Windows host:
```bash
bash .claude/scripts/gemini-orchestrator-launcher.sh [task-file | domain-filter | dry-run]
```

---

## 2. Configuration Setup

The orchestrator integrates multiple LLM providers: DashScope Qwen, OpenRouter, and Ollama (local/cloud).

### A. DashScope Qwen API
Qwen workers require a valid DashScope API key stored in the user profile directory.
- **Path**: `~/.config/scmorc/dashscope.env`
- **Contents**:
  ```bash
  DASHSCOPE_API_KEY=sk-your_dashscope_api_key_here
  ```
- **Permission**: The launcher script sources this file at runtime to inject credentials.
- **Quota Monitoring**: Model availability, enabled status, and remaining free quota limits are detailed in the [Qwen Model Quota Ledger](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/docs/QWEN_QUOTA_LEDGER.md).

### B. OpenRouter API
Used for Aider/MiMo Code integrations and general fallback models:
- **Required Environment Variables**:
  ```bash
  export OPENROUTER_API_KEY="your_openrouter_key"
  export ANTHROPIC_BASE_URL="https://openrouter.ai/api"
  export ANTHROPIC_AUTH_TOKEN="your_openrouter_key"
  unset ANTHROPIC_API_KEY
  ```
- **Model format**: Prefixed with `openrouter/`, e.g., `openrouter/nex-agi/nex-n2-pro:free`.

### C. Ollama Local and Cloud
Ollama handles local-first executions and tunnels to high-capacity cloud models.
- **Ollama Host**: Configured via `OLLAMA_HOST` (defaults to `localhost:11434`).
- **Local Models (Tier 1)**: Pulled locally. Examples: `scm-coder:7b`, `scm-thinker:14b`, `qwen2.5-coder:7b`.
- **Cloud Models (Tier 2)**: Sourced via Ollama Cloud proxy. These are identified by a `:cloud` suffix (e.g., `glm-5.1:cloud`, `qwen3-coder:480b:cloud`, `deepseek-v3.2:cloud`). The launcher skips local storage checks for `:cloud` suffixes and routes them directly.

---

## 3. Windows Process Slot Calculations

To prevent CPU exhaustion and context locks, the orchestrator implements a strict OS-level process-gating hierarchy.

### Slots and Topology
- **MAX_OS_PROCESSES**: Hard limit of **3** concurrently running `claude.exe` processes on the Windows host.
- **Two-Tier Division**:
  - Tier 1: **1 Lead Orchestrator** process.
  - Tier 2: **Max 2 Workers** (sub-agents) running in parallel.

### Dynamic Slot Formula
At startup, `.claude/orchestrator_manager.sh` calculates the remaining capacity (`MAX_SUBAGENTS`):
```bash
MAX_SUBAGENTS = MAX_OS_PROCESSES - active_claude_exe_count
```
- **1 active `claude.exe`** (just the Orchestrator) -> **2** free worker slots.
- **2 active `claude.exe`** (Orchestrator + 1 worker/interactive session) -> **1** free worker slot.
- **3 active `claude.exe`** -> **0** worker slots (HARDLOCK). Launches are refused until a process terminates.

### CLI Check Command
To inspect current active `claude` processes on Windows:
```powershell
tasklist.exe | findstr /I "claude.exe"
```

---

## 4. Model Fallback Routing Table

The orchestrator selects models dynamically based on task type. If the primary model fails or hits limits, it routes to the fallback model.

| Task Pattern | Primary Model | Fallback Model | Rationale |
|---|---|---|---|
| Mechanical (docs, strings, lint, formatting) | `qwen-plus` | `qwen-turbo` / `gemini-3-flash-preview:cloud` | Low complexity, cost-effective |
| Standard Rust (Core, CLI, WASM features) | `qwen-turbo` | `glm-5.1:cloud` | Speed + strong Rust parsing |
| Kotlin/Android (UI, Compose, Gradle) | `qwen-turbo` | `qwen-max` | High quality Kotlin output |
| Hard Multi-file Refactoring | `glm-5.1:cloud` | `qwen-max` / `qwen3-coder:480b:cloud` | High reasoning and context limits |
| Test Authoring & Property Tests | `qwen-plus` | `qwen-turbo` | Scoped and deterministic |
| Pre-dispatch Validation & Triage | `qwen-plus` | `gemini-3-flash-preview:cloud` | Read-only operations |
| Non-Crypto Security Review | `deepseek-v3.2:cloud` | `deepseek-v4-pro:cloud` | Strong adversarial detection |
| Crypto & Transport Review | `FABLE` (native) | None | MANDATORY. Staged for post-reset |

---

## 5. Host Verification & Commit Workflow

Because foreign workers operate in a sandboxed environment, their output must be compiled and verified on the Windows host before being committed.

### Step 1: Parse the Worker Output
Locate the response in `tmp/gemini-orchestrator/responses/<slug>.response.md`.
- Ensure it starts with `RESULT: DONE`. If it contains `BLOCKED` or `FAILED`, review the explanation and re-queue.

### Step 2: Validate Git Diff
Run `git diff --stat` to verify that code has changed.
- [WARNING] A zero-diff response represents a failed task. Re-queue the task file immediately.

### Step 3: Run the Compilation and Build Gates
Before compiling, you must set the incremental compilation environment variable to prevent Windows process page file locks:
```bash
export CARGO_INCREMENTAL=0
```
Run the target-specific gates:
- **Rust Core / CLI**: `cargo build --workspace` or `cargo check --workspace`
- **Android Target**: `./gradlew assembleDebug`
- **WASM Target**: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`

### Step 4: Staging for Adversarial Security Audit
If the modified files lie within the security envelope:
- Paths: `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, or `core/src/privacy/`
- Action: **Do not consider the task fully completed.**
- Move it to the audit queue file: `tmp/gemini-orchestrator/fable_audit_queue.md`.
- Label it as `[await-fable-audit]`. A native Claude Fable agent must run an adversarial audit on these files before they can be merged.

### Step 5: Finalize and Commit
If the gates pass (and security reviews are staged or passed):
1. Move the task file from `HANDOFF/todo/` to `HANDOFF/done/`:
   ```bash
   mv HANDOFF/todo/task_file.md HANDOFF/done/
   ```
2. Commit the changes:
   ```bash
   git add -A
   git commit -m "swarm: completed [Task Name]"
   ```
3. Log the outcome in `tmp/gemini-orchestrator/dispatch_log.md`.
