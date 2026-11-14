# Swarm Implementation Roadmap: Autonomous Completion of SCMessenger

**Purpose:** TODO-driven roadmap for fully autonomous swarm completion of the SCMessenger application.
**Escalation policy:** Only philosophical-level decisions are escalated to the human operator. All implementation, testing, documentation, and optimization work is handled autonomously by the swarm.
**Last updated:** 2026-04-28

## Phase 0: Infrastructure Hardening (COMPLETED)

These tasks establish the foundation that enables autonomous operation.

- [x] Update `agent_pool.json` with all 39 ollama cloud models
- [x] Update `model_capability_mapping.json` with full routing table
- [x] Add `WebFetch(url:https://ollama.com/api/tags)` to `settings.local.json` allowlist
- [x] Add `WebSearch(ollama model availability)` to `settings.local.json` allowlist
- [x] Fix `nematron-3-super` → `nemotron-3-super` typo in `agent_pool.json`
- [x] Update CLAUDE.md model roster with all 39 models (including kimi-k2:1t, kimi-k2.5, nemotron-3-nano:30b, qwen3-vl:235b-instruct, qwen3-next:80b)
- [x] Create `.claude/rules/security.md` — adversarial review protocol, compaction poisoning defense, supply chain rules
- [x] Create `.claude/rules/build.md` — build verification, compile gate, docs sync, model availability check
- [x] Create `.claude/rules/rust.md` — core crate rules, platform gates, code quality, testing, UniFFI
- [x] Create `.claude/rules/android.md` — build env, architecture, cross-compilation, pre-merge checklist
- [x] Update CLAUDE.md with Harness Engineering Best Practices section (prompt architecture, parallel execution, context window management, multi-model routing, escalation policy)
- [x] Create `docs/CLAUDE_CODE_ARCHITECTURE_RESEARCH.md` — full research distillation
- [x] Create `docs/MULTI_MODEL_ORCHESTRATION_STRATEGY.md` — tier routing, fallback strategy, concurrency management
- [x] Create `.claude/prompts/architect.md` — system architect prompt template
- [x] Create `.claude/prompts/adversarial-reviewer.md` — security auditor prompt template
- [x] Create `.claude/prompts/implementer.md` — primary implementer prompt template
- [x] Create `.claude/prompts/gatekeeper.md` — pre-merge gatekeeper prompt template
- [x] Create `.claude/prompts/coordinator.md` — swarm coordinator prompt template
- [x] Create `.claude/skills/adversarial_review.json` + `.sh` — adversarial review skill
- [x] Create `.claude/skills/build_verify.json` + `.sh` — build verification skill
- [x] Create `.claude/skills/model_check.json` + `.sh` — model availability verification skill

## Phase 1: Core Compilation & Wiring (AUTONOMOUS)

These tasks bring the core crate to a compilable, testable state. All handled by `rust-coder` (glm-5.1:cloud) with `precision-validator` (deepseek-v3.2:cloud) review.

### 1A: Compilation Baseline
- [ ] Verify `cargo check --workspace` passes with zero errors
- [ ] Resolve all remaining compile errors from the AccioWork merge
- [ ] Fix `mobile_bridge.rs` conflicts (15 conflicts reported in prior session)
- [ ] Verify `cargo test --workspace --no-run` compiles all tests
- [ ] Run `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`
- [ ] Run `cargo fmt --all -- --check`

### 1B: Core Module Wiring
- [ ] Wire `transport/` — ensure all libp2p Swarm handlers connect to `IronCore`
- [ ] Wire `drift/` — protocol framing, compression (lz4), relay custody, sync
- [ ] Wire `routing/` — adaptive routing with TTL budgets, multipath, reputation
- [ ] Wire `relay/` — bootstrap nodes, client/server, delegate prewarm, peer exchange
- [ ] Wire `notification/` — classification and delivery policy
- [ ] Wire `abuse/` — spam detection, reputation, auto-block
- [ ] Wire `privacy/` — onion routing, cover traffic, padding, timing obfuscation

