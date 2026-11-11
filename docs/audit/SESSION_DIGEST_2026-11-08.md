# Session digest - 2026-11-08 (Freebuff lane, `lane/burndown`)

Fixed schema. Every field is either filled from a read taken in this session or marked
`UNVERIFIED`; no field is inherited from an earlier status.

Schema: `1. SEAT` / `2. PROGRAM` / `3. READS` / `4. BOARD ROWS` / `5. GATE RECEIPT` /
`6. NEXT ACTION`.

## 1. SEAT

| Field | Value | Read |
|---|---|---|
| Branch | `lane/burndown` | `git rev-parse --abbrev-ref HEAD` |
| HEAD | `8125faa63b8fa169305197285e82041ef3766e7c` | `git rev-parse HEAD` |
| Push parity | `HEAD` == `origin/lane/burndown`; 39 ahead / 0 behind `origin/main` (`ba7fa778`) | `git rev-parse HEAD origin/lane/burndown`, `git rev-list --left-right --count origin/main...HEAD` |
| Working tree | `M scripts/harness_admission.json` (unowned, stat-only CRLF flag from a prior session - not touched); four untracked evidence archives (not touched) | `git status --short --branch` |
| Disk | `disk_budget.py` -> FAIL, below the hard floor | `python scripts/disk_budget.py` |
| Consequence | No `cargo`/`gradle` build on this host this session (rule 17). CI is the builder. | - |

## 2. PROGRAM

### (1) The pending `Test (ubuntu-latest)` verdict at `b3453a188` - CLOSED, GREEN

Read at `b3453a1887032b5d11cd9f3b0565495a2109ca00`
(read: `commits/b3453a188.../status`, 2026-11-08T04:00Z): combined state `success`,
`total_count` **5**, 5 success, 0 non-success.

| context | state | concluded |
|---|---|---|
| `ci/circleci: Handoff ownership scope` | success | 2026-11-07T03:39:34Z |
| `ci/circleci: Repository Hygiene Checks` | success | 2026-11-07T03:39:53Z |
| `ci/circleci: Rust Linting` | success | 2026-11-07T03:40:03Z |
| `ci/circleci: Lint` | success | 2026-11-07T03:44:59Z |
| `ci/circleci: Test (ubuntu-latest)` | success | 2026-11-07T04:05:22Z |

The long leg is read to terminal, so the standing 4-of-5 disposition is discharged: wave 1 is
**5 of 5**. Note the brief's premise ("pending verdict at `b3453a188`") is stale by five commits -
the lane has since advanced to `8125faa63` - but the verdict is verified rather than assumed, and
the later heads carry it.

### (2) The approved wave-2 port - ALREADY ON THE BRANCH, and green

The success branch of the brief condition ("only if successful, start the approved wave-2 port")
required no new work: the port landed earlier at `d1d6c1025`, was hardened at `1b1308545` and its
FFI red was cured at `2dffad301`.

Read at HEAD: `.circleci/config.yml` blob `08631fb5` defines **11 jobs**
(`repository_hygiene_checks`, `lint`, `rust_linting`, `test_ubuntu`, `handoff_ownership_scope`,
`orchestration_control_plane`, `docs_build`, `ffi_surface_contract`, `forgery_regression`,
`kotlin_linting`, `javascript_linting`) plus the `ci` workflow, and is
**+296 / -3 lines** against `b3453a188`.

Battery, read at two heads (read: `commits/<sha>/status`):

| head | `ci/circleci:*` total | state |
|---|---|---|
| `2dffad301` | 11 | success |
| `8125faa63` | 11 | success (incl. `Test (ubuntu-latest)` 07:53:44Z) |

Two of the eleven are `[NON-ENFORCING]` by construction and are not offered as enforcement:
`Kotlin Linting` (`ktlint { ignoreFailures = true }` - green means "ktlint ran") and
`JavaScript Linting` (no `wasm/package.json` target). Neither may enter branch protection.

### (3) Clause (f) - discharged on this exact head from fresh annotations

20 Actions check-runs on `8125faa63`. The five bare required names read `success`
(`Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`,
`Handoff ownership scope`); the eight non-required failures each carry
`The job was not started because your account is locked due to a billing issue`:

`Windows CLI Artifact` `128341013351`, `Test (macos-latest)` `128341013340`,
`Test (windows-latest)` `128341013291`, `Kotlin Linting` `128340946038`,
`Swift Linting` `128340945963`, `JavaScript Linting` `128340717917`,
`WASM` `128340717460`, `label` `128340717280`
(read: `check-runs/<id>/annotations`, 2026-11-08T04:03Z).

A job that never started is not a defect in the change.

## 3. READS

