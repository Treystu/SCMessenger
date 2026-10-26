# V051 — Implementer: route the required Linux leg off the two 2-vCPU OCI runners

Status: ACTIVE (issued 2026-10-23T13:56Z, authored by a read-only tab)
Priority: P1 — second authoring job; branch only, no PR while the pool is starved
Lane: Implementer (branch and PR author)
Authority: #607 / #609 acceptance; PR #608 rejection note (scope); plan v7 merge gate
Writer: Watchdog (read-only tab); this file is the durable copy of the chat brief

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
| main | `ec25a3ea6fd8bdfa08316842b8b4fb7ad68a4582` |
| registered runners | **2 total**: `oci-a1-phx-1`, `oci-a1-phx-2` |
| runner labels | `self-hosted` + `Linux` + `ARM64` + `oci` + `arm64-oci`, on both |
| runner state | both `busy=true` at 13:52Z |
| OCI queue | 30 runs queued/in-progress; 23 queued runs are dependabot branches |
| #636 ubuntu leg | job `120419810738`, `queued` since `12:42:42Z` — 70 min at 13:52Z |
| #642 branch | `ci/empty-required-check-20261023` @ `dc9c480ba`, 0 runs, no PR |

The required context `Test (ubuntu-latest)` is pinned to those two 2-vCPU ARM
runners. Today's recorded runtime is 47-60 min per run, and one leg has been
queued for 70 minutes without starting.

## Premise check BEFORE any edit — do not skip this

The brief given in chat asserts an "8-vCPU ARM Ubicloud trial runner, idle".
This tab could NOT verify that:

- `gh api repos/Sovereign-Communication/SCMessenger/actions/runners` returns
  exactly the two OCI runners above. No Ubicloud runner is present at repo scope.
