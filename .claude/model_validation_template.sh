#!/bin/bash
# Model Validation Template for Agent Launch
# Returns ONLY the model name on stdout; all status messages go to stderr.

MODEL_API="https://ollama.com/api/tags"

validate_model_before_launch() {
    local model="$1"
    local agent_type="$2"

    # Ensure cloud suffix
    if [[ "$model" != *":cloud" ]]; then
        model="${model}:cloud"
    fi

    # Check model availability
    base_model="${model%:cloud}"
    if ! curl -s "$MODEL_API" | grep -q "\"$base_model\""; then
        echo "Model $model not available via Ollama Cloud, using fallback" >&2

        # Fallback logic based on agent type
        case "$agent_type" in
            "architect") fallback="qwen3-coder-next:cloud" ;;
            "implementer") fallback="deepseek-v3.2:cloud" ;;
            "precision_validator") fallback="qwen3-coder:480b:cloud" ;;
            "worker") fallback="gemini-3-flash-preview:cloud" ;;
            "triage_router") fallback="qwen3-coder-next:cloud" ;;
            "gatekeeper_reviewer") fallback="deepseek-v3.2:cloud" ;;
            "swarm_orchestrator") fallback="mistral-large-3:675b:cloud" ;;
            "rust_coder") fallback="qwen3-coder-next:cloud" ;;
            *) fallback="qwen3-coder-next:cloud" ;;
        esac

        echo "Falling back to: $fallback" >&2
        model="$fallback"
    else
        echo "Model $model verified available" >&2
    fi

    # Only the model name goes to stdout — caller captures this line only
    echo "$model"
}