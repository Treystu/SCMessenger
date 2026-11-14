# CHUNK W2-T1 — wiring triage batch 1: iron_core + wasm surface (155 fns)

- **Lane**: W (wiring) · **Type**: Worker (cargo-bearing; own target dir) ·
  **Branch**: `zai/w2t1-wiring-triage-1` · **Briefed**: 2026-08-16
- **Parallel with**: U1 (disjoint file sets — this chunk touches NO files
  U1 touches; U1 owns escalation/mobile_bridge/SmartTransportRouter/
  MeshRepository).

## Scope (from the repaired 788 scoreboard, top non-overlapping files)

| File | Unwired |
|---|---|
| `core/src/iron_core.rs` | 72 |
| `wasm/src/lib.rs` | 58 |
| `wasm/src/daemon_bridge.rs` | 25 |

NOT in scope (later batches): swarm.rs, mobile_bridge.rs (U1 overlap),
privacy/padding.rs, crypto/encrypt.rs, routing/*, transport/* (merge-blocked
trees → separate Validator-gated batches).

## What "triage" means (per ZAI_BACKLOG W2)

For each unwired function in scope, classify with evidence and act:
1. **WIRE** — has real callers that the graph's simple-name resolution
   missed (cross-file same-name calls under-wire — check callers manually
   via grep before trusting "unwired"). If genuinely called: document, no
   code change needed; the scoreboard fix is a future resolver improvement,
   out of scope here.
2. **EXPOSE** — part of the product surface that platforms need but can't
   reach (candidate UniFFI/FFI surface). List as a proposal table in the
   chunk record; do NOT change `core/src/api.udl` in this chunk (U1 is
   adding UDL functions — file-set conflict).
3. **RETIRE** — dead code: no callers, no plausible platform need. Remove
   with `#[allow(dead_code)]` audit note in the commit body; if removal is
   risky (trait impls, callback contracts), mark `#[deprecated]`-style
   TODO in the record instead and leave code.

Conservative default when uncertain: **document, don't delete** (PHIL-013:
retiring a function the resolver under-wired is a scoreboard lie in the
other direction).

## Done-when

1. `docs/zai/chunks/W2-T1-triage-record.md`: per-function classification
   table (function, class, evidence, action) for all 155.
2. RETIRE actions executed: `cargo check -p scmessenger-core` and
   `cargo check -p scmessenger-wasm` green (own target dir, via gate.sh).
3. Burndown regenerated post-changes; delta explained line-by-line in the
   record (expected: removed functions drop the count; WIRE-classified
   entries stay 788-counted — say so explicitly).
4. No changes outside the three in-scope files + the record doc.
5. Report contract per org §5 (RESULT/VERIFICATION/FILES/COPIES/CANON/NOTES).

## Constraints

- Own worktree `../.zai-worktrees/W2-T1`, own
  `CARGO_TARGET_DIR=.../.scm-zai-target/W2-T1`, `CARGO_BUILD_JOBS=4`.
- NO gradle. NO changes to api.udl, escalation.rs, mobile_bridge.rs,
  SmartTransportRouter.kt, MeshRepository.kt (U1's files).
- All checks via `bash scripts/gate.sh <label> <cmd...>`.
- Canon: PHIL-016 (scoreboard truth), PHIL-013 (no unproven retirements).
