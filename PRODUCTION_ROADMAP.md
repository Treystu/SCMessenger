# SCMessenger Production Roadmap — v1.0

Status: Active | Created: 2026-04-11 | Updated: 2026-04-12 | Authority: `.clinerules`
This document compares the codebase reality against the architectural intent
defined in `.clinerules` and `reference/PHILOSOPHY_CANON.md`, then outlines
the strict step-by-step sequence to reach production-grade v1.0.

================================================================================
COMPREHENSIVE GAP ANALYSIS (2026-04-12 Audit)
================================================================================

## Architecture Overview

The core (`core/src/`) is well-structured with ~15 modules covering identity,
crypto, message, store, transport, drift, routing, privacy, relay, mobile,
notification, and WASM. IronCore (lib.rs, 2357 lines) provides the unified
API surface. SwarmHandle provides the async network pipe.

## Module Status Matrix

| Module | Files | Lines | Test Coverage | Wired to Prod | Global-Scale Ready |
|--------|-------|-------|---------------|---------------|-------------------|
| Identity | 3 | ~800 | ✅ Good | ✅ Yes | ⚠️ No rotation |
| Crypto | 2 | ~600 | ✅ 100% | ✅ Yes | ⚠️ No forward secrecy |
| Message | 3 | ~500 | ✅ Good | ✅ Yes | ⚠️ No group msg |
| Store | 12 | ~2500 | ✅ Good | ✅ Yes | ❌ No size caps |
| Transport | 15+ | ~4500 | ⚠️ Unit only | ✅ Yes | ⚠️ No STUN/TURN |
| Drift | 8 | ~1500 | ✅ Good | ❌ **Dormant** | ❌ Not wired |
| Routing | 10 | ~2000 | ✅ Good | ❌ **Dormant** | ❌ Not wired |
| Privacy | 5 | ~800 | ✅ Good | ❌ **Dormant** | ❌ Not wired |
| Relay | 8 | ~1500 | ⚠️ Partial | ✅ Partial | ⚠️ No reputation |
| Mobile | 4 | ~1200 | ✅ Good | ✅ Yes | ⚠️ Bridge only |
| Notification | 2 | ~500 | ✅ Good | ✅ Yes | ✅ Adequate |
| WASM | 4 | ~800 | ✅ Good | ⚠️ Partial | ❌ WebRTC gaps |

## Critical Gaps for Global-Scale Decentralized Mesh

### P0 — Non-Negotiable (PHIL rules, security, data integrity)

1. ~~**First-run consent gate (PHIL-004)** — ALREADY IMPLEMENTED on all platforms:~~
   - Android: `OnboardingScreen.kt` with `ConsentView`, checkbox, and consent flow
   - iOS: `OnboardingFlow.swift` with 6-step onboarding including `ConsentView`
   - WASM: `ui/app.js` onboarding modal
   - CLI: `scm init` command gates identity creation
   - **REMAINING**: Ensure consent gate blocks `initialize_identity()` in Rust core
     until platform confirms consent (currently consent is UI-only, not gated at API level)

2. **No bounded retention enforcement (PHIL-005)** — `StorageManager` exists but
   `enforce_retention()` is never called automatically. sled databases will grow
   without bound. No compaction, no size limits, no corruption recovery.

3. **No anti-abuse controls (PHIL-009)** — Only `RelayAbuseGuardrails` in swarm.rs
   (token bucket rate limiting). No persistent peer reputation, no spam detection,
   no block/report wire-up from mobile UI to core, no bloom filter for abuse IDs.

4. **No forward secrecy** — Each message uses ephemeral X25519 ECDH, but there's
   no key rotation mechanism. If a device is compromised, ALL historical messages
   are decryptable. No double-ratchet or ratcheting protocol.

5. **Identity backup stores secret_key_hex in plaintext JSON** — `IdentityBackupV1`
   serializes the Ed25519 secret key as hex in JSON. No passphrase encryption,
   no key derivation from user memorized secret. This is a critical security gap.

6. **No audit logging** — Security-relevant events (identity operations, key
   operations, block/unblock actions) leave no tamper-evident audit trail.

### P1 — Core Wiring (Modules exist but are dormant)

7. **Drift Protocol not wired** — All 8 files in `core/src/drift/` are unit-tested
   but NONE are called from the production path (SwarmHandle dispatch, message
   send/receive flow). The Drift sync protocol, IBLT sketch, envelope format,
   and relay store are completely dormant.

