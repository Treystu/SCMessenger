# SCMessenger Orchestration Playbook (Quick Reference)

Subordinate to docs/ORCHESTRATION.md (master protocol) and docs/GEMINI_ORCHESTRATOR.md (role protocol). This file is the quick command reference.

This playbook outlines the exact commands and workflow for dispatching tasks using the external model fleet (Qwen, OpenRouter, Ollama). We use the universal `scripts/delegate_task.py` tool to coordinate task delegation with verification loops and tier routing.

## Prerequisites
- **Qwen API Key**: Exported as `QWEN_API_KEY`/`DASHSCOPE_API_KEY`, or read automatically from `~/.config/scmorc/dashscope.env`. Never hardcode keys.
- **OpenRouter API Key**: Exported as `OPENROUTER_API_KEY`, or read automatically from `~/.config/scmorc/openrouter.env`.
- **Ollama**: Local Ollama server running on `http://localhost:11434` (if using Ollama).
- **Python 3**: Available as `python` or `python3` with standard libraries.

## 1. Pick the Next Task
Look at `HANDOFF/todo/_QUEUE.md` for the next task.
For example: `<TASK>.md`

Identify the relevant files for the task. For example: `core/src/crypto/ratchet.rs` and `core/tests/integration_pq_session.rs`.

## 2. Dispatch Task to Worker
Run the delegation script for your desired provider. The script will automatically read the task file, append the current contents of the source files, and query the model.

### Using Qwen (Primary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider qwen \
  --tier max \
  --model qwen3-max \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs \
  --verify "cargo check -p scmessenger-core" \
  --max-rounds 3
```

### Using OpenRouter (Secondary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider openrouter \
  --model nvidia/nemotron-3-super-120b-a12b:free \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs \
  --verify "cargo check -p scmessenger-core" \
  --max-rounds 3
```

### Using Ollama (Tertiary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider ollama \
  --model qwen2.5:72b \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs \
  --verify "cargo check -p scmessenger-core" \
  --max-rounds 3
```

## 3. Review Generated Code
By default, the script will output the model's raw response to the `tmp/` folder (e.g., `tmp/<TASK>_response.md`).

Review the generated code in `tmp/` to ensure it is correct and preserves existing logic. 

If you are confident in the model's output, you can run the script with the `--apply` flag to automatically write the code blocks directly to the source files:
```bash
python scripts/delegate_task.py --task HANDOFF/todo/<TASK>.md --provider qwen --tier max --model qwen3-max --files core/src/crypto/ratchet.rs --apply --verify "cargo check -p scmessenger-core" --max-rounds 3
```

## 4. Run CI Verification Gates
After the code is applied, run the Windows build verification gates:

On Linux/macOS:
```bash
export CARGO_INCREMENTAL=0
cargo check --workspace
cargo test -p scmessenger-core --test integration_pq_session
scripts/docs_sync_check.sh
```

On Windows PowerShell:
```powershell
$env:CARGO_INCREMENTAL="0"; $env:PATH += ";C:\Users\SCM\.cargo\bin"
cargo check --workspace
cargo test -p scmessenger-core --test integration_pq_session
scripts/docs_sync_check.sh
```

## 5. Audit & Finalize
- Inspect `git diff` for any anomalies.
- Seek adversarial review if the task modifies `core/src/crypto/`.
- If successful, move the task to done:
```bash
mv HANDOFF/todo/<TASK>.md HANDOFF/done/
```
- Update `HANDOFF/todo/_QUEUE.md` to mark the task COMPLETE (same change set
  as the file move -- atomicity rule, see docs/ORCHESTRATION.md Section 1).
- Do NOT run `git commit` or `git push`. Gemini-family orchestrators report
  only (AGENTS.md); a Claude session or the operator makes the commit.
