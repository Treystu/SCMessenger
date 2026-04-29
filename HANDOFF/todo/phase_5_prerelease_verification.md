# Phase 5: Pre-Release Verification (Gatekeeper Review)

**Priority:** P0 (Release gate)
**Assigned Agent:** gatekeeper-reviewer (kimi-k2-thinking:cloud)
**Fallback:** kimi-k2.6:cloud
**Status:** TODO
**Depends On:** phase_4_documentation_polish
**REQUIRES HUMAN SIGN-OFF**

## 5A: Build Verification
- [ ] `cargo build --workspace` — release mode
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `./gradlew assembleDebug` — Android
- [ ] `wasm-pack build --target web` — WASM

## 5B: Integration Test Suite
- [ ] Full integration test pass (all scenarios from Phase 1C)
- [ ] Property test pass (`proptest` harness)
- [ ] Kani proofs pass (if `kani-proofs` feature)

## 5C: Documentation Final
- [ ] `scripts/docs_sync_check.sh` — zero failures
- [ ] All canonical docs reflect current state
- [ ] `REMAINING_WORK_TRACKING.md` updated
- [ ] `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` updated

## 5D: Human Escalation Points (REQUIRE EXPLICIT APPROVAL)
- [ ] **[HUMAN]** Approve release timing and version number
- [ ] **[HUMAN]** Approve any remaining risk items in risk register
- [ ] **[HUMAN]** Sign off on security review findings

## Gate Criteria
APPROVE only when ALL checklist items pass. REQUEST CHANGES for specific deficiencies. BLOCK for critical issues requiring architect or human intervention.

## Rules
- Use `.claude/prompts/gatekeeper.md` prompt template
- Use `.claude/skills/build_verify.sh` skill for build verification