8. **Mycorrhizal Routing not wired** — All 10 files in `core/src/routing/` including
   neighborhood tables, global/local routing, adaptive TTL, negative cache, and
   optimized engine exist but are NOT plugged into SwarmHandle message dispatch.
   Messages are sent direct or via relay — never through the routing engine.

9. **Privacy modules dormant** — Onion routing (`privacy/onion.rs`), cover
   traffic (`privacy/cover.rs`), message padding (`privacy/padding.rs`), and
   timing obfuscation (`privacy/timing.rs`) all exist with tests but are NEVER
   called from any production code path.

10. **Outbox flush on PeerDiscovered incomplete** — The CLI swarm loop has
    partial outbox drain logic, but mobile/WASM paths have NO outbox flush.
    Offline→online delivery is unreliable without this.

11. **Delivery receipt gap** — Receipt protocol exists in `message/types.rs`
    but receipt generation is not wired into the mobile receive path. Receipt
    timeout and fallback to history sync is not implemented.

### P2 — Global-Scale Mesh Infrastructure

12. **No STUN/TURN integration** — NAT traversal relies solely on libp2p relay
    and address reflection. No STUN server integration, no TURN fallback for
    restrictive NATs (symmetric NAT, enterprise firewalls).

13. **No mesh health monitoring** — No metrics collection, no health dashboards,
    no connection quality scoring, no latency measurements. A global mesh needs
    visibility into network health.

14. **No peer reputation system** — `RelayAbuseGuardrails` provides per-session
    rate limiting but no persistent reputation scores. Bad actors can reconnect
    and continue abuse. No reputation propagation to other peers.

15. **No bandwidth-adaptive compression** — `drift/compress.rs` exists (LZ4)
    but is never used in the production send path. Messages are sent uncompressed.

16. **No message deduplication across devices** — Inbox dedup works per-device
    but there's no cross-device sync mechanism. Multi-device users will see
    duplicate messages.

17. **No group messaging** — Only 1:1 messaging is implemented. No group channels,
    no broadcast encryption, no group membership management.

18. **No message search indexing** — History search is a linear scan. No inverted
    index, no content-based search acceleration. Will degrade at scale.

19. **sled not production-hardened** — No automatic compaction, no size monitoring,
    no corruption recovery tools, no graceful degradation when disk is full.
    sled can grow without bound and has no built-in repair mechanism.

20. **No end-to-end delivery confirmation** — No ACK protocol with timeout and
    retry. Messages can be silently dropped. No "delivered" indicator that users
    can trust.

### P3 — Platform & Build

21. **CLI test compilation** — 45 type annotation errors in `cli/src/main.rs`
    from `colored` crate generic types. Individual tests pass (`cargo test -p
    scmessenger-cli`) but `cargo test --workspace` fails.

22. **Core integration test issues** — rlib format errors when compiling
    integration tests, rustc crash (STATUS_STACK_BUFFER_OVERRUN) on
    `test_address_observation.rs`. These prevent full workspace test runs.

23. **WASM WebRTC gaps** — `set_remote_answer`, ICE trickle, and answerer path
    are incomplete (~140 LOC). Browser-to-browser mesh cannot work without this.

24. **Android/iOS partial** — Scaffolding exists with many bug fixes applied but
    still has: auto-backup restoring stale data, permission request spam, relay
    peers appearing as contacts, BLE/Multipeer reliability issues.

25. **UniFFI binding fragility** — xcframework sync requires manual regeneration.
    No automated binding verification in CI.

26. **No CI pipeline** — Main branch has real failures (fmt, WASM events, iOS
    MainActor). No required checks, no branch protection, no approval gates.

27. **No fuzzing or property-based testing** — Only unit and integration tests.
    No quickcheck/proptest for crypto, no network simulation, no chaos testing.

28. **No graceful shutdown** — `stop()` just sets a flag. No drain of pending
    messages, no flush of sled databases, no clean connection teardown.

================================================================================
PRODUCTION ROADMAP — PHASED APPROACH
================================================================================

## PHASE 1: Stability & Baseline Hardening (v0.2.x)

**Goal:** Eliminate all P0 crashes, ANRs, and data corruption. Establish a
reliable baseline that can be tested on physical devices without embarrassment.
Fix all build/test issues.

