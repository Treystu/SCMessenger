# v0.4.0 Orchestration Prompt for Kimi K3

**Authority:** Lucas (operator). You are orchestrating the final push to ready v0.4.0 (the Josh alpha test) for Lucas to hand to his brother for real-world testing.

**Goal:** Two independent identities (Lucas + Josh) exchange delivered+receipted messages over the real internet via the AWS alpha relay. APK + CLI downloadable and installable. All v0.4.0 tasks committed locally; CI green before push.

**Scope:** v0.4.0 only. Not v1.0.0 full. v1.0.0 (farm sim + iOS + PQC + KMP) is separate.

**Status:** As of 2026-07-20: relay LIVE at 100.56.248.69:9001. Lucas's CLI + emulator both connected and proven via `ss -tn`. CI was red (Lint + FFI Surface + Test failures) but commit bc94ffbb fixed all three locally — not yet pushed. A-04 and D-05 prior dispatches produced empty output and need re-dispatch.

---

## CRITICAL: You Must Verify This First

Before dispatching anything, verify on the actual Windows dev machine:

1. Check GitHub Actions live status (public UI, no login needed):
   - Go to `https://github.com/Sovereign-Communication/SCMessenger/actions`
   - Check the latest 3 runs on `main`
   - **Report back:** Are they red or green? List the failed jobs if red.

2. Verify the CI fix commit locally:
   ```
   cd SCMessenger
   git log --oneline -5
   # Look for "bc94ffbb" (iOS Swift compile errors, NDK env, ffi exec bit, 9876 health port unification)
   # It should be in recent history
   cargo clippy --workspace --all-features -- -D warnings
   # Should exit 0 locally
   cargo test --workspace --all-features --no-run
   # Should exit 0 (compile only, not running tests yet — that's slow)
   ```

3. Verify the relay is still live:
   ```
   curl http://100.56.248.69:9876/health
   # Should return {"status":"healthy"}
   ```

**Report these three items before proceeding.**

---

## Task Breakdown for v0.4.0

### Phase 0: Push CI fix (PREREQUISITE for everything else)

**Current state:** Commit bc94ffbb exists locally, fixes all three CI failures. NOT pushed yet. This blocks CI green on origin, which blocks testing artifacts on GitHub.

**What you must do:**
1. **Verify** bc94ffbb fix is real locally (the verification step above does this).
2. **Lucas pushes only.** You do NOT push — Lucas must approve and run `git push origin main`.
3. **Wait for CI to run** on GitHub Actions (typically 5-10 min). Verify all three jobs pass:
   - Lint (`cargo clippy --workspace --all-features -- -D warnings`)
   - FFI Surface Contract (`scripts/ffi_surface.sh`)
   - Test on ubuntu-latest, windows-latest, macos-latest

**Acceptance criteria:** CI.yml shows green check mark on the commit on GitHub Actions.

---

### Phase 1a: Prove second independent endpoint (CRITICAL for Josh test credibility)

**Current state:** Lucas's two identities (CLI + emulator) proven via relay. Josh's path still unproven.

**Ticket:** `HANDOFF/todo/PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md`

**Exactly what to do:**
1. Spin up a **second CLI instance** on the same Windows dev machine (separate data dir, fresh identity "Josh"):
   ```bash
   mkdir -p C:\Users\SCM\.scm_josh
   cd C:\Users\SCM\.scm_josh
   scm init
   scm config bootstrap add /ip4/100.56.248.69/tcp/9001
   scm start
   # Leave this running in a second terminal
   ```

2. From Lucas's CLI in another terminal:
   ```bash
   scm contact add <Josh's public_key>  # Exchange keys however
   # Send a message to Josh
   scm send <josh_id> "Hello Josh, can you see this?"
   ```

3. Verify from Josh's side:
   ```bash
   # In Josh's CLI terminal
   scm status
   # Should show Lucas as a connected peer
   scm inbox
   # Should show the message received + delivered receipt
   ```

4. **Reverse direction:** Josh sends to Lucas, verify receipt.

5. **Document the proof:**
   - Take terminal output showing both directions.
   - Move ticket to done/ with evidence logged.
   - Update `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-21.md` (create new dated doc).

**Acceptance criteria:** Both directions work, both show delivered + receipt ACK, proof logged.

---

### Phase 1b: Install Artifact for Alpha Testers

**Current state:** No public downloadable link yet. Local APK + CLI binaries exist but not on GitHub Releases.

**Ticket:** `HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md`