| Read | Method | When |
|---|---|---|
| Verdict at `b3453a188`, `2dffad301`, `8125faa63` | `commits/<sha>/status` (CircleCI-published contexts) | 2026-11-08T04:00Z |
| Required-context runner and step execution at `8125faa63` | `actions/runs/<id>/jobs` | 2026-11-08T04:05Z |
| Failing non-required annotations at `8125faa63` | `check-runs/<id>/annotations` | 2026-11-08T04:03Z |
| Branch protection on `main` | `branches/main/protection/required_status_checks` | 2026-11-08T04:01Z |
| #489 held state | `gh pr view 489` | 2026-11-08T04:01Z |
| #569 / #570 / #572 closure causes | `issues/<n>/timeline`, `git log --grep` | 2026-11-08T04:02Z |
| #595 checkbox state | `gh issue view 595` | 2026-11-08T04:02Z |
| Assumption register | `HANDOFF/plans/V051_V052_TOTAL_BURNDOWN_PLAN.md` section "Assumption register" | 2026-11-08T04:03Z |

`ci/circleci:*` contexts are CircleCI's own commit statuses read through the GitHub API. They are
not Actions logs and not the combined status of an unrelated rail; the Actions reads above are used
only for pool routing and annotation classification, never as a code verdict.

## 4. BOARD ROWS

**DONE**

- Pending verdict at `b3453a188` closed green (5 of 5, `Test (ubuntu-latest)` 04:05:22Z).
- Wave-2 port confirmed present and green at HEAD (11 of 11 at two independent heads).
- Clause (f) discharged on fresh annotations for all eight non-required failures.
- #595 reconciled as ordered: the three stale checkboxes now carry their real closure causes
  (`#569` Blacksmith rotation, 2026-10-14; `#570` `c2bbb2b7c` wasm test-build gating, 2026-10-22;
  `#572` `proxy-addr` 2.0.8 pinned, verified on main `c71de9d3adef`, 2026-10-26). Unchecked rows
  go from 15 to 12.
- Assumption register row A2 closed: all five required contexts on `8125faa63` ran on
  `oci-a1-phx-1` with executed steps (21/21, 13/13, 9/9, 16/16, 9/9), which is the PR-event-head
  re-read A2 asked for.

**WIP** - none. Nothing is half-written.

**NEXT**

- Wave 3 if the port stays open: `naming-policy` is the last cheap real gate not yet on the
  CircleCI rail (it already reads `success` on Actions, so porting it would give it an admissible
  verdict).
- The merge frontier, which is the only motion left and is owner-held (see BLOCKERS).

**BLOCKERS**

1. **Protection mismatch - DECISION-READY, owner-held.** Read now
   (`branches/main/protection/required_status_checks`, 2026-11-08T04:01Z): `strict: true`, and the
   five required contexts are the bare names `Repository Hygiene Checks`, `Lint`, `Rust Linting`,
   `Test (ubuntu-latest)`, `Handoff ownership scope`, all with `app_id: 15368` (the GitHub Actions
   app). The rail publishes `ci/circleci: <job>`. The two sets do not intersect, so a green
   CircleCI battery satisfies zero required contexts. Exact setting to change it, packet at
   `docs/audit/PROTECTION_SWITCH_DECISION_READY_2026-11-07.md` (filed at `b3453a188`):

   ```
   PATCH /repos/{owner}/{repo}/branches/main/protection/required_status_checks
   {
     "strict": true,
     "contexts": [
       "ci/circleci: Repository Hygiene Checks",
       "ci/circleci: Lint",
       "ci/circleci: Rust Linting",
       "ci/circleci: Test (ubuntu-latest)",
       "ci/circleci: Handoff ownership scope"
     ]
   }
   ```

   Rollback is the same PATCH with the five bare names. Consequence to accept deliberately:
   protected landings become dependent on CircleCI availability, and the self-hosted OCI pool no
   longer gates a merge. The lane performs no protection change, no force-push and no admin merge.
2. **The ordering contradiction** (unchanged): every car head carries 0 `ci/circleci:*` contexts
   because `.circleci/config.yml` exists only on `lane/burndown`, so "merge each car on a green
   battery at its head" and "#653 lands last" cannot both hold. Recommended resolution, stated as a
   decision: land the port first.
3. **Clause 10 is unsatisfiable structurally.** The repo has one collaborator (`Treystu`), so two
   independent non-author PASSes cannot be recorded (A16). A named second reviewer is an owner
   input.

**DIRECTIVES**

- Newest owner steering (2026-11-08T03:38:28Z) asked for: `Test (ubuntu-latest)` read to terminal,
  the full wave-1 battery verdict, no wave-2 start before checks 1-5 clean, status + head + next
  action + push ETA. The first three are answered above; wave 1 is 5 of 5 and wave 2 was already
  green before that instruction arrived.
- `#489` is preserved green-and-held: read only (`OPEN`, non-draft, head `fc3540e481bc`,
  `mergeStateStatus=DIRTY`, `mergeable=CONFLICTING`, 0 reviews). Not touched, not merged, not
  closed.
- No force merges performed or attempted.

**ASSUMPTION REGISTER** - 30 rows. Every row's `Verify by` date is on or before 2026-11-07, so the
prior "30 rows, 0 overdue" line cannot stand unqualified on 2026-11-08: by the register's own rule
a row past its date that is not closed is OVERDUE. Accounting this session: 22 rows carry a
terminal verification read (VERIFIED / CONFIRMED / REFUTED / WITHDRAWN / DISCHARGED), **A2 closed
by this session's read**, and **7 remain open and therefore OVERDUE**: A3, A5, A7, A8, A9, A20,
A26. Of those 7, six are held by an owner or a non-author reviewer (A7, A8, A9, A26 and the owner
halves of A3/A5/A20); none is a lane-side task that this session could have closed.

