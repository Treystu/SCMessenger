# Cluster burndown - 2026-11-09 (C1 -> C2 -> C4 -> C3(#178) -> C5 -> #208, all 44 open PRs)

Status: filed, evidence only. Nothing merged, closed, refreshed, labelled or re-run producing this record.

Reads in this file were taken 2026-11-09T09:20Z-09:32Z. Every row below is a fresh read on the head
that exists now. The previous census (`CLUSTER_CENSUS_2026-11-08.md`) is superseded for counts, not
for findings that still hold; where a number moved, the move is recorded in section 7.

## 0. Fence, restated because it decides everything else

**main is UNVERIFIABLE.** `git cat-file -e origin/main:.circleci/config.yml` is ABSENT at
`ba7fa7785` (read 09:22Z), so no car based on main can produce a CircleCI battery. The rail config
exists only on `lane/burndown` and on the port branch `ci/circleci-port-main-20261108` (#677).

## 1. The port (#677) - C1's first item, and the only item that changes what is possible

### 1.1 Head read

| field | value |
|---|---|
| head | `53a0e5d83beab844df26f42af118ed7d62b99479` |
| base / behind | `main` / **0** |
| mergeable / state | **MERGEABLE** / UNSTABLE |
| required contexts (5) | **5/5 success** - `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope` |
| failing check-runs | 8 - `Test (windows-latest)`, `Test (macos-latest)`, `Windows CLI Artifact`, `Swift Linting`, `Kotlin Linting`, `JavaScript Linting`, `WASM`, `label` |
| failure class | **8 of 8 billing-annotated**: each carries `The job was not started because your account is locked due to a billing issue.`, `runner_name` empty, no steps executed |
| diff | 1 file, `.circleci/config.yml`, +972/-0 |
| reviews | 1 (`cubic-dev-ai[bot]`, COMMENTED, "All reported issues were addressed") - not an approval, not non-author human |

Clause (f) of the standing merge override is satisfied on this head: every failing non-required check
was read individually by id and is a billing refusal, not a defect.

### 1.2 The rail battery is absent at the current content, and that is the blocker

`commits/<head>/status` filtered to `ci/circleci:*`:

| commit | subject | `ci/circleci:*` contexts |
|---|---|---|
| `f3b0db3cb241` | port the rail config to main | **12, all success** |
| `201d2c1c9332` | fix the four review-thread defects | **0** |
| `53a0e5d83bea` | re-fire the rail on an unchanged tree | **0** |
| `a00dda682f79` (`lane/burndown`, 17:16:55Z same day) | docs head | **12, all success** |

So the rail does report on this repository - it posted twelve green contexts on `lane/burndown` in the
same minute the port branch received none. What has never run is the **post-fix config**: `53a0e5d8`'s
tree is identical to `201d2c1c`'s (`git diff --stat` empty), and `201d2c1c` is the commit that added
the four review-thread fixes (17 insertions, 5 deletions) on top of the green head.

Supporting reads: the CircleCI webhook is present and active
(`https://circleci.com/hooks/github`, events `push,pull_request`), and the config at `53a0e5d8` parses
as valid YAML (`12 jobs`, workflow `ci`). A webhook is not a pipeline and YAML validity is not CircleCI
schema validity, so neither proves the config good.

**Standing merge law is therefore not met: there is no green CircleCI battery at this head.** The PR
was NOT merged. Candidate causes, each with the read that would separate it, are filed on #677:

1. **CircleCI pipeline not started** (plan/credit refusal, or a dropped pipeline) - separated by the
   project's pipeline list for `53a0e5d8`; needs a CircleCI token, which is unset in this session
   (`CIRCLECI_TOKEN`/`CIRCLE_TOKEN`/`CIRCLECI_CLI_TOKEN` all unset).
2. **Config error at processing time** (CircleCI rejects the config after the webhook fires and posts
   no commit status) - separated by the same pipeline list; a local CircleCI CLI validate also needs a
   token.

Both are one owner read apart and neither is a code question. This is the single fence under every
landing in the backlog, so it is filed rather than worked around.

## 2. C1 - Batch D landing decision

| Car | Head | Draft/mergeable/behind | Required | Fails | Gated paths | Reviews | Disposition |
|---|---|---|---|---|---|---|---|
| #677 port | `53a0e5d8` | no / MERGEABLE / 0 | 5/5 | 8 billing | 0 | 1 bot COMMENTED | **BLOCKED - rail battery absent at current content** (section 1) |
| #660 combined | `210b7d932f4e` | no / MERGEABLE / 0 | 5/5 | 13 billing | **3** | 0 | **DECISION-READY** - clause 10 + A2 unmet |
| #550 | `6441203a1617` | no / MERGEABLE / 46 | 5/5 | 0 | 0 | 0 | **DECISION-READY** - superseded by #660 |
| #554 | `5b3efc47de11` | no / MERGEABLE / 46 | 5/5 | 0 | 3 | 1 | **DECISION-READY** - superseded by #660 |
| #556 | `9282583a93e4` | no / MERGEABLE / 46 | 5/5 | 0 | 0 | 0 | **DECISION-READY** - superseded by #660 |
| #509 | `32e64212cf17` | no / MERGEABLE / 51 | 5/5 | 0 | 0 | 0 | **DECISION-READY** - superseded by #660 |

#660 is not merely review-blocked: it touches **3 gated paths**, so the standing override's clause (d)
requires an A2 APPROVE naming `210b7d932f4e`, and no such verdict exists. #554's single review is
`Treystu` COMMENTED - the account this lane runs as, and not a pass - so clause 10 is unmet there too.

## 3. C2 - CI lane

| Car | Head | Draft/mergeable/behind | Required | Fails | Disposition |
|---|---|---|---|---|---|
| #611 | `e03c1d7df43a` | no / MERGEABLE / 0 | 5/5 | 8 billing | **DECISION-READY** (rail) |
| #643 | `0340c094468f` | no / **CONFLICTING** / 34 | 5/5 | 0 | **DECISION-READY (CI owner)** - pool re-scope; carve-out half already on the lane |
| #609 | `4ed8168bbf1f` | no / **CONFLICTING** / 46 | 5/5 | 0 | **DECISION-READY (infra owner)** - `ubicloud-standard-2` has no registered runner |

## 4. C4 - Batch C platform cars

| Car | Head | Draft/mergeable/behind | Required | Fails | Disposition |
|---|---|---|---|---|---|
| #506 | `16b4edf7540b` | no / MERGEABLE / 0 | 5/5 | 11 billing | **DECISION-READY** (rail + clause 10) |
| #507 | `f4d5155d4af5` | no / MERGEABLE / 0 | **4/5** (`Test (ubuntu-latest)` red, billing-annotated) | 11 billing | **BLOCKED - A25 contract**: inert Kotlin-only diff routes the required leg to the locked hosted pool |
| #508 | `475816012256` | no / MERGEABLE / 0 | 5/5 | 11 billing | **DECISION-READY** (rail + clause 10) |

## 5. C3 - #178 first

| Car | Head | Draft/mergeable/behind | Required | Fails | Gated paths | Disposition |
|---|---|---|---|---|---|---|
| #178 | `c1b457bbb1ce` | no / **CONFLICTING** / 1137 | **0/5 published** | 3 other: `iOS Build` exit 65, plus two Node-20 deprecation notices | **1** (`core/src/transport/ble/gatt.rs`) | **OWNER-HELD (A6)** - fork head, lane cannot push; iOS gates are `xcodebuild`-only on the operator's Mac |

## 6. C5 and #208

| Car | Head | Base | Required | Fails | Gated | Disposition |
|---|---|---|---|---|---|---|
| #521 | `5c6b5e52ed29` | main, 0 behind | 5/5 | 14 billing | **40** | **DECISION-READY** - needs a non-author verdict (A16), not code |
| #599 | `a7ef2f299753` | **`feat/conn-admission-dynamic`** | 0/5 published | 0 | 6 | **PARKED** - cannot produce required contexts while based there; unblocks when #521 lands |
| #208 | `11f3e5e93f2a` | main, 807 behind | **1/5** (`Repository Hygiene Checks`, `Rust Linting`, `Lint` red) | 6 (3 required + 3 deprecation) | 0 | **DECISION-READY (fork author)** - genuine reds unchanged; root causes and patches already filed on the PR (A29/A30) |

## 7. The full population, including every dependabot row

44 open PRs at read time (was 43). Two population changes since the 2026-11-08 census: **#654 closed**
(`closedAt` 2026-11-07T18:55:28Z, unmerged) and **#678 opened**, stacked on the port branch.

| PR | Cluster | Required | Fails | Class |
|---|---|---|---|---|
| #678 | C1 add (#651 fix) | **0/5 - base is the port branch** | 1 billing | **STACKED** - inherits #677's blocker; cannot publish required contexts on main until the port lands |
| #489 | HELD | 5/5 success | 0 | **HELD** - read for the census row only: `fc3540e481bc`, `CONFLICTING`, 89 behind, 44 files, 39 gated, 0 required red. No action. |
| #656 | straggler | 5/5 | 0 | **DECISION-READY (CI owner)** - draft; path filters for heavy workflows interact with required-context definition |
| #657 | straggler | 5/5 | 0 | **PARKED** - draft; author lane decides finish or close |
| #216 | identity chain | 0/5 - base `feat/identity-id-unification` | 3 (deprecation + `iOS Build` "Failed to resolve action download info") | **DECISION-READY (identity chain)** - draft, 807 behind its base |
| #220 | identity chain | 0/5 - same base | 2 deprecation | **DECISION-READY (identity chain)** |

Dependabot population, 20 cars, each read individually at its current head:

**D-RED - genuine failed required context (4).**

| Car | Head | Red | Read |
|---|---|---|---|
| #528 | `1e523e61c4a8` | `Rust Linting`, `Test (ubuntu-latest)`, `Lint`, `Repository Hygiene Checks` (1/5 success) | 8 billing + 4 executed failures; the toolchain pin is `1.120.0`, a version that has never existed (filed read, `RED_DEFECT_INVENTORY_2026-11-08.md`) |
| #538 | `990d7ad28e6b` | `Lint`, `Test (ubuntu-latest)` (3/5) | 14 executed failures, one cause: `rand_core` / `x25519-dalek` trait mismatch; 3 gated paths; rule-8 review absent |
| #633 | `b605d0c08dab` | `Lint` (3/5) | 8 executed failures, one cause: `rand` 0.9 call-site migration |
| #647 | `582cad1ea918` | `Repository Hygiene Checks` (4/5) | exit 1, trailing whitespace in its own content |

**D-PARK - behind, no required red (12).** #664 (`cbcf4bf16fa6`, 12 billing), #663 (`ce475f431b1b`, 8 billing),
#662 (`62ba6d14430f`, 8 billing), #646 (`c0e91ee3cfaf`, 0), #645 (`7752fce95615`, 0), #635 (`de27304f982c`, 0),
#634 (`e9cb31d880b4`, 0), #632 (`50c1a661823d`, 0), #631 (`21165ad9cd58`, 4), #630 (`08af5fa64924`, 0),
#542 (`b3e1fe82dd6f`, 2), #540 (`77bbec27f3fa`, 3).

**D-LOW-RISK-SMALL - one line, no required red (4).** #536 (`f95dd9e9bd63`, 0), #535 (`32bb0fae9ba0`, 0),
#534 (`9c049fcf7780`, 1), #529 (`b5fa9b135bb0`, 1).

Two classification corrections forced by this read, stated rather than buried:

- **#631 is no longer "cancelled, not a defect".** Its four failures include `iOS Build & Simulator Test`
  exit **101**, an executed failure. The other three carry only Node-20 deprecation notices. It stays
  D-PARK because no *required* context is red, but the earlier "cancelled" label was wrong.
- **#529, #534, #540, #542 carry executed failures with no diagnostic in the annotation tree** - only a
  Node-20 deprecation notice. They are not classified as defects here because the annotation tree
  cannot say, and job logs were not fetched for them. They are **non-required** failures, so they do not
  block the required set, but "no red" must not be read as "green".

## 8. What this changes

1. **#677 is closer than any prior record says and still not landable.** Five of five required contexts
   are green on the head, all eight failures are billing, and the branch is 0 behind. The single missing
   gate is the rail battery, and the missing battery is on the two commits that carry the review-thread
   fixes - not on the config that was already proven green.
2. **Nothing else moved.** Twelve cars are clean on required contexts and blocked on the rail; four
   dependabot cars are genuinely red; three cars are dirty-only; two are stacked on non-main bases.
3. **No car can land today without either the rail battery or an owner decision**, which is the same
   three-fence shape the 2026-11-08 record filed, now with #677 measured rather than assumed.

## 9. Next action

Owner read: the CircleCI pipeline (or absence of one) for `53a0e5d8` / `201d2c1c` on project
`Sovereign-Communication/SCMessenger`, or a `CIRCLE_TOKEN` for this lane. If that pipeline exists and is
green, #677 merges under the standing merge law with the gate receipt in section 1 attached; if it does
not exist, the port needs a re-fire or a config fix before main becomes verifiable. In parallel, #528's
repair (drop the `toolchain:` input, keep the pin in the action ref) is the one dependabot change whose
absence reds `Rust Linting` for every future commit on main.

GATE: local pass (decision-signal) - the blocker is filed with the exact missing artifact and the read
that separates its two candidate causes, instead of a "blocked" label standing in for a diagnosis.
