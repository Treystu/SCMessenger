
import json

mapping_path = '.claude/model_capability_mapping.json'
with open(mapping_path, 'r') as f:
    mapping = json.load(f)

capabilities = mapping['model_capabilities']

# Detailed data based on search results and common knowledge
model_data = {
    "glm-5.1:cloud": {
        "capabilities": ["agentic_coding", "long_horizon_execution", "iterative_optimization", "rust_core"],
        "role": "lead_orchestrator",
        "size": "1.5T",
        "recommended_for": ["P0_orchestration", "complex_agentic_tasks", "rust_core_implementation"]
    },
    "deepseek-v4-pro:cloud": {
        "capabilities": ["advanced_reasoning", "multi_step_planning", "architecture_design", "security_audit"],
        "role": "architect_planner",
        "size": "1.6T MoE",
        "recommended_for": ["P0_planning", "SOTA_reasoning", "complex_architecture"]
    },
    "qwen3-coder-next:cloud": {
        "capabilities": ["repository_scale_coding", "complex_software_engineering", "bug_hunting"],
        "role": "implementer",
        "size": "81B",
        "recommended_for": ["P0_implementation", "feature_landing", "deep_code_refactoring"]
    },
    "cogito-2.1:671b:cloud": {
        "capabilities": ["complex_analysis", "deliberative_reasoning", "tradeoff_evaluation"],
        "role": "deliberator",
        "size": "671B",
        "recommended_for": ["P0_deliberation", "critical_decision_making"]
    },
    "minimax-m2.7:cloud": {
        "capabilities": ["self_evolving_agents", "office_benchmarks", "agentic_efficiency"],
        "role": "agentic_worker",
        "size": "480B",
        "recommended_for": ["iterative_improvement", "agentic_workflows"]
    },
    "nemotron-3-super:cloud": {
        "capabilities": ["multi_agent_collaboration", "high_throughput", "hybrid_transformer_mamba"],
        "role": "swarm_coordinator",
        "size": "120B MoE",
        "recommended_for": ["throughput_intensive_tasks", "multi_agent_sync"]
    },
    "devstral-2:123b:cloud": {
        "capabilities": ["tool_use", "file_editing", "swe_bench_optimization"],
        "role": "precision_implementer",
        "size": "123B",
        "recommended_for": ["autonomous_file_edits", "tool_heavy_workflows"]
    },
    "gemma4:31b:cloud": {
        "capabilities": ["multimodal_reasoning", "configurable_thinking", "efficient_worker"],
        "role": "multimodal_worker",
        "size": "31B",
        "recommended_for": ["vision_tasks", "general_labor", "unit_tests"]
    },
    "mistral-large-3:675b:cloud": {
        "capabilities": ["multilingual_support", "instruction_following", "enterprise_rag"],
        "role": "general_purpose_orchestrator",
        "size": "675B MoE",
        "recommended_for": ["pipeline_management", "stable_coordination"]
    },
    "gpt-oss:120b:cloud": {
        "capabilities": ["transparent_cot", "adjustable_reasoning_effort", "high_efficiency"],
        "role": "reasoning_specialist",
        "size": "120B MoE",
        "recommended_for": ["debug_reasoning", "complex_logic_verification"]
    },
    "rnj-1:8b:cloud": {
        "capabilities": ["stem_tasks", "math_code", "high_tool_calling_accuracy"],
        "role": "lightweight_specialist",
        "size": "8B",
        "recommended_for": ["quick_math", "tool_calling_gatekeeping"]
    },
    "qwen3-vl:235b:cloud": {
        "capabilities": ["vision_analysis", "screenshot_review", "ui_mockup_parsing"],
        "role": "vision_analyst",
        "size": "235B",
        "recommended_for": ["UI_review", "diagram_analysis"]
    },
    "gemini-3-flash-preview:cloud": {
        "capabilities": ["fast_triage", "low_latency_parsing", "ci_gatekeeping"],
        "role": "triage_router",
        "size": "small",
        "recommended_for": ["quick_fixes", "triage", "linting"]
    },
    "kimi-k2.6:cloud": {
        "capabilities": ["advanced_review", "contextual_understanding", "quality_gate"],
        "role": "gatekeeper_reviewer",
        "size": "595B",
        "recommended_for": ["pre_merge_review", "quality_assurance"]
    },
    "kimi-k2-thinking:cloud": {
        "capabilities": ["extended_thought", "deep_verification", "logical_proofs"],
        "role": "deep_gatekeeper",
        "size": "1.1T",
        "recommended_for": ["final_verification", "critical_security_gates"]
    }
}

# Apply updates
for model, data in model_data.items():
    if model in capabilities:
        capabilities[model].update(data)
    else:
        capabilities[model] = data

# For the rest of the 39 models, ensure they have at least basic data
all_39 = ["glm-4.6", "minimax-m2.1", "gemini-3-flash-preview", "minimax-m2.5", "devstral-small-2:24b", "rnj-1:8b", "glm-5", "glm-5.1", "qwen3-coder-next", "deepseek-v3.2", "deepseek-v3.1:671b", "gpt-oss:120b", "ministral-3:3b", "ministral-3:8b", "kimi-k2.6", "deepseek-v4-pro", "ministral-3:14b", "gemma3:4b", "gemma3:27b", "gemma4:31b", "qwen3.5:397b", "nemotron-3-super", "glm-4.7", "qwen3-coder:480b", "nemotron-3-nano:30b", "qwen3-vl:235b-instruct", "qwen3-vl:235b", "mistral-large-3:675b", "kimi-k2:1t", "kimi-k2.5", "kimi-k2-thinking", "gemma3:12b", "qwen3-next:80b", "deepseek-v4-flash", "minimax-m2", "devstral-2:123b", "cogito-2.1:671b", "gpt-oss:20b", "minimax-m2.7"]

for m in all_39:
    k = f"{m}:cloud"
    if k not in capabilities:
        capabilities[k] = {
            "capabilities": ["general_reasoning", "ollama_cloud"],
            "role": "generalist",
            "size": "unknown",
            "recommended_for": ["general_tasks"]
        }

mapping['last_updated'] = "2026-05-04T04:10:00Z"

with open(mapping_path, 'w') as f:
    json.dump(mapping, f, indent=2)

print("Mapping update with detailed capabilities complete.")
