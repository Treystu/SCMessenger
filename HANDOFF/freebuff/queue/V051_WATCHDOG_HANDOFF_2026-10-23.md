# V051 — Watchdog/Overseer handoff, 2026-10-23T14:15Z

Status: ACTIVE — successor lane inherits from this file
Priority: P0 — the train is unblocked but nothing has landed yet
Lane: Watchdog + Overseer (combined), operator-authorized for CI remediation
Repository: Sovereign-Communication/SCMessenger (the only owner of this handoff)
Writer: outgoing Watchdog tab; everything below was read live, not recalled

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Board at handoff (14:11Z, read live)

| item | value |
| --- | --- |
| main | `ec25a3ea6fd8bdfa08316842b8b4fb7ad68a4582` |
| #636 (A3) | head `381eaca9f236a0de392869ce636d15a9cb7e775c`, 22 pass / 0 fail / 4 pending, `BLOCKED` |
| #636 ubuntu leg | job `120419810738` **in_progress on `oci-a1-phx-1` since 14:06:35Z**, after **83 min 53 s** queued |
| #643 (this tab's) | head `6739faf95`, OPEN, `BLOCKED`, **0 failures**; `Test (ubuntu-latest)` queued for `self-hosted+oci` |
| runners | `oci-a1-phx-1`, `oci-a1-phx-2`, both `busy`, both 2-vCPU ARM64 |
| queued runs | **4** (was 26 at 13:57Z) |
| #599 | `aab17e5b0`, base `feat/conn-admission-dynamic`, `UNSTABLE` — held behind #521 |
| #637 / #639 | `4a8e0cf04` / `be6dfaeaf`, both `BEHIND`, untouched |
| #642 branch | `ci/empty-required-check-20261023` @ `dc9c480ba`, 0 runs, no PR |
| required contexts | `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope` |

A3's leg finishing at 47-60 minutes of runtime puts it terminal near **14:55-15:07Z**.

## What changed in the last hour, and by whom

1. **The manual cancel fired** between 13:57:54Z and 14:01:46Z: queued runs 26 -> 2. This tab did not fire it.
2. **This tab opened PR #643** (`ci/use-all-runner-pools-20261023`, commit `6739faf95`) under the operator's explicit authorization to resolve CI and use all runners. Three files:
   - `ci.yml`: `docs` and `ffi-surface` now use `${{ vars.CI_SMALL_RUNNER || 'ubuntu-latest' }}`;
   - `ci.yml` `test` + `security-regression-tests.yml` `forgery-regression`: OCI selection excludes dependabot heads;
   - `scripts/ci_pool_route.py`: asserts the allocation, fails on an unapproved OCI consumer.
3. **This tab set repo variable `CI_SMALL_RUNNER=ubicloud-standard-2`** (14:09Z). **The label is UNVERIFIED**: repo-scope runner list shows only the two OCI runners, and org-scope enumeration returns HTTP 403. Docs and FFI are **not** required contexts, so a wrong label cannot block a merge — it only leaves two non-required jobs queued.
4. **Four briefs exist** in this directory: `V051_LANDING_CANCEL_AND_MERGE_2026-10-23.md`, `V051_IMPLEMENTER_EMPTY_REQUIRED_CHECK_2026-10-23.md`, `V051_IMPLEMENTER_RUNNER_ROUTING_2026-10-23.md`, `V051_IMPLEMENTER_POOL_CONTAINMENT_2026-10-23.md`.

## Runner inventory — settled facts, do not re-derive

- Two self-hosted runners only: `oci-a1-phx-1`, `oci-a1-phx-2`. Labels: `self-hosted`, `Linux`, `ARM64`, `oci`, `arm64-oci`.
- `gh api orgs/Sovereign-Communication/actions/hosted-runners` -> **404 "GitHub hosted runners are not supported for this organization"**. There are no larger GitHub-hosted runners to provision; the "1 available runner" in the UI is the standard hosted pool, not a machine.
- Historical labels, from `git log -S` over `.github/workflows/`: `ubuntu-latest` -> `blacksmith-{2,4,8}vcpu-ubuntu-2404` (2026-10-11) -> `ubicloud-standard-2` (2026-10-19) -> `["self-hosted","oci"]` (current). The remembered 8-vCPU runner is the **Blacksmith x86_64** label, not an ARM Ubicloud trial.
- `USE_UBICLOUD=true` exists and is read by **no workflow** on main; the only mention is a comment in `security-regression-tests.yml` pointing at a `ci.yml` path that does not exist.

