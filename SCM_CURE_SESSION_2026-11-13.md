# SCM-CURE session record, 2026-11-13 (v3, lane/cure-579)

Lane: scm-cure. Base: `main` @ `ced2189e9d036d389d4aa4e7e40d42357a61f5c6`.
This copy travels with `lane/cure-579`; the copy on `lane/cure-545` records the
#545 draft, and the copy on `lane/cure-684` records the #684 Part A cure. Each
branch carries the record for the work on it rather than one file spanning
branches that may land separately.

## Stand-up

Canonical check: **NOT CHECKED**. This session runs in a cloud sandbox whose home
is not the operator's machine; none of the three doctrine key locations resolves:

- `~/.typesafe_key`: No such file or directory
- `~/SCM*` (SCM ORC folder keys): No such file or directory
- `~/Documents/GitHub` (Harness-configured root): No such file or directory

No key was printed, created, or claimed absent from the operator's machines.

Toolchain: `which cargo rustc python3` -> `/usr/local/bin/python3` only. Disk is
9.6 G free of 16 G.

## Write channel: the same gap, recorded once for this branch

Re-checked at the start of this turn rather than assumed: `gh auth status` reports
no logged-in host, `~/.config/gh`, `~/.netrc` and `~/.git-credentials` do not
exist, no credential helper is configured, and no token-shaped environment
variable is set. The anonymous GitHub API was exhausted earlier in the session
(`limit: 60, remaining: 0`).

So nothing has been posted to #602, no PR exists for this branch, and no check-run
or CircleCI status has been read. The only write available is the branch push.
Everything below is therefore a committed artifact; tracker-side actions are
listed as pending, not claimed.

## #579 delivered: the kernel reads the declared V051 queue

The issue asked for one of two things - point the kernel at the active queue
through the execution pointer or a manifest field, or add the V051 cars to a jsonl
it reads - and to document the choice. This does both halves, because each alone
leaves a hole: a manifest field with no jsonl names a file that does not exist,
and a jsonl with no manifest field is a file the kernel never looks at.

What changed, with anchors:

- `orchestration/manifest.yaml`: new `queue` block after `kernel` -
  `path: orchestration/v051_cars.jsonl`, `release_target: v0.5.1`,
  `source_document: HANDOFF/freebuff/queue/V051_UNIFIED_MERGE_TRAIN_2026-10-14.md`,
  `generator: scripts/v051_queue_sync.py`. Extra top-level keys are accepted by
  `scripts/orchestration_contract.py:20-24` (`REQUIRED_TOP_LEVEL` checks only for
  missing keys), so the contract still validates unchanged.
- `scripts/orchestrate_strict.py`:
  - new `resolve_queue(explicit, manifest)` after `read_next_task`: `--queue`
    wins, else `manifest["queue"]["path"]`, else `ContractError`. A controller
    that cannot name its queue no longer guesses at one.
  - new `list_queue(queue_file, manifest)` next to it: prints provenance and
    every car with status, review gate and dependencies, no truncation.
  - `--queue` lost its `default="scm_v1_farm_queue.jsonl"` (was line 738, the
    line the issue cites) and became an override with no default.
  - `main()` resolves the queue after `load_manifest()` and, when `--dry-run` is
    set, lists it before the dispatch loop; the dispatch loop now reads
    `queue_file` instead of `args.queue`.
- `scripts/v051_queue_sync.py` (new): derives `orchestration/v051_cars.jsonl`
  from sections 3, 4 and 5 of the merge-train document, so the document stays the
  execution authority. `--check` exits 1 on drift. Statuses are mapped from the
  document's own words (DONE -> done, operator-gated -> blocked, superseded-close
  -> retired, else open), `MT-08..MT-12` expands to five cars because #580 tracks
  them one by one, and only dependencies the document states in words are
  encoded (`R-15` on `R-00a`/`R-00b`; `F-2` on `F-1`; `F-3` on `F-1`/`F-2`).
- `orchestration/v051_cars.jsonl` (new, generated): 36 cars - 31 open, 3 done,
  1 blocked, 1 retired.
- `scripts/test_orchestration_v2.py`: new
  `test_kernel_reads_the_declared_v051_queue`, which asserts the manifest
  declaration, the flag-wins and fails-closed behaviour of `resolve_queue`, that
  the jsonl is in sync with the document, and that a dry run lists every car.
  It lives in this suite because `.circleci/config.yml` already runs it, and a
  test outside CI's path is a dead test.
- `docs/ORCHESTRATION.md`: new "Queue resolution" section, placed before
  "Live-provider validation status", documenting the choice as the issue asks.
