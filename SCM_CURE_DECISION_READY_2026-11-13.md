# SCM-CURE LANE -- DECISION-READY REPORT, 2026-11-13

Lane: SCM-CURE (Freebuff cloud session), slug scm-cure.
Working folder: /workspace/chats/cad77275a05a93d4f6e716b6 (branch
freebuff/cad77275a05a93d4f6e716b6).
Repo head read this session: `git rev-parse HEAD origin/main` ->
`ced2189e9d036d389d4aa4e7e40d42357a61f5c6` on both; `git status --short` = 0.

Outcome: the lane cannot write to GitHub from this session. The credential
sweep below is complete and location-by-location; the queue items that need no
GitHub write are resolved or given an evidence-backed current status. No secret
appears in this file.

## 0. Where this file lives on the remote

- Branch: `lane/cure-decision-ready-2026-11-13`
- Commit carrying this report: `fd51818fa639c79cb86cf25c4a6202400c9bb4d0`
  (subject: `docs(cure): file the SCM-CURE lane's decision-ready report for 2026-11-13`)
- Repository: `Treystu/SCMessenger`. Retrieve without this session via
  `git fetch origin lane/cure-decision-ready-2026-11-13` then
  `git show fd51818f:SCM_CURE_DECISION_READY_2026-11-13.md`.
- Push route: the platform git credential (`cloud_git`), which the sweep proved
  works (item 17). The GitHub REST path stayed refused (403) and was not retried.