### 1C: Integration Test Pass
- [ ] `integration_e2e` — end-to-end message flow
- [ ] `integration_contact_block` — contact/block lifecycle
- [ ] `integration_offline_partition_matrix` — offline/partition recovery
- [ ] `integration_ironcore_roundtrip` — encrypt/decrypt roundtrip
- [ ] `integration_registration_protocol` — identity registration
- [ ] `integration_nat_reflection` — NAT traversal
- [ ] `integration_relay_custody` — relay message custody
- [ ] `integration_retry_lifecycle` — retry and delivery lifecycle
- [ ] `integration_receipt_convergence` — receipt convergence
- [ ] `integration_all_phases` — multi-phase scenario

**Gate:** Phase 1 is complete when `cargo test --workspace` passes all integration tests and `cargo clippy` reports zero warnings.

## Phase 2: Platform Clients (AUTONOMOUS)

These tasks build out the platform clients that consume the core. Handled by `implementer` (qwen3-coder-next:cloud) with `worker` (gemma4:31b:cloud) for bindings/docs.

### 2A: CLI Daemon
- [ ] Verify `scmessenger-cli` builds and serves on 127.0.0.1:9002
- [ ] Wire HTTP + WebSocket server (`server/` module)
- [ ] Wire `transport_bridge` and `transport_api`
- [ ] Wire `ble_daemon` and `ble_mesh`
- [ ] Wire `config`, `ledger`, `bootstrap`, `contacts`, `history`

### 2B: WASM Thin Client
- [ ] Verify `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`
- [ ] Wire `mesh`, `daemon_bridge`, `connection_state`, `transport`
- [ ] Wire `notification_manager`, `storage`, `worker`
- [ ] Test `wasm-pack build --target web`

### 2C: Mobile Bridge
- [ ] Generate UniFFI Kotlin bindings: `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
- [ ] Generate UniFFI Swift bindings: `cargo run -p scmessenger-core --features gen-bindings --bin gen_swift`
- [ ] Verify `scmessenger-mobile` compiles for `aarch64-linux-android` and `x86_64-linux-android`
- [ ] Verify `scmessenger-mobile` compiles for iOS targets

### 2D: Android App
- [ ] `./gradlew assembleDebug -x lint --quiet` passes
- [ ] Wire `MeshRepository` → ViewModels → Compose UI
- [ ] Wire BLE/WiFi transport managers
- [ ] Wire foreground service, notification channels
- [ ] Pass `RoleNavigationPolicyTest`

**Gate:** Phase 2 is complete when CLI daemon serves successfully, WASM builds, and Android debug APK compiles.

## Phase 3: Security Hardening (AUTONOMOUS — ADVERSARIAL REVIEW)

All security-critical modules undergo adversarial review by `precision-validator` (deepseek-v3.2:cloud) or `deep-analyst` (deepseek-v4-pro:cloud).

### 3A: Crypto Module Review
- [ ] Adversarial review of `core/src/crypto/` — all files
- [ ] Verify X25519 ECDH constant-time operations
- [ ] Verify XChaCha20-Poly1305 authenticated encryption
- [ ] Verify key lifecycle management
- [ ] Verify ratcheting implementation
- [ ] Run Kani proofs (`kani-proofs` feature)

### 3B: Transport & Routing Review
- [ ] Adversarial review of `core/src/transport/` — BLE, relay, QUIC
- [ ] Adversarial review of `core/src/routing/` — TTL budgets, multipath, reputation
- [ ] Verify transport race conditions
- [ ] Verify negative cache correctness

### 3C: Privacy & Abuse Review
- [ ] Adversarial review of `core/src/privacy/` — onion routing, cover traffic
- [ ] Adversarial review of `core/src/abuse/` — spam detection, auto-block
- [ ] Verify timing obfuscation

### 3D: Supply Chain Audit
- [ ] Audit `Cargo.lock` for unexpected additions
- [ ] Verify no secrets/keys in committed files (`git diff --cached`)
- [ ] Run `scripts/docs_sync_check.sh`

**Gate:** Phase 3 is complete when all adversarial reviews produce zero CRITICAL or HIGH findings, and Kani proofs pass.

## Phase 4: Documentation & Polish (AUTONOMOUS)

Handled by `worker` (gemma4:31b:cloud) with `triage-router` (gemini-3-flash-preview:cloud) for lint.

### 4A: Documentation Sync
- [ ] Run `scripts/docs_sync_check.sh` — resolve all failures
- [ ] Update `DOCUMENTATION.md` index
- [ ] Update `docs/DOCUMENT_STATUS_INDEX.md` lifecycle tracking
- [ ] Update `docs/CURRENT_STATE.md` with verified architecture
- [ ] Update `REMAINING_WORK_TRACKING.md` backlog

### 4B: Code Quality
- [ ] Remove all `eslint-disable` equivalent: fix all `clippy` warnings
- [ ] Remove all `_DEPRECATED` flagged functions or update callers
- [ ] Verify no `unwrap()` in production paths
- [ ] Verify all `// SAFETY:` comments on unsafe blocks
- [ ] Final `cargo fmt --all -- --check`

