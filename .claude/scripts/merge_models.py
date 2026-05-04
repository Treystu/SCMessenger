
import json

# Latest models from Ollama API
latest_models = ["glm-4.6", "minimax-m2.1", "gemini-3-flash-preview", "minimax-m2.5", "devstral-small-2:24b", "rnj-1:8b", "glm-5", "glm-5.1", "qwen3-coder-next", "deepseek-v3.2", "deepseek-v3.1:671b", "gpt-oss:120b", "ministral-3:3b", "ministral-3:8b", "kimi-k2.6", "deepseek-v4-pro", "ministral-3:14b", "gemma3:4b", "gemma3:27b", "gemma4:31b", "qwen3.5:397b", "nemotron-3-super", "glm-4.7", "qwen3-coder:480b", "nemotron-3-nano:30b", "qwen3-vl:235b-instruct", "qwen3-vl:235b", "mistral-large-3:675b", "kimi-k2:1t", "kimi-k2.5", "kimi-k2-thinking", "gemma3:12b", "qwen3-next:80b", "deepseek-v4-flash", "minimax-m2", "devstral-2:123b", "cogito-2.1:671b", "gpt-oss:20b", "minimax-m2.7"]

mapping_path = '.claude/model_capability_mapping.json'
with open(mapping_path, 'r') as f:
    mapping = json.load(f)

capabilities = mapping['model_capabilities']

for m in latest_models:
    cloud_name = f"{m}:cloud"
    if cloud_name not in capabilities:
        print(f"Adding missing model: {cloud_name}")
        # Default capability template for new models
        capabilities[cloud_name] = {
            "capabilities": ["general_reasoning", "ollama_cloud"],
            "role": "generalist",
            "size": "unknown",
            "recommended_for": ["general_tasks"]
        }
    else:
        print(f"Model already in mapping: {cloud_name}")

# Fix 'nematron' spelling in keys if it exists
stale_keys = [k for k in capabilities.keys() if 'nematron' in k]
for k in stale_keys:
    new_k = k.replace('nematron', 'nemotron')
    capabilities[new_k] = capabilities.pop(k)
    print(f"Renamed {k} to {new_k}")

mapping['last_updated'] = "2026-05-04T04:00:00Z"

with open(mapping_path, 'w') as f:
    json.dump(mapping, f, indent=2)

print("Mapping update complete.")
