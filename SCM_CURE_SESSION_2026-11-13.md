# SCM-CURE session record, 2026-11-13

Lane: scm-cure. Base: `main` @ `ced2189e9d036d389d4aa4e7e40d42357a61f5c6`
(`git rev-parse HEAD`, 2026-11-13T01:54Z; `git fetch origin main --depth=50`
reported the same SHA, so the checkout is current as of that read).

This note is filed in the repository because the ledger repository could not be
reached from this session. See "Bootstrap deviation" below.

## Stand-up

Canonical check: **NOT CHECKED**. This session runs in a cloud sandbox whose home
directory is not the operator's machine, and none of the three doctrine key
locations resolved here:

- `~/.typesafe_key`: No such file or directory (`ls -la ~/.typesafe_key`)
- `~/SCM*` (the SCM ORC folder keys): No such file or directory (`ls -d ~/SCM*`)
- `~/Documents/GitHub` (Harness-configured working root): No such file or
  directory (`ls -d ~/Documents/GitHub`)

No key was printed and no key was created. This is a NOT CHECKED line, not a
claim that no key exists on the operator's machines.

Toolchain: `which cargo rustc python3` returns `/usr/local/bin/python3` only, so
the Python/CI-only rail is honoured by fact, not by choice. Disk:
`df -h /workspace` reports 16 G total, 5.3 G used, 9.6 G available (36%).

## Bootstrap deviation, filed once

- `git ls-remote https://github.com/Treystu/slow-lane-ledger.git` ->
  `fatal: could not read Username for 'https://github.com'`.
- `GET https://api.github.com/repos/Treystu/slow-lane-ledger` -> HTTP 404 for an
  anonymous caller, i.e. the ledger is not publicly readable.
- `gh auth status` -> "You are not logged into any GitHub hosts".

Consequence: `dispatch/scm-cure/PROMPT.md` (standing prompt v4),
`dispatch/scm-cure/DISPATCH.md` (entries past the last ack) and the Lucas Gate v2
material under `dispatch/gates/` were all unreadable. The session ran on the
carrier text alone, as the carrier directs when the ledger is unreachable.

Second consequence, and the material one: this session has no GitHub write
channel. It cannot claim on #602, cannot post a digest or a verification comment,
cannot open a PR, and cannot read check-runs or CircleCI statuses. The only
authenticated write available is a branch push to Treystu/SCMessenger. Every
"file this" step below is therefore discharged as a committed record on
`lane/cure-684`, and every tracker-side action is listed as pending for a session
that holds the channel.

## #622, key mismatch: verified cured cold on current main

The queue header orders this first. It is no longer dead on arrival on `main`;
this session re-derived that independently rather than reading the earlier claim.

- `python3 scripts/test_orchestration_v2.py` -> `Ran 26 tests ... OK`, including
  `test_completion_gate_accepts_the_real_bucketed_pack_it_is_handed_in_production`
  (`scripts/test_orchestration_v2.py:590`), which is the key-format regression
  test the owner ruling required.
- End to end with the real helper, no fixtures: feeding
  `jev_canonical_check.build_questions(["identity","routing","instruction"])`
  (keys `identity.canon_identity`, `identity.recipient_parse_safe`,
  `routing.canon_routing_feed`, `routing.callbacks_validated`,
  `instruction.instruction_matches`) into `_derive_signals`:
  - all-pass -> `(0.95, 0.95, True)`;
  - `instruction.instruction_matches` at 0.02 -> `(0.02, 0.95, True)`, i.e. it
    fails the supported floor as it must;
  - bare three-key pack -> `(0.93, 0.0, False)`, the pre-bucket shape still
    validated.
  The original failure mode (a genuine pass rejected with
  "answers do not match the canonical questions") no longer reproduces.

Residual against the 2026-10-28 owner ruling in #622 (canonical contract is
schema 1.0.0, three bare Noul keys, emitter gains `--canon-only`, the gate does
not learn bucket keys, no dual-key shim): that ruling is **not** implemented on
main. `grep -n "canon-only\|canon_only" scripts/jev_canonical_check.py` returns
nothing, and the gate resolves `bucket.qid` keys by design. `main` instead
carries the 2026-11-10 owner comment's Option 2, which the same owner voice
described as "Fix verified" with a CI read-back. Newest owner word supersedes the
earlier ruling, so this is recorded as superseded, not filed as a defect. No
issue is filed for it; a session with the write channel should say so on #622 and
let the owning plan step close it.

