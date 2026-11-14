# CHUNK W2-M — wiring burndown pipeline repair (measurement before work)

- **Lane**: W (wiring) · **Type**: Worker (scripts-only, no cargo/gradle) ·
  **Branch**: `zai/w2m-burndown-pipeline` · **Briefed**: 2026-08-16
- **Why first**: `FFI_WIRING_BURNDOWN.md` is the W lane's scoreboard and the
  charter's exit criterion #1, but the fork's strip removed its data inputs
  (`HANDOFF/discovery/REPO_MAP.jsonl`, `log-visualizer/public/data/`), and the
  current 162 count includes 28 functions from the removed `AgentSwarmCline/`
  tree. Driving v1.0.0 off a broken scoreboard repeats upstream's "backlog as
  graveyard" failure.

## Context

- Pipeline as it exists: `scripts/build_wiring_graph.py` (parses
  `HANDOFF_AUDIT/REPO_MAP.jsonl` + `HANDOFF/discovery/REPO_MAP.jsonl` →
  writes `wiring_graph.json` + `unwired_functions.json` under
  `log-visualizer/public/data/`) → `scripts/generate_wiring_burndown.py`
  (reads `unwired_functions.json` → rewrites `FFI_WIRING_BURNDOWN.md`).
  In this fork both input #2 and the output dir are gone.
- Surviving corpus: `HANDOFF_AUDIT/REPO_MAP.jsonl` (+ per-area maps
  `REPO_MAP_android.jsonl`, `REPO_MAP_other.jsonl`), all committed.

## Done-when

1. `build_wiring_graph.py` reads only surviving inputs (drop
   `HANDOFF/discovery/` path with a loud SKIPPED-GONE note, never a silent
   skip), writes its outputs to a committed location: `docs/zai/metrics/`
   (`wiring_graph.json`, `unwired_functions.json`).
2. `generate_wiring_burndown.py` reads the new location and regenerates
   `FFI_WIRING_BURNDOWN.md` with the post-strip TRUE unwired count.
3. `FFI_WIRING_BURNDOWN.md` regenerated and committed; every file it lists
   exists in this tree (stale entries for removed paths — e.g.
   `AgentSwarmCline/` — are filtered out with a printed count of dropped
   ghost entries; "express reduced confidence by printing MORE, never less").
4. Generated markdown passes `python scripts/rules_check.py` (no emoji, no
   trailing whitespace — generator output was never rules_check'd upstream).
5. `scripts/verify_incremental_gate.py` gate 5 (wiring non-regression)
   re-pointed at the new pipeline so it actually runs.

## Constraints

- Scripts-only: no changes under `core/`, `android/`, `iOS/`, no Cargo/gradle
  anything. Python 3.14 is the interpreter.
- Preserve upstream flag/CLI compatibility where cheap (diff-friendly for
  eventual upstream export); mark fork-only behavior with a `ZAI:` comment.
- All runs through `bash scripts/gate.sh <label> <cmd...>`; report quotes
  GATE_EXIT lines + terminal output.

## Report contract

RESULT / VERIFICATION (exact commands + GATE_EXIT + quoted output) / FILES /
COPIES (n/a — state that) / NOTES ≤8 lines, per `ZAI_AGENT_ORG.md` §5.

---

## Completion record (2026-08-16)

- **Worker report**: DONE — 4 commits on `zai/w2m-burndown-pipeline`
  (5120edc1 re-point inputs/outputs, 83adcf6b rules-clean md + new metrics
  dir, 67efbe6d baseline regeneration, 982bc064 gate-5 re-point).
- **Integrator re-verification [V-RUN]** (this session, W2-M worktree, via
  `scripts/gate.sh`): graph build `GATE_EXIT=0` (Unwired: 788; ghost drop
  7 files / 72 funcs, each printed; HANDOFF/discovery input loudly
  SKIPPED-GONE), md generation `GATE_EXIT=0` (788 across 123 modules),
  `rules_check.py FFI_WIRING_BURNDOWN.md` `GATE_EXIT=0`. Determinism:
  regeneration is timestamp-only drift vs committed artifacts (data
  byte-identical). Diff reviewed per-file (loud-skip parser dual-format,
  committed metrics outputs, ZAI: markers, CLI compat preserved).
- **Deferred**: full `verify_incremental_gate.py` run (gates 1-3 are cargo
  invocations) — serialized behind the in-flight gradle build per R-Z5;
  node/port pre-flight per R-Z6 came back clean (no listeners on
  5404/7744/9001, no scm processes).
- **Finding of record**: true unwired baseline is **788**, not 162. The old
  scoreboard read a corpus that no longer exists (pre-strip +
  discovery overlay) and its BOM-split parser yields 0 records on today's
  plain-JSONL corpus. PHIL-016 enacted in the same breath as its evidence.
- **Backlog impact**: W2 re-ranked — concentration is in core Rust
  (iron_core 72, wasm/lib 58, swarm 36, mobile_bridge 30), not Kotlin;
  W2 is now a wire/expose/retire triage with Validator coverage on
  merge-blocked trees.
