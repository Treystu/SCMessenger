# Defect inventory: the four genuine required-context reds, 2026-11-08

Status: filed, evidence only. No workflow, source file or check on any of these PRs was modified. All four are
other-authored heads the lane cannot push to (A10), so this is a handoff to the CI owner, not a repair.

Scope: the four PRs the census found failing a **required** context for a non-billing reason - #528, #538, #633,
#647. Every failing check-run on each head was read, its annotations fetched, and its check-run `output`
payload read.

## Method and its limit, stated first

- `commits/<head>/check-runs` paginated to `total_count` per head: 38 failing check-runs in total
  (#528 12, #538 14, #633 8, #647 4).
- `check-runs/<id>/annotations` per failing record: **81 annotations total**.
- `check-runs/<id>` output payload per failing record: **`output.summary` is empty on all 38**.

The limit matters and should be quoted to anyone who asks why the lane did not simply classify these from the
API: **the annotation tree carries no diagnostic.** Across all 81 annotations every genuine failure reports one
of two things - `Process completed with exit code N` against a `.github` workflow line number, or a deprecation
notice. There is not one compiler diagnostic, one failing test name, or one file:line from project source. So
annotation-driven triage, which is how every prior car disposition was built, cannot classify a genuine defect at
all. Job logs are mandatory, and the run ids below make that one command per run.

## Root causes found

Two are now identified exactly. Both were invisible in the check-run tree.

### #528 `Lint` - a pinned Rust toolchain version that does not exist

```
info: syncing channel updates for 1.120.0-aarch64-unknown-linux-gnu
error: could not download nonexistent rust version `1.120.0-aarch64-unknown-linux-gnu`:
  could not download file from 'https://static.rust-lang.org/dist/rust-1.120.0-aarch64-unknown-linux-gnu.tar.gz.sha256'
##[error]Process completed with exit code 1.
```

The workflow installs `dtolnay/rust-toolchain@1.120.0` (run 41838384667, log line 178). Rust 1.120.0 does not
exist, so the install fails before any lint runs. This is a workflow pin defect, not a source defect: it will
fail identically on any head carrying it, and it is not curable by re-running.

### #647 `Repository Hygiene Checks` - trailing whitespace

```
[ERROR] ERROR: Trailing whitespace found
##[error]Process completed with exit code 1.
```

Run 40122748073. A real hygiene violation in the PR content, and the one defect in this set that is a genuine
content fix (the hook refuses it by design).

## Not yet identified: the exit-101 cluster

Ten of #538's fourteen failures and four of #633's eight exit **101**, which is the Rust/Compose toolchain's
abort code rather than a test failure. The logs for those runs were not read in this pass. Run ids, so the read
is exact:

- #538: `39743241670` (most jobs), `39743241596` (iOS/Android), `39743241715` (forgery tests),
  `39743241863` (macOS native / iOS simulator), `39743241607` (WASM).
- #633: `39949734561`, `39949734533`, `39949734343`, `39949734447`.
- Reproduce: `gh run view <run-id> --repo Treystu/SCMessenger --log-failed`.

## Per-PR failing records

| PR | check-run | exit | class |
|---|---|---|---|
| #528 | Lint | 1 | toolchain pin (see above) |
| #528 | Rust Linting | 1 | warning `Unexpected input(s) 'toolchain'` on the setup action |
| #528 | Test (ubuntu-latest) | 1 | unclassified, log required |
| #528 | Repository Hygiene Checks | 1 | unclassified, log required |
| #528 | Kotlin Linting, Swift Linting, Windows CLI Artifact, Test (macos-latest), Test (windows-latest), WASM, label, JavaScript Linting | 0 steps | billing-lock refusals, not defects |
| #538 | Lint, FFI Surface Contract, Docs, Test (ubuntu-latest), Test (macos-latest), iOS Build, iOS Build & Simulator Test, macOS Native Tests, Forgery/Negative Security Regression Tests, WASM | 101 | exit-101 cluster, log required |
| #538 | Test (windows-latest), Windows CLI Artifact, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #633 | Lint, WASM, macOS Native Tests, iOS Build & Simulator Test | 101 | exit-101 cluster, log required |
| #633 | Test (windows-latest), Windows CLI Artifact, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #647 | Kotlin Linting, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #647 | Repository Hygiene Checks | 1 | trailing whitespace (see above) |

## What this hands to the CI owner

1. #528's `Lint` and every head carrying `dtolnay/rust-toolchain@1.120.0` fail for a version that has never
   existed. Grep the workflows for that pin before diagnosing any Rust lint failure anywhere in the repository.
2. #647 needs a whitespace fix in its own content.
3. The exit-101 cluster in #538 and #633 needs five and four log reads respectively; the run ids above are the
   whole remaining work, and none of it is lane-pushable.

## What this does not claim

It does not claim any of these PRs is otherwise landable, and it changes no classification of the car set: the
cars are billing-only or conflict-only as the census found. It also does not assert a root cause for the exit-101
jobs, which is why they are listed as unread rather than guessed.

GATE: local pass (decision-signal) - hand the CI owner named defects with their verbatim diagnostics instead of
counts, and state the API's own limit rather than presenting a classification the API cannot support.