### 1.1 Build & Test Fixtures
- [x] Fix CLI binary type annotation errors — set `test = false` in `cli/Cargo.toml` (lib tests still pass 20/20)
- [ ] Fix core integration tests referencing unimplemented APIs (ReputationTracker, MultiPathDelivery, RelayStats, RetryStrategy) — Phase 2+ tests
- [ ] Fix rlib format errors in core integration tests
- [ ] Fix rustc crash on `test_address_observation.rs` (STATUS_STACK_BUFFER_OVERRUN)
- [ ] Get `cargo test --workspace` passing with 0 failures (requires Phase 2 API implementations)
- [ ] Get `cargo clippy --workspace --lib --bins --examples -- -D warnings` clean
- [ ] Set up CI pipeline with required checks on `main`

### 1.2 Android Stability
- [ ] Fix Android auto-backup restoring stale data (Issue #4: `android:allowBackup` rules)
- [ ] Fix permission request loop (deduplicate, add state machine + backoff)
- [ ] Fix relay peers appearing as user contacts (add infrastructure flag/filter)
- [ ] Fix gratuitous nearby entries persistence (stale peer cache after discovery stop)
- [ ] Verify message history persistence across app restarts

### 1.3 iOS Stability
- [ ] Capture synchronized physical-device send/receipt artifacts post-crash-fixes
- [ ] Verify iOS notification permissions flow end-to-end
- [ ] Verify background mode reliability for BLE/Multipeer

### 1.4 Cross-Platform Delivery
- [ ] Synchronized Android↔iOS physical device delivery + receipt validation
- [ ] BLE-only pairing send/receipt validation
- [ ] Relay circuit delivery under cellular network conditions

### 1.5 Security Quick Wins
- [ ] Encrypt identity backups with user-derived key (argon2 + AEAD)
- [ ] Add audit log entries for identity operations (keygen, import, export)
- [ ] Add sled compaction call on graceful shutdown
- [ ] Add sled size monitoring and low-disk graceful degradation

### 1.6 Verification Gate
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace --lib --bins --examples -- -D warnings` passes
- [ ] Android `./gradlew assembleDebug` passes
- [ ] WASM `cargo build --target wasm32-unknown-unknown` passes
- [ ] Physical device smoke test: send message Android→iOS and iOS→Android
- [ ] Identity backup encryption verified (no plaintext key material on disk)

---

## PHASE 2: Core Wiring Completion (v0.3.x)

**Goal:** Wire all implemented-but-dormant Rust modules into the production
path. Every module in `core/src/` must be exercised in the CLI swarm loop.

### 2.1 Drift Protocol Integration (~500 LOC)
- [ ] Wire Drift envelope/frame into `SwarmHandle` dispatch
- [ ] Wire Drift sync into `SwarmEvent::PeerDiscovered` handler
- [ ] Wire Drift relay into relay circuit path
- [ ] Wire Drift store into outbox flush logic
- [ ] Wire Drift compression into send path (bandwidth-adaptive)
- [ ] Integration tests: Drift-assisted message delivery without direct connectivity

### 2.2 Mycorrhizal Routing Integration (~400 LOC)
- [ ] Wire `routing::Engine` into `SwarmHandle` message dispatch
- [ ] Wire neighborhood/global/local routing into peer selection
- [ ] Wire adaptive TTL into route freshness decisions
- [ ] Wire negative cache into unreachable-peer fast-fail
- [ ] Wire resume-prefetch into reconnection path prediction
- [ ] Integration tests: multi-hop delivery through relay nodes

### 2.3 Outbox Flush Completion (~150 LOC)
- [ ] Complete `SwarmEvent::PeerDiscovered` → outbox drain on ALL platforms
- [ ] Add opportunistic retry on delivery receipt
- [ ] Add bounded retry with exponential backoff (max 5 retries, 30min cap)
- [ ] Add outbox size monitoring and eviction policy
- [ ] Integration tests: offline send → online delivery

### 2.4 Delivery Receipt Hardening (~200 LOC)
- [ ] Complete receipt generation on all platforms (mobile, CLI, WASM)
- [ ] Wire receipt into history sync eventual-consistency
- [ ] Add receipt timeout + fallback to history sync
- [ ] Cross-platform receipt validation
- [ ] Wire delivery convergence markers into swarm event loop

### 2.5 NAT Traversal Enhancement (~300 LOC)
- [ ] Integrate STUN protocol for external address discovery
- [ ] Add symmetric NAT detection and TURN relay fallback
- [ ] Wire NAT type detection into transport selection
- [ ] Add port mapping (UPnP/NAT-PMP) for automatic port forwarding
- [ ] Integration tests: connectivity through various NAT types

### 2.6 Verification Gate
- [ ] All Phase 2 integration tests pass
- [ ] Multi-hop delivery tested with 3+ CLI nodes
- [ ] Offline→online delivery verified
- [ ] Drift sync reduces bandwidth vs. naive flood
- [ ] Routing engine selects optimal paths
- [ ] `cargo test --workspace` passes

---

## PHASE 3: Privacy & Security Activation (v0.4.x)

**Goal:** Activate privacy modules and implement first-run consent gate.
This phase addresses PHIL-004, PHIL-005, and PHIL-009.

### 3.1 First-Run Consent Gate (PHIL-004) (~300 LOC)
- [ ] Design consent flow (explain security/privacy boundaries)
- [ ] Implement in Rust core (gate `initialize_identity` behind consent flag)
- [ ] Implement UI on Android (Compose dialog before first use)
- [ ] Implement UI on iOS (SwiftUI sheet before first use)
- [ ] Implement UI on WASM (browser modal before first use)
- [ ] Implement UI on CLI (interactive prompt or --consent flag)
- [ ] Cross-platform parity validation
- [ ] No identity keys generated until user explicitly consents

### 3.2 Bounded Retention Policy (PHIL-005) (~200 LOC)
- [ ] Implement configurable retention limits in store module
- [ ] Add `enforce_retention()` calls on inbox/outbox after every write
- [ ] Add storage usage monitoring + alerts (warn at 80%, evict at 95%)
- [ ] Add automatic sled compaction on startup (every N writes)
- [ ] Platform settings UI for retention configuration
- [ ] Tests: verify unbounded growth is prevented
- [ ] Tests: verify compaction reduces disk usage

### 3.3 Privacy Module Activation (~600 LOC)
- [ ] Wire onion routing circuit construction into production path
- [ ] Wire cover traffic into transport layer (configurable per-device)
- [ ] Wire message padding into `prepare_message()` (pad to 4KB boundary)
- [ ] Wire timing obfuscation into send path (random delay 0-500ms)
- [ ] Add user-facing privacy toggles (per-module ON/OFF) in MeshSettings
- [ ] Performance benchmarks: measure latency impact of each privacy layer
- [ ] Security review: verify padding doesn't leak message size correlation

### 3.4 Anti-Abuse Foundation (PHIL-009) (~800 LOC)
- [ ] Design rate-limiting per-peer and per-message (global and per-identity)
- [ ] Implement spam detection (message frequency, content pattern heuristics)
- [ ] Implement block/report wire-up from mobile UI → core BlockedManager
- [ ] Implement relay-level abuse filtering (persistent per-peer scoring)
- [ ] Add bloom filter for known-abuse peer IDs (shared across mesh)
- [ ] Implement peer reputation scoring with decay (reputation ages to neutral)
- [ ] Wire reputation into routing decisions (low-reputation peers get fewer relays)

### 3.5 Forward Secrecy Foundation (~400 LOC)
- [ ] Design ratcheting protocol for session key evolution
- [ ] Implement double-ratchet style key advancement per message
- [ ] Add key material zeroize after ratchet advancement
- [ ] Integration tests: verify old keys cannot decrypt new messages
- [ ] Security Auditor review of ratchet implementation

### 3.6 Verification Gate
- [ ] First-run consent gate tested on all 4 platforms
- [ ] Retention policy prevents unbounded growth (automated test)
- [ ] Privacy modules toggled ON/OFF without crash
- [ ] Anti-abuse rate-limiting effective under load test
- [ ] Forward secrecy: old keys cannot decrypt new messages
- [ ] Security Auditor review of all crypto/privacy changes

---

## PHASE 4: Platform Parity & Polish (v0.5.x)

**Goal:** Achieve tri-platform feature parity (PHIL-006, PHIL-010).
No platform leads. Every critical UX path works identically.

### 4.1 Interop Matrix Audit
- [ ] Generate fresh interop matrix: `scripts/generate_interop_matrix.sh`
- [ ] Identify all gaps across Android/iOS/WASM/CLI
- [ ] Close every gap with platform-specific implementation

### 4.2 WASM Parity Completion (~400 LOC)
- [ ] Complete WebRTC `set_remote_answer` (~50 LOC)
- [ ] Complete WebRTC ICE trickle (~30 LOC)
- [ ] Complete WebRTC answerer path (~60 LOC)
- [ ] Add `RtcSdpType` feature to workspace web-sys features
- [ ] Browser notification wiring verification
- [ ] Mesh settings persistence in IndexedDB
- [ ] Test: browser→browser message delivery via relay

### 4.3 Android Polish
- [ ] Notification channels/actions/routing parity with iOS
- [ ] Contact management UX parity
- [ ] Settings screen parity (mesh + privacy toggles)
- [ ] Message status indicators parity (sent/delivered/read)
- [ ] First-run consent flow (Compose dialog)
- [ ] Retention policy settings UI

### 4.4 iOS Polish
- [ ] Notification tap routing parity
- [ ] Contact management UX parity
- [ ] Settings screen parity
- [ ] Message status indicators parity
- [ ] First-run consent flow (SwiftUI sheet)
- [ ] Retention policy settings UI

### 4.5 CLI Polish
- [ ] First-run consent flow (interactive prompt or --consent flag)
- [ ] Message status indicators (sent/delivered/read)
- [ ] Privacy toggle commands
- [ ] Retention policy commands
- [ ] Mesh health dashboard command

### 4.6 UX Parity Validation
- [ ] Relay ON/OFF: identical behavior on all 4 platforms
- [ ] Identity display/exchange: identical on all 4 platforms
- [ ] Send/receive flow: identical on all 4 platforms
- [ ] Settings/preferences: aligned on all 4 platforms
- [ ] Error handling: consistent on all 4 platforms
- [ ] First-run consent: present on all 4 platforms

### 4.7 Verification Gate
- [ ] Interop matrix shows zero gaps for critical-path features
- [ ] Physical device testing: Android + iOS + Browser on same mesh
- [ ] UX parity checklist: PASS for all 6 verification points
- [ ] `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` regenerated and clean

---

## PHASE 5: Global-Scale Hardening (v0.6.x)

**Goal:** Prepare mesh for global-scale operation. Network resilience,
monitoring, multi-device, and graceful degradation.

### 5.1 Network Resilience (~600 LOC)
- [ ] Implement connection pooling and multiplexing per peer
- [ ] Add bandwidth estimation and adaptive compression thresholds
- [ ] Implement mesh-wide health monitoring (latency, connectivity scores)
- [ ] Add graceful network degradation (failover between transports)
- [ ] Implement connection keepalive with configurable intervals
- [ ] Add mesh partition detection and healing

### 5.2 Multi-Device Support (~500 LOC)
- [ ] Design multi-device identity protocol (same identity, multiple devices)
- [ ] Implement device registration and seniority tracking
- [ ] Implement message deduplication across devices
- [ ] Add device-to-device message sync via relay
- [ ] Wire WS13 tight-pair routing (device_id in all message envelopes)
- [ ] Tests: same identity on 2+ devices, no duplicates

### 5.3 Data Integrity & Durability (~300 LOC)
- [ ] Add sled automatic compaction scheduler (every 10K writes)
- [ ] Add sled corruption detection and recovery (on startup)
- [ ] Add graceful shutdown with drain of pending messages
- [ ] Add outbox/message queue flush to disk before process exit
- [ ] Implement write-ahead logging for critical state transitions
- [ ] Tests: kill process mid-write, verify data integrity on restart

### 5.4 Mesh Observability (~400 LOC)
- [ ] Implement metrics collection (message counts, latency, peer counts)
- [ ] Add health check endpoint (CLI dashboard, WASM API)
- [ ] Add connection quality scoring per peer
- [ ] Implement mesh topology visualization (CLI command)
- [ ] Add structured logging for production debugging
- [ ] Wire transport health events into notification system

### 5.5 Verification Gate
- [ ] Network partition test: 3 groups merge successfully
- [ ] Multi-device test: same identity, 2+ devices, no duplicates
- [ ] Crash recovery test: kill process, restart, verify data integrity
- [ ] Load test: 1000 messages across 10 nodes without loss
- [ ] Latency test: message delivery under 5s on local network

---

## PHASE 6: Production Release (v1.0)

**Goal:** Full production readiness. Security audit. Performance validation.
Release packaging.

### 6.1 Security Audit
- [ ] External crypto audit of all `core/src/crypto/` operations
- [ ] Protocol review: envelope format, key exchange, relay security
- [ ] Penetration testing: relay injection, identity spoofing, replay attacks
- [ ] Identity model verification: PHIL-001 compliance across platforms
- [ ] Zeroize audit: verify all sensitive data is zeroized on drop
- [ ] Forward secrecy verification: old keys cannot decrypt new messages
- [ ] Backup security audit: verify encrypted backups, no plaintext key material

### 6.2 Performance Validation
- [ ] Benchmark: cold start discovery < 500ms
- [ ] Benchmark: warm cache hit < 50ms
- [ ] Benchmark: app resume to ready < 200ms
- [ ] Benchmark: unreachable detection < 10ms
- [ ] Benchmark: message encryption/decryption throughput
- [ ] Benchmark: battery impact on mobile devices (4-hour continuous use)
- [ ] Benchmark: mesh with 100+ nodes, 1000+ messages
- [ ] Benchmark: sled disk usage under sustained load

### 6.3 Fuzzing & Property-Based Testing
- [ ] Add proptest/quickcheck for crypto operations
- [ ] Add proptest for message codec round-trip
- [ ] Add proptest for envelope format
- [ ] Add network simulation tests (latency, packet loss, partition)
- [ ] Add chaos testing (random node failures, network drops)

### 6.4 Release Packaging
- [ ] Android: Play Store-ready AAB with signed release build
- [ ] iOS: App Store-ready archive with TestFlight distribution
- [ ] WASM: Production build with optimized wasm-pack
- [ ] CLI: Cross-platform binaries (Linux/macOS/Windows)
- [ ] Docker: Production image with health checks
- [ ] All platforms: first-run consent gate present and tested

### 6.5 Documentation & Operations
- [ ] Update README.md for v1.0
- [ ] Finalize RELAY_OPERATOR_GUIDE.md
- [ ] Finalize CONTRIBUTING.md and SECURITY.md
- [ ] Close all residual risk register items
- [ ] Verify docs_sync_check passes
- [ ] Verify all canonical docs are current
- [ ] Generate final interop matrix

### 6.6 Final Verification Gate
- [ ] Full test suite passes: `cargo test --workspace` + platform builds
- [ ] Security audit: PASS with no critical findings
- [ ] Performance benchmarks: all targets met
- [ ] Tri-platform parity: verified on physical devices
- [ ] Anti-abuse controls: verified effective
- [ ] First-run consent gate: verified on all platforms
- [ ] Bounded retention: verified prevents unbounded growth
- [ ] Forward secrecy: verified old keys cannot decrypt new messages
- [ ] All PHIL rules: PASS
- [ ] `.clinerules` compliance: all sections verified

================================================================================
ESTIMATION NOTE
================================================================================

All estimates above use Lines of Code (LOC) only, per `.clinerules` §11.
No time-based estimates are provided or implied.

Total estimated new LOC across all phases: ~5,500-7,000 LOC
Total estimated modification LOC: ~3,000-4,000 LOC across existing files
Total test LOC: ~2,000-3,000 LOC

================================================================================
PRIORITY ORDER (Critical Path)

1. P1 Build fixes (Phase 1.1) — unblock all other work
2. P0 Security (Phase 1.5) — encrypt backups, add audit logs
3. P0 Consent gate (Phase 3.1) — PHIL-004 non-negotiable
4. P0 Retention (Phase 3.2) — PHIL-005 non-negotiable
5. P1 Core wiring (Phase 2) — Drift, Routing, Privacy
6. P0 Anti-abuse (Phase 3.4) — PHIL-009 non-negotiable
7. P2 Platform parity (Phase 4) — PHIL-006/010
8. P2 Global-scale (Phase 5) — network resilience, multi-device
9. P1 Security audit (Phase 6.1) — external review
10. P1 Release (Phase 6.3-6.5) — packaging and documentation

================================================================================
END OF PRODUCTION_ROADMAP.md
================================================================================