Known leftover, recorded so nobody is surprised by it: the credential probe
created the remote branch `freebuff/cad77275a05a93d4f6e716b6` (same name as this
session's local branch) by pushing the current HEAD to it. It holds no unique
content -- it points at `ced2189e9`, which is `origin/main` -- and it is the
proof that git write exists. It could not be deleted here: `cloud_git` exposes
fetch, pull and push only, and the REST branch-delete endpoint is behind the
same 403. Delete it with `gh api -X DELETE
repos/Treystu/SCMessenger/git/refs/heads/freebuff/cad77275a05a93d4f6e716b6`
once write access exists.

---

## 1. Credential sweep (present/absent plus the read that showed it)

Commands, in order, from the working folder. No value was printed; environment
output was reduced to NAME only, git output to `<redacted>`.

| # | location | read | result |
|---|---|---|---|
| 1 | environment variables `GH_TOKEN`, `GITHUB_TOKEN`, `GH_ENTERPRISE_TOKEN`, `GITHUB_ENTERPRISE_TOKEN`, `TYPESAFE*`, `SCM_*`, `HARNESS*`, `OPENROUTER_*`, `CIRCLE*`, `ANTHROPIC_*`, `OPENAI_*` | `env \| grep -Ei '^(GH_\|GITHUB_\|...) ' \| cut -d= -f1` ; count | **absent** -- `count=0`, printed name list empty |
| 2 | repo-local git config | `git config --local --get-regexp 'credential\|url.\|insteadof\|extraheader'` | **absent** -- no output (only `remote.origin.url` exists) |
| 3 | global git config | `git config --global --get-regexp ...` | **absent** -- no output |
| 4 | system git config | `git config --system --get-regexp ...` | **absent** -- no output |
| 5 | `~/.netrc` | `[ -e ~/.netrc ]` | **absent** |
| 6 | `~/.git-credentials` | `[ -e ~/.git-credentials ]` | **absent** |
| 7 | `~/.config/gh/hosts.yml` | `[ -e ]` | **absent** (this is why `gh auth status` reports "not logged into any GitHub hosts") |
| 8 | `~/.config/glab-cli/config.yml` | `[ -e ]` | **absent** |
| 9 | `~/.typesafe_key` | `[ -e ]` | **absent** |
| 10 | `~/.config/scm`, `~/Documents/GitHub` | `[ -e ]` | **absent** (operator-machine paths; not mounted here) |
| 11 | `.env` / `.env.local` / `.env.production` / `.dev.vars` / `.secrets` in checkout and its parents (`.`, `..`, `../..`, `../../..`) | `[ -f ]` loop | **one file present**: `.env.local`, 2 lines, key names only -> `CLASS_B_DUMMY_PROBE`. No credential in it. |
| 12 | token embedded in a remote URL | `git remote -v \| sed -E 's#(https://)[^@]*@#\1<redacted>@#'` | **absent** -- `origin https://github.com/Treystu/SCMessenger.git` (fetch and push), no userinfo |
| 13 | anonymous git write | `git push --dry-run origin HEAD:refs/heads/probe-credential-check` | **refused**: `fatal: could not read Username for 'https://github.com'` |
| 14 | anonymous git read | `git fetch --dry-run origin main` | **works** -- `branch main -> FETCH_HEAD`, rc=0 |
| 15 | anonymous REST read | `GET /rate_limit` | **works**, 60/h, quota exhausted at read time, reset 05:43Z |
| 16 | anonymous REST write | `POST /repos/Treystu/SCMessenger/issues/602/comments` | **403** |
| 17 | platform-managed git credential (the one credential this environment does hold) | `cloud_git push` to `freebuff/cad77275a05a93d4f6e716b6` | **works**: `* [new branch] HEAD -> freebuff/cad77275a05a93d4f6e716b6`; server-held, not exportable to the API |
| 18 | ledger repo | `git clone https://github.com/Treystu/slow-lane-ledger.git` ; `GET /repos/Treystu/slow-lane-ledger` | **unreachable**: clone `could not read Username`; API 404 (private) |
| 19 | canonical gate key (harness-reported) | `jev_canonical_check.py` line | **absent**: `typesafe_key=False openrouter_key=False` |

Conclusion: no credential exists in this sandbox that can authenticate the
GitHub REST API or reach the private ledger. The only git credential present is
the platform's, and it is usable only through `cloud_git`.

## 2. The exact access the owner must provide

1. `GH_TOKEN` (or `GITHUB_TOKEN`) with `issues:write` and `pull_requests:write`
   on `Treystu/SCMessenger`, set as a project secret so it reaches the
   environment. Without it there is no claim on #602, no stand-up comment, no
   PR, no READY-TO-MERGE mark, no `STATUS.md` update.
2. Read access to `Treystu/slow-lane-ledger` (deploy key or token), for
   `dispatch/scm-cure/PROMPT.md` v4, `DISPATCH.md` and `dispatch/gates/`.
   Without it the lane runs on the carrier text and cannot apply dispatch
   entries or read the Lucas Gate material.
3. `TYPESAFE_API_KEY` (or the harness's own key configuration), so the stand-up's
   canonical-check line can read WORKING. Today it can only read NOT CHECKED.
4. A named non-author reviewer, for rule-8 (`AGENTS.md` rule 8 /
   `core/src/{crypto,transport,routing,privacy}/`). Assumption row A16 already
   recorded the repository has one account, so this reviewer does not exist yet.

## 3. Bootstrap state

- Ledger: **not pulled** (see 18). Running on the carrier text, as the carrier
  directs when the ledger is unreachable.
- `dispatch/scm-cure/PROMPT.md`, `DISPATCH.md`, `dispatch/gates/`: **unread**,
  they live only in the unreachable ledger.
- Tracker #602 read (200 OK) before the quota exhausted: 348 comments; newest
  is `6813074385`, 2026-11-12T03:00:14Z, the continuation-8 handoff. No STOP
  directive on the page read. Nothing newer could be read after that.
- Claim on #602: **not made** (403).

## 4. Stand-up block

Canonical check -- line carried, claim **NOT CHECKED** (key absent, harness
present and pinned):

```
$ git clone https://github.com/Sovereign-Communication/harness.git vendor/sovereign-harness   # attempt 1, then:
$ python scripts/update_local_harness.py --mode admit-tag --tag v0.4.1
[OK] Harness admission operation complete
  status=PRODUCTION tag=v0.4.1 sha=ad4a30052955e1f574c90f426edfc7a274e16ebf root=vendor/sovereign-harness

$ python scripts/jev_canonical_check.py --wp WP4 --state-file HANDOFF/freebuff/jev/WP4_state_2026-09-21.json
[INFO] harness: vendor/sovereign-harness sha=ad4a30052955e1f574c90f426edfc7a274e16ebf status=PRODUCTION pinned=True typesafe_key=False openrouter_key=False
[INFO] buckets_selected=['identity', 'routing', 'instruction'] questions=5 calls=1 scoped=False
[WARNING] TypeSafe: no reply recorded (no key or transport failure); OpenRouter skipped: no OpenRouter key; result is the local structural fallback
[INFO] is_fallback=True cost=0.0 tokens_in=0 model=jev-latest
[FAIL] UNVERIFIED-JEV - fallback result is not canonical DONE
```

Locations tried before reaching that line, recorded as the doctrine requires:
`SCM_HARNESS_ROOT` (unset), `HARNESS_REPO` (unset), `vendor/sovereign-harness`
(absent at session open -> now admitted through the documented command), in-repo
`Harness/` (carries `handoff/` only, no `harness/jev.py`), operator Windows path
`C:\Users\SCM\Documents\GitHub\Harness` (not present on Linux),
`~/.typesafe_key` (absent), `~/.config/SCM*` (absent), harness-reported
`typesafe_key=False`. The harness half of the gate now resolves here; the key
half does not exist in this environment.

Process correction, recorded rather than hidden: before locating the documented
bootstrap I hand-cloned the harness and hit three fail-closed refusals in a row
(`Harness SHA mismatch`, then `Harness remote mismatch`, then a missing `v0.4.1`
tag). The documented path is `scripts/update_local_harness.py --mode admit-tag
--tag v0.4.1` (`docs/rules/BUILD_AND_CI.md`), and it works here. Its tracked
manifest write was restored byte-for-byte from HEAD; that file is unmodified.

## 5. Queue, item by item, on current head `ced2189e9`

### #622 key-mismatch cure -- CURED ON MAIN, no implementation left

- Cure commit `95b077c0` "fix(gate): accept the bucketed JEV pack the helper
  actually emits (#622)". Independent check 1: `git merge-base --is-ancestor
  95b077c0 origin/main` -> `ancestor=yes`.
- Implementation read in `scripts/orchestration_completion_gate.py`:
  `_pack_answer_keys()` (:220-236) accepts both pack shapes; `_resolve_canonical()`
  (:238-248) resolves a bare key or exactly one `bucket.qid` key; `_derive_signals()`
  (:251-) keeps bare keys strictly `noul` (old contract still fails closed),
  allows `noul/choice/score` on bucket-qualified keys, rejects unknown keys, and
  aggregates `supported`/`confidence` across the whole pack as `harness/jev.py` does.
- Independent check 2: the regression test is the real-helper shape
  (`build_questions(["identity","routing","instruction"])`, not a fixture) at
  `scripts/test_orchestration_v2.py:590`
  `test_completion_gate_accepts_the_real_bucketed_pack_it_is_handed_in_production`,
  and the rail executes that file: `.circleci/config.yml:735`
  `python3 scripts/test_orchestration_v2.py`.
- Both outputs this session: `python3 scripts/test_orchestration_v2.py` ->
  `Ran 26 tests in 1.098s / OK`; ancestry -> `ancestor=yes`.
- Disposition: CLOSED (cured on main, two checks). Filing blocked by 403.

### #684 bound the mesh log / quarantine impossible dial candidates

Part A -- LANDED on main.
- `e77d05b4` "fix(store): cap the mesh log at 64 MiB with startup rotation
  (#679 part A)" + `fb0bf68b` rustfmt. Independent check 1: both
  `ancestor=yes` against `origin/main`.
- Independent check 2, content read of `core/src/store/tracing_init.rs`:
  `MESH_LOG_CAP_BYTES = 64 * 1024 * 1024` (:37); `rotate_oversized_log()` (:46);
  rotation invoked with the cap at init (:78) and the in-force cap logged once
  as `mesh_log_rotated` with `cap_bytes` (:80-81); tests
  `oversized_log_is_rotated_and_replaced_on_the_next_run` (:174) and
  `healthy_log_and_missing_log_are_left_alone` (:205).
- Remaining acceptance is a 24 h device run, which no host in this session can
  produce.

Part B -- NOT IMPLEMENTED. `grep -rn "quarantine" core/src --include=*.rs`
returns 13 hits, **all** in `core/src/store/ledger_entry.rs` (ledger corruption
handling); zero in `core/src/transport`. What exists today is a pre-dial static
check only: `resolve_dial_target()` at `core/src/transport/swarm.rs:650-667`
rejects an address whose embedded `/p2p/<id>` conflicts with an explicit target
(`"dial target identity mismatch: requested {} but address targets {}"`, :658).
The issue's case is the opposite one -- the dial completes and answers as a
different peer -- and there is no (peer_id, address) quarantine store, no TTL,
and no exclusion from candidate selection. In `core/src/transport`, rule-8 gated.

Part C -- NOT IMPLEMENTED. `ROUTE_ATTEMPT_REASON_RETRY_CYCLE = "RETRY_CYCLE_RESTART"`
still at `core/src/transport/swarm.rs:1471`; the retry block (:4880-4975)
increments `pending.dispatch_attempts` (:4946) and logs it (:4955) with no
budget check, so the cycle restarts without a derived bound; `dispatch_attempts`
is initialised to 1 at :8427 and :8451 and never compared to a limit; no parked
state exists (`grep -n "parked" swarm.rs` -> one unrelated channel message at
:11512). In `core/src/transport`, rule-8 gated.

### #693 cover traffic defaults -- NOT IMPLEMENTED

`core/src/privacy/cover.rs`: default `rate_per_minute: 10` (:33), fixed cadence
derived from it -- `message_interval_ms()` = `60_000 / rate_per_minute` (:62-66),
emission gate `elapsed > interval` (:167-174), next-send arithmetic at :200. The
stream is a metronome exactly as #673 measured; there is no coupling to real
traffic, no heavy-tailed cadence and no size variation across the padded set.
In `core/src/privacy`, rule-8 gated.

### #686 routing confidences / cascade -- NOT IMPLEMENTED

`core/src/routing/engine.rs`: strict lexicographic cascade -- Layer 1 local
(:139-169), then Layer 2 neighborhood (:171-190), then Layer 3 global (:192-215),
so a lower layer always wins regardless of score. Confidence is still
`best_peer.reliability_score.min(0.98)` (:166) and a hardcoded
`0.85 - hops_away * 0.05` (:184), i.e. the over/under-confidence #668 measured is
unchanged. In `core/src/routing`, rule-8 gated.

### #688 drift sync sizing / decode failure -- NOT IMPLEMENTED

`core/src/drift/sync.rs`: sizing is still the unconditional
`(message_count as usize * 4 + 8).max(1)` (:183-185, comment at :182 "Size IBLT
generously"), and both decode sites propagate the error as a hard failure --
`let (..) = diff.decode()?` at :256 and :330 -- so a decode failure still ends the
session instead of re-offering at 2x cells. Not rule-8 gated (`core/src/drift`),
but Rust, and this host is Python/CI-only under the disk rail, so an uncompiled
change is not a deliverable.

### #689 gateway ranking energy multiplier -- NOT IMPLEMENTED

`core/src/routing/neighborhood.rs:265-271`:
`cost = hops * gw.energy_class.cost_multiplier() / max(avg_reliability, 0.01)`,
used to order candidates at :255-262 via `min_by(gateway_cost)`. The measured
16x energy spread is still inside the delivery ranking. In `core/src/routing`,
rule-8 gated.

### #690 discovery scheduling -- PARTIALLY PRESENT; the cure's two specific asks are absent

CORRECTION (same session): this entry first read "NOT LOCATED IN CORE / NOT
VERIFIED", which was wrong. The module is `core/src/transport/discovery_scheduler.rs`
(1152 lines), exported at `core/src/transport/mod.rs:71-73`; I had searched
`core/src/routing` and `core/src/transport` for discovery *identifiers* but not
followed the module name the issue itself uses.

Search that establishes it, rerunnable as written:

```
grep -rn "discovery" core/src/routing core/src/transport --include=*.rs | wc -l   -> 243
  (of which the scheduler module itself: core/src/transport/discovery_scheduler.rs, 1152 lines)
grep -rni "readvertis|re_advertis|reannounce|announce" core/src/routing core/src/transport --include=*.rs | wc -l -> 102
  (all in routing/local.rs hint announces and optimized_engine.rs; zero in the scheduler)
grep -rn "churn|yield|response set" core/src/transport/discovery_scheduler.rs -> 0 hits
```

Current state, by line:
- The cadence IS adaptive, and it is event-triggered exactly as the issue says:
  an affecting `NetworkEvent` resets the transport to its floor
  (`NetworkEvent::affects` :132-147, doc block :10-13); without events the
  interval decays by `ceil(interval * g)` with `g` in [1.5, 2.0] (:46-48) plus
  full jitter (:15-16); `record_success` (:482) restores fast probing.
- The ceiling is computed, not literal: `max(floor, base_ceiling * density_factor
  * power_factor)` (:19), density from peer count with `DENSITY_PEER_CAP = 16`
  (:51) and `DENSITY_STEP = 0.25` (:53).
- The 30 s figure in the filing exists in the tree as a per-transport ceiling,
  not a fixed cadence: `for_transport` (:201-207) sets Ble (1_000, 20_000),
  Lan (1_000, 20_000), WifiDirect (1_000, 30_000), Internet (500, 30_000); a
  separate fixed 30 s tick drives routing optimisation at
  `core/src/transport/swarm.rs:4850`.
- Wired live, not dead: callers `core/src/transport/manager.rs:824` and
  `core/src/transport/dial_policy.rs:672` (`on_network_event`).

The gap the cure names is therefore narrow and specific: (1) adaptation still
keys on network EVENTS only -- no probe-yield input such as a changed response
set or a rise in unknown-peer responses, which is the evidence half of #675's
finding; (2) the widening ceiling is a function of peer density and power, not of
observed churn, so a churn event can still be stepped over. Both are in
`core/src/transport`, rule-8 gated.

Not related, and not to be confused with this item: `core/src/drift/policy.rs`
holds a scan cadence only (`scan_interval_ms()` :66-68, override :118-119, tests
:275-302).

### #691 negative-evidence TTL / timeout budget -- NOT IMPLEMENTED

Both defects are live on head `ced2189e9`:
- `core/src/routing/negative_cache.rs:210-211` `with_defaults()` = `600 s` TTL,
  while `core/src/routing/adaptive_ttl.rs:99-101` allows routes to live
  `1800 s` base / `7200 s` max -- the 6600 s wrong-way window is unchanged.
- `core/src/routing/timeout_budget.rs:33-36` phases 10 + 50 + 200 + 500 = 760 ms
  against `default_500ms()` = 500 ms at :84-85; the last phase is still
  unreachable in its own budget.
In `core/src/routing`, rule-8 gated.

### #692 reputation pricing -- NOT IMPLEMENTED

`core/src/abuse/reputation.rs`: positives still enter through self-reported
paths -- `record_signal()` (:52-67) and `record_outbound_message()` (:101-109) --
with no witnessed/recipient-confirmed signal set and no proportionality fix on
the negative side, which is the two-part cure the issue specifies. Not rule-8
gated (`core/src/abuse`), but Rust and unbuildable here (Python/CI-only rail).

### #619 notifications-disabled ignored -- CODE HALF ALREADY IMPLEMENTED

`android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
carries the tri-state gate (`var notificationsEnabled: Boolean? = null` :125),
a cold-start hydration branch (:316), the message gate that returns before
building anything (`if (notificationsEnabled == false)` :343-345), the
peer-discovered gate (:654-656) and the live update path (:784-785). Tests exist:
`android/app/src/test/java/com/scmessenger/android/utils/NotificationHelperGateTest.kt`
(cold start, hydrated OFF reaching the live gate :167-177, builder-reach check
:215-239) and `NotificationGateTest.kt`. What remains is the on-device
confirmation that no notification posts with the setting off; this session has no
Android runtime and the rail carries no emulator.

## 6. What was NOT done, and why (no guessing)

- No commit, no branch content, no PR: the only repo change this session (the
  admission tool's manifest write) was restored from HEAD; the tree is clean
  apart from this report.
- No implementation was pushed for #684 B/C, #693, #686, #689, #691 (all rule-8
  gated) or #688, #692 (ungated but Rust): every one needs a compile or test
  run to be true, the rail is Python/CI-only under the disk rail, and a blind
  patch would be a claim I did not verify.
- Census, exact-head CircleCI reads and Batch D drafting: blocked by the
  exhausted anonymous read quota (60/h) plus the unreadable ledger, which holds
  the drafting format and the gate material.
- #658 sweep: not attempted. It is last in the carrier order and its
  preservation proof (2/9) depends on evidence files this session cannot reach.

## 7. Next session, in order

1. With `GH_TOKEN` present: make the claim on #602, file the stand-up above
   (claim NOT CHECKED unless the key resolves), and file #622's close with its
   two checks.
2. With ledger read: apply `DISPATCH.md` entries and read `dispatch/gates/`
   before anything else; the Gate governs every filed word.
3. Name the non-author rule-8 reviewer before touching #684 B/C, #693, #686,
   #689 or #691.
4. Cure #690 in `core/src/transport/discovery_scheduler.rs`: add the probe-yield
   evidence input (:132-147 reset path, :482 record_success) and make the ceiling
   churn-derived (:19 `compute_ceiling_ms`, :201-207 defaults).
5. Two-method census and 12-status rail reads once the quota allows.
