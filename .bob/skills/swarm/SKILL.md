---
name: swarm
description: Launch an SCMessenger agent from the pool
metadata:
  user-invocable: true
  disable-model-invocation: true
---

Launch an SCMessenger agent from the pool

**The orchestrator script lives in `/Users/scmessenger/Documents/Github/SCMessenger/`.** Before running, anchor to the project root:

```bash
cd /Users/scmessenger/Documents/Github/SCMessenger
```

If the directory is missing, **STOP and tell the user** — do not guess at a fallback path.

```bash
bash .claude/orchestrator_manager.sh pool launch $ARGUMENTS
```

**On Windows, use the full Git Bash path:**
```bash
cd /Users/scmessenger/Documents/Github/SCMessenger
"C:\Program Files\Git\bin\bash.exe" .claude/orchestrator_manager.sh pool launch $ARGUMENTS
```

Arguments: $ARGUMENTS (agent_name [task_file])
