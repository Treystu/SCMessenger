#!/usr/bin/env bash
# Gemini 3.5 Flash Orchestrator Launcher
# Bootstraps the orchestration loop by reading HANDOFF backlog and dispatching to Qwen/ollama workers
# Usage: bash .claude/scripts/gemini-orchestrator-launcher.sh [task-file|domain|dry-run]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ORCHESTRATOR_DIR="$REPO_ROOT/tmp/gemini-orchestrator"
DISPATCH_LOG="$ORCHESTRATOR_DIR/dispatch_log.md"
PROMPT_DIR="$ORCHESTRATOR_DIR/prompts"
RESPONSE_DIR="$ORCHESTRATOR_DIR/responses"

# Ensure directories exist
mkdir -p "$ORCHESTRATOR_DIR" "$PROMPT_DIR" "$RESPONSE_DIR"

# Initialize dispatch log if needed
if [[ ! -f "$DISPATCH_LOG" ]]; then
  cat > "$DISPATCH_LOG" <<'EOF'
# Gemini Orchestrator Dispatch Log

Status: Active
Started: $(date -u +%Y-%m-%dT%H:%M:%SZ)

## Dispatch Summary

| Timestamp | Model | Task File | Result |
|-----------|-------|-----------|--------|

EOF
fi

# Validate prerequisites
check_prerequisites() {
  echo "[INFO] Checking Gemini orchestrator prerequisites..."

  # Check Qwen API key
  if [[ ! -f ~/.config/scmorc/dashscope.env ]]; then
    echo "[WARNING] ~/.config/scmorc/dashscope.env not found — Qwen dispatch will fail"
    echo "[INFO] Set DASHSCOPE_API_KEY in ~/.config/scmorc/dashscope.env to enable Qwen workers"
  fi

  # Check HANDOFF directory
  if [[ ! -f "$REPO_ROOT/HANDOFF/todo/_QUEUE.md" ]]; then
    echo "[ERROR] $REPO_ROOT/HANDOFF/todo/_QUEUE.md not found"
    exit 1
  fi

  # Check ollama-cloud status (advisory only)
  echo "[INFO] Checking ollama-cloud model availability..."
  curl -s https://ollama.com/api/tags | grep -q "qwen3-coder:480b:cloud" && \
    echo "[OK] ollama-cloud is available" || \
    echo "[WARNING] ollama-cloud may be offline"
}

# Read the next task from _QUEUE.md
pick_next_task() {
  local domain_filter="${1:-}"
  local queue_file="$REPO_ROOT/HANDOFF/todo/_QUEUE.md"

  if [[ -z "$domain_filter" ]]; then
    # Pick first todo/ file in _QUEUE.md that is not [DEVICE]
    grep -E '^\s+[0-9]+\.' "$queue_file" | \
      grep -v '\[DEVICE\]' | \
      head -1 | \
      sed 's/.*`\(HANDOFF\/[^`]*\)`.*/\1/'
  else
    # Pick first task in domain filter
    grep -E "^\s+[0-9]+\." "$queue_file" | \
      grep -i "$domain_filter" | \
      grep -v '\[DEVICE\]' | \
      head -1 | \
      sed 's/.*`\(HANDOFF\/[^`]*\)`.*/\1/'
  fi
}

# Pre-dispatch validation
validate_task() {
  local task_file="$1"

  if [[ ! -f "$REPO_ROOT/$task_file" ]]; then
    echo "[ERROR] Task file not found: $task_file"
    return 1
  fi

  # Extract target and grep for it
  local target=$(grep -E '^\s+TARGET' "$REPO_ROOT/$task_file" | head -1 | sed 's/.*TARGET:\s*//' | tr -d ' ')

  if [[ -z "$target" ]]; then
    echo "[WARNING] No TARGET line found in $task_file — assuming valid"
    return 0
  fi

  # Check if target is a test (FALSE_POSITIVE)
  if echo "$target" | grep -qE '(test_|Test|_test\.rs|proptest|kani)'; then
    echo "[INFO] Target '$target' appears to be test/scaffolding — marking as done"
    return 2  # Special code for FALSE_POSITIVE
  fi

  # Check if target already has callers (ALREADY_WIRED)
  if rg -q "$target" "$REPO_ROOT/core/src" "$REPO_ROOT/cli/src" "$REPO_ROOT/android/app/src" 2>/dev/null; then
    echo "[INFO] Target '$target' already has callers — marking as done"
    return 3  # Special code for ALREADY_WIRED
  fi

  echo "[OK] Task $task_file validated"
  return 0
}

