# STRIP_MANIFEST — Zai-Complete Selective Clone

Base: `upstream/tracking/pre-v040-tag-work` @ `0aac48b2` (2026-08-16)
This repo is a stripped, completion-oriented fork of
`Sovereign-Communication/SCMessenger`. Every removal from the upstream tree is
recorded here so that diffs, cherry-picks, and eventual PRs back to upstream
can be mapped mechanically. **Nothing here was removed upstream** — the main
repo and its worktrees are untouched.

## Mapping for upstream sync-out

- File paths not listed below are identical to upstream (modulo zai commits).
- Curated upstream planning docs were **moved, not deleted** (content byte-identical):

| Upstream path | Zai path |
|---|---|
| `HANDOFF/V1_0_0_EXECUTION_PLAN.md` | `docs/zai/plans/V1_0_0_EXECUTION_PLAN.md` |
| `HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md` | `docs/zai/plans/CODEBASE_UNIFICATION_PLAN.md` |
| `HANDOFF/todo/UNIFY_CODEBASE_DECONFLICT.md` | `docs/zai/plans/UNIFY_CODEBASE_DECONFLICT.md` |
| `HANDOFF/WIRING_TASK_INDEX.md` | `docs/zai/plans/WIRING_TASK_INDEX.md` |
| `HANDOFF/DESKTOP_BRIDGE_WIRING_SPEC.md` | `docs/zai/plans/DESKTOP_BRIDGE_WIRING_SPEC.md` |
| `HANDOFF/IN_PROGRESS/GITHUB_V1_0_0_RUNNER_SETUP.md` | `docs/zai/plans/GITHUB_V1_0_0_RUNNER_SETUP.md` |
| `reference/PHILOSOPHY_CANON.md` | `docs/zai/reference/PHILOSOPHY_CANON_2026-02-24.md` (verbatim recovery; superseded for execution by `docs/zai/PHILOSOPHY_CANON.md`) |

## Removed: process/governance state (upstream-operational, not code)

| Path (1,233 files) | Reason |
|---|---|
| `HANDOFF/*` (except the 6 moved above) | CTO/CEO seat state machine: todo/done/IN_PROGRESS/archive tickets, session logs, batch records. 48 MB of coordination history. Superseded here by `docs/zai/`. |
| `TRACKING_PRE_V040_TAG_WORK.md` | Upstream branch-governance doc for the trunk branch this repo forked. |
| `CLAUDE.md`, `GEMINI.md`, `launch_claude.ps1` | Agent-client instruction files/launchers. Replaced by `docs/zai/ZAI_CHARTER.md`. |

## Removed: agent-orchestration infrastructure

| Path | Reason |
|---|---|
| `.claude/` (83), `.codex/` (16), `.kiro/` (32), `.agents/` (5), `.bob/` (6), `.mimocode/` (7), `.opencode/` (3), `.qwen/`, `.clinerules`, `.claudeignore` | Per-agent client config/rules/skills for the swarm that operates upstream. |
| `AgentSwarmCline/` (8) | Cline swarm definitions (incl. the 28 unwired observability test fns called out in FFI_WIRING_BURNDOWN.md — those were test-fixture noise, not product code). |
| `scmessenger_swarm/`, `orchestration/`, `scm_v1_farm_queue.jsonl` | Swarm manifests/orchestration queue. |
| `ollama_cfg/` | Local model runner config for agent lanes. |
| `.github/workflows/scm-test-diagnose-fix.lock.yml` | gh-aw autonomous agentic workflow — must not run in the silent repo. |

## Removed: historical/one-off baggage

| Path | Reason |
|---|---|
| `tasks/` (31) | Superseded task tracker (T1.1–T5.9 era). |
| `reference/` (13) | Reference snapshots incl. PHILOSOPHY_CANON and historical handoff notes. |
| `patch/` (7), `scratch/` (4) | One-off patches and scratch scripts. |
| `dist/` (4) | Stale build artifacts (Netlify rebuilds `dist/` from `scripts/build-headless.sh`, which is kept; `netlify.toml` kept). |
| `log-visualizer/` (10) | Standalone log visualizer experiment. |
| `scripts/` one-offs (113 files) | Ollama/cartographer/agent monitors (`OllamaQuotaScraper*`, `Run-Cartographer*`, `SwarmHeartbeat`, `TaskGovernor`, `agent_monitor`, `agy_*`, `ai_health_check*`, `*_monitor*`, `resource_manager`, `scmdriver`, `run_local_micro`), delegation/orchestration experiments (`delegate*.py`, `orchestrate_strict`, `orchestrator_*`, `orchestration_*`, `dispatch_dial`, `launch_agent`, `run_surgeon`, `fusion_lite`, `morph_lite`, `supervisor.py`, `cloud_dispatch`, `parse_orchestration_footer`), HANDOFF-coupled tools (`batch_handoff`, `check_handoff`, `watch_handoff_lanes`, `archive_task`, `branch_disposition`, `docs_sync_check*`, `inbox_bridge.py`), one-off historical fixers (`fix_*.py/sh/swift`, `p2_06_*` ×16, `count_braces`, `deduce_files`, `core_cli_driver`, `mince_logs`, `parse_ios_log`, `comprehensive_log_capture`, `debug-session.sh`, `diagnose_message_issue`, `watch_message_send`), lane/issue triage for upstream process (`lane_probe`, `lanes.json`, `lake_route`, `triage*`, `burndown_issues`, `analyze_issues`, `qwen_*` ×7, `list_b1_tasks`, `delete_merged_branches`, `verify_branch_merges`, `verify_task_*`, `validate_v020_final`, `progress_dashboard`, `prune_sim_logs`, `repo`-side monitors), junk (`test.swift`, `jq_wrapper.sh` — hardcoded a foreign user's path, `error_handler.sh` — depends on `.claude/state`). |

## Kept deliberately (verify before assuming loss)

All product code (`core`, `cli`, `mobile`, `desktop_bridge`, `wasm`, `ui`,
`headless`, `android`, `iOS`, `shared`), `docs/`, `HANDOFF_AUDIT/` (dual-pass
audit corpus: `REPO_MAP.jsonl`, `WORLDWIDE_AUDIT_RESULTS.jsonl` — the v1.0.0
remediation worklist), `audit_system/` engine, `FFI_WIRING_BURNDOWN.md`,
`SHIP_PLAN.md` (v0.4.0 exit criteria context), `REMAINING_WORK_TRACKING.md`,
`iOS_V040_PARITY_IMPLEMENTATION_PLAN.md`, `iOS_ANDROID_PARITY_AUDIT_v040.md`,
curated `scripts/` (build/verify/gates/wiring/FFI/device-ops/soak: 90 files),
curated `.github/workflows` (27 of 28), `.githooks/`, `skills/`
(mesh-diagnostics + platform-parity-check are operationally useful),
`netlify.toml`, `cloud/`, `infra/`, `docker/`, gradle wrapper, all cargo
workspace files.

## Known dangling references (accepted, prose-only)

`docs/` and kept scripts reference removed paths (`HANDOFF/...`, `tasks/...`,
some removed scripts) in explanatory prose and comments. Functional references
were checked: no kept CI workflow, hook, or build system invokes a removed
script. If a doc link 404s locally, consult this manifest before "fixing" it —
content intentionally lives upstream only.
