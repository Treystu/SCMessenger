# SCM-CLOUD burndown session record, 2026-11-14/15

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Lane: cloud build lane (desktop lane asleep). Base: `main` @
`265d5a3c217a18f634577de3f8b2cb3f2aa5feaa` (`git rev-parse 265d5a3c`).
Branch of this record: `lane/cloud-session-2026-11-14`.

## What this session could and could not do - read the limits first

| Capability | State this session | Evidence |
|---|---|---|
| Repo + commits | YES | branch pushes accepted by origin (see the Branch table) |
| GitHub API credential | **NO** | `gh auth status` -> "You are not logged into any GitHub hosts" |
| Open a PR / comment / file to the ledger | **NO** | no write channel exists; nothing below claims one |
| Anonymous GitHub reads | YES, 24 core calls remaining at start | `GET https://api.github.com/rate_limit` -> `core limit 60 remaining 24` |
| Rust compiler | **NO** | `command -v cargo rustc` -> both MISSING |
| Python / node | YES | `command -v python3 node` -> `/usr/local/bin/python3`, `/usr/local/bin/node` |

Consequences, stated once and applied throughout:

1. **No PR exists for any branch below.** They are pushed branches only. No
   comment was posted on #602, #595 or any issue; no ledger row was written.
   Everything tracker-side is listed as PENDING for a session with the channel.
2. **Nothing is compiled.** Every Rust change here is RAIL-UNVERIFIED. The repo
   doctrine that the Windows host is the only authoritative build environment
   applies unchanged; this lane could not even produce the weaker container-signal
   build, because there is no toolchain in the sandbox.
3. **No CI status was read.** No check-run, CircleCI job or required-context
   result is claimed for any branch. The rail is unread, not green.

## Census (read this session)

Read: `GET /repos/Treystu/SCMessenger/issues?state=open&per_page=100&page=1`
and `...&page=2`, 2026-11-15T10:1xZ, anonymous, HTTP 200 both pages. The first
page returned exactly **100 rows**, which is the API page cap and therefore a
ceiling, not a count; the second page was read to settle the total rather than
infer it.

**111 open items = 95 issues + 16 pull requests.** That matches the queue the
tasking stated ("about 95 open issues / 16 open PRs").

Held by the tasking, confirmed present and open in this census, and **not
touched**: PR #599 (rule-8 hold), #712 (parked by oversight), #534, #542, #645,
#540 (parked on the #609/#643 runner decision), #528 and #647 (ungated).

The full item list is in the annex at the end of this file - every row, no
truncation, because a count with a hidden tail is not a census.

## Burned or dispositioned this session

### #651 - `scm test` still fails with "Error: Consent required" - CURED, branch `lane/cloud-651`

Root cause found by reading, not inferred from the report. `cmd_test` granted
consent to `alice` and `bob`, and then built the third identity for the security
stage without one:

- `cli/src/main.rs` (pre-change) `let eve = IronCore::new(); eve.initialize_identity()?;`
  had no `grant_consent()`.
- `core/src/iron_core.rs:949` `initialize_identity` returns
  `IronCoreError::ConsentRequired` when `*self.consent.read() != ConsentState::Granted`,
  and the error's text is exactly "Consent required" (`core/src/lib.rs:58`).

That is the reported symptom precisely: three `[OK]` lines, then a bare
`Error: Consent required`, exit 1 - the #377 shape, still live.

The cure, and why it is shaped this way:

- `cli/src/selftest.rs` (new) holds the probe, and **every** `IronCore` it
  creates comes from the single `consented_core()` constructor, so a new stage
  cannot forget the grant. It runs in memory only, so it is a plain unit of work.
- It lives in the library because `cli/Cargo.toml` sets `[[bin]] test = false`.
  A unit test inside `main.rs` would never be run by `cargo test` - a dead test.
  This is also why `cmd_test` now just prints the probe's stages.
- A failure carries its `Stage`, and the command prints the stage name plus a
  remediation, which is the second half of the issue's acceptance: no stage may
  fail anonymously.
