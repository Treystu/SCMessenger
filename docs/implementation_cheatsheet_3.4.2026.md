# SCMessenger — Implementation Cheat Sheet
## Global Viability: 10 Action Items with Exact Code Plans

**Date:** 2026-03-04  
**Use:** Implementation reference — exact files, line numbers, what to change, how to verify

> Items 1–2 are hard prerequisites. Items 3–5 can run in parallel after builds are stable. Items 6–10 can follow in any order.

---

## ITEM 1 — iOS Crash-Free BLE Send Path

**Issue:** `SIGTRAP` crash in `BLEPeripheralManager.sendDataToCentral` + CPU watchdog kill under retry load  
**Priority:** 🔴 P0 — nothing else matters until iOS doesn't crash

### 1a. Audit BLE send path for remaining crash triggers

**File:** [iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift)

The current `sendDataToCentral` (lines 160–204) is actually already hardened — no force-unwrap, all guards present. The crash is almost certainly happening **above** this function, in the caller that invokes `broadcastDataToCentrals` or `sendDataToConnectedCentral`.

**Check these callers in [MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift):**

```bash
grep -n "sendDataToCentral\|broadcastDataToCentrals\|sendDataToConnectedCentral\|blePeripheral" \
  iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift | head -40
```

Look for any call site that:
- Does not nil-check `blePeripheralManager` before calling
- Is called from a non-main-queue context without dispatch
- Is called from inside a loop that doesn't check `subscribedCentrals` is non-empty first

**Expected fix pattern:**
```swift
// Anywhere you call blePeripheralManager.sendDataToConnectedCentral:
guard !blePeripheralManager.subscribedCentralIds().isEmpty else {
    appendDiagnostic("ble_peripheral_send_skip_no_subscribers")
    return false
}
```

### 1b. Add retry throttle to prevent CPU watchdog kill

**File:** [iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift)

Find the pending outbox flush loop. It currently retries with rapid fire on every `peer_identified` / `peer_discovered` event. Add a minimum inter-retry delay:

```swift
// In the outbox flush / retry dispatch section:
private let retryThrottleMs: Int = 150  // minimum ms between retry passes

// Wrap each flush call:
DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(retryThrottleMs)) { [weak self] in
    self?.flushPendingOutbox(reason: reason)
}
```

Also in [MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift), find `flushPendingOutbox` and add an in-flight guard:

```swift
private var outboxFlushInFlight = false

func flushPendingOutbox(reason: String) {
    guard !outboxFlushInFlight else { return }
    outboxFlushInFlight = true
    defer { outboxFlushInFlight = false }
    // ... existing flush logic
}
```

> Note: Android already has a coroutine mutex for this (`serializedFlushMutex`). iOS needs the equivalent.

### 1c. Deploy and verify

```bash
# Build device binary
./iOS/build-device.sh

# Install to physical iPhone
APPLE_TEAM_ID=<team> DEVICE_UDID=<udid> ./iOS/install-device.sh

# After install: reproduce the crash scenario
# Send 10+ messages to paired Android device in rapid succession
# Then check:
xcrun devicectl device info --device <udid> | grep -i crash
# Or pull crash logs:
idevicecrashreport -u <udid> -e /tmp/crash_pull/
ls /tmp/crash_pull/ | grep SCMessenger
```

**Acceptance:** Zero new `SCMessenger-*.ips` files generated. No `cpu_resource_fatal` file in crash storage.

---

## ITEM 2 — Android ↔ iOS End-to-End Delivery Proof

**Issue:** No synchronized evidence of `stored → delivered` in both directions on real devices  
**Priority:** 🔴 P0 — required for alpha claim

### 2a. Prerequisite: clean device builds

```bash
# Android
cd android && ANDROID_HOME=~/Library/Android/sdk ./gradlew assembleDebug
./android/install-clean.sh

# iOS (after Item 1 is done)
APPLE_TEAM_ID=<team> DEVICE_UDID=<udid> ./iOS/install-device.sh
```

### 2b. Run the live verification harness

```bash
# Attempt 1: 6 minute window, 3 tries, receipt gate required
./scripts/run5-live-feedback.sh \
  --step=CROSS-PAIR-001 \
  --time=6 \
  --attempts=3 \
  --require-receipt-gate
```

### 2c. Manual verification if harness doesn't catch it

While the session is running, watch Android delivery state live:

