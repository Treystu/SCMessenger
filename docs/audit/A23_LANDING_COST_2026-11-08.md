# A23 landing-cost measurement, 2026-11-08

Status: measured. Every value below is a value the API returned in this session. Nothing is inferred; where a value
cannot be resolved, the row says so and names the reason.

Register row A23 asked for the landing cost of the cars that are already level with `main` (`behind_by=0`), because
those are the only ones whose merge costs no `update-branch` and no fresh required run. This is that measurement,
plus the ordered sequence it implies.

## Seat and authority at measurement time

- `lane/burndown` local head == `origin/lane/burndown`, 53 ahead / 0 behind `main`.
- Steering re-read newest-first on both surfaces: **0 comments newer** than the block-close posts `6719653761`
  (#653) and `6719654058` (#602). Required-status-check policy read read-only:
  `strict=true`; contexts `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`,
  `Handoff ownership scope`.
- `strict=true` is the whole reason the order below matters: a merge makes every other car behind, so each merge
  forces a fresh required run on every car that has not merged yet.

## The set is ten, not the eight the census recorded - and the eight is still right

Live read of all 42 open PRs (`pulls?state=open`, one page, 42 returned, no page cap) with a compare read per head
gives **10** non-draft cars at `behind_by=0` (#489 excluded as owner-protected, #653 excluded as this lane's own
PR): #506, #507, #508, #521, #528, #611, #660, #662, #663, #664.

The two extra are **#507** (`f4d5155d4af5`, updated `2026-11-05T09:35:37Z`) and **#528** (`1e523e61c4a8`, updated
`2026-11-04T13:28:14Z`). Both timestamps precede the 2026-11-08 census, so neither head moved between reads: the
census **undercounted by two**, the set did not change. That correction does not disturb the landing-eligible count,
because both extras are blocked:

- **#507** is `mergeable: true`, `mergeable_state: blocked`, with one red required context -
  `Test (ubuntu-latest)` `failure` `127192677132`.
- **#528** is `blocked` with four red required contexts - hygiene `126603174358`, `Lint` `126607622589`,
  `Rust Linting` `126634515150`, `Test (ubuntu-latest)` `126607622870` - and is already PARKED by oversight comment
  `6552034042`.

So: **10 cars at zero behind, 8 landable**, and the 8 are exactly the census's 8.

## Measurements

Required reds are the five protected contexts as read on each head. `pending` counts non-completed check-runs.
Diff is the PR's own additions/deletions against `main`. `gated` counts changed files under
`core/src/{crypto,transport,routing,privacy}/` or `vendor/`, which AGENTS rule 8 puts behind an adversarial review.

| # | head | required red | latest required run ids | pending | mergeable / state | files | diff | gated |
|---|---|---|---|---|---|---|---|---|
| 506 | `16b4edf7540b` | 0 | hygiene 127164876967; lint 127167080031; rust 127167203816; test 127167080057; handoff 127164878865 | 0 | true / unstable | 3 | +127 / -9 | 0 |
| 508 | `475816012256` | 0 | hygiene 127164931115; lint 127184903678; rust 127166774377; test 127184903717; handoff 127164931978 | 0 | true / unstable | 2 | +273 / -0 | 0 |
| 521 | `5c6b5e52ed29` | 0 | hygiene 127240409561; lint 127243054646; rust 127242751420; test 127243054834; handoff 127240410639 | 0 | true / unstable | 45 | +19423 / -100 | 40 |
| 611 | `e03c1d7df43a` | 0 | hygiene 127191285762; lint 127212203135; rust 127212911872; test 127212203097; handoff 127191283237 | 0 | true / unstable | 1 | +4 / -2 | 0 |
| 660 | `210b7d932f4e` | 0 | hygiene 126603538485; lint 126648237517; rust 126634391423; test 126648237730; handoff 126603540570 | 0 | true / unstable | 23 | +2318 / -254 | 3 |
| 662 | `62ba6d14430f` | 0 | hygiene 128495987887; lint 128505009595; rust 128496049997; test 128505009604; handoff 128495988605 | 0 | true / unstable | 1 | +1 / -1 | 0 |
| 663 | `ce475f431b1b` | 0 | hygiene 128496003334; lint 128505090697; rust 128496230715; test 128505090338; handoff 128496003459 | 0 | true / unstable | 2 | +7 / -7 | 0 |
| 664 | `cbcf4bf16fa6` | 0 | hygiene 128496724888; lint 128505057792; rust 128516082129; test 128505057787; handoff 128496724998 | 0 | true / unstable | 1 | +10 / -10 | 0 |
| 507 | `f4d5155d4af5` | **1** - `Test (ubuntu-latest)` 127192677132 | hygiene 127164899483; lint 127192677311; rust 127167142409; handoff 127164899320 | 0 | true / blocked | 1 | +28 / -10 | 0 |
| 528 | `1e523e61c4a8` | **4** - see above | - | 0 | true / blocked | 10 | +25 / -25 | 0 |

