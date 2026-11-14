
import json

latest_models = ["glm-4.6", "minimax-m2.1", "gemini-3-flash-preview", "minimax-m2.5", "devstral-small-2:24b", "rnj-1:8b", "glm-5", "glm-5.1", "qwen3-coder-next", "deepseek-v3.2", "deepseek-v3.1:671b", "gpt-oss:120b", "ministral-3:3b", "ministral-3:8b", "kimi-k2.6", "deepseek-v4-pro", "ministral-3:14b", "gemma3:4b", "gemma3:27b", "gemma4:31b", "qwen3.5:397b", "nemotron-3-super", "glm-4.7", "qwen3-coder:480b", "nemotron-3-nano:30b", "qwen3-vl:235b-instruct", "qwen3-vl:235b", "mistral-large-3:675b", "kimi-k2:1t", "kimi-k2.5", "kimi-k2-thinking", "gemma3:12b", "qwen3-next:80b", "deepseek-v4-flash", "minimax-m2", "devstral-2:123b", "cogito-2.1:671b", "gpt-oss:20b", "minimax-m2.7"]

pool_path = '.claude/agent_pool.json'
with open(pool_path, 'r') as f:
    pool = json.load(f)

available = pool.get('available_models', [])
existing_names = [m['name'] for m in available]

for m in latest_models:
    if m not in existing_names:
        print(f"Adding to available_models: {m}")
        available.append({
            "name": m,
            "params": "unknown",
            "specialty": "newly discovered"
        })

pool['available_models'] = available

with open(pool_path, 'w') as f:
    json.dump(pool, f, indent=2)

print("Agent pool sync complete.")
