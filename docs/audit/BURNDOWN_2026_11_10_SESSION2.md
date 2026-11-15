# Burndown session 2 -- 2026-11-10

Status: Active record
Tracker: #602
Lane: `lane/burndown`

Every state claim below was read in this session; the command or tool is named beside it.
Main remains UNVERIFIABLE until the CircleCI port lands (`git ls-tree -r origin/main` has no
`.circleci`; `lane/burndown` and `ci/circleci-port-main-20261108` both do).

## Closed or advanced

| Item | State |
|---|---|
| 1. Device-log capture | DONE -- 63 files, 312,213,991 bytes, read back from the ledger remote |
| 2. CircleCI port #677 | Mechanism PROVEN, single act named, needs write access this seat lacks |
| #622 completion-gate key mismatch | FIXED `95b077c06`, CI-green on the job that runs its test |
| #565 cfg misclassification | FIXED `788ae12be`, CI-green on the job that runs its test |
| #679 (new) | filed -- the 4.19 GB runaway retry loop found by the capture |
| #178, #208 | DECISION-READY with fresh reads and reproduced defects |

## #677 -- the mechanism is settled

A docs-only commit pushed to `lane/burndown` produced **12 `ci/circleci:*` contexts within seven
seconds** (6 success, 6 pending at first read; 04:26Z). Same repository, different branch, zero
branch filters in the 972-line ported config. So the port head's zero contexts is explained
entirely by it never having been pushed since the rail existed. Both earlier candidates (queued
behind concurrency; project not wired) are retired.

The one act: push any commit to `ci/circleci-port-main-20261108`. That branch belongs to the
cloud session and the owner fence makes it read-only for this seat.

## #622 -- reproduced, then fixed

The original gate, loaded from `HEAD` and fed a genuine pack from the real helper, rejected it:

```
missing: canon_identity, canon_routing_feed, instruction_matches
unexpected: the five bucket.qid ids
```

`_derive_signals` assumed the pack carries canonical question names as keys. The real pack keys
answers by `bucket.qid`. The fix resolves a canonical question under its bare key or its single
`bucket.qid` key, keeps bare-key answers strictly `noul`, and accepts only the real pack's 21
keys. Scoring now aggregates over every answer in the pack; the old three-key view would have
ignored a genuine 0.40 on `identity.recipient_parse_safe`.

Verified: `scripts/test_orchestration_v2.py` 26/26 locally, and in CI `ci/circleci: Orchestration
control plane` = success on `95b077c06` (`.circleci/config.yml:679` runs that file).

## #565 -- the issue named one direction, there were two

The issue asked that `cfg(all(test, ...))` be recognised as test-only. The gate's two substring
checks were wrong both ways:

| Predicate | Old answer | Correct |
|---|---|---|
| `cfg(all(test, not(target_arch = "wasm32")))` | production | test-only |
| `cfg(any(test, feature = "x"))` | test-only | production |

The second is the more dangerous direction: `any(test, X)` holds when `test` is absent, so the
item can be compiled into production and the gate was skipping it.

Replaced with a three-valued evaluation: `test` is pinned True and False in turn while every
other atom stays unknown, and an item is test-only only when the predicate cannot hold without
`test`. Unparsable predicates fail closed as production, because skipping code is the
unrecoverable error for this gate and a false positive is a visible one.

Verified: 23/23 locally (four new cases), whole-repo gate clean at exit 0 with 0 allowlisted, and
in CI `ci/circleci: Rust Linting` = success on `788ae12be` (`.circleci/config.yml:615-616` runs
both commands).

## #208 -- both defects reproduced at the head

Head `11f3e5e93f2a8bc3ddab5e61f573b6dbdb5e1d4c`, unmoved, `+2008/-164`, 16 files, DIRTY.

- `rustfmt --check` on the head's own `cli/src/ble_daemon.rs` exits 1 with the diff at line 76,
  byte-identical to what `Rust Linting` and `Lint` report -- so those two jobs are one defect.
- `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md` at the head ends `` ``` `` + `\n\n` -- a
  blank line at EOF. Main's sibling ends with a single `\n`. Branch-local, one byte to remove.

`docs/audit/red_head_remediation/208-hygiene-and-rustfmt.patch` fixes both and was already
delivered as a PR comment (`6661021545`).

## A10's delivery premise needs revisiting

A10 was refuted on `gh api repos/pixiegirlchristy/SCMessenger --jq .permissions` returning
`push:false`. That field describes direct repo access, not the `maintainerCanModify=true` grant
both fork-headed cars carry. A dry-run push to the fork was authorised (`* [new branch]`, exit 0),
so the delivery route is probably open.

This is evidence, not proof: a dry-run transfers no pack. A definitive answer needs one real push
of a throwaway ref. That was not done -- writing to a third party's repository is an owner
decision, and A10's re-homing is recorded as owner-held.

Two options, both operator-held:

1. Authorise one push of the existing patch to the fork branch (the author's
   `maintainerCanModify=true` is their consent).
2. Re-home the car: land the same change on a branch in this repository and close the fork PR as
   superseded.

For #178 the same delivery question applies, but its gate is heavier: 13 conflicts of which five
are the rules contract (`AGENTS.md`, `GEMINI.md`, `docs/rules/DELEGATION.md`,
`docs/ORCHESTRATION.md`, `.agents/rules/follow-all-rules.md`), plus rule-8 gated
`core/src/transport/ble/gatt.rs`, plus iOS Swift files whose gates need `xcodebuild` on the
operator's Mac.

## Blockers

B1 -- clause 10 has no non-author reviewer: `reviewDecision` empty on every car, one collaborator
who is the author of record. Owner action: name a reviewer or waive clause 10 on #602.

B2 -- hosted Actions jobs are billing-refused (`The job was not started because your account is
locked due to a billing issue.`, #507 check-run 127192677132). #507 changes one Kotlin file, so
inert-diff routing sends its required context to the locked pool; #506/#508 change Rust and read
5/5 required green on OCI under the same lock.

#642 is coupled to B2, not independent of it: its proposed routing change would move contexts that
currently run onto the pool that cannot start, turning a green-if-vacuous required context red on
every Kotlin-only head. That is a CI-owner decision with the billing state as an input.

## Verification limits of this session

`scripts/disk_budget.py` reads BLOCKED (5.77 GB free of 236.25 GB), so no cargo or gradle build
ran here at all. Both fixes are Python and were verified by running their suites directly and
then by reading the exact CI job that runs them. This host is also not authoritative for the
project's Windows and physical-device gates.
