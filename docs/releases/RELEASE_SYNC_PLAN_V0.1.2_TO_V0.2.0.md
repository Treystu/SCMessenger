# SCMessenger Release Synchronization Plan (v0.1.2 -> v0.2.0)

Status: Active (in-repo prep complete; tag/publish actions pending maintainer execution)  
Last updated: 2026-03-03

This plan outlines the steps to synchronize the SCMessenger versions between the codebase and GitHub releases, and prepare for the upcoming v0.2.0 release.

## Execution Snapshot (2026-03-03)

Completed in repository:

1. Workspace/package version metadata bumped to `0.2.0`.
2. Android/iOS displayed app version metadata aligned to `0.2.0`.
3. Repo-local release notes artifacts created under `docs/releases/`.

Pending maintainer-run release ops:

1. Create/push git tags (`v0.1.2`, `v0.2.0`) per release policy.
2. Draft/publish GitHub releases using `docs/releases/RELEASE_NOTES_V0.1.2_GH.md` and `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`.

## User Review Required

> [!IMPORTANT]
> This plan involves bumping the project version to `0.2.0` across the workspace. Please confirm if this matches the intended release cadence.

## Proposed Changes

### Versioning and Releases

#### [MODIFY] `Cargo.toml`
#### [MODIFY] `core/Cargo.toml`
#### [MODIFY] `cli/Cargo.toml`
#### [MODIFY] `wasm/Cargo.toml`
#### [MODIFY] `mobile/Cargo.toml`

- Bump version from `0.1.2` to `0.2.0` in the root workspace and all sub-packages.

---

### GitHub Release Workflow

#### v0.1.2 (Immediate)
1. **Tag the Repository**: Create a git tag `v0.1.2` at the current HEAD.
2. **Draft Release**: Use `docs/releases/RELEASE_NOTES_V0.1.2_GH.md` to create the release on GitHub.
3. **Publish**: Finalize as a "Prerelease" or "Release" based on stability confidence.

#### v0.2.0 (Future)
1. **Complete Release Sweep**: Execute the Post-WS12 Residual Risk Closure Sweep.
2. **Bump Versions**: Update all `Cargo.toml` files to `0.2.0`.
3. **Commit and Tag**: Commit the version bump and tag as `v0.2.0`.
4. **Draft Release**: Use `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`.
5. **CI Assets**: Ensure GitHub Actions successfully builds and attaches binaries/artifacts for all platforms.

---

### Documentation and Release Notes

#### [NEW] `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`
- Consolidated release notes for v0.1.2 based on `docs/RELEASE_NOTES_V0.1.2_ALPHA.md`, ready for GitHub pasting.

#### [NEW] `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`
- Draft release notes for v0.2.0 based on the major WS0-WS12 implementation work.

---

### Verification and Gates

#### [MODIFY] `REMAINING_WORK_TRACKING.md`
- Update status to reflect that v0.2.0 work is transitioning from execution closure to release verification.

## Verification Plan

### Automated Tests
- Run full workspace tests:
  ```bash
  cargo test --workspace
  ```
- Verify platform builds:
  - Android: `cd android && ./gradlew assembleDebug`
  - iOS: `bash ./iOS/verify-test.sh`
  - WASM: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`

### Manual Verification
- Review the drafted release notes for accuracy against `REMAINING_WORK_TRACKING.md` outcomes.
- Execute the **Post-WS12 Residual Risk Closure Sweep** (item 14 in `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`) to ensure no open critical risks remain.
