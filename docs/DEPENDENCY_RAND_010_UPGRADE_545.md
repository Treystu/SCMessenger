# Draft: rand 0.8 to 0.10 upgrade, and what #545 actually needs

Status: **DRAFT. NOT READY-TO-MERGE. Nothing in this document has been compiled.**
This sandbox has no `rustc`/`cargo` (`which cargo rustc` -> python3 only), so every
Rust shape below is described, not executed. It touches `core/src/crypto`,
`core/src/privacy` and `core/src/identity`, so AGENTS.md rule 8 applies and an
independent adversarial review naming the head SHA is required before it can land.
It is also sequenced behind the dependabot park ruling recorded on #545
(2026-10-24) and the disposition of PR #633.

Issue: #545, "Upgrade rand 0.8 -> 0.10 (prerequisite for x25519-dalek 3.0)".

## 1. What the issue asks

Tasks, from the issue body: upgrade workspace `rand` 0.8 -> 0.10 (`OsRng` ->
`SysRng`, RNG is now fallible, update call sites); audit every `rand` API use for
0.8 -> 0.10 breakage; add RFC 7748 section 6.1 X25519 known-answer tests; full
crypto suite green on the new rand; then reattempt PR #538 (x25519-dalek 3.0).

Acceptance: `rand = "0.10"` in the workspace with zero rand-0.8 API uses left,
RFC 7748 KATs pass, and #538 becomes mergeable.

## 2. Premise check against this tree

### 2.1 Manifest and lock (read this session)

- `Cargo.toml:53` `x25519-dalek = { version = "2.0", features = ["static_secrets"] }`
- `Cargo.toml:59` `rand = "0.8"`; consumed by `core/Cargo.toml:27` and `:126`.
- `Cargo.lock` already holds **three** rand generations: `rand 0.8.7`, `0.9.5`
  and `0.10.2`; `rand_core 0.6.4`, `0.9.5`, `0.10.1`; and `x25519-dalek` both
  `2.0.1` and `3.0.0`.
- Reverse dependencies in the lock: the 0.10 copies arrive through
  `libp2p-*` 0.44 (core, gossipsub, kad, mdns, noise, ping, quic, relay,
  request-response, autonat, swarm, identity), `hickory-*` 0.26, `quinn-proto`
  0.11, `proptest` 1.12, `igd-next` 0.17 and `core-models` 0.0.7.
  `libp2p-noise 0.47` is what already pulls `x25519-dalek 3.0.0`.

Consequence: the upgrade changes the workspace's own `rand` edge only. It does not
collapse the tree to one rand; `3.0.0`'s `rand_core 0.10` arrives beside it either
way. Any "one rand version" claim in a PR body would be wrong.

### 2.2 Crate APIs (read from docs.rs, 2026-11-13)

`rand 0.10.2`:
- Free functions: `rng()`, `random()`, `random_range()`, `fill()`, `make_rng()`.
- Traits: `TryRng` (fallible base), `Rng` (infallible), `RngExt` (user-level
  sampling surface), `Fill`, `SeedableRng`, `CryptoRng`, `TryCryptoRng`.
- `rngs` module: `SysRng` behind feature `sys_rng`, a `TryRng` with error
  `SysError`; `ThreadRng` behind feature `thread_rng`, returned by `rand::rng()`.
  **There is no `OsRng` in the module.** The rename in the issue is correct.
- `rand_core` is re-exported; `RngCore` is deprecated in favour of `TryRng`.

`x25519-dalek 3.0.0`:
- `StaticSecret::random_from_rng<R: CryptoRng + ?Sized>(csprng: &mut R) -> Self`,
  where `CryptoRng` is **rand_core 0.10's**. This is the compile break the issue
  names, and it is real.
- Note the signature takes `&mut R`; the 2.0 call sites in this tree pass by
  value (`random_from_rng(rand::rngs::OsRng)`), so every site changes shape even
  if the trait question is ignored.
- `StaticSecret::random()` exists behind the crate's `getrandom` feature.
  `EphemeralSecret::random()` is shown in the crate's own front-page example.
