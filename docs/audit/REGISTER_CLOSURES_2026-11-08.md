# Assumption register: four rows read to a verdict, 2026-11-08

Status: filed
Scope: LANE PROTOCOL clause 12 (assumption register). Four of the seven overdue rows closed by a read
taken in this session; three remain open and are recorded as such, not worked around.

Before this pass the register read 30 rows with **7 overdue** (A3, A5, A7, A8, A9, A20, A26), which is a
standing blocker on every lane status comment. Every one of the seven was listed as owner- or
reviewer-held. Four of them were not: their own `Verification method` cell specifies a read this lane can
perform, and the overdue flag was an artefact of nobody having performed it. Those four were performed.

| row | before | verdict now | what closed it |
|---|---|---|---|
| A3 | OVERDUE | **CLOSED** - classification verified, cure not claimed | every failure on all four C1 car heads is billing residue |
| A5 | OVERDUE | **FALSIFIED IN PART** - holds for 2 of 3 cars | a required failure survives the refreshed #507 head |
| A20 | OVERDUE | **CLOSED** - `indefinitely` refuted | the #599 base now carries main's `ci.yml` byte-for-byte |
| A26 | OVERDUE | **RE-CONFIRMED** - split stands | main carries `CI_SMALL_RUNNER` zero times |
| A7, A8, A9 | OVERDUE | **still open, unchanged** | they need a non-author reviewer, which does not exist (A16) |

Register after this pass: 30 rows, **3 overdue** (A7, A8, A9). No row was re-dated; re-dating is the owner's
move, and none of these rows is being quietly extended.

## A3 - C1 cars: the classification holds, the cure was never run

The row asserts the Batch D car failures are hosted-pool refusals and stale-base effects rather than code
defects. Its verification method is explicit: a failure line that is neither the billing annotation nor a
stale-base conflict falsifies the row.

| head | check-runs | failures | of which billing | required contexts |
|---|---|---|---|---|
| #550 `6441203a1617` | 46 | 0 | - | 5/5 success |
| #554 `5b3efc47de11` | 47 | 0 | - | 5/5 success |
| #556 `9282583a93e4` | 57 | **3** | **0** | 5/5 success - see the correction |
| #660 `210b7d932f4e` | 29 | 13 | **13** | 5/5 success |

44 billing-annotated failures and **3 non-billing** failures across the four heads. All three sit on #556,
are Windows-only jobs (`Test (windows-latest)` x2, `Windows CLI Artifact`) dated 2026-10-22, and carry the
Node.js 20 deprecation notice as their annotation: environmental, non-required, and unrunnable on the Linux
OCI pool in any case. No code defect, so nothing falsifies the row - but this section originally claimed
"zero non-billing failures", which was wrong. The correction is below.

Two precisions that must not be lost, because both would otherwise be reported as green:

1. **#556 reads 5/5 required success, and my first read of it was wrong.** The initial pass reported that
   `Repository Hygiene Checks` never ran on that head. It did. The read used
   `commits/<sha>/check-runs` without a page size, the endpoint returns 30 by default, and `#556` has 57
   check-runs, so the context was on a page that was never fetched. See the correction below.
2. **The cure half is not executed.** #550, #554 and #556 still read `BEHIND` and #660 reads `UNSTABLE`.
   No refresh and no re-run was performed, so the row's second clause ("a refresh plus a re-run on the
   OCI pool cures them") remains UNPROVEN here. The merge frontier is owner-held and this lane did not
   mutate another session's PR branches to manufacture the proof.

## A5 - C4 cars: the refresh cured two cars and provably cannot cure the third

The row asserts the Batch C red is `BEHIND`-only and is cured by `update-branch`, with its own falsifier:
a failure that survives the refreshed head.

| car | refreshed head | required contexts | failures | merge |
|---|---|---|---|---|
| #506 | `16b4edf7540b` | 5/5 success | 11, all billing | UNSTABLE |
| #507 | `f4d5155d4af5` | **4/5** - `Test (ubuntu-latest)` failure | 11, all billing | **BLOCKED** |
| #508 | `475816012256` | 5/5 success | 11, all billing | UNSTABLE |

#506 and #508 are exactly what the row predicted. #507 is not, and the reason is already on file as A25:
its diff is Kotlin-only, so the rust-routing guard sends the required leg to hosted `ubuntu-latest`, where
the account lock refuses it - which is why this failure carries the billing annotation even though the
code is fine. Refreshing again reproduces it by design. The cure is SR-1, or a deliberate change to the
#642 inert-diff contract, and that decision is the owner's.

