# Chat Pinch-to-Zoom Text Sizing + Small-Feature Runbook

## Summary

- Ship a **chat-only, persistent text-size control** for Android and iOS.
- Use **pinch-to-zoom inside open conversation threads on both platforms**.
- Save one shared chat scale for the app, not per conversation.
- Use these fixed rules:
  - default `1.00x`
  - min `0.85x`
  - max `1.60x`
  - round persisted values to `0.05x`
  - snap back to `1.00x` when the final value lands within `±0.05`
- Keep iOS **Dynamic Type-friendly** by applying the app multiplier on top of system text sizing instead of hard-coding chat text to fixed sizes only.

## Key Changes

- Android:
  - Implement the live thread change in [ChatScreen.kt](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt), because that file contains the active in-thread `MessageBubble`; do not assume the separate `ui/chat/MessageBubble.kt` is the one currently driving chat.
  - Add a new `conversation_text_scale` app preference in `PreferencesRepository`.
  - Expose that preference through a **small chat-appearance preference surface** dedicated to this feature, rather than reusing the heavier settings-loading path.
  - Attach pinch detection to the chat transcript container, keep one-finger scrolling untouched, preview scale live during the gesture, and persist only the normalized final value.
  - Apply the scale to:
    - message body text
    - timestamps
    - delivery-state labels
    - composer text and placeholder
  - Do not scale:
    - top app bar
    - bottom navigation
    - conversation list
    - settings typography
  - Add a simple **“Reset chat text size”** control plus current value display under App Preferences.

- iOS:
  - Implement the live thread change in [MainTabView.swift](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift), because `ChatView` and `MessageBubble` currently live there.
  - Persist the same `conversation_text_scale` key in `UserDefaults`, ideally through `@AppStorage` at the chat/settings layer.
  - Add `MagnificationGesture` to the transcript area with the same normalize/clamp/snap rules as Android.
  - Convert chat-only text rendering to a Dynamic-Type-based baseline plus the app multiplier.
  - Add the same reset control and current value display under Settings > App Preferences.

- Documentation and runbook:
  - Add a new active doc: `docs/SMALL_FEATURE_RUNBOOK.md`.
  - Make it a **general small-feature runbook**, not UI-only:
    - intent/scope lock
    - affected surfaces and state inventory
    - parity expectations
    - accessibility and fallback/reset rules
    - docs-sync checklist
    - build/test checklist
    - manual QA checklist
    - rollback note
  - Link it from `DOCUMENTATION.md` and classify it in `docs/DOCUMENT_STATUS_INDEX.md`.
  - Update the canonical change-tracking docs in the same run:
    - `docs/CURRENT_STATE.md`
    - `REMAINING_WORK_TRACKING.md`
    - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
    - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` if the feature introduces any UX/accessibility or regression risk worth tracking

## Interfaces / Stored State

- Add one new Android app-preference key: `conversation_text_scale`
- Add one matching iOS defaults key: `conversation_text_scale`
- No Rust core, UniFFI, protocol, or wire-format changes

## Test Plan

- Android unit coverage:
  - scale normalization/clamping/snap behavior
  - persistence write/read behavior for the new preference surface
- Manual QA on both platforms:
  1. Pinch out enlarges text in an open chat
  2. Pinch in reduces text in an open chat
  3. Leaving and reopening the app preserves the chosen size
  4. Reset returns to `1.00x`
  5. One-finger scroll still works normally
  6. Send flow still works normally
  7. Large and small scales do not clip bubbles or composer text
  8. iOS still responds correctly to system Text Size / Dynamic Type changes
- Required verification commands for the implementation run:
  - `cd android && ANDROID_HOME="$HOME/Library/Android/sdk" ./gradlew app:assembleDebug app:testDebugUnitTest`
  - `bash ./iOS/verify-test.sh`
  - `./scripts/docs_sync_check.sh`

## Assumptions

- The saved scale is **global to chat threads**, not per-contact or per-thread.
- v1 includes a **reset control**, but not a full slider/editor UI.
- This is intentionally a **hybrid behavior**:
  - more gesture-driven than stock iOS
  - more persistent than stock Android pinch behavior
