# SCMessenger Release Readiness Assessment — 2026-07-02

Assessment of `main` @ `cd582f8` and PR #1 (`claude/v1-0-0-code-gaps-7d849x`,
head `1f52b42`). Everything below marked **verified** was reproduced in this
session with a real command run; everything marked **unverified** says why.

> **Updated post-merge (2026-07-02):** PR #1 has since been merged into
> `main` as `cbec1f4` — H3 is done and this branch is rebuilt on the merged
> result (the S1 reconciliation recipe below was executed here: take main's
> content, re-run `cargo fmt`). The Actions run on the merge commit
> (`28575087181`) shows the **runners are still dead** — every job again
> "completed" in 2 s with `runner_id: 0` and no logs — so **H1 remains the
> blocking item**. §7 (new) triages all 26 review comments left on PR #1
> into concrete tasks.

Scope note: the `cloud/` tree (Telegram/GCE build orchestrator) is build
tooling, not part of the product. The product is the serverless P2P mesh
(Rust core + CLI + Android/iOS/WASM). `cloud/` is excluded from this
assessment apart from one hygiene commit (untracking `.pyc` files).

---

## 1. The headline finding: CI has never run — and it is NOT a code problem

Every one of the 18 workflow runs visible on `main` (6 commits × `CI`,
`Cross`, `Mobile`, from `53370fe` 2026-06-15 through `cd582f8` 2026-07-02) is
`failure`. **None of them executed a single step.** Verified via the Actions
API on runs `28571527109` (2026-07-02) and `27523269349` (2026-06-15), which
bracket the failure window:

- Every job "completed" **1–2 seconds** after creation.
- Every job has `runner_id: 0`, `runner_name: ""` — **no runner was ever
  assigned**.
- No logs exist for any job (`404`), including jobs from the same morning —
  because nothing ever produced output. (This is why earlier log-pull
  attempts "expired": there was never anything to expire.)
- The only check that passes on PR #1 is "cubic · AI code reviewer" — the one
  check that runs on external infrastructure instead of GitHub-hosted
  runners.

This signature (instant failure, no runner, no logs, GitHub-hosted jobs only)
is an **account-level runner-assignment problem**: exhausted Actions
minutes/spending limit, a billing payment failure, or an account restriction.
It cannot be fixed from inside the repository.

**→ Human action required (see §6, item H1).** Until this is fixed, no CI
verdict on any branch means anything, and it has meant nothing since at least
2026-06-15.

## 2. What CI *would* say: the gates run locally on `main` @ `cd582f8`

Since the runners are dead, I executed the `CI` workflow's gates directly on
Linux (rustc 1.94.1 stable, the same channel `rust-toolchain.toml` pins):

