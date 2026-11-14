# SCMessenger Controller Directive

Status: Active
Last updated: 2026-08-10

This is a frontend-neutral bootstrap pointer. Read `AGENTS.md`, then the full
`docs/ORCHESTRATION.md`, validate `orchestration/manifest.yaml` with
`scripts/orchestration_contract.py`, and operate through
`scripts/orchestrate_strict.py`.

The controller coordinates. It may not author application source, test
implementation, generated patches, or compile repairs, including tiny changes.
On worker failure, malformed output, gate failure, or disagreement, record the
durable non-success state and route a fresh worker, planner, reviewer, second
opinion, or operator escalation as specified by the canonical protocol.

Use `scripts/orchestrator_guard.py` before an authority-sensitive action. Do
not treat controller approval as a release gate; release evidence is evaluated
by an independent role.
