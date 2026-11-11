# Coverage gap: the 15 open issues the plan's 82-row crosswalk never covered

Date: 2026-11-08 (session 16:40Z-17:35Z)
Scope: the annex (comment 6723992714) is the coverage baseline - 97 open issues
plus 43 open PRs at filing. The plan of record's "Open-issue crosswalk" carries
82 issue rows. This record accounts for the 15 rows that crosswalk does not
cover, so the annex has no silent gap.

Reads: `issues/<n>` per row (state, labels, comments, `updated_at`, body) at
2026-11-08T17:03Z. No row below was classified from its title.

## The set, and why it is 15

Annex minus crosswalk, computed with `comm` over the two number sets:
#650, #651, #658, #665, #666, #667, #668, #669, #670, #671, #672, #673, #674,
#675, #676. Nothing in the crosswalk has closed since it was written, so the
gap is purely additions after that record.

| Issue | State read | Disposition | Owner / next decision |
|---|---|---|---|
| #651 `scm test` fails with "Error: Consent required" | open, 0 comments, filed 2026-10-29 with a live repro on main `347127ae` | **WORKED THIS SESSION** - root cause is a missing `eve.grant_consent()` in `cmd_test`; fix + regression test on PR #678 (`b91a6634b`) | PR #678's battery; issue closes on that merge |
| #650 CLI control API port hard-coded to 9876 | open, 0 comments, same audit, three numbered live symptoms | **DECISION-READY (lane-repairable, not started)** - the defect is `cli/src/api.rs:23-24` (`API_PORT`/`API_ADDR` constants) with no per-instance override and no endpoint resolution from `SCMESSENGER_DATA_DIR` | needs a scoped packet: configurable port + instance-affine CLI resolution + an explicit collision error; the acceptance is a two-node same-host run, so it is a larger slice than an hour |
| #658 507 stale branches (2026-11-01) | open, 4 comments, `updated_at` 2026-11-08T04:07Z, label `branch-hygiene` | **BLOCKED - operator only** | deleting branches is a destructive operation under AGENTS.md rule 12; the lane may survey and propose, never delete. The 4-comment thread already carries the survey |
| #665 Mesh mechanism audit program (master tracker) | open, 0 comments, labels `enhancement,docs`, filed 2026-11-07 | **DECISION-READY (owner)** - program tracker, not a lane work item | needs an evaluation owner and a sequencing decision; the 11 children below hang off it |
| #666 Design-time delivery-probability estimator + rare-event discipline | open, 0 comments, labels `enhancement,rust,tests` | **DECISION-READY (owner)** - prerequisite for #667-#676 | measurement design first: the estimator does not exist, and every child row measures a mechanism against it |
| #667 Circuit selection: joint-objective bake-off vs greedy | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666: state the decision, replay the greedy champion, record the delta |
| #668 Routing engine scoring: calibrate `reliability_score`, cascade confidences, layer priority | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #669 Gateway ranking: measure `gateway_cost` regret | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #670 Multipath: no disjointness today; measure union delivery under correlated failure | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666; the issue itself states the current state (no disjointness), which is a finding, not a defect report |
| #671 Drift sync: size IBLT sketches to the difference; retry on decode failure | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #672 Routing decision parameters: TTL tiers, 500 ms budget phases, dual retry paths, negative-cache TTL | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #673 Cover traffic and gossip: build the privacy/overhead measurement that does not exist | open, 0 comments, labels add `crypto` | **DECISION-READY (owner)** - depends on #666 | after #666; crosses a rule-8 path, so any landing needs the adversarial review |
| #674 Dial policy: replay the ladder+backoff champion against ranked/bandit challengers | open, 0 comments, label `transport` | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #675 Discovery scheduling: fixed cadences vs yield-adaptive probing | open, 0 comments, label `transport` | **DECISION-READY (owner)** - depends on #666 | after #666 |
| #676 Reputation and abuse scoring: scripted-adversary calibration | open, 0 comments | **DECISION-READY (owner)** - depends on #666 | after #666 |

## What this changes

1. The annex now has a disposition for every one of its 97 issue rows: 82 in
   the plan's crosswalk, 15 here. One of the 15 (#651) moved from "unknown" to
   a fix on a PR head in this session.
2. The audit family is one programme, not eleven independent tasks. #666 is its
   prerequisite, so #667-#676 cannot be dispatched in parallel before it - a
   scheduling fact the annex's flat list hides.
3. #650 is the largest lane-repairable row in the gap: a real cross-instance
   identity hazard (a command can act on a different node than the operator
   selected while reporting success), with a known file and a two-node
   acceptance test. It is the natural next C1 add after the port lands.
4. #658 cannot progress in this lane at all. It is a deletion task, and
   deletions are operator-only here.

## What this does not do

It closes nothing, changes no label, and assigns no owner who has not been
named by the plan of record. It reads issue metadata and bodies only; no code
was changed for any row except #651, which is on #678.
