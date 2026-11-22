# SCM-CURE CLOUD LANE -- stand-up filing, 2026-11-21 (oversight)

Mission acknowledged verbatim in substance: **SCMESSENGER CLOUD CURE LANE --
2026-11-21 (oversight)**. Cloud workspace, branch
`standup/scm-cure-2026-11-21`, filed as the git-write proof the mission asks
for first.

Turn window: 2026-11-21 23:2x HST (2026-11-22 09:2x UTC) at first census read,
filed before the 00:00 HST cutoff.

## 1. Census

### 1.1 main head -- two methods

| method | command | result |
| --- | --- | --- |
| A (authoritative remote read) | `git ls-remote origin refs/heads/main` | `16f7171acfe2620818c1042e3779b58826e5e3eb` -- `Merge pull request #728 from Treystu/dependabot/cargo/rust-dependencies-edc051d422` |
| B (local remote-tracking ref) | `git rev-parse origin/main` | `19612b7608cea2c9de500f6ab867047d62d2e8bb` -- `Merge pull request #723 ...fix/571-contact-storage-errors`, committed 2026-11-20 15:31:43 -1000 |

**The two methods disagree, and the disagreement is the finding.** This clone
carries no fetch refspec at all (`git config --get-all remote.origin.fetch`
prints nothing), so `origin/main` is a frozen snapshot from cloning time, not a
tracking branch. Method B is two merges stale; method A is the head. All
statements about main below use method A.

Local `HEAD` of `freebuff/9caaf975875224d5d0c56970` is the same object as
method A's main (`git log --oneline -1` -> `16f7171a Merge pull request #728 ...`).

### 1.2 Open issues -- 91

Read: anonymous GitHub REST search API,
`/search/issues?q=repo:Treystu/SCMessenger is:issue is:open&per_page=100&sort=created&order=asc`,
2026-11-22T09:26Z. `total_count 91`, `items 91`, `incomplete_results False`,
`per_page=100` -- **91 is a count, not a page ceiling**: the request page size
is 100 and 91 items came back, so the full set is present and is reproduced in
full below, not truncated.

```
155 417 418 419 448 449 455 456 457 458 459 460 461 462 463 464 465 466 467
468 469 477 478 490 491 492 493 494 495 497 499 517 525 544 545 548 549 552
553 555 561 563 567 573 574 575 576 577 578 580 581 582 583 584 585 586 587
588 589 590 591 592 593 594 595 600 602 603 607 619 620 623 624 625 642 665
675 679 682 683 684 686 688 689 690 691 692 693 704 719
```

Second method attempted for this axis -- **it failed, filed verbatim**:

```
$ gh issue list --repo Treystu/SCMessenger --state open --limit 200
You are not logged into any GitHub hosts. To log in, run: gh auth login
```

Third method attempted (unauthenticated REST core endpoint, different endpoint
from the search index) -- **it failed, filed verbatim**:

```
$ curl -s https://api.github.com/rate_limit
  "core":     { "limit": 60, "remaining": 0,  "reset": 1795341656, "used": 60 }
  "search":   { "limit": 10, "remaining": 10, "reset": 1795339577, "used": 0  }
  "graphql":  { "limit": 0,  "remaining": 0 }

$ curl -s https://api.github.com/repos/Treystu/SCMessenger
HTTP 403 {"message":"API rate limit exceeded for 208.72.218.131. (But here's
the good news: Authenticated requests get a higher rate limit. ...",
"documentation_url":"https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"}
```

So the issue count rests on ONE method (the search index), with the two
independent channels unavailable from this workspace. It is reported as
single-method, not as a two-method agreement. `core` reset 1795341656 is
2026-11-22T10:00:56Z -- **56 seconds after the 00:00 HST cutoff**
(epoch 1795341600), so it cannot be re-read inside this turn window.

### 1.3 Open PRs -- 19

Same search API, `is:pr is:open`, 2026-11-22T09:27Z: `total_count 19`,
`items 19`, `incomplete_results False`. Full set:

