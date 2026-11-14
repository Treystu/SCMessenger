# CHUNK U1 — escalation single-authority consolidation (WS-B B2)

- **Lane**: U (unification) · **Type**: Worker + **Validator verdict required**
  (touches `core/src/transport/escalation.rs` — merge-blocked tree) ·
  **Branch**: `zai/u1-escalation-authority` · **Briefed**: 2026-08-16
- **Upstream mapping**: `docs/zai/plans/V1_0_0_EXECUTION_PLAN.md` line 269
  (B2, [AUDIT-GATE]); design mandate per approved zai plan.

## Problem (verified this session)

Three transport-decision authorities coexist:

1. **Core `EscalationEngine`** (`core/src/transport/escalation.rs`, 698 lines)
   — policy-scored (Balanced default), instantiated as a `OnceLock` in
   `mobile_bridge.rs:3494`, exposed over UniFFI as `recommended_transport` /
   `update_peer_transports` — **called by no platform** (checked: no Kotlin
   callers; only generated Swift bindings exist). Static capabilities only,
   no live health. Inert.
2. **Kotlin `SmartTransportRouter`**
   (`android/.../transport/SmartTransportRouter.kt`) — de-facto Android
   authority: per-peer health scores (success rate × latency EWMA) pick the
   preferred transport; races the rest with a 500 ms preferred timeout.
   (Its historical dead params from ANDROID_SWEEP_02 were already fixed
   upstream — all params now used.)
3. **`MeshRepository` per-delivery heuristics** — `strictBleOnly`,
   `prefer_connected_device` target fallback, `preferred*` hint overrides
   shape what's offered per delivery.

## Design (the consolidation)

**Core becomes the single authority, fed by all platforms.**

1. `EscalationEngine` gains live health per (peer, transport):
   `record_delivery_outcome(peer, transport, success, latency_ms)` storing
   success/failure counts + latency EWMA; `select_best_transport` /
   `find_better_transport` / `find_worse_transport` multiply the policy score
   by a health factor = Laplace-smoothed success rate clamped to [0.25, 1.0].
   **Neutral (1.0) with zero observations** — every existing unit test must
   stay green unchanged (they run without health data).
2. New UniFFI surface (string-typed to avoid enum churn/checksum cascade):
   - `escalation_record_outcome(peer_id: String, transport: String,
     success: Bool, latency_ms: u64)`
   - `escalation_preferred_order(peer_id: String, available: Vec<String>)
     -> Vec<String>` — subset of input ordered best-first; unknown transport
     strings keep input order at the end (loud, not lossy).
3. Kotlin `SmartTransportRouter` demoted to executor: constructor-injected
   collaborators (defaults null = current behavior, so existing Android unit
   tests keep passing):
   - `transportOrderAdvisor: (suspend (String, List<TransportType>) -> List<TransportType>)?`
     — when present, ordering comes from core; local health sort becomes
     fallback only.
   - `outcomeReporter: ((String, TransportType, Boolean, Long) -> Unit)?` —
     `recordSuccess`/`recordFailure` forward outcomes to core (local cache
     retained for offline/fallback).
4. `MeshRepository` wires both collaborators to the new UniFFI calls and
   keeps its heuristics as **inputs** — they already shape the available
   list handed to the router; caller decides what exists, authority decides
   order. That is the "input, not rival policy" relationship upstream
   specified.

## Done-when

1. Core unit tests: new tests for health-adjusted selection (unhealthy
   preferred transport demoted; neutral-with-no-data exact old scores) —
   `cargo test -p scmessenger-core escalation` green.
2. `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings`
   green (via `scripts/gate.sh`).
3. Kotlin compiles + existing SmartTransportRouter/MeshRepository tests pass
   (gradle, main checkout, Integrator-run).
4. UniFFI bindings regenerated (`scripts/generate_uniffi_kotlin.sh`), Swift
   binding regen attempted; if impossible on Windows, BLOCKED-flag it (do
   NOT hand-edit generated files — AGENTS.md rule).
5. Validator verdict artifact `docs/zai/chunks/U1-validator-verdict.md`
   naming the reviewed SHA (fresh instance, read-only).

## Audit-style commit notes (merge-blocked dir)

Every commit touching `core/src/transport/escalation.rs` carries: what
changed in the decision path, why health weighting cannot regress
no-health behavior (neutral factor proof), and the test that pins it.
