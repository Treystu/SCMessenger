# SCMessenger v0.1.2 Alpha Release Plan (Core + iOS + Android + WASM)

## 1) Release intent

Ship **v0.1.2-alpha** as the first partner-usable build where:

1. Core messaging workflows are at the same practical feature level across Core, iOS, Android, and WASM.
2. Identity, contacts, and messages survive app upgrades with no manual repair.
3. Internet connectivity is robust across Wi-Fi/cellular/public networks:
   - direct P2P when reachable,
   - GCP bootstrap/signaling always available,
   - relay data path as guaranteed fallback.

This release optimizes for **continuity, determinism, and resilience** over net-new features.

---

## 2) Non-negotiable alpha outcomes

### A. Persistence continuity across updates
- Identity keys remain stable after app update, restart, and OS reboot.
- Contacts and conversations remain available through the same lifecycle.
- Storage migrations are idempotent, ordered, and crash-safe.
- No silent identity regeneration unless user explicitly resets identity.

### B. Cross-platform parity floor
- Consistent behavior for send/receive, contact add/import, conversation list, retry state.
- Unified inbox/outbox/pending state semantics.
- Equivalent error taxonomy for connection and delivery failures.
- WASM uses libp2p swarm as first-class transport path (no standalone relay-only bypass).

### C. Internet-ready routing behavior
- Reach GCP bootstrap from non-LAN networks.
- Deterministic path sequence:
  1) bootstrap,
  2) direct probe,
  3) relay settle if direct unavailable.
- Telemetry always exposes active path and recent transition reason.

---

## 3) Scope boundaries

### In scope (must ship)
1. Persistence hardening and upgrade-safe migration framework.
2. Transport orchestration: direct P2P + relay fallback + promotion back to direct.
3. Core API/DTO parity across UniFFI and WASM bindings.
4. Release quality gates for partner alpha.

### Out of scope (defer)
- Major cryptographic redesign.
- Brand-new transport families not already integrated.
- UI overhaul unrelated to reliability/parity.
- Rich media beyond minimal metadata support.

---

## 4) Architecture unification + deduplication strategy

### 4.1 Core as source of truth
Move/keep all policy in Core:
- contact merge and identity canonicalization,
- message lifecycle transitions,
- retry/backoff and path escalation,
- connection state machine.

Platform layers (iOS/Android/WASM) should be thin:
- OS-specific secure storage adapters,
- network capability hooks,
- rendering core state/events only.

### 4.2 Canonical shared contracts
Freeze a canonical domain contract for 0.1.2-alpha:
- `IdentityState`
- `ContactRecord`
- `ConversationState`
- `MessageRecord`
- `DeliveryState`
- `ConnectionPathState`

Unify DTO shapes and error taxonomy for Swift/Kotlin/WASM consumers.

### 4.3 Dedup targets
- One contact merge engine in Core.
- One message transition reducer in Core.
- One retry/escalation policy in Core.
- One schema constants module in Core.
- Remove parallel platform-specific policy branches unless OS-constrained.

---

## 5) Workstreams with LOC estimates

> LOC estimates are intentionally conservative ranges for planning and risk control. They include tests for each area.

### Workstream 1 — Persistence and upgrade safety

#### 5.1.1 Canonical data ownership map
- Identity: keypair + metadata + local node settings.
- Contacts: peer identity mapping + trust labels + aliases.
- Messages: immutable payload record + mutable delivery/index metadata.

#### 5.1.2 Schema versioning and migrator contract
- Ordered migration runner per domain.
- Startup lock + fail-fast on irrecoverable migration errors.
- Crash-safe write sequencing with transactional boundaries.
- Downgrade guardrail behavior (explicit UX + logs, no corruption).

#### 5.1.3 Durable semantics
- Atomic message append/index update (or journaled recoverability).
- Idempotent contact mutation operations.
- Write-once core key material with explicit rotate path only.

#### 5.1.4 LOC estimate
- Core migration/runtime logic: **450–700 LOC**
- Platform storage adapter adjustments: **300–500 LOC**
- Migration + restart/upgrade tests: **300–450 LOC**
- **Subtotal: 1,050–1,650 LOC**

### Workstream 2 — Internet transport + orchestrator

#### 5.2.1 Unified orchestrator state machine (Core)
States:
- `Bootstrapping`
- `NodeReachable`
- `DirectProbe`
- `DirectEstablished`
- `RelayEstablished`
- `DegradedRetry`
- `Offline`

Deterministic, event-driven transitions to ensure cross-platform parity.

#### 5.2.2 Bootstrap/direct/relay policy
- Standardize GCP endpoint/TLS policy and backoff strategy.
- Bounded direct probe window with candidate set prioritization.
- Relay as guaranteed data path fallback.
- Background probe for relay->direct promotion.

#### 5.2.3 Message safety across path switch
- Stable message IDs.
- Idempotent receive apply.
- ACK reconciliation across route changes.

