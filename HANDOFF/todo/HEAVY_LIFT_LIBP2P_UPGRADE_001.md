# HEAVY_LIFT_LIBP2P_UPGRADE_001

**Status:** VERIFIED REMAINING WORK — BLOCKED by breaking API changes
**Agent:** rust-coder (glm-5.1:cloud) + architect-planner (deepseek-v4-pro:cloud)
**Budget:** 5400s+ (HEAVY-LIFT tier, requires Tier 1 window)
**Phase:** v0.2.1+ security hardening
**Source:** `BATCH_SECURITY_CARGO_AUDIT_RESIDUAL.md` dependency chain analysis

---

## Verified Gap

Cargo.lock contains MEDIUM severity vulnerable transitive dependencies that CANNOT be fixed without upgrading `libp2p` from 0.53.2 to 0.56.0+.

**Current Cargo.lock State:**
| Crate | Version | Status | Blocker |
|-------|---------|--------|---------|
| `ring` | 0.16.20 | **VULNERABLE** (MEDIUM AES panic) | libp2p-tls 0.4.1 -> rcgen 0.11.3 |
| `rustls-webpki` | 0.101.7 | **VULNERABLE** (MEDIUM name constraints) | libp2p-tls 0.4.1 |
| `instant` | 0.1.13 | **Unmaintained** | libp2p 0.53.2 |
| `bincode` | 1.3.3 | **Unmaintained** | direct workspace dep (separate task) |

**Dependency Chain (verified):**
```
libp2p 0.53.2
  -> libp2p-quic 0.10.3 (requires libp2p-tls ^0.4.0)
    -> libp2p-tls 0.4.1
      -> rcgen ^0.11.3 -> ring 0.16.20 (VULNERABLE)
      -> rustls-webpki ^0.101.4 -> rustls-webpki 0.101.7 (VULNERABLE)
```

## Scope

### Phase 1: Upgrade Assessment (architect-planner)

1. Research libp2p 0.53 -> 0.56 breaking changes:
   - `libp2p-tls` API changes (0.4.x -> 0.6.x)
   - `libp2p-quic` API changes (0.10.x -> 0.11.x)
   - `libp2p::SwarmBuilder` changes
   - `Transport` trait changes
2. Identify all `core/src/transport/` files that need updating
3. Produce migration plan document in `docs/`

### Phase 2: Dependency Upgrade (rust-coder)

1. Update workspace `Cargo.toml` libp2p version to 0.56.0
2. Update `core/Cargo.toml` and `cli/Cargo.toml` affected deps
3. Run `cargo update` and verify:
   - `ring` 0.16.20 eliminated from Cargo.lock
   - `rustls-webpki` 0.101.7 eliminated from Cargo.lock
   - `instant` 0.1.13 eliminated from Cargo.lock
4. Fix all compilation errors in transport layer

### Phase 3: Transport Layer Migration (rust-coder)

1. Update `core/src/transport/swarm.rs`:
   - `SwarmBuilder` API changes
   - `start_swarm()` signature changes
   - `Transport` composition changes
2. Update `core/src/transport/behaviour.rs`:
   - `libp2p-tls` 0.6.x API
   - `rcgen` 0.13+ usage
3. Update all test files that use `start_swarm()` or transport mocks
4. Update `core/examples/nat_reflection_demo.rs` if needed

### Phase 4: Verification

1. `cargo build --workspace` passes
2. `cargo test --workspace` passes (all existing tests)
3. `cargo clippy --workspace -- -D warnings` passes
4. Verify `grep 'ring 0\.16' Cargo.lock` returns empty
5. Verify `grep 'rustls-webpki 0\.101' Cargo.lock` returns empty

## Constraints

- Do NOT break WASM compilation (libp2p deps are cfg-gated)
- Do NOT break Android compilation (cargo-ndk cross-compile)
- Maintain transport priority: BLE -> WiFi -> mDNS -> QUIC/TCP relay -> Internet relay
- All unsafe blocks in transport/crypto require `// SAFETY:` comments
- Changes require `deepseek-v3.2:cloud` adversarial review per `security.md`

## File Targets

- `Cargo.toml` (workspace root) [EDIT]
- `core/Cargo.toml` [EDIT]
- `cli/Cargo.toml` [MAY EDIT]
- `core/src/transport/swarm.rs` [EDIT]
- `core/src/transport/behaviour.rs` [EDIT]
- `core/src/transport/manager.rs` [MAY EDIT]
- `core/examples/nat_reflection_demo.rs` [MAY EDIT]
- `Cargo.lock` [AUTO-GENERATED]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo fmt --all -- --check
grep 'ring 0\.16' Cargo.lock
grep 'rustls-webpki 0\.101' Cargo.lock
```

## Acceptance Gates

1. `cargo build --workspace` passes
2. `cargo test --workspace` passes
3. `Cargo.lock` contains NO `ring` version < 0.17.0
4. `Cargo.lock` contains NO `rustls-webpki` version < 0.102.0
5. `REMAINING_WORK_TRACKING.md` updated with new cargo audit state
6. `docs/` migration plan updated

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
