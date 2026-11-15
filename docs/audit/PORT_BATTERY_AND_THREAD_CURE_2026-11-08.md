# CI port: the four thread defects cured, and the battery read that follows

Date: 2026-11-08 (session 16:40Z-17:05Z)
Surface: PR #677 (`ci/circleci-port-main-20261108`), tracker #602, PR #653
Standing fence: **main is UNVERIFIABLE** - `git cat-file -e
origin/main:.circleci/config.yml` fails, so no car based on main can carry an
admissible battery until this port lands.

## 1. The four unresolved threads were real, and they were defects in the copy

The cubic review on head `f3b0db3cb` filed four threads, all unresolved, all in
`.circleci/config.yml`. Each was read in full and checked against the file and
against the Actions equivalent before any change was made. None is a style
preference:

| Line | Finding | Why it is a defect | Cure in `201d2c1c9` |
|---|---|---|---|
| 127 | `[ -d .git/shallow ]` | `.git/shallow` is a FILE. The test never fired, so a shallow CircleCI checkout skipped `--unshallow` and the history-dependent gates (hygiene, handoff scope, naming) ran on a truncated range - the exact failure the command exists to prevent | `[ "$(git rev-parse --is-shallow-repository)" = "true" ]` |
| 314 | `mapfile -t CHANGED < <(git diff ...)` | A process substitution's exit status is not the command's; `mapfile` succeeds even when the `git diff` inside it fails. A failed diff produced an empty `CHANGED` array, which the next branch reads as "only line-ending changes" and exits 0 - a gate that fails open | capture with `if ! CHANGED_DIFF=$(...)`, exit 2 on failure, then populate the array only when non-empty |
| 523 | queue-status gate ran enforcing on every branch | `.github/workflows/hygiene.yml:357-359` enforces only on `pull_request` and passes `--warn-only` on push. The port enforced on push to `main` too, which would red a main push for a queue-ticket drift the Actions rail tolerates | branch on `CIRCLE_BRANCH=main` and pass `--warn-only`, matching Actions |
| 44 | header listed `naming-policy` as unported | the job is defined and scheduled in the same file (`naming_policy`, workflow `ci`), so the header contradicted the config | removed from the remaining-unported list |

After the push, all four threads read `isResolved=true` (GraphQL
`reviewThreads`, 2026-11-08T16:52Z), so clause A1(e) - no unresolved review
threads - is satisfied on the new head.

## 2. Verification actually run

- `python -c "yaml.safe_load(...)"` on the edited file: parses, **12 jobs, 1
  workflow**.
- The `mapfile` replacement was exercised in bash against three inputs - empty,
  one line, two lines - and produced counts 0, 1 and 2 respectively, which is
  the behaviour the gate depends on.
- `git diff --stat origin/main...HEAD`: **1 file changed, 972 insertions**,
  clear of `core/src/{crypto,transport,routing,privacy}` and `vendor/`.
- Branch is `behind_by=0` against main, `MERGEABLE`, no conflicts
  (`pr_scope.sh 677` at 16:47Z reported only the failing-checks reason and no
  conflict).

## 3. The battery read, and the trigger gap it exposes

Recorded as an observation with its method, because it is not yet a conclusion:

- Prior head `f3b0db3cb`: 12 `ci/circleci:*` contexts, **12 success**, posted
  from `2026-11-08T10:49:52Z` (read: `commits/<sha>/status`, `target_url`
  `circleci.com/gh/Treystu/SCMessenger/<pipeline>`).
- New head `201d2c1c9` (pushed `16:47Z`): at `16:57:38Z`, **0** `ci/circleci`
  contexts and combined `state=pending` (read: `commits/201d2c1c9/status`).
  GitHub Actions on the same head reported 9 success / 8 failure / 4 pending,
  and the 8 failures are the same account-lock class already classified on the
  prior head (empty `runner_name` at the single-run endpoint, billing
  annotation on every run, none a required context).
- No CircleCI check-suite exists for the head either (read:
  `commits/201d2c1c9/check-suites`, 16:56Z), and the CircleCI project page
  requires a login this session does not hold, so the pipeline cannot be
  inspected directly.

**No green claim is made for `201d2c1c9`.** The port is not merged, and it must
not be merged, on the strength of the previous head's battery: that head
carried the four defects. The merge basis is a terminal, green battery on the
head that would land, plus the filed gate receipts.

## 4. What this does not do

- It changes no branch protection, no required context, no workflow, no
  reviewer and no secret. It touches one file.
- It does not disposition #489, per the standing order.
- It does not merge anything.

## 5. Next action

Read `ci/circleci:*` on `201d2c1c9` to terminal. Green: merge #677 under the
merge law (one non-destructive file, blanket approval). Red: the failing job is
read before anything else moves. No battery at all: the trigger gap is filed as
a blocker on the PR and the CI owner is named, because a port whose own rail
does not start on its own head is not verifiable and must not land on faith.

Once it lands, C1 resumes at #660 and C4 at #506/#508 - each `update-branch`
onto the post-port main so the car can carry a battery instead of nothing.