#### 5.2.4 LOC estimate
- Core orchestrator + policy logic: **500–850 LOC**
- Transport wiring/refactor in existing modules: **350–650 LOC**
- Integration tests (direct/relay/transitions): **300–500 LOC**
- **Subtotal: 1,150–2,000 LOC**

#### 5.2.5 WASM swarm parity (required)
- Implement `start_swarm_with_config()` on `wasm32` using `SwarmBuilder::with_wasm_bindgen()`.
- Use browser-capable websocket transport (`websocket-websys`) with Noise + Yamux.
- Keep protocol IDs stable:
  - `/sc/message/1.0.0`
  - `/sc/address-reflection/1.0.0`
  - `/sc/relay/1.0.0`
  - `/sc/ledger-exchange/1.0.0`
  - `/sc/id/1.0.0`
- Preserve deterministic wasm command semantics:
  - `Listen`: unsupported on browser transport
  - `GetListeners`: empty list
  - `Dial`, `SendMessage`, `GetPeers`, `SubscribeTopic`, `PublishTopic`: supported

### Workstream 3 — API parity and binding consistency

#### 5.3.1 UniFFI/WASM API consolidation
- Canonical DTO alignment.
- Unified error taxonomy.
- Normalized connection status payloads/events.

#### 5.3.2 Platform bridge cleanup
- iOS/Android repository/view-model bridges consume same event contracts.
- WASM adapter aligns init/recover/flush/reconnect lifecycle.

#### 5.3.3 LOC estimate
- Core API/DTO adjustments: **250–450 LOC**
- Swift/Kotlin/WASM binding/adapter updates: **350–700 LOC**
- Contract tests + fixture updates: **200–350 LOC**
- **Subtotal: 800–1,500 LOC**

### Workstream 4 — Observability and alpha readiness

#### 5.4.1 Operational diagnostics
- Structured logs for path transitions + failure reasons.
- Correlation IDs for session/message tracing.
- User-safe diagnostics export for partner feedback.

#### 5.4.2 LOC estimate
- Core telemetry/logging + export hooks: **180–320 LOC**
- Platform exposure/UI surfacing glue: **120–220 LOC**
- Validation tests/scripts: **80–140 LOC**
- **Subtotal: 380–680 LOC**

### 5.5 Total projected delta
- **Expected implementation LOC (incl. tests): 3,380–5,830 LOC**

---

## 6) Platform-by-platform delivery plan

### Core (Rust)
- Finalize reducers/services for identity/contact/message/transport orchestration.
- Freeze FFI/API for 0.1.2-alpha once parity tests pass.
- Add integration suites for migration continuity and route failover.

### iOS
- Ensure Keychain + app storage continuity through update/restart flows.
- Route all state rendering through core events/contracts.
- Background/foreground wake paths restore queue and connection state.

### Android
- Mirror iOS parity with secure key storage + durable DB semantics.
- Lifecycle-safe reconnect and queue flush behavior.
- Ensure identical delivery state mapping to core status model.

### WASM/Web
- IndexedDB-backed persistence with migration/version support.
- Align lifecycle behavior for tab suspend/resume reconnect.
- Ship browser libp2p swarm runtime (`startSwarm/stopSwarm/sendPreparedEnvelope/getPeers`).
- Keep `startReceiveLoop(relayUrl)` only as deprecated shim that delegates to swarm bootstrap.

---

## 7) Test matrix and quality gates

### 7.1 Automated tests (required)
- Core unit tests:
  - migration chain correctness,
  - contact merge determinism,
  - message lifecycle reducer,
  - orchestrator transition determinism.
- Core integration tests:
  - restart persistence,
  - bootstrap + direct connect,
  - forced relay fallback,
  - relay->direct promotion,
  - route-switch ACK reconciliation.
- Contract tests:
  - Swift/Kotlin/WASM decode/encode parity,
  - status/error mapping parity.
- Browser/native compatibility matrix:
  - Browser `v0.1.2-alpha` ↔ native tag `v0.1.0`
  - Browser `v0.1.2-alpha` ↔ native tag `v0.1.1`
  - Browser `v0.1.2-alpha` ↔ current head

### 7.2 Manual alpha scenarios (required)
- Same-LAN direct messaging.
- Cross-network messaging (Wi-Fi vs cellular).
- Relay-only messaging with direct blocked.
- Upgrade from previous build with active chat history.
- Network switch mid-send.
- extended intermittent-connectivity soak test.

### 7.3 Hard quality gates
1. **Persistence gate:** no data loss in 10/10 upgrade runs.
2. **Connectivity gate:** successful messaging in LAN-direct, cross-network, and relay-only scenarios.
3. **Parity gate:** same acceptance checklist passes on iOS/Android/WASM.
4. **Regression gate:** existing core integration suite remains green.

---

## 8) LOC-based execution sequencing (no time estimates)

### Phase A — Foundation scope (target LOC: 1,200–2,000)
- Deliver Workstream 1 core migration framework and startup safety checks.
- Deliver Workstream 2 orchestrator state model and transition engine.
- Expose canonical orchestrator state/events to UniFFI and WASM bindings.