| Gate (from `.github/workflows/ci.yml`) | Result | Notes |
|---|---|---|
| `cargo fmt --check` | ❌ **FAIL** → ✅ fixed on this branch | 21 diff sites across 6 files; violations date back to v1.0.0-rc2 (see §3) |
| `cargo clippy --workspace --all-features -- -D warnings` | ✅ PASS | Only after installing `libdbus-1-dev` (pulled in by `btleplug`); the workflow has no such install step — see task S2 |
| `cargo test --workspace --all-features` | ✅ **PASS** | Full workspace, zero failures (unit + 20+ integration suites + doc-tests) |
| `cargo doc --workspace --no-deps` | ✅ PASS | 7 rustdoc warnings, non-fatal |
| `cargo deny check` | ⚠️ partial | `bans licenses sources` PASS; `advisories` unverifiable here (this sandbox's network is repo-scoped; the RustSec DB clone is blocked). CI covers it once runners work |
| `cargo run --bin gen_kotlin --features gen-bindings` | ❌ **FAIL** (panic, exit 101) | cdylib not prebuilt — `cargo run` alone doesn't build it. PR #1's ci.yml fix (prebuild step) verified to resolve this locally |
| `scripts/ffi_surface.sh` | ❌ **FAIL** (and the gate is unsound) | With bindings absent it prints WARN and **exits 0 — a vacuous pass**. With bindings actually generated, it fails on a stale Kotlin snapshot. PR #1 updates the snapshots + extraction regex but does not fix the vacuous pass (task S3) |
| `cargo build --target wasm32-unknown-unknown -p scmessenger-wasm --release` (from `cross.yml`) | ✅ PASS | |

**Not verifiable in this environment:** macOS test job, iOS/Android
cross-builds and app assembly (`cross.yml`, `mobile.yml`,
`ios-build-test.yml`), Windows. These need working runners (H1) or local
hardware. There is no evidence they pass anywhere.

Net: the Rust core is in much better shape than the wall of red CI implies —
**the full test suite genuinely passes** — but two of the five CI jobs
(`Lint` via fmt, `FFI Surface Contract` twice over) would fail on `main` even
with working runners.

## 3. Trust audit: CHANGELOG and checklists vs. reality

- `CHANGELOG.md` for `1.0.0-rc2` claims `cargo fmt --check` **passed**.
  Verified false **on the very commit that added the entry**: at `0a49d32` a
  worktree run of `cargo fmt --check` fails with 13 diff sites — several in
  the same Gemini-contributed files the entry celebrates. The rc2 claims for
  Android/iOS/WASM builds and `ffi_surface.sh` are unverifiable (no CI ever
  ran; no logs exist), and the FFI gate demonstrably fails on today's `main`.
  A correction note now sits at the top of `CHANGELOG.md`.
- `tasks/T*/progress.md` unchecked boxes on `main` (T1.2 ×3, T1.3 ×2,
  T1.4 ×4, T2.4 ×4, T2.5 ×3, T4.5 ×4) are **broadly accurate** — the honest
  kind of stale. PR #1's edits to these files are also honest: it checks
  boxes it has tests for and leaves hardware-dependent boxes unchecked with
  explanations.
- `fable5plan.md`'s claim that the core subsystems are implemented and tested
  is consistent with the passing workspace suite.

## 4. PR #1 review (head `1f52b42`, 68 files, +11023/−805)

Read in full on the security-relevant surfaces. Verdict: **substantive, well
engineered, and its claims check out where they can be checked. Recommend
merging it first** (before any other backlog work), with the follow-ups in §6.

Verified strengths:

- **Argon2id backup rework (`core/src/crypto/backup.rs`)** — T4.5's "one
  likely real crypto gap" is properly closed: Argon2id (19 MiB, t=2, p=1 —
  OWASP interactive minimums) with a format-tag byte, random salt stored in
  the blob, AEAD-authenticated fallback chain for both legacy PBKDF2 formats,
  key zeroization, fails closed with `CorruptionDetected`. Tests cover format
  tag, memory-hardness, both legacy decrypt paths, and tamper rejection.
- **Backup payload v2 (`core/src/iron_core.rs`)** — identity + ratchet
  sessions + contacts, validate-everything-then-commit import (no partial
  state), audit events on export and import, legacy bare-hex fallback.
- **Message-request handling (`cli/src/server.rs`)** — accept uses the
  Ed25519 key captured from the message envelope (verified at receive time),
  not an unauthenticated discovery broadcast; reject blocks the peer;
  pending-list peeks the inbox instead of draining it.
- **WiFi Aware fixes** — async dial instead of blocking a shared tokio
  worker (documented deadlock), oneshot confirmation, IPv6 `SocketAddr`
  construction fix (unbracketed IPv6 previously could never parse).
- **T1.4 GO-intent** — Kotlin (`WifiDirectTransport.kt`, live
  `BatteryManager` state) and Rust (`compute_group_owner_intent`) match, both
  tested.
- **CI fixes** — the cdylib prebuild step in `ci.yml` verifiably fixes the
  `gen_kotlin` panic (reproduced + fix confirmed locally); FFI snapshots
  regenerated; `Cargo.lock` committed with correct `.gitignore` rationale.

