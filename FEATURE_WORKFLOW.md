> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# SCMessenger Feature Workflow Runbook

This guide outlines the streamlined process for adding new features to SCMessenger across all platforms (Core, iOS, Android, Web/WASM, CLI). Follow this checklist to ensure consistency, quality, and complete coverage.

## [Current] Section Action Outcome (2026-02-23)

- `rewrite`: workflow applies to unified Android+iOS+Web parity delivery.
- `move`: launch-gating execution and priorities are in `docs/GLOBAL_ROLLOUT_PLAN.md` and `REMAINING_WORK_TRACKING.md`.
- `keep`: this file is the implementation process runbook (how work ships), not current-state proof.

## [Needs Revalidation] 1. Core Implementation (Rust)

**Location**: `core/src/`

- [ ] **Define Data Structures**: Update structs/enums in `api.udl` (for UniFFI) and `lib.rs`.
- [ ] **Implement Logic**: Add business logic in the appropriate Rust modules.
- [ ] **Expose API**: Ensure public methods are exposed via `IronCore` or other UniFFI-compatible structs.
- [ ] **Unit Tests**: Add Rust unit tests in `core/src/...`.
- [ ] **Verify Bindings**: Run `cargo check` and ensure `api.udl` is valid.

## [Needs Revalidation] 2. Generate Bindings

- [ ] **Android**: Run `./gradlew :app:generateUniFFIBindings` (in `android/`).
- [ ] **iOS**: Run `iOS/copy-bindings.sh`.
- [ ] **WASM**: Run `wasm-pack build` (in `core/`).

## [Needs Revalidation] 3. Platform Integration

For each platform, ensure the feature is integrated into the UI and logic.

**Identity Export Requirements:**

- **Completeness**: When implementing identity export, ensure ALL information is provided:
  - Identity ID
  - Nickname
  - Public Key
  - Direct Connection Info (IP/Port/Multiaddr) - _Crucial for direct connections_
  - Last-known/Current Relay
- **Deduplication**: Use shared helper functions where possible to avoid code duplication between `init`, `identity show`, and other commands.
- **Security**: Always warn users about the sensitivity of private keys if exporting full credentials.

**Checklist:**

- [ ] **Data Layer**: Update repositories to expose new core functionality.
- [ ] **UI/UX**: Create or update UI components (Views/Screens).
  - [ ] Ensure **scrolling works properly** on mobile screens (iOS/Android).
  - [ ] Verify buttons and interactive elements are accessible.
- [ ] **Manual Test**: Build and run on Emulator/Device.

### [Needs Revalidation] iOS (Swift)

**Location**: `iOS/SCMessenger/`

- [ ] **Update Repository**: Update `MeshRepository.swift` to use the new Core API methods.
- [ ] **Update UI**: Add SwiftUI Views/ViewModels to expose the feature.
- [ ] **Manual Test**: Build and run in Simulator (Cmd+R).

### [Needs Revalidation] Android (Kotlin)

**Location**: `android/app/src/main/java/`

- [ ] **Update Repository**: Update `MeshRepository.kt` to use the new Core API actions.
- [ ] **Update UI**: Add Jetpack Compose screens/components.
- [ ] **Manual Test**: Build and run on Emulator/Device.

### [Needs Revalidation] Web / WASM (TypeScript/Rust)

**Location**: `ui/` or `wasm/`

- [ ] **Update WASM Bridge**: Ensure `wasm-pack` output is consumed correctly.
- [ ] **Update UI**: Add HTML/JS/TS components.
- [ ] **Test**: Run local dev server.

### [Needs Revalidation] CLI (Rust)

**Location**: `cli/`

- [ ] **Update Commands**: Add new subcommands or flags in `cli/src/main.rs`.
- [ ] **Test**: Run `cargo run -- <command>`.

## [Needs Revalidation] 4. Feature Harmonization & Rollout

Detailed audit and implementation status are tracked in [`FEATURE_PARITY.md`](FEATURE_PARITY.md).

### [Needs Revalidation] Streamlined Rollout Process

To avoid regressions and ensure consistency:

1.  **Core First**: Implement logic in Rust (`core/src`). Verify with `cargo test`.
2.  **API Definition**: Update `core/src/api.udl`.
3.  **CLI Verification**: Update CLI to use new Core features. This verifies the Rust logic in a real app context.
4.  **Bindings Generation**: Run `uniffi-bindgen` (wrapped in platform build scripts) to generate Swift/Kotlin bindings.
5.  **Platform Data Layer**:
    - **iOS**: Update `MeshRepository.swift`.
    - **Android**: Update `MeshRepository.kt`.
6.  **Platform ViewModels**: Update ViewModels (e.g., `SettingsViewModel`) to prepare data.
7.  **UI Implementation**:
    - **iOS**: SwiftUI Views (e.g., `SettingsView`).
    - **Android**: Jetpack Compose Screens (e.g., `SettingsScreen`).
8.  **Verification**: Ensure correct behavior on all platforms. Specifically check:
    - **Identity**: Does the exported ID/Key match across platforms?
    - **Storage**: Is data persisted correctly?
    - **UI Resiliency**: Does the UI handle empty/loading states gracefully?

## [Needs Revalidation] 4. Verification & Testing

- [ ] **Docker Simulation**: Update `verify_simulation.sh` if the feature involves networking/messaging.
- [ ] **Integration Tests**: Run `./verify_integration.sh` (if applicable).
- [ ] **End-to-End**: Verify feature works between platforms (e.g., iOS <-> Android).

## [Needs Revalidation] 5. Documentation

- [ ] **User Guide**: Update `README.md` or specific guides in `docs/` with new feature usage.
- [ ] **Developer Docs**: Update struct/API documentation if significant changes were made.
- [ ] **Changelog**: Add entry to user-facing changelogs.

## [Needs Revalidation] 6. Commit & Context Cleanup

- [ ] **Commit**: `git commit -m "feat(scope): Description"`
- [ ] **AI Context**: When starting a new major task with an AI agent, summarize the finished state and start a fresh session to keep context clean.

## [Needs Revalidation] 7. Final Validation Protocol

Before marking a task as complete:

1.  **Review Requirements**: Re-read the initial user request and any subsequent clarifications.
2.  **Platform Parity Check**: Ensure the feature behaves consistently on both iOS and Android (unless platform-specific differences are intended).
3.  **Error Handling Review**: Verify that error states (network issues, invalid input, etc.) are handled gracefully and communicated to the user.
4.  **Self-Correction**: If any issues are found during this final check, fix them immediately before notifying the user.
