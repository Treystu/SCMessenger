# BATCH_ANDROID_WS14_3_NOTIFICATION_PARITY

**Status:** VERIFIED REMAINING WORK
**Agent:** Android/Kotlin implementer
**Budget:** 1800s (MIXED tier)
**Phase:** WS14.3
**Source:** REMAINING_WORK_TRACKING.md WS14 gap, docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md

---

## Verified Gap

Android `NotificationHelper.kt` is fully implemented with DM vs DM Request classification, channels, actions, foreground suppression, and tap routing. **BUT** the action handlers and Requests Inbox UI are missing, creating a notification parity gap vs iOS.

**Verified Code State:**
- `NotificationHelper.kt` — Complete (lines 1-649): 5 channels, DM/DM Request classification, grouped messaging, identicon avatars, settings parity
- `MeshForegroundService.kt:596` — `NotificationHelper.showMessageNotification()` IS called from message receive path with WS14 classification
- `MeshRepository.kt` — `logDeliveryAttempt` uses proper `messageId` (fixed since March 2026)

**Verified Missing:**
1. **No BroadcastReceivers** for notification actions: `ACTION_REPLY`, `ACTION_MARK_READ`, `ACTION_MUTE`, `ACTION_OPEN_REQUESTS`
2. **No manifest registration** for action receivers in `AndroidManifest.xml`
3. **No Requests Inbox UI screen** — `android/app/src/main/java/com/scmessenger/android/ui/screens/` has no `RequestsInboxScreen.kt`
4. **No MainActivity intent handling** for `ACTION_OPEN_REQUESTS` — only handles `Intent.ACTION_VIEW` deep links (MainActivity.kt:122,256)

## Scope

### Part A: Notification Action BroadcastReceivers

1. Create `android/app/src/main/java/com/scmessenger/android/notification/NotificationActionReceiver.kt`
   - Handle `ACTION_REPLY`: extract RemoteInput text, forward to `MeshRepository.sendMessage()`
   - Handle `ACTION_MARK_READ`: forward to `MeshRepository.markConversationRead()`
   - Handle `ACTION_MUTE`: forward to `MeshRepository.mutePeer()`
   - Handle `ACTION_OPEN_REQUESTS`: emit broadcast/event for MainActivity navigation
2. Register receiver in `AndroidManifest.xml` with intent filters for all 4 actions
3. Ensure `PendingIntent` request codes in `NotificationHelper.kt` are unique and stable

### Part B: Requests Inbox UI

1. Create `android/app/src/main/java/com/scmessenger/android/ui/screens/RequestsInboxScreen.kt`
   - LazyColumn of pending DM Request items
   - Each item: sender identicon, display name/nickname, preview text, timestamp
   - Actions: Accept (adds contact + opens chat), Reject (blocks + deletes), Block & Delete
   - Empty state: "No pending message requests"
2. Create `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/RequestsInboxViewModel.kt`
   - Exposes `requests: StateFlow<List<RequestItem>>`
   - Loads from `MeshRepository` pending-request state
   - Methods: `acceptRequest(peerId)`, `rejectRequest(peerId)`, `blockAndDelete(peerId)`
3. Add `RequestsInbox` route to NavHost in `MainActivity.kt` or wherever NavHost is defined

### Part C: Tap Routing

1. In `MainActivity.kt` `onCreate` and `onNewIntent`:
   - Add branch for `ACTION_OPEN_REQUESTS`
   - Navigate to RequestsInbox route with optional `EXTRA_PEER_ID` highlight
2. Ensure `NotificationHelper.createOpenRequestsIntent()` flags match `MainActivity` handling

## Constraints

- Use Material3 design language (matches existing Compose screens)
- Respect `notifyDmRequestEnabled` / `notifyDmRequestInForeground` settings
- Follow existing architecture: Repository -> ViewModel -> Compose UI
- Do NOT modify `NotificationHelper.kt` action constants or channel IDs (they are correct)
- All new strings must go in `strings.xml` (per android.md rules)

## File Targets

- `android/app/src/main/java/com/scmessenger/android/notification/NotificationActionReceiver.kt` [NEW]
- `android/app/src/main/java/com/scmessenger/android/ui/screens/RequestsInboxScreen.kt` [NEW]
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/RequestsInboxViewModel.kt` [NEW]
- `android/app/src/main/AndroidManifest.xml` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` [EDIT]
- `android/app/src/main/res/values/strings.xml` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT — add request-list/query methods if missing]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
./gradlew :app:lintDebug -q
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. `./gradlew :app:lintDebug` passes (no new errors)
3. `AndroidManifest.xml` contains registered `NotificationActionReceiver`
4. `RequestsInboxScreen.kt` renders in Android Studio preview
5. `MainActivity.kt` handles `ACTION_OPEN_REQUESTS` intent
6. No new hardcoded strings in UI code (all in `strings.xml`)
7. `REMAINING_WORK_TRACKING.md` updated to mark WS14.3 Android notification parity complete

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
