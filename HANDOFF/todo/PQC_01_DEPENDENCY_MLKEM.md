# TASK: PQC-01 — Add ML-KEM-768 dependency and smoke-test module

Read `PQC_00_MASTER_PLAN.md` first. Depends on: nothing. Wave 0. Min tier: Haiku.

## Scope

Add `libcrux-ml-kem` to the workspace and prove it compiles and round-trips on every target this repo builds. No integration with any existing crypto path — that is PQC-05/06.

## Steps

1. Add to root `Cargo.toml` under `[workspace.dependencies]`: `libcrux-ml-kem = { version = "<latest stable on crates.io>", default-features = false, features = ["mlkem768"] }`. Check the crate's docs.rs for the exact feature name for ML-KEM-768-only builds; if no such feature exists, use default features. Record the chosen version and why in this file.
2. Add `libcrux-ml-kem = { workspace = true }` to `core/Cargo.toml` `[dependencies]`.
3. Create `core/src/crypto/pq/mod.rs` (and register `pub mod pq;` in `core/src/crypto/mod.rs`) containing thin wrappers ONLY:
   - `pub struct MlKem768KeyPair { ... }` wrapping keygen (from OS randomness via `rand::rngs::OsRng`, 64 bytes of seed material as the crate requires),
   - `pub fn generate() -> MlKem768KeyPair`
   - `pub fn encapsulate(encaps_key: &[u8]) -> Result<(Vec<u8> /*ct*/, [u8; 32] /*ss*/)>`
   - `pub fn decapsulate(keypair: &MlKem768KeyPair, ct: &[u8]) -> Result<[u8; 32]>`
   - Validate input lengths: encapsulation key MUST be 1184 bytes, ciphertext MUST be 1088 bytes; return errors (anyhow), never panic.
   - Secret halves must be zeroized on drop (wrap in a struct implementing `Zeroize`/`Drop`; follow `RatchetKey` in `core/src/crypto/ratchet.rs`).
4. Unit tests in the same file:
   - roundtrip: generate, encapsulate to the public half, decapsulate, assert shared secrets equal and 32 bytes.
   - wrong-length inputs rejected.
   - tampered ciphertext (flip one byte) decapsulates WITHOUT error but yields a DIFFERENT shared secret (ML-KEM is implicit-rejection; assert inequality, not error).
5. Security rule (`.claude/rules/security.md`): run `git diff Cargo.lock` and list every added/removed crate in this task file under a "Cargo.lock audit" heading. If anything unexpected appears (crates unrelated to libcrux), stop and escalate.

## Definition of Done

- [ ] Standard gates (PQC-00) all PASS — the wasm32 check gate is the critical one here; if `libcrux-ml-kem` fails on wasm32-unknown-unknown, stop and escalate with the error (fallback candidate to propose: RustCrypto `ml-kem`).
- [ ] If an Android NDK toolchain is available on the host: `cargo ndk -t arm64-v8a check -p scmessenger-core` PASSES; if not available, state so explicitly here.
- [ ] New unit tests pass: `cargo test -p scmessenger-core pq`
- [ ] Cargo.lock audit section written.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Touch `encrypt.rs`, `ratchet.rs`, `session_manager.rs`, or any wire format.
- Add any other new dependency.
