#!/bin/bash
# enforce_local_priority.sh — Patches agent_pool.json local-first at runtime
# Called by orchestrator_manager.sh before each pool_launch
#
# For each agent in the pool:
#   - If a local model with equivalent capability exists, set as primary
#   - Cloud model becomes fallback
#   - Preserves original model as secondary fallback

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
POOL_CONFIG="$REPO_ROOT/.claude/agent_pool.json"

if [ ! -f "$POOL_CONFIG" ]; then
    echo "WARN: agent_pool.json not found, skipping local-first patch" >&2
    exit 0
fi

# Determine python
PYTHON="${PYTHON:-python3}"

# List local models
LOCAL_MODELS=$(ollama list 2>/dev/null | awk 'NR>1 && $1 != "NAME" {print $1}' | sort -u || echo "")

# Patch logic: for each agent, check if a local model matches its specialization
# If so, prepend local model as primary, keep cloud as fallback
$PYTHON - "$POOL_CONFIG" "$LOCAL_MODELS" <<'PYEOF'
import json, sys

pool_file = sys.argv[1]
local_models = set(sys.argv[2].strip().splitlines()) if len(sys.argv) > 2 else set()

with open(pool_file) as f:
    cfg = json.load(f)

# Map: cloud model base name -> agent role
CLOUD_TO_LOCAL = {
    # Architecture/planning -> local thinker
    "qwen3-coder:480b:cloud": "scm-thinker:14b",
    "deepseek-v4-pro:cloud": "deepseek-r1-distill-14b-iq2xs:latest",
    "qwen3.5:397b:cloud": "scm-thinker:14b",
    "qwen3-coder-next:cloud": "scm-coder:7b",
    # Implementation -> local coder
    "glm-5.1:cloud": "scm-coder:7b",
    "minimax-m2.7:cloud": "scm-coder:7b",
    "qwen3-coder-next:cloud": "scm-coder:7b",
    # Security -> local reasoner
    "deepseek-v3.2:cloud": "deepseek-r1-distill-14b-iq2xs:latest",
    "deepseek-v4-pro:cloud": "deepseek-r1-distill-14b-iq2xs:latest",
    "deepseek-v3.1:671b:cloud": "deepseek-r1-distill-14b-iq2xs:latest",
    # Review -> local thinker
    "kimi-k2-thinking:cloud": "scm-thinker:14b",
    "kimi-k2.6:cloud": "scm-thinker:14b",
    # Quick/triage -> local small
    "gemini-3-flash-preview:cloud": "qwen2.5-coder:1.5b",
    "ministral-3:8b:cloud": "qwen2.5-coder:1.5b",
    "ministral-3:3b:cloud": "llama3.2:3b",
    # Docs/tests -> local small
    "gemma4:31b:cloud": "qwen2.5-coder:7b",
    "devstral-2:123b:cloud": "qwen2.5-coder:7b",
    # Orchestration -> local thinker
    "mistral-large-3:675b:cloud": "scm-thinker:14b",
    "minimax-m3:cloud": "scm-thinker:14b",
}

patched = 0
for agent in cfg.get("agents", []):
    if agent.get("launch_type") != "cli":
        continue
    orig_model = agent.get("model", "")
    # Strip :cloud suffix for matching
    cloud_base = orig_model.replace(":cloud", "")
    
    # Check if we have a local replacement
    local_replacement = None
    for cloud_pattern, local_model in CLOUD_TO_LOCAL.items():
        if orig_model == cloud_pattern or cloud_base in cloud_pattern:
            if local_model in local_models:
                local_replacement = local_model
                break
    
    if local_replacement and local_replacement != orig_model:
        # Set local as primary, keep original cloud as fallback
        old_model = agent["model"]
        old_fallback = agent.get("fallback_model", "")
        
        agent["model"] = local_replacement
        agent["fallback_model"] = old_fallback if old_fallback else old_model
        agent["_local_first_patched"] = True
        patched += 1
        print(f"  PATCHED: {agent['name']}: {old_model} → {local_replacement} (fallback: {agent['fallback_model']})")

if patched > 0:
    with open(pool_file, "w") as f:
        json.dump(cfg, f, indent=2, default=str)
    print(f"  Total agents patched: {patched}")
else:
    print("  No patches needed — all agents already local-first or no match found")
PYEOF