# Generate worker prompt
generate_prompt() {
  local task_file="$1"
  local slug=$(basename "$task_file" .md)
  local prompt_file="$PROMPT_DIR/${slug}.prompt.md"

  cat > "$prompt_file" <<'PROMPT_HEADER'
You are a foreign worker for the SCMessenger project (AGENTS.md "FOREIGN WORKER" class).

CRITICAL CONSTRAINTS:
- Do NOT run `cargo`/`gradlew` — Windows host serializes all builds.
- Do NOT commit, push, or move HANDOFF files.
- Do NOT run `git` commands except `git diff`.
- Locate code with Grep; read only ~20-40 lines you need.
- No emojis. Use [OK], [ERROR], [WARNING], [INFO], [DONE], [FAIL].

REPORT FORMAT (final output, nothing after):
Line 1: RESULT: DONE|BLOCKED|FAILED
Line 2: FILES: <comma-separated paths>
Then max 8 lines: what changed, risks, Windows verification notes.

PROMPT_HEADER

  # Append task content
  cat "$REPO_ROOT/$task_file" >> "$prompt_file"

  echo "$prompt_file"
}

# Dispatch to Qwen (DashScope)
dispatch_qwen() {
  local prompt_file="$1"
  local slug=$(basename "$prompt_file" .prompt.md)
  local response_file="$RESPONSE_DIR/${slug}.response.md"

  if [[ ! -f ~/.config/scmorc/dashscope.env ]]; then
    echo "[ERROR] Qwen dispatch requires ~/.config/scmorc/dashscope.env"
    return 1
  fi

  source ~/.config/scmorc/dashscope.env

  local prompt_json=$(sed 's/"/\\"/g; s/$/\\n/g' "$prompt_file" | tr -d '\n')

  echo "[INFO] Dispatching to Qwen (qwen-turbo)..."

  curl -X POST https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions \
    -H "Authorization: Bearer $DASHSCOPE_API_KEY" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"qwen-turbo\", \"messages\": [{\"role\": \"user\", \"content\": \"$prompt_json\"}]}" \
    > "$response_file" 2>&1 || {
    echo "[ERROR] Qwen dispatch failed"
    return 1
  }

  echo "[OK] Qwen response saved to $response_file"
}

# Log dispatch
log_dispatch() {
  local model="$1"
  local task_file="$2"
  local result="$3"

  echo "| $(date -u +%Y-%m-%dT%H:%M:%SZ) | $model | $task_file | $result |" >> "$DISPATCH_LOG"
}

# Main orchestrator loop
run_orchestration() {
  local args="${1:-}"

  echo "[INFO] SCMessenger Gemini 3.5 Flash Orchestrator"
  echo "[INFO] Repo: $REPO_ROOT"
  echo "[INFO] Work dir: $ORCHESTRATOR_DIR"

  check_prerequisites

  if [[ "$args" == "dry-run" ]]; then
    echo "[INFO] Dry-run mode: validation only, no dispatch"
    local task_file=$(pick_next_task)
    validate_task "$REPO_ROOT/$task_file"
    echo "[INFO] Next task: $task_file"
    return 0
  fi

  # Pick task (from args or top of queue)
  local task_file="$args"
  if [[ -z "$task_file" ]]; then
    task_file=$(pick_next_task)
  fi

  if [[ -z "$task_file" ]]; then
    echo "[ERROR] No tasks found in queue"
    return 1
  fi

  echo "[INFO] Processing task: $task_file"

  # Validate
  validate_task "$task_file"
  local validation_result=$?

  if [[ $validation_result -eq 2 ]]; then
    echo "[INFO] Marking $task_file as false-positive (test scaffolding)"
    mv "$REPO_ROOT/$task_file" "$REPO_ROOT/HANDOFF/done/$(basename "$task_file")"
    log_dispatch "validation" "$task_file" "false-positive"
    return 0
  elif [[ $validation_result -eq 3 ]]; then
    echo "[INFO] Marking $task_file as already-wired"
    mv "$REPO_ROOT/$task_file" "$REPO_ROOT/HANDOFF/done/$(basename "$task_file")"
    log_dispatch "validation" "$task_file" "already-wired"
    return 0
  elif [[ $validation_result -ne 0 ]]; then
    echo "[ERROR] Task validation failed"
    return 1
  fi

  # Generate prompt
  local prompt_file=$(generate_prompt "$task_file")
  echo "[OK] Prompt generated: $prompt_file"

  # Dispatch to Qwen
  if dispatch_qwen "$prompt_file"; then
    log_dispatch "qwen-turbo" "$task_file" "pending"
    echo "[INFO] Dispatch logged. Worker response available."
  else
    log_dispatch "qwen-turbo" "$task_file" "failed"
    echo "[ERROR] Dispatch failed"
    return 1
  fi
}

# Parse args and run
main() {
  local args="${@:-}"
  run_orchestration "$args"
}

main "$@"