- `From<[u8; 32]>` is implemented for `StaticSecret`.

**Unverified here, and load-bearing:** `ThreadRng` implements `TryCryptoRng`, and
`CryptoRng` has a blanket impl for `R: TryCryptoRng<Error = !>`. Whether that
blanket actually accepts `ThreadRng` depends on `ThreadRng`'s associated error
type, which this session did not read and cannot compile-check. So do **not**
assume `random_from_rng(&mut rand::rng())` compiles; the draft below avoids
depending on it.

### 2.3 Where the issue's scope is too small

`scripts/rand_010_migration_check.py` (added with this draft) scans every Rust
file under `core`, `cli`, `wasm`, `desktop_bridge` for the 0.8 API shapes:

```
total remnant hits: 118 in 28 files
  production: 63
  test:       55
    thread_rng: 17
    OsRng: 56
    RngCore: 36
    gen_range: 3
    gen_call: 2
    rand_Rng_import: 4
  production files (16): core/src/crypto/backup.rs, core/src/crypto/encrypt.rs,
    core/src/crypto/pq/hybrid.rs, core/src/crypto/pq/mldsa.rs,
    core/src/crypto/pq/mod.rs, core/src/crypto/ratchet.rs,
    core/src/identity/keys.rs, core/src/iron_core.rs, core/src/privacy/circuit.rs,
    core/src/privacy/cover.rs, core/src/privacy/onion.rs,
    core/src/privacy/padding.rs, core/src/privacy/timing.rs,
    core/src/transport/discovery_scheduler.rs, core/src/transport/reflection.rs,
    core/src/transport/swarm.rs
```

(`fill_bytes` is not counted as a remnant: 0.10 still provides it through
`Rng`/`Fill`. 54 occurrences are printed for visibility only.)

The issue names three files and "8+ call sites". The real surface is 16
production files and 63 hits, of which only 14 are the `random_from_rng` sites the
issue is about. The other 49 are non-crypto call sites in `privacy/`, `transport/`
and `iron_core.rs` that will fail on `thread_rng`, `gen_range` and `use rand::Rng`.

## 3. Path A: migrate the workspace edge (the issue's acceptance)

### 3.1 Manifest change

```toml
# Cargo.toml (workspace)
rand = { version = "0.10", features = ["sys_rng", "thread_rng"] }
```

`sys_rng` is required for `rand::rngs::SysRng`; `thread_rng` for `rand::rng()`.
Whether one of them is already default is to be confirmed from the resolved
feature set at implementation, not assumed here.

### 3.2 Mechanical transformation

| 0.8 shape in this tree | 0.10 shape | Sites |
|---|---|---|
| `rand::thread_rng()` | `rand::rng()` | 17 hits, `use` lines included |
| `use rand::Rng;` + `.gen_range(a..=b)` | `use rand::RngExt;` + `.random_range(a..=b)` | timing.rs:156-160, padding.rs:138-140, circuit.rs:173-175 |
| `.gen::<f64>()` | `.random::<f64>()` | timing.rs:163 |
| `rand::rngs::OsRng.fill_bytes(&mut buf)` | `SysRng.try_fill_bytes(&mut buf)?` (fallible) | every `OsRng` hit that is not a `random_from_rng` argument (56 `OsRng` hits in total, including `use` lines) |
| `use rand::RngCore;` + UFCS `rand::RngCore::fill_bytes(rng, buf)` | `use rand::Rng;` (infallible) or `TryRng` (fallible) | 36 hits, `use` lines included |
| `x25519_dalek::EphemeralSecret::random_from_rng(OsRng)` | see 3.3 | encrypt.rs:132, onion.rs:211, onion.rs:328 |
| `x25519_dalek::StaticSecret::random_from_rng(thread_rng())` | see 3.3 | onion.rs:620, :639, :806, :809 (tests), hybrid.rs:149-230 (tests) |

### 3.3 The 14 `random_from_rng` sites: pick a source explicitly

Three ways out, and the draft recommends choosing per site rather than uniformly:

1. **`EphemeralSecret::random()`** with `x25519-dalek`'s `getrandom` feature. No
   rand involvement at all, so the trait question in 2.2 disappears. Best fit for
   the pure-ephemeral sites: `encrypt.rs:132`, `onion.rs:211`, `onion.rs:328`.
2. **`StaticSecret::from([u8; 32])`** fed by the workspace RNG. This is already
   the shape of `pq/hybrid.rs:67`, which fills `ephemeral_secret_bytes`, builds
   `StaticSecret::from(bytes)` and zeroizes the buffer. Reuse that pattern for the
   remaining static-secret sites: it keeps one RNG story, keeps the zeroize step,
   and fails closed if the syscall fails.
3. `random_from_rng(&mut <rand_core 0.10 CryptoRng>)`. Only after the associated
   error type of the chosen RNG is settled; do not plan around it.

Recommendation: 1 for ephemeral, 2 for static. Both are small, both are provably
free of the unverified blanket-impl question, and neither weakens the zeroize
discipline already present in `hybrid.rs`.

### 3.4 The fallibility decision, which is an API break and therefore an owner call

`SysRng` is fallible. Today `KeyPair::generate() -> Self`
(`core/src/identity/keys.rs:242`) calls `rand::rngs::OsRng.fill_bytes(...)` and
cannot fail. Under 0.10 the same code either has to become
`Result<Self>`, retry, or panic.

- `Result<Self>` is honest and propagates, but it is a public API change inside
  `core/src/identity`: every caller of `KeyPair::generate()` must change.
- Panicking keeps the API and concentrates the failure in one place, at the cost
  of a crypto path that aborts.

Per AGENTS.md rule 9 an API-contract break is escalated, not improvised. This is
the one place in #545 where the drafting lane should stop and ask: **which of the
two shapes should `KeyPair::generate()` take?** Everything else in Path A is
mechanical.

### 3.5 Order of work

1. Manifest + `cargo update -p rand` (lock only, no other package).
2. Non-crypto sites first: `privacy/timing.rs`, `privacy/padding.rs`,
   `privacy/circuit.rs`, `transport/discovery_scheduler.rs`,
   `transport/reflection.rs`, `transport/swarm.rs:8490`, `iron_core.rs:2421-2422`
   and `:3259-3260`. These are `thread_rng`/`gen_range` renames with no error
   handling, and they prove the manifest change before the crypto surface moves.
3. Crypto fill sites: `backup.rs:180`, `encrypt.rs:143`, `pq/mod.rs:41,88`,
   `pq/mldsa.rs:59`, `ratchet.rs` (9 sites), `identity/keys.rs:245,291,296,419`.
   These all need the fallibility decision.
4. The 14 `random_from_rng` sites per 3.3.
5. Tests: the 55 test-classified hits, including `proptest_harness.rs` and the
   `core/tests/` files, so the suite exercises the new path rather than the old.
6. KATs (section 3.6).
7. `python3 scripts/rand_010_migration_check.py --enforce` must exit 0.

### 3.6 RFC 7748 known-answer test, as text to review (uncompiled)

Wire a new module in `core/src/crypto/mod.rs` (`pub mod x25519_kat;` or
`#[cfg(test)] mod x25519_kat;` — rule 16: a file nobody declares is dead):

```rust
//! RFC 7748 known-answer tests for X25519 derivation (#545).
//! Constants are validated independently by scripts/x25519_rfc7748_kat_check.py.

use x25519_dalek::{PublicKey, StaticSecret};

fn arr(hexstr: &str) -> [u8; 32] {
    let bytes = hex::decode(hexstr).expect("valid hex");
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

const ALICE_PRIVATE: &str = "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a";
const ALICE_PUBLIC:  &str = "8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a";
const BOB_PRIVATE:   &str = "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb";
const BOB_PUBLIC:    &str = "de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f";
const SHARED:        &str = "4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742";

#[test]
fn rfc7748_6_1_public_keys_and_shared_secret() {
    let alice = StaticSecret::from(arr(ALICE_PRIVATE));
    let bob = StaticSecret::from(arr(BOB_PRIVATE));
    assert_eq!(PublicKey::from(&alice).to_bytes(), arr(ALICE_PUBLIC));
    assert_eq!(PublicKey::from(&bob).to_bytes(), arr(BOB_PUBLIC));
    assert_eq!(
        alice.diffie_hellman(&PublicKey::from(&bob)).to_bytes(),
        arr(SHARED)
    );
    assert_eq!(
        bob.diffie_hellman(&PublicKey::from(&alice)).to_bytes(),
        arr(SHARED)
    );
}
```

