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

**Prior disposition, which this corroborates rather than discovers:** oversight comment `6552034042` on #653
(2026-11-01) already PARKED #528 on exactly this ground - the head moves 25 `dtolnay/rust-toolchain` refs from
1.99.0 to a 1.120.0 that returned 404, main's `rust-toolchain.toml` pins 1.99.0, and hygiene fails the pin check.
That comment carries the Jev assessment and the re-entry trigger. Nothing here reopens it; the lane's log read
independently reproduces the same 404 and the same pin disagreement.

### #647 `Repository Hygiene Checks` - trailing whitespace

```
[ERROR] ERROR: Trailing whitespace found
##[error]Process completed with exit code 1.
```

Run 40122748073. A real hygiene violation in the PR content, and the one defect in this set that is a genuine
content fix (the hook refuses it by design).

## Classified: the exit-101 cluster is two defects, one per PR

All nine runs were read with `gh run view <id> --log-failed`. The nine runs are not nine defects, and the two PRs do
not share a cause. Both are dependency-version breaks in Rust, and both are visible in every failed job of their PR,
which is why the job count was misleading: one compile error fans out across every matrix leg.

### #538 - one defect: `rand_core` version mismatch against `x25519-dalek`

```
error[E0277]: the trait bound `chacha20poly1305::aead::OsRng: x25519_dalek::rand_core::CryptoRng` is not satisfied
error[E0277]: the trait bound `ThreadRng: x25519_dalek::rand_core::CryptoRng` is not satisfied
error[E0277]: the trait bound `OsRng: x25519_dalek::rand_core::CryptoRng` is not satisfied
```

Every job in all five runs aborts on this one bound: Lint, Docs, FFI Surface Contract, Test (ubuntu/macos/windows),
Windows CLI Artifact, WASM, Forgery/Negative Security Regression Tests, macOS Native Tests, iOS Build,
iOS Build & Simulator Test, Android JVM Unit Tests, Android Debug APK. `x25519-dalek` is compiled against a
different `rand_core` than the RNG types passed to it, so the trait is unimplemented for those concrete types. This
is a dependency-graph fix (align `rand`/`rand_core` with what `x25519-dalek` expects), not fourteen code fixes.

| location | occurrences |
|---|---|
| `core/src/crypto/encrypt.rs:132` | 16 |
| `/Users/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-3.0.0/src/x25519.rs:217` | 16 |
| `/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-3.0.0/src/x25519.rs:88` | 15 |
| `core/src/privacy/onion.rs:211` | 12 |
| `/Users/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-3.0.0/src/x25519.rs:88` | 10 |
| `/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-3.0.0/src/x25519.rs:217` | 8 |
| `x25519.rs:217` | 8 |
| `/home/runner/.cargo-2/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-3.0.0/src/x25519.rs:217` | 8 |
| `x25519.rs:88` | 5 |
| `core/src/crypto/pq/hybrid.rs:149` | 4 |
| (33 further distinct locations in `tmp/log_run_*.txt`) | - |

### #633 - one defect class plus one missing trait import: `rand` 0.9 API removal

```
error[E0425]: cannot find value `OsRng` in module `rand::rngs`
error[E0432]: unresolved import `rand::RngCore`
error[E0425]: cannot find function `thread_rng` in crate `rand`
error[E0433]: cannot find `RngCore` in `rand`
error[E0599]: no method named `choose_multiple` found for struct `Vec<&circuit::PeerInfo>`
```

`OsRng`, `thread_rng` and `RngCore` no longer live where this code imports them from - the shape of a `rand` 0.8 to
0.9 migration applied to the manifest without migrating the call sites. `E0599 choose_multiple` is separate and
smaller: the method exists as `SliceRandom::choose_multiple` and needs that trait in scope. Every failed job in all
four runs is this same break: Lint, Test (windows-latest), Windows CLI Artifact, macOS Native Tests,
iOS Build & Simulator Test, Android JVM Unit Tests, Android Debug APK, WASM.

| location | occurrences |
|---|---|
| `core/src/identity/keys.rs:403` | 12 |
| `core/src/crypto/backup.rs:169` | 6 |
| `core/src/crypto/backup.rs:170` | 6 |
| `core/src/crypto/encrypt.rs:23` | 6 |
| `core/src/crypto/pq/hybrid.rs:21` | 6 |
| `core/src/crypto/pq/hybrid.rs:22` | 6 |
| `core/src/crypto/pq/mldsa.rs:5` | 6 |
| `core/src/crypto/pq/mod.rs:8` | 6 |
| `core/src/crypto/pq/mod.rs:9` | 6 |
| `core/src/crypto/ratchet.rs:19` | 6 |
| (134 further distinct locations in `tmp/log_run_*.txt`) | - |

### What this changes in the earlier classification

The nine runs were "unread" in the previous revision of this document and are now fully classified. Two of the four
genuine required-context reds are therefore single-cause defects that a dependency change repairs at once, which
matters for the owner: #538 and #633 are not "many failing jobs" and should not be triaged job by job. Neither is
lane-pushable (A10 - other-authored heads), and both sit BEHIND main with heads unchanged since 2026-10-24, so the
reads above are still the latest for those heads.

## Per-PR failing records

| PR | check-run | exit | class |
|---|---|---|---|
| #528 | Lint | 1 | toolchain pin (see above) |
| #528 | Rust Linting | 1 | warning `Unexpected input(s) 'toolchain'` on the setup action |
| #528 | Test (ubuntu-latest) | 1 | unclassified, log required |
| #528 | Repository Hygiene Checks | 1 | unclassified, log required |
| #528 | Kotlin Linting, Swift Linting, Windows CLI Artifact, Test (macos-latest), Test (windows-latest), WASM, label, JavaScript Linting | 0 steps | billing-lock refusals, not defects |
| #538 | Lint, FFI Surface Contract, Docs, Test (ubuntu-latest), Test (macos-latest), iOS Build, iOS Build & Simulator Test, macOS Native Tests, Forgery/Negative Security Regression Tests, WASM | 101 | classified: one `rand_core` mismatch |
| #538 | Test (windows-latest), Windows CLI Artifact, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #633 | Lint, WASM, macOS Native Tests, iOS Build & Simulator Test | 101 | classified: `rand` 0.9 migration |
| #633 | Test (windows-latest), Windows CLI Artifact, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #647 | Kotlin Linting, Android JVM Unit Tests, Android Debug APK | 1 | log required |
| #647 | Repository Hygiene Checks | 1 | trailing whitespace (see above) |

## What this hands to the CI owner

1. #528's `Lint` and every head carrying `dtolnay/rust-toolchain@1.120.0` fail for a version that has never
   existed. Grep the workflows for that pin before diagnosing any Rust lint failure anywhere in the repository.
2. #647 needs a whitespace fix in its own content.
3. The exit-101 cluster is read and classified above: #538 is one `rand_core`/`x25519-dalek` mismatch, #633 is a
   `rand` 0.9 call-site migration plus one missing `SliceRandom` import. Both are dependency fixes on
   other-authored heads, so neither is lane-pushable.

## What this does not claim

It does not claim any of these PRs is otherwise landable, and it changes no classification of the car set: the
cars are billing-only or conflict-only as the census found. The exit-101 jobs are no longer in that category: their
root causes are asserted from their own logs, with the error text and trigger locations recorded above.

GATE: local pass (decision-signal) - hand the CI owner named defects with their verbatim diagnostics instead of
counts, and state the API's own limit rather than presenting a classification the API cannot support.
