# A23 landing-cost measurement, extension: the `behind_by` 31-63 set, 2026-11-08

Status: measured, read-only. Every value below came from an API or git read in this session. Where a value
cannot be resolved, the text says so and names the reason rather than estimating.

This extends `docs/audit/A23_LANDING_COST_2026-11-08.md`. That document measured the cars already level with
`main` (`behind_by=0`) and closed with one unmeasured sentence: *"Every other non-draft PR is `behind_by` 31 to
1137, so each costs an `update-branch` plus a full required run before it can even enter this ordering."* This
document replaces that sentence with a measurement of the 31-63 band.

## Why the band is 31-63 and what is outside it

Window bounds come from the steering directive (PR #653 comment 2026-11-08T10:07:35Z, item 1) and match the four
cars row A23 already named (#654 31, #643 34, #609 46, #509 51). The band is inclusive at both ends.

Full census this session: **42 open PRs**, paginated to completion (one page, 42 returned, no page cap hit),
`main = ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad`. Distribution of `behind_by`:

| behind_by | cars | PRs |
|---|---|---|
| 0 | 11 | #664 #663 #662 #660 #653 #611 #528 #521 #508 #507 #506 |
| 26 | 1 | #656 |
| 31-63 (this document) | 25 | see the tables below |
| 89 | 1 | #489 (owner-protected: not read, not measured) |
| 807 | 3 | #220 #216 #208 |
| 1137 | 1 | #178 (owner-held, A6) |

#656 at `behind_by=26` falls just under the band and is named here so its absence is accounted for rather than
silent. The four cars at 807 and 1137 are past any plausible level-and-land cost and are not part of this set.

## The set is 25 cars, not the four A23 named

Row A23 recorded four behind cars as an aside. Read completely, the band holds **25**. That is the substantive
correction: a plan built on four would have omitted twenty-one cars, including six dependabot cars at 43 and the
whole `#550/#554/#556` family at 46.

Required contexts were read from branch protection rather than assumed: 5 contexts -
`Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope`. Check-runs were read per head with `filter=latest`
paginated to `total_count`; no car hit the 100-run page cap, so every run count below is a count and not a
ceiling.

## Tier 1 - behind only: no red required context, no gated path, no conflict

Fourteen cars. Each needs one `update-branch` and therefore one fresh run of the five required contexts. Ordered
by diff size, cheapest first, so the widest revert surface loses the fewest revalidations.

| # | head | behind | required red | mergeable / state | files | diff | gated | conflicts |
|---|---|---|---|---|---|---|---|---|
| 645 | `7752fce95615` | 37 | 0 | true / behind | 1 | +1 / -1 | 0 | 0 |
| 542 | `b3e1fe82dd6f` | 49 | 0 | true / behind | 1 | +1 / -1 | 0 | 0 |
| 540 | `77bbec27f3fa` | 49 | 0 | true / behind | 1 | +1 / -1 | 0 | 0 |
| 534 | `9c049fcf7780` | 46 | 0 | true / behind | 1 | +1 / -1 | 0 | 0 |
| 529 | `b5fa9b135bb0` | 31 | 0 | true / behind | 1 | +2 / -2 | 0 | 0 |
| 646 | `c0e91ee3cfaf` | 31 | 0 | true / behind | 1 | +2 / -2 | 0 | 0 |
| 635 | `de27304f982c` | 43 | 0 | true / behind | 2 | +3 / -3 | 0 | 0 |
| 535 | `32bb0fae9ba0` | 46 | 0 | true / behind | 2 | +4 / -4 | 0 | 0 |
| 536 | `f95dd9e9bd63` | 46 | 0 | true / behind | 2 | +6 / -12 | 0 | 0 |
| 632 | `50c1a661823d` | 43 | 0 | true / behind | 2 | +30 / -11 | 0 | 0 |
| 634 | `e9cb31d880b4` | 43 | 0 | true / behind | 4 | +20 / -87 | 0 | 0 |
| 630 | `08af5fa64924` | 43 | 0 | true / behind | 2 | +40 / -62 | 0 | 0 |
| 509 | `32e64212cf17` | 51 | 0 | true / behind | 1 | +50 / -6 | 0 | 0 |
| 657 | `2b886c5cb7b9` | 31 | 0 | true / behind | 5 | +255 / -2 | 0 | 0 |

## Tier 2 - behind plus a red required context

Three cars. #631's only red is a `cancelled` run (annotation: *"The run was canceled by @Treystu"*), which is not
a code defect: after the refresh it needs a re-run, not a fix. #633 and #647 carry genuine non-zero exits
(101 and 1 respectively), so each needs a fix before its refresh is worth spending.

| # | head | behind | required red | mergeable / state | files | diff | gated | conflicts |
|---|---|---|---|---|---|---|---|---|
| 631 | `21165ad9cd58` | 43 | `Test (ubuntu-latest)` cancelled (120404028407) | true / behind | 3 | +194 / -32 | 0 | 0 |
| 633 | `b605d0c08dab` | 43 | `Lint` failure (120405449384); `Test (ubuntu-latest)` cancelled (120405449375) | true / behind | 2 | +7 / -7 | 0 | 0 |
| 647 | `582cad1ea918` | 37 | `Repository Hygiene Checks` failure (120957642987) | true / behind | 4 | +124 / -106 | 0 | 0 |

## Tier 3 - behind plus a content conflict

Both conflict on exactly one file, `.github/workflows/ci.yml`, established with `git merge-tree --write-tree`
against `origin/main` rather than from the `dirty` label alone. That is a measured conflict, not an inferred one.

| # | head | behind | required red | mergeable / state | files | diff | gated | conflicts |
|---|---|---|---|---|---|---|---|---|
| 643 | `0340c094468f` | 34 | 0 | false / dirty | 3 | +174 / -15 | 0 | `.github/workflows/ci.yml` |
| 609 | `4ed8168bbf1f` | 46 | 0 | false / dirty | 2 | +38 / -15 | 0 | `.github/workflows/ci.yml` |

This confirms row A24's 2026-11-05 refutation rather than adding a new finding: A24 recorded that `#609`
`4ed8168bbf1f` and `#643` `0340c094468f` each conflict on exactly one path and that it is the same path, so the two
are **one decision, not two**. This measurement was taken independently on today's heads and agrees.

The consequence for ordering is bounded: they are not two sequential conflict resolutions. What the measurement
does not decide is the resolution itself - A24 requires it to be taken on the guard-preserving side (A11/A12) or it
re-creates the stranded-context or rollback-switch defect, and row A24's owner cell is `CI owner`, not this lane.
Landing both after one resolution, or landing one and closing the other as superseded, are both consistent with the
measurement; choosing between them is a scope decision and is not claimed here.

## Tier 4 - gated, or large enough to own a step

| # | head | behind | required red | mergeable / state | files | diff | gated | conflicts |
|---|---|---|---|---|---|---|---|---|
| 550 | `6441203a1617` | 46 | 0 | true / behind | 9 | +506 / -125 | 0 | 0 |
| 556 | `9282583a93e4` | 46 | 0 | true / behind | 12 | +776 / -45 | 0 | 0 |
| 510 | `c48cb42b0920` | 37 | 0 | true / behind | 3 | +2142 / -375 | 0 | 0 |
| 554 | `5b3efc47de11` | 46 | 0 | true / behind | 7 | +1010 / -103 | 3 | 0 |
| 538 | `990d7ad28e6b` | 49 | `Lint` failure (119705911685); `Test (ubuntu-latest)` failure (119705911552) | true / behind | 7 | +17 / -29 | 3 | 0 |
| 599 | `a7ef2f299753` | 63 | 0 | true / clean | 8 | +1618 / -165 | 6 | 0 |

Three of these are rule-8 gated and cannot land on a lane vote however green they read:

- **#599** - 6 of its **8 own** changed paths are under `core/src/transport/`. It also needs an
  adversarial review from a reviewer who did not author it, and per row A16 the repository has exactly one
  collaborator. **Blocked owner-side on access (naming a reviewer), not on code.**

  The 6, not 41, is a correction applied during this measurement. #599's base is `feat/conn-admission-dynamic`
  (which is #521's branch), not `main`. Comparing it against `main` returns 48 files with 41 gated, but that
  baseline folds in #521's own 45 files and is therefore not #599's diff. Against its real base #599 is 8 files,
  6 gated, `+1618 / -165` - which agrees with the PR's own `changed_files`/`additions`/`deletions`. Measured both
  ways; the base-relative figure is the one that describes this car.
- **#554** - 3 gated paths (`core/src/routing/local.rs`, `core/src/routing/optimized_engine.rs`,
  `core/src/transport/swarm.rs`).
- **#538** - 3 gated paths and they are the highest-consequence ones in the set: `core/src/crypto/encrypt.rs`,
  `core/src/crypto/pq/hybrid.rs`, `core/src/privacy/onion.rs`. It is also red on `Lint` and
  `Test (ubuntu-latest)`. A crypto-history change must not be refreshed into a green reading without its
  review; it is last for that reason, not because it is small.

## One structural finding: #599 is not an independent car

#599's base branch is `feat/conn-admission-dynamic`, i.e. **#521's branch**. Read against that base it is
**65 behind / 16 ahead**, so it is not merely behind `main` - it is behind the car it lands into. It cannot merge
to `main` at all; it merges into #521, and #521 then carries both. Its position in any ordered plan is therefore
dependent on #521's, and its `behind_by=63` against `main` is a composite that mixes #521's 45 files into its own
eight.

Any plan that places #599 as a standalone step would be wrong about both its cost and its target. It is listed
last here for that reason as well as the rule-8 reason.

## Cost of leveling the set

Measured, not estimated: 25 cars at `behind_by > 0` under `strict=true`. Every merge re-behind-ifies the cars
that have not merged, so the band is not leveled by 25 independent refreshes. The floor is 25 `update-branch`
calls and 25 fresh runs of the five required contexts, and the ceiling is quadratic in the landing order.

Two facts change what that costs in practice:

1. **The verifier situation changed since A23 was written.** The CircleCI rail reports **12 contexts** on the
   lane head, and all five required contexts read `success` there (pipelines 261-272, one non-required failure:
   `Kotlin Linting`, pipeline 266). It is not on `main` yet, so `main` stays UNVERIFIABLE and the five required
   contexts on every car in this document are still Actions contexts. The rail does not carry these cars.
2. **Four cars carry red that a refresh cannot fix** (#631 is a cancelled run; #633, #647 and #538 are genuine
   exits). Refreshing them buys a fresh run against the same defect.

## Ordered plan

1. **Tier 1 in the order tabled** (14 cars, one required run each, no gate, no conflict).
2. **#631** - refresh and re-run only; its red is a cancellation.
3. **#633**, then **#647** - fix lint, fix hygiene, then refresh each.
4. **#643 and #609 as one decision** - they conflict only on `.github/workflows/ci.yml` and therefore share a
   single guard-preserving resolution (A24, A11/A12). Whether that means landing both or landing one and closing
the other is the CI owner's scope call, not a measurement output.
5. **#550**, **#556**, **#510** - large but ungated; after the small diffs so nothing re-behind-ifies them.
6. **#554**, then **#538** - gated; each needs a non-author adversarial review before merge. #538 last.
7. **#599** - blocked twice over: it lands into #521 (not `main`), and it is 6-of-8 gated with no non-author
   reviewer existing (A16).

Step 7 does not block steps 1-6. Nothing in this ordering requires the protection switch.

## Genuinely unmeasurable, with reason

- **Whether each refreshed head stays green.** A refresh creates a head that does not exist yet; no pre-refresh
  read can report its result.
- **Per-car CI wall-clock after refresh.** The Actions pool is billing-locked, which is why the timings observed
  on these heads are unrepresentative.
- **Semantic interaction between cars.** Change-level conflict is measured above; semantic conflict is not
  derivable from check-run data and is not claimed.

## Reads behind this document

- `pulls?state=open&per_page=100` (42 returned, no cap hit) and `commits/main` -> `ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad`.
- `compare/ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad...<head>` per open PR for `behind_by`/`ahead_by` (42 calls).
- `pulls/<n>` per window car for `mergeable`, `mergeable_state`, additions/deletions/changed_files (25 calls).
- `commits/<head>/check-runs?filter=latest&per_page=100` per window car, paginated to `total_count` (25 calls).
- `branches/main/protection/required_status_checks` for the required context list.
- `check-runs/<id>/annotations` for every red required context (5 annotations across 4 cars).
- `git merge-tree --write-tree --name-only origin/main <head>` for the two `dirty` cars, after fetching exactly
  those two heads.
- `compare/.../files` paginated per window car for the rule-8 gated-path count.

GATE: local pass (decision-signal) - measured the band that A23 named but never measured, corrected four cars to
twenty-five, and separated the red that a refresh cures from the red it cannot.
