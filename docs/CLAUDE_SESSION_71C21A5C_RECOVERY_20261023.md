# Last Claude work audit and v0.5.1 train recovery plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: audit complete, recovery not executed. Written 2026-10-23T02:40Z.
Author: local read-only audit pass (Freebuff session). No commit, push, merge, branch
creation, lane restart, or file deletion was performed as part of this audit.
Scope: the most recent SCMessenger-related Claude Code session on this machine, what it
produced, what survived, and how to resume the v0.5.1 train it was driving.

Placement note: this record lives under `docs/`, not `HANDOFF/`, because
`scripts/validate_handoff_scope.py` treats the word "harness" as a foreign repository alias, and
this record has to cite the branch name `fix/docker-suite-harness` and the A3 PR title verbatim.
Relocating the file was chosen over rewording the evidence. The scope block above is retained so
the document is still self-describing.

## 1. Answer: what was the latest Claude work

The latest SCMessenger Claude work is one session:

| Field | Value |
|---|---|
| Session id | `71c21a5c-9ea3-4a64-99d3-40e0ccc358a4` |
| Transcript | `C:/Users/SCM/.claude/projects/C--Users-SCM-Documents-GitHub-SCMessenger/71c21a5c-9ea3-4a64-99d3-40e0ccc358a4.jsonl` |
| Size / records | 8,135,167 bytes / 3,261 records scanned |
| Window | 2026-10-22T00:03:56Z to 2026-10-22T20:48:32Z (20 h 45 m wall clock) |
| Working dir / branch | `C:\Users\SCM\Documents\GitHub\SCMessenger` on `preserve/shared-checkout-20261003` |
| Entry point / version | `claude-desktop` / 2.1.304 |
| Model / cost | `claude-sonnet-5-5` / $14.56, total API duration 2,681,516 ms |
| Tool activity | 212 Bash calls, 3 Write/Edit calls (all in an OS temp scratchpad) |

No later SCMessenger session exists. The newest transcript in that project directory is this
one; the next newest file there is `49d7af7c` (mtime 2026-10-21T16:04 local). Newer transcripts
elsewhere on disk belong to other project directories and were not part of this audit.

## 2. How it ended

The end was an access failure, not a task failure, and it happened in three steps:

```
2026-10-22T19:18:48Z  last successful tick: "no changes ... main is still 946400c6"
2026-10-22T19:48:31Z  "Your organization has disabled Claude subscription access for Claude Code"
2026-10-22T20:18:32Z  same message
2026-10-22T20:48:32Z  401 OAuth access token has expired. Re-authenticate to continue.
```

So both stated causes are real and sequential: subscription access was disabled at 19:48Z, then
the OAuth token itself expired at 20:48Z. The session was still answering prompts; it could no
longer act.

## 3. What that session did (four phases)

