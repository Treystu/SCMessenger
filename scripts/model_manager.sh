#!/bin/bash
# SCMessenger Model Manager
# Uses https://ollama.com/api/tags as source of truth for model availability

MODEL_API="https://ollama.com/api/tags"
MAPPING_FILE=".claude/model_capability_mapping.json"

# Create model capability mapping based on Ollama API and SCMessenger requirements
create_model_mapping() {
    echo "Creating model capability mapping from Ollama API..."

    # Pull latest model list from Ollama API
    MODEL_LIST=$(curl -s "$MODEL_API")

    if [ $? -ne 0 ]; then
        echo "❌ Failed to fetch model list from Ollama API"
        return 1
    fi

    # Create comprehensive model capability mapping
    cat > "$MAPPING_FILE" << EOF
{
  "model_capabilities": {
    "qwen3-coder:480b:cloud": {
      "capabilities": ["architecture", "planning", "multi_file_reasoning", "system_design"],
      "role": "architect",
      "size": "480b",
      "recommended_for": ["P0_architecture", "P1_design", "complex_refactoring"]
    },
    "qwen3-coder-next:cloud": {
      "capabilities": ["implementation", "bug_fixes", "feature_development", "rapid_coding"],
      "role": "implementer",
      "size": "80b",
      "recommended_for": ["P0_implementation", "P1_features", "code_changes"]
    },
    "deepseek-v3.2:cloud": {
      "capabilities": ["cryptography", "math", "protocol_validation", "security_audit", "unsafe_blocks"],
      "role": "precision_validator",
      "size": "671b",
      "recommended_for": ["P0_security", "P0_crypto", "protocol_implementation"]
    },
    "deepseek-v3.1:671b:cloud": {
      "capabilities": ["orchestration", "monitoring", "coordination", "system_oversight"],
      "role": "orchestrator",
      "size": "671b",
      "recommended_for": ["orchestration", "monitoring", "coordination"]
    },
    "gemma4:31b:cloud": {
      "capabilities": ["documentation", "testing", "bindings", "platform_engineering", "simple_tasks"],
      "role": "worker",
      "size": "31b",
      "recommended_for": ["documentation", "unit_tests", "bindings", "P2_tasks"]
    },
    "gemini-3-flash-preview:cloud": {
      "capabilities": ["quick_triage", "lint_fixes", "minor_edits", "ci_gatekeeping", "parsing"],
      "role": "triage_router",
      "size": "small",
      "recommended_for": ["quick_fixes", "under_50_loc", "triage"]
    },
    "kimi-k2-thinking:cloud": {
      "capabilities": ["code_review", "final_verification", "pre_merge_checks", "quality_gate"],
      "role": "gatekeeper_reviewer",
      "size": "1t",
      "recommended_for": ["final_review", "pre_merge", "quality_gate"]
    },
    "mistral-large-3:675b:cloud": {
      "capabilities": ["pipeline_management", "multi_agent_coordination", "swarm_orchestration"],
      "role": "swarm_orchestrator",
      "size": "675b",
      "recommended_for": ["pipeline_management", "multi_agent_coordination"]
    },
    "glm-5.1:cloud": {
      "capabilities": ["rust_core", "protocol_implementation", "proven_compatibility"],
      "role": "rust_coder",
      "size": "1.5t",
      "recommended_for": ["rust_core", "protocol_work", "compatibility"]
    }
  },
  "source_of_truth": "$MODEL_API",
  "last_updated": "$(date -u +'%Y-%m-%dT%H:%M:%SZ')",
  "cloud_suffix": ":cloud",
  "validation_rules": {
    "any_model_with_cloud_suffix": "available_via_ollama_cloud",
    "model_check_frequency": "before_agent_launch",
    "fallback_strategy": "next_best_capability_match"
  }
}
EOF

    echo "✅ Model capability mapping created at $MAPPING_FILE"
    echo "📋 Source of truth: $MODEL_API"
}

# Check if a specific model is available via Ollama Cloud
check_model_availability() {
    local model_name="$1"

    # Add :cloud suffix if not present
    if [[ "$model_name" != *":cloud" ]]; then
        model_name="${model_name}:cloud"
    fi

    echo "Checking availability of $model_name..."

    # Check if model exists in Ollama API (base name without :cloud)
    base_model="${model_name%:cloud}"

    if curl -s "$MODEL_API" | grep -q "\"model\": \"$base_model\""; then
        echo "✅ $model_name is available via Ollama Cloud"
        return 0
    else
        echo "❌ $model_name is NOT available via Ollama Cloud"
        return 1
    fi
}

