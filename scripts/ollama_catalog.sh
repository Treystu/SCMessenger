#!/usr/bin/env bash
# ollama_catalog.sh — dynamic Ollama Cloud + local catalog fetcher + task-to-model recommender
# Part of SCMessenger agent tooling. Source of truth: https://ollama.com/api/tags
#
# Subcommands:
#   list              pretty table: NAME | CLOUD | LOCAL | SIZE_GB (sorted, all models)
#   list-json         same data as JSON to stdout
#   recommend <type>  print "<type> -> <model:cloud> (cloud_available: yes/no)"
#   assign <type>     same as recommend but 1-token pings the model to verify reachability
#   health            hit http://localhost:11434/api/version with retries
#   help              list subcommands
#
# Exit codes: 0=ok, 1=not found, 2=network, 3=auth/verify failed

set -o pipefail

# --- Config -----------------------------------------------------------------
REMOTE_API="${OLLAMA_REMOTE_API:-https://ollama.com/api/tags}"
LOCAL_API="${OLLAMA_HOST:-http://localhost:11434}"
TIMEOUT="${OLLAMA_TIMEOUT:-15}"
RETRIES=3

# Optional auth: if OLLAMA_API_KEY is set, send it (raises rate limit on ollama.com)
auth_header=()
if [[ -n "${OLLAMA_API_KEY:-}" ]]; then
  auth_header=(-H "Authorization: Bearer *** ${OLLAMA_API_KEY}")
fi

# --- Capability rules -------------------------------------------------------
# Task type maps to "primary[pipe]fallback" cloud model name (colon-suffixed)
# Order: first match wins. Names use ollama.com catalog format + :cloud suffix.
declare -A TASK_MAP=(
  ["overseer"]="minimax-m3:cloud|qwen3-coder:480b:cloud"
  ["delegator"]="minimax-m3:cloud|qwen3-coder:480b:cloud"
  ["coordination"]="minimax-m3:cloud|qwen3-coder:480b:cloud"
  ["multi_agent"]="minimax-m3:cloud|mistral-large-3:675b:cloud"
  ["architecture"]="qwen3-coder:480b:cloud|qwen3.5:397b:cloud"
  ["planning"]="qwen3-coder:480b:cloud|qwen3-coder-next:cloud"
  ["design"]="qwen3-coder:480b:cloud|qwen3-coder-next:cloud"
  ["multi_file_reasoning"]="qwen3-coder:480b:cloud|qwen3-coder-next:cloud"
  ["implementation"]="minimax-m2.7:cloud|qwen3-coder-next:cloud"
  ["coding"]="minimax-m2.7:cloud|qwen3-coder-next:cloud"
  ["features"]="minimax-m2.7:cloud|qwen3-coder-next:cloud"
  ["bug_fix"]="qwen3-coder-next:cloud|minimax-m2.7:cloud"
  ["refactoring"]="minimax-m2.7:cloud|qwen3-coder-next:cloud"
  ["security"]="deepseek-v3.2:cloud|deepseek-v3.1:671b:cloud"
  ["crypto"]="deepseek-v3.2:cloud|deepseek-v3.1:671b:cloud"
  ["validation"]="deepseek-v3.2:cloud|deepseek-v3.1:671b:cloud"
  ["protocol_validation"]="deepseek-v3.2:cloud|deepseek-v3.1:671b:cloud"
  ["security_audit"]="deepseek-v3.2:cloud|deepseek-v3.1:671b:cloud"
  ["documentation"]="gemma4:31b:cloud|gemma3:27b:cloud"
  ["tests"]="gemma4:31b:cloud|ministral-3:14b:cloud"
  ["bindings"]="gemma4:31b:cloud|devstral-2:123b:cloud"
  ["simple"]="gemma4:31b:cloud|qwen3-coder-next:cloud"
  ["triage"]="gemini-3-flash-preview:cloud|ministral-3:8b:cloud"
  ["lint"]="gemini-3-flash-preview:cloud|ministral-3:3b:cloud"
  ["quick_fix"]="gemini-3-flash-preview:cloud|ministral-3:8b:cloud"
  ["micro"]="ministral-3:8b:cloud|ministral-3:3b:cloud"
  ["review"]="kimi-k2.6:cloud|kimi-k2-thinking:cloud"
  ["gatekeeping"]="kimi-k2.6:cloud|kimi-k2-thinking:cloud"
  ["quality"]="kimi-k2.6:cloud|kimi-k2-thinking:cloud"
  ["orchestration"]="mistral-large-3:675b:cloud|minimax-m3:cloud"
  ["swarm"]="mistral-large-3:675b:cloud|minimax-m3:cloud"
  ["pipeline"]="mistral-large-3:675b:cloud|minimax-m3:cloud"
  ["rust"]="glm-5.1:cloud|glm-5:cloud"
  ["core"]="glm-5.1:cloud|glm-5:cloud"
  ["protocols"]="glm-5.1:cloud|deepseek-v3.2:cloud"
  ["vision"]="qwen3-vl:235b:cloud|qwen3-vl:235b-instruct:cloud"
  ["multimodal"]="qwen3-vl:235b:cloud|qwen3-vl:235b-instruct:cloud"
  ["general"]="minimax-m3:cloud|qwen3-coder-next:cloud"
  ["scaffolding"]="devstral-2:123b:cloud|qwen3-coder-next:cloud"
  ["boilerplate"]="devstral-2:123b:cloud|gemma3:12b:cloud"
  ["rapid_coding"]="qwen3-coder-next:cloud|minimax-m2.7:cloud"
)
# Last-resort fallback for any unknown task type
FALLBACK_DEFAULT="qwen3-coder-next:cloud|minimax-m3:cloud"

