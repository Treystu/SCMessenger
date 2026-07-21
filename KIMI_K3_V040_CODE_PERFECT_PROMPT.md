# v0.4.0 Code-Perfect Push  Kimi K3 Orchestration

**Date:** 2026-07-21  
**Goal:** Complete v0.4.0 (Josh alpha test) code work tonight. Code perfect. Testing next.  
**Skip:** Farm simulation, v1.0.0 scope items.  
**Authority:** Lucas (operator).

---

## VERIFIED STATE (as of 2026-07-21, checked via subagent)

 **Done and pushed:**
- Relay live at `100.56.248.69:9876` (health: healthy)
- All recent commits on `origin/main` (HEAD = 50e8ab8c, SwiftLint fixes)
- Two-endpoint delivery proof committed (1f6e47b8)
- Phase 0 (CI fix): bc94ffbb fixes are in place and pushed

️ **Must verify before proceeding:**
- Compile gate: `cargo clippy --workspace --all-features -- -D warnings`  **needs local Windows verification** (not run from bash env)
- 980 uncommitted changes in working tree (mostly orchestration/CI, not core platform code)  **must confirm no blocking conflicts**

 **Not yet done for v0.4.0:**
- Phase 1b: Install artifact (release.yml validation)
- Phase 1c: Graceful AF items 3-4 (per-peer backoff + prefer relay)
- Phase 1d: A-04 Android receipt unification (re-dispatch  prior attempt failed)
- Phase 1e: D-05 unwrap/panic hardening (low priority, conditional)
- Phase 1f: Swarm port-exclusion fix (file + implement)

---

## IMMEDIATE NEXT STEP: Compile Gate Verification

**You must run on Windows dev machine (PowerShell or native shell):**

```powershell
cd C:\Users\SCM\Documents\GitHub\SCMessenger

# 1. Confirm no stale git lock
rm -Force .git/index.lock -ErrorAction Ignore
git status

# 2. Run compile gate (exactly as CI will)
$env:CARGO_INCREMENTAL=0
cargo clippy --workspace --all-features -- -D warnings

# Report back:
# - Exit code (0 = PASS, non-zero = FAIL)
# - Any clippy errors (if FAIL, copy the exact error lines)
```

**Do NOT proceed to Phase 1b-1f until compile gate is verified as PASS locally.**

---

## IF COMPILE GATE PASSES: Phase 1b-1f Task Sequence

Once `cargo clippy` exits 0, the remaining v0.4.0 work is:

### Phase 1b: Install Artifact (Release Workflow)

**What:** Validate `.github/workflows/release.yml` builds APK + CLI binaries.