- `cli/tests/selftest.rs` (new) asserts the four stages run in order and report.
  If consent is ever dropped from any core again, `selftest::run()` returns Err
  and that test fails.

Head: `934f0a6990c00843fae0b440fe75270d7155e277`. **RAIL-UNVERIFIED.**

### #561 - compiler and rustdoc warning cleanup - PARTIAL, branch `lane/cloud-561`

The `unused_mut` group is cleared: nine `let mut` bindings in
`cli/tests/integration_peer_identified_gate.rs` (45, 78, 114, 156, 160, 213) and
`cli/tests/integration_mesh_acceptance.rs` (105, 175, 225) hold a
`ConnectionLedger` used only through `&self` methods, verified by reading the
receiver signatures in `cli/src/ledger.rs` (`record_identified_peer`,
`record_topic`, `merge_shared_entries`, `merge_shared_entries_via` all take
`&self`; the `&mut self` methods are `add_bootstrap`, `try_begin_dial` and the
dial-state mutators).

The tenth binding in the first file stays `mut` on purpose: line 184 calls
`add_bootstrap`, which needs `&mut self`.

Not done, deliberately:

- the ten rustdoc intra-doc link warnings are untouched. Seven of the ten files
  they sit in are Rule-8 paths (`privacy/padding.rs`, `routing/global.rs`,
  `transport/swarm.rs`), and a link fix is not verifiable here without rustdoc.
  Doing half of them would split a warning-only cleanup across a review gate for
  no benefit.