```bash
adb logcat --pid=$(adb shell pidof -s com.scmessenger.android) \
  -T 1 -v threadtime | grep -E "delivery_state|delivery_attempt|stored|delivered|receipt"
```

Watch iOS delivery state (device syslog):
```bash
idevicesyslog -u <udid> | grep -E "delivery_state|delivery_attempt|stored|delivered|receipt"
```

**Look for this pattern on the Android side (sender):**
```
delivery_attempt msg=<id> state=forwarding
delivery_attempt msg=<id> state=delivered  ← this line = success
```

**Look for this on iOS side (receiver):**
```
on_message_received sender=<android-id> msg=<id>
ble_rx_complete size=NNN
```

### 2d. Extract synchronized artifact for closure

```bash
# After a successful session:
mkdir -p logs/cross-pair-proof/$(date +%Y%m%d_%H%M%S)
adb shell run-as com.scmessenger.android cat files/mesh_diagnostics.log \
  > logs/cross-pair-proof/$(date +%Y%m%d_%H%M%S)/android.log

# Verify with script:
bash ./scripts/verify_receipt_convergence.sh \
  logs/cross-pair-proof/<timestamp>/android.log \
  <ios-diagnostics-fragment>
```

**Acceptance:** `verify_receipt_convergence.sh` finds matching message IDs in both logs. `CROSS-PAIR-001` closed in burndown plan.

---

## ITEM 3 — Message Ordering: Enforce `sender_timestamp` as Sort Key

**Issue:** iOS and Android sort by local `timestamp` but this field isn't guaranteed to be the sender's creation time  
**Priority:** 🔴 P0 — fix is fast, impact is high

### 3a. Understand the data model first

`MessageRecord` in [core/src/api.udl](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl) (line 372) has **one** timestamp field: `timestamp: u64`.

The question is: **what does the store layer write into `timestamp` for received messages?**

```bash
grep -n "timestamp\|sender_timestamp" core/src/store/history.rs | head -30
```

If `timestamp` is set to `sender_timestamp` from `on_message_received`, then both platforms are already using sender time — the bug is elsewhere (e.g. clock skew or local insertion order from the DB layer).

If `timestamp` is set to local receive time, we need a second field. Check:

```bash
grep -n "sender_timestamp\|MessageRecord {" core/src/lib.rs | head -20
```

### 3b. iOS fix: sort by `senderTimestamp` if available, else `timestamp`

**File:** [iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift), line 32

**Current (broken — sorts by local timestamp only):**
```swift
messages = fetched.sorted { $0.timestamp < $1.timestamp }
```

**Fix:**
```swift
// MessageRecord.timestamp should already be sender_timestamp if core stores it correctly.
// Sort ascending by timestamp. If it comes back from core already sorted, this is a no-op.
// If core returns unsorted, this enforces correct chronological order.
messages = fetched.sorted { $0.timestamp < $1.timestamp }
// Note: if MessageRecord gains a senderTimestamp field, switch to that field here.
```

If the store IS writing local receive time into `timestamp`, the real fix is in **core** — add `sender_timestamp` as a separate `MessageRecord` field:

**[core/src/api.udl](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl) — add to `MessageRecord` dictionary:**
```
dictionary MessageRecord {
    string id;
    MessageDirection direction;
    string peer_id;
    string content;
    u64 timestamp;        // local store time (for display)
    u64 sender_timestamp; // canonical sort key from sender
    boolean delivered;
};
```

Then update the store layer to populate it, and update both platform sort calls to use `sender_timestamp`.

### 3c. Android fix

**File:** [android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt), line 85

**Current:**
```kotlin
_messages.value = messageList.sortedBy { it.timestamp }
```

**Fix (after `sender_timestamp` field is available in UniFFI):**
```kotlin
_messages.value = messageList.sortedBy { it.senderTimestamp }
```

If `sender_timestamp` is already stored as `timestamp` in core, no change is needed — just verify.

### 3d. Add regression test

**iOS — in a test or verify script:**
```swift
// Create 3 messages with out-of-insertion-order sender timestamps
// Assert the rendered order matches sender_timestamp ascending
```

**Android — add to `ChatViewModelTest.kt`:**
```kotlin
@Test
fun `messages sorted by senderTimestamp regardless of insertion order`() {
    // Insert messages in reverse timestamp order
    // Assert _messages emits them in ascending senderTimestamp order
}
```

