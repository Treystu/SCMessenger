# V051 — Implementer: stop the dependabot waves from regenerating in front of the required leg

Status: ACTIVE (issued 2026-10-23T14:05Z, authored by a read-only tab)
Priority: P2 — third parked branch; no PR while the pool is starved
Lane: Implementer (branch and PR author)
Authority: #607 / #609 acceptance; #642 contract; plan v7 merge gate
Writer: Watchdog (read-only tab); this file is the durable copy of the chat brief
Repository: Sovereign-Communication/SCMessenger (the only owner of this handoff)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Board at issue time (read live)

| item | value |
| --- | --- |
| main | `ec25a3ea6fd8bdfa08316842b8b4fb7ad68a4582` |
| the manual cancel | **fired** between 13:57:54Z and 14:01:46Z: queued runs **26 -> 2** |
| remaining queued runs | #636's own CI run `39948447516` (the docker-suite branch), plus `32215840190` Hygiene on `fix/h2-rustsec-2026-0258`, **queued since 2026-08-19T04:27:42Z** |
| #636 ubuntu job `120419810738` | still `queued`, 80 min at 14:03Z; `Test (macos-latest)` moved to `IN_PROGRESS`, `Swift Linting` passed -> 22 pass / 0 fail / 4 pending |
| runners | `oci-a1-phx-1` on uniffi's CI run `39949387526`, `oci-a1-phx-2` on zbus `Forgery` `39948417611` |

The pool is about to clear. The thing that filled it will refill it: nothing authored
today prevents six dependabot PRs from re-heading on the next main move and each
spawning a fresh wave.

## Premise check BEFORE any edit — two of the three assumptions are already false

1. **Concurrency cancel-in-progress already exists** in `ci.yml`, `cross.yml`,
   `desktop.yml`, `hygiene.yml`, `ios-build-test.yml`, `lint.yml`, `mobile.yml` and
   `security-regression-tests.yml`, all keyed
   `group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}`
   with `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`. It **works**:
   on `dependabot/cargo/rand-0.10.2` the 11:57:15Z re-head shows `CI`, `Lint`, `Mobile`
   and `Security Regression Tests` all `completed/cancelled`, superseding their own
   08:15:47Z runs. So "superseded waves must cancel themselves" is already the
   behaviour for the same workflow on the same PR. Do not rewrite it to prove a point.

2. **The queue was not stale waves of one workflow.** Each dependabot re-head fires
   **eight** workflows: `Auto Label`, `CI`, `Cross`, `Lint`, `Mobile`,
   `Repository Hygiene`, `Security Regression Tests`, `iOS Build & Test` (read from the
   run list of two PRs, same set on both). 6 PRs x 8 workflows = 48 runs per main move;
   at 13:57Z the queue held 26, i.e. roughly one wave. No concurrency key can collapse
   eight *different* workflows, and none can collapse six different PRs.

3. **What is actually missing** is grouping and reachability, not supersession:
   `.github/dependabot.yml` has `groups.rust-dependencies` with `update-types:
   [minor, patch]` only, so every **major** bump is its own PR (`rand 0.10`,
   `uniffi 0.32`, `thiserror 2.0`, `dirs 7`, `zbus 5.19`, `argon2 0.6` are exactly
   those), and `github-actions` and `gradle` have **no groups at all**
   (`open-pull-requests-limit` 10 / 5 / 5).

If the real files disagree with any of that, STOP and report the divergence rather
than implementing the brief as written.

## The invariant, stated once and enforced mechanically

> **The scarce self-hosted pool may only ever be spent on a job whose result someone
> will act on.** A job whose run cannot gate a merge, or whose PR will not be merged
> while the train is running, must not consume an ARM slot.

Every hunk below must be defensible against that sentence, and the script must fail
closed when a change makes it untrue.

## Scope: disjoint files from the other two branches

In scope:

- `.github/dependabot.yml` — group cargo majors (either into `rust-dependencies`, or a
  `rust-majors` group; one PR per ecosystem per cycle is the goal), and add groups for
  `github-actions` and `gradle`.
- `scripts/ci_pool_guard.py` (new, stdlib only, no runner, no network) — static
  assertions over the real workflow files:
  1. every workflow with a `pull_request` trigger declares `concurrency`;
  2. every such group contains `github.event.pull_request.number` (or `github.ref` on a
     pull_request event, which is `refs/pull/N/merge`); a group that is a bare
     `${{ github.workflow }}`, a fixed string, or `github.run_id` on a PR workflow is a
     failure because it can cancel a car's wave;
  3. `cancel-in-progress` is true for `pull_request` events on those workflows;
  4. report (initially as a listed PENDING, then as a hard failure once the routing
     branch lands) whether the self-hosted selection excludes `dependabot[bot]`.
