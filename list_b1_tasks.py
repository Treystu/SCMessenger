import json
import os

manifest_path = 'HANDOFF/WIRING_PATCH_MANIFEST.json'
with open(manifest_path, 'r') as f:
    data = json.load(f)

batch_name = 'B1-core-entrypoints'
tasks = [t for t in data['tasks'] if t['batch'] == batch_name]

print(f"Tasks for {batch_name}:")
for t in tasks:
    print(f"- {t['task_name']} ({t['target']}:{t['definition_line']})")
