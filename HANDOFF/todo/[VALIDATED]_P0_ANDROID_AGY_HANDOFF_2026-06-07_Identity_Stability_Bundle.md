# [VALIDATED] P0_ANDROID — Agy Handoff: Identity Stability & UI Bundle (6 bugs, 1 ops ref)

**Status:** READY-FOR-DISPATCH
**Priority:** P0 (1 race) + P1 (4) + P2 (1) — all ship-block
**Source:** Agy (Gemini Pro) ran 2026-06-07 19:46 → 20:24 PT. Full handoff in `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`
**Agy process:** STOPPED (PID 19276, 20:24 PT)
**Quota state:** 5h=1.2%, 7d=0.2% — TIER 1, 240 min to reset
**Slots:** 3/3 free (Agy didn't count; local LLMs don't count)

---

## What's in this bundle (from Agy's 35-min investigation)

Agy did real on-device Android testing + local swarm testing. Findings are concrete, evidence-backed, and ready to land.

### Bug 1 (P0) — Concurrent `createIdentity()` race
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**Symptom:** 4 threads call `ironCore.initializeIdentity()` simultaneously. Log evidence (01:48:19) shows 4 entries in 12ms.
**Fix:** Add `private val identityCreationMutex = Mutex()` and wrap `createIdentity()` body in `mutex.withLock { ... }`.
**Pre-check:** Skip if `ironCore?.getIdentityInfo()?.initialized == true`.

### Bug 2 (P1) — `IdentityViewModel` re-entrancy guard missing
**File:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityViewModel.kt`
**Fix:** Add `_isCreating` StateFlow gate.

### Bug 3 (P1) — `createIdentity()` fires when identity already exists
**Same fix as Bug 1 (pre-check).**

### Bug 4 (P2) — Redundant backup writes (10+ in 30s)
**File:** `MeshRepository.kt` `ensureLocalIdentityFederation()` lazy path.
**Fix:** AtomicBoolean latch, run once per process lifecycle.

### Bug 5 (P1) — Brief identity "disappearance" (UI flash)
**File:** `MeshRepository.kt` `isIdentityInitialized()` returns `true` before restore completes.
**Fix:** Await the restore job before returning.

### Bug 6 (P1) — mDNS / discovered peers disconnect
**File:** `android/app/src/main/java/com/scmessenger/android/discovery/MdnsServiceDiscovery.kt`
**Symptom:** `MdnsServiceDiscovery: mDNS peer removed from discovered list...`
**Live signature from Agy's task-115.log (dozens of `os error 10061 — No connection could be made because the target machine actively refused it`):**
```
2026-06-07T08:51:18Z ⚠ Outgoing connection error to 12D3KooWPD7Mkc9k5Xjyk6BJW2zUKCQpAXEFxv6PvyWCjwiB9z98: ...tcp/9001: : No connection could be made because the target machine actively refused it. (os error 10061)
```
**Likely cause:** Listener-port mismatch between mDNS-advertised port and SwarmBridge's expected port. Audit both sides.

### UI Bug A (P2) — Left column not weighted in `ContactItem`
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` line 333
**Fix:** `Column(modifier = Modifier.weight(1f)) { ... }` (matches `NearbyPeerCard` at `AddContactScreen.kt:617`).

### UI Bug B (P2) — Missing bottom padding for `+` FAB
**File:** `ContactsScreen.kt` (LazyColumn) + `MeshApp.kt` (outer Scaffold)
**Fix:** Apply `paddingValues` to LazyColumn contentPadding so the FAB doesn't overlap the last list item.

### Bonus — CLI Operations Reference
**Source:** Agy's structured message `e854a371-f1dd-482e-a6e4-83edd6f144bb.json`
Full CLI command list, port map (9000/9001/9002), config.json schema, bootstrap nodes, 4 ways to start the node. **Promote to `docs/CLI_OPERATIONS_REFERENCE.md` after a worker validates against current source.**

---

## Recommended dispatch

**Option A — Single worker (1 slot, ~1800s budget):** Land Bugs 1–5 + UI A/B in one go. All touch overlapping files (MeshRepository, IdentityViewModel, ContactsScreen). Use `rust-coder:7b` or `scm-coder:7b`. Bug 6 is the riskiest (mDNS) — keep it for a separate verification pass.

**Option B — Two workers (2 slots):**
- **Slot 1 (scm-coder:7b):** Bugs 1–5 + UI A/B (single Kotlin/Android workstream)
- **Slot 2 (scm-thinker:14b):** Audit Bug 6 (mDNS listener port vs SwarmBridge) — read-only review, write a fix plan to `HANDOFF/done/`

**Option C — Cloud override (free cloud slot, your "fire on all cylinders" mode):**
- Flip `agent_pool.json` policy from `local_only` to `mixed`
- **Slot 1:** `qwen3-coder-next:cloud` (or `glm-5.1:cloud`) — implement Bugs 1–5 + UI A/B
- **Slot 2:** `qwen3-coder:480b:cloud` (or `minimax-m3:cloud`) — audit Bug 6 with deep architecture review
- **Slot 3:** `deepseek-v3.2:cloud` — review the mDNS fix for security/correctness

---

## Pre-flight artifacts

| File | Purpose |
|---|---|
| `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` | **Full ticket** — read this first |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-…\android_issue_catalog.md` | Original 6-bug catalog with log evidence |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-…\.system_generated\messages\a8e8635e-…` | Contact UI research |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-…\.system_generated\messages\e854a371-…` | CLI Setup research |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-…\.system_generated\tasks\task-115.log` | Overnight swarm test (152 KB) |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-…\.system_generated\tasks\task-389.log` | Current session swarm test (528 KB) |

All WSL-readable at `/mnt/c/Users/SCMessenger/.gemini/antigravity-cli/...`

---

## Verification gates (must pass before moving to `done/`)

- [ ] `cargo check --workspace` — clean
- [ ] `./gradlew assembleDebug` — builds
- [ ] `./gradlew :app:testDebugUnitTest` — passes
- [ ] `clippy --workspace -D warnings` — no new warnings
- [ ] No new uncommitted files (the 2 dirty test files from 2026-06-05 must be either committed or stashed)
- [ ] `./scripts/docs_sync_check.sh` — passes
- [ ] If Bug 6 fix lands, manual verification: 2 CLI nodes start, mDNS peer stays connected for ≥60s (requires Pixel 6a back online, currently blocked)

---

## Do NOT do

- ❌ Don't reopen the uncommitted test files (`MeshRepositoryHistoryTest.kt`, `BleScannerTest.kt`) — they're leftover from the 2026-06-05 P0_024 fix and should be in their own commit
- ❌ Don't delete the `IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` file — it's the source of truth for the work
- ❌ Don't use `local_only` override without Lucas's explicit GO

---

## Orchestrator decision tree

```
If quota is TIER 1 and Lucas hasn't said "fire on all cylinders":
    Use Option A (single local worker, scm-coder:7b)
If Lucas says "fire on all cylinders" or "use the cloud slot":
    Use Option C (cloud override, 2-3 slots)
If Pixel 6a is offline AND Bug 6 needs live retest:
    Land Bugs 1–5 + UI A/B now, defer Bug 6 to a verification slot
```

---

*Dispatched by Hermes (passive audit) on behalf of Lucas — 2026-06-07 20:27 PT*
