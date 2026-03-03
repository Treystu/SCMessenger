# SCMessenger v0.2.0 Alpha Milestone Plan

Status: Draft for execution lock  
Date: 2026-03-03  
Scope: Core + Android + iOS + Desktop GUI + Relay topology

---

## 1) Planning policy alignment (repo philosophy)

This plan follows repository policy from `CONTRIBUTING.md` and `CLAUDE.md`:

1. LoC estimates only. No time-based estimates.
2. Relay and messaging remain coupled.
3. Every node is a relay-capable network participant.
4. Internet improves transport and bootstrap but is never a hard dependency.
5. Keep policy in Core; keep platform adapters thin.
6. Complete started functionality before adding net-new features.

---

## 2) Locked product decisions for v0.2.0

These are confirmed and treated as contract inputs:

1. Feature scope is text messaging only.  
   Large sync batches (1000+ messages) are reliability scenarios, not new media scope.
2. All nodes/relays are open.  
   User can disable mesh participation entirely, which disables inbound/outbound messaging and relay participation.
3. Messages must retry indefinitely until delivered.
4. Route expansion is not fixed-width; prioritize peers/relays by best "recipient seen most recently" signal.
5. No TTL-based give-up for undelivered messages.
6. Priority objective is highest delivery success first; balanced optimization later.
7. Network remains decentralized; headless/identity-less nodes provide rendezvous/backhaul utility.
8. End-to-end encryption is mandatory for user message content.
9. Platform coverage target is broad (mobile + desktop families), with legacy-device usefulness as a design objective.
10. Public beta readiness is required (TestFlight and equivalent channels).
11. Headless mode must have parity in relay/network behavior; UI omits user identity panes when identity is absent.
12. Message history should behave as immutable/distributed history while preserving reliable delivery semantics.

Follow-up locks:

1. Message payload target cap: 8 KB.
2. Disk pressure policy: dynamic quota + rolling purge; never let SCM storage push device past 90% used.
3. Anti-abuse: low effort, high reward guardrails only for alpha.
4. iOS floor: keep iOS 17 for v0.2.0.
5. Desktop target: full GUI parity (not CLI-only parity).

---

## 3) Numeric clarifications

1. `8 KB` = `8192` bytes.
2. For ASCII payloads, max length is `8192` characters.
3. For UTF-8 multi-byte characters, max character count is lower than `8192`.

Current code reference to update:

- `core/src/message/codec.rs` currently sets `MAX_PAYLOAD_SIZE = 64 * 1024`.

---

## 4) Verified baseline gaps to close before milestone completion

1. Core compile is currently broken due to type mismatch and lock misuse.
2. Retry pipeline is finite/capped in active swarm path; terminal failure branch exists.
3. Relay forwarding path can reject when destination is not connected, instead of custody storage.
4. Android WiFi send path includes explicit "Not fully implemented" behavior.
5. iOS Multipeer transport exists but is not fully wired into active send path.
6. Mobile startup currently assumes non-headless operation when swarm starts.
7. Existing outbox/relay retention behavior is not aligned with no-give-up delivery semantics.
8. Documentation and runtime behavior currently drift in key areas.

---

## 5) v0.2.0 non-negotiable outcomes

1. Build integrity: workspace compile, tests, and platform builds pass.
2. Direct-first routing with deterministic fallback order.
3. Infinite eventual retry for undelivered messages.
4. Relay custody store-and-forward integrated in active transport path.
5. Delivery receipt propagation stops duplicate concurrent retry attempts.
6. Android WiFi direct path and iOS Multipeer path are both fully wired.
7. Headless-default behavior when no identity is present.
8. Desktop GUI reaches functional parity with mobile core workflows.
9. Dynamic disk safety policy prevents SCM from pushing device beyond 90% utilization.

---

## 6) Dynamic storage and purge policy (locked)

### 6.1 Objectives

1. Keep eventual delivery as strong as possible.
2. Prevent device storage exhaustion.
3. Prioritize local identity relevance.

