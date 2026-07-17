import json
import os
import subprocess

def main():
    with open('scm_v1_farm_queue.jsonl', 'r', encoding='utf-8') as f:
        tasks = [json.loads(line) for line in f if line.strip()]

    done_ids = {t['id'] for t in tasks if t['status'] == 'done'}

    dispatchable = []
    for t in tasks:
        if t['status'] == 'open' and t['wave'] not in ('H', 'Z'):
            ready = all(dep in done_ids for dep in t.get('depends', []))
            if ready:
                dispatchable.append(t)

    print(f'Found {len(dispatchable)} dispatchable tasks:')
    
    os.makedirs('tmp/scmorc', exist_ok=True)
    
    for t in dispatchable:
        print(f"[{t['id']}] {t['description']}")
        
        # Write prompt file
        prompt_path = f"tmp/scmorc/{t['id']}.prompt.md"
        with open(prompt_path, 'w', encoding='utf-8') as pf:
            pf.write(f"""You are a headless SCMessenger worker. Your stdout is parsed by an orchestrator.
TOKEN PROTOCOL (mandatory):
- Do NOT run `cargo build`/`cargo check`/`cargo test` or `./gradlew` yourself, even to self-verify. The orchestrator runs ALL build verification centrally.
- Locate code with rg, then read ONLY the surrounding ~20-40 lines. No whole-file reads unless the file is under 200 lines.
- Do NOT commit. Do NOT push. The orchestrator commits after verifying your diff.
- Do NOT move HANDOFF files. The orchestrator owns the state machine.
- No emojis anywhere.
REPORT FORMAT (your final message, nothing after it):
Line 1: RESULT: DONE|BLOCKED|FAILED
Then max 10 lines: what changed, files touched, anything the orchestrator must know before it runs verification.

### Requirements:
Task {t['id']}: {t['description']}
""")
        
        # Dispatch to Qwen
        tier_map = {
            "FLASH": "qwen-turbo",
            "CODER": "qwen3-coder-32b-instruct",
            "THINK": "qwen3-max",
            "MAX": "qwen-max"
        }
        
        model = tier_map.get(t['tier'], "qwen-plus")
        
        cmd = [
            "python", "scripts/delegate_task.py",
            "--task", prompt_path,
            "--provider", "qwen",
            "--model", model
        ]
        
        print(f"Dispatching {t['id']} via Qwen...")
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode == 0:
            print(f"[{t['id']}] SUCCESS")
        else:
            print(f"[{t['id']}] FAILED: {res.stderr}")

if __name__ == '__main__':
    main()
