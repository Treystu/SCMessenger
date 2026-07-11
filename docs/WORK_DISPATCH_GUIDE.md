# SCMessenger Unified Work Dispatch & Orchestration Playbook

Status: Active
Last updated: 2026-07-10

This playbook defines the exact commands, model mappings, and configuration targets for executing and verifying work across all orchestration methods in SCMessenger.

---

## 1. Method 1: Gemini 3.5 Flash + DashScope Qwen (`gemini-orchestrator`)

Use this method to drive standard implementation and mechanical tasks without burning native Anthropic subscription quota.

### Configuration
Requires a valid API key and custom workspace-specific endpoint defined in:
`~/.config/scmorc/dashscope.env`

Format:
```bash
DASHSCOPE_API_KEY=sk-...
DASHSCOPE_OPENAI_BASE=https://...maas.aliyuncs.com/compatible-mode/v1
```

### Command Execution
All commands are executed from the repository root via Git Bash on Windows:

*   **Dry-Run (Validate and stage next task without dispatching):**
    ```bash
    "C:\Program Files\Git\bin\bash.exe" .claude/scripts/gemini-orchestrator-launcher.sh dry-run
    ```

*   **Dispatch Next Queue Task:**
    ```bash
    "C:\Program Files\Git\bin\bash.exe" .claude/scripts/gemini-orchestrator-launcher.sh
    ```

*   **Dispatch a Specific Task File:**
    ```bash
    "C:\Program Files\Git\bin\bash.exe" .claude/scripts/gemini-orchestrator-launcher.sh HANDOFF/todo/P1-19_Phase_1_Exit_Review.md
    ```

*   **Dispatch in Domain-Filtered Mode:**
    ```bash
    "C:\Program Files\Git\bin\bash.exe" .claude/scripts/gemini-orchestrator-launcher.sh rust
    ```

---

## 2. Method 2: Qwen-Native Orchestration (`/scmqwen`)

Use this method to directly run individual tasks using DashScope models via command-line prompts.

### Command Execution
```bash
"C:\Program Files\Git\bin\bash.exe" tmp/scmorc/qwen.sh <model> <prompt-file>
```

### Rotation Roster
Refer to the [Qwen Model Quota Ledger](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/docs/QWEN_QUOTA_LEDGER.md) for exact quota tracking.

*   **[FLASH] Rotation:** `qwen3-coder-flash` -> `qwen3.5-flash`
    *   *Usage:* Mechanical fixes, strings.xml, header updates, linting.
*   **[CODER] Rotation:** `qwen3-coder-plus` -> `qwen3-coder-plus-2025-09-23` -> `qwen3-coder-plus-2025-07-22`
    *   *Usage:* General code changes, Rust/Kotlin features.
*   **[THINK] Rotation:** `qwen3-235b-a22b-thinking-2507` -> `qwen3.5-122b-a10b` -> `qwen3-max`
    *   *Usage:* Deeper analysis, root-cause diagnostics, adversarial security reviews.
*   **[MAX] Rotation:** `qwen3-max` -> `qwen-max` -> `qwen3-max-preview`
    *   *Usage:* Structural blocks, design deadlocks.

---

## 3. Method 3: Native Claude Code Orchestration (`/scmorc` & `/scm`)

Use this method for final compilation checking, complex logic verification, and fable-level adversarial security audits. Runs on the local Windows host subscription.

### Command Execution
```powershell
# From PowerShell on Windows host
claude --model <model> --effort <effort> -p "your prompt text or path/to/task.md"
```

### Models & Aliases
*   `haiku` (claude-haiku-4-5) - Cheapest, best for triage.
*   `sonnet` (claude-sonnet-5) - Default capability for standard implementation.
*   `opus` (claude-opus-4-8) - Deep multi-file logic.
*   `fable` (claude-fable-5) - Specialized model for cryptography and protocol verification.

---

## 4. Method 4: Swarm Agent Pool (`/orchestrate`)

Use this method for concurrent swarm tasks mapped to local/cloud Ollama instances.

### Command Execution
```bash
"C:\Program Files\Git\bin\bash.exe" .claude/orchestrator_manager.sh pool launch <model> <task-file>
```

### Quota governor tiers
*   **TIER 1 (Vanguard):** Quota <= 25% (All models enabled).
*   **TIER 2 (Execute):** Quota <= 50% (Cloud models for complex tasks only).
*   **TIER 3 (Mixed):** Quota <= 75% (Cloud reserved for critical P0).
*   **TIER 4 (Light):** Quota <= 90% (Local models only, small scope).
*   **TIER 5 (Micro):** Quota <= 99.5% (Local models only, P0 compile fixes).
*   **TIER 6 (Hardlock):** Quota > 99.5% (All Cloud dispatches refused).

---

## 5. Post-Completion Host Verification & Commit Protocol

When a worker completes a task and returns `RESULT: DONE`, the operator (or orchestrator) must execute the following verification steps on the Windows host:

1.  **Reconcile Changes:**
    Verify the changes applied using Git.
    ```bash
    git status
    git diff --stat
    ```
    *Note: Zero-diff reports represent a failure. Re-queue the task.*

2.  **Run Build Gates:**
    Prevent incremental compilation lock issues:
    ```bash
    export CARGO_INCREMENTAL=0
    ```
    Execute target compilation:
    ```bash
    # Rust Core / CLI Crate
    cargo check --workspace
    
    # Android Kotlin
    cd android && ./gradlew assembleDebug
    ```

3.  **Audit Gate Handling:**
    If changes affect `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, or `core/src/privacy/`:
    - Move task details to `tmp/gemini-orchestrator/fable_audit_queue.md`.
    - Mark as `[await-fable-audit]`. Do not merge until reviewed by a native `fable` agent.

4.  **Finalize Task & Commit:**
    If all verification checks pass:
    ```bash
    # Move task file to done/
    mv HANDOFF/todo/task_name.md HANDOFF/done/
    
    # Commit changes
    git add -A
    git commit -m "swarm: completed [Task Name]"
    ```
