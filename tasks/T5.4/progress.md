# T5.4 — CI workflow: core build + test matrix

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.3
**Blocks:** T5.5, T5.7, T1.5, T1.6, T2.1, T3.1, T4.1

## Technical Context
- Create `.github/workflows/ci.yml`
- Tests: 21 integration files in `core/tests/`, 1,145+ unit tests, proptests
- `gen-bindings` feature gates `gen_kotlin.rs`/`gen_swift.rs`

## Implementation
1. Job 1: `fmt` + `clippy` + `deny` (after T5.3)
2. Job 2: `cargo test --workspace` on ubuntu + macos
3. Job 3: `cargo test -p scmessenger-core --features phase2_apis`
4. Job 4: doc build
5. Cache with `Swatinem/rust-cache`
6. Pin toolchain via `rust-toolchain.toml` (new file, stable channel)

## Edge Cases
- mDNS/network-touching integration tests may fail in CI containers — core already "gracefully disables mDNS in containers" per `behaviour.rs`
- Mark genuinely network-dependent tests `#[ignore]` with a dedicated `-- --ignored` nightly job rather than letting them flake
- proptest/Kani: run Kani in a separate optional job (needs `cargo kani` install, slow)

## Verification
- [x] Workflow green on a test PR
- [x] Total wall time < 20 min
- [x] Failure of any single test fails the pipeline
