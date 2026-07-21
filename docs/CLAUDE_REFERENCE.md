# CLAUDE_REFERENCE — On-Demand Detail for Agent Sessions

Status: Active
Last updated: 2026-07-20

Companion to `CLAUDE.md`. CLAUDE.md is re-injected into context every turn, so
it holds only the always-needed high-level rules and points here for detail.
Read the section you need; do not preload the whole file.

---

## 1. Full Build & Test Commands (Windows)

Prefer the `build-verify` skill (`full|rust|android|wasm|compile_gate`) over
running these by hand. Always `export CARGO_INCREMENTAL=0` first (rlib-lock
safety), and never run two cargo/gradle invocations concurrently — a Gradle
target can silently pull a cargo-ndk build in as an upstream dependency.

### Rust workspace

```bash
cargo build --workspace              # full build
cargo check --workspace              # faster iteration
cargo fmt --all -- --check           # format gate
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace               # run all tests
cargo test --workspace --no-run     # compile gate (must pass before task-complete)
cargo test -p scmessenger-core --test integration_e2e   # single integration test
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown  # WASM gate
cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin  # Kotlin bindings
cargo run -p scmessenger-core --features gen-bindings --bin gen_swift   # Swift bindings
```

### Android

```bash
cd android
./gradlew assembleDebug -x lint --quiet
./gradlew :app:testDebugUnitTest              # unit suite (re-enabled 2026-07-06)
./android/install-clean.sh                    # fresh device install
./android/verify-build-setup.sh               # env check
```

Rust cross-compilation requires `cargo-ndk` + targets `aarch64-linux-android`,
`x86_64-linux-android` (plus `armv7-linux-androideabi`, `i686-linux-android`
for full coverage). UniFFI Kotlin bindings generate into
`core/target/generated-sources/uniffi/kotlin` during the Gradle build (not
checked in).

### WASM pack / docs sync

```bash
cd wasm && wasm-pack build --target web
wasm-pack test --headless --firefox
./scripts/docs_sync_check.sh          # or scripts/docs_sync_check.ps1
```

---

## 2. Core Module Map (`scmessenger-core`)

`IronCore` is the single entry point; all state behind `Arc<RwLock<…>>`
(parking_lot).

- `identity/` — Ed25519 key management, creation/restore/backup, seniority
- `crypto/` — X25519 ECDH + XChaCha20-Poly1305, ratcheting, backup, Kani proofs
- `transport/` — libp2p Swarm multi-transport: TCP, QUIC, mDNS, BLE, relay;
  `multiport.rs` = port-ladder machinery; `swarm.rs` = the big event loop
- `drift/` — protocol framing, lz4 compression, relay custody, sync
- `store/` — sled persistence (contacts, inbox/outbox, history, blocked, logs)
- `routing/` — adaptive routing: TTL budgets, multipath, reputation, negative
  cache, smart retry
- `relay/` — bootstrap, relay client/server, delegate prewarm, FindMy, peer
  exchange, invites
- `privacy/` — onion routing, cover traffic, padding, timing obfuscation
- `abuse/` — spam detection, reputation, auto-block
- `notification/` — classification + delivery policy
- `wasm_support/` — JSON-RPC bridge (`rpc.rs`) browser <-> CLI daemon
- `mobile_bridge/` (+ `contacts_bridge/`, `blocked_bridge/`) — UniFFI surface.
  As of 2026-07-06 the swarm FFI surface is ASYNC (`async fn` -> Kotlin
  suspend); internal sync Rust callers must use the `*_blocking` helpers,
  never from a tokio context.

Platform cfg gates: `wasm32` (rexie/IndexedDB, no tokio) · desktop (full
tokio, TCP+QUIC+mDNS+DNS) · Android (full tokio, TCP+QUIC, NO mDNS/DNS).
Features: `gen-bindings`, `wasm`, `kani-proofs`, `phase2_apis`, `test-utils`.

Other crates: `cli/` (daemon + warp web UI on 127.0.0.1:9002 — modules: api,
server, transport_bridge, transport_api, ble_daemon, ble_mesh, config, ledger,
bootstrap, contacts, history) · `wasm/` (browser thin-client over WebSocket
`/ws` JSON-RPC) · `mobile/` (thin UniFFI bridge crate, gen_kotlin/gen_swift bins).

---

## 3. Test Inventory

Integration tests in `core/tests/` (naming: `integration_<domain>_<scenario>`):
e2e, contact_block, offline_partition_matrix, ironcore_roundtrip,
registration_protocol, nat_reflection, relay_custody, retry_lifecycle,
receipt_convergence, all_phases, plus test_address_observation,
test_multiport, test_persistence_restart, test_mesh_routing.

Property-based: `core/src/crypto/proptest_harness.rs`. Formal: Kani proofs
behind `kani-proofs`. Android unit suite: `android/app/src/test/` (101 tests
as of re-enablement 2026-07-06; quarantined stragglers in
`android/app/src/test-quarantine/` — see its README).

---

## 4. Ollama Swarm Detail (the `swarm` backend)

The `swarm` backend of the unified `/orchestrate` command (`docs/ORCHESTRATION.md`
Section 5) drives the ollama pool. Its full operating detail — 6-tier quota
governor, agent routing table, and cloud model roster — was recovered when the old
`/orchestrate` command folded into the unified launcher and now lives in
`.claude/archive/commands/orchestrate-swarm-legacy.md` — fix drift there, not here. Orchestrator reads `HANDOFF/todo|IN_PROGRESS/`,
launches via `.claude/orchestrator_manager.sh pool launch <agent> <task>`;
workers implement one `BATCH_*.md`, run compile gates, move the file to
`HANDOFF/done/`, commit `swarm: completed [Task]`. Fire-and-forget: once the
2 slots fill, commit, arm monitors, exit — the `/loop 30m` cron wakes you.
Swarm-managed processes stop via `pool stop <id>` (bookkeeping), not raw kill.
`ORCHESTRATOR_DIRECTIVE.md` has stale paths/roster — `.claude/agent_pool.json`
and `orchestrate.md` win on conflict.

Windows process-tree kill (for non-pool processes): `kill -9 <PID>; taskkill //F //T //PID <PID>`.