**Gate:** Phase 4 is complete when docs sync check passes and clippy reports zero warnings.

## Phase 5: Pre-Release Verification (GATEKEEPER REVIEW)

Final gate by `gatekeeper-reviewer` (kimi-k2-thinking:cloud). This is the only phase that requires explicit human sign-off before release.

### 5A: Build Verification
- [ ] `cargo build --workspace` — release mode
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `./gradlew assembleDebug` — Android
- [ ] `wasm-pack build --target web` — WASM

### 5B: Integration Test Suite
- [ ] Full integration test pass (all scenarios from Phase 1C)
- [ ] Property test pass (`proptest` harness)
- [ ] Kani proofs pass (if `kani-proofs` feature)

### 5C: Documentation Final
- [ ] `scripts/docs_sync_check.sh` — zero failures
- [ ] All canonical docs reflect current state
- [ ] `REMAINING_WORK_TRACKING.md` updated
- [ ] `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` updated

### 5D: Human Escalation Points
- [ ] **[HUMAN]** Approve release timing and version number
- [ ] **[HUMAN]** Approve any remaining risk items in risk register
- [ ] **[HUMAN]** Sign off on security review findings

**Gate:** Phase 5 is complete when the gatekeeper reviewer APPROVES and the human operator signs off on escalation items.

## Swarm Operation Rules

### Autonomous Operation
The swarm operates autonomously on all implementation, testing, documentation, and optimization work. No human intervention is required for:

- Writing, modifying, and debugging code
- Running tests and fixing failures
- Generating and updating documentation
- Performing code reviews and security audits
- Managing git operations (commits, merges, rebases)
- Launching and monitoring agents

### Human Escalation
Only these decision types are escalated to the human operator:

1. **Architectural direction changes** — altering the project's core design philosophy
2. **Security/privacy trade-offs** — accepting risk vs. hardening
3. **Technology stack migrations** — adding or removing major dependencies
4. **API contract breaking changes** — changing public interfaces
5. **Release timing** — when to cut a release and what version number

### Agent Lifecycle
- Maximum 2 concurrent agents (`.claude/agent_pool.json` max_concurrent)
- Use `bash .claude/orchestrator_manager.sh pool status` to check availability
- Use `bash .claude/orchestrator_manager.sh pool launch <agent>` to start
- Use `bash .claude/orchestrator_manager.sh pool stop <id>` to stop
- If 2 slots occupied, queue task in `HANDOFF/todo/`
- On agent completion, dequeue next task and launch

### Context Management
- Use `head`, `tail`, bounded `sed` instead of full file dumps
- Prefer `grep`/`ripgrep` and `git log`/`git diff` over sequential reads
- Phrase prompts with explicit parallel language to trigger concurrent execution
- Monitor context window — if approaching limits, trigger `consolidate-memory` skill
- Use MEMORY.md pointer index for structural lookups, not full file contents

### Model Availability
Before launching any agent, verify model availability:
```bash
bash .claude/model_validation_template.sh
```
Or use WebFetch: `https://ollama.com/api/tags`

If primary model unavailable, use designated fallback from routing matrix. Never silently downgrade from Tier 1 to Tier 3.