This pins "the derivation does not change" to vectors. The same constants, plus
the section 5.2 single-call and iterated vectors, are checked here now:

```
$ python3 scripts/x25519_rfc7748_kat_check.py
[OK]   6.1 Alice public key: 8520f009...b4e6a
[OK]   6.1 Bob public key: de9edb7d...82b4f
[OK]   6.1 Alice-side shared secret: 4a5d9d5b...61742
[OK]   6.1 Bob-side shared secret: 4a5d9d5b...61742
[OK]   5.2 single call: c3da5537...28552
[OK]   5.2 after 1 iteration: 422c8e7a...ae3079
[OK]   5.2 after 1,000 iterations: 684cf59b...532c51
[INFO] 5.2 after 1,000,000 iterations: NOT EVALUATED in this script (pure-Python cost)
[DONE] every evaluated RFC 7748 vector matched
rc=0
```

Note the `hex` crate dependency in the test snippet: if `core` does not already
depend on it for test code, use literal byte arrays instead. That is a detail to
settle at implementation, listed here so it is not a surprise.

### 3.7 Acceptance mapping

| #545 acceptance | How it is met | verified here? |
|---|---|---|
| `rand = "0.10"` in workspace | 3.1 | no (manifest edit is trivial; compile is not) |
| zero rand-0.8 API uses remain | `scripts/rand_010_migration_check.py --enforce` exits 0 | the checker exists and currently exits 1 with 63 production hits |
| RFC 7748 KATs pass | 3.6 plus the Python vector check | constants and arithmetic: yes, rc 0; the Rust test: no |
| #538 mergeable | after this lands, rebase #538 | no |

## 4. Path B: unblock #538 without the rand migration

`x25519-dalek = { version = "3.0", features = ["static_secrets", "getrandom"] }`,
then replace the 14 `random_from_rng` sites with `random()` (ephemeral) and
`From<[u8; 32]>` (static, fed by the existing 0.8 RNG). This unblocks #538 with
a small, reviewable diff and no workspace-wide rand change.

It does **not** satisfy #545's acceptance, so it is presented as an alternative
for the owner to choose, not as a substitute quietly swapped in. If Path B is
chosen, #545 should be rewritten or closed as superseded rather than left open
against a contract nothing in the tree intends to meet.

## 5. What is verified here, and what is not

Verified in this session:
- Every manifest, lock and call-site fact above, read from this checkout at
  `ced2189e`.
- The RFC 7748 constants and the arithmetic: `scripts/x25519_rfc7748_kat_check.py`
  exits 0 on section 6.1 DH, section 5.2 single-call, and the 1- and
  1,000-iteration vectors.
- The inventory: `scripts/rand_010_migration_check.py` reports 118 hits, 63
  production, 16 production files; report mode exits 0, `--enforce` exits 1.
- The rand 0.10.2 and x25519-dalek 3.0.0 API surfaces, read from docs.rs.

Not verified, and not claimed:
- That any of this compiles. No Rust toolchain exists in this sandbox.
- Whether `ThreadRng` satisfies x25519-dalek's `CryptoRng` bound (see 2.2).
- Behaviour under the test suite, on any device, or on CI.
- The rule-8 review this change requires.

## 6. Reviewability limits

- Uncompiled. `cargo test --workspace --no-run` and the full suite are the first
  things a reviewer must run, and neither can run here.
- Rule-8 gated: `core/src/crypto`, `core/src/privacy`, `core/src/identity`.
- Sequenced: dependabot is parked through v0.5.1 by the owner comment on #545, and
  PR #633 must be dispositioned rather than duplicated.
- One owner decision is open by construction: the `KeyPair::generate()` shape in
  3.4.