## #684, bound the mesh log: Part A was bounded across restarts only. Cured here.

The queue's second item. Part A shipped on main (cap constant, start-up
rotation, one audit line, two tests), but the bound is enforced **only at
process start**:

- `core/src/store/tracing_init.rs:87` opens
  `tracing_appender::rolling::never(...)`, which applies no size policy at all.
- `grep -rn "MESH_LOG_CAP_BYTES\|rotate_oversized_log"` shows the cap is read
  only inside `init_file_tracing` and the tests. Nothing checks length after the
  appender is open.

So a session that outlives the cap crosses it. At the rate #679 measured
(~140 MB/day at the default `info` filter) a 64 MiB ceiling is about eleven hours
of sustained logging, and the node this hurts most is the one that never restarts:
the always-on cloud node runs for weeks. #679's 4.19 GB growth recurs without
any restart.

Cure in this branch (`core/src/store/tracing_init.rs`, ungated by AGENTS.md
rule 8 because it is `core/src/store`): a `CappedFileWriter` sink that counts the
bytes it writes and rotates to `<path>.1` at the same `MESH_LOG_CAP_BYTES`,
keeping the existing two-file shape and therefore the `2 * cap` total. Each
rotation writes one JSON line straight to the new file
(`event: mesh_log_rotated`, with `cap_bytes`, `rotation` and `rotated_to`) so the
bound is auditable from the file itself; it is written directly rather than
through `tracing` to avoid re-entering the appender that called the sink. The
handle is dropped before the rename because Windows cannot rename an open file,
and a failed rotation degrades to today's append instead of retrying a rename per
line. The start-up rotation is kept: it is what bounds a file written by an older
build.

Test added: `live_log_rotates_within_a_session_at_the_cap`, which crosses the cap
inside one sink and asserts the live file stays under the cap, the crossing write
lands, the rotation marker is present, and the pre-rotation bytes are preserved
in `.1`. AGENTS.md rule 2 is honoured (scratch space under repo-local `tmp/`).

Acceptance status: **partial**. The issue's Part A acceptance asks for a 24 h
device run whose log cannot exceed the ceiling. That run cannot be produced from
this sandbox. What is claimed here is the code and a unit test, nothing about
device behaviour.

Parts B and C are **not** implemented. `core/src/transport/swarm.rs:1471` still
carries only the `ROUTE_ATTEMPT_REASON_RETRY_CYCLE` string; no structural-mismatch
quarantine exists (`grep -rn` for quarantine in `core/src/transport` returns
nothing). Both touch `core/src/transport`, so they need the independent
adversarial rule-8 review this lane cannot author.

## C2 batch and the rest of the queue: read, not started, no claim

#686, #688, #689, #690, #691, #692 were read in full (bodies, 2026-11-13T01:5xZ).
All six are rule-8-gated Rust cures (routing/transport/reputation/cover traffic)
whose acceptance needs an adversarial review naming the head SHA and, for several,
a measurement re-run. With no compiler and no reviewer available here they were
not started, and no claim is made on any of them.

The anonymous GitHub API rate limit was reached
("API rate limit exceeded for 208.72.218.131") during this read, so #693, the
Batch D list (#545, #579, #580, #586, #604, #605, #612, #642), #650, #651, #619
and the #658 sweep were not read this session and are not dispositioned here.

## Pending for a session with the GitHub write channel

1. Claim the lane on #602 and post this stand-up with the NOT CHECKED canonical
   line above.
2. Post the #622 cold re-verification on #622 (evidence in this note) and let the
   owning plan step close it.
3. Open the PR for `lane/cure-684` and mark it READY-TO-MERGE only after the rail
   reads green on the exact head; Parts B and C stay open on #684.
4. Read the ledger, then apply every DISPATCH entry past the last ack.

## Verification actually performed here

- `python3 scripts/rules_check.py --staged` -> exit 0, no findings.
- No emoji in the changed file (scripted scan of characters above U+2100).
- Bracket balance of the changed Rust file after stripping strings and comments:
  30/30 braces, 236/236 parens, 19/19 brackets.
- `python3 scripts/test_orchestration_v2.py` -> 26 tests, OK.
- The Rust change is **not compiled**: this sandbox has no `rustc`/`cargo`. CI is
  the only verifier for it, and this session cannot read CI.