Findings to carry as follow-up tasks (none should block the merge; §6 S4–S6):

1. **Contacts key-prefix migration gap** (`core/src/store/contacts.rs`) — the
   new `contact:` prefix is correct, but contacts stored by existing installs
   under bare `peer_id` keys become invisible after upgrade. No migration is
   included.
2. **`safety_number()` returns all-zeros on error** (`core/src/mobile_bridge.rs`,
   now `#[uniffi::export]`ed) — two clients fed malformed keys both render
   the same all-zero 60-digit number, which a user can "verify" as matching.
   Pre-existing behavior, but the PR promotes it into the new verification UI.
3. **`identity_signing_key_for_test`** — a plain `pub fn` handing out a clone
   of the Ed25519 signing key, reachable by any Rust consumer of the crate
   (not FFI-exported, and required by integration tests, so it cannot be
   `#[cfg(test)]` — but it should be harder to reach by accident).
4. **PR #1 fails `cargo fmt --check` itself** — 30 diff sites (inherited +
   new). One `cargo fmt` commit fixes it (§6 S1 includes the exact recipe).
5. Kotlin/Swift changes reviewed by reading only — no Android SDK/Xcode
   here. The PR's own progress notes say the same; CI (post-H1) is the gate.

Independently verified on PR #1's branch (head `1f52b42`) in an isolated
worktree: `cargo clippy --workspace --all-features -- -D warnings` **passes**
and `cargo test --workspace --all-features` **passes** (exit 0, zero
failures). Combined with the fmt fix being one mechanical command, PR #1 is
in strictly better shape than its all-red checks page suggests — every red ✗
on it is either the dead-runner problem (§1) or the fmt debt it inherited.

## 5. Fixes applied and proven on this branch

| Commit | What | Proof |
|---|---|---|
| `cargo fmt` across core | Fixes the `Lint` job's first failure on `main`; also normalizes `iron_core.rs` from CRLF to LF (whole-file diff, formatting only) | `cargo fmt --check` exits 0; full test suite re-verified green on this branch |
| Untrack `cloud/**/__pycache__/*.pyc` + ignore `__pycache__/`, `*.py[cod]` | Repo hygiene named in the briefing | `git ls-files | grep pyc` empty |
| `CHANGELOG.md` correction note | Reconciles rc2's false verification claims to evidence | §3 |
| This report | | |

## 6. Handoff — ordered, atomic tasks

### Human-only (not code; nothing downstream is trustworthy until H1 is done)

- **H1 — Restore GitHub Actions runners.** Check
  https://github.com/settings/billing (Actions minutes/spending limit and
  payment method) for the `Treystu` account; runs fail in 1–2 s with no
  runner assigned, which is billing/quota/restriction, not workflow content.
  Done when: re-running the `CI` workflow on `main` shows jobs with a
  non-empty runner name and step logs.
- **H2 — Physical-device procedures** (`docs/device-testing.md` — the doc is
  real and runnable): two-device WiFi Aware/Direct and BLE tests, three-device
  DTN mule test. These are the remaining unchecked boxes in T1.3/T1.4/T1.8
  and the v1.0.0 field-evidence gate. Requires hardware; cannot be delegated
  to a model.
- **H3 — Decide the PR #1 merge.** Recommendation: merge (§4). S1 assumes it
  merged. **Done — merged as `cbec1f4` on 2026-07-02.**

### Model-executable (each item: files, exact change, verify command, done condition)