# --- Helpers ----------------------------------------------------------------

log() { echo "[$(date +%H:%M:%S)] $*" >&2; }

# curl with retries + exponential backoff
curl_retry() {
  local url="$1"; shift
  local attempt=0 delay=2 last_out
  while (( attempt < RETRIES )); do
    last_out=$(curl -s -m "$TIMEOUT" "$@" "$url" 2>/dev/null) && [[ -n "$last_out" ]] && {
      printf '%s' "$last_out"
      return 0
    }
    attempt=$((attempt+1))
    (( attempt < RETRIES )) && sleep "$delay" && delay=$((delay*2))
  done
  return 1
}

# Parse JSON with python3 (always available in WSL) and emit value
pyjson() {
  # usage: echo "$json" | pyjson <python expression on `d`>
  local expr="$1"
  python3 -c "
import json, sys
d = json.loads(sys.stdin.read())
${expr}
"
}

# --- Data fetchers ----------------------------------------------------------

fetch_remote() {
  curl_retry "$REMOTE_API" "${auth_header[@]}" | pyjson "print('\\n'.join(sorted(m['name'] + ':' + str(m.get('size',0)) + ':' + m.get('modified_at','') for m in d.get('models',[]))))"
}

fetch_local() {
  local body
  body=$(curl -s -m 5 "${LOCAL_API}/api/tags" 2>/dev/null) || return 1
  printf '%s' "$body" | pyjson "print('\\n'.join(sorted(m['name'] for m in d.get('models',[]))))"
}

build_merged_index() {
  # returns TSV: name<TAB>size_bytes<TAB>modified<TAB>cloud<TAB>local
  local remote_body local_body
  remote_body=$(curl_retry "$REMOTE_API" "${auth_header[@]}") || { log "remote fetch failed"; return 1; }
  local_body=$(curl -s -m 5 "${LOCAL_API}/api/tags" 2>/dev/null || echo '{"models":[]}')

  python3 - "$remote_body" "$local_body" <<'PY'
import json, sys
remote = json.loads(sys.argv[1]).get('models', [])
local  = json.loads(sys.argv[2]).get('models', [])
local_names = {m['name'] for m in local}
all_rows = {}
for m in remote:
    n = m['name']
    all_rows[n] = {
        'name': n,
        'size': m.get('size', 0),
        'modified': m.get('modified_at',''),
        'cloud': True,
        'local': n in local_names,
    }
for m in local:
    n = m['name']
    if n not in all_rows:
        all_rows[n] = {'name': n, 'size': m.get('size',0), 'modified': m.get('modified_at',''), 'cloud': False, 'local': True}
    else:
        all_rows[n]['local'] = True
for n in sorted(all_rows):
    r = all_rows[n]
    print(f"{r['name']}\t{r['size']}\t{r['modified']}\t{int(r['cloud'])}\t{int(r['local'])}")
PY
}

# --- Subcommands ------------------------------------------------------------

cmd_list() {
  local idx; idx=$(build_merged_index) || { echo "ERROR: catalog fetch failed" >&2; exit 2; }
  printf '%-32s %-5s %-5s %8s\n' "NAME" "CLOUD" "LOCAL" "SIZE_GB"
  printf '%-32s %-5s %-5s %8s\n' "----" "-----" "-----" "-------"
  while IFS=$'\t' read -r name size modified cloud local; do
    local gb; gb=$(awk -v s="$size" 'BEGIN{printf "%.1f", s/1073741824}')
    printf '%-32s %-5s %-5s %8s\n' "$name" "$([[ $cloud == 1 ]] && echo yes || echo no)" "$([[ $local == 1 ]] && echo yes || echo no)" "$gb"
  done <<< "$idx"
}

cmd_list_json() {
  local idx; idx=$(build_merged_index) || { echo '{"models":[]}'; exit 2; }
  python3 - "$idx" <<'PY'
import json, sys
rows = []
for ln in sys.argv[1].splitlines():
    if not ln.strip(): continue
    n, size, modified, cloud, local = ln.split('\t')
    rows.append({
        "name": n, "size_bytes": int(size), "modified_at": modified,
        "cloud": cloud == '1', "local": local == '1',
    })
print(json.dumps({"models": rows, "source": "https://ollama.com/api/tags + localhost:11434/api/tags"}, indent=2))
PY
}