**What to do:**
1. **Do NOT push or tag yourself.** Only Lucas tags.
2. **Verify the workflow:** Read `.github/workflows/release.yml` once CI is green.
   - Does it build Windows CLI, Android APK, Linux/macOS CLI, WASM?
   - Does it auto-create a GitHub Release when you push a tag?
   - Does it upload artifacts as release assets?
3. **Prepare the tag command** for Lucas (write it in a ticket update or in chat):
   ```bash
   git tag v0.4.0-alpha.1
   git push origin v0.4.0-alpha.1
   # (Lucas runs this, not you)
   ```
4. **Wait for GitHub Actions** to run the release workflow (typically 10-30 min).
5. **Verify the GitHub Release page:**
   - URL: `https://github.com/Sovereign-Communication/SCMessenger/releases/tag/v0.4.0-alpha.1`
   - Has downloadable APK + CLI binaries as release assets.
6. **Generate the share link for Josh:**
   - Direct him to the release page.
   - For Android: download `app-debug.apk`, install via `adb install app-debug.apk` or direct transfer.
   - For Windows: download `scmessenger-cli.exe`, copy to PATH, run `scm init && scm config bootstrap add /ip4/100.56.248.69/tcp/9001 && scm start`.

**Acceptance criteria:** GitHub Release page live with downloadable artifacts. Lucas has reviewed the page and approved sending to Josh.

---

### Phase 1c: Graceful AF dial — items 3-4 (per-peer backoff + prefer relay)

**Current state:** Items 1-2 done + adversarial review PASS. Items 3-4 still open.

**Ticket:** `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`

**What to do:**
1. **Read the ticket.** Sections "Items 3+4" specify exactly what's needed:
   - Per-peer backoff state (5s, 30s, 120s, 5min, 30min exponential).
   - Max 3 concurrent outbound dials to unknown peers.
   - After relay circuit established: prefer circuit-relay over direct dials.

2. **Assess complexity:** Items 3-4 need dial-loop restructuring (cli/src/ledger.rs + cli/src/main.rs).
   - If straightforward: dispatch to Qwen THINK (design) then CODER.
   - If unclear or touching transport: ask Lucas before dispatching.

3. **Dispatch steps:**
   - Qwen THINK: Read ticket, produce a 1-page design note (data structure for per-peer state, dial loop pseudocode).
   - Fusion Lite: Review the design note.
   - Qwen CODER: Implement based on design note + approved spec.
   - Mandatory gate: `crypto-security-auditor` review (transport-layer concern).

4. **Acceptance:** 
   - `cargo check` + `clippy -D warnings` clean on cli/.
   - Adversarial review verdict file in HANDOFF/review/.
   - Ticket moved to done/ with design note + implementation diff + review verdict attached.

---

### Phase 1d: A-04 Android receipt unification (re-dispatch)

**Current state:** Prior Qwen dispatch failed (0-byte output). Ticket sits in HANDOFF/IN_PROGRESS/.

**Ticket:** `HANDOFF/IN_PROGRESS/A-04_ANDROID_RECEIPT_UNIFICATION.md` (or the done/ copy if it was already moved)

**What to do:**
1. **Read the ticket.** It's straightforward: call core's unified `encode_receipt()` / `decode_receipt()` from Kotlin instead of duplicating receipt logic.

