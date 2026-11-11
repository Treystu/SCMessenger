# #643 dependabot carve-outs adopted on the lane (carve-out half only)

Status: filed
Scope: the guard-preserving subset of #643 identified on 2026-11-05, applied to `lane/burndown`. #643
itself is untouched: this is not a merge of #643, and the pool-allocation re-scope it also contains is
deliberately NOT adopted.

## What was adopted

Two hunks, exactly as verified on 2026-11-05 in
`docs/audit/ci_resolution_2026-11-05/643-guarded.patch` (the artifact re-applied cleanly today:

    git apply --check -v docs/audit/ci_resolution_2026-11-05/643-guarded.patch

printed `Checking patch .github/workflows/ci.yml...` and
`Checking patch .github/workflows/security-regression-tests.yml...` with no error line):

1. `.github/workflows/ci.yml` required Test leg: adds
   `&& github.event.pull_request.user.login != 'dependabot[bot]'` to the routing guard. A dependency re-head
   is sent to hosted runners instead of one of the two ARM OCI slots.
2. `.github/workflows/security-regression-tests.yml` `forgery-regression`: the same carve-out on a line that
   already carried the guard. This suite runs ~41-50 min, so it is the worst possible tenant for a two-slot
   pool.

Each site also gained a short comment naming the reason, because the expression alone does not explain
itself and the next reader would otherwise have to re-derive it from this file.

## Verification (run against the edited files, not the artifact)

| check | result |
|---|---|
| `python scripts/check_workflow_yaml.py` | exit 0, `[OK] 27 workflow/action files contain no duplicate mapping keys` |
| `python scripts/ci_route_check.py` | exit 0, **8/8 invariants**, including "a Rust-relevant diff still takes the pool" and "the guard never fires on any contract fixture" |
| rollback switch preserved | `USE_OCI_RUNNER` `ci.yml` 8 -> 8, `security-regression-tests.yml` 1 -> 1 |
| dependabot token present in both | yes, one occurrence per file |

The rollback-switch count is A12's exact metric: the #661 rollback switch is the mechanism whose deletion
made #643 unlandable as written, and it does not move here.

## What was NOT adopted, and why

The rest of #643 is not a carve-out. Its own `scripts/ci_pool_route.py` rejects main's routing for seven
jobs (`docs`, `ffi-surface`, `changes`, `handoff-scope`, `naming-policy`, `lint`, `orchestration`), and
`CI_SMALL_RUNNER` is unset, so adopting it now would route Docs and FFI Surface to hosted runners - which
is exactly what the billing lock refuses. That is a pool-allocation policy decision for the CI owner, and
`docs/audit/CI_ROUTING_RESOLUTION_2026-11-05.md` section 2b records it as such.

Re-confirmed today: main's `ci.yml` carries `CI_SMALL_RUNNER` **0** times, so #643's routing cannot be a
refresh of main - it replaces the #661 switch wholesale.

## Effect on the lane

This is CI routing for Actions, which is not the verdict rail (`ci/circleci:*`). It changes which runner a
dependabot PR uses; it does not add, remove or alter any required context, and it does not touch branch
protection. No protection setting, merge, billing change or credential was touched.
