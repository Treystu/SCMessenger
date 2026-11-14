# SCMessenger orchestration adapter: Claude

This is a thin Claude bootstrap, not an orchestration authority. Read
`AGENTS.md`, then `docs/ORCHESTRATION.md` in full. Validate the v2 contract:

```bash
python3 scripts/orchestration_contract.py --print-version
```

Run the same repo-owned kernel used by every frontend:

```bash
python3 scripts/orchestrate_strict.py --dry-run
```

Use Claude-native isolated workers only when they provide real isolation;
otherwise use the repo worktree manager. Map any native model to the semantic
role in `orchestration/manifest.yaml`. The controller never edits application
source, tests-as-implementation, generated files, or compile fixes. A worker
failure is re-dispatched or escalated under the canonical lifecycle.