### 6.2 Dynamic quota model

Definitions:

- `T`: total device bytes
- `U`: current used device bytes
- `S`: current SCM storage bytes
- `N = U - S`: non-SCM used bytes
- `hard_ceiling = floor(0.90 * T) - N`

`hard_ceiling` is the absolute maximum SCM can use without pushing device above 90% total utilization.

Dynamic target quota:

- If device used <= 20%: target up to 70% of free bytes, bounded by `hard_ceiling`.
- If device used 20-50%: target up to 45% of free bytes, bounded by `hard_ceiling`.
- If device used 50-70%: target up to 25% of free bytes, bounded by `hard_ceiling`.
- If device used 70-80%: target up to 10% of free bytes, bounded by `hard_ceiling`.
- If device used 80-90%: target up to 3% of free bytes, bounded by `hard_ceiling`.
- If device used > 90%: emergency mode; reject new non-critical writes and immediately purge until under bound.

### 6.3 Rolling purge order

When `S > target_quota`, purge oldest-first in this order:

1. Non-identity-related messages.
2. Identity-related messages.

Within each group, purge ordering:

1. Delivered/acknowledged records first.
2. Undelivered records only when necessary under sustained pressure.

Identity-related means `to` or `from` matches local identity.

---

## 7) Route preference policy (locked)

Primary goal: maximize delivery success.

Path order:

1. Direct recipient connection attempt.
2. Relay candidates sorted by:
   - recipient recency signal ("seen recipient last"),
   - relay success score,
   - tie-break by latest successful path.
3. Attempt candidates one-at-a-time in ranked order.
4. If all known candidates fail in current pass, continue cyclic retries indefinitely while refreshing candidate graph.

No terminal "exhausted" final state for undelivered messages.

---

## 8) Completion inventory for started-but-incomplete systems

These systems are already present in code and must be completed in v0.2.0:

1. Core build and UniFFI surface consistency.
2. Routing engine integration into live send path.
3. Relay mailbox custody integration with active swarm loop.
4. Infinite retry state machine replacing finite cap.
5. Android WiFi direct send pipeline.
6. iOS Multipeer send/receive pipeline integration.
7. Headless startup mode parity across app variants.
8. Desktop GUI parity with mobile user workflows.
9. Storage pressure controls and global purge propagation.
10. Receipt fanout convergence for concurrent forwarders.

---

## 9) Workstreams with LoC estimates only

LoC ranges below include implementation + tests + minimal docs updates for each stream.

### WS0 - Trunk recovery and contract consistency

Scope:

1. Fix compile blockers in core.
2. Align UniFFI return types and constructors for contacts/history surfaces.
3. Ensure CI blocks merges when workspace compile fails.

Estimate:

- `180-320 LoC`

Acceptance:

1. `cargo test --workspace --no-run` succeeds.
2. Android and iOS verification builds pass.

### WS1 - Message cap and infinite retry engine

Scope:

1. Set payload cap to 8 KB.
2. Replace finite retry cap with persisted infinite retry behavior.
3. Remove terminal delivery-exhausted branch for undelivered message flows.

Estimate:

- `450-700 LoC`

Acceptance:

1. Undelivered messages continue retries across app restarts.
2. No terminal drop due to retry counter exhaustion.

### WS2 - Route preference integration (direct-first + recency ranking)

Scope:

1. Wire routing policy into live swarm send path.
2. Build recipient recency scoring from ledger/identity signals.
3. Add deterministic route decision logging/reason codes.

Estimate:

- `500-850 LoC`

Acceptance:

1. Each send attempt records selected route and reason.
2. Route ordering follows locked ranking policy.

### WS3 - Relay custody store-and-forward in active path

Scope:

1. Integrate relay server/client custody semantics with active swarm path.
2. Accept and persist messages when destination is offline.
3. Deliver on pull/reconnect and mark custody transitions.

