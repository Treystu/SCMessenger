# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SCMessenger is a sovereign encrypted decentralized messaging mesh. A Rust core (`scmessenger-core`) handles identity, crypto, P2P transport, and storage. Platform clients (Android/Kotlin, iOS/Swift, WASM/browser, CLI) consume the core via UniFFI bindings or JSON-RPC.

**Active release line:** v0.3.4 (working toward v1.0.0; confirmed against `Cargo.toml` and the installed Android build — 2026-07-03).

**Two operating modes share this repo:**
1. **Native Claude Code** (default) — a single Claude Code session (you, most of the time) working directly, using the subagents/skills/hooks in [Native Claude Code Setup](#native-claude-code-setup) below.
2. **The ollama-cloud swarm** — a separate multi-agent orchestration system invoked via `/orchestrate` or `/swarm`, documented in [Agent Swarm Integration](#agent-swarm-integration-ollama-cloud-orchestrate--swarm-only). Skip that section entirely unless one of those commands is active.

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
HANDOFF/       → Task tracking: todo/, IN_PROGRESS/, done/ (agent orchestration backlog — native sessions can pull from here directly too, see below)
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

Prefer the **`build-verify` skill** (`full|rust|android|wasm|compile_gate`) over running these one at a time — see [Native Claude Code Setup](#native-claude-code-setup).

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

Or just invoke the **`docs-sync` skill**.

## Native Claude Code Setup

This repo has real Claude Code subagents, Skills, and hooks configured — use them instead of re-deriving the same checks by hand every time. (These are distinct from the ollama swarm's own `.claude/skills/*.json` + `.sh` files, which are a separate, older mechanism used only by `.claude/orchestrator_manager.sh`'s `model_dispatch.sh` — don't confuse the two.)

### Subagents (`.claude/agents/*.md`)

| Subagent | Tools | Use for |
|---|---|---|
| `rust-implementer` | full (Read/Edit/Write/Bash/Grep/Glob) | Landing a well-scoped Rust change from a plan/task spec and self-verifying against the build gates |
| `crypto-security-auditor` | read-only + Bash | Adversarial review before merging any change to `crypto/`, `transport/`, `routing/`, `privacy/` — this is the repo's mandatory Adversarial Review Protocol |
| `release-gatekeeper` | read-only + Bash | Final pre-merge checklist (compilation, correctness, tests, security, docs) — read-only, verdict only |
| `android-qa` | Read/Edit/Grep/Glob/Bash | Android build/test verification, plus mechanical compliance fixes (hardcoded strings, manifest entries) |
| `docs-sync-auditor` | Read/Edit/Grep/Glob/Bash | Doc sync verification after a change lands, plus mechanical doc fixes |

`.claude/agents/` is gitignored for per-run subdirectories only (`*/`) — the swarm uses that same directory as a runtime workspace (`AGENT_ROOT` in `orchestrator_manager.sh`), but the top-level `*.md` subagent definitions are tracked normally and won't collide with it.

### Skills (`.claude/skills/<name>/SKILL.md`)

| Skill | Does |
|---|---|
| `build-verify [full\|rust\|android\|wasm\|compile_gate]` | Runs the existing `.claude/skills/build_verify.sh` gates |
| `docs-sync` | Runs `scripts/docs_sync_check.sh`, fixes mechanical header/link issues directly |
| `finalize-checklist` | Composite: scopes the change, runs `build-verify` + `docs-sync`, scans staged changes for secrets, checks a canonical doc was updated. Does **not** commit. |

### Hooks (`.claude/settings.json`)

- **SessionStart** → `.claude/hooks/session_orientation.sh`: prints `git status --short`, HANDOFF todo/IN_PROGRESS counts, and the `REMAINING_WORK_TRACKING.md` header, so a new session doesn't have to re-derive backlog state manually.
- **PostToolUse** (Edit|Write) → `.claude/hooks/check_no_emoji.py`: blocks (exit 2) if an edited file contains emoji characters, enforcing `.claude/rules/no-emojis.md` deterministically instead of relying on the model remembering it every turn. Fails open on any script error — it only ever blocks on an actual positive match.

### Permissions

`.claude/settings.json` (checked in, project-wide) holds a `permissions.allow` list built from actual usage via the `fewer-permission-prompts` skill. Re-run that skill periodically as usage patterns grow — it's additive and safe. `.claude/settings.local.json` is gitignored, personal, per-machine overrides — it is not shared via git.

## Windows-Specific Notes

- **Git Bash path:** On Windows, all shell scripts MUST be invoked with the full path to Git Bash: `"C:\Program Files\Git\bin\bash.exe" <script>` or `bash` if already inside the Git Bash shell. Do NOT rely on `bash` being in PATH from PowerShell/CMD.
- Incremental compilation is disabled (`.cargo/config.toml`: `incremental = false`) to prevent rlib metadata and paging file errors during integration test builds.
- **Runtime rule:** Before running `cargo check` or `cargo build`, you MUST set `export CARGO_INCREMENTAL=0` in your terminal. This prevents `.rlib` file-lock corruption during concurrent Rust builds on Windows.
- Shell scripts in `scripts/` require Git Bash or WSL. PowerShell equivalents exist for key scripts (`.ps1`).
- The `if-watch` crate is patched to an Android-compatible stub (`patch/if-watch-full/`) because the native version requires system APIs not available on Android.
- CI runs on ubuntu-latest and macos-latest only. Windows builds are local-only; verify with `cargo build --workspace` and `cargo test --workspace`.

## Mandatory Rules

### Before Finalizing Any Run

Run the **`finalize-checklist` skill** — it covers all of the following in one pass:

1. **Build verification**: scoped to what changed. If you edited Rust code, that means `cargo build --workspace` (or the `build-verify rust` skill scope). If you edited Android code, `./gradlew assembleDebug` (`build-verify android`).
2. **Doc sync**: the `docs-sync` skill / `scripts/docs_sync_check.sh` (or `.ps1`). Resolve failures before finalizing.
3. **Canonical doc updates**: if a run changes behavior, scope, risk posture, scripts, tests, or verification workflow, update the canonical docs in the same run — see [Canonical Documentation](#canonical-documentation).
4. **Final summary rule**: state which docs were updated, or why no doc updates were needed, and report build verification status for edited targets.
5. **Git checkpoint**: after completing work, run `git add -A` followed by `git commit -m "swarm: completed [Task Name]"` if acting as the swarm, or a plain descriptive message if this was a native session. Do not push to remote; commit locally to prevent data loss.

For anything touching `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, or `core/src/privacy/`, also run the **`crypto-security-auditor`** subagent before considering it mergeable. Use **`release-gatekeeper`** as the final pre-merge check.

### Escalation

Stop and ask the human operator before proceeding on any of the following — this applies whether you're a native session, a delegated subagent, or the ollama swarm:

- Architectural direction changes that alter the project's core design philosophy
- Security/privacy trade-off decisions
- Technology stack migrations or additions
- API contract breaking changes
- Release timing and versioning strategy

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

## Canonical Documentation

The authoritative, enforced list of canonical docs is the `HEADER_FILES` array in `scripts/docs_sync_check.sh` — treat that script as the single source of truth, not a hand-copied list here. (This section previously listed a 7-item priority order that had drifted from the script, including `AGENTS.md` — **that file does not exist in this repo**; don't cite it.)

Priority reading order for context (a subset of the full enforced list — see the script for all ~22 files):

1. `DOCUMENTATION.md` — project documentation hub
2. `docs/CURRENT_STATE.md` — current architecture and verified state
3. `REMAINING_WORK_TRACKING.md` — active backlog
4. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` — milestone plan
5. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` — risk tracking
6. `docs/DOCUMENT_STATUS_INDEX.md` — doc lifecycle tracking

Run the `docs-sync` skill (or `docs-sync-auditor` subagent) rather than checking headers by hand — it stays correct as the script evolves; this list doesn't automatically.

Do not treat mixed or historical docs as execution truth unless canonical docs explicitly point to them. Historical docs live in `docs/historical/`.

## Agent Swarm Integration (ollama cloud, `/orchestrate` + `/swarm` only)

**Skip this entire section for a normal native session.** It only applies when you are explicitly acting as the swarm orchestrator or a swarm worker.

- **Quota governor, agent routing table, cloud model roster**: all defined once in `.claude/commands/orchestrate.md`, loaded automatically when `/orchestrate` runs. Native Claude Code sessions run on Anthropic's API and are not subject to the ollama-cloud rolling quota windows described there. Do not re-copy those tables here — if they drift, fix them in `orchestrate.md`.
- **Identity**: determine orchestrator vs. worker from your initiating prompt.
  - **Orchestrator**: reads `HANDOFF/todo/` and `HANDOFF/IN_PROGRESS/`, writes task batches, launches workers via `.claude/orchestrator_manager.sh pool launch <agent_name> <task_file>`.
  - **Worker**: reads one `BATCH_...md` file, implements it, runs compile gates, moves the file to `HANDOFF/done/`.
- **HANDOFF state machine (CRITICAL)**: claim → execute + compile gates → move the task file from `todo/`/`IN_PROGRESS/` to `HANDOFF/done/` → `git add -A && git commit -m "swarm: completed [Task Name]"`. A task is not complete until the file has moved.
  - If a **native** session (not the swarm) completes a HANDOFF task directly, follow the same todo → done move so the backlog stays accurate, but commit as `native: completed [Task Name]` rather than `swarm: ...`, so provenance stays honest.
- **Orchestrator fire-and-forget**: once workers fill the 2 available slots, commit pending changes, arm log monitors, and exit immediately — do not `sleep` or wait. The `/loop 30m` cron (or an armed `Monitor` on agent logs) wakes the orchestrator on completion or timeout.
- **Windows process management**: standard `kill` won't clear a Windows process tree — use the dual-kill method: `kill -9 <PID>; taskkill //F //T //PID <PID>`. For swarm-managed agent processes specifically, go through `.claude/orchestrator_manager.sh pool stop <agent_id>` rather than killing directly, so pool bookkeeping stays correct — this restriction doesn't apply to unrelated processes you spawn in a native session (e.g. a hung local build).
- Swarm operator duties are also written up in `ORCHESTRATOR_DIRECTIVE.md`, but that file currently has stale, machine-specific absolute paths (`C:\Users\kanal\...`, a different contributor's environment) and an agent roster that doesn't match `.claude/agent_pool.json`. If the two disagree, `.claude/agent_pool.json` and `.claude/commands/orchestrate.md` are the source of truth — `ORCHESTRATOR_DIRECTIVE.md` needs a cleanup pass at some point, but that's swarm-internal housekeeping, not something to fix incidentally.

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

## Key Cross-Cutting Patterns

- **Identity**: Ed25519 signing keys, X25519 for message encryption. `public_key_hex` is the canonical cross-platform identifier. `identity_id` and `libp2p_peer_id` are derived/operational.
- **Crypto path**: X25519 ECDH → shared secret → XChaCha20-Poly1305 authenticated encryption.
- **Storage**: sled (embedded key-value). Core data lives in `Store` behind `Arc<RwLock<…>>`.
- **Transport priority**: BLE → WiFi Aware/Direct → mDNS/LAN → QUIC/TCP relay → Internet relay. Transport races with <500ms fallback.
- **Message flow**: `prepare_message` → `Outbox` → transport send → receipt → `mark_message_sent`. Inbound: `receive_message` → `Inbox` → dedup → notify.
- **UniFFI**: Core exposes `api.udl` + proc macros. Mobile bindings generated via `gen_kotlin`/`gen_swift` bins. WASM uses `wasm-unstable-single-threaded` feature.
- **Relay custody**: Messages held by relay until receipt confirmation. Relay registry persisted in sled.
- **Notification classification**: `classify_notification` determines if/when/how to surface a message based on platform, privacy config, and app state.

## Context & Prompt Engineering Notes

CLAUDE.md is re-injected into context on **every** turn — keep it dense and non-duplicative. If content already lives somewhere loaded on-demand (a skill, `.claude/commands/orchestrate.md`, `ORCHESTRATOR_DIRECTIVE.md`), point to it here instead of copying it — a second copy just drifts.

- **Bounded tool output**: prefer `grep`/`rg` with context flags, `head`/`tail`, or a scoped `git diff <file>` over reading whole files or running a raw `git diff` across the whole workspace. Use `git diff --stat`/`--name-only` for an overview.
- **Parallel independent operations**: batch independent reads/checks into one turn (e.g. reading two unrelated files, or running a build gate alongside a doc-sync check) instead of serializing them.
- **Delegate, don't duplicate**: use the subagents in [Native Claude Code Setup](#native-claude-code-setup) for scoped, reviewable work instead of doing everything in the main thread — it keeps the main session's context focused on synthesis and decisions rather than raw output.
