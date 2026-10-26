# V051 — Landing: cancel the dependabot waves, then merge A3

Status: ACTIVE (issued 2026-10-23T13:35Z)
Priority: P0 — the train is stopped by queue, not by a car
Lane: Landing (sole merge authority)
Authority: ov9 sections 2 and 3; plan v7 (merge gate, one merge at a time)
Writer: Watchdog (read-only tab); this file is the durable copy of #602 entries

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Board at issue time (read live, not from a checkpoint)

| item | value |
| --- | --- |
| `main` | `ec25a3ea6` — MERGED A4 (#641), 46 success / 2 skipped / 0 failing / 0 pending, terminal 12:49:22Z |
| #636 A3 | head `381eaca9f236a0de392869ce636d15a9cb7e775c` — 21 pass / 0 fail / 5 pending |
| #637 | `4a8e0cf04`, BEHIND, 20 pass / 5 skip (pre-advisory, earned 08:41Z) |
| #639 | `be6dfaeaf`, BEHIND, 26 pass / 6 skip (pre-advisory) |
| #599 | `aab17e5b0`, 6 pass / 2 pending, Rule-8 SIGN-OFF at this head |
| `oci-a1-phx-1` | held by #634 `Forgery` (dependabot dirs 7.0.0) since ~12:49Z |
| `oci-a1-phx-2` | held by #630 `Forgery` (dependabot zbus) since 13:12:06Z |

#636's five pending checks: `Test (ubuntu-latest)` (job `120419810738`, created
12:42:42Z, **25 jobs ahead** in a 26-deep OCI queue), `Docs`, `FFI Surface Contract`
(same queue), `Test (macos-latest)` and `Swift Linting` (macOS queue).
Not one of them is failing. The car is ready; the pool is not.

## ACTION 1 — cancel the non-car waves (ov9 s2/s3)

There is no "reason" field on a run cancel; the justification is the journal
line below. The filter is safe by construction: #636, #599 and main are not on
`dependabot/` branches, so they cannot be selected.

```
gh api "repos/Sovereign-Communication/SCMessenger/actions/runs?per_page=100" \
  --jq '.workflow_runs[] | select(.head_branch|startswith("dependabot/"))
        | select(.status=="queued" or .status=="in_progress") | .id' \
  | xargs -n1 gh run cancel
```

Expected scale at issue time: **28 live dependabot runs** across #630, #631,
#632, #633, #634, #635, each running CI + Lint + Mobile + Security + iOS.
They hold 23-24 queued OCI jobs and both runners.

Verify (all three, inside 5 minutes):

1. `gh api repos/Sovereign-Communication/SCMessenger/actions/runners --jq '.runners[]|.name+" busy="+(.busy|tostring)'`
   Expect both `busy=false` within ~1 minute, because both current holders are
   dependabot jobs.
2. Re-run the queue measurement: expect **3 or fewer** OCI jobs
   (#636's `Docs`, `FFI Surface Contract`, `Test (ubuntu-latest)`), against 26 now.
3. `gh api repos/Sovereign-Communication/SCMessenger/actions/jobs/120419810738 --jq '{status,created_at,started_at,runner_name}'`
   `started_at - created_at` is the wait #636's ubuntu leg incurred (47+ minutes at
   issue time). After the cancel it should start within ~13-20 minutes.

ABORT: if after 10 minutes either runner still shows a dependabot job, or the OCI
queue is still above 8, the cancellation has not done its job. Stop there, post
the two runner lines and the queue depth, and escalate. Do not cancel anything
else, and do not cancel #599 or #636 to "free" a runner.

JOURNAL (post to #602 once, after verification):

```
[LANDING] CANCELLED <n> dependabot runs per ov9 s2/s3. They held <m> queued +
<k> running OCI jobs ahead of #636's required ubuntu leg (created 12:42:42Z,
25th of 26). Excluded main ec25a3ea6, #636 (incl. its Forgery), #599.
After: phx-1 <state>, phx-2 <state>, OCI queue 26 -> <n>; ubuntu wait <x>m.
```

## ACTION 2 — merge A3 on exact-head green

Preconditions, all of them:

- `main` is still `ec25a3ea6` (no other merge happened);
- `gh pr checks 636` reports **0 pending and 0 failing** at
  `381eaca9f236a0de392869ce636d15a9cb7e775c`; the five required contexts are
  `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`,
  `Handoff ownership scope`;
- `gh pr view 636 --json mergeable,mergeStateStatus` shows `MERGEABLE` / `CLEAN`;
- the diff is still exactly two files, `docker/Dockerfile.rust-test` (+6/-0) and
  `docker/docker-compose.test.yml` (+1/-0).

```
gh pr merge 636 --repo Sovereign-Communication/SCMessenger \
  --merge --match-head-commit 381eaca9f236a0de392869ce636d15a9cb7e775c
```

Method: `--merge` (the same method #641 used; if you choose squash instead, say
so explicitly in the journal line, because A1/A2/B are already squashed and the
revert command differs).

If `Test (macos-latest)` and `Swift Linting` are still queued when the required
five are green: they cannot observe a docker-only diff, but they do execute
(fail-open). Do not merge on a red, and do not cancel them. If they are still
unstarted 60 minutes after the required five are green, stop and escalate rather
than merging on a partial wave.

Post to #602: the merge SHA, the new `main` head, and the tally at merge.

## ACTION 3 — what follows, one at a time

1. After #636 merges, ask the Implementer for **one** `update-branch` of #637
   (`4a8e0cf04`) onto the new main. Shape must stay 2 files / +226/-0.
   Merge only on its own exact-head green.
2. Then the same for #639 (`be6dfaeaf`), and `Android JVM Unit Tests` must
   EXECUTE on the refreshed head - a skip is not a pass for that car.
3. Never two cars refreshed ahead of the one in flight. Never merge on a green
   earned at a different head. Never merge a pre-advisory green.

## NEVER

- Merge #599 (base is `feat/conn-admission-dynamic`; it moves only inside batch E).
- Merge #521, #637 or #639 before #636 lands.
- Refresh #636 again. Its single refresh is spent and Lint is verifiably green
  on `381eaca9f` (job `120419810677`, `cargo deny check` and lockfile steps both
  success).
- Cancel main's runs, #636's `Forgery`, or #599's macOS legs.
- Touch any branch. Branch writes belong to the Implementer.

## Note for the operator

`HANDOFF/freebuff/README.md` tells lanes to run
`python scripts/check_queue_status.py` before a paste wave. That file does not
exist on `main` (`ls scripts/ | grep -i queue` returns nothing at `ec25a3ea6`).
Do not block on it; the README line is stale and should be corrected.
