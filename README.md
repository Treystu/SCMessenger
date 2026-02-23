# SCMessenger

[![CI](https://github.com/Treystu/SCMessenger/workflows/CI/badge.svg)](https://github.com/Treystu/SCMessenger/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Decentralized, end-to-end encrypted messaging built on Rust + libp2p with mobile and WASM targets.

## Start Here

- Documentation hub: `DOCUMENTATION.md`
- Full repo context map: `docs/REPO_CONTEXT.md`
- Verified current state: `docs/CURRENT_STATE.md`
- Active gap backlog: `REMAINING_WORK_TRACKING.md`
- Global rollout plan: `docs/GLOBAL_ROLLOUT_PLAN.md`
- Unified global app plan: `docs/UNIFIED_GLOBAL_APP_PLAN.md`
- Triple-check verification report: `docs/TRIPLE_CHECK_REPORT.md`
- Full file-level documentation tracker: `docs/DOC_PASS_TRACKER.md`

## Quick Start (Development)

```bash
# Build all workspace crates
cargo build --workspace

# Run all workspace tests
cargo test --workspace

# Show CLI help
cargo run -p scmessenger-cli -- --help
```

If you prefer a short local command name:

```bash
alias scm='scmessenger-cli'
```

The repository also includes a helper script:

```bash
./scripts/scm.sh status
```

## Workspace Layout

```text
core/      scmessenger-core    Core crypto, identity, storage, transport, relay, privacy
cli/       scmessenger-cli     Interactive CLI + local control API + web dashboard server
mobile/    scmessenger-mobile  UniFFI mobile bindings
wasm/      scmessenger-wasm    WASM bindings for browser-facing clients
android/   Android app          Kotlin/Compose client integrating UniFFI surface
iOS/       iOS app              SwiftUI client integrating UniFFI surface
docs/      Documentation        Architecture, protocol, testing, platform notes
```

## Cryptography

| Operation | Algorithm |
| --- | --- |
| Identity | Ed25519 |
| Identity hash | Blake3 |
| Key exchange | X25519 ECDH |
| Key derivation | Blake3 `derive_key` |
| Encryption | XChaCha20-Poly1305 |
| Envelope auth | Ed25519 signatures + AAD binding |

## Verified Test Snapshot (2026-02-23)

From `cargo test --workspace`:

- CLI: 17 passed
- Core unit: 227 passed, 7 ignored
- Core integration: 52 passed
- Mobile crate: 4 passed
- WASM crate (native test mode): 24 passed
- Total: **324 passed, 0 failed, 7 ignored**

See `docs/TESTING_GUIDE.md` for exact commands and constraints.

## License

MIT
