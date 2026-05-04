
import json

user_models = [
    "devstral-small-2:24b", "qwen3-next:80b", "gpt-oss:120b", "qwen3-vl:235b-instruct",
    "ministral-3:8b", "mistral-large-3:675b", "gemini-3-flash-preview", "glm-5",
    "kimi-k2.6", "kimi-k2-thinking", "qwen3-coder:480b", "minimax-m2.5",
    "devstral-2:123b", "gemma4:31b", "nemotron-3-super", "kimi-k2:1t",
    "qwen3-coder-next", "deepseek-v4-pro", "gpt-oss:20b", "gemma3:27b",
    "glm-4.6", "nemotron-3-nano:30b", "rnj-1:8b", "glm-5.1", "minimax-m2.7",
    "gemma3:12b", "qwen3.5:397b", "cogito-2.1:671b", "glm-4.7", "minimax-m2.1",
    "ministral-3:14b", "gemma3:4b", "kimi-k2.5", "deepseek-v3.2", "deepseek-v4-flash",
    "deepseek-v3.1:671b", "qwen3-vl:235b", "minimax-m2", "ministral-3:3b"
]

with open('.claude/model_capability_mapping.json', 'r') as f:
    mapping = json.load(f)

mapping_models = [m.replace(':cloud', '') for m in mapping['model_capabilities'].keys()]

missing = []
for um in user_models:
    if um not in mapping_models:
        missing.append(um)

print(f"Missing models in mapping: {missing}")

with open('.claude/agent_pool.json', 'r') as f:
    pool = json.load(f)

pool_available_models = [m['name'] for m in pool.get('available_models', [])]
missing_in_pool = []
for um in user_models:
    if um not in pool_available_models:
        missing_in_pool.append(um)

print(f"Missing models in pool available_models: {missing_in_pool}")

# Check for 'nematron' typo
for a in pool.get('agents', []):
    if 'nematron' in a.get('model', '') or 'nematron' in a.get('fallback_model', ''):
        print(f"Found 'nematron' typo in agent: {a['name']}")

for m in pool.get('available_models', []):
    if 'nematron' in m.get('name', ''):
        print(f"Found 'nematron' typo in available_models: {m['name']}")
