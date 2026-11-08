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
| #550 `6441203a1617` | 30 | 0 | - | 5/5 success |
| #554 `5b3efc47de11` | 30 | 0 | - | 5/5 success |
| #556 `9282583a93e4` | 30 | 0 | - | **4/5** - see below |
| #660 `210b7d932f4e` | 29 | 13 | **13** | 5/5 success |

Zero non-billing failures across all four heads, so nothing falsifies the row and it closes on its own
method.

Two precisions that must not be lost, because both would otherwise be reported as green:

1. **#556 does not read 5/5.** It has zero failures but only **four** of the five required contexts -
   `Repository Hygiene Checks` never ran on that head - plus one cancelled non-required
   `Forgery/Negative Security Regression Tests`. "Zero failures" and "5 of 5 green" are different facts,
   and the earlier register text recorded the second for this head.
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