Conflicting files: **0 for all ten**, and this is a measured value rather than an inference - every head returns
`mergeable: true`, GitHub's own merge simulation, and no head returns `mergeable_state: dirty`. The API names no
conflicting path because there is no conflict to name.

Non-required reds: 8 to 13 per car, identical job family on every head (`WASM`, `Android Debug APK`, `Android JVM
Unit Tests`, `Android Wiring Gate`, `iOS Build & Simulator Test`, `macOS Native Tests`, `Test (windows-latest)`,
`Test (macos-latest)`, `Windows CLI Artifact`, `label`, `JavaScript Linting`, `Kotlin Linting`, `Swift Linting`).
That is the billing-locked pool set the 2026-11-08 census classified, which is why each car reads `unstable` rather
than `clean`. These are not re-read per-car for billing annotations in this measurement, so the attribution is the
census's, not a fresh one.

## Ordered landing sequence

Cost model, measured: every merge under `strict=true` re-behind-ifies the remaining cars, so step `k` of 8 costs
`8-k` revalidation cycles (one `update-branch` plus one fresh run of the five required contexts each). Ordering
cheapest-and-least-gated first therefore also means the expensive cars lose the fewest revalidations, and each step
leaves a small blast radius if it turns `main` red.

| step | car | measured cost | reason for this position |
|---|---|---|---|
| 1 | **#662** | 1 file, +1/-1, 0 gated, 0 red, 0 conflicts | Smallest possible diff in the set - a one-line `.github` edit. Nothing to review adversarially, nothing to revert widely if it is wrong. |
| 2 | **#611** | 1 file, +4/-2, 0 gated, 0 red | Same class as step 1, four lines larger. Second because it is configuration, not code, and its required set is fully green. |
| 3 | **#663** | 2 files, +7/-7, 0 gated, 0 red | Two `.github` files; still pure configuration, still no review gate, marginally wider than #611. |
| 4 | **#664** | 1 file, +10/-10, 0 gated, 0 red | One file but it is `Cargo.lock`. A lockfile change can move resolved dependency versions, so it sits after the pure-configuration cars and before any code change. |
| 5 | **#506** | 3 files, +127/-9, 0 gated, 0 red | First code car: `cli` (2 files) and `android` (1). Larger than the config cars but outside every gated directory. |
| 6 | **#508** | 2 files, +273/-0, 0 gated, 0 red | Fewer files than #506 but more than twice its added lines (`android`, `cli`), so it costs more review attention per file. |
| 7 | **#660** | 23 files, +2318/-254, **3 gated**, 0 red | First car that needs an adversarial review: `core/src/routing/local.rs`, `core/src/routing/optimized_engine.rs`, `core/src/transport/swarm.rs`. Large diff plus a review gate, so it goes after the ungated code cars. |
| 8 | **#521** | 45 files, +19423/-100, **40 gated**, 0 red | Largest diff in the set by an order of magnitude and the most gated surface (34 `vendor/` files plus `core/src/transport/*`). Last: the car with the highest review cost and the largest revert surface must not sit in front of seven revalidation cycles it would invalidate. |

### Excluded, with the exact reason

- **#489** - owner-protected, not measured, not read for anything but its presence in the population.
- **#507** - at zero behind but `blocked`: one red required context. It does not need a rebase, it needs a fix;
  placing it in the sequence would stall the train at whatever step it occupied.
- **#528** - four red required contexts and an existing PARK disposition (`6552034042`).
- Every other non-draft PR is `behind_by` 31 to 1137, so each costs an `update-branch` plus a full required run
  before it can even enter this ordering.

### Genuinely unmeasurable, with reason

- **Per-step wall-clock CI duration.** The API reports durations for runs that happened, not for a hypothetical run
  after a future merge, and the billing lock already makes observed durations unrepresentative (jobs that never
  started, `steps=0`). Recorded as unmeasured rather than estimated.
- **Whether `main` stays green after each step.** Pre-merge reads cannot report the post-merge state of a branch that
  does not exist yet. This is a reason the order front-loads small, ungated diffs, not a value that can be measured.
- **Cross-car semantic interactions.** Two cars can each be green and still conflict semantically. Change-level
  conflict is measured above (none); semantic interaction is not resolvable from check-run data and is not claimed.

## What this changes for the owner

The protection switch is still the gate, and this measurement does not touch it. What it adds is that the switch,
once made, has a defined first move: **eight landable cars, zero conflicts, zero red required contexts, ordered so
the four configuration-class cars land first and the two gated cars land last**, with the two zero-behind-but-blocked
cars named so they are not mistaken for part of the train.

GATE: local pass (decision-signal) - measured every value the API exposes, corrected the census undercount with
timestamps rather than narrating it, and recorded three unmeasured quantities with their reasons instead of
inferring them.