- `gh api orgs/Sovereign-Communication/actions/runners` returns HTTP 403
  ("must be an org admin or have the runners and runner groups fine-grained
  permission"). Org-scope runners are therefore UNVERIFIABLE from this token.
- The repo's only recorded Ubicloud spec is `ubicloud-standard-2`, i.e. **2
  vCPU**, in `.github/workflows/security-regression-tests.yml:89-93`:
  "the Ubicloud trial tier (ubicloud-standard-2, see ci.yml / USE_UBICLOUD) is
  queue-clearing backup for short jobs -- a cold-cache 2-vCPU runner would make
  this suite slower, not faster."

So the vCPU count and the architecture in the chat brief contradict the spec
recorded in this repo. Resolve it with the runner itself, not with either
document: whichever label set the operator registers, `runs-on` must name
exactly those labels. **GitHub Actions has no fallback in `runs-on`**: a label
no runner carries does not fail, it queues, and a queued required context
blocks every merge in the train. That asymmetry is why the order below is
runner-first, variable-second.

Also recorded here because it is real and unfixed: repo variable
`USE_UBICLOUD=true` exists and is read by **no workflow** at `main`
(`git grep -n USE_UBICLOUD origin/main` matches only the comment above). The
comment points at a code path that does not exist. Do not "fix" the comment as
part of this change; note it in the PR so the dangling variable is visible.

## Capacity inventory — what the UI calls "1 available runner"

Read live at 14:06Z, because this changes the shape of the fix:

- `gh api orgs/Sovereign-Communication/actions/hosted-runners` returns HTTP 404,
  **"GitHub hosted runners are not supported for this organization"**. There are no
  larger GitHub-hosted runners (4-core, 8-core, 64-core) to provision or to use. The
  "Larger GitHub-hosted runners — Unprovisioned" line in the UI is that fact, not a
  pending resource.
- The count in the UI is therefore the **standard hosted pool**, not a named machine.
  `ubuntu-latest` is the runner the repo used before 2026-10-11 and still uses for
  every other job. It is on-demand and effectively unlimited, and it is currently
  being *withheld* from the three OCI-eligible jobs by the routing expression.
- What the repo actually had, from `git log -S` over `.github/workflows/`:
  `ubuntu-latest` (original) -> `blacksmith-2vcpu/4vcpu/8vcpu-ubuntu-2404` selector
  (2026-10-11, `d16606f8c` and `8544cc87a`) -> `ubicloud-standard-2` (2026-10-19,
  `0d9143cec`, `ba9077367`) -> `["self-hosted","oci"]` (current, ARM64). The 8-vCPU
  memory is the **Blacksmith x86_64** label, not an ARM Ubicloud trial.
- Usable capacity today: **two 2-vCPU ARM64 OCI runners** plus the **standard hosted
  Linux pool**. Three ways to spend, one of which is switched off by design.

### The consequence for the change: use the hosted pool for the short legs

Today all three OCI-eligible jobs in `ci.yml` compete for two slots: `test` (47-60 min),
`Docs` (~13 min), `FFI Surface Contract` (~20 min). That is 80-93 runner-minutes of
serial-ish work behind a 47-60 minute job, which is why `#636` waited 80 minutes for a
slot while a free, unlimited hosted pool sat idle.

Recommended split, and it is the smallest change that uses all capacity:

1. `Docs` and `FFI Surface Contract` -> **hosted `ubuntu-latest`**, unconditionally. They
   need no ARM parity, no preinstalled toolchain and no OCI cache; they are the two jobs
   whose hosted runtime is already the baseline.
2. `test`'s ubuntu leg -> stays on the OCI ARM pool, because it is the one job whose
   cache and architecture matter.
3. Never route a third job back to OCI "because it fits". Two slots, one consumer is the
   invariant; the pool exists to shorten the longest leg, not to run everything.

That split alone removes ~33 runner-minutes from the critical path per wave and gives the
required leg a slot it does not have to wait 80 minutes for. It also means the routing
expression should be evaluated **per job** rather than copied three times -- which is what
`scripts/ci_route_check.py` in the #642 branch is already built to assert.

## What #608 already established, and why it was rejected

PR #608 `ci/ubicloud-ubuntu-runners` @ `a1ffccba1` (base `main`) was CLOSED by
the owner 2026-10-20T03:23:29Z, unmerged, with this reason:

> "Closing because this PR changes 16 runner labels across five workflows,
> beyond the five-job acceptance recorded on #607. The broader proposal is not
> the accepted scope; the branch is retained for reference."

Reuse its technique, not its scope. Read the branch
(`git show a1ffccba1`) before writing your own; if it contains the expression
shape you need, cite it in the PR rather than silently diverging.

`docs/EXPERIENCE.md:17` records the runtime failure that killed the attempt:
"action's apt path failed on Ubicloud runners (2026-10-19, PR #608)". That is
the defect this brief is built around.

## Scope: three files, three jobs, zero renames

In scope, in `.github/workflows/ci.yml` at `main`:

| line | job | today |
| --- | --- | --- |
| 202 | `test` (`Test (ubuntu-latest)` leg) | `["self-hosted","oci"]` when push/same-repo PR and `vars.CI_RUNNER == 'oci'` |
| 255 | `docs` | same OCI expression, job-gated on `needs.changes.outputs.rust_relevant == 'true'` |
| 276 | `ffi-surface` | same as `docs` |

Out of scope, and say so in the PR body:

- `forgery-regression` in `security-regression-tests.yml` stays on OCI. Its own
  comment records the reasoning for a 2-vCPU cold-cache runner. If the operator
  confirms 8 vCPU, that reasoning no longer holds and it becomes a separate,
  later change -- not a rider on this one.
- `lint`, `changes`, `handoff-scope`, `naming-policy`, `orchestration`,
  `windows-cli-artifact`, `hygiene-checks` are hosted; do not touch them.
- No branch-protection change, no required-context change, no job rename.

Preserve exactly:

1. **Static matrix.** `os: [ubuntu-latest, macos-latest, windows-latest]` stays
   literal. A dynamic matrix would delay creation of `Test (ubuntu-latest)`
   until `changes` finishes -- today that gap is the difference between a check
   that exists at PR-open and one that appears 56 minutes later.
2. **Job name.** `name: Test (${{ matrix.os }})` and `matrix.os` values stay as
   they are, so the required context string is byte-identical. Add the runner
   as a separate matrix key only if `name:` does not reference it.
3. **Fail-closed steps.** `Fail closed if change detection failed` stays first
   or second; nothing may be reordered in front of it. If #642 lands first, its
   "Refuse an empty required check" guard is already first -- do not duplicate
   it or fight it.

## The part that is not a `runs-on` edit

Both self-hosted helpers key on `runner.environment`:

- `.github/actions/linux-build-deps/action.yml`: when
  `RUNNER_ENVIRONMENT == 'self-hosted'` it runs **verify-only, never sudo or
  apt**, and exits 1 naming anything missing (libdbus-1-dev, pkg-config,
  protoc, rustc/clippy/rustfmt 1.99.0).
- `.github/actions/self-hosted-rust/action.yml`: selects a **preinstalled**
  toolchain from `CARGO_HOME`, prints `uname -m`, and exits 1 if rustc is not
  `1.99.0`. `ci.yml:261` then skips `dtolnay/rust-toolchain` whenever
  `runner.environment == 'self-hosted'`.

A fresh, ephemeral Ubicloud VM is `self-hosted` and has none of that, so the
routing edit alone produces a RED required check at the first dependency step --
the #608 failure mode, one layer down. The change must therefore distinguish a
**persistent** self-hosted runner (preinstalled, verify-only is correct) from an
**ephemeral** one (must install), by a capability label the operator sets, not
by the environment string. Concretely: gate the verify-only branch on a label
such as `persistent` (or `oci`), and let the ephemeral branch run the real
install path non-interactively, failing loudly with an actionable message if it
cannot. Do not silently skip dependency checks to make it green.

Architecture parity: today's runner is **ARM64** (`arm64-oci` label, and
`self-hosted-rust` prints `uname -m`). If the trial runner is x86_64, the change
is not parity-preserving and needs an explicit operator ruling before it lands;
state the arch in the PR body either way, read from a real `uname -m`.