Estimate:

- `850-1300 LoC`

Acceptance:

1. Offline recipient receives message after reconnect without sender manual resend.
2. Relay no longer rejects solely because destination is currently disconnected.

### WS4 - Receipt convergence and retry-stop propagation

Scope:

1. Broadcast delivery completion marker to active forwarders.
2. Purge duplicate pending attempts network-wide when final receipt is observed.
3. Keep message history state coherent with receipt transitions.

Estimate:

- `300-520 LoC`

Acceptance:

1. No repeated forward attempts after final delivery convergence.
2. Pending queues are cleaned on all active forwarders.

### WS5 - Dynamic storage pressure controls

Scope:

1. Implement dynamic quota calculation and pressure modes.
2. Implement rolling purge with identity-priority ordering.
3. Add emergency write-throttle behavior for >90% device usage scenarios.

Estimate:

- `700-1100 LoC`

Acceptance:

1. SCM storage does not push device beyond 90% utilization.
2. Purge ordering matches locked policy.

### WS6 - Android WiFi direct path completion

Scope:

1. Complete WiFi send pipeline where currently partial.
2. Integrate with primary send flow and fallback sequencing.
3. Add transport soak tests for local high-throughput messaging sync.

Estimate:

- `500-850 LoC`

Acceptance:

1. Android local high-throughput transport is functional in normal send path.
2. Fallback to BLE/swarm occurs automatically when unavailable.

### WS7 - iOS Multipeer path completion

Scope:

1. Wire Multipeer transport into active send/receive path.
2. Integrate with existing fallback graph.
3. Add reconnection and throughput tests.

Estimate:

- `520-900 LoC`

Acceptance:

1. iOS local high-throughput path participates in standard delivery flow.
2. Fallback behavior remains deterministic and loss-safe.

### WS8 - Headless-default mode and role parity

Scope:

1. Make no-identity startup enter relay-only mode by default.
2. Ensure full network behavior parity between headless and full nodes.
3. Apply UI role gating: hide Messages/Contacts panes when identity absent.

Estimate:

- `600-1000 LoC`

Acceptance:

1. Fresh install without identity launches relay-only UX.
2. Enabling identity upgrades to full app behavior without network role loss.

### WS9 - Desktop full GUI parity

Scope:

1. Deliver desktop GUI workflows matching mobile core flows:
   - onboarding/identity,
   - contacts,
   - chat/send/receive,
   - mesh dashboard,
   - relay-only mode.
2. Package for Windows/macOS/Linux in parity-ready form.

Estimate:

- `1800-3000 LoC`

Acceptance:

1. Desktop users can execute all core workflows without CLI fallback.
2. Role-based UI parity (full vs relay-only) is consistent across desktop and mobile.

### WS10 - Minimal anti-abuse guardrails (alpha level)

Scope:

1. Per-peer token bucket/rate limits.
2. Global inflight limits for relay queue protection.
3. Duplicate suppression and cheap abuse heuristics.

Estimate:

- `300-550 LoC`

Acceptance:

1. Single abusive peer cannot starve relay processing.
2. Legitimate low-volume users remain unaffected.
3. Guardrail trigger reasons are observable in runtime logs.

### WS11 - Public beta readiness surfaces

Scope:

1. Delivery-state UX clarity (`pending`, `stored`, `forwarding`, `delivered`).
2. Diagnostics export quality for family/beta testers.
3. App-store/tester-facing reliability notes and permissions rationale.

Estimate:

- `550-900 LoC`

Acceptance:

1. Testers can understand message state without developer guidance.
2. Install + first-message path is reliable and observable.

Execution note (2026-03-03):

