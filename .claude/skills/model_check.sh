#!/usr/bin/env bash
set -euo pipefail

COMMAND="${1:-all}"
AGENT_NAME="${2:-}"

POOL_FILE=".claude/agent_pool.json"

if [ ! -f "$POOL_FILE" ]; then
  echo "❌ Agent pool file not found: ${POOL_FILE}"
  exit 1
fi

echo "=== Model Availability Check ==="
echo ""

case "$COMMAND" in
  all)
    echo "Checking all models in pool..."
    # Extract model names from pool
    python3 -c "
import json, sys
with open('${POOL_FILE}') as f:
    data = json.load(f)
models = set()
for agent in data.get('agents', []):
    models.add(agent['model'].replace(':cloud', ''))
    if 'fallback_model' in agent:
        models.add(agent['fallback_model'].replace(':cloud', ''))
for m in sorted(models):
    print(f'  {m}')
print(f'\nTotal: {len(models)} models to verify')
"
    echo ""
    echo "To verify availability, use WebFetch to check:"
    echo "  https://ollama.com/api/tags"
    echo ""
    echo "Or run: bash .claude/model_validation_template.sh"
    ;;
  required)
    echo "Checking primary and fallback models for all agent roles..."
    python3 -c "
import json
with open('${POOL_FILE}') as f:
    data = json.load(f)
for agent in data.get('agents', []):
    name = agent['name']
    primary = agent['model']
    fallback = agent.get('fallback_model', 'none')
    print(f'  {name}: {primary} / {fallback}')
"
    ;;
  agent)
    if [ -z "$AGENT_NAME" ]; then
      echo "❌ Agent name required. Usage: model_check.sh agent <agent_name>"
      echo ""
      echo "Available agents:"
      python3 -c "
import json
with open('${POOL_FILE}') as f:
    data = json.load(f)
for agent in data.get('agents', []):
    print(f'  {agent[\"name\"]}')
"
      exit 1
    fi
    echo "Checking models for agent: ${AGENT_NAME}"
    python3 -c "
import json
with open('${POOL_FILE}') as f:
    data = json.load(f)
for agent in data.get('agents', []):
    if agent['name'] == '${AGENT_NAME}':
        print(f'  Primary: {agent[\"model\"]}')
        print(f'  Fallback: {agent.get(\"fallback_model\", \"none\")}')
        print(f'  Purpose: {agent[\"purpose\"]}')
        break
else:
    print('  Agent not found')
"
    ;;
  help|*)
    echo "Usage: model_check.sh <command> [args]"
    echo ""
    echo "Commands:"
    echo "  all       List all models in the agent pool"
    echo "  required  List primary/fallback models for all agent roles"
    echo "  agent     Check models for a specific agent role"
    echo "  help      Show this help"
    ;;
esac
