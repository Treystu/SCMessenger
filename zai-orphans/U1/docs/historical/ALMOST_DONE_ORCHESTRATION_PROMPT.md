# "Almost Done!" — SCMessenger v1.0.0 Completion & Readiness Verification

**Objective:** Verify the codebase is production-ready, run comprehensive sanity checks, ensure Windows/Android parity, and confirm 99% likelihood of successful testing.

**Scope:** The codebase is at v1.0.0 with all major code tasks (S2-S8, T1-T17) verified complete. This run validates final state and gates readiness.

---

## Task: Orchestrate Final Completion

### Route to Specialized Agents (Use Best Models for Each)

**Agent 1: Core & Rust Validation** (Model: `glm-5.1:cloud`)
- Run `cargo build --workspace` and record output
- Run `cargo test --workspace --no-run` (compile gate only)
- Run `cargo fmt --all -- --check` and `cargo clippy --workspace`
- Check `docs/release-readiness-2026-07-02.md` for known code blockers
- Report: build status, any failures, remediation if needed

**Agent 2: Android Build & Parity Check** (Model: `qwen3-coder-next:cloud`)
- Run `cd android && ANDROID_HOME=[detected] ./gradlew assembleDebug -x lint --quiet`
- Compare Android `MeshRepository.kt`, transport/BLE, messaging paths against iOS equivalents
- Document exact Windows/Android parity score (features, transports, message flow, identity handling)
- Report: build status, parity gaps (if any), remediation

**Agent 3: iOS/WASM Build & API Parity** (Model: `qwen3-coder-next:cloud`)
- Verify `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes
- Run `./iOS/copy-bindings.sh` and `./iOS/verify-test.sh`
- Compare WASM `wasm/src/lib.rs` JSON-RPC surface against Android/iOS APIs
- Check notification, settings, identity, contact, history parity across all three platforms
- Report: build status, API coverage completeness, gaps if any

**Agent 4: Script & Docs Sync Verification** (Model: `gemini-3-flash-preview:cloud`)
- Run `./scripts/docs_sync_check.sh` (or `.ps1` on Windows)
- Scan `REMAINING_WORK_TRACKING.md` for any open "[ ]" items
- Verify critical docs (CLAUDE.md, CURRENT_STATE.md, release-readiness) are in sync with code
- Report: docs sync status, any stale checkpoints, required updates

**Agent 5: Final Readiness Report** (Model: `kimi-k2.6:cloud`)
- Aggregate all four agent reports
- Assess: Can code pass testing with 99% confidence?
- List any residual risks or blockers (code-level, infrastructure, physical hardware)
- Produce one final "Release Readiness" doc with exact go/no-go recommendation

---

## Success Criteria

✅ All builds pass locally (Windows Rust, Android Gradle, iOS bindings, WASM).  
✅ Format and lint gates are clean.  
✅ Compilation tests pass (no runtime required).  
✅ Windows/Android messaging, transport, identity, contact, history, settings, notification, and privacy API surfaces achieve ≥95% parity.  
✅ Docs are in sync with code state.  
✅ No code-level blockers remain.  
✅ Final readiness report confirms 99% testing confidence or documents exact residual risks.

---

## Known Context

- **Completed:** All S-tasks (S2-S8, automation/scripts), all T-tasks (T1-T17, Rust/CLI/Android/iOS codebase fixes)
- **Blocked by infrastructure (not code):** H1 (GitHub Actions runners), H2 (physical device tests)
- **Architecture:** Rust core (`scmessenger-core`), CLI daemon, Android/Kotlin app, iOS/Swift app, WASM/browser client
- **Key parity target:** Windows/Android must be completely interoperable (message delivery, identity federation, contact persistence, notification handling, privacy settings)
- **Build gates:** `cargo check --workspace`, `./gradlew assembleDebug`, `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`, docs sync check
- **Definition of "ready to test":** Code passes all sanity checks, we are 99% confident it will work during real testing (no unexpected runtime surprises)

---

## Prompt for Co-Work / Agents

*You have comprehensive project documentation in CLAUDE.md, CURRENT_STATE.md, and REMAINING_WORK_TRACKING.md. The codebase snapshot is complete. Your job is to systematically verify all five areas above, aggregate findings, and produce a final readiness gate. Aim for high confidence — catch any missed issues before testing.* ✨

