# V051 — Implementer: author the #642 fix, runner-free

Status: OPEN — branch PUSHED 2026-10-23T13:35:35Z, `ci/empty-required-check-20261023` head `dc9c480ba`, no PR, **0 workflow runs created** (correct: branch push with no PR). Superseded in text by the token-frugal lane prompt issued 13:45Z; the git work below is done.
Priority: P1 — queue efficiency, not a release blocker
Lane: Implementer (branch and PR authority; never merges)
Authority: issue #642; plan v7 (merge gate, one merge at a time)
Writer: Watchdog (read-only tab)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Why this is runner-free

Every `push` trigger in this repository is `branches: [main]`, a tag push, or
`workflow_dispatch` / `schedule`. Verified across all 20 workflow files at
`ec25a3ea6`. A branch push with **no PR** creates zero workflow runs.

The pool currently has two runners and a 26-deep OCI queue; #636's A3 has waited
47 minutes for `Test (ubuntu-latest)`. Do not open a PR until A3 has merged or
Landing confirms capacity. Push the branch, stop, report.

## The defect (read the real files, do not trust this text)

`.github/workflows/ci.yml` job `test` is the only job that (a) starts
unconditionally and (b) can select the self-hosted runner. `docs` and
`ffi-surface` are OCI-routed but gated at job level on
`needs.changes.outputs.rust_relevant == 'true'`; `typecheck`, `lint`,
`changes`, `handoff-scope`, `naming-policy`, `orchestration` and
`windows-cli-artifact` are hosted.

So a docs-only or android-only diff starts `Test (ubuntu-latest)` on
`oci-a1-phx-1/2`, skips every gated step, and reports success in 9-31 seconds.
Measured today: #637 `4a8e0cf04` 31s, #639 `be6dfaeaf` 9s, both with all ten
real steps `skipped`. A real run of the same job measures 47m34s.

## What to change (one expression, one guard)

(a) `runs-on` for job `test`, gated on the same flag that gates the steps:

```yaml
runs-on: ${{ matrix.os == 'ubuntu-latest'
             && needs.changes.outputs.rust_relevant == 'true'
             && (github.event_name == 'push'
                 || (github.event_name == 'pull_request'
                     && github.event.pull_request.head.repo.full_name == github.repository))
             && vars.CI_RUNNER == 'oci'
            && fromJSON('["self-hosted","oci"]')
            || matrix.os }}
```

`needs` is permitted in `jobs.<job_id>.runs-on` by the GitHub Contexts reference
(allowed: github, needs, strategy, matrix, vars, inputs). Keep the matrix STATIC:
a dynamic matrix postpones creation of `Test (ubuntu-latest)` until `changes`
finishes, which is exactly the 56-minute gap that stalled #636.

(b) First step of the job, before the existing fail-closed step:

```yaml
- name: Refuse an empty required check on the scarce pool
  if: runner.environment == 'self-hosted' && needs.changes.outputs.rust_relevant != 'true'
  run: |
    echo "[FAIL] Test (ubuntu-latest) landed on a self-hosted runner with"
    echo "rust_relevant=${{ needs.changes.outputs.rust_relevant }} -- it would run no steps."
    echo "A required context may never be satisfied by a job that did nothing."
    exit 1
```

## Evidence is the deliverable

Add `scripts/ci_route_check.py` (stdlib only, no network, no runner):

- parse the three path regexes from
  `.github/actions/detect-platform-change/action.yml` and the inert rule
  `^docs/|^HANDOFF/|\.md$`; do not hardcode a private copy;
- apply them to five fixtures and print the four flags;
- resolve the `runs-on` decision for `matrix.os=ubuntu-latest`,
  `pull_request`, same-repo head, `vars.CI_RUNNER=oci`;
- assert this contract, non-zero exit on any mismatch:

| diff shape | rust_relevant | runner for Test (ubuntu-latest) |
| --- | --- | --- |
| docs-only | false | `ubuntu-latest` |
| Cargo.lock-only | true | `self-hosted` + `oci` |
| android-only | false | `ubuntu-latest` |
| docker-only | true (fail-open) | `self-hosted` + `oci` |
| unclassified path | true (fail-open) | `self-hosted` + `oci` |

Fixtures, from today's real board:

```
docs-only        docs/CANON_PLAN.md
                 HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md
Cargo.lock-only  Cargo.lock
android-only     android/app/src/test/java/com/scmessenger/android/ui/viewmodels/ContactsViewModelTest.kt
docker-only      docker/Dockerfile.rust-test
                 docker/docker-compose.test.yml
unclassified     newtool/config.yml
```

Save the output to `tmp/ci-route-evidence.txt` and paste it into the PR body
verbatim. `docker/` must still take the ARM runner: it is unclassified, fail-open
is correct, and the suite has to really run.

## Authoring sequence — DONE, do not repeat it

```
git switch -c ci/empty-required-check-20261023 ec25a3ea6
<edit ci.yml> <add scripts/ci_route_check.py>
python scripts/ci_route_check.py            # must exit 0
git push -u origin ci/empty-required-check-20261023     # NO PR
```

Delivered: head `dc9c480ba`, one commit, `.github/workflows/ci.yml` +23/-1 and
`scripts/ci_route_check.py` +484, zero workflow runs, no PR. The verifier reads
the branch; it does not rebuild it.
Open the PR only after A3 merges: base `main`, title
`[ci] required check must not be satisfiable by a job that did nothing`,
body = #642 reference, evidence block, and "changed contexts: none".

## Do not

- change which contexts are required, any job name, or branch protection;
- make the matrix dynamic, or add a job;
- touch `docs` / `ffi-surface` / `lint` routing (they cannot start empty);
- edit #636, #637, #639 branches, HANDOFF files, or any `core/` source;
- fix anything else in `ci.yml`, even if it looks wrong: one defect per PR.

## Kill conditions

- `needs` unresolvable in `runs-on` for this repository: fall back to a JSON
  runner string emitted by the `changes` job and
  `fromJSON(needs.changes.outputs.test_ubuntu_runner)`. Say so in the PR; do not
  invent a third mechanism.
- The change would rename a check or alter the required set: stop.
- You cannot reproduce the classification from the real `action.yml` (i.e. you
  had to hand-copy the regexes): stop, the source of truth moved.
- Anything in the working tree you did not write: leave it alone.