- `HANDOFF/freebuff/queue/V051_UNIFIED_MERGE_TRAIN_2026-10-14.md`: section 6 said
  no jsonl existed and the kernel had to be pointed by hand. A dated update
  records the closure and the edit-then-regenerate order, leaving the original
  paragraph as history.

### Acceptance, as the issue states it, read this session

| Command | Result |
|---|---|
| `python3 scripts/orchestrate_strict.py --dry-run` | rc 0; lists 36 cars, `blocked=1, done=3, open=31, retired=1`, then plans the first ready cars |
| `python3 scripts/orchestration_contract.py --print-version` | `2.0.0`, rc 0 |
| `python3 scripts/v051_queue_sync.py --check` | `[OK] ... matches the document: 36 cars`, rc 0 |
| `python3 scripts/test_orchestration_v2.py` | `Ran 27 tests ... OK`, rc 0 |
| `bash scripts/docs_sync_check.sh` | `docs-sync-check: PASS`, rc 0 |
| `python3 scripts/validate_handoff_scope.py --document HANDOFF/freebuff/queue/V051_UNIFIED_MERGE_TRAIN_2026-10-14.md` | `[OK] ... SCMessenger-only handoff`, rc 0 |

Baseline for contrast, read before the change on the same base: `--dry-run` printed
`[INFO] no ready task` and rc 0 - it "passed" while reading a queue that holds none
of the cars, which is exactly the defect #579 describes.

One invocation caveat, recorded so nobody repeats my confusion:
`python3 -m unittest scripts/test_orchestration_v2.py` fails with
`ModuleNotFoundError: No module named 'orchestration_contract'` because that form
does not put `scripts/` on `sys.path`. It is a harness artifact of this suite, not
a defect in this change: the CI form, `python3 scripts/test_orchestration_v2.py`,
runs 27 tests and passes.

## Carried, not re-litigated

- #622: verified cured on `main` earlier in the session (26/26 orchestration
  tests; real merged pack yields `(0.95, 0.95, True)` all-pass and `(0.02, ...)`
  on a weak `instruction_matches`; bare three-key pack still accepted). The
  2026-10-28 `--canon-only` ruling is not implemented on main; the 2026-11-10
  owner comment endorses the shipped Option 2 instead.
- #684 Part A: `CappedFileWriter` sits uncompiled on `lane/cure-684` at
  `b4302e39`. Parts B and C are absent from the tree. Untouched here.
- #545: draft plus RFC 7748 vector check and the rand-0.8 inventory sit on
  `lane/cure-545` at `f5a11779`. Untouched here.
- #612: already merged, landed on main at `946400c62b92a684e458b48578d18042d33d76fb`
  per its PR page. It appears in no car table, so the generated queue does not
  list it at all; the Batch D entry in the carrier text is stale.

## Pending for a session that holds the write channel

1. Claim on #602 and post this stand-up with the NOT CHECKED canonical line.
2. Open the PR for `lane/cure-579`. It is a Python/orchestration change with no
   protected path, so it needs the standard CI run on the exact head plus the
   finale checklist - not the rule-8 gate. It is not READY-TO-MERGE until that
   rail reads green, and this session could not read the rail at all.
3. Decide the two open owner questions carried from the earlier passes: the
   `KeyPair::generate()` shape in the #545 draft, and the #633/#538 disposition.
4. Read the ledger, then apply every DISPATCH entry past the last ack.

## Verification actually performed for this branch

- `python3 scripts/orchestrate_strict.py --dry-run` -> rc 0, 36 cars listed, then
  `[PLAN]` lines for the first ready cars (the dial is local: `dispatch_dial.py`
  plus `scripts/lanes.json`, no network).
- `python3 scripts/orchestration_contract.py --print-version` -> rc 0, `2.0.0`.
- `python3 scripts/v051_queue_sync.py --check` -> rc 0.
- `python3 scripts/v051_queue_sync.py` -> rc 0, wrote the 36-car queue.
- `python3 scripts/test_orchestration_v2.py` -> rc 0, 27 tests OK.
- `bash scripts/docs_sync_check.sh` -> rc 0, PASS.
- `python3 scripts/validate_handoff_scope.py ...` -> rc 0, SCMessenger-only.
- `python3 scripts/rules_check.py --staged` -> rc 0 for the committed set.
- No emoji in any changed file; brace and paren balance checked on the edited
  Python.

Nothing here is compiled Rust; nothing here needed to be. No CI status was read
and no claim is made about one.
