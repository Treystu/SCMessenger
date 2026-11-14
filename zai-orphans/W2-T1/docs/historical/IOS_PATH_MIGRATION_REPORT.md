# iOS Path Migration Report

Date: 2026-02-23
Branch: `codex/ios-path-canonicalization`

## Scope

Canonicalize repository iOS path references to `iOS/` (uppercase-I), preserve all existing iOS content, and prevent future case-drift regressions.

## Pre-Migration Snapshot

Artifacts captured before edits:

- `docs/_ios_migration_pre_status.txt`
- `docs/_ios_migration_pre_refs.txt`
- `docs/_ios_before_inventory.txt`

Inventory result:

- `iOS/` files: 59
- `ios/` files: 59
- Relative path overlap: 59 common, 0 upper-only, 0 lower-only

Interpretation: this workspace is case-insensitive for these paths (alias/collision behavior), so migration required reference normalization rather than file relocation.

## Migration Actions Executed

1. Path governance formalized
- Added canonical rule in `DOCUMENTATION.md`:
  - `iOS/` is the only valid iOS root path in docs/scripts.
- Added matching rule in `docs/REPO_CONTEXT.md`.

2. Reference normalization
- Rewrote path references from `ios/` to `iOS/` across canonical docs and historical iOS docs where references represented repository paths.
- Updated backlog item to mark case-split resolution complete in `REMAINING_WORK_TRACKING.md`.

3. iOS script hardening and canonical path enforcement
- `iOS/copy-bindings.sh`
  - normalized to one generated output path:
  - `iOS/SCMessenger/SCMessenger/Generated/`
- `iOS/verify-test.sh`
  - hardened to use workspace if present, otherwise project fallback:
  - `SCMessenger/SCMessenger.xcworkspace` -> fallback `SCMessenger/SCMessenger.xcodeproj`

4. iOS doc consistency updates
- `iOS/README.md` updated so privacy toggle status matches canonical parity state.

## Section-Level Action Conversion (iOS Historical-Heavy Docs)

| File | Action | Notes |
| --- | --- | --- |
| `iOS/COMPLETE_STATUS.md` | `keep + historical` | kept as historical snapshot; path references normalized to `iOS/` |
| `iOS/FINAL_STATUS.md` | `keep + historical` | kept as historical snapshot; path references normalized to `iOS/` |
| `iOS/IMPLEMENTATION_STATUS.md` | `keep + historical` | kept as historical snapshot; path references normalized to `iOS/` |
| `iOS/IMPLEMENTATION_SUMMARY.md` | `keep + historical` | kept as historical snapshot; path references normalized to `iOS/` |
| `iOS/PHASE4_IMPLEMENTATION.md` | `keep + historical` | retained as implementation-phase reference |
| `iOS/PHASES_4-15_GUIDE.md` | `keep + historical` | retained as phased blueprint context |
| `iOS/PLAN_REVIEW.md` | `keep + historical` | retained as review context |
| `iOS/README.md` | `rewrite + keep` | aligned with canonical cross-platform state |
| `iOS/XCODE_SETUP.md` | `rewrite + keep` | canonical path references standardized to `iOS/` |
| `iOS/iosdesign.md` | `keep + historical` | retained; large design artifact, not canonical truth source |

Canonical truth remains in:

- `docs/CURRENT_STATE.md`
- `REMAINING_WORK_TRACKING.md`
- `docs/REPO_CONTEXT.md`
- `DOCUMENTATION.md`

## Validation Gates

1. Path consistency scan
- After filtering intentional governance mentions and non-path tokens, no active lowercase `ios/` path references remain in canonical docs/scripts.

2. Shell script syntax
- `bash -n iOS/copy-bindings.sh iOS/verify-build-setup.sh iOS/verify-test.sh android/verify-build-setup.sh` -> pass

3. iOS verification
- `bash ./iOS/verify-build-setup.sh` -> pass
- `bash ./iOS/verify-test.sh` -> pass (project fallback path used; build succeeded)

4. Android verification
- `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk bash ./android/verify-build-setup.sh` -> pass

## Residual Notes

- `iOS/verify-test.sh` currently reports warnings (build succeeds). Warning reduction is tracked separately under reliability/tooling hardening.
- Because filesystem path aliases exist in this environment, enforcing `iOS/` reference governance in docs/scripts is the main regression control.

## Regression-Prevention Recommendation

Implemented in `.github/workflows/ci.yml`:

- `check-path-governance` now fails CI on new lowercase `ios/` path references in active docs/code roots, with exclusions for approved textual exceptions (`apple-ios` target triples, `iOS/Android` prose, and explicit governance text).