**Steps:**
1. Read `.github/workflows/release.yml` (already exists; check if it's complete).
2. Verify it triggers on `git tag v0.4.0-alpha.*` pushes.
3. Verify it builds:
   - Windows CLI (`target/release/scmessenger-cli.exe`)
   - Android APK (`app/build/outputs/apk/debug/app-debug.apk`)
   - Linux CLI (if applicable)
4. Confirm it auto-creates a GitHub Release with downloadable assets.
5. **Prepare** (do NOT push tag yet):
   ```powershell
   # This is what Lucas will run when code is perfect:
   git tag v0.4.0-alpha.1
   git push origin v0.4.0-alpha.1
   ```

**Acceptance:** Release workflow documented, tag command ready for Lucas.

**Tier:** [HAIKU]  read + verify, no code changes.

---

### Phase 1c: Graceful AF Items 3-4 (Per-Peer Backoff + Prefer Relay)

**Ticket:** `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md` (items 3-4)

**What it does:**
- Per-peer backoff state: 5s  30s  120s  5m  30m exponential.
- Max 3 concurrent outbound dials to unknown peers.
- After relay circuit established: prefer circuit-relay over direct dials (no promiscuous dial spam).

**Current state:** Items 1-2 done + adversarial review PASS. Items 3-4 need dial-loop restructuring.

**Dispatch plan:**
1. **Design first (Qwen THINK):**
   - 1-page design note: per-peer state data structure, dial-loop pseudocode.
   - Acceptance: clear spec for CODER to follow.
2. **Verify design (Fusion Lite):**
   - 2-model panel review of design note. Cost cap: $0.01.
3. **Implement (Qwen CODER):**
   - Follow design spec exactly. Files: `cli/src/ledger.rs`, `cli/src/main.rs`.
   - Acceptance: `cargo check -p scmessenger-cli`, `cargo clippy -p scmessenger-cli -- -D warnings` both exit 0.
4. **Security audit (fable adversarial):**
   - Mandatory gate: transport/routing concerns.
   - Ticket: `HANDOFF/review/GRACEFUL_AF_DIAL_POLICY_ITEMS_3_4_AUDIT.md`
   - Verdict must be on file before commit.

**Acceptance criteria:**
- Design note + design review verdict in HANDOFF/.
- Implementation diff committed locally.
- Adversarial review verdict in HANDOFF/review/.
- Ticket moved to done/.

**Tier:** [OPUS+/THINK] design  [SONNET] impl  [AUDIT-GATE] fable review.

---

### Phase 1d: A-04 Android Receipt Unification (Re-Dispatch)

**Ticket:** `HANDOFF/IN_PROGRESS/A-04_ANDROID_RECEIPT_UNIFICATION.md`

**What it does:** Call core's unified `encode_receipt()` / `decode_receipt()` from Kotlin (via UniFFI) instead of duplicating receipt logic locally.

**Why re-dispatch:** Prior Qwen dispatch produced 0-byte output (silent failure). Model was insufficient.

**Dispatch:**
```bash
python scripts/delegate_task.py \
  --task HANDOFF/IN_PROGRESS/A-04_ANDROID_RECEIPT_UNIFICATION.md \
  --provider qwen \
  --tier coder-max \
  --files android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt \
  --apply \
  --verify "cd android && ./gradlew :app:compileDebugKotlin -x lint --quiet" \
  --max-rounds 2
```

**Acceptance:** 
- `./gradlew :app:compileDebugKotlin` exits 0.
- Diff shows receipt handling using core's UniFFI exports, not local code duplication.
- Ticket moved to done/.

**Tier:** [SONNET] Kotlin-side.

---

### Phase 1e: D-05 Unwrap/Panic Hardening (OPTIONAL  low priority)

**Ticket:** `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md`

**Status:** Lower priority for v0.4.0. Only do if time permits after 1b-1d.

**Quick check:**
```bash
CARGO_INCREMENTAL=0 cargo check --workspace
CARGO_INCREMENTAL=0 cargo test --workspace --no-run
```

If compile gate passes: ticket already done. If errors: dispatch targeted Qwen HAIKU fix for exact errors only.

**Tier:** [HAIKU/SONNET] conditional.

---

### Phase 1f: Swarm Port Exclusion (Small Fix)

**Ticket:** File new  `HANDOFF/todo/SWARM_PORT_EXCLUSION_FIX.md`

**What it does:** Prevent swarm's adaptive port listener from using port 9876 (control-API port), eliminating a class of port-collision bugs.

**Dispatch:**
1. File ticket with spec (1 paragraph: exclude 9876 from port ladder).
2. Dispatch Qwen CODER: modify `core/src/transport/swarm.rs` port-binding logic.
3. Mandatory adversarial review (transport/).
4. Accept: compile gate clean, adversarial review PASS, ticket done/.

**Tier:** [SONNET] impl  [AUDIT-GATE] fable review.

---

## Sequencing & Parallelism

```
Prerequisite: Compile gate PASS

Phase 1b (release.yml)  independent, quick, can run anytime
Phase 1c (graceful AF)  THINK design (parallel start)  Fusion review  CODER impl  fable audit
Phase 1d (A-04 receipt)  parallel with 1c's design phase; depends on 1c being mostly stable
Phase 1e (D-05)  optional, run if time; independent
Phase 1f (swarm port)  small, independent, run anytime
```

**Safe parallelism:**
- 1b + 1c start in parallel (1c: THINK design; 1b: read workflow).
- 1d starts after 1c's THINK phase completes (unblocks Kotlin re-dispatch).
- 1e + 1f can be truly parallel (no dependencies).

---

## Critical Gates & Rules (Non-Negotiable)

1. **No push to origin**  Lucas pushes after you verify locally.
2. **No tag/release**  Lucas tags after all Phase 1 items committed.
3. **Transport/crypto changes (1c + 1f)**  Mandatory adversarial review before commit. No exceptions.
4. **Delivery logic (1d receipt)**  Triangulation: 3 independent Qwen verifiers OR 1 Fusion panel.
5. **Compile gate before every commit**  `cargo check --workspace` + `cargo clippy --workspace -- -D warnings` must exit 0.
6. **Move tickets through state machine**  todo/  IN_PROGRESS  review/ (if gated)  done/.

---

## Success Definition for v0.4.0 (Tonight)

When all of 1b-1f are done:

- [ ] Compile gate verified locally (Windows PowerShell).
- [ ] Release workflow validated (1b).
- [ ] Graceful AF items 3-4 implemented + audited (1c).
- [ ] A-04 Android receipt re-dispatched + compiles (1d).
- [ ] D-05 or swarm-port done (1e/1f, or skipped as time-permits).
- [ ] All changes committed locally (not pushed).
- [ ] Session handoff written (`HANDOFF/SESSION_HANDOFF_2026-07-21_KIMI_K3.md`).

Then: **Lucas reviews, approves, pushes, tags v0.4.0-alpha.1, sends to Josh.**

---

## Tools & Resources

- **Delegation:** `scripts/delegate_task.py` (Qwen CODER/THINK, max-tier model for A-04 re-dispatch).
- **Verification:** `scripts/fusion_lite.py` (1c design review, $0.01 cap).
- **CI:** `.github/workflows/release.yml` (read-only).
- **HANDOFF:** Ticket files, state machine (todo/  done/).
- **Local:** Windows dev machine, PowerShell, Cargo, Android toolchain.

---

## START HERE (Tonight)

1. **Run compile gate** on Windows:
   ```powershell
   cd C:\Users\SCM\Documents\GitHub\SCMessenger
   rm -Force .git/index.lock -ErrorAction Ignore
   $env:CARGO_INCREMENTAL=0
   cargo clippy --workspace --all-features -- -D warnings
   # Report: PASS or FAIL + any errors
   ```

2. **If PASS:** Proceed with Phase 1b-1f in order (or parallelized as noted).

3. **If FAIL:** Report the exact clippy error lines. Dispatch targeted Qwen HAIKU fix for just those errors.

4. **Commit and move tickets** through HANDOFF state machine as work lands.

5. **Write session handoff** when done.

---

**You have authority to:**
- Dispatch freely to Qwen/Groq/Fusion within cost ceilings.
- Make implementation-detail decisions within ticket scope.
- Commit locally and move tickets through the state machine.
- Escalate to Lucas only if: security judgment call, architecture decision, ambiguous scope, or blocked deadlock.

**You do NOT:**
- Push to origin (Lucas does).
- Tag or publish release (Lucas does).
- Skip security gates (adversarial review is mandatory on transport/crypto).

Go. Code perfect. Report when v0.4.0 is commit-ready.
