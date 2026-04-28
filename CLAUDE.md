# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SCMessenger is a sovereign encrypted decentralized messaging mesh. A Rust core (`scmessenger-core`) handles identity, crypto, P2P transport, and storage. Platform clients (Android/Kotlin, iOS/Swift, WASM/browser, CLI) consume the core via UniFFI bindings or JSON-RPC.

**Active release line:** v0.2.1 alpha (v0.2.0 was the baseline).

## Workspace Structure

```
core/          → scmessenger-core (lib + cdylib): identity, crypto, transport, store, relay
cli/           → scmessenger-cli (bin): headless daemon + embedded web server (127.0.0.1:9002)
mobile/        → scmessenger-mobile (cdylib/staticlib): UniFFI bridge crate for Android/iOS
wasm/          → scmessenger-wasm (cdylib/rlib): WASM bindings for browser thin-client
android/       → Kotlin/Compose app (Gradle, minSdk 26, compileSdk 35)
iOS/           → Swift app (Xcode workspace, uppercase-I path convention enforced by CI)
patch/         → Cargo patches: if-watch (Android stub), if-watch-full (Android compat), libp2p-quic, libp2p-tcp
scripts/       → Build/test/ops shell scripts and Python utilities
docs/          → Canonical documentation (see DOCUMENTATION.md index)
HANDOFF/       → Task tracking: todo/, IN_PROGRESS/, done/ (agent orchestration backlog)
```

## Key Architecture

### Core (`scmessenger-core`)

The central crate exports `IronCore` — the main entry point. It holds identity, outbox, inbox, contact manager, history manager, storage manager, log manager, blocked manager, relay registry, and audit log. All state is behind `Arc<RwLock<…>>` (parking_lot).

**Module map:**

- `identity/` — Ed25519 key management, identity creation/restore/backup, seniority tracking
- `crypto/` — X25519 ECDH + XChaCha20-Poly1305 encryption, ratcheting, backup, Kani proofs
- `transport/` — libp2p Swarm-based multi-transport: TCP, QUIC, mDNS, BLE, internet relay
- `drift/` — Protocol-level message framing, compression (lz4), relay custody, sync
- `store/` — sled-backed persistence: contacts, inbox, outbox, history, blocked, logs, relay custody
- `routing/` — Adaptive routing engine: TTL budgets, multipath, reputation, negative cache, smart retry
- `relay/` — Bootstrap nodes, relay client/server, delegate prewarm, FindMy, peer exchange, invite protocol
- `privacy/` — Onion routing, cover traffic, padding, timing obfuscation
- `abuse/` — Spam detection, reputation, auto-block
- `notification/` — Notification classification and delivery policy
- `wasm_support/` — JSON-RPC bridge (`rpc.rs`) between browser WASM client and CLI daemon
- `mobile_bridge/`, `contacts_bridge/`, `blocked_bridge/` — UniFFI scaffolding for mobile targets

**Platform-specific compilation (core Cargo.toml):**

- `cfg(target_arch = "wasm32")` — WASM: uses rexie (IndexedDB), wasm-bindgen-futures, getrandom/js
- `cfg(all(not(wasm32), not(android)))` — Desktop: full tokio, libp2p TCP+QUIC+mDNS+DNS, quinn, ureq, tungstenite
- `cfg(all(not(wasm32), android))` — Android: full tokio, libp2p TCP+QUIC (no mDNS, no DNS), libc, tungstenite

**Features:** `gen-bindings` (UniFFI codegen), `wasm`, `kani-proofs`, `phase2_apis`, `test-utils`

### CLI (`scmessenger-cli`)

Daemon binary that runs the mesh node and serves a local web UI. Modules: `api` (CLI commands), `server` (warp HTTP + WebSocket on 127.0.0.1:9002), `transport_bridge`, `transport_api`, `ble_daemon`, `ble_mesh`, `config`, `ledger`, `bootstrap`, `contacts`, `history`.

### WASM (`scmessenger-wasm`)

Browser thin-client connecting to the CLI daemon via WebSocket `/ws` (JSON-RPC). Modules: `mesh`, `daemon_bridge`, `connection_state`, `transport`, `notification_manager`, `storage`, `worker`.

### Mobile (`scmessenger-mobile`)

Thin UniFFI bridge crate — compiles to `cdylib` + `staticlib`. Generates Kotlin bindings (`gen_kotlin` bin) and Swift bindings (`gen_swift` bin) when `gen-bindings` feature is enabled.

### Android App