**Phase A completion criteria**
- Core upgrade simulation suite passes.
- iOS/Android/WASM render shared orchestrator states from the same contract.

### Phase B — Parity and hardening scope (target LOC: 1,400–2,400)
- Complete transport behavior wiring (direct probe, relay fallback, direct promotion).
- Complete API/DTO parity cleanup across bindings and adapters.
- Complete message ACK reconciliation and duplicate-suppression behaviors.

**Phase B completion criteria**
- End-to-end scenarios pass for LAN, cross-network, and relay-only modes.
- No P0/P1 parity blockers remain.

### Phase C — Release readiness scope (target LOC: 600–1,200)
- Complete observability and diagnostics export requirements.
- Complete full regression sweep and checklist signoff artifacts.
- Publish release notes and partner test playbook.

---

## 9) Deliverables checklist for v0.1.2-alpha

- [x] Migration framework validated across Core/iOS/Android/WASM.
- [ ] Identity/contact/message continuity verified across update flows.
- [x] Unified connection orchestrator live across all clients.
- [ ] GCP bootstrap + direct P2P + relay fallback validated.
- [x] WASM swarm path validated (no `wasm32` swarm bail-out).
- [ ] ACK-safe path switching validated (no duplicates/loss on transitions).
- [x] Parity audit closed for core chat workflows.
- [x] Structured diagnostics export available for partner bug reports.
- [x] Release notes + partner test playbook published.

### 9.1 Current execution status (in-repo verifiable)

- Completed:
  - wasm32 core swarm path implemented and compiled.
  - browser swarm wasm bindings implemented (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`).
  - legacy `startReceiveLoop(relayUrl)` converted to deprecated swarm bootstrap shim.
  - CI checks added for wasm32 core+wasm crate compile.
  - docs updated for wasm swarm-first architecture.
  - connection path state contract exported in UniFFI and WASM APIs.
  - structured diagnostics export implemented for mobile and web clients.
  - release notes, parity audit, and partner test playbook documents added.
  - storage layout/schema hardening and persistent outbox/inbox initialization implemented.
  - schema v2 legacy-root migration added for identity/outbox/inbox continuity on upgrade.
  - identity manager now hydrates persisted identity+nickname on startup without auto-generation.
  - continuity tests added for identity restart hydration, legacy migration, contacts/local nickname persistence, and history delivery-state persistence.
  - Android onboarding completion now waits for successful identity+nickname persistence (no premature onboarding bypass).
  - Android and iOS now trigger deferred swarm startup immediately after identity/nickname creation to avoid stalled internet transport after first-run onboarding.
  - CLI headless relay mode now persists a stable libp2p network key (`relay_network_key.pb`) so bootstrap peer IDs survive restarts without requiring user identity initialization.
  - Android+iOS build pipelines revalidated after migration changes.
  - wasm target checks revalidated with WebRTC deprecation cleanup (`set_sdp`).
- Remaining (requires additional feature work and/or external validation):
  - full migration continuity signoff across real mobile/web package upgrade runs.
  - cross-network + relay fallback operational matrix signoff against live infrastructure.
  - ACK/path-switch reliability acceptance across partner scenarios.

---

## 10) Risk register and mitigations

1. **Risk:** platform-specific storage behavior causes silent reset.
   - **Mitigation:** startup storage assertions + continuity test suite.

2. **Risk:** NAT variability reduces direct success rate.
   - **Mitigation:** bounded probe, deterministic fallback, route telemetry.

3. **Risk:** duplicate or out-of-order delivery during path changes.
   - **Mitigation:** stable IDs + idempotent apply + ACK reconciliation.

4. **Risk:** parity drift reappears after fixes.
   - **Mitigation:** shared acceptance script required per platform pre-merge.

5. **Risk:** migration regression discovered late in release window.
   - **Mitigation:** run upgrade simulations at every merge to release branch.

---

## 11) Definition of done (alpha)

v0.1.2-alpha is done when two real users can:

1. Keep identity, contacts, and history across app updates.
2. Communicate reliably from different networks.
3. Continue messaging if direct fails (relay fallback).
4. Observe clear connection state + export diagnostics on issues.

**Success criterion:** all four hold with no critical data-loss or message-loss defects in the alpha partner pilot.

---

## 12) Canonical Remaining-Work Source (Process Lock)

- `REMAINING_WORK_TRACKING.md` is the canonical open-work ledger.
- Open TODO/incomplete/stub items must not live only in ad-hoc docs, comments, or test notes.
- For execution clarity, remaining work is split into two chapters there:
  - `Chapter 1: v0.1.2-alpha Ready` (alpha testing/action blockers).
  - `Chapter 2: v1.0.0 Readiness` (global rollout prerequisites).
- Any newly discovered actionable gap during alpha testing must be added to `REMAINING_WORK_TRACKING.md` in the same change set.
