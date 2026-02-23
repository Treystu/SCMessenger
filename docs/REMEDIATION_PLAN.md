> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

 # SCMessenger Implementation Progress & Roadmap

## [Needs Revalidation] 📊 Current Status Summary

A cross-platform audit reveals that while the core mesh connectivity (via UniFFI) is functional, there is significant non-parity between Android and iOS, particularly in UX (Onboarding) and advanced Privacy features.

---

## [Needs Revalidation] 🛠 Phase 1: High-Priority UI/UX Parity

**Objective:** Align the first-run experience and core messaging interface.

- [ ] **Android Onboarding**: Implement `OnboardingScreen.kt` using the iOS 5-step flow (Welcome, Identity, Permissions, Relay Info, Completion).
- [ ] **iOS Notification Enhancements**: Update `NotificationManager.swift` to support Identicons and grouped messaging (matching Android's `NotificationHelper`).
- [ ] **Android Chat Polish**: Fix the "TODO: Create icon" in `MeshForegroundService` and ensure the `ChatScreen` scroll-to-bottom logic is robust.
- [ ] **iOS Chat Error Handling**: Improve `ChatViewModel` error mapping to be as descriptive as the Android equivalent.

## [Needs Revalidation] 🔐 Phase 2: Privacy & Core Feature Wiring

**Objective:** Connect the UI toggles to the Rust backend.

- [ ] **UniFFI Expansion**: Update `api.udl` to expose `CoverTraffic`, `MessagePadding`, and `TimingObfuscation` settings.
- [ ] **Settings Wiring (iOS)**: Implement the `TODO` items in `SettingsViewModel.swift` to forward privacy toggles to the core.
- [ ] **Settings Wiring (Android)**: Ensure `SettingsViewModel.kt` is synchronized with the new UniFFI privacy settings.
- [ ] **Identity Management**: Unify the "Identity Export" formats between platforms to ensure seamless contact sharing.

## [Needs Revalidation] 📡 Phase 3: Transport & Reliability

**Objective:** Finalize local discovery and background stability.

- [ ] **iOS Multipeer Integration**: Fully wire `MultipeerTransport.swift` into the Rust swarm.
- [ ] **Android Background Stability**: Optimize `MeshForegroundService` for better battery life during long-running mesh sessions.
- [ ] **Dashboard Sync**: Update `MeshDashboardView.swift` (iOS) to allow service control (Start/Stop) matching the Android `DashboardScreen`.

## [Needs Revalidation] 📦 Phase 4: Final Polish & Assets

- [ ] **Vector Assets**: Generate/Add proper notification icons (`ic_notification.xml`) for Android.
- [ ] **Theme Alignment**: Ensure HSL color tokens are identical across `Theme.swift` and `Color.kt`.
- [ ] **Documentation**: Finalize the user-facing `README.md` for both platforms.

---

## [Needs Revalidation] 🚀 Execution Plan

I will now begin iterating through Phase 1, starting with the **Android Onboarding** implementation to match the iOS standard.