1. WS11 implemented in Android+iOS surfaces with explicit delivery-state mapping, tester-focused diagnostics bundles, and permissions/reliability rationale text.
2. Residual-risk review and evidence are recorded in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` under the WS11 section.

### WS12 - Test matrix expansion and docs parity lock

Scope:

1. Expand deterministic integration tests for offline/partition scenarios.
2. Add cross-platform parity checks for role mode and transport fallback.
3. Update canonical docs/backlog to match runtime behavior.

Estimate:

- `700-1200 LoC`

Acceptance:

1. Cross-platform parity checks pass in CI.
2. Canonical docs no longer contradict runtime behavior for milestone scope.

---

## 10) Total v0.2.0 planning estimate

Planned implementation envelope:

- Low range: `7,950 LoC`
- High range: `12,390 LoC`

Notes:

1. Estimate intentionally excludes generated code.
2. Estimate includes tests and required documentation updates.
3. If desktop packaging complexity expands, additional LoC should be tracked in a scoped addendum.

---

## 11) Definition of Done (v0.2.0)

v0.2.0 is done when all are true:

1. Workspace compiles cleanly; core/mobile/desktop build gates pass.
2. Messages retry indefinitely until delivered (no terminal retry exhaustion).
3. Direct-first routing and recency-based relay fallback are active.
4. Relay custody path stores offline messages and delivers on reconnect.
5. Receipt propagation halts duplicate forward attempts across participating nodes.
6. Android WiFi direct and iOS Multipeer paths are fully wired.
7. Headless-default startup works with UI/network role parity.
8. Desktop GUI parity is available across Windows/macOS/Linux targets.
9. Dynamic storage policy protects device usage ceiling and purges by locked ordering.
10. Public beta readiness criteria are satisfied for external testers.
11. Residual risk closure sweep is complete: each risk in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` is either `Closed` or explicitly `Accepted/Deferred` with rationale and verification evidence.

---

## 12) Immediate execution order

To reduce risk and unblock integration quickly:

1. WS0
2. WS1
3. WS2
4. WS3
5. WS4
6. WS5
7. WS6 + WS7 (in parallel where possible)
8. WS8
9. WS9
10. WS10
11. WS11
12. WS12
13. Post-WS12 residual risk closure sweep (release gate)
14. WS13 kickoff (v0.2.1) - Single Active Device/Tight Pairing

This ordering is for dependency flow only. Execution planning remains LoC-scoped, not time-scoped.

---

## 13) Residual Risk Tracking (Active)

Residual risks discovered during v0.2.0 execution are tracked in:

- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Each phase must update the register before phase signoff, and the release gate must run a final closure sweep after WS12.

---

## 14) Post-WS12 Residual Risk Closure Sweep (Release Gate)

Run this sweep immediately after WS12 and before calling v0.2.0 complete:

1. Enumerate all open residual risks from `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`.
2. Attempt closure of release-critical risks that can be fixed without net-new feature scope.
3. Re-run verification for any risk addressed during the sweep.
4. For remaining open risks, classify as `Accepted` or `Deferred` with explicit rationale.
5. Record a final go/no-go decision for external alpha distribution based on residual-risk posture.

This sweep is mandatory and is part of v0.2.0 Definition of Done.

---

## 15) Deferred Net-New Feature (Tracked for v0.2.1)

Feature:

- Single Active Device per Identity ("Tight Pairing Architecture")

Decision:

- Deferred from `v0.2.0` to `v0.2.1`.
- Execution anchor in next release: `WS13` (decomposed as `WS13.1`-`WS13.6`).

Why deferred:

1. It is cross-cutting net-new scope (identity schema, contacts metadata, transport protocol, relay registry state machine, custody enforcement, UX error semantics).
2. It conflicts with the `v0.2.0` policy to complete started functionality first.
3. Introducing it during WS6-WS12 would materially increase regression risk for alpha app-store readiness.

Tracking document:

- `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`
- WS13 kickoff prompt template: `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md` (section "WS13 Kickoff (v0.2.1) - Tight Pairing start")

`v0.2.0` rule:

1. Planning/docs and backlog tracking are allowed.
2. Enforcement/protocol implementation is out-of-scope unless milestone is explicitly re-scoped.
