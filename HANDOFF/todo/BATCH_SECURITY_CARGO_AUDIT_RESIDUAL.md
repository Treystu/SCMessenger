# BATCH_SECURITY_CARGO_AUDIT_RESIDUAL

**Status:** VERIFIED REMAINING WORK
**Agent:** Rust/security implementer (requires `deepseek-v3.2:cloud` adversarial review)
**Budget:** 1800s (MIXED tier)
**Source:** REMAINING_WORK_TRACKING.md 2026-04-29 audit entry, Cargo.lock current state

---

## Verified Gap

Cargo.lock still contains vulnerable transitive dependency versions. The `quinn-proto` HIGH vulnerability is **already fixed** (0.11.14 >= 0.11.14), but `ring` and `rustls-webpki` remain.

**Current Cargo.lock State (verified 2026-05-20):**

| Crate | Version | Status | CVE/Risk |
|-------|---------|--------|----------|
| `quinn-proto` | 0.11.14 | **FIXED** | HIGH DoS — upgrade satisfied |
| `ring` | 0.16.20 | **VULNERABLE** | MEDIUM AES panic via `rcgen` / `libp2p-tls` transitive |
| `ring` | 0.17.14 | Safe | Used by newer dep chain |
| `rustls-webpki` | 0.101.7 | **VULNERABLE** | Name constraint + CRL parsing issues |
| `rustls-webpki` | 0.102.8 | Safe | |
| `rustls-webpki` | 0.103.13 | Safe | |
| `instant` | 0.1.13 | **Unmaintained** | Warning only |
| `bincode` | 1.3.3 | **Unmaintained** | Warning only |

**Transitive origin verified:**
- `ring 0.16.20` is pulled in via `rcgen 0.12.1` -> `ring 0.16.20` and `libp2p-tls` -> `ring 0.16.20`
- `rustls-webpki 0.101.7` is pulled in via `rustls 0.21.12` -> `rustls-webpki 0.101.7`

## Scope

### Part A: Determine Upgrade Path (Research)

1. Run `cargo tree -p ring -i` to confirm exact reverse dependency chain
2. Run `cargo tree -p rustls-webpki -i` to confirm exact reverse dependency chain
3. Check if `rcgen` has a newer version that uses `ring 0.17.x`
4. Check if `rustls 0.22+` or `0.23+` is available and which workspace deps need updating

### Part B: Upgrade Vulnerable Transitive Dependencies

1. Update `Cargo.toml` workspace dependencies to eliminate `ring 0.16.20` requirement:
   - If `rcgen` upgrade exists: bump `rcgen` version
   - If `libp2p-tls` can be updated: bump `libp2p` patch version
   - If direct `ring` feature flag change helps: document and apply
2. Update `rustls` or intermediate crates to eliminate `rustls-webpki 0.101.7`
3. Run `cargo update` and verify `Cargo.lock` no longer contains vulnerable versions

### Part C: Compile + Test Verification

1. `cargo build --workspace` — must pass
2. `cargo test --workspace` — must pass (all existing tests)
3. `cargo clippy --workspace -- -D warnings` — must pass
4. Verify `grep 'ring 0.16' Cargo.lock` returns empty
5. Verify `grep 'rustls-webpki 0.101' Cargo.lock` returns empty

## Constraints

- Do NOT break libp2p transport compilation (TLS, QUIC)
- Do NOT change core crypto (XChaCha20-Poly1305, X25519) — this task is dependency management only
- If upgrade path requires `libp2p` version bump, scope it carefully — libp2p upgrades can cascade
- If upgrade is infeasible without breaking changes, document the blocker and propose alternative mitigation (feature flag isolation, etc.)
- All changes require `deepseek-v3.2:cloud` adversarial review per `security.md`

## File Targets

- `Cargo.toml` (workspace root and/or core/Cargo.toml) [EDIT]
- `Cargo.lock` [AUTO-GENERATED — do not hand-edit]
- `core/Cargo.toml` [MAY EDIT if libp2p/rcgen/rustls versions need changing]
- `cli/Cargo.toml` [MAY EDIT if affected]

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
2. `cargo test --workspace` passes (0 failures)
3. `cargo clippy --workspace` passes
4. `Cargo.lock` contains NO `ring` version < 0.17.0
5. `Cargo.lock` contains NO `rustls-webpki` version < 0.102.0
6. `REMAINING_WORK_TRACKING.md` updated with new cargo audit state
7. If any vulnerability cannot be remediated, document exact blocker in task file before moving to done/

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