- **S1 — After PR #1 merges: one formatting + branch-reconciliation pass.**
  **Done — executed on this branch after the merge** (branch rebuilt on
  `cbec1f4`, `cargo fmt` re-run, `cargo fmt --check` exits 0). Recipe kept
  for reference:
  PR #1 edited `core/src/iron_core.rs` in its original CRLF form; this branch
  normalized that file to LF, so a naive merge of both will conflict on the
  whole file. Mechanical recipe (works regardless of merge order):
  1. `git checkout main && git pull`
  2. Merge the *other* branch (`claude/fable5-readiness-prompt-lhg4uf` or the
     PR branch, whichever is still unmerged). For every conflicted file under
     `core/`: `git checkout --theirs <file> && git add <file>` if merging the
     PR into this branch's history, or `git checkout --ours` in the reverse
     direction — the rule is **always take the PR side for content**, because
     this branch's core changes are formatting-only and step 3 regenerates
     them.
  3. `cargo fmt && cargo test --workspace --all-features`
  4. Done when: `cargo fmt --check` exits 0 and the test suite passes; commit
     and push.
- **S2 — Install `libdbus-1-dev` in Linux CI jobs.**
  File: `.github/workflows/ci.yml`. In each ubuntu job that compiles the
  workspace with `--all-features` (`lint`, `test` ubuntu leg, `docs`,
  `ffi-surface`), insert after the `Swatinem/rust-cache` step:
  ```yaml
      - run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config
  ```
  Rationale: `btleplug` → `libdbus-sys` needs the C headers; the build panics
  without them (reproduced locally). Harmless if the runner image already has
  them. Verify: CI `Lint` job passes the clippy step (requires H1). Done
  when: no `libdbus-sys` build panic in any Linux job log.
- **S3 — Make `scripts/ffi_surface.sh` fail closed.**
  File: `scripts/ffi_surface.sh` (anchors are post-PR-#1 but the structure is
  identical on main). In the four WARN branches — missing Kotlin snapshot
  (`line ~67`), missing Kotlin bindings (`line ~71`), missing Swift snapshot
  (`line ~90`), missing Swift bindings (`line ~94`) — replace each bare
  `echo "WARN: ..."` with the same echo plus `EXIT_CODE=1`. Keep `--update`
  behavior unchanged. Verify:
  `rm -rf core/target/generated-sources && scripts/ffi_surface.sh; echo $?`
  prints `1`; then regenerate bindings (`cargo build -p scmessenger-core
  --features gen-bindings && cargo run --bin gen_kotlin --features
  gen-bindings`) and `scripts/ffi_surface.sh` exits 0. Done when: both
  behaviors hold.