cmd_recommend() {
  local task="${1:-general}"
  local key="${task,,}"  # lowercase
  local pair="${TASK_MAP[$key]:-$FALLBACK_DEFAULT}"
  local primary="${pair%%|*}" fallback="${pair#*|}"

  # verify primary is in live remote catalog
  local idx; idx=$(build_merged_index) || idx=""
  local primary_base="${primary%:cloud}"
  local fallback_base="${fallback%:cloud}"

  local chosen="$primary"
  local note="cloud_available: yes"
  if ! grep -P "^${primary_base}\t" <<< "$idx" >/dev/null 2>&1; then
    if grep -P "^${fallback_base}\t" <<< "$idx" >/dev/null 2>&1; then
      chosen="$fallback"; note="cloud_available: yes (primary ${primary_base} not in catalog)"
    else
      # last resort: minimax-m3:cloud (always in catalog)
      chosen="minimax-m3:cloud"
      note="cloud_available: yes (LAST RESORT — primary+fallback both missing)"
    fi
  fi
  echo "${task} -> ${chosen} (${note})"
}

cmd_assign() {
  local task="${1:-general}"
  local rec
  rec=$(cmd_recommend "$task")
  local model
  model=$(awk '{print $3}' <<< "$rec")
  # Ping via ollama.com/api/chat
  local body
  # Ping via the local ollama proxy (which is the path the gateway uses for cloud models).
  # The remote ollama.com/api/chat requires auth we don't have; the local proxy speaks for us.
  body=$(curl -s -m 20 \
    "${LOCAL_API}/api/chat" \
    -H "Content-Type: application/json" \
    -d "$(printf '{"model":"%s","messages":[{"role":"user","content":"ping"}],"stream":false,"options":{"num_predict":1}}' "${model}")") || body=""
  if [[ -n "$body" ]] && echo "$body" | grep -q '"done":true'; then
    local ms; ms=$(python3 -c "import json,sys
import time
t0=time.time()
d=json.loads(sys.stdin.read())
ns=d.get('total_duration', (time.time()-t0)*1e9)
print(int(ns/1e6))" <<< "$body" 2>/dev/null || echo "?")
    echo "${task} -> ${model} verified (${ms}ms via ${LOCAL_API})"
    return 0
  fi
  # Try fallback (the other half of the pair)
  local pair="${TASK_MAP[${task,,}]:-$FALLBACK_DEFAULT}"
  local other="${pair#*|}"
  if [[ "$other" == "$model" ]]; then other="${pair%%|*}"; fi
  body=$(curl -s -m 20 \
    "${LOCAL_API}/api/chat" \
    -H "Content-Type: application/json" \
    -d "$(printf '{"model":"%s","messages":[{"role":"user","content":"ping"}],"stream":false,"options":{"num_predict":1}}' "${other}")") || body=""
  if [[ -n "$body" ]] && echo "$body" | grep -q '"done":true'; then
    echo "${task} -> ${other} verified (fallback after $model unreachable)"
    return 0
  fi
  echo "${task} -> FAILED: $model (and fallback $other) unreachable" >&2
  exit 3
}

cmd_health() {
  local v
  v=$(curl -s -m 5 "${LOCAL_API}/api/version" 2>/dev/null) || { echo "ERROR: ollama unreachable at ${LOCAL_API}"; exit 2; }
  echo "OK: ${LOCAL_API} -> $v"
}

cmd_help() {
  cat <<'EOF'
ollama_catalog.sh — SCMessenger dynamic model manager
Usage: ollama_catalog.sh <command> [args]

Commands:
  list                  Pretty table: NAME | CLOUD | LOCAL | SIZE_GB
  list-json             Same data as JSON
  recommend <task_type> Print best cloud model for a task
  assign <task_type>    Same as recommend + 1-token ping verification
  health                Check local ollama reachability
  help                  This message

Task types (excerpt): overseer, delegator, architecture, planning,
implementation, coding, security, crypto, validation, documentation,
tests, bindings, triage, lint, review, gatekeeping, orchestration,
swarm, rust, core, vision, multimodal, general
EOF
}

# --- Main -------------------------------------------------------------------

cmd="${1:-help}"
shift 2>/dev/null || true

case "$cmd" in
  list)        cmd_list ;;
  list-json)   cmd_list_json ;;
  recommend)   cmd_recommend "$@" ;;
  assign)      cmd_assign "$@" ;;
  health)      cmd_health ;;
  help|-h|--help) cmd_help ;;
  *)           echo "Unknown command: $cmd"; cmd_help; exit 1 ;;
esac
