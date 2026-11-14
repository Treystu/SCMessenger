# Merge frontier, decision-ready: every car re-read 2026-11-08

Status: filed, evidence only - no merge, refresh, label or close was performed to produce it.
Purpose: the frontier is the only motion left in the burndown, and every prior ordering was argued from a
census taken on a different day. This is one read of every candidate, on the heads that exist now.

## Method

- `pulls/<n>` for state, head, base and `mergeable_state`.
- `compare/main...<head>` for `behind_by`.
- `pulls/<n>/reviews?per_page=100` for review records.
- `commits/<head>/check-runs?per_page=100&filter=all` **paginated to `total_count`** for the required set and
  the failure count, with every failure's annotation read individually and classified **billing** (the
  account-lock string) or **other**.
- `commits/<head>/status` for `ci/circleci:*` presence.

## The census

| car | head | mergeable | behind | reviews | required contexts | failures | class |
|---|---|---|---|---|---|---|---|
| #660 | `210b7d932f4e` | unstable | 0 | 0 | **5/5 all success** | 13 | 13 billing, 0 other |
| #550 | `6441203a1617` | behind | 46 | 0 | **5/5 all success** | 0 | - |
| #554 | `5b3efc47de11` | behind | 46 | 1 | **5/5 all success** | 3 | 0 billing, 3 other (Node 20 deprecation, Windows-only) |
| #556 | `9282583a93e4` | behind | 46 | 0 | **5/5 all success** | 3 | 0 billing, 3 other (same class) |
| #506 | `16b4edf7540b` | unstable | 0 | 0 | **5/5 all success** | 11 | 11 billing, 0 other |
| #507 | `f4d5155d4af5` | **blocked** | 0 | 0 | 5/5 present, **`Test (ubuntu-latest)` failure** | 11 | 11 billing, 0 other |
| #508 | `475816012256` | unstable | 0 | 0 | **5/5 all success** | 11 | 11 billing, 0 other |
| #521 | `5c6b5e52ed29` | unstable | 0 | 0 | **5/5 all success** | 14 | 14 billing, 0 other |
| #611 | `e03c1d7df43a` | unstable | 0 | 0 | **5/5 all success** | 8 | 8 billing, 0 other |
| #643 | `0340c094468f` | **dirty** | 34 | 0 | **5/5 all success** | 0 | - |
| #609 | `4ed8168bbf1f` | **dirty** | 46 | 0 | **5/5 all success** | 1 | 0 billing, 1 other (Node 20 deprecation + registry lookup) |
| #599 | `a7ef2f299753` | clean | 63 | 0 | **0/5 - base is a feature branch** | 0 | - |

Two things in that table are new relative to the filed dispositions:

1. **#643 and #609 now read 5/5 required success with zero and one failures respectively.** Neither is red; both
   are blocked purely on conflict (`dirty`). #643's carve-out half has already landed on the lane in
   `7bc5e0250`, so what remains for that PR is the pool re-scope the CI owner already owns.
2. **#554's single review is not a PASS.** It is `Treystu` `COMMENTED` on `c53a3223ffc8`, i.e. the same account
   the lane runs as, and a comment rather than an approval. Clause 10 needs two independent non-author
   PASSes; this is neither independent nor a pass. Nothing about A16 changes.

## The ordering contradiction, now measured rather than asserted

Every candidate above carries **zero `ci/circleci:*` contexts** - `commits/<head>/status` returns
`total_statuses=0` for #660, #521, #611, #643, #609 and #506 alike. That is not a failing battery; it is the
absence of one, because `.circleci/config.yml` exists only on `lane/burndown`.

Consequence, stated plainly: **no car can hold a rail verdict until it is rebuilt on a base that contains the
rail.** "Every car merges on a green battery at its head" and "#653 lands last" cannot both be true. One of
the two has to give, and only the owner can say which.

## Proposed order, with each car's exact precondition

This is a proposal the lane can defend, not a decision. Every row's blocker is named with what would clear it.

1. **#653 (`lane/burndown`)** - the rail carrier. 12/12 green at `5ef5a8667`, 0 behind main. Landing it is what
   makes any car's rail verdict possible at all. Owner precondition: the protection-switch decision, because
   until the required contexts name the rail, a green rail still satisfies zero required checks.
2. **#660** - the C1 combined head. 5/5 required, 13/13 failures billing, 0 reviews, 0 behind. Precondition:
   clause 10 (A16 blocks it) and a rail verdict, which needs row 1 first.
3. **#506, #508** - C4. 5/5 required, billing-only failures, 0 behind, 0 reviews. Same two preconditions.
4. **#521** - C5. 5/5 required, 14/14 billing, 0 behind, 0 reviews. Same, plus the crypto verdict its base
   chain requires.
5. **#611** - the CI repair. 5/5 required, 8/8 billing, 0 behind, 0 reviews. Same.
6. **#643** - conflict-only. `dirty` on `.github/workflows/ci.yml`; the carve-out half is already landed, so
   what remains is the pool-allocation re-scope. Precondition: CI owner decision, not a merge resolution.
7. **#507** - blocked by contract, not by code. Its required leg fails on the **refreshed** head because a
   Kotlin-only diff routes to hosted and the account lock refuses it (A25). Precondition: SR-1 or a deliberate
   change to the #642 inert-diff contract. No re-run changes it.
8. **#550, #554, #556** - close as superseded behind #660 once #660 lands. Their content is clean on every
   readable signal; refreshing them would spend three 30-plus-minute Test runs on content #660 already carries.
9. **#599** - clean but 0/5 required, because its base is `feat/conn-admission-dynamic`. Precondition: #521
   lands, the base re-points at main, and the crypto verdict arrives (A17/A20).
10. **#609** - 5/5 required but `dirty` and 46 behind; its routing points at `ubicloud-standard-2`, which has
    no registered runner, so landing it would strand two required contexts (A11/A27). Precondition: provision
    the runner class and invert the default, or withdraw.

## What this document is not

It is not a merge authorisation, and it changes no protection, context, branch or label. Where it says
"precondition", that precondition is unmet today: rows 1-5 all rest on clause 10's non-author PASSes, which
A16 shows this repository cannot supply, and row 1 additionally rests on the owner's protection decision.