- **S4 — Contact key-prefix migration (requires PR #1 merged).**
  (Same defect as PR #1 review comment `contacts.rs:155` — this spec
  resolves that thread.)
  File: `core/src/store/contacts.rs`. In `ContactManager::new`, after
  construction and before returning, run a one-time migration:
  `backend.scan_prefix(b"")`; for each `(key, value)` where the key does
  **not** start with `contact:` and `serde_json::from_slice::<Contact>(&value)`
  succeeds **and** the parsed `peer_id`'s bytes equal the key (this
  equality is the disambiguator against other subsystems' records), write the
  value under `contact_key(&peer_id)` and remove the old key. Log count via
  `tracing::info!`. Add test `test_unprefixed_contacts_migrate_on_open`:
  put a serialized `Contact` under its bare `peer_id` key directly on a
  `MemoryStorage` backend, construct `ContactManager`, assert `list()`
  returns it and the bare key is gone. Verify:
  `cargo test -p scmessenger-core contacts`. Done when: new test passes and
  the existing contacts suite stays green.
- **S5 — Stop rendering all-zero safety numbers as real (requires PR #1
  merged).** Also covers PR #1 review comments `MeshRepository.kt:3787`
  and `MeshRepository.swift:2947` (both flag the same all-zeros fallback
  surfacing through `computeSafetyNumber`; fixing it at the source below
  plus the UI error state resolves both — platform-side hex validation per
  the comments' suggestions is optional belt-and-braces).
  File: `core/src/mobile_bridge.rs`, fn `safety_number`
  (`#[uniffi::export]`, currently `unwrap_or_else(|_| "00000 ...")`). Change
  the fallback to return an empty string `""` (UniFFI-stable, no signature
  change). Files
  `android/.../ui/contacts/VerifySafetyNumberScreen.kt` and
  `ios/.../Views/Contacts/VerifySafetyNumberSheet.swift`: where the safety
  number is displayed, treat an empty string as an error state ("Safety
  number unavailable — key data invalid") and disable the mark-as-verified
  action. Rust verify: add unit test asserting `safety_number("not-hex".into(),
  "junk".into()) == ""`; run `cargo test -p scmessenger-core safety_number`.
  Mobile verify: compiles in CI (post-H1). Done when: Rust test passes and
  neither UI can mark verified with an empty number.
- **S6 — Fence `identity_signing_key_for_test` (requires PR #1 merged).**
  File: `core/src/iron_core.rs` (fn at ~line 2383 on the PR branch). Add
  `#[doc(hidden)]` and rename to `test_only_identity_signing_key`, updating
  the three call sites in `core/tests/integration_backup.rs` (~lines 304,
  316, 373).
  Verify: `cargo test --workspace --all-features` green;
  `cargo doc --workspace --no-deps` shows no public docs for it. Done when:
  both hold.
- **S7 — Normalize remaining CRLF Rust sources and pin line endings.**
  Files: `git grep -Il $'\r' -- '*.rs'` (currently ~10 files under
  `cli/src/`). Create `.gitattributes` at repo root with `*.rs text eol=lf`,
  run `git add --renormalize .`, commit. Verify: `git grep -Il $'\r' --
  '*.rs'` empty; `cargo fmt --check` still exits 0; `cargo test --workspace
  --all-features` green. Done when: all three hold.
- **S8 — Reconcile task checklists with post-merge evidence (requires S1).**
  Files: `tasks/T1.4/progress.md`, `tasks/T2.4/progress.md`,
  `tasks/T4.5/progress.md`. On the merged result, re-run
  `cargo test --workspace --all-features` and `cargo clippy --workspace
  --all-features -- -D warnings`; for each checklist line that names one of
  those commands and is still unchecked, check it and append
  `(verified <date>, local run)`. Do NOT check any box requiring hardware,
  Robolectric, XCTest, or CI. Done when: every checked box in those three
  files names a command that was actually run in the session that checked it.
- **S9 — Trigger the manual cross-platform workflow for real evidence
  (requires H1).** Run the `cross-platform-test.yml` workflow_dispatch on
  `main` after S1; collect job logs for Android/iOS/Windows legs. Done when:
  every leg has a log with a real runner name, and failures (if any) are
  filed as new tasks with the failing step's output attached.

### Explicitly out of scope / unverified

- `cargo deny check advisories` — network-blocked here; covered by CI post-H1.
- Robolectric/XCTest infrastructure (unchecked boxes in T1.2/T1.3/T2.4) —
  genuinely missing, acknowledged by PR #1's notes; needs a decision on
  whether to invest before v1.0.0.
- The `cloud/` orchestrator — build tooling, per owner direction not part of
  this assessment.

---

## 7. PR #1 review-comment triage → tasks T1–T18

PR #1 received 26 review threads (24 from cubic-dev-ai, 2 from
chatgpt-codex-connector) that were **not addressed before the merge** — all
26 are still open against `main`. Every thread maps to exactly one item
below (or to an existing S-task where noted). Verdicts: **CONFIRMED** =
re-verified in this session by reading/running the merged code;
**VALID** = mechanism checked in source, effect not executed;
**PLAUSIBLE** = mobile-platform claim consistent with the code, no
Android SDK/Xcode here to prove it.

Recommended order: T1 → T3 → T4 → T2 → S4 → T5 → T6 → T15 → T8 → the rest.
T1–T6 are Rust/CLI and verifiable with `cargo test`; T8–T18 are mobile and
need CI (H1) or a local SDK to prove.

### Core (Rust)

- **T1 (P1, CONFIRMED — codex `iron_core.rs:1278`) — Identity backup
  exports the wrong contact store for mobile users.**
  `build_identity_backup_payload` reads core's internal
  `store::ContactManager` (`self.contact_manager`, iron_core.rs:2585), but
  Android/iOS create contacts through the UniFFI
  `contacts_bridge::ContactManager` (iron_core.rs:1727), a **separate
  `contacts.db`** with a different record shape (`verifiedAt`, tombstones).
  A mobile user who adds contacts and exports gets a backup whose contact
  list is empty/stale; restore silently loses the address book.
  Fix (smallest safe): add `bridge_contacts_json: Option<String>`
  (`#[serde(default)]`) to `IdentityBackupPayload`; on export, serialize
  `contacts_manager().list()`; on import, restore via the bridge with the
  same all-or-nothing discipline as T4. Longer term the two stores should
  be unified (see T2 — same root cause). Verify: integration test — add a
  contact via `contacts_manager()`, export, wipe storage, import, assert
  the bridge contact is back with `verifiedAt` intact.
- **T2 (P1, CONFIRMED — codex `server.rs:1335` + cubic `server.rs:1402`) —
  CLI message-request flow reads/writes the wrong contact store, and the
  key lookup takes the first match only.**
  `cli/src/server.rs` uses `core.contacts_manager()` (UniFFI bridge DB) at
  lines 481, 520, 554, 1335, 1413, while `UiCommand::Send` and every
  `scm contacts` path uses `core.contacts_store_manager()` (CLI store).
  Consequences verified in source: an existing CLI contact's messages still
  show as pending requests (1335 checks the wrong store), and
  `accept_message_request` adds the contact where the send path never looks
  (1413) — the accepted peer still has no key for sending. Additionally the
  accept-path key lookup (`.find(|m| m.sender_id == request_id)
  .and_then(|m| m.sender_public_key_hex)`) fails if the first matching
  inbox record has `None` — use
  `.filter(...).filter_map(|m| m.sender_public_key_hex).last()` per the
  cubic suggestion. Fix: switch all five `server.rs` sites to
  `contacts_store_manager()` + the lookup fix; grep for other
  `contacts_manager()` callers in `cli/` first. Note: any deployment that
  already accepted requests has contacts stranded in the bridge DB — the
  switch should migrate/merge them (or T1's unification supersedes this).
  Verify: `cargo test -p scmessenger-cli` plus a server test that adds a
  CLI contact, receives a message from them, and asserts it is NOT listed
  as a pending request.
- **T3 (P1, CONFIRMED — cubic `iron_core.rs:1360`) — Backup import
  "validation" passes while restore silently drops invalid ratchet
  sessions.** The import probe calls
  `RatchetSessionManager::deserialize_sessions`, which only fails on
  JSON-level errors; per-session conversion failures are skipped
  (`if let Ok(session) = serializable.into_session()` —
  `session_manager.rs:149`), so a backup can "validate" and still lose
  sessions. Fix: make the conversion strict for the import path — either a
  `deserialize_sessions_strict()` used by both the probe and the actual
  restore, or make the existing fn return `Err` on any invalid entry and
  keep a lenient variant only for best-effort startup loads (audit that
  caller before choosing). Verify: unit test — sessions JSON with one
  corrupted hex field must make `import_identity_backup` return
  `CorruptionDetected`, not succeed.
- **T4 (P1, CONFIRMED — cubic `iron_core.rs:1384`) — Import swallows
  contact-persist failures.** `let _ = contact_manager.add(contact);`
  inside `import_identity_backup` breaks the method's all-or-nothing
  contract — import reports success with a partially restored contact
  list. Fix: `contact_manager.add(contact)?;` (the validate-then-commit
  structure means failures here should abort). Verify:
  `cargo test -p scmessenger-core backup`.
- **T5 (P2, CONFIRMED — cubic `iron_core.rs:1278`) — Export masks contact
  storage-read failures.** `self.contact_manager.read().list()
  .unwrap_or_default()` turns a storage error into a "successful" backup
  with zero contacts. Fix: `.list()?`. Verify: same suite as T4.
- **T6 (P2, VALID — cubic `inbox.rs:37`) — `ReceivedMessage` schema change
  can orphan previously persisted inbox records.** The new
  `sender_public_key_hex` field's `#[serde(default)]` helps JSON only;
  bincode is not self-describing, so old bincode-persisted records fail to
  decode and get skipped. Task: first determine exposure — find where
  inbox entries are persisted with bincode vs JSON (`core/src/store/
  inbox.rs` + its storage backend); if bincode reaches disk on any
  platform, add a legacy-struct fallback decode
  (try new, then old-shape struct mapped into the new type with `None`).
  If inbox is JSON/ephemeral-only, close the thread with that evidence.
  Verify: unit test decoding a pre-change serialized record.
- **T7 (P3, VALID — cubic `backup.rs:298`) — Timing-based Argon2id test is
  flaky by construction.** The `>= 5ms` wall-clock assertion will
  eventually flake and can also pass vacuously on slow CI even if the KDF
  regressed to something weak. Fix: replace with a known-answer test —
  fixed passphrase + fixed salt → assert the exact derived key bytes
  (Argon2id 19 MiB, t=2, p=1 is deterministic). Keep the format-tag and
  fallback-chain tests as they are. Verify:
  `cargo test -p scmessenger-core backup` twice; identical results.

### Android (PLAUSIBLE unless noted — reviewed by reading; CI post-H1 or a
local SDK is the proof gate)

- **T8 (P1 — cubic `QrCode.kt:28` + P2 `QrCode.kt:61`) — QR composable:
  stale cache key and 262K JNI calls.** (a) `remember(data)` must be
  `remember(data, size)` or a size change serves the old bitmap. (b) The
  per-pixel `bitmap.setPixel` nested loop on the main thread should be an
  `IntArray` filled in Kotlin + one
  `Bitmap.createBitmap(pixels, width, height, Config.RGB_565)` call.
  File: `android/.../ui/components/QrCode.kt`.
- **T9 (P2 — cubic `VerifySafetyNumberScreen.kt:66`) — Safety number
  memoizes `null` forever if identity initializes after first
  composition.** `remember(contact.publicKey)` caches the pre-identity
  `null`; key the memo on identity readiness too (e.g. collect
  `identityInfo` as state and use it as a second `remember` key).
- **T10 (P2 — cubic `SettingsScreen.kt:165`) — Import dialog reopens with
  the previous passphrase/backup text still populated.** Clear
  `importText`/`importPassphrase` in `onImportIdentity` before
  `showImportDialog = true` (the export handler already does this —
  mirror it).
- **T11 (P2 — cubic `BackupPassphraseValidator.kt:19`) — Min-length check
  counts UTF-16 units, not code points.** `passphrase.codePointCount(0,
  passphrase.length) < MIN_...` per the suggestion; add a test with a
  4-emoji passphrase asserting `TooShort`.
- **T12 (P2 — cubic `WifiAwareTransport.kt:403`, `:465`, `:490`) — Three
  defects in the new loopback proxy, one file.** (a) Bind the
  `ServerSocket` to the same `LOOPBACK_ADDRESS` reported in
  `onDataPathConfirmed` instead of `InetAddress.getLoopbackAddress()`
  (IPv6-loopback devices dial an address nobody listens on). (b)
  `acceptAndPump` joins both pump jobs before closing — when one side
  closes, the other blocks forever in `read()`; close the bridge when
  *either* pump finishes, then cancel the survivor. (c)
  `PeerConnection.send()` is now a hard-coded `false` no-op while
  `sendData`-capable routes still call it — either restore a real write
  path or unregister WiFi Aware from those routes so delivery fails over
  loudly, not silently.
- **T13 (P3 — cubic `MeshApplicationScheduleTest.kt:1`) — Package
  declaration `com.scmessenger.android` doesn't match the
  `.../android/test/` directory.** One-line fix to
  `package com.scmessenger.android.test`.

### iOS (PLAUSIBLE — same proof gate)

- **T14 (P1 — cubic `IdentityBackupSheets.swift:112`) — Import sheet
  dismisses before the user can see success or failure.** The Import
  button calls `viewModel.importIdentityBackup(...)` then `dismiss()`
  unconditionally; the ViewModel's `error`/`successMessage` is never seen.
  Keep the sheet open and render the result (disable the button while
  running), or surface the result in SettingsView after dismissal.
- **T15 (P2 — cubic `VerifySafetyNumberSheet.swift:54` + P3 `:66`) —
  Verification actions swallow errors; wrong fallback copy.**
  (a) `try? viewModel.verifyContact/unverifyContact` — failed state
  changes leave the UI unchanged with no feedback; catch and display.
  (b) The `else` branch shows "Your identity isn't initialized yet" even
  when the actual nil case is contact-not-found; split the two messages.
- **T16 (P2 — cubic `SettingsViewModel.swift:75`) — Backup export/import
  runs synchronous Argon2id (~19 MiB KDF) on the MainActor.** Settings UI
  freezes for the KDF duration. Run the repository call off the main actor
  (`Task.detached` or an async repository API) and publish
  `backupExportResult` back on the main actor.
- **T17 (P2 — cubic `MeshBackgroundServiceTests.swift:58` + P3 same line) —
  Background-service tests race and under-cover.** (a) The tests sleep
  500 ms instead of awaiting the unstructured `Task` spawned inside
  `simulateBackgroundRefresh()/simulateBackgroundProcessing()` — have the
  simulate methods return the `Task` handle and `await` it. (b)
  `simulateBackgroundRefresh()` skips the `quickPeerDiscovery()` step the
  real handler runs — include it so regressions there are caught.

### Cross-references

- cubic `contacts.rs:155` → **S4** (spec already in §6).
- cubic `MeshRepository.kt:3787` + `MeshRepository.swift:2947` → **S5**
  (spec updated in §6 to name both threads).
- **T18 — after each fix lands, resolve the corresponding review thread on
  PR #1** (they stay open on the merged PR otherwise and the next audit
  re-triages them). The thread URLs are in the appendix source; use
  `pull_request_review_write`/`resolve_thread` or the GitHub UI.

---

## Appendix: evidence index

- Actions runs inspected: `28571527109` (CI @ `cd582f8`, 2026-07-02),
  `27523269349` (CI @ `53370fe`, 2026-06-15) — both: all jobs completed in
  1–2 s, `runner_id: 0`, no logs. 18/18 runs on `main` since 2026-06-15 are
  `failure`.
- Local gate runs on `main` @ `cd582f8` and on this branch: §2 table.
- Worktree run at `0a49d32` proving the rc2 fmt claim false: §3.
- PR #1 (`1f52b42`) worktree runs: clippy PASS, full test suite PASS: §4.
- Post-merge (2026-07-02): run `28575087181` on `cbec1f4` (the PR #1 merge
  commit) — all 6 CI jobs done in ≤2 s, `runner_id: 0`, no logs; runners
  still dead (H1 open).
- PR #1 review threads: 26 total (24 cubic-dev-ai, 2 chatgpt-codex-connector),
  all unresolved at merge time; fetched 2026-07-02 via the GitHub API and
  triaged in §7. Store-split claims (T1/T2) re-verified against merged
  source: `contacts_manager()` → `contacts_bridge` (iron_core.rs:1727) vs
  `contacts_store_manager()` → core store (iron_core.rs:2585); lenient
  session skip at `session_manager.rs:149`.
