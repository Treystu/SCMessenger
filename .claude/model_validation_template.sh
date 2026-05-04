#!/bin/bash
# Model Validation Template for Agent Launch (v3.5)
# Returns ONLY the model name on stdout; all status messages go to stderr.
# v3.5: Robust JSON parsing + Dynamic Model Discovery

MODEL_API="https://ollama.com/api/tags"

# Use Python for robust JSON parsing
validate_model_before_launch() {
    local requested_model="$1"
    local agent_type="$2"

    # Ensure cloud suffix for the check
    local check_model="$requested_model"
    if [[ "$check_model" != *":cloud" ]]; then
        check_model="${check_model}:cloud"
    fi

    local base_model="${check_model%:cloud}"

    echo "Checking model $check_model availability at $MODEL_API..." >&2

    # Fetch latest tags
    local api_response
    api_response=$(curl -s "$MODEL_API")

    if [ -z "$api_response" ]; then
        echo "Error: Could not reach Ollama Cloud API. Using requested model $check_model blindly." >&2
        echo "$check_model"
        return 0
    fi

    # Detect Python
    local python_cmd=""
    if command -v python3 &>/dev/null; then python_cmd="python3"; elif command -v python &>/dev/null; then python_cmd="python"; fi

    if [ -n "$python_cmd" ]; then
        # Robust check using Python
        local result
        result=$($python_cmd -c "
import json, sys
try:
    data = json.loads(sys.argv[1])
    models = [m['name'] for m in data.get('models', [])]
    requested = sys.argv[2]
    base = requested.split(':')[0]
    
    if requested in models:
        print(f'EXACT_MATCH:{requested}')
    elif base in models:
        print(f'BASE_MATCH:{base}:cloud')
    else:
        # Discovery: Look for better versions in the same family
        family = base.split('-')[0]
        better = [m for m in models if m.startswith(family) and m > base]
        if better:
            print(f'DISCOVERY:{max(better)}:cloud')
        else:
            print('MISSING')
except Exception as e:
    print('ERROR')
" "$api_response" "$base_model")

        case "$result" in
            EXACT_MATCH:*)
                model="${result#EXACT_MATCH:}"
                echo "Model $model verified available." >&2
                ;;
            BASE_MATCH:*)
                model="${result#BASE_MATCH:}"
                echo "Model $base_model found (matched without version). Using $model." >&2
                ;;
            DISCOVERY:*)
                discovered="${result#DISCOVERY:}"
                echo "NOTE: $base_model not found, but newer/better version discovered: $discovered" >&2
                echo "Auto-upgrading to $discovered" >&2
                model="$discovered"
                ;;
            *)
                echo "Model $requested_model not available via Ollama Cloud, using fallback." >&2
                
                # Fallback logic based on agent type
                case "$agent_type" in
                    "architect") fallback="qwen3-coder:480b:cloud" ;;
                    "implementer") fallback="qwen3-coder-next:cloud" ;;
                    "precision_validator") fallback="deepseek-v3.2:cloud" ;;
                    "worker") fallback="gemma4:31b:cloud" ;;
                    "triage_router") fallback="gemini-3-flash-preview:cloud" ;;
                    "gatekeeper_reviewer") fallback="kimi-k2-thinking:cloud" ;;
                    "swarm_orchestrator") fallback="mistral-large-3:675b:cloud" ;;
                    "rust_coder") fallback="glm-5.1:cloud" ;;
                    *) fallback="qwen3-coder-next:cloud" ;;
                esac

                echo "Falling back to: $fallback" >&2
                model="$fallback"
                ;;
        esac
    else
        # Fallback to grep if Python is missing
        if echo "$api_response" | grep -q "\"$base_model\""; then
            echo "Model $check_model verified available (via grep)." >&2
            model="$check_model"
        else
            echo "Model $requested_model not available (via grep), using fallback." >&2
            model="qwen3-coder-next:cloud" # Safe default
        fi
    fi

    # Only the final model name goes to stdout
    echo "$model"
}