- Optional, only if it costs nothing: `cross-platform-test.yml`, `docker-publish.yml`,
  `release.yml`, `security.yml`, `stale.yml` have no concurrency block at all — adding
  one is fine **only** if it cannot cancel a `main` push run (see P3 below).

Out of scope, and say so in the PR body:

- `ci.yml`'s `runs-on` expression. It is owned by the routing branch (see stack order).
- `forgery-regression`, any job name, any required context, branch protection.
- The stale `32215840190` run: that is Landing's cancel, not a code change.
- `cargo`/`gradlew` execution on this host.

## The three proofs the PR must carry

**P1 — a dependabot PR re-heads while its older wave is queued.**
Same `(workflow, pull_request.number)` group -> the older run is cancelled, verified
from real run history rather than argued: cite the 11:57:15Z rand re-head where `CI`,
`Lint`, `Mobile`, `Security Regression Tests` show `cancelled`. After grouping, the
wave count per main move drops from 6 PRs x 8 workflows to 1-2 PRs x 8 workflows; show
the arithmetic from the actual `dependabot.yml` diff.

**P2 — a merge to main while dependabot waves are live.**
Main's push runs share no group with any pull request (`pull_request.number` is empty on
push, so the group falls back to `github.sha`), and with the actor exclusion carried by
the routing branch, dependabot's self-hosted legs become hosted legs. Demonstrate from
run history that a main push run was never cancelled by a dependabot re-head, and state
the ordering fact plainly: `Test (ubuntu-latest)` on main and on a car are the only
consumers that matter.

**P3 — a car PR's own wave must never be cancelled by the dependabot policy.**
This is the **kill condition**. A car's wave is identified by its `pull_request.number`.
If any proposed key is `${{ github.workflow }}` alone, a fixed string, or anything that
does not vary per PR, it can cancel a car's wave and the design is wrong — reject it,
do not "mitigate" it. Prove safety by construction: two PRs, two groups, and a fixture
in `ci_pool_guard.py` that asserts a dependabot-keyed group can never equal a
car-keyed group.

## Stack order — one recommendation, not options

**Land strictly third: #642 -> routing -> this one.** Author it now against `main`, but
do not expect it to merge before the other two.

Reason, concretely: all three want the same object. #642 and routing both rewrite the
`runs-on` expression in `ci.yml`; the invariant here needs one clause added to that same
expression (`github.event.pull_request.user.login != 'dependabot[bot]'`). Three branches
editing one line is three conflicts and three re-verifications. So **one diff owns the
expression**: the routing branch carries the actor exclusion, and this branch carries
`dependabot.yml` plus the guard that fails when the exclusion is missing. That keeps the
files disjoint, which is what makes parallel authoring safe at all.

Do **not** stack this on `dc9c480ba`. #642 will be behind main after the next merge, and
a stack inherits that rebase tax on every landing. Author against `ec25a3ea6` (or
whatever main is when you start), rebase once at PR time.

## Evidence that makes it landable when the pool clears

- `python scripts/ci_pool_guard.py` exit 0 on the working tree, plus a negative test in a
  throwaway copy: delete one group from `dependabot.yml` (or set a group to
  `${{ github.workflow }}`) and confirm it exits non-zero naming the exact workflow.
- The grouped-PR arithmetic: 6 cargo majors today -> N PRs after the change, with the
  current PR numbers quoted.
- The supersession citation from P1 with run ids and conclusions.
- `gh api repos/.../actions/runs?branch=<dependabot branch>` output showing 8 workflows
  per re-head today, pasted verbatim.
- One sentence on what the change does **not** do: it does not cancel a live car wave, it
  does not change any check name, and it does not make a required context skippable.

Reviewer one-pass checklist (put it in the PR body so the reviewer does not have to
derive it): files touched are `dependabot.yml` + `scripts/ci_pool_guard.py` only; no
`ci.yml` hunk; no job renamed; no required context added or removed; the guard's negative
test output pasted; the three proofs present with run ids.

## Do not

- Touch `ci.yml`, the `runs-on` expression, `forgery-regression`, or any job name.
- Edit `ci/empty-required-check-20261023` or the routing branch.
- Touch #636, #637, #639 or their branches; no refreshes.
- Open a PR on this or the other two branches while the pool is starved.
- Delete or "tidy" the stale `32215840190` run — that is a cancel, and it belongs to
  Landing.