**Acceptance:** Same conversation on Android and iOS shows identical message order after a live exchange where messages arrive at different local times on each device.

---

## ITEM 4 — iOS Message List Scroll Stability (UX-IOS-002)

**Issue:** List jumps to top and flickers during active conversation  
**File:** [iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift)

### 4a. Identify the scroll trigger bug

**Current code (lines 307–312):**
```swift
.onChange(of: viewModel?.messages.count ?? 0) { _ in
    // Auto-scroll when new messages arrive
    withAnimation(.easeOut(duration: 0.2)) {
        proxy.scrollTo("bottom", anchor: .bottom)
    }
}
```

**Problem:** This fires on EVERY reload of `messages`, including when the whole list refreshes after a state update (receipt, peer_identified). If the list briefly has fewer items mid-reload, count changes trigger a scroll.

Also in [ChatViewModel.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift) (lines 78–88): `subscribeToNewMessages` calls [loadMessages()](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt#77-94) on every `messageUpdates` event — this does a full list replacement, causing a count change even if no new messages arrived.

### 4b. Fix: track scroll position, only auto-scroll when at bottom

**Replace [ChatView](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt#21-294) in [MainTabView.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift) (starting around line 272):**

```swift
struct ChatView: View {
    @Environment(MeshRepository.self) private var repository
    let conversation: Conversation
    @State private var viewModel: ChatViewModel?
    @State private var scrollProxy: ScrollViewProxy?
    @State private var isAtBottom = true        // ← track user scroll position
    @State private var lastMessageId: String?   // ← track last message, not count

    var body: some View {
        VStack(spacing: 0) {
            DeliveryStateLegend()
                .padding(.horizontal, Theme.spacingMedium)
                .padding(.top, Theme.spacingSmall)

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(viewModel?.messages ?? [], id: \.id) { message in
                            MessageBubble(
                                message: message,
                                deliveryState: repository.deliveryStatePresentation(for: message)
                            )
                            .id(message.id)
                        }
                        Color.clear
                            .frame(height: 1)
                            .id("bottom")
                    }
                }
                .onAppear {
                    scrollProxy = proxy
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                        proxy.scrollTo("bottom", anchor: .bottom)
                    }
                }
                // ← Only scroll when a NEW message appears (id changed), not on count change
                .onChange(of: viewModel?.messages.last?.id) { newLastId in
                    guard newLastId != nil, newLastId != lastMessageId else { return }
                    lastMessageId = newLastId
                    // Only auto-scroll if user is already at/near the bottom
                    if isAtBottom {
                        withAnimation(.easeOut(duration: 0.2)) {
                            proxy.scrollTo("bottom", anchor: .bottom)
                        }
                    }
                }
            }
            // ... rest unchanged
        }
    }
}
```

### 4c. Debounce `messageUpdates` reload in [ChatViewModel](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt#21-294)

**File:** [iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift)

Replace the current `subscribeToNewMessages` (lines 78–88):

```swift
private var reloadDebounceTask: Task<Void, Never>?

private func subscribeToNewMessages() {
    repository?.messageUpdates
        .filter { [weak self] message in
            message.peerId == self?.conversation.peerId
        }
        .sink { [weak self] _ in
            // Debounce: cancel any pending reload, schedule a new one 80ms out
            self?.reloadDebounceTask?.cancel()
            self?.reloadDebounceTask = Task { @MainActor [weak self] in
                try? await Task.sleep(nanoseconds: 80_000_000)  // 80ms
                guard !Task.isCancelled else { return }
                self?.loadMessages()
            }
        }
        .store(in: &cancellables)
}
```

**Acceptance:** 10+ messages exchanged in live conversation, no scroll-to-top, no visible flicker.

---

## ITEM 5 — Local Transport Isolation Tests

**Issue:** BLE, WiFi Direct, Multipeer, mDNS all coded but none field-proven in isolation  
**Priority:** 🟠 P1

### 5a. BLE-only test (Android ↔ iOS)

```bash
# On Android device: set env variable before launch, or toggle in debug settings
# The flag SC_BLE_ONLY_VALIDATION=1 blocks WiFi/Multipeer/Core paths

# 1. Disable WiFi on both devices (use airplane mode + re-enable BLE only)
# 2. Launch both apps
# 3. Send 5 messages Android → iOS
# 4. Send 5 messages iOS → Android

# Verify on Android:
adb logcat --pid=$(adb shell pidof -s com.scmessenger.android) \
  | grep -E "ble_tx|ble_rx|ble_send|delivery_attempt"

# Acceptance: delivery_attempt lines show BLE path, no relay-circuit attempts
bash ./scripts/verify_ble_only_pairing.sh \
  <android-log-from-session> <ios-log-from-session>
```

### 5b. WiFi Direct test (Android ↔ Android, if two Android devices available)

```bash
# Both devices on same room, no internet, BLE off
# Launch app on both, add contact via QR
# Send message
# Verify in logcat:
adb logcat | grep -iE "wifidirect|WifiDirect|wifi_direct|p2p"
# Look for: "delivery via WiFi Direct" or "WifiDirectTransport: sent"
```

### 5c. mDNS / LAN test

```bash
# Both devices on same WiFi, internet blocked at router level (or hotspot with no uplink)
# Launch both apps
# Verify peer discovery via mDNS:
adb logcat | grep -iE "mdns|mDNS|local_peer|lan_peer"
# Look for: peer discovered via mDNS, then delivery via LAN path
```

### 5d. Document results

Update [docs/INTEROP_MATRIX_V0.2.0_ALPHA.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/INTEROP_MATRIX_V0.2.0_ALPHA.md) — add a "Runtime Field Evidence" section for each path:

```markdown
| Transport | Pair | Date | Result |
| BLE-only | Android↔iOS | YYYY-MM-DD | Pass/Fail |
| WiFi Direct | Android↔Android | ... | ... |
| mDNS LAN | Android↔iOS | ... | ... |
| Multipeer | iOS↔iOS | ... | ... |
```

---

## ITEM 6 — Confirm Stale Route/BLE Cache Fix (WS12.31)

**Issue:** 291 `Network error` events + repeated stale MAC `65:99:F2:D9:77:01` retries  
**Priority:** 🟠 P1

### 6a. Deploy WS12.31+ Android binary

```bash
cd android && ANDROID_HOME=~/Library/Android/sdk ./gradlew assembleDebug
./android/install-clean.sh
```

### 6b. Run 10-minute live session and extract log

```bash
# Start log capture
adb shell run-as com.scmessenger.android \
  cat files/mesh_diagnostics.log > /tmp/android_diag_post_ws1231.log &

# Let it run 10 minutes with a paired iOS device
sleep 600

# Pull the diagnostics
adb shell run-as com.scmessenger.android \
  cat files/mesh_diagnostics.log > logs/stale-route-fix/android_$(date +%Y%m%d_%H%M%S).log
```

### 6c. Audit for stale-route regression

```bash
# Count Network error rate (should be dramatically lower than 291)
grep -c "Network error" logs/stale-route-fix/android_*.log

# Check for the specific stale MAC address
grep "65:99:F2:D9:77:01" logs/stale-route-fix/android_*.log
# Expected: zero results

# Check for stale route peer ID
grep "12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27" logs/stale-route-fix/android_*.log
# Expected: zero results
```

**Acceptance:** Zero hits for the old stale MAC and peer ID. `Network error` count < 20 for a 10-minute session. Mark `AND-ROUTE-001` and `AND-BLE-001` closed.

---

## ITEM 7 — Internet Global Reach (Cross-Carrier Proof)

**Issue:** Global relay path not proven across different carrier networks  
**Priority:** 🟡 P1

### 7a. Test setup

- **Device A:** One carrier's cellular network (WiFi completely off — airplane mode + cellular only)
- **Device B:** Different network (either different carrier cellular, or a geographically separate WiFi)
- Both pointed at GCP relay: `34.135.34.73:9001`

### 7b. Execute test

```bash
# Verify GCP relay is reachable from each device's network before starting:
# On Android:
adb shell "nc -z -w 5 34.135.34.73 9001 && echo REACHABLE || echo BLOCKED"

# Send 5 messages each direction
# Capture logs:
adb shell run-as com.scmessenger.android \
  cat files/mesh_diagnostics.log > logs/global-proof/android_carrier_$(date +%Y%m%d_%H%M%S).log
```

### 7c. Mid-session network switch test

```bash
# While conversation is active, toggle WiFi on on Device A
# (simulates WiFi → cellular roaming)
# Verify: no messages lost, delivery eventually completes
# Look for in logs:
grep -E "network_changed|reconnect|roaming|bootstrap" logs/global-proof/android_*.log
```

**Acceptance:** Messages deliver cross-carrier within 60s. No permanent `stored` state after 5 minutes. Closes `VALIDATION-001`.

---

## ITEM 8 — Dynamic Bootstrap / Multi-Relay Infrastructure

**Issue:** Single GCP relay is a single point of failure; dynamic bootstrap fetch not implemented  
**Priority:** 🟡 P1

### 8a. Verify `BootstrapResolver` is already wired in UDL

[core/src/api.udl](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl) lines 292–315 confirm `BootstrapResolver` is already implemented with the `env → remote → static` chain. Check it's actually called at app startup:

```bash
grep -rn "BootstrapResolver\|bootstrap_resolver\|resolve()" \
  iOS/SCMessenger/SCMessenger/ android/app/src/main/java/ | head -20
```

### 8b. Wire remote bootstrap URL into app settings (if not already done)

If `BootstrapResolver` isn't being called with a `remote_url`, add it:

**iOS ([MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift)):**
```swift
let bootstrapConfig = BootstrapConfig(
    staticNodes: ["<GCP1-multiaddr>", "<GCP2-multiaddr>"],  // multiple static fallbacks
    remoteUrl: "https://bootstrap.scmessenger.net/nodes.json",  // future
    fetchTimeoutSecs: 5,
    envOverrideKey: "SCM_BOOTSTRAP_NODES"
)
let resolver = BootstrapResolver(config: bootstrapConfig)
let nodes = resolver.resolve()
meshService.setBootstrapNodes(addrs: nodes)
```

**Android ([MeshRepository.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt)) — same pattern via `setBootstrapNodes()`.**

### 8c. Deploy a second relay node

```bash
# On a second VPS (EU or Asia region):
# Copy deploy script and config
scp scripts/deploy_gcp_node.sh user@<vps2>:/tmp/
ssh user@<vps2> "bash /tmp/deploy_gcp_node.sh"

# Get its peer ID and multiaddr:
bash scripts/get-node-info.sh <vps2-ip>
```

Add the second relay's multiaddr to the `staticNodes` list above.

### 8d. Test failover

```bash
# Take the primary GCP relay offline:
ssh gcp-relay "sudo systemctl stop scmessenger-relay"

# Wait up to 60s, verify clients reconnect to second relay
watch -n 5 'adb logcat -d | grep -E "bootstrap|relay|reconnect" | tail -5'

# Bring primary back:
ssh gcp-relay "sudo systemctl start scmessenger-relay"
```

**Acceptance:** App stays connected (or reconnects within 60s) with primary relay offline.

---

## ITEM 9 — Store-and-Forward Proof (Offline Recipient)

**Issue:** Relay custody works in unit tests but has no field proof on real devices  
**Priority:** 🟡 P2

### 9a. Test setup

- **Sender:** Android device, online
- **Recipient:** iOS device, go to airplane mode (fully offline)

### 9b. Execute

```bash
# 1. Put iOS in airplane mode
# 2. Send message from Android to iOS contact
# Verify Android shows it as 'stored' (not 'delivered'):
adb shell run-as com.scmessenger.android cat files/pending_outbox.json

# 3. Wait 60 seconds
# 4. Bring iOS back online (disable airplane mode)
# 5. Watch iOS receive the message without any resend from Android:
idevicesyslog | grep -E "on_message_received|ble_rx_complete|relay_delivery"
```

### 9c. Verify no resend from sender

```bash
# Android should NOT resend — GCP relay holds the message and delivers it.
# Check that Android outbox clears (message gone from pending_outbox.json)
# without a new send attempt:
adb shell run-as com.scmessenger.android cat files/pending_outbox.json
# Expected: {} or empty array for the sent message ID
```

**Acceptance:** iOS shows message without user action. Android pending_outbox entry clears. Message delivered exactly once (no duplicate).

---

## ITEM 10 — iOS Background Lifecycle and Power Evidence

**Issue:** iOS background keep-alive / reconnect not field-validated  
**Priority:** 🟡 P2 (beta gate)

### 10a. Background message delivery test

```bash
# 1. Open conversation on iOS with Android
# 2. Background the iOS app (home button)
# 3. Send 3 messages from Android
# 4. Wait 30 seconds
# 5. Foreground iOS
# 6. All 3 messages should appear — no manual refresh needed

# Watch OS-level wake events:
idevicesyslog | grep -E "background|foreground|push|wake|SCMessenger" | head -30
```

### 10b. Power profile transition evidence

The power profile code is in [MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift). Capture it working:

```bash
# Drain battery below 20% OR put device in Low Power Mode
# Watch for profile transition in syslog:
idevicesyslog | grep -E "powerProfile\|applyPowerAdjustments\|AdjustmentProfile\|Standard\|Low"
# Expected: "Applying power profile: Reduced" or similar when battery drops
```

### 10c. Reconnect after background kill

```bash
# Force-kill iOS app from iOS Settings > SCMessenger > Force Quit
# Send message from Android
# Relaunch iOS app
# Message should appear immediately on reopen (store-and-forward + local history)
```

**Acceptance:** All 3 backgrounded messages appear on foreground without manual refresh. Power profile log shows `Standard → Low/Reduced → Standard`. Reinstall/kill doesn't lose messages.

---

## Quick Reference: File Map

| Area | iOS File | Android File |
|---|---|---|
| BLE send path | [Transport/BLEPeripheralManager.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift) | [transport/ble/BleGattClient.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt), [BleGattServer.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt) |
| BLE scan/discovery | [Transport/BLECentralManager.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift) | [transport/ble/BleScanner.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt), [BleAdvertiser.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt) |
| Message repo / outbox | [Data/MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) | [data/MeshRepository.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) |
| Chat VM + sort | [ViewModels/ChatViewModel.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift) line 32 | [ui/viewmodels/ChatViewModel.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt) line 85 |
| Chat UI + scroll | [Views/Navigation/MainTabView.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift) line 284–312 | [ui/screens/ChatScreen.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt) |
| Multipeer transport | [Transport/MultipeerTransport.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/MultipeerTransport.swift) | [transport/WifiDirectTransport.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/WifiDirectTransport.kt) |
| Bootstrap wiring | [Data/MeshRepository.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) | [data/MeshRepository.kt](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) |
| Settings/privacy | [ViewModels/SettingsViewModel.swift](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift) | `ui/viewmodels/SettingsViewModel.kt` |
| Core API contracts | [core/src/api.udl](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl) | [core/src/api.udl](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl) |
| Message history store | [core/src/store/history.rs](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/store/history.rs) | [core/src/store/history.rs](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/store/history.rs) |

## Quick Reference: Verification Commands

```bash
# Run all Rust tests
cargo test --workspace

# Android compile check
cd android && ANDROID_HOME=~/Library/Android/sdk ./gradlew :app:compileDebugKotlin :app:lintDebug

# iOS build check
bash ./iOS/verify-test.sh

# Full 5-node live test with receipt gate
./scripts/run5-live-feedback.sh --step=<id> --time=6 --attempts=3 --require-receipt-gate

# Verify receipt convergence in logs
bash ./scripts/verify_receipt_convergence.sh <android-log> <ios-log>

# Verify no BLE-only regression
bash ./scripts/verify_ble_only_pairing.sh <android-log> <ios-log>

# Verify no relay flap regression
bash ./scripts/verify_relay_flap_regression.sh <ios-log>

# Pull Android diagnostics
adb shell run-as com.scmessenger.android cat files/mesh_diagnostics.log > /tmp/android_diag.log
adb shell run-as com.scmessenger.android cat files/pending_outbox.json

# Pull iOS crash reports
idevicecrashreport -u <udid> -e /tmp/crash_pull/
```

## Completion Criteria Summary

| # | Done When |
|---|---|
| 1 | Zero crash `.ips` files after iOS send scenario on physical device |
| 2 | `verify_receipt_convergence.sh` finds matched IDs in synchronized Android+iOS logs |
| 3 | Same conversation shows identical order on both platforms with cross-platform live exchange |
| 4 | 10+ messages in live conversation, zero scroll-to-top, zero flicker |
| 5 | Each transport path field-proven in isolation with evidence in INTEROP_MATRIX |
| 6 | Zero hits for stale MAC / peer ID in post-WS12.31 Android diagnostics log |
| 7 | Cross-carrier message delivery within 60s, no permanent `stored` state |
| 8 | App stays connected with primary relay offline; second relay handles traffic |
| 9 | Offline recipient receives message on reconnect without sender resend |
| 10 | Backgrounded messages appear on foreground; power profile transitions logged |
