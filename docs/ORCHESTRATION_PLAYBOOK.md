# SCMessenger Orchestration Playbook (Phase 2)

This playbook outlines the exact commands and workflow for dispatching the Phase 2 post-quantum tasks (PQC-07 through PQC-14) using the external model fleet (Qwen, OpenRouter, Ollama). 

We use the universal `scripts/delegate_task.py` tool to coordinate task delegation without automated token loops.

## Prerequisites
- **Qwen API Key**: Exported as `QWEN_API_KEY` (or uses the fallback hardcoded test key if un-exported).
- **OpenRouter API Key**: Exported as `OPENROUTER_API_KEY` (if using OpenRouter).
- **Ollama**: Local Ollama server running on `http://localhost:11434` (if using Ollama).
- **Python 3**: Available as `python` or `python3` with standard libraries.

## 1. Pick the Next Task
Look at `HANDOFF/todo/_QUEUE.md` for the next task.
For example: `PQC_07_PQ_RATCHET.md`

Identify the relevant files for the task. For PQC-07, these are `core/src/crypto/ratchet.rs` and `core/tests/integration_pq_session.rs`.

## 2. Dispatch Task to Worker
Run the delegation script for your desired provider. The script will automatically read the task file, append the current contents of the source files, and query the model.

### Using Qwen (Primary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/PQC_07_PQ_RATCHET.md \
  --provider qwen \
  --model qwen-max \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs
```

### Using OpenRouter (Secondary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/PQC_07_PQ_RATCHET.md \
  --provider openrouter \
  --model anthropic/claude-3.5-sonnet \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs
```

### Using Ollama (Tertiary)
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/PQC_07_PQ_RATCHET.md \
  --provider ollama \
  --model qwen2.5:72b \
  --files core/src/crypto/ratchet.rs core/tests/integration_pq_session.rs
```

## 3. Review Generated Code
By default, the script will output the model's raw response to the `tmp/` folder (e.g., `tmp/PQC_07_PQ_RATCHET_response.md`).

Review the generated code in `tmp/` to ensure it is correct and preserves existing logic. 

If you are confident in the model's output, you can run the script with the `--apply` flag to automatically write the code blocks directly to the source files:
```bash
python scripts/delegate_task.py --task HANDOFF/todo/PQC_07_PQ_RATCHET.md --provider qwen --model qwen-max --files core/src/crypto/ratchet.rs --apply
```

## 4. Run CI Verification Gates
After the code is applied, run the Windows build verification gates:
```bash
export CARGO_INCREMENTAL=0
cargo check --workspace
cargo test -p scmessenger-core --test integration_pq_session
scripts/docs_sync_check.sh
```

## 5. Audit & Finalize
- Inspect `git diff` for any anomalies.
- Seek adversarial review if the task modifies `core/src/crypto/`.
- If successful, move the task to done:
```bash
mv HANDOFF/todo/PQC_07_PQ_RATCHET.md HANDOFF/done/
```
- Update `HANDOFF/todo/_QUEUE.md` to mark the task as `COMPLETE`.
- Commit the changes locally:
```bash
git add -A
git commit -m "swarm: completed PQC-07 via Qwen"
```