2. **Why prior dispatch failed:** Qwen model was insufficient (qwen3-coder-plus doesn't handle Android/Kotlin well sometimes) or the input was malformed.

3. **Re-dispatch to Qwen CODER (upgraded tier):**
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

4. **Acceptance:**
   - `cd android && ./gradlew :app:compileDebugKotlin` exits 0.
   - Diff shows receipt functions using core's exported UniFFI bindings, not local duplication.
   - Ticket moved to done/.

---

### Phase 1e: D-05 Unwrap/panic hardening (re-dispatch, careful scope)

**Current state:** Prior dispatch failed (0-byte). Ticket sits in HANDOFF/done/ but actual implementation state is unclear. **This is lower priority for v0.4.0 — only do if time permits after 1a-1d.**

**Ticket:** `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md`

**What to do:**
1. **Revert scope-creep first** (orchestrator action, not a dispatch):
   ```bash
   git diff core/src/api.udl
   # If Receipt dict or encode_receipt_from_components appear: revert them
   git checkout HEAD -- core/src/api.udl
   # (only if needed)
   ```

2. **Run compile gate:**
   ```bash
   CARGO_INCREMENTAL=0 cargo check --workspace
   CARGO_INCREMENTAL=0 cargo test --workspace --no-run
   ```

3. **If errors:** Dispatch targeted Qwen HAIKU fix for the exact errors only.

4. **Move to done/** if compile gate passes.

---

### Phase 1f: Swarm port-exclusion fix (file + fix)

**Current state:** Not yet filed. Must exclude control-API port (9876) from swarm's adaptive port listener to prevent port-collision bug class recurrence.

**What to do:**
1. **File the ticket** (orchestrator action):
   ```
   HANDOFF/todo/SWARM_EXCLUDE_CONTROL_API_PORT_FROM_LISTENER.md
   
   Title: Swarm adaptive listener must exclude control-API port from its own port range
   
   Description: The swarm's adaptive port listener (core/src/transport/swarm.rs) 
   should exclude port 9876 (control API) from its port ladder so the earlier 
   port-collision bug class (where swarm's 8080 and API's 9876 raced) can't 
   recur by construction.
   
   Files: core/src/transport/swarm.rs
   
   Tier: [SONNET]
   Gate: cargo check + cargo test --workspace --no-run
   Audit: crypto-security-auditor (transport/)
   ```

2. **Dispatch to Qwen CODER:**
   - Read the source (`core/src/transport/swarm.rs` around the port-binding logic).
   - Filter out 9876 from the port ladder.
   - Test the change locally.
   - Mandatory adversarial review.

3. **Acceptance:** Compile gate clean, adversarial review PASS, ticket done/.

---

## Sequencing & Parallelism

```
Phase 0: CI fix push (BLOCKER, must be done first)
         ↓
Phase 1a: Prove second endpoint (proof of concept for Josh path)
Phase 1b: Install artifact (release workflow verification)
Phase 1c: Graceful AF items 3-4 (design + impl, can run in parallel with 1b)
Phase 1d: A-04 receipt unif (re-dispatch, depends on 1c being mostly stable)
Phase 1e: D-05 hardening (low priority for v0.4.0, do if time)
Phase 1f: Swarm port-exclusion (small fix, can run anytime)
```

**Parallelism safe:**
- 1a, 1b, 1c can run concurrently (different file domains + test scenarios).
- 1d depends on 1c not introducing UDL changes (so Kotlin bindings are stable).
- 1e + 1f can be truly parallel (independent changes).

---

## Critical Gates & Rules

1. **No push to origin** — Lucas pushes after you verify locally.
2. **No tag/release** — Lucas tags after CI is green and all Phase 1 items are committed.
3. **Transport/crypto changes** — Mandatory adversarial review before done (1c + 1f both qualify).
4. **Compile gate before commit** — `cargo check --workspace` + `cargo clippy -- -D warnings` on any change.
5. **Delivery logic (receipt/outbox/custody)** — Needs triangulation (3 independent Qwen verifiers OR 1 Fusion panel) before done (1d qualifies).

---

## Success Criteria for v0.4.0

When all of Phase 0-1 are done:

- [ ] CI.yml green on origin/main (all three jobs pass).
- [ ] Phase 1a proven locally (two CLI instances messaging bidirectional via relay).
- [ ] Phase 1b release link live and assets downloadable.
- [ ] Phase 1c implemented + adversarial review PASS.
- [ ] Phase 1d committed + Kotlin compiles.
- [ ] All changes committed locally (not pushed).
- [ ] Session handoff written.

Then: **Lucas reviews, approves, pushes, tags v0.4.0-alpha.1, sends to Josh.**

---

## Tools & Resources You'll Need

- **Local:** Windows dev machine, SCMessenger repo, Cargo + Android toolchain.
- **Delegation:** `scripts/delegate_task.py` (Qwen/Groq/OpenRouter).
- **Verification:** `scripts/fusion_lite.py` (Fusion Lite 2-model panel).
- **CI:** GitHub Actions public status page (no auth).
- **HANDOFF:** `HANDOFF/V040_ORCHESTRATION_PLAN.md`, session handoffs, ticket files.

---

## Start Here

1. Run the verification checks at the top of this prompt.
2. Report the three items (GitHub Actions status, bc94ffbb local verification, relay health).
3. Once Lucas confirms all three are good, proceed with Phase 0 (push).
4. Then Phase 1a (second endpoint proof).
5. Parallelize the rest.

Good luck. You have the authority to make implementation decisions, commit locally, and dispatch freely within budget. Ask only if you hit a genuine structural deadlock or a security/architecture judgment call.
