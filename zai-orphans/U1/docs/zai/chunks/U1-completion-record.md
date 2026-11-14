# U1 completion record — escalation single-authority

- Worker chunk: `docs/zai/chunks/U1-escalation-authority.md`
- Branch: `zai/u1-escalation-authority` (worktree `.zai-worktrees/U1`, base
  `3fe1b8f0`)
- Status: **IMPLEMENTED, GATES DEFERRED — disk pause** (see Verification)
- Commits: `d3b4bded` (core health weighting), `18c5e267` (UniFFI surface),
  `ccbfc8b7` (Kotlin demotion), plus this record.

## What moved, per function

### Core authority — `core/src/transport/escalation.rs` (commit d3b4bded)

| Function | Before U1 | After U1 |
|---|---|---|
| `record_delivery_outcome(peer, transport, success, latency_ms)` | did not exist | THE input port: platforms report outcomes; stores success/failure counts + latency EWMA per (peer, transport) |
| `health_factor_for` / `TransportHealthStats::health_factor` | did not exist | Laplace-smoothed success rate clamped [0.25, 1.0]; **exactly 1.0 with zero observations** (neutral-factor proof) |
| `weighted_score` (new, private) | did not exist | `policy_score * health_factor` — the only place health touches scores |
| `select_best_transport_for_peer` (new) | did not exist | peer-aware winner used by `init_peer` and `recommended_transport` |
| `select_best_transport` | policy-only winner | unchanged signature/behavior; now documented health-blind degenerate case (legacy tests call it directly) |
| `find_better_transport` / `find_worse_transport` (private) | policy-only | peer-aware weighted scores; serve `should_escalate`/`escalate`/`deescalate` — so escalation AND deescalation honor live health |
| `preferred_order` (new) | did not exist | stable best-first ordering; the engine-side core of the UniFFI ordering surface |
| `health_stats` (new) | did not exist | read-only diagnostics snapshot |
| `cleanup_peer` | dropped state only | also drops that peer's health entries (bounded map) |
| `transport_type_from_name` (new, free fn) | did not exist | canonical string <-> `TransportType` parsing for the string-typed surface |

### UniFFI surface — `core/src/api.udl` + `core/src/mobile_bridge.rs` (commit 18c5e267)

| Function | Contract |
|---|---|
| `escalation_record_outcome(peer_id: String, transport: String, success: Bool, latency_ms: u64)` | feeds the authority; malformed peer id (not 64-hex/32 bytes) or unknown transport name -> `tracing::warn` + drop (loud, not lossy) |
| `escalation_preferred_order(peer_id: String, available: Vec<String>) -> Vec<String>` | subset of input, best-first (policy x health); unknown transport names appended in input order; malformed peer id -> input order returned |

UDL-declared (scaffolded by `build.rs`), implemented as plain `pub fn`s in
`mobile_bridge.rs` beside the existing `ESCALATION_ENGINE` OnceLock — same
style as `blocked_identity_*` / `encode_receipt`. Scaffolding include is
`cfg(not(wasm32))` and `mobile_bridge` is a non-wasm module, so no wasm
impact.

### Kotlin executor — `SmartTransportRouter.kt` (commit ccbfc8b7)

| Element | Before U1 | After U1 |
|---|---|---|
| constructor | no args | + `transportOrderAdvisor` (suspend, nullable, default null), `outcomeReporter` (nullable) — defaults = old behavior, existing tests unaffected |
| `attemptDelivery` preferred pick | `getPreferredTransport` (local health) — the rival authority | `resolvePreferredTransport`: advisor order decides when injected; local sort fallback only (advisor null/threw/nothing usable) |
| `recordSuccess`/`recordFailure` | local only | local update (fallback cache) + forward to `outcomeReporter` outside the mutex; never throws into callers |
| `getPreferredTransport` / `getAvailableTransportsSorted` | authority | documented local fallback / local diagnostics view |

### Kotlin wiring — `MeshRepository.kt` (commit ccbfc8b7)

- Router constructed with both collaborators bound to
  `uniffi.api.escalationPreferredOrder` / `uniffi.api.escalationRecordOutcome`.
- `coreEscalationPeerId`: router peer id -> 64-hex canonical id via
  `transportToCanonicalMap`; unresolvable (relays map to base58) degrades to
  input order / local-only outcomes, never crashes a delivery.
