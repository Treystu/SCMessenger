# iOS local-work merge record

## Source and destination

- Local source: `../SCMessenger-iOS` (a non-Git, iOS-focused working copy).
- Destination: this full SCMessenger checkout.
- Upstream base: `origin/main` at `341dfdc1` (`fix(docker): remove --all-features from rust-core-test command in docker-compose.test.yml`).
- Transfer date: 2026-07-22.

## Merged editable changes

- Core contact-store sharing and backup/restore durability updates, with expanded fast-backup coverage.
- Swift binding generator concurrency annotations and UniFFI configuration alignment.
- iOS identity hydration and cached public-identity state, contact backup checkpoints, onboarding/settings/tab gates, transport behavior, and safer device-install defaults.

## Generated files

- `iOS/SCMessenger/SCMessenger/Generated/api.swift` and `apiFFI.h` were copied from the local iOS working copy. They correspond to the merged Rust source changes.
- A fresh `./iOS/copy-bindings.sh` was attempted in this environment after dependencies were downloaded. The native Cargo compile detached from the command runner and left an orphaned build lock, so its generated output could not be used as verification.

## Intentionally not copied

- `iOS/SCMessengerCore.xcframework`: prebuilt artifact/header changes were left untouched.
- `SCMessenger.xcodeproj/project.pbxproj`: local changes reorder existing file references and replace the configured Apple development-team identifier. The checkout's project configuration was retained to avoid changing signing ownership.
- Local `.DS_Store`, build folders, `target/`, and `contacts.db`: machine-specific or generated data.

## Required authoritative follow-up

1. Regenerate bindings with `./iOS/copy-bindings.sh` in a normal developer shell.
2. Run the repository's Windows-host verification gates and an iOS simulator/device build.
3. Review the regenerated binding diff and the existing unrelated working-tree modifications before staging.
4. Commit only after those checks complete. This environment is not authorized to create the commit.