- `RUSTDOCFLAGS=-D warnings` (the issue's last suggestion) is not enabled, for
  the same reason: the surface is not clean yet, so the flag would red the Docs
  job immediately.

Head: `3fc8c78fe5bf6acfb1fd65a0482dc62de39c88f1`. **RAIL-UNVERIFIED.**

### #642 - required context that executes no steps still claims an OCI runner - VERIFIED CURED ON MAIN, no branch

No code change made, because the fix is already on main. Read this session in
`.github/workflows/ci.yml:219`: the `test` job's `runs-on` consults the same flag
that gates its steps -
`matrix.os == 'ubuntu-latest' && needs.changes.outputs.rust_relevant == 'true' && ...`
- so a docs-only or Android-only diff stays on hosted `ubuntu-latest` and a
Rust-relevant one still takes the OCI runner. The issue's own four acceptance
items each map onto code that is present:

- check name unchanged: the job's `name` is still `Test (ubuntu-latest)` with a
  static matrix (the comment at `ci.yml:210-214` explains why the matrix must
  stay static);
- fail-closed path unguarded: `Fail closed if change detection failed` has its
  own `if` (`ci.yml:245-249`), not the `rust_relevant` gate;
- other `matrix.os` values unaffected: the expression is guarded on
  `matrix.os == 'ubuntu-latest'`.

Main carries one thing the issue did not ask for, at `ci.yml:228-238`: a
`Refuse an empty required check on the scarce pool` step that turns a routing
regression into a red check instead of a silent empty success. That is strictly
stronger than the proposal.

**What is not claimed:** the issue's acceptance is an observation about run
durations ("completes in under 60s on a GitHub-hosted runner"), and this session
read no Actions run. The code path is cured; the measurement is unread.

### #549 - persist env-derived `bootstrap_nodes` into config.json - PREMISE SUPERSEDED, needs an owner ruling, no branch

Not implemented, because the issue's premise no longer survives contact with the
code, and AGENTS.md is explicit that a task whose premise fails should be
reported rather than implemented.

The issue assumes `SC_BOOTSTRAP_NODES` is a live seed source that merely fails to
persist. Main has removed that source under the #469 doctrine:

- `cli/src/bootstrap.rs:1-12` - "There are NO static bootstrap seeds: no
  environment variable, no build-time option, no compiled-in or config-supplied
  address list. The only seed sources are an invite (`SCI1:` seed ledger) and
  live discovery."
- `cli/src/bootstrap.rs` `test_env_var_is_not_a_seed_source` asserts the env var
  is not read anywhere.
- `cli/src/config.rs:51-67` keeps `bootstrap_nodes` as a **legacy, read-once**
  key that is migrated into the ledger as unproven seed entries and then dropped
  from the file (`cli/src/main.rs:2090-2126`).

So writing the env value back into `config.json` would reintroduce exactly the
transitional mechanism the doctrine retired. The durable answer to the incident
issue #549 describes (a node starting with zero bootstrap candidates) is
ledger-sharing-first discovery plus the invite seed - which is V050-B1/B2, not a
config write-back. **This needs the operator to either close #549 as superseded
or restate the requirement against the ledger; the lane should not choose.**

### #719 - Android CI verification never executes - BLOCKED, needs Actions visibility, no branch

Read the workflow and could not establish the cause from the repository, which is
what the issue itself says about its evidence ("the logs are gone", "the failure
text cannot be recovered"). What this session could and did check:

- the three failing jobs are all `runs-on: ubuntu-latest`
  (`mobile.yml:110`, `:137`, `:273`), so a self-hosted/OCI label mismatch cannot
  explain them;
- the action refs they use (`actions/checkout@v7`, `actions/setup-java@v4`,
  `dtolnay/rust-toolchain@1.99.0`, `Swatinem/rust-cache@v2`) are the **same
  refs the green workflows use** (`ci.yml:31,137,141`, `lint.yml:26,50`), so a
  bad action ref is ruled out as the differentiator;
- `permissions: contents: read` and the concurrency block in `mobile.yml` are
  ordinary and match the rest of the repo;
- billing/spending state is not a candidate either: the base definition has the
  five required contexts reading success on main's head, and a spending-limit
  stop would take those down together with mobile.

That leaves the one place this session cannot read: the run page's start-up
error. **A session with Actions visibility or the repo's `gh` credential must
read one failing run's start-up message** before anyone edits `mobile.yml`. The
issue's fallback option - removing the jobs so the red marks stop implying
coverage that does not exist - is an architecture/CI-policy call and is
ESCALATE-class here, not something this lane should land unilaterally.

## Branch table - the exact heads oversight must read

| Issue | Branch | Head SHA | State | Rail |
|---|---|---|---|---|
| #651 | `lane/cloud-651` | `934f0a6990c00843fae0b440fe75270d7155e277` | cure + regression test, pushed | RAIL-UNVERIFIED (no CI read, no toolchain) |
| #561 | `lane/cloud-561` | `3fc8c78fe5bf6acfb1fd65a0482dc62de39c88f1` | partial cure (unused_mut half), pushed | RAIL-UNVERIFIED (no CI read, no toolchain) |
| (record) | `lane/cloud-session-2026-11-14` | this commit | this document | n/a |

Both cure branches are cut from `265d5a3c` and are independent of each other;
neither contains the other.

## Verification actually performed here

Per AGENTS.md rule 13, each line names what was run:

| Check | Result |
|---|---|
| `python3 scripts/rules_check.py --staged` (both commits) | exit 0, no findings |
| brace/paren/bracket balance of every changed Rust file, strings and comments stripped | balanced (e.g. `main.rs` 920/920 braces, 3496/3496 parens) |
| blocked-emoji scan of every changed file, using the exact ranges in `scripts/rules_check.py` | 0 hits |
| `grep -n "^async fn cmd_test" ...` after splice | the rewritten function is present; no leftover `let eve = IronCore::new()` |
| `git branch --contains` / `git rev-parse` | heads recorded above |

Not performed, and not claimed: any `cargo` command, any test run, any CI or
CircleCI read, any Actions API read, any PR creation.

## Pending for a session that holds the write channel

1. Open PRs for `lane/cloud-651` and `lane/cloud-561`. Neither touches
   `core/src/{crypto,transport,routing,privacy}/`, so neither needs the Rule-8
   gate; both need the standard rail on the exact head above before they are
   READY-TO-MERGE.
2. Post the #642 read (`ci.yml:219`) on #642 and let its owning step close it -
   the code is cured on main and only the run-duration measurement is missing.
3. Put the #549 premise question to the operator (close as superseded by the
   #469 ledger-first doctrine, or restate it against the ledger).
4. Read one failing `mobile.yml` run's start-up error for #719. Until that read
   exists, no Android change on this repository is CI-verified by its own
   workflow - which is also why the parked Android PRs cannot be certified
   either way.
5. Claim the lane on #602 / tracker #595 and mirror this record.

## Next candidates audited for cure-ready scope (not started)

- **#563** (`core/src/transport/observation.rs:112-119`): read. `observations` is
  keyed by `PeerId`, so `confirmation_count` grows per report from one peer; the
  fix is per-observer weight 1 in `recalculate_consensus`. Small and local, but
  it is a Rule-8 path, so it cannot be *done* from here - it needs the
  independent adversarial review. Good first car for a lane with a compiler.
- **#548** (self-recipient poison): read the issue; needs the enqueue guard in
  `cli/src/api.rs` plus a queue-cancel route, and `grep -n cancel cli/src/api.rs
  cli/src/api_axum.rs` reproduced the issue's zero-hit finding. Single-flag
  cure of real value, next on this lane's list.
- **#650** (hard-coded `API_PORT: u16 = 9876`): read; confirmed at
  `docker/entrypoint.sh:76` that a bind override exists via `SCM_HTTP_BIND`, so
  the cure is to make the CLI's client resolve the instance's bound port rather
  than the constant. Needs care threading config, and a second-node test this
  sandbox cannot run.
- **#571** (ledger migration replay + false-empty `contact list`): read; the
  read-side fix is a bounded retry plus a non-zero exit on degraded storage, and
  the migration-marker half needs the store's versioning surface. Larger than it
  looks; not a quick car.

## Annex - full open-item census, all 111 rows

Every open issue and PR as read this session, no truncation. Generated from the
two API pages kept in the session's `tmp/census/` (not committed: scratch).

| Kind | # | Held | Labels | Title |
|---|---|---|---|---|
| ISSUE | 155 |  | bug, ci, tests | Fix UniFFI cdylib debuginfo stripping in Docker test suite (docker-compose.test.yml) |
| PR | 208 |  |  | docs(apple): 4-node parity rollout status, coordination journal & iOS link-local filter (AW4N-STATUS-005) |
| PR | 216 |  | tests, android | fix(android): wire up unreachable features + KeyStore passphrase; wiring gate 32 -> 0 |
| PR | 220 |  | tests, android | fix(android): reachability fixes, split from the blocked passphrase change |
| ISSUE | 365 |  |  | Handoff from OC lane (2026-09-23): PR #361 green-pending one flake rerun; Rule-8 sign-off is the only merge blocker |
| ISSUE | 417 |  | bug, rust, android | Cell handover: relay denies a phone's fresh connections while its dead Wi-Fi sockets hold the per-peer cap (RCA); cap raised under an A2 waiver, retroactive review required |
| ISSUE | 418 |  | enhancement, rust, android | Dynamic per-device/situation connection-cap throttling in Advanced Settings |
| ISSUE | 419 |  | bug, rust, android | Android: native libraries fail the 16 KB RELRO alignment check (warning dialog on Android 17) |
| ISSUE | 448 |  |  | NAT Simulation Tests job is red for three unrelated reasons, and has no PR trigger so it rots silently |
| ISSUE | 449 |  |  | Latent toolchain-ordering defect in docker/Dockerfile.rust-test and Dockerfile.wasm-test (same class as the #447 android fix) |
| ISSUE | 452 |  |  | Consolidation tracking: 69 open PRs — merge, fix, or close every one |
| ISSUE | 455 |  |  | SCMessenger 1.0.0 completion program (Jev-driven) — master tracker |
| ISSUE | 456 |  |  | Phase 2d: re-judge 164 critical-context functions with expanded context |
| ISSUE | 457 |  |  | Apply Jev implementation-pattern prescriptions (102 at ≥0.80 confidence) |
| ISSUE | 458 |  |  | Security deep-dive A: crypto / identity / store (117 functions) |
| ISSUE | 459 |  |  | Security deep-dive B: platform bridges / CLI / WASM (71 functions) |
| ISSUE | 460 |  |  | Security deep-dive C: transport / relay (54 functions) |
| ISSUE | 461 |  |  | Security deep-dive D: generated bindings + integration tests (158 functions) |
| ISSUE | 462 |  |  | Test-gap closure A: core transport (222 high-blast-radius untested functions) |
| ISSUE | 463 |  |  | Test-gap closure B: crypto / store / core (232 functions) |
| ISSUE | 464 |  |  | Test-gap closure C: platform — Android / iOS / WASM / CLI (408 functions) |
| ISSUE | 465 |  |  | Dead-code removal: 23 dead/experimental functions |
| ISSUE | 466 |  |  | Concurrency-risk triage A: core (660 functions at risk ≥ 0.5) |
| ISSUE | 467 |  |  | Concurrency-risk triage B: platform (694 functions at risk ≥ 0.5) |
| ISSUE | 468 |  |  | Lifecycle: 4 confirmed resource leaks + RAII sweep |
| ISSUE | 469 |  |  | 1.0.0 blocker: bootstrap/rendezvous story for stock builds |
| ISSUE | 477 |  | bug, tests | Flaky test: iron_core_fast_backup_tracks_bridge_contact_removals (StorageError on restored iOS contacts store) |
| ISSUE | 478 |  |  | 3-node mesh validation on 0.5.0: WiFi pair + always-on cloud node via ledger propagation |
| ISSUE | 490 |  |  | Route swarm Identify/gateway routing writes through routing_peer_seen (single entry point canon) |
| ISSUE | 491 |  |  | CLI: macOS network-change notifications for discovery interface monitor (#469 follow-up) |
| ISSUE | 492 |  |  | Android: MeshRepository constructs HistoryManager/ContactManager in init, possibly on the main thread (#413 follow-up) |
| ISSUE | 493 |  |  | Rule-8 follow-up: #479 nits (routing path canonicalization, neighborhood ranking) |
| ISSUE | 494 |  |  | Rule-8 follow-up: #485 nits (migration loop batch handling, record_rejected bounds) |
| ISSUE | 495 |  |  | Rule-8 follow-up: #413 nits (contact_open_gate pruning, wasm sweep documentation) |
| ISSUE | 497 |  |  | Rule-8 follow-up: #484 discovery scheduler nits |
| ISSUE | 499 |  |  | Rule-8 follow-up: #489 conn-cap v2 nits and known limitations |
| ISSUE | 517 |  |  | Root-cause: register_identity request-response never resolving (PR #512 mitigation) |
| ISSUE | 525 |  |  | Review 76 production .expect() sites (panic paths) -- convert to error propagation or justify |
| PR | 528 | HELD | dependencies, github_actions, ci, tests | chore(deps): bump dtolnay/rust-toolchain from 1.99.0 to 1.120.0 |
| PR | 529 |  | dependencies, github_actions, ci | chore(deps): bump actions/setup-java from 4 to 6 |
| PR | 534 | HELD | dependencies, java, android | chore(deps): bump com.google.zxing:core from 3.5.2 to 3.5.4 in /android |
| PR | 538 |  | dependencies, rust, tests | chore(deps): bump x25519-dalek from 2.0.1 to 3.0.0 |
| PR | 540 | HELD | dependencies, java, android | chore(deps): bump kotlin_version from 1.9.20 to 2.4.20 in /android |
| PR | 542 | HELD | dependencies, java, android | chore(deps): bump androidx.navigation:navigation-compose from 2.7.5 to 2.10.2 in /android |
| ISSUE | 544 |  | rust, tests | Migrate bincode 1.x → 3.x (163 call sites + wire-format break) |
| ISSUE | 545 |  | rust, tests, crypto | Upgrade rand 0.8 → 0.10 (prerequisite for x25519-dalek 3.0) |
| ISSUE | 548 |  | bug, rust, cli | fix(cli): add queue-cancel route and self-recipient guard for poison outbox entries |
| ISSUE | 549 |  | bug, rust, cli | fix(cli): persist env-derived bootstrap_nodes into config.json so Kademlia starts with known peers |
| ISSUE | 552 |  |  | POST /api/send accepts a message but GET /api/history does not surface it; bare /send is a dead 404 while /api/send is live |
| ISSUE | 553 |  |  | GET /api/peers returns stored peer records with no live connection fields even though logs show Gossipsub/identify/relay-custody traffic |
| ISSUE | 555 |  |  | Routing: authenticate announced hints + hint-collision hardening (follow-up to #554 Rule-8 nits) |
| ISSUE | 561 |  |  | Clean up compiler and rustdoc warnings surfaced by CI (unused_mut in cli tests, 11 rustdoc link warnings) |
| ISSUE | 563 |  |  | Transport: cap per-observer weight in AddressObserver confirmations (one peer can outvote others) |
| ISSUE | 567 |  |  | Wind-down tracking: untracked in-flight work from 2026-10 orchestration session |
| ISSUE | 571 |  |  | CLI replays ledger migration on every invocation; contact list reports false 'No contacts yet.' with exit 0 when DB lock is held by the running node |
| ISSUE | 573 |  |  | [v0.5.1] P0 BLOCKING: #521 fix A - never evict a peer's last path under the total bound |
| ISSUE | 574 |  |  | [v0.5.1] #521 fix B - replay/freshness check before a contact-signed envelope earns the traffic rank bit |
| ISSUE | 575 |  |  | [v0.5.1] #521 fix C - keep one pending-inbound slot per /24 when headroom is 0 |
| ISSUE | 576 |  |  | [v0.5.1] #521 follow-ups (LOW) |
| ISSUE | 577 |  |  | [v0.5.1] P0: Independent Rule-8 adversarial review of #521 on its final head |
| ISSUE | 578 |  |  | [v0.5.1] Land #521 (MT-05) and close superseded #489, #372, #364 |
| ISSUE | 580 |  |  | [v0.5.1] Verify status of queue cars MT-04 and MT-08..MT-12 |
| ISSUE | 581 |  |  | [v0.5.1] Version bump to 0.5.1 (supersedes #471) |
| ISSUE | 582 |  |  | [v0.5.1] Tag and release v0.5.1 |
| ISSUE | 583 |  |  | [v0.5.1] WASM outbox sweep (native-only sweep from #413) |
| ISSUE | 584 |  |  | [v0.5.1] A-05 iOS receipt unification |
| ISSUE | 585 |  |  | [v0.5.1] Core block-gate identifier fix |
| ISSUE | 586 |  |  | [v0.5.1] Crypto coder dispatch wave |
| ISSUE | 587 |  |  | [v0.5.1] Farm sim Phase 2/3 comprehensive testing |
| ISSUE | 588 |  |  | [v0.5.1] GitHub v1.0.0 macOS runner setup |
| ISSUE | 589 |  |  | [v0.5.1] U2 topic constants centralization |
| ISSUE | 590 |  |  | [v0.5.1] D1 desktop bridge UniFFI verification (partly done) |
| ISSUE | 591 |  |  | [v0.5.1] PQC-09 hybrid onion investigation (partly done) |
| ISSUE | 592 |  |  | [v0.5.1] Android relay inbound evidence on cellular (operator device) |
| ISSUE | 593 |  |  | [v0.5.1] Android Stop/Start lifecycle Pixel acceptance run (operator device) |
| ISSUE | 594 |  |  | [v0.5.1] P0-8 BK-01 backup purge (operator-gated, do not run) |
| ISSUE | 595 |  |  | [v0.5.1] Burndown tracker (Freebuff lane) |
| PR | 599 | HELD | rust | [RULE-8] fix(transport): close #521 review blockers (#573, #574, #575) |
| ISSUE | 600 |  | bug, android | Android: mesh stop/start button intermittently does not work — full RCA requested |
| ISSUE | 602 |  | v0.5.1 | [v0.5.1] Land all local work to GitHub — commit and push safely (operator-approved) |
| ISSUE | 603 |  |  | [release] Ship a known-working release: three-node gate is the trigger |
| ISSUE | 607 |  |  | [ci] Switch GitHub-hosted Linux jobs to Ubicloud runners (ubicloud-standard-2) |
| PR | 609 |  | ci | [ci] Switch the five core jobs to Ubicloud standard-2 |
| ISSUE | 619 |  | bug, android | fix(android): notifications-disabled setting is ignored — notifications still post to the notification bar |
| ISSUE | 620 |  | ci | Launch readiness: 24/7 permanent-roles lanes on the Windows box (Freebuff) |
| ISSUE | 623 |  | transport, P0 | [Transport] Implement HTTP-compliant transport (proxy-aware WSS + HTTPS mailbox) |
| ISSUE | 624 |  | transport, ops | [Transport] Nodes listen promiscuously; detect protocol on inbound and dispatch |
| ISSUE | 625 |  | transport, P0 | [Transport] Inbound protocol detection and dispatch on the listen path |
| PR | 633 |  | dependencies, rust | chore(deps): bump rand from 0.8.7 to 0.10.2 |
| ISSUE | 642 |  |  | [ci] A required context that executes no steps still claims an OCI runner (docs-only / android-only refreshes) |
| PR | 643 |  | ci, tests | ci: reserve the two ARM OCI runners for car waves, use hosted for the rest |
| PR | 645 | HELD | dependencies, java, android | build(deps): bump com.google.android.gms:play-services-base from 18.2.0 to 18.11.1 in /android |
| PR | 647 | HELD | dependencies, java, android | build(deps): bump gradle-wrapper from 8.13 to 9.9.0 in /android |
| ISSUE | 650 |  |  | CLI control API port is hard-coded to 9876: a second node on the same host loses its API and its CLI commands are silently served by the first node |
| ISSUE | 651 |  |  | Regression: `scm test` still fails with "Error: Consent required" (closed #377 reproduces on current main) |
| ISSUE | 665 |  | enhancement, docs | [Audit] Mesh mechanism audit program — evaluation-first burndown (master tracker) |
| ISSUE | 675 |  | enhancement, rust, tests, transport | [Audit] Discovery scheduling: fixed cadences vs yield-adaptive probing |
| ISSUE | 679 |  |  | Android: unbounded retry cycle on an undeliverable message grew the mesh log to 4.19 GB |
| ISSUE | 682 |  |  | [Audit] Circuit hop count, not relay choice, costs delivery probability (owner decision: depth vs delivery) |
| ISSUE | 683 |  |  | [Bug] privacy/circuit.rs: diversity loop has no progress guarantee, and select_random_hops is documented weighted but is uniform |
| ISSUE | 684 |  |  | [cure] #679: bound the mesh log and quarantine structurally impossible dial candidates |
| ISSUE | 686 |  |  | [Cure] routing confidences are miscalibrated and the layer cascade loses to a cross-layer comparison (#668) |
| ISSUE | 688 |  |  | [Cure] drift sync must not fail a session on decode, and 4n+8 sizing is not safe at heavy divergence (#671) |
| ISSUE | 689 |  |  | [Cure] remove the energy multiplier from gateway ranking; keep energy as a bounded post-ranking policy (#669) |
| ISSUE | 690 |  |  | [Cure] discovery scheduling: adapt on evidence of churn and price peers-never-seen in the acceptance (#675) |
| ISSUE | 691 |  |  | [Cure] negative evidence expires 6600s before the routes it condemns; 500ms budget cannot contain 760ms of phases (#672) |
| ISSUE | 692 |  |  | [Cure] reputation: price effort, not claims -- 3 self-reported deliveries currently buy the trusted band (#676) |
| ISSUE | 693 |  |  | [Cure] cover traffic defaults are strictly dominated: 23.5x bytes for a perfect fingerprint at idle (#673) |
| ISSUE | 704 |  |  | [Audit] Pass E Jev audit: identity 0.44, routing feed 0.61, instruction 0.11 - all three below bar |
| PR | 712 | HELD | dependencies, github_actions | chore(deps): bump github/gh-aw from 0.87.3 to 1.2.2 |
| ISSUE | 719 |  |  | Android CI verification never executes: mobile.yml Android jobs fail 1-2s after start on every commit, so no Android change is CI-verified |

Held rows above are the eight the tasking closed to this lane. They are
listed so the census is complete, not because they were worked.