## Verification that PR #643 actually routes as designed

Read from the PR's own run `39967132253`, not asserted:

- `Docs` -> `labels=ubuntu-latest`, `runner=GitHub Actions 1000057046` (hosted, not OCI)
- `FFI Surface Contract` -> `labels=ubuntu-latest`, `runner=GitHub Actions 1000057047`
- `Test (ubuntu-latest)` -> `labels=self-hosted+oci`, queued (correct: car first, FIFO)

## What the successor must do, in order

1. **Verify #636 to terminal and land it.** Merge commit only, `--match-head-commit 381eaca9f236a0de392869ce636d15a9cb7e775c`, only on 0 failed / 0 pending at that exact head. Kill condition: a failure on `registry_lock_is_not_held_across_open_retry` or under `contacts_bridge` means no rerun, no refresh, no merge — post the evidence and stop.
2. **Declare MAIN-GREEN** on the resulting main, naming run id, SHA and tally, in #602.
3. **Get #643 reviewed and merged by someone who is not its author.** This tab authored it, so it cannot merge it. If the review finds the `CI_SMALL_RUNNER` design wrong, the fallback is to drop that one line and keep the dependabot exclusion.
4. **Validate or revert the Ubicloud label.** First Docs/FFI job created after 14:09Z either lands on a `ubicloud-standard-2` runner (label exists) or stays queued (it does not). If queued past ~15:00Z: `gh variable delete CI_SMALL_RUNNER --repo Sovereign-Communication/SCMessenger`, then cancel the stranded jobs. One command, no code change.
5. **Request one `update-branch` each for #637 and #639** from the Implementer, recording the ask in `tmp/lane/landing_requests.txt`.
6. **#642's PR may open** once #636 lands and the pool has one free slot; its branch is `dc9c480ba` and it must be rebased on the new main first.
7. **Containment stays a parked branch** (`V051_IMPLEMENTER_POOL_CONTAINMENT_2026-10-23.md`): `.github/dependabot.yml` only, land after #642 and #643.

## Stack order (settled; do not relitigate)

`#642` (empty required check) -> `#643` (pool allocation, already open) -> containment (`dependabot.yml` only) -> #609/routing reduces to the single `CI_SMALL_RUNNER` switch and should be closed or rewritten as such.

Reason: #642 and #643 both edit the same `runs-on` expression. One diff owns it at a time.

## Lane state and supersessions the successor must know

- **`V051_IMPLEMENTER_RUNNER_ROUTING_2026-10-23.md` is superseded by PR #643** for everything except
  the `ubicloud-standard-2` probe and the #609 rewrite. Do not ask the Implementer to author a
  separate routing branch now; that would collide with #643 on the same expression.
- **The Implementer lane is alive** in this checkout: `tmp/lane/state.txt` last read
  `phase=P1; step=route-check-ran-once-on-target-commit; exit=0`, with `tmp/lane/status.sh` and
  `tmp/lane/evidence.txt` beside it. Its parked branch is still `dc9c480ba`, 0 runs, no PR.
- **The previous Landing lane** was on its closing pass when this handoff was written; it produced
  no cancel, no journal entry and no `tmp/landing/state.txt` in that pass. Treat its lane as dead
  and inherit only from this file and #602.
- **Landing successor prompt was delivered in chat, not in the repo.** If it is lost, rebuild it
  from the duty order above; this file is authoritative for obligations, not for prompt wording.

## Hard rules for the successor

- Never merge on a non-terminal wave, a different head, or a green with pending checks.
- Never rerun to get a different colour; a cancelled run is recreated by a refresh.
- Never write a branch, refresh, or force-push from a Landing-scoped lane; branch writes are the Implementer's.
- Do not merge a PR this tab authored (#643) without an independent reviewer.
- The shared checkout is dirty with other sessions' work. Touch only your files; a clean `git status` is not a goal.