Kotlin + Jetpack Compose. UniFFI-generated `uniffi.api` package. Architecture: `MeshRepository` → ViewModels → Compose UI. BLE/WiFi transport managers, foreground service, notification channels. Built with Gradle 8.13, AGP 8.13.2, Kotlin 1.9.20, Hilt DI.

## Build Commands (Windows)

### Rust Workspace

```bash
# Full workspace build
cargo build --workspace

# Check only (faster iteration)
cargo check --workspace

# Format check
cargo fmt --all -- --check

# Lint
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments

# Run all workspace tests
cargo test --workspace

# Run specific crate tests
cargo test -p scmessenger-core
cargo test -p scmessenger-cli

# Run a single integration test
cargo test -p scmessenger-core --test integration_e2e

# Compile gate (builds tests without running)
cargo test --workspace --no-run

# WASM target build (requires wasm32-unknown-unknown)
cargo build -p scmessenger-wasm --target wasm32-unknown-unknown

# WASM check only
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown

# Generate UniFFI Kotlin bindings
cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin

# Generate UniFFI Swift bindings
cargo run -p scmessenger-core --features gen-bindings --bin gen_swift
```

### Android

```bash
# From repo root on Windows (Git Bash or CMD with ANDROID_HOME set)
cd android
./gradlew assembleDebug -x lint --quiet

# Fresh device install (uninstalls old, installs new, grants perms)
./android/install-clean.sh

# Verify build environment
./android/verify-build-setup.sh

# Role/fallback parity unit tests
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"
```

**Android Rust cross-compilation** requires `cargo-ndk` and targets: `aarch64-linux-android`, `x86_64-linux-android` (plus `armv7-linux-androideabi`, `i686-linux-android` for full coverage).

### WASM Pack

```bash
cd wasm
wasm-pack build --target web
wasm-pack test --headless --firefox
```

### Docs Sync Check

```bash
# Git Bash / Unix
./scripts/docs_sync_check.sh

# PowerShell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1
```

## Windows-Specific Notes

- Incremental compilation is disabled (`.cargo/config.toml`: `incremental = false`) to prevent rlib metadata and paging file errors during integration test builds.
- Shell scripts in `scripts/` require Git Bash or WSL. PowerShell equivalents exist for key scripts (`.ps1`).
- The `if-watch` crate is patched to an Android-compatible stub (`patch/if-watch-full/`) because the native version requires system APIs not available on Android.
- CI runs on ubuntu-latest and macos-latest only. Windows builds are local-only; verify with `cargo build --workspace` and `cargo test --workspace`.

## Mandatory Rules

### Before Finalizing Any Run

1. **Build verification**: If you edited Rust code, run `cargo build --workspace`. If you edited Android code, run `./gradlew assembleDebug`. Record build status in commit messages.
2. **Doc sync**: Run `scripts/docs_sync_check.sh` (or `.ps1`). Resolve failures before finalizing.
3. **Canonical doc updates**: If a run changes behavior, scope, risk posture, scripts, tests, or verification workflow, update the canonical docs in the same run.
4. **Final summary rule**: State which docs were updated, or why no doc updates were needed, and report build verification status for edited targets.

### File Storage

- **NEVER** use system `/tmp`, `/var/tmp`, or `/dev/shm` for temp files.
- **ALWAYS** use repo-local `tmp/` directory: `tmp/session_logs/`, `tmp/work_files/`, `tmp/audit_reports/`.
- All temp files must be gitignored (already configured in `.gitignore`).

### Path Conventions

- Use `iOS/` (uppercase-I) for all path references. CI enforces this — lowercase `ios/` will fail the path-governance check.
- XCFramework canonical location: `iOS/SCMessengerCore.xcframework/`. Never place in repo root.
- No `.py` files in repo root; use `scripts/`.
- No build artifacts committed; verify with `git ls-files "*.log" "*.pid" "*.logcat"`.

### Git

- `git add -A` before commits.
- Commit messages must include: issues fixed, files modified, test/build status, canonical docs updated.

## Canonical Documentation (Priority Order)

1. `AGENTS.md` — Agent coordination and rules
2. `DOCUMENTATION.md` — Project documentation hub
3. `docs/DOCUMENT_STATUS_INDEX.md` — Doc lifecycle tracking
4. `docs/CURRENT_STATE.md` — Current architecture and verified state
5. `REMAINING_WORK_TRACKING.md` — Active backlog
6. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` — Milestone plan
7. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` — Risk tracking

Do not treat mixed or historical docs as execution truth unless canonical docs explicitly point to them. Historical docs live in `docs/historical/`.

## Agent Swarm Integration

