# SCMessenger iOS

iOS client app for SCMessenger using SwiftUI with UniFFI bindings into Rust core.

## Prerequisites

- macOS + Xcode command line tools
- Rust toolchain
- iOS Rust targets:
  - `aarch64-apple-ios`
  - `aarch64-apple-ios-sim`

## Verify Environment

```bash
./iOS/verify-build-setup.sh
```

Latest local verification summary (2026-02-23):

- Rust toolchain: pass
- iOS Rust targets: pass
- UniFFI Swift generation: pass
- Static library compilation (`scmessenger-mobile`): pass

## Build Workflow

```bash
# Generate and copy UniFFI outputs into the iOS app tree
./iOS/copy-bindings.sh

# Optional command-line build verification
xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build
```

Then open:

- `iOS/SCMessenger/SCMessenger.xcodeproj`

The app project and source tree are already present under `iOS/SCMessenger/SCMessenger/`.

## Architecture Summary

- SwiftUI app and view models live in `iOS/SCMessenger/SCMessenger/`.
- Rust-facing integration is handled through UniFFI-generated APIs in:
  - `iOS/SCMessenger/SCMessenger/Generated/`
- `MeshRepository.swift` is the central boundary for service lifecycle, contacts, history, and transport bridge calls.

## Known Open Gaps

- Non-core privacy toggle UI is intentionally disabled pending Rust core toggle APIs:
  - `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
- Background operation helpers in repository are currently placeholders (sync/discovery maintenance methods):
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`

See `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md` for cross-platform backlog context.