**CONFORMANCE** - `b3453a188` battery accepted on record (5 of 5, ids above). `2dffad301` accepted
on record (11 of 11). `8125faa63` accepted on record (11 of 11), and its required contexts are
additionally confirmed as executing on `oci-a1-phx-1`. The wave-2 port is accepted on record as
present at HEAD, not merely described. #595 reconciliation is accepted on record (read back after
the edit: 12 unchecked, three checked rows carrying causes). No PR state changed, nothing merged,
protection untouched.

## 5. GATE RECEIPT

GATE: local pass (decision-signal) - close the wave-1 verdict on a primary read and take the
next unblocked lane item instead of re-deriving owner-held items - checks 1-5 clean

Head-bound form required by the owner: receipt attached to
`8125faa63b8fa169305197285e82041ef3766e7c`.

## 6. NEXT ACTION

Single unblocked item, in order:

1. Port `naming-policy` to the CircleCI rail as wave 3, and print its enforcement limit honestly
   the way the other jobs do - unless the owner answers the protection switch first, in which case
   the port lands instead and this item waits. **STARTED AND PUSHED this session - see section 7.**
2. If the protection decision arrives: land the port under the owner's chosen option, then
   re-read the required contexts on the new main.

## 7. WAVE 3 - the naming-policy ratchet, ported and pushed (after the seat read in section 1)

The seat table in section 1 records the head the session's reads were taken at. The lane tip has
since advanced by this session's two commits:

| commit | subject |
|---|---|
| `02ce6fdda` | `docs(audit): close the wave-1 verdict at 5 of 5 and reconcile the register` |
| **`68e6f6b4a`** | `ci(circleci): port the naming-policy ratchet as wave 3, enforcing on both steps` |

`naming_policy` is the 12th job on the rail, ported from `.github/workflows/ci.yml`'s
`naming-policy`. It is deliberately **ENFORCING on both steps**, unlike the two wave-2 contexts:

- `scripts/check_naming.py --changed-from "$GATE_BASE"` fails on a denied name added by the change.
  It is a ratchet, so the tree's pre-existing 57 `isRelay`, 38 `delete_*` and 165 `lastSeen` stay the
  audit's backlog and are never re-litigated here.
- `--self-test` proves on every build that the matcher and the policy validation can still deny
  something, because a gate that has stopped denying anything looks exactly like a gate with
  nothing to deny.

Local verification actually run (no build; disk is below the hard floor):

| Check | Result |
|---|---|
| `.circleci/config.yml` parsed under a duplicate-key loader | `[OK]` 12 jobs, 12 workflow entries, **no dangling job ids**, `naming_policy` present in both |
| `python scripts/check_naming.py --repo-root . --self-test` | `[OK] self-test passed: 13 scan cases + 8 policy-validation cases`, exit 0 |
| `python scripts/check_naming.py --staged` | `[OK]` 0 files in scope, no denied name added, exit 0 |

One limit stated plainly: the config's own gate scopes to source files, so the `--staged` run above
is a trivial pass on a YAML file and proves only that the gate does not miscount; the self-test is
the evidence that the matcher works, and the first real behavioural verdict for this job belongs to
the `68e6f6b4a` battery on the rail.

The `68e6f6b4a` battery is **running and is reported as it reads, not asserted**. No merge, no
protection change, no force-push was performed this session.

GATE: local pass (decision-signal) - port the last cheap real gate with both steps enforcing, rather
than shipping an eleventh context that cannot fail - checks 1-5 clean (head
`68e6f6b4a8ab39ea3a21f892357684e7fa3d1ee3`)

## SUPERSEDED-BY (appended 2026-11-08, later the same day)

This digest's NEXT ACTION said to port `naming-policy` as wave 3. That is **done**: `68e6f6b4a`, green on the
rail at `b7e59787f` (`ci/circleci: Naming policy` success `04:09:13Z`). A resume from this file would redo
finished work, which is the specific failure this note prevents.

State now, so the next resume point is unambiguous:

| item | state |
|---|---|
| wave-3 `naming-policy` port | **DONE** - `68e6f6b4a`, 12/12 green at `b7e59787f` |
| register overdue rows | **7 -> 3** - A3/A5/A20/A26 read to a verdict in `06ea1e4c7`, then A3's numbers corrected in `a1c2d6cd6` |
| #643 carve-outs | **DONE** - `7bc5e0250`, guard-preserving half only |
| C1/C4 car table | **REFRESHED** - `9f4a62223`, section 3 of `CLUSTER_DISPOSITIONS_2026-11-07.md` |
| still overdue | **A7, A8, A9** - all need a named, granted non-author reviewer |

Records added since this file: `REGISTER_CLOSURES_2026-11-08.md`, `CI_CARVEOUTS_LANDED_2026-11-08.md`,
`SESSION_DIGEST_2026-11-08_PM.md`.