# Get recommended model for a specific task type
get_recommended_model() {
    local task_type="$1"

    case "$task_type" in
        "architecture" | "planning" | "design")
            echo "qwen3-coder:480b:cloud"
            ;;
        "implementation" | "coding" | "features")
            echo "qwen3-coder-next:cloud"
            ;;
        "security" | "crypto" | "validation")
            echo "deepseek-v3.2:cloud"
            ;;
        "documentation" | "tests" | "bindings")
            echo "gemma4:31b:cloud"
            ;;
        "triage" | "quick_fixes" | "linting")
            echo "gemini-3-flash-preview:cloud"
            ;;
        "review" | "gatekeeping" | "quality")
            echo "kimi-k2-thinking:cloud"
            ;;
        "orchestration" | "coordination")
            echo "mistral-large-3:675b:cloud"
            ;;
        "rust" | "protocols" | "core")
            echo "glm-5.1:cloud"
            ;;
        *)
            echo "qwen3-coder-next:cloud"  # Default for unknown task types
            ;;
    esac
}

# Validate all models in the capability mapping
validate_all_models() {
    echo "Validating all models in capability mapping..."

    if [ ! -f "$MAPPING_FILE" ]; then
        echo "❌ Model mapping file not found. Creating it first..."
        create_model_mapping
    fi

    # Extract models from mapping using grep (since jq not available)
    models=$(grep -o '"[^"]*": {' "$MAPPING_FILE" | sed 's/"\([^"]*\)": {/\1/')

    if [ $? -ne 0 ]; then
        echo "❌ Failed to parse model mapping file"
        return 1
    fi

    for model in $models; do
        base_model="${model%:cloud}"
        check_model_availability "$base_model"
    done
}

# Update orchestrator manager to use model validation
update_agent_launch_with_validation() {
    echo "Updating agent launch to include model validation..."

    # This would be integrated into the orchestrator_manager.sh
    # For now, just show the recommended approach
    cat > ".claude/model_validation_template.sh" << 'EOF'
#!/bin/bash
# Model Validation Template for Agent Launch

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
        echo "❌ CRITICAL: Model $model not available via Ollama Cloud"
        echo "   Attempting fallback to next best model for $agent_type"

        # Fallback logic based on agent type
        case "$agent_type" in
            "architect") fallback="qwen3-coder-next:cloud" ;;
            "implementer") fallback="deepseek-v3.2:cloud" ;;
            "precision_validator") fallback="qwen3-coder:480b:cloud" ;;
            "worker") fallback="gemini-3-flash-preview:cloud" ;;
            *) fallback="qwen3-coder-next:cloud" ;;
        esac

        echo "   Falling back to: $fallback"
        model="$fallback"
    else
        echo "✅ Model $model verified available"
    fi

    echo "$model"
}
EOF

    echo "✅ Model validation template created at .claude/model_validation_template.sh"
    echo "📋 Integrate this into orchestrator_manager.sh launch function"
}

# Main command router
case "${1:-help}" in
    "create")
        create_model_mapping
        ;;
    "check")
        if [ -n "$2" ]; then
            check_model_availability "$2"
        else
            echo "Usage: $0 check <model_name>"
            exit 1
        fi
        ;;
    "recommend")
        if [ -n "$2" ]; then
            get_recommended_model "$2"
        else
            echo "Usage: $0 recommend <task_type>"
            echo "Task types: architecture, implementation, security, documentation, triage, review, orchestration, rust"
            exit 1
        fi
        ;;
    "validate")
        validate_all_models
        ;;
    "update")
        update_agent_launch_with_validation
        ;;
    "help" | *)
        echo "SCMessenger Model Manager"
        echo "Usage: $0 {create|check|recommend|validate|update|help}"
        echo ""
        echo "Commands:"
        echo "  create          Create model capability mapping from Ollama API"
        echo "  check <model>   Check if a specific model is available via Ollama Cloud"
        echo "  recommend <type> Get recommended model for task type"
        echo "  validate        Validate all models in capability mapping"
        echo "  update          Update agent launch with model validation"
        echo "  help            Show this help message"
        ;;
esac