| # | created | author | title |
| --- | --- | --- | --- |
| 208 | 2026-08-22 | pixiegirlchristy | docs(apple): 4-node parity rollout status, coordination journal & iOS link-local |
| 216 | 2026-08-23 | Treystu | fix(android): wire up unreachable features + KeyStore passphrase; wiring gate 32 |
| 220 | 2026-08-23 | Treystu | fix(android): reachability fixes, split from the blocked passphrase change |
| 528 | 2026-10-10 | dependabot[bot] | bump dtolnay/rust-toolchain 1.99.0 -> 1.120.0 |
| 534 | 2026-10-10 | dependabot[bot] | bump com.google.zxing:core 3.5.2 -> 3.5.4 (/android) |
| 538 | 2026-10-10 | dependabot[bot] | bump x25519-dalek 2.0.1 -> 3.0.0 |
| 540 | 2026-10-10 | dependabot[bot] | bump kotlin_version 1.9.20 -> 2.4.20 (/android) |
| 542 | 2026-10-10 | dependabot[bot] | bump androidx.navigation:navigation-compose 2.7.5 -> 2.10.2 |
| 599 | 2026-10-14 | Treystu | [RULE-8] fix(transport): close #521 review blockers (#573, #574, #575) |
| 609 | 2026-10-20 | Treystu | [ci] Switch the five core jobs to Ubicloud standard-2 |
| 643 | 2026-10-23 | Treystu | ci: reserve the two ARM OCI runners for car waves, use hosted for the rest |
| 645 | 2026-10-24 | dependabot[bot] | bump com.google.android.gms:play-services-base 18.2.0 -> 18.11 |
| 647 | 2026-10-24 | dependabot[bot] | bump gradle-wrapper 8.13 -> 9.9.0 (/android) |
| 712 | 2026-11-14 | dependabot[bot] | bump github/gh-aw 0.87.3 -> 1.2.2 |
| 725 | 2026-11-21 | dependabot[bot] | bump actions/github-script 7 -> 9 |
| 726 | 2026-11-21 | dependabot[bot] | bump softprops/action-gh-release 2 -> 3 |
| 727 | 2026-11-21 | dependabot[bot] | bump actions/cache/save 4.3.0 -> 6.1.0 |
| 729 | 2026-11-21 | dependabot[bot] | bump lz4_flex 0.11.6 -> 0.14.2 |
| 732 | 2026-11-21 | dependabot[bot] | bump base64 0.22.1 -> 0.23.1 |

Second method for this axis, partial by construction: `refs/pull/*/head` on the
remote enumerates **601** pull-request refs and a highest number of **733**
(`git ls-remote origin 'refs/pull/*/head' | wc -l` -> 601; max N -> 733). That
is a lifetime total including closed PRs, so it corroborates the namespace and
the numbering but **cannot** corroborate the open count. `gh pr list` fails with
the same `not logged into any GitHub hosts` line filed in 1.2.