## Evidence the PR must carry (assertion is not evidence)

1. **Runner identity, first-hand.** Label set, `uname -m`, `nproc`, and the
   presence of pkg-config/libdbus-1-dev/protoc/rustc 1.99.0 on the trial runner,
   printed by a `workflow_dispatch`-only probe job on the trial label. Use a
   dispatch probe, not a PR, so it costs the OCI pool nothing and can run before
   the routing variable is flipped. Job-duration must be non-zero.
2. **Per-diff-shape table**, resolved from the real `runs-on` expression and the
   real `detect-platform-change` regexes (do not hand-copy them):

   | diff shape | rust_relevant | runner for `Test (ubuntu-latest)` |
   | --- | --- | --- |
   | docs-only | false | hosted `ubuntu-latest` |
   | Cargo.lock-only | true | trial runner |
   | android-only | false | hosted `ubuntu-latest` |
   | docker-only | true (fail-open) | trial runner |
   | unclassified path | true (fail-open) | trial runner |

3. **One live PR run** after the variable flip, showing `Test (ubuntu-latest)`
   executing on the trial runner with non-zero step durations for
   `Install or verify Linux build dependencies`, `Select preinstalled Rust
   toolchain` (or its replacement), and `Run workspace tests` -- plus the same
   for `Docs` and `FFI Surface Contract`. A queued or skipped job is not
   evidence.
4. **Duration comparison** against today's measured leg: #636's ubuntu job
   `120419810738` queued `12:42:42Z`, 70 min without starting at 13:52Z, with a
   47-60 min runtime once started. Report queue time and run time separately.

## Order of operations (fail-safe)

1. Verify the runner exists, is idle, and carries the labels you will name. If
   it does not exist: STOP. Write the finding on #609 and do not touch ci.yml --
   a `runs-on` pointing at nothing is a merge freeze, not a test.
2. Author the branch and the helper-action change. Push, no PR.
3. Probe run on the trial label via dispatch. Record output in the PR body.
4. Only then flip `vars.CI_RUNNER` (or add the new value) and open one PR.
5. If the probe shows the suite cannot run there for a dependency or
   architecture reason: leave OCI as the required home, post the probe output to
   #609, and stop. A half-routed config that fails open is worse than today's
   queue.

## Do not

- Edit `ci/empty-required-check-20261023` (that is #642, a different branch and
  a different defect). Do not open a PR on either branch while the pool has no
  room; #636 must merge first.
- Touch #636, #637, #639, their branches, or any refresh.
- Run `cargo` or `gradlew` on this host; disk is at 7.61 GB free and
  `disk_budget.py` reports BLOCKED.
- Change which contexts are required, any job name, or branch protection.
- Widen to 16 labels across five workflows. One defect, three files.
