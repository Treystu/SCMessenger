> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

 # SCMessenger Implementation Progress & Roadmap

## [Current] Section Action Outcome (2026-02-23)

- `move`: active remediation backlog belongs in `REMAINING_WORK_TRACKING.md`.
- `move`: rollout-phase execution belongs in `docs/GLOBAL_ROLLOUT_PLAN.md`.
- `delete/replace`: roadmap checkpoints here are historical unless reflected in canonical backlog and rollout docs.
- `keep`: retain this file as historical remediation context.

## [Historical] Checklist Triage (2026-03-03 HST)

- Open checklist markers in this file were converted to historical status tags.
- Active implementation truth is tracked in `REMAINING_WORK_TRACKING.md`.
- Planned future work ownership remains in WS13/WS14 plans under `docs/V0.2.1_*`.

## [Needs Revalidation] 📊 Current Status Summary

A cross-platform audit reveals that while the core mesh connectivity (via UniFFI) is functional, there is significant non-parity between Android and iOS, particularly in UX (Onboarding) and advanced Privacy features.

---

## [Needs Revalidation] 🛠 Phase 1: High-Priority UI/UX Parity

**Objective:** Align the first-run experience and core messaging interface.

- [Historical - Re-scoped] **Android Onboarding**: Implement `OnboardingScreen.kt` using the iOS 5-step flow (Welcome, Identity, Permissions, Relay Info, Completion).
- [Historical - Re-scoped] **iOS Notification Enhancements**: Update `NotificationManager.swift` to support Identicons and grouped messaging (matching Android's `NotificationHelper`).
- [Historical - Re-scoped] **Android Chat Polish**: Fix the "TODO: Create icon" in `MeshForegroundService` and ensure the `ChatScreen` scroll-to-bottom logic is robust.
- [Historical - Re-scoped] **iOS Chat Error Handling**: Improve `ChatViewModel` error mapping to be as descriptive as the Android equivalent.

## [Needs Revalidation] 🔐 Phase 2: Privacy & Core Feature Wiring

**Objective:** Connect the UI toggles to the Rust backend.

- [Historical - Re-scoped] **UniFFI Expansion**: Update `api.udl` to expose `CoverTraffic`, `MessagePadding`, and `TimingObfuscation` settings.
- [Historical - Re-scoped] **Settings Wiring (iOS)**: Implement the `TODO` items in `SettingsViewModel.swift` to forward privacy toggles to the core.
- [Historical - Re-scoped] **Settings Wiring (Android)**: Ensure `SettingsViewModel.kt` is synchronized with the new UniFFI privacy settings.
- [Historical - Re-scoped] **Identity Management**: Unify the "Identity Export" formats between platforms to ensure seamless contact sharing.

## [Needs Revalidation] 📡 Phase 3: Transport & Reliability

**Objective:** Finalize local discovery and background stability.

- [Historical - Re-scoped] **iOS Multipeer Integration**: Fully wire `MultipeerTransport.swift` into the Rust swarm.
- [Historical - Re-scoped] **Android Background Stability**: Optimize `MeshForegroundService` for better battery life during long-running mesh sessions.
- [Historical - Re-scoped] **Dashboard Sync**: Update `MeshDashboardView.swift` (iOS) to allow service control (Start/Stop) matching the Android `DashboardScreen`.

## [Needs Revalidation] 📦 Phase 4: Final Polish & Assets

- [Historical - Re-scoped] **Vector Assets**: Generate/Add proper notification icons (`ic_notification.xml`) for Android.
- [Historical - Re-scoped] **Theme Alignment**: Ensure HSL color tokens are identical across `Theme.swift` and `Color.kt`.
- [Historical - Re-scoped] **Documentation**: Finalize the user-facing `README.md` for both platforms.

---

## [Needs Revalidation] 🚀 Execution Plan

I will now begin iterating through Phase 1, starting with the **Android Onboarding** implementation to match the iOS standard.