- `smartTransportToCoreName` / `coreNameToSmartTransport` (companion,
  bijective): BLE->"BLE", WIFI_DIRECT->"WiFiDirect", CORE->"Internet",
  TCP_MDNS->"Local" (LAN-direct = core's local-network tier).
- `strictBleOnly`, `prefer_connected_device` BLE target fallback, preferred
  hints: **unchanged** — they stay INPUTS shaping the available list handed
  to the router; the authority only orders what the caller says exists.

## Test evidence (core)

New tests in `escalation.rs` (13 added, all alongside the 24 pre-existing
tests which are untouched):

- `test_health_factor_neutral_with_zero_observations` — factor == 1.0 exactly.
- `test_health_factor_laplace_smoothed_and_clamped` — clamp floor 0.25, all-success 11/12, 5/1 -> 0.75.
- `test_unhealthy_preferred_transport_demoted` — PreferHighBandwidth: WiFiDirect failures hand the win to Internet.
- `test_unhealthy_preferred_transport_demoted_balanced` — same under the default policy the bridge uses.
- `test_neutral_health_preserves_legacy_selection_exactly` — the regression pin: weighted winner == policy-only winner across all 4 policies and legacy lists with zero health.
- `test_recorded_health_is_per_peer` — health keying per (peer, transport).
- `test_latency_ewma_recorded_on_success_only` — seeds 100, EWMA(0.25) -> 150; failure adds no latency sample.
- `test_preferred_order_stable_and_best_first` — ordering + flip after demotion.
- `test_health_cleared_with_cleanup_peer` — no health leak after cleanup.
- `test_transport_type_from_name_round_trip` — string surface parsing.
- `test_deescalate_uses_health_weighted_scores` — fallback honors health (poisoned Internet skipped for BLE).

## Audit note text (carried verbatim in commit d3b4bded)

> Decision-path change: select_best_transport_for_peer (new),
> find_better_transport, find_worse_transport and preferred_order (new)
> now multiply the static policy score by a per-(peer,transport) health
> factor fed by record_delivery_outcome (new). escalate/deescalate/
> should_escalate/init_peer/recommended_transport route through these,
> so escalation AND deescalation now honor live health.
>
> Why no-health behavior cannot regress (neutral-factor proof): the
> factor is EXACTLY 1.0 with zero observations -- no smoothing, no
> clamping -- so weighted_score == escalation_score bit-for-bit and
> every max_by/sort_by comparison is unchanged; health can only alter a
> comparison after an outcome is recorded for one of the compared
> transports. cleanup_peer also clears that peer's health entries so
> the map stays bounded with the peer set.

## Verification status — GATES DEFERRED (disk pause 2026-08-16)

- Operator disk-critical directive received mid-session: one
  `cargo test -p scmessenger-core escalation` gate run (via
  `scripts/gate.sh`, env `CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/
  .scm-zai-target/U1 CARGO_BUILD_JOBS=4`) was killed during compilation —
  no pass/fail signal existed. No cargo command was run after the pause.
- **Integrator must run (batch check, in order):**
  1. `cargo test -p scmessenger-core escalation`
  2. `cargo fmt -p scmessenger-core -- --check`
  3. `cargo clippy -p scmessenger-core --all-targets -- -D warnings`
  4. `bash scripts/generate_uniffi_kotlin.sh` — REQUIRED before gradle:
     MeshRepository references `uniffi.api.escalationPreferredOrder` /
     `escalationRecordOutcome`, absent from committed generated bindings.
  5. gradle (main checkout only): compile + ReceiptUnificationTest (uses
     `SmartTransportRouter()` no-arg — must remain green).
- All Kotlin claims are [V-READ] (read-verified, never compiled here).

## COPIES (escalation/transport-order concept, post-U1 state)

1. `core/src/transport/escalation.rs` — THE authority (this chunk).
2. `android/.../transport/SmartTransportRouter.kt` — demoted to executor,
   advisor/reporter wired in MeshRepository (this chunk).
3. `android/.../data/MeshRepository.kt` heuristics — inputs only, unchanged.
4. `iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift` —
   NOT touched (outside U1 file set): still a local-health authority on
   iOS. Same two UniFFI functions are available to it; aligning it is
   follow-up work for the iOS lane. Recorded here per PHIL-013.
5. `core/src/transport/manager.rs` — consumes the engine for
   re-escalate/deescalate on connect/prune; no ordering policy of its own;
   automatically benefits from health weighting (no change needed).

## Canon

- **PHIL-012** satisfied for Android: one decision-maker (core engine),
  router is executor, repository heuristics are inputs. iOS copy remains
  (see COPIES #4) — partial, tracked.
- **PHIL-013**: copies #1-#3 now bound by the tested UniFFI string contract;
  copy #4 unbound (iOS follow-up).
- Merge-blocked dir `core/src/transport/`: audit notes in every commit
  body; **Validator verdict still required** before merge.