So A5 is not a lane toil item that stayed undone; it is a row whose general claim is false for one of its
three subjects.

## A20 - #599: the base no longer forbids the contexts

The row asserted #599's feature-branch base forbids the required contexts existing, indefinitely.
Verification method: read the base's own workflow content.

- Base `feat/conn-admission-dynamic` tip: `5c6b5e52ed296edb7b08d04477a621d0e552185f`.
- That ref's `.github/workflows/ci.yml` blob: `ed6ca3b6ca60c054882e645ad134a7552125b517`, **identical to
  main's blob of the same path**, carrying `USE_OCI_RUNNER` 8 times.
- #599 head `a7ef2f299753` still has exactly **3** runs, all created `2026-10-24T01:55:36Z` (`Mobile`,
  `iOS Build & Test`, `Auto Label`, all success) - i.e. all from before the base advanced.

`indefinitely` is therefore refuted: the base now carries main's routing, so the next run can produce the
required contexts. Producing them requires a push or a re-base, and re-targeting is the owner's decision in
A17. The PR reads `merge=CLEAN`.

## A26 - #643: still two changes, not a refresh

Verification method: run the branch's own contract script against a candidate resolution; a script that
rejects main's routing means the PR is two changes.

- #643 head `0340c094468f`, `mergeable_state=dirty`.
- Its `scripts/ci_pool_route.py` (blob `c6683f4b8289`) fails any job that "selects a self-hosted pool
  directly" and requires `vars.CI_SMALL_RUNNER ||` on the small architecture-agnostic jobs (Docs, FFI
  Surface among them).
- main's `ci.yml` carries `CI_SMALL_RUNNER` **zero** times (`USE_OCI_RUNNER` 8 times).

Adopting #643's routing cannot therefore be a refresh of main: it replaces the #661 rollback switch. The
split already recorded on 2026-11-05 stands - the dependabot carve-outs are landable as one PR (artifact
`docs/audit/ci_resolution_2026-11-05/643-guarded.patch`), the pool re-scope belongs to the CI owner. This
read verifies the row; it neither re-dates it nor takes the CI owner's disposition.

## What is still owed, and by whom

A7, A8 and A9 all reduce to one missing actor: LANE PROTOCOL clause 10 requires two independent non-author
Jev PASSes, and A16 established that this repository has exactly one collaborator, so no non-author
reviewer exists to produce them. Those three rows cannot be closed by any read this lane can perform. The
unblocking input is an owner decision - name and grant a reviewer, or amend the gate for this repo.

No billing setting, credential, branch protection setting or merge was touched in this pass.

## Correction, same session: the check-runs page cap

Everything above about required contexts was re-read after a contradiction appeared. Recorded because the
error is instructive and because the earlier numbers were posted to #653 and mirrored to #602.

`GET /repos/{owner}/{repo}/commits/{sha}/check-runs` returns **30 items by default**. Passing no `per_page`
therefore prints a ceiling, not a count, and four of the seven heads I read had `total_count` **above** 30:
`#550` 46, `#554` 47, `#556` 57. Any "N check-runs" line where N equals exactly 30 in the earlier text of
this file or in the register should be read as "page 1 of at least 30".

What that changed:

| claim as first posted | corrected |
|---|---|
| `#550` 30 check-runs | **46** |
| `#554` 30 check-runs | **47** |
| `#556` 30 check-runs, 0 failures | **57** check-runs, **3 failures** |
| `#556` has only 4 of 5 required contexts; `Repository Hygiene Checks` never ran | **false** - it ran and is `success`; all **5 of 5** required contexts are present and green |
| "zero non-billing failures across all four heads" | **3 non-billing failures**, all environmental (Node.js 20 deprecation annotations on Windows-only jobs, 2026-10-22) |
| A5's car readings (27 each) | unchanged - 27 is below the cap, so those were real counts |

What did not change: A20, A26 and the A5 verdict (#507's surviving required failure is billing-annotated and
the PR is `BLOCKED`, so the refresh still cannot cure it). The register's row count and the overdue set
(A7, A8, A9) are also unaffected.

The lesson is the one rule 15 already states: a value sitting exactly on a known API page size is a ceiling
until pagination proves otherwise. The contradiction that exposed it was mechanical - `#556`'s own workflow
run list showed a `Repository Hygiene` workflow that had run, which cannot coexist with "the context never
ran".