Fence check against the list: #599 and #571 are the local lane's. #571 is not
in the open set (it merged as PR #724, `681d2991`). #599 **is** open -- it is
named here only as census, untouched, per the fence. The held set is present as
expected: #712 parked; the Android dependabot PRs #534, #540, #542, #645 and
the CI pair #609, #643 with #719 attached are all still open.

### 1.4 Environment census (the thing that decides the queue)

```
$ which cargo rustc python3
/usr/local/bin/python3
$ cargo --version
(not found)  -> no Rust toolchain in this workspace
$ git config --get core.hooksPath
(no hooksPath set -- .githooks/pre-commit and pre-push exist but are not wired)
$ df -h /workspace    (not read this turn -- not needed for a doc-only filing)
```

## 2. Queue, in mission order -- what this workspace can and cannot do

| item | state here | reason, with the read |
| --- | --- | --- |
| #704 Pass E re-run | **BLOCKED** | The re-run is `python scripts/jev_repo_insights.py --mode canonical_completion`, which needs the keyed Jev harness. Key resolution reads: `~/.typesafe_key` -> No such file or directory; `~/.config/typesafe*` -> No such file or directory; no token-shaped env var; `vendor/sovereign-harness` -> No such file or directory in this tree. Canonical Jev from this workspace reads **NOT CHECKED**, exactly as the mission pre-states. The blocker is now narrowed to the key alone, and the probe produced three findings larger than the blocker -- see 2.2. |
| #693 cover traffic parameterization | **NOT STARTED -- two independent blockers** | `core/src/privacy/cover.rs` is rule-8 gated (`core/src/privacy/`), so it needs a recorded non-author adversarial APPROVE this workspace cannot produce; and there is no `cargo` here, so no change could be compiled, let alone measured against the issue's 23.5x-bytes claim. |
| #692 reputation rebalance | **NOT STARTED -- one blocker** | `core/src/abuse/reputation.rs` is not rule-8 gated, but it is Rust and this workspace has no Rust toolchain; an uncompiled change is not a deliverable under the lane's rules. |
| #691 TTL/route gap + phase timing | **NOT STARTED -- two blockers** | `core/src/routing/negative_cache.rs`, `adaptive_ttl.rs`, `timeout_budget.rs` are all in `core/src/routing/` -- rule-8 gated -- and uncompilable here for the same missing-toolchain reason. |

No implementation was attempted for any of the four. Nothing was pushed to
#599, #571, main, or any open PR head. Nothing was merged, tagged, or closed.

### 2.1 The queue premises were re-read on today's head, not carried from 2026-11-13

Main moved two merges past the 2026-11-13 audit base (`ced2189e`), so the four
premises were re-checked against `16f7171a` before any of them was called
blocked. Read: the five cited files are **byte-identical in history** since the
audit base -- `git log --oneline ced2189e..HEAD -- core/src/privacy/cover.rs
core/src/abuse/reputation.rs core/src/routing/negative_cache.rs
core/src/routing/adaptive_ttl.rs core/src/routing/timeout_budget.rs` prints
nothing -- and the line-level reads below agree with the audit text.

| item | live read on `16f7171a` | premise |
| --- | --- | --- |
| #693 | `cover.rs:33` `rate_per_minute: 10`; `:66` `(60_000 / self.rate_per_minute as u64).max(1)`; `:174` `elapsed > interval` | met -- fixed metronome, no coupling to real traffic |
| #692 | `reputation.rs:52` `record_signal`, `:101` `record_outbound_message`; a `witness|confirm` grep over the whole file returns **zero** hits | met -- both positive paths are self-reported, no witnessed set |
| #691 (a) | `negative_cache.rs:211` `Self::new(Duration::from_secs(600), 10000)`; `adaptive_ttl.rs:39` `adaptive_ttl: Duration::from_secs(1800)` | met -- 600 s negative evidence inside 1800 s routes |
| #691 (b) | `timeout_budget.rs:33-36` phases `10`, `50`, `200`, `500` = 760 ms; `:85` `Self::new(Duration::from_millis(500))` | met -- the 500 ms phase cannot be reached inside a 500 ms budget |

So none of the four is premise-refuted: they are open, correctly filed, and
blocked here purely by environment. That distinction is the point of this
section -- a later session with a Rust toolchain and a rule-8 reviewer inherits
four live defects, not four maybe-defects.

### 2.2 #704: the harness half was a bootstrap gap, and the instrument has two silent-failure defects

First probe, filed verbatim, was a refusal rather than a rating:

```
$ python3 scripts/jev_repo_insights.py --mode canonical_completion --out tmp/passE_probe.md
[FAIL] Harness checkout missing: /workspace/chats/.../vendor/sovereign-harness
```

That path is `.gitignore:397-398` ("Local sovereign-harness consumer copy"), and the script's own usage line names the remedy. Running it worked:

```
$ python3 scripts/update_local_harness.py
[OK] Harness admission operation complete
$ git -C vendor/sovereign-harness rev-parse HEAD
ad4a30052955e1f574c90f426edfc7a274e16ebf   (matches scripts/harness_admission.json: tag v0.4.1, PRODUCTION)
```

So on a fresh cloud clone the **harness absence is a one-command bootstrap
step, not a missing dependency** -- record it as friction, not as a blocker.
The residual blocker for a real Pass E re-run is exactly one thing: the key.

```
$ python3 scripts/jev_repo_insights.py --mode canonical_completion --out tmp/passE_probe2.md
[INFO] harness=.../vendor/sovereign-harness sha=ad4a3005... status=PRODUCTION pinned=True keyed=False
[OK] pack=canonical_completion batches=1 tokens=0 cost=0.0
  Batch 0 [FALLBACK] verdict=fail supported=0.0 conf=0.0
  - reason: Empty candidate state returned.
```

An unkeyed run is `[FALLBACK]`, and the script's own docstring is explicit that
unkeyed "is_fallback and must not be treated as live canonical judgment". The
`supported=0.0` above is therefore **not** a re-measurement of #704's 0.11/0.44/
0.61 and must never be read as one. #704 stays open, unmeasured, and cannot be
closed from here.

Two defects found while doing it, both of which would corrupt a Pass E run from
this workspace **even with a key** -- these are the deliverable of this item:

1. **The instrument audits a stale head in a fresh clone.** The first report's
   header read `Main tip (origin): 19612b76` while the remote head was
   `16f7171a` -- two merges behind (see 1.1). `git fetch origin main` does not
   fix it: with no fetch refspec in this clone (`.gitconfig` `remote.origin.fetch`
   is empty) the fetch writes only `FETCH_HEAD`, and `git rev-parse origin/main`
   still returned `19612b76` afterwards. It was repaired for evidence with
   `git update-ref refs/remotes/origin/main $(git rev-parse FETCH_HEAD)`, after
   which the header read `16f7171a`. A Pass E run in a cloud clone must assert
   `origin/main == git ls-remote origin refs/heads/main` before it is trusted.
2. **A failed read becomes a zero.** With `gh` unauthenticated the harvest
   printed `Open PR count (approx from gh json keys): 0` -- no warning, no
   failure marker. The true count is 19. That zero is fed to the pack that
   scores `instruction_matches`, so the failure mode is precisely the one
   AGENTS.md rule 15 exists to forbid: a tool that cannot see, reporting a
   number. The fix belongs in the harvest (emit `unavailable` and fail the
   section, never 0), and it should precede any re-scored Pass E.

No patch to `scripts/jev_repo_insights.py` is filed here: it is a gate
instrument, this workspace cannot run its test suite against a keyed live pass,
and an unverified change to the thing that scores the gate is worse than the
documented defect. Filed for a session with the key and the rail.

## 3. Actions this workspace CANNOT perform -- for oversight to mirror

Filed verbatim rather than worked around, per the mission:

1. `gh issue list`, `gh pr list`, `gh pr view`, `gh run list`, `gh api` -- all
   fail with `You are not logged into any GitHub hosts. To log in, run: gh auth
   login`. No token exists in this workspace.
2. Therefore, for oversight to mirror, on the lane's behalf:
   - no comment can be posted on the tracker (#602) mirroring this STATUS;
   - no claim can be made on #704/#693/#692/#691;
   - no check-run or CircleCI status can be read on any exact head, so **no
     landing decision of any kind can be evidenced from here** -- the mission's
     "land only at green exact heads" rule is unsatisfiable in this workspace,
     and no landing was attempted;
   - `CONFORMANCE` (protocol clause 14) cannot be filled: acceptance is a
     read-back of check-run IDs, and check-runs are unreadable here.
3. Anonymous REST core quota is exhausted (0/60) and resets 2026-11-22T10:00:56Z,
   past the cutoff. The search index (10/10 at census time) is the only live
   read channel and cannot list comments, check-runs, or commit status.
4. No Rust toolchain: `cargo`/`rustc` are absent, so no queue item can be
   compiled or run. CI remains the only verifier available to this lane, and
   pushing to invoke it is a write this turn did not need.

## 4. Status

```
DONE    stand-up filed on standup/scm-cure-2026-11-21 (git-write test); census
        taken: main 16f7171a (ls-remote), 91 open issues, 19 open PRs, lists in
        full; single-method status on issue/PR counts declared as such.
WIP     none.
NEXT    #704 re-run needs only the key now (harness bootstrap is a command);
        fix the harvest's silent zero and the stale-tip read first, or the
        re-score is measured on wrong inputs. #693/#692/#691 need a Rust
        toolchain plus, for #693 and #691, a non-author rule-8 review. All four
        remain environment-blocked, not premises-refuted.
BLOCKERS  gh unauthenticated; anonymous core quota 0 until 10:00:56Z; canonical
        Jev NOT CHECKED (harness admitted at ad4a3005, key absent); no cargo/rustc.
        Instrument defects for #704: stale origin/main header, and gh failure
        reported as "Open PR count 0" (see 2.2).
DIRECTIVES none read this turn -- comment reads are impossible, so this session
        cannot rule out a newer Treystu directive; newest-wins rule is
        unserviceable from here and is declared, not assumed.
```
