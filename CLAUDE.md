# CLAUDE.md

Guidance for Claude Code sessions in this repo. This file is re-injected every
turn — it stays high-level; per-topic detail lives in `docs/CLAUDE_REFERENCE.md`
(build/test command reference, core module map, test inventory, swarm internals).
Read that file's relevant section on demand instead of re-deriving.

## Project Overview

SCMessenger is a sovereign encrypted decentralized messaging mesh. A Rust core
(`scmessenger-core`) handles identity, crypto, P2P transport, and storage;
platform clients (Android/Kotlin, iOS/Swift, WASM/browser, CLI) consume it via
UniFFI bindings or JSON-RPC.

**Active release line:** v0.3.4, working toward v1.0.0. Sequencing is governed
by `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (two-phase DAG; Phase 1 = Windows/Android
transport parity — top priority). Dispatch order: `HANDOFF/todo/_QUEUE.md`.

**Four operating modes share this repo:**
1. **Native Claude Code** (default) — single session using the subagents/skills/hooks below.
2. **Ollama-cloud swarm** — only under `/orchestrate` or `/swarm`; see `docs/CLAUDE_REFERENCE.md` section 4.
3. **`/scmorc`** — headless per-task `claude -p` workers on the Anthropic subscription (`.claude/commands/scmorc.md`). Gold standard for batch processing.
4. **`/scm`** — native cowork orchestrator using the `Agent` tool (`.claude/commands/scm.md`).

## Workspace Structure

```
core/     -> scmessenger-core: identity, crypto, transport, store, routing, relay, privacy
cli/      -> headless daemon + embedded web server (127.0.0.1:9002)
mobile/   -> UniFFI bridge crate (Android/iOS bindings)
wasm/     -> browser thin-client (JSON-RPC over WebSocket /ws)
android/  -> Kotlin/Compose app (Gradle 8.13, AGP 8.13.2, Kotlin 1.9.20, Hilt, minSdk 26)
iOS/      -> Swift app (uppercase-I path enforced by CI)
patch/    -> cargo patches (if-watch Android stub, libp2p-quic/tcp)
scripts/  -> build/test/ops scripts   docs/ -> canonical documentation
HANDOFF/  -> task backlog: todo/ (see _QUEUE.md), IN_PROGRESS/, done/
```

## Architecture Essentials

- `IronCore` is the single entry point; all state behind `Arc<RwLock<…>>` (parking_lot).
- Crypto path: Ed25519 identity, X25519 ECDH -> XChaCha20-Poly1305. `public_key_hex` is the canonical cross-platform identifier.
- Transport priority: BLE -> WiFi Aware/Direct -> mDNS/LAN -> QUIC/TCP relay -> internet relay; races with <500ms fallback.
- Message flow: `prepare_message` -> Outbox -> transport -> receipt -> `mark_message_sent`; inbound `receive_message` -> Inbox -> dedup -> notify. Relay holds custody until receipt confirmation.
- Storage: sled, only via `store/` module. Notification surfacing via `classify_notification`.
- **The mobile swarm FFI surface is async (2026-07-06):** `SwarmBridge`/`MeshService` methods are `async fn` -> Kotlin `suspend fun`; internal sync Rust callers use the `*_blocking` helpers in `mobile_bridge.rs` (never from a tokio context). `start_swarm` blocks until the first listener binds — `Ok` means listening.
- Full module map: `docs/CLAUDE_REFERENCE.md` section 2.

## Build & Verify

Prefer the **`build-verify` skill** (`full|rust|android|wasm|compile_gate`).
The gates you'll run most (full command reference: `docs/CLAUDE_REFERENCE.md` section 1):

```bash
export CARGO_INCREMENTAL=0            # ALWAYS, before any cargo command (Windows rlib safety)
cargo build --workspace               # Rust gate
cargo test --workspace --no-run       # compile gate (required before task-complete)
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown   # WASM gate
cd android && ./gradlew assembleDebug -x lint --quiet             # Android gate
cd android && ./gradlew :app:testDebugUnitTest                    # Android unit tests (re-enabled 2026-07-06)
```

## Native Claude Code Setup

Real subagents/skills/hooks are configured — use them instead of re-deriving
checks. (Distinct from the swarm's older `.claude/skills/*.json` mechanism.)

| Subagent | Use for |
|---|---|
| `rust-implementer` | Landing a scoped Rust change from a plan/task spec, self-verifying against gates |
| `crypto-security-auditor` | MANDATORY adversarial review before merging changes to `crypto/`, `transport/`, `routing/`, `privacy/` (read-only) |
| `release-gatekeeper` | Final pre-merge checklist verdict (read-only) |
| `android-qa` | Android build/test verification + mechanical compliance fixes |
| `docs-sync-auditor` | Doc-sync verification + mechanical doc fixes after a change lands |

| Skill | Does |
|---|---|
| `build-verify [scope]` | Runs the build gates via `.claude/skills/build_verify.sh` |
| `docs-sync` | Runs `scripts/docs_sync_check.sh`, fixes mechanical issues |
| `finalize-checklist` | Composite pre-completion pass: scoped build-verify + docs-sync + secret scan + canonical-doc check (does not commit) |

Hooks: SessionStart prints git status + HANDOFF counts + backlog header;
PostToolUse (Edit|Write) blocks files containing emoji (`.claude/rules/no-emojis.md`).
Tool-agnostic enforcement: a versioned pre-commit hook (`.githooks/pre-commit`
-> `scripts/rules_check.py`, activated via `core.hooksPath`) blocks emoji,
artifacts, root `.py`, lowercase `ios/`, and key material at commit time for
EVERY tool — Claude, Cowork, Gemini/agy, humans. Never `--no-verify`.
Permissions: project allowlist in `.claude/settings.json` (extend via the
`fewer-permission-prompts` skill); `.claude/settings.local.json` is personal/gitignored.

## Windows-Specific Rules

- Invoke shell scripts via Git Bash explicitly: `"C:\Program Files\Git\bin\bash.exe" <script>` — never assume `bash` is in PATH from PowerShell/CMD.
- `export CARGO_INCREMENTAL=0` before every cargo command; incremental is disabled repo-wide (`.cargo/config.toml`) to prevent rlib corruption.
- NEVER run two build-tool invocations concurrently (cargo/gradle in any combination) — Gradle targets can silently spawn cargo-ndk upstream builds. Check `tasklist //FI "IMAGENAME eq cargo.exe"` / `java.exe` first.
- CI runs ubuntu/macos only; Windows builds are verified locally.

## Mandatory Rules

### Before finalizing any run
Run the **`finalize-checklist` skill**, which covers: (1) build verification
scoped to what changed; (2) docs-sync; (3) canonical-doc updates for any
behavior/scope/risk/verification change; (4) a final summary stating which docs
were updated (or why none) and build status; (5) git checkpoint — `git add -A`
then commit locally (`native: completed [Task]`, or `swarm:` when acting as the
swarm); never push unless asked.

For anything touching `core/src/{crypto,transport,routing,privacy}/`: run the
**`crypto-security-auditor`** subagent before considering it mergeable, and
**`release-gatekeeper`** as the final pre-merge check.

### Escalation — stop and ask the operator before
Architecture-direction changes; security/privacy trade-offs; tech-stack
changes; API-contract breaks; release timing/versioning. Applies to every mode.

### File storage & paths
- Temp files ONLY in repo-local `tmp/` (`tmp/session_logs/`, `tmp/work_files/`, `tmp/audit_reports/`) — never system `/tmp`.
- `iOS/` uppercase-I everywhere (CI-enforced); XCFramework at `iOS/SCMessengerCore.xcframework/`; no `.py` in repo root; no build artifacts committed (`git ls-files "*.log" "*.pid" "*.logcat"` must be empty).
- Commit messages: issues fixed, files modified, test/build status, docs updated.

## Canonical Documentation

Source of truth for the enforced list: `HEADER_FILES` in
`scripts/docs_sync_check.sh` (~22 files) — run the `docs-sync` skill rather
than hand-checking. Priority reading order: `DOCUMENTATION.md` ->
`docs/CURRENT_STATE.md` -> `REMAINING_WORK_TRACKING.md` ->
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` -> `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
-> `docs/DOCUMENT_STATUS_INDEX.md`. Historical docs live in `docs/historical/`
and are not execution truth. `AGENTS.md` is the model-agnostic contract for
non-Claude/remote agents (Cowork sandboxes, Gemini/agy) — this file supersets
it for Claude sessions.

## Testing

Integration tests in `core/tests/` (`integration_<domain>_<scenario>`);
proptest harness in `core/src/crypto/proptest_harness.rs`; Kani proofs behind
`kani-proofs`; Android unit suite re-enabled 2026-07-06 (quarantined stragglers
in `android/app/src/test-quarantine/`). Inventory: `docs/CLAUDE_REFERENCE.md` section 3.

## Context & Prompt Engineering Notes

- Bounded tool output: prefer scoped grep/diff over whole-file reads or full-workspace diffs (`git diff --stat` first).
- Batch independent reads/checks into one turn.
- Delegate scoped work to the subagents above; keep the main thread for synthesis and decisions.
- This file is re-injected every turn: keep it dense, non-duplicative, and pointer-based — if content lives in an on-demand doc/skill/command file, link it instead of copying (a second copy just drifts).
