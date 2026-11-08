# Cluster census and dispositions - 2026-11-08 (all 43 open PRs)

Every open PR on the repository is in this record: **43 rows**, one per PR, nothing omitted and
nothing summarised away. It replaces the 2026-11-07 census' nineteen-row population and removes the
gap that record left by dispositioning the dependabot population individually rather than as "the
dependabot class".

This record changes no PR state. It merged nothing, closed nothing, refreshed no branch, dispatched
no run, and read nothing belonging to #489 beyond the census row it already carried.

## 0. Method, and the one correction it forces on earlier reads

- Population: `pulls?state=open&per_page=100`, paginated. 42 open PRs at session start; **43** at the
  read below, because this session opened the CI port (#677).
- Per PR: `pulls/<n>` (mergeable, mergeStateStatus, additions, deletions, changed_files, commits),
  `compare/main...<head>` (`behind_by`, `ahead_by`), `commits/<head>/check-runs?per_page=100&filter=latest`
  paginated, `pulls/<n>/files?per_page=100` paginated, `pulls/<n>/reviews?per_page=100` paginated.
- **Read errors: 0 across 43 rows.** Full pagination, no default page size accepted anywhere:
  `check-runs` defaults to 30 items and `files` to 30, and both are paginated here to `total_count`.
- One parse defect was caught and fixed in-flight rather than reported as a finding: the first pass
  read `--paginate` output for `check-runs` as a bare array, but the endpoint returns
  `{"total_count":..,"check_runs":[..]}`, so the first pass produced empty check-run columns for
  all 43 rows. The columns were empty, not zero; the read was re-run with the object unwrapped and
  the numbers below come from that second pass.

The census also forces one correction on this lane's own prior work, recorded in section 4: the
2026-11-08 band read reported **3** cars in the 31-63 band carrying a red required context. Read
completely, there are **4**, and the missed one is genuine.

## 1. The fence, measured rather than restated: 41 of 43 heads have no rail at all

`commits/<head>/status` on **every** open head, filtered to `ci/circleci:*`:

| Read | Result |
|---|---|
| heads carrying any `ci/circleci` context | **2** - #653 (lane head, 12 contexts) and #677 (this session's port, 12 contexts) |
| heads carrying **none** | **41** |
| `.circleci/config.yml` on `origin/main` (`git cat-file -e`) | **absent** |

So the standing statement - "main is UNVERIFIABLE" - is not a slogan and it now has a successor
sentence: **no car in any cluster can produce an admissible battery today**, because the rail config
exists only where it was ported, and every other car is based on a `main` that does not have it.

That is what #677 changes. It is the only item in the backlog whose completion makes the other 41
rows possible, and it is why the port was taken as C1's first item this session rather than a
disposition pass alone.

## 2. Census (fresh reads, 2026-11-08)

`runs` is every check-run on the head, paginated. `fail` is runs concluding `failure`. `req red` names
the failed contexts among the five required names. `gated` counts changed paths under
`core/src/{crypto,transport,routing,privacy}/` or `vendor/`. `reviews` counts APPROVED /
CHANGES_REQUESTED / COMMENTED review records.

| PR | cluster | draft | head | mergeState | behind | files | runs | fail | req red | gated | reviews |
|---|---|---|---|---|---|---|---|---|---|---|---|
| #677 | C1 port | no | `f3b0db3cb241` | blocked | 0 | 1 | 11 | 3 | - | 0 | 0 |
| #664 | dependabot | no | `cbcf4bf16fa6` | unstable | 0 | 1 | 28 | 12 | - | 0 | 0 |
| #663 | dependabot | no | `ce475f431b1b` | unstable | 0 | 2 | 20 | 8 | - | 0 | 0 |
| #662 | dependabot | no | `62ba6d14430f` | unstable | 0 | 1 | 20 | 8 | - | 0 | 0 |
| #660 | C1 | no | `210b7d932f4e` | unstable | 0 | 23 | 29 | 13 | - | 3 | 0 |
| #657 | straggler | yes | `2b886c5cb7b9` | behind | 31 | 5 | 26 | 0 | - | 0 | 0 |
| #656 | straggler | yes | `7a588667110f` | behind | 26 | 4 | 26 | 0 | - | 0 | 0 |
| #653 | surface | yes | `ae869a48ef3c` | blocked | 0 | 73 | 20 | 8 | - | 0 | 0 |
| #647 | dependabot | no | `582cad1ea918` | behind | 37 | 4 | 27 | 4 | Repository Hygiene Checks | 0 | 0 |
| #646 | dependabot | no | `c0e91ee3cfaf` | behind | 31 | 1 | 26 | 0 | - | 0 | 0 |
| #645 | dependabot | no | `7752fce95615` | behind | 37 | 1 | 27 | 0 | - | 0 | 0 |
| #643 | C2 | no | `0340c094468f` | dirty | 34 | 3 | 26 | 0 | - | 0 | 0 |
| #635 | dependabot | no | `de27304f982c` | behind | 43 | 2 | 29 | 0 | - | 0 | 0 |
| #634 | dependabot | no | `e9cb31d880b4` | behind | 43 | 4 | 30 | 0 | - | 0 | 0 |
| #633 | dependabot | no | `b605d0c08dab` | behind | 43 | 2 | 29 | 8 | Lint | 0 | 0 |
| #632 | dependabot | no | `50c1a661823d` | behind | 43 | 2 | 29 | 0 | - | 0 | 0 |
| #631 | dependabot | no | `21165ad9cd58` | behind | 43 | 3 | 30 | 4 | - | 0 | 0 |
| #630 | dependabot | no | `08af5fa64924` | behind | 43 | 2 | 30 | 0 | - | 0 | 0 |
| #611 | C2 | no | `e03c1d7df43a` | unstable | 0 | 1 | 22 | 8 | - | 0 | 0 |
| #609 | C2 | no | `4ed8168bbf1f` | dirty | 46 | 2 | 27 | 0 | - | 0 | 0 |
| #599 | C5 | no | `a7ef2f299753` | clean | 63 | 8 | 9 | 0 | - | 6 | 0 |
| #556 | C1 | no | `9282583a93e4` | behind | 46 | 12 | 35 | 0 | - | 0 | 0 |
| #554 | C1 | no | `5b3efc47de11` | behind | 46 | 7 | 35 | 0 | - | 3 | 1 |
| #550 | C1 | no | `6441203a1617` | behind | 46 | 9 | 35 | 0 | - | 0 | 0 |
| #542 | dependabot | no | `b3e1fe82dd6f` | behind | 49 | 1 | 32 | 2 | - | 0 | 0 |
| #540 | dependabot | no | `77bbec27f3fa` | behind | 49 | 1 | 32 | 3 | - | 0 | 0 |
| #538 | dependabot | no | `990d7ad28e6b` | behind | 49 | 7 | 34 | 14 | Lint, Test (ubuntu-latest) | 3 | 0 |
| #536 | dependabot | no | `f95dd9e9bd63` | behind | 46 | 2 | 35 | 0 | - | 0 | 0 |
| #535 | dependabot | no | `32bb0fae9ba0` | behind | 46 | 2 | 35 | 0 | - | 0 | 0 |
| #534 | dependabot | no | `9c049fcf7780` | behind | 46 | 1 | 33 | 1 | - | 0 | 0 |
| #529 | dependabot | no | `b5fa9b135bb0` | behind | 31 | 1 | 32 | 1 | - | 0 | 0 |
| #528 | dependabot | no | `1e523e61c4a8` | blocked | 0 | 10 | 21 | 12 | Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest) | 0 | 0 |
| #521 | C5 | no | `5c6b5e52ed29` | unstable | 0 | 45 | 29 | 14 | - | 40 | 0 |
| #510 | C4 | no | `c48cb42b0920` | behind | 37 | 3 | 35 | 3 | - | 0 | 0 |
| #509 | C1 | no | `32e64212cf17` | behind | 51 | 1 | 34 | 0 | - | 0 | 0 |
| #508 | C4 | no | `475816012256` | unstable | 0 | 2 | 27 | 11 | - | 0 | 0 |
| #507 | C4 | no | `f4d5155d4af5` | blocked | 0 | 1 | 27 | 11 | Test (ubuntu-latest) | 0 | 0 |
| #506 | C4 | no | `16b4edf7540b` | unstable | 0 | 3 | 27 | 11 | - | 0 | 0 |
| #489 | HELD | no | `fc3540e481bc` | dirty | 89 | 44 | 37 | 0 | - | 39 | 0 |
| #220 | straggler | no | `92b8ec12378c` | unstable | 807 | 9 | 7 | 2 | - | 0 | 0 |
| #216 | straggler | yes | `b2020a20fbbd` | unstable | 807 | 11 | 7 | 3 | - | 0 | 0 |
| #208 | #208 | no | `11f3e5e93f2a` | dirty | 807 | 16 | 27 | 6 | Repository Hygiene Checks, Lint, Rust Linting | 0 | 0 |
| #178 | C3 | no | `c1b457bbb1ce` | dirty | 1137 | 51 | 6 | 3 | - | 1 | 0 |

Two whole-population facts, both reads rather than estimates: **zero reviews on all 43 heads**, and
**exactly one car (#554) has ever received a review record at all**, which is a COMMENTED record, not
an approval. Clause 10's precondition is unmet everywhere, for the reason A16 already established.

Wait - those two statements contradict each other, and the table is the authority: `reviews` is 0 on
every row except #554, which carries 1. The corrected statement is: **one of 43 heads carries any
review record (#554, 1, COMMENTED), and none carries an approval.** The contradiction is left visible
rather than edited out, because a record that quietly tidies itself is not evidence.

## 3. Dispositions, all 43

Vocabulary unchanged from the 2026-11-07 record: **CLOSED**, **DECISION-READY** (every lane-side step
done, a named owner input remains), **PARKED-WITH-PACKET**, **HELD**.

### C1 - Batch D, and the port that unblocks it

| Car | Disposition | Basis |
|---|---|---|
| #677 (CI port) | **NEW THIS SESSION - awaiting its own battery** | One file, off `main`. Its head carries **12 `ci/circleci` contexts already: 6 success, 6 pending, 0 failure** at read time, which is the first main-lineage rail battery on this repo. Merge basis is that battery plus this receipt; nothing is merged by this record. |
| #660 (combined head) | **DECISION-READY** | Unchanged: 0 reviews, 13 failures of which 13 are billing-annotated (A3/A18), 3 gated paths. Two preconditions remain outside the lane: a rail battery (now #677) and two non-author PASSes (A16). |
| #550, #554, #556, #509 | **DECISION-READY** | Superseded in content by #660. #554 additionally carries the only review record in the population, and it is a COMMENTED record, so it changes nothing about clause 10. |

### C2 - CI lane / scope decisions

| Car | Disposition | Basis |
|---|---|---|
| #611 | **DECISION-READY** (CI owner) | 1 file, behind 0, 8 failures, no required red. CI UNREAD on the admissible rail and stays unread until #677 lands and it is forward-merged. |
| #643 | **DECISION-READY** (CI owner) | DIRTY, 34 behind, 3 files, 0 reds. Carve-out half already landed on the lane in `7bc5e0250`; what remains is the pool re-scope. |
| #609 | **DECISION-READY** (infra #607) | DIRTY, 46 behind, 2 files, conflicts on the single `ci.yml` path. `ubicloud-standard-2` still has no registered runner. |
| #601 | **CLOSED** (closed 2026-11-05) | Re-read: not in the open population. |

### C4 - Batch C platform cars

| Car | Disposition | Basis |
|---|---|---|
| #506 | **DECISION-READY** | behind 0, 3 files, 11 failures all billing residue (A5/A3). Needs the rail, then clause 10. |
| #508 | **DECISION-READY** | behind 0, 2 files, 11 billing-residue failures. Same two preconditions. |
| #507 | **DECISION-READY** (CI owner) | **Still BLOCKED, and the blocking failure is still the billing lock**: `Test (ubuntu-latest)` check-run `127192677132`, `runner_name` empty, `steps=0`, annotation "The job was not started because your account is locked due to a billing issue." This session re-read it and it is unchanged since 2026-11-05. A25's inert-diff contract is why the required leg lands on hosted in the first place; the cure is SR-1 or a contract change, not a refresh. |
| #510 | **PARKED-WITH-PACKET** | behind 37, unblocked only by the owner's plan next-action 4. |

### C3 - #178

| Car | Disposition | Basis |
|---|---|---|
| #178 | **DECISION-READY** (owner) | DIRTY, 1137 behind, 51 files, 1 gated path (`core/src/transport/ble/gatt.rs`), 6 check-runs and 3 failures all from 2026-08-17. Unchanged from A6/A10/A28: the red is a genuine content defect, the patch exists and reproduces a clean build, the head is a fork so the lane cannot deliver it, and iOS gates are `xcodebuild`-only on the operator's Mac. |

### C5 - Batch E transport chain

| Car | Disposition | Basis |
|---|---|---|
| #521 | **DECISION-READY** | behind 0, 45 files, 40 gated paths, 0 reviews. Gate is a non-author verdict (A16), not code. |
| #599 | **PARKED-WITH-PACKET** | base is `feat/conn-admission-dynamic`, so it cannot produce required contexts at all - confirmed here by the read: 9 check-runs and **none** of the five required names present. Unblocks when #521 lands and its base re-points at main. |

### #208

| Car | Disposition | Basis |
|---|---|---|
| #208 | **DECISION-READY** (fork author / owner) | DIRTY, 807 behind, 16 files, three genuine reds unchanged since 2026-08-22 (`Repository Hygiene Checks`, `Lint`, `Rust Linting`, each exit 1 with real steps executed). A29/A30 root causes and patches already filed on the PR because the fork is `push:false`. |

### Stragglers

| Car | Disposition | Basis |
|---|---|---|
| #657 | **PARKED-WITH-PACKET** | draft, 31 behind, 5 files. Author lane decides whether it is finished or closed. |
| #656 | **DECISION-READY** (CI owner) | draft, 26 behind, 4 files. Path filters for heavy workflows interact with the required-context definition, so the CI owner must confirm filtered contexts can never be required. |
| #489 | **HELD** | Standing order. Read for the census row only - mergeState, head, counts - nothing else, and no action. |
| #220 | **DECISION-READY** (identity chain) | base `feat/identity-id-unification`, 807 behind that base, 9 files, 0 gated. |
| #216 | **DECISION-READY** (identity chain) | draft, same base, 11 files, 0 gated. |

### Dependabot population - 20 PRs, dispositioned individually

The 2026-11-07 record dispositioned two dependabot cars and left the rest to a class rule. All 20 open
dependabot PRs are named here instead.

**Class D-RED - must not land until repaired (4 cars).** Each carries a genuine failed required
context, each read below with the annotation and the job records:

| Car | Red | Read |
|---|---|---|
| #528 | 4 required reds | `Rust Linting` `126634515150`, `Test (ubuntu-latest)` `126607622870`, `Lint` `126607622589`, `Repository Hygiene Checks` `126603174358`; all on `oci-a1-phx-2` with executed steps, none billing. See section 5: this one is repo-wide, not PR-local. |
| #538 | `Lint`, `Test (ubuntu-latest)` | `119705911685` and `119705911552`, exit 101 both, `2026-10-22`. Touch `core/src/crypto/encrypt.rs`, `core/src/crypto/pq/hybrid.rs`, `core/src/privacy/onion.rs` - **rule-8 gated** with 0 reviews. |
| #647 | `Repository Hygiene Checks` | `120957642987`, exit 1, 7 successful steps, `2026-10-24`. Gradle wrapper 8.13 to 9.8.2. |
| #633 | `Lint` | `120405449384`, exit 101, 7 successful steps, `2026-10-23`. `rand` 0.8.7 to 0.10.2. |

**Class D-PARK - behind, no required red, needs the rail (12 cars).** #664, #663, #662, #646, #645,
#635, #634, #632, #631, #630, #542, #540. Named so no row is missing; #631's four failures include a
`cancelled` `Test (ubuntu-latest)`, which is not a defect and is not counted as a red.

**Class D-PARK-THROUGH-BASE - behind a non-main base (0 cars).** None: every open dependabot PR bases
on `main`.

**Class D-LOW-RISK-SMALL - one line, no red, no gated path (4 cars).** #536 (`rustc-hash`),
#535 (`tower-http`), #534 (`zxing`), #529 (`actions/setup-java`). These are the cheapest first
landings once the rail is on main; they remain clause-7 gate-note landings, not lane merges.

That accounts for 4 + 12 + 4 = 20, which is the full dependabot population.

## 4. Corrections to the filed record

1. **The 31-63 band carried 4 red-required cars, not 3.** The 2026-11-08 band read
   (`docs/audit/A23_LANDING_COST_BEHIND_2026-11-08.md`, status comment `6720422364`) named three:
   #631 (cancelled, not a defect), #633 (`Lint` exit 101), #647 (hygiene exit 1). **#538** sits at
   `behind_by=49`, inside the band, and carries two genuine required reds (`Lint` `119705911685` and
   `Test (ubuntu-latest)` `119705911552`, both exit 101, both 2026-10-22). It was read as a rule-8
   car and its reds were not carried into the band's red list. The band therefore has **5** cars
   carrying a red required context once #538 is counted, of which **4 are genuine** and one (#631) is a
   cancelled run.
2. **#654 is CLOSED, not open.** `closedAt` `2026-11-07T18:55:28Z`, `mergedAt` null. The 2026-11-07
   record parked it as an open dependabot car; it closed thirteen hours later and is now out of scope.
   It is not in this census, and its absence is accounted for here rather than left as a silent gap in
   the count.
3. **The population moved from 42 to 43** in one direction only: this session's port (#677). No car
   was merged or closed by this session.
4. **"Zero reviews everywhere" was imprecise**: 42 heads carry none, #554 carries one COMMENTED record.
   No head carries an approval, so clause 10's finding is unchanged while the statement is now exact.

## 5. The one finding that changes behaviour: #528 would break every Rust job on main

#528 bumps `dtolnay/rust-toolchain` from `1.99.0` to `1.120.0`. Its head's `Rust Linting` check-run
`126634515150` carries this annotation in the real run:

```
warning: Unexpected input(s) 'toolchain', valid inputs are ['targets', 'target', 'components']
```

That is not a version-resolution complaint, it is the action rejecting an input the repository passes
it. On `origin/main`, `lint.yml:53` passes exactly that input:

```
- uses: dtolnay/rust-toolchain@1.99.0
  with:
    toolchain: 1.99.0
```

So the bump and the workflow are incompatible, and the incompatibility is in a **required** context
(`Rust Linting`). The failure is repo-wide in mechanism: landing #528 as it stands reds the Rust
Linting job for every subsequent commit on main, and the other three reds on that head are the same
cause reaching the jobs that install the toolchain.

Disposition: **D-RED, not landable as filed.** The cure is a two-line change in the same PR (drop the
`toolchain:` input, keep the pin in the action ref), which makes #528 a repair rather than a
withdrawal. This record does not perform that repair: it would mean writing to a dependabot branch,
and the acceptance test for the repair is a rail battery that does not exist until #677 lands. Filed
here so the repo-wide blast radius is visible before anyone dispositions #528 as a routine one-line bump.

## 6. What this does not prove

- It does not prove any car is landable. Merging needs a green CircleCI battery at the head, and 41 of
  43 heads cannot have one until #677 lands and the car is forward-merged onto it.
- It does not prove #677 is green. Its battery read 6 success / 6 pending / 0 failure at census time and
  was not read to terminal in this pass. No green claim is made for it.
- It does not disposition #489 beyond its census row. Standing order: green and held, untouched.
- It changes no protection, no required check, no reviewer and no branch.

## 7. Next action

Read #677's battery to terminal and land it under the merge law, then the cluster order resumes at C1
with #660 - which, once the port is on main, becomes the first car in the backlog able to carry an
admissible battery rather than only a billing annotation.