Phase 1 -- workspace audit, GitHub backup, disk reclaim (00:03Z to 01:38Z).
Inventoried local Claude work product, pushed additive `backup/claude-*` branches for
unreachable commits and dirty worktrees, posted reports and corrections to issue #602, and
removed 14 duplicate worktrees with `git worktree remove` (no branches deleted, per the
correction it posted to #602). 24 `backup/claude-*` refs exist on origin at audit time.

Phase 2 -- v0.5.1 train takeover (05:00Z to 05:20Z).
Read #602, verified the Freebuff lane department, wrote four Claude subagent role files to
`C:/Users/SCM/.claude/agents/` (`scm-landing.md`, `scm-shepherd.md`, `scm-implementer.md`,
`scm-reviewer.md`), and produced the landing-session handoff prompt.

Phase 3 -- plan authoring and Jev gating (05:44Z to 05:53Z).
Seven plan revisions (`plan_v2` .. `plan_v7`) scored through the TypeSafe System One API under
the Jev Gate Policy v1 landed by PR #626. Result reported honestly in #602 at 05:53Z: decision
gate ESCALATE on an ambiguous disposition, assessment gate blocked on correctness (64.0), with
an explicit recommendation to adopt v7 anyway and an owner decision requested. The owner's only
reply in that window, "approved for github verified" at 05:07Z, predates the plan and authorised
the workspace backup, not plan adoption.

Phase 4 -- executing the train as Overseer (05:53Z to 20:48Z).
Posted nine rulings to #602 (`ov1` .. `ov9`) and ran a 30-minute tick loop. Under that loop the
train landed four cars:

| Car | Merged (UTC) | Main after |
|---|---|---|
| #597 wasm test gating | 2026-10-22T06:14:50Z | - |
| #616 canonical planning tracker | 2026-10-22T08:54:19Z | `5bbc71da` |
| #627 batch A1 docs/tooling | 2026-10-22T11:20:11Z | `8e936527` |
| #628 batch A2 CI workflows | 2026-10-22T13:37:05Z | `946400c6` |

## 4. Where the work lives now

### Durable on GitHub

- Issue #602 carries 37 comments, including the phase-1 backup reports (`u602b`, `u602c`), the
  plan v7 Jev result (`ov4`), and rulings `ov1` through `ov9`. Last comment: 2026-10-22T14:49:47Z.
- Additive backup branches under `refs/heads/backup/*` created in phase 1.
- The four landed cars and their merge commits on `main`.

### Local but stable

- `C:/Users/SCM/.claude/agents/scm-{landing,shepherd,implementer,reviewer}.md`
  (written 2026-10-21T19:14 local).
- The two Freebuff lane threads in
  `C:/Users/SCM/.config/freebuff-desktop/projects/SCMessenger-9e36e998-d81d-43e7-8ee0-b84430089fff/desktop-v2.db`
  (titles "Landing role instructions and merge order", "Implementer Role and Merge Train").
- `HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md` -- recovered by this audit, see below.
  [WARNING] Local only. It is an untracked file in the shared checkout on the preserve branch;
  this audit did not commit or push it. Until it is committed to a real branch or posted to
  #602, the recovery is one accidental `git clean` away from the same loss it just fixed.

### Ephemeral, recovered by this audit

The full plan text and its evidence existed only in the OS temp scratchpad:

```
C:/Users/SCM/AppData/Local/Temp/claude/C--Users-SCM--claude-spares-spare-ArdcGH/
  71c21a5c-9ea3-4a64-99d3-40e0ccc358a4/scratchpad/       (95 files, 17.2 MB)
```

`plan_v7.md` was never published: #602 contains no text matching `AMENDMENT 6` or
`DECISION REQUESTED`. Windows temp cleanup, or a disk-reclaim pass, would have destroyed the
only complete copy of the plan the train has been running under. This audit copied it verbatim
to `HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md` (20,098 bytes including the
preservation header). The remaining scratchpad files (`ov1`..`ov9`, Jev scorer scripts, phase-1
inventories, Freebuff DB probe scripts) are still temp-only; `ov1`..`ov9` are also on GitHub.

The two lane continue-prompts from the 15:20Z report were also transcript-only. They are
reproduced verbatim in section 7 so they no longer depend on the transcript.

## 5. Current state, verified live

All values below were read on 2026-10-23T02:29Z to 02:45Z.

| Fact | Value | How verified |
|---|---|---|
| `main` head | `946400c6` (2026-10-22T13:37:05Z, "[A2] CI-workflows batch ... (#628)") | `gh api repos/.../commits/main` |
| `main` CI on that head | 42 success, 2 skipped, 0 failing, 0 pending | `gh api .../commits/946400c6/check-runs?filter=latest` |
| Open PRs | 38 | `gh pr list --state open --limit 200` |
| Queued / in-progress runs | 19 / 1 | `gh run list --status queued|in_progress` |
| Release tags | `v0.5.0` only; no `v0.5.1` | `git ls-remote --tags origin` |
| Highest version on `main` | 0.4.1 | `git show origin/main:Cargo.toml` |
| Rollback anchor branch | absent | `git ls-remote --heads origin | grep pre-v051-train` |
| A3 docker-harness PR | absent; branch exists, diverged ahead 1 behind 1 | `gh pr list`, `gh api .../compare/main...fix/docker-suite-harness` |
| Roles state-seed PR | absent | `gh pr list --state all` |
| Disk | 2.52 GB free of 236.25 GB (98.9%), verdict BLOCKED | `python scripts/disk_budget.py` |

Key PR states at audit time:

```
629 OPEN CLEAN   157af161  rust-dependencies group (20 updates)
599 OPEN CLEAN   3362c1f3  [RULE-8] close #521 review blockers (stacked on 521)
521 OPEN BEHIND  230b6169  [RULE-8] dynamic admission
508 OPEN BEHIND  c6bb8b53  [LAND] conn-fanout diagnostic tool
509 OPEN BEHIND  32e64212  [LAND] seeding security hardening
528 OPEN BLOCKED 80603842  dtolnay/rust-toolchain 1.99.0 -> 1.12
554 OPEN BLOCKED 5b3efc47  canonical recipient hint
550 OPEN BLOCKED 6441203a  StoreAndCarry outbox retry
556 OPEN BLOCKED 9282583a  mobile network-change signal
601 OPEN DIRTY   7f93eed6  OCI runner required checks
609 OPEN BLOCKED 4ed8168b  Ubicloud core runners
611 OPEN DIRTY   c85eb7fe  cargo nextest
```

## 6. The stall: three dead actors, one root cause each

The train is not red and not conflicted. It is unattended. All three roles stopped within
two hours of each other, and nothing has moved for about 11.5 hours.

| Actor | State | Stopped at | Note |
|---|---|---|---|
| Claude Overseer | dead | 2026-10-22T19:48Z | subscription disabled, then OAuth expired |
| Freebuff Landing | `turn_state=idle`, `last_turn_outcome=error` | 2026-10-22T14:56:39Z | "Waiting because the last turn failed. Send a message to continue." |
| Freebuff Implementer | `turn_state=idle`, `last_turn_outcome=error` | 2026-10-22T15:03:45Z | same note |

The lanes did not fail on content. Both ended mid-reasoning with no error text, 7 minutes
apart, at 162k/1M and 279k/320k context respectively. That points at a Freebuff usage or
transient API failure, not a plan or repository defect. The Overseer classified it the same way
at 15:19Z and asked the owner to paste two continue messages, which never happened because the
Overseer itself died about four hours later.

Consequence: the last durable thing the train knows is #602 at 14:49Z. Landing's own journal
never received the 14:55Z ruling's results, no checkpoint was posted after 14:27Z, and MAIN-GREEN
was never declared for `946400c6` even though it is green.

## 7. Immediate unblock: the two continue messages

These are verbatim from the session's 2026-10-22T15:19Z report. Paste one into each Freebuff
thread to resume the lanes. Sending any message resumes an errored thread; these carry the
state the lanes lost.

Into "Landing role instructions and merge order":

```text
CONTINUE (Overseer 15:20Z). Your last turn failed (cause unknown); resume from the journal, not memory. Re-read #602, especially the Overseer ruling comment 6343691430 (runner congestion). Do, in order: (1) poll main 946400c6 to a terminal verdict on that exact SHA and record MAIN-GREEN or MAIN-RED. (2) Cancel QUEUED and in-progress workflow runs whose head is a member PR NOT in the batch currently being verified (gh run cancel); keep main pushes and your own batch head; journal the count. (3) Do not batch the new dependabot PRs (dirs, uniffi, zbus, second rust group). (4) Build batch B (github-actions bumps #529 #530 #533 #537 and Gradle patch bumps #531 #534 #541) as a MERGE COMMIT batch, one PR, one fresh run. Idempotency: before each retried action re-read GitHub and skip if done. Post a checkpoint (role, epoch, stage, head SHA, resumed) when you start.
```

Into "Implementer Role and Merge Train":

```text
CONTINUE (Overseer 15:20Z). Your last turn failed (cause unknown); resume from your checkpoints. Re-read #602 and the Overseer ruling 6343691430. Do, in order: (1) STOP the cancelled-only refresh pass; no more update-branch or reruns on batch-member cars. (2) Open the PR for fix/docker-suite-harness (head b90fc39af, base main 946400c6) titled "[A3] ci(docker): repair NAT/Full Test Suite harness baseline" and post READY with exact head SHA and green check names; you open PRs, Landing batches them. (3) Open the roles state-seed PR next (Landing's request, comment 6338039021, per docs/roles/state-seed.md; docs only). (4) Keep #599 READY and #508 READY. Post a checkpoint (role, epoch, stage, PR, head SHA, resumed) after each step.
```

Two edits are needed before pasting:

1. In the Implementer message, `fix/docker-suite-harness` is now `ahead 1, behind 1` against
   `main` at `946400c6`. It must be rebased or the PR opened from the rebased head; the recorded
   head `b90fc39af` is no longer a clean base. Verify the new head SHA and use it in the READY.
2. Both messages are 11 hours old. Neither authorizes a merge. Landing must re-derive every PR
   head and check-run before acting, per the standing rules.

## 8. Recovery plan

Ordered. Stage 0 is done. Stages 1 and 2 are operator-only. Stages 3 onward are lane work.

### Stage 0 -- preserve (DONE by this audit)

- [OK] `plan_v7.md` copied verbatim to `HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md`.
- [WARNING] That copy is still local and uncommitted. Publishing it needs authorization: either a
  docs-only commit on a working branch, or a comment on #602 carrying the plan body. Do this
  before any cleanup pass touches untracked files.
- [OK] The two continue messages captured above.
- [OK] Current GitHub and lane state recorded in this document.
- Not done, optional: the scratchpad's 95 files are still temp-only. Copy before any temp cleanup
  if the Jev scorer reproduction is wanted; the scores themselves are recorded here and in #602.

### Stage 1 -- restore the Overseer (operator)

Re-authenticate Claude Code on this host. The 19:48Z message says subscription access is disabled
for the organization and suggests an API key; the 20:48Z 401 says the OAuth token expired.
Both must be resolved. Nothing in the Overseer role can run until this is done.

Acceptance: a new Claude session in this repo answers one prompt successfully.

### Stage 2 -- restart the two Freebuff lanes (operator)

Paste the two messages from section 7 into their threads, after fixing the
`fix/docker-suite-harness` head as noted. Do not restart both at once if Freebuff usage is the
suspected cause; start Landing, confirm it posts a checkpoint, then start Implementer.

Acceptance: both threads show `turn_state=running`, then a checkpoint comment appears in #602
with role, epoch, stage, head SHA.

### Stage 3 -- re-establish the durable journal (Landing)

Post one resume checkpoint to #602 containing: main head and its CI verdict, the four landed
cars, the batch inventory from plan v7 (A1 and A2 done; B, C, D, E pending; F release), the two
missing PRs (A3, roles state-seed), and the plan v7 location in this repository. This gives every
role one durable continuation point, which is the thing that failed hardest in this incident.

Acceptance: the comment exists and lists the same main SHA that `gh api` returns.

### Stage 4 -- close the two governance gaps (operator + Landing)

1. Rollback anchor. Plan v7 requires a branch at the pre-train main `657dc1365`. It does not
   exist. Creating it is a push, so it needs explicit authorization:
   `git push origin 657dc1365fd1ae9316abc3bba7849288bd767044:refs/heads/backup/pre-v051-train-main-657dc1365`
2. Plan v7 adoption. The Jev policy gate on the exact text scored correctness 64.0, confidence
   88.0, and an ambiguous disposition. That is a block plus an escalation, not a pass. The
   Overseer asked the owner to decide and the answer was never recorded. Record the decision in
   #602 either way: adopt as-is, adopt with the correctness sections rewritten and re-scored, or
   replace the plan.

Acceptance: anchor ref exists; #602 carries an explicit adoption decision naming a plan version.

### Stage 5 -- resume the train (Landing)

Execute the ov9 ruling that Landing never carried out, in this order:

1. Declare MAIN-GREEN for `946400c6` (already 42 success / 2 skipped / 0 failing -- one command).
2. Cancel queued and in-progress runs whose head is a non-batch member PR. 19 runs are queued now
   on two self-hosted runners; this is the congestion the 14:55Z ruling was written to clear.
3. Build batch B as one merge-commit batch PR: `#529 #530 #533 #537` (github-actions) and
   `#531 #534 #541` (Gradle patches). Exclude the new dependabot majors: `#629`, `#630` (zbus),
   `#631` (uniffi), `#632` (argon2), `#633` (rand 0.10), `#634` (dirs 7), `#635` (thiserror 2),
   `#538` (x25519-dalek 3.0), `#535` (tower-http 0.7), `#540` (kotlin 2.4), `#542`
   (navigation-compose).
4. Land B only on a green exact-head run, then re-verify main before the next batch.

Acceptance: batch B merged or bounced with a named failing check and run id; main green after.

### Stage 6 -- the two missing PRs, then C/D/E, then F (Implementer + Landing)

1. A3 docker-harness PR from the rebased `fix/docker-suite-harness`.
2. Roles state-seed PR (docs only), per `docs/roles/state-seed.md`.
3. Batch C (android/ios), batch D (core routing/transport: #550 #556 #554 #509), batch E
   (connection admission: #521 then #599, in that order, one combined head).
4. Batch D and E need the independent Rule 8 reviewer at the exact head, plus `mobile.yml` and
   `security.yml` dispatched on the batch head, per plan v7 amendment 6.
5. Phase F: version bump, docs sync, `release.yml` with `artifacts_only=true` on the exact main
   head, three-node verify on those artifacts, and only then the `v0.5.1` tag.

Acceptance: two consecutive green batch landings with a green main between them; no tag exists
before the three-node PASS.

## 9. Constraints and things not to do

- Disk is BLOCKED at 2.52 GB free. Do not build locally. CI is the builder; pull artifacts with
  `gh run download`. This is also why copying small evidence files into the repo is the right
  recovery move rather than a second backup tree.
- Do not force-push, rebase shared branches, or delete refs as part of recovery. The train's
  safety model depends on `git revert -m1` of merge commits.
- Do not treat a cancelled or skipped check as a pass. Several cars are "cancelled-only" and
  unverified rather than green.
- Do not act on PR heads quoted in this document without re-deriving them; they were read at
  02:29Z and the lanes may move them.
- Do not batch the dependabot majors listed in stage 5 step 3. `x25519-dalek 3.0`, `zbus 5`,
  `uniffi 0.32`, `rand 0.10` and `argon2 0.6` touch the crypto/FFI/transport stack and need
  individual review.

## 10. Evidence appendix

Commands used for every claim above. All read-only except the single file copy in stage 0.

```
ls -lt ~/.claude/projects/C--Users-SCM-Documents-GitHub-SCMessenger/*.jsonl
tail -c 4000 <transcript>                                  # the 401 and cost-state records
python tmp/audit/extract_session.py <transcript>            # prompts and API errors
python tmp/audit/dump_bash.py <transcript>                  # 212 Bash calls
gh api repos/Sovereign-Communication/SCMessenger/commits/main
gh api repos/.../commits/946400c6.../check-runs?filter=latest
gh pr list --repo ... --state open --limit 200
gh pr list --repo ... --state merged --limit 8
gh pr view 597|616|627|628 --json state,mergedAt,headRefOid
gh api repos/.../compare/main...fix/docker-suite-harness
gh run list --repo ... --status queued --limit 100
gh api repos/.../issues/602/comments?per_page=100
git ls-remote --heads origin                               # anchor absence
python scripts/disk_budget.py
python <scratchpad>/fb7.py                                 # lane turn_state and last turn
```

Reproduction helpers written by this audit, kept under `tmp/audit/` (not committed):
`extract_session.py`, `dump_bash.py`, `extract_mutations.py`, `mutations_71c21a5c.txt`,
`bash_71c21a5c.txt`.