The ollama-based agent swarm is configured in `ORCHESTRATOR_DIRECTIVE.md` (the former CLAUDE.md content). Models are routed through ngrok to cloud providers; append `:cloud` to any model name from the roster below. The pool is defined in `.claude/agent_pool.json` with a 2-slot concurrency limit. Use `bash .claude/orchestrator_manager.sh` for lifecycle management. On Windows, use `'C:\Program Files\Git\bin\bash.exe'` if not inside Claude's bash emulator. **Never** use `Stop-Process`, `taskkill`, or `kill` directly — always go through the manager script.

### Available Cloud Models (`:cloud` suffix)

**Flagship / Reasoning:**
- `glm-5.1` — Latest GLM, 1.5T params, strongest general reasoning
- `deepseek-v4-pro` — DeepSeek V4 Pro, 1.6T params, deep analysis
- `deepseek-v4-flash` — DeepSeek V4 Flash, 140B params, fast reasoning
- `kimi-k2.6` — Kimi K2.6, 595B params, latest Kimi
- `kimi-k2-thinking` — Kimi K2 with extended thinking, 1.1T params
- `qwen3-coder:480b` — Specialized coding model, 510B params
- `qwen3-coder-next` — Next-gen coding model, 81B params
- `qwen3.5:397b` — General reasoning, strong fallback
- `mistral-large-3:675b` — Large Mistral, pipeline coordination

**Mid-Tier / Specialized:**
- `cogito-2.1:671b` — Deliberative reasoning, 688B params
- `deepseek-v3.2` — Crypto/math/protocol validation
- `deepseek-v3.1:671b` — Previous DeepSeek flagship
- `gemma4:31b` — Lightweight worker, tests/bindings/docs
- `gemini-3-flash-preview` — Quick triage, lint, CI
- `glm-5` — GLM 5 base (predecessor to 5.1)
- `glm-4.7`, `glm-4.6` — Earlier GLM generations
- `minimax-m2.7`, `minimax-m2.5`, `minimax-m2.1`, `minimax-m2` — MiniMax generations
- `nemotron-3-super` — 230B, strong validation
- `devstral-2:123b`, `devstral-small-2:24b` — Developer tooling models
- `qwen3-vl:235b` — Vision-language (if needed)

**Small / Local-Fallback:**
- `gemma3:27b`, `gemma3:12b`, `gemma3:4b` — Lightweight Gemma variants
- `ministral-3:14b`, `ministral-3:8b`, `ministral-3:3b` — Minimal Mistral
- `rnj-1:8b` — 8B lightweight
- `gpt-oss:120b`, `gpt-oss:20b` — Open-source GPT variants

## Key Cross-Cutting Patterns

- **Identity**: Ed25519 signing keys, X25519 for message encryption. `public_key_hex` is the canonical cross-platform identifier. `identity_id` and `libp2p_peer_id` are derived/operational.
- **Crypto path**: X25519 ECDH → shared secret → XChaCha20-Poly1305 authenticated encryption.
- **Storage**: sled (embedded key-value). Core data lives in `Store` behind `Arc<RwLock<…>>`.
- **Transport priority**: BLE → WiFi Aware/Direct → mDNS/LAN → QUIC/TCP relay → Internet relay. Transport races with <500ms fallback.
- **Message flow**: `prepare_message` → `Outbox` → transport send → receipt → `mark_message_sent`. Inbound: `receive_message` → `Inbox` → dedup → notify.
- **UniFFI**: Core exposes `api.udl` + proc macros. Mobile bindings generated via `gen_kotlin`/`gen_swift` bins. WASM uses `wasm-unstable-single-threaded` feature.
- **Relay custody**: Messages held by relay until receipt confirmation. Relay registry persisted in sled.
- **Notification classification**: `classify_notification` determines if/when/how to surface a message based on platform, privacy config, and app state.

## Testing

Integration tests are in `core/tests/`:

- `integration_e2e` — End-to-end message flow
- `integration_contact_block` — Contact/block lifecycle
- `integration_offline_partition_matrix` — Offline/partition recovery
- `integration_ironcore_roundtrip` — IronCore encrypt/decrypt roundtrip
- `integration_registration_protocol` — Identity registration
- `integration_nat_reflection` — NAT traversal
- `integration_relay_custody` — Relay message custody
- `integration_retry_lifecycle` — Retry and delivery lifecycle
- `integration_receipt_convergence` — Receipt convergence
- `integration_all_phases` — Multi-phase scenario
- `test_address_observation`, `test_multiport`, `test_persistence_restart`, `test_mesh_routing`

Property-based testing: `proptest` harness in `core/src/crypto/proptest_harness.rs`. Formal verification: `kani` proofs behind `kani-proofs` feature.