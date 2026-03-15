# SCMessenger Master Bug Tracker

**Status:** Active
**Last Updated:** 2026-03-15
**Purpose:** Centralized tracking of all known bugs, issues, and risks across the SCMessenger codebase.

> **Note:** This tracker consolidates issues from all documentation sources. For detailed implementation plans, see [`docs/implementation_cheatsheet_3.4.2026.md`](docs/implementation_cheatsheet_3.4.2026.md). For edge-case scenarios, see [`docs/EDGE_CASE_READINESS_MATRIX.md`](docs/EDGE_CASE_READINESS_MATRIX.md).

---

## Status Legend

| Symbol | Status | Description |
|--------|--------|-------------|
| 🔴 | **Open** | Known and unresolved |
| 🟡 | **In Progress** | Fix in development or testing |
| 🟢 | **Fixed** | Fix implemented, awaiting verification |
| ✅ | **Closed** | Resolved with verification evidence |
| ⏸️ | **Deferred** | Not accepted for current release; explicit follow-up required |
| ⚠️ | **Accepted** | Known and intentionally tolerated for this release |

---

## Priority Legend

| Level | Description |
|-------|-------------|
| **P0** | Critical - Blocks core functionality or causes crashes |
| **P1** | High - Significantly impacts user experience |
| **P2** | Medium - Noticeable but workaround exists |
| **P3** | Low - Minor issue, cosmetic or edge case |

---

## Active Open Issues (v0.2.1 Release Blockers)

### AND-BACKUP-001: Android Auto Backup Restores Stale Data on Fresh Install

| Field | Value |
|-------|-------|
| **ID** | AND-BACKUP-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | v0.2.0 / v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Last Verified** | 2026-03-14 |
| **Source** | `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md`, `V0.2.0_RESIDUAL_RISK_REGISTER.md` (R-2026-03-14-01) |

**Symptom:**  
Fresh install shows pre-existing messages from previous installations. 2026-03-14 audit found 4 pre-existing messages on fresh install.

**Root Cause:**  
`android:allowBackup="true"` in AndroidManifest.xml enables automatic restore of SharedPreferences and database files. Backup exclusion rules do not cover contacts.db, history.db, or identity_backup_prefs.

**Fix Required:**
- Update `backup_rules.xml` to exclude database files (contacts.db, history.db) and identity backup prefs
- Update `data_extraction_rules.xml` similarly
- OR set `android:allowBackup="false"` (simpler but loses identity on reinstall)

**Test Case:**  
Fresh install should have zero pre-existing messages.

---

### AND-RELAY-CONTACTS-001: Relay Peers Auto-Discovered as User Contacts

| Field | Value |
|-------|-------|
| **ID** | AND-RELAY-CONTACTS-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Last Verified** | 2026-03-14 |
| **Source** | `REMAINING_WORK_TRACKING.md` |

**Symptom:**  
Relay server (external relay peer) auto-discovered and shown with nickname "peer-93a35a87" in user contact list.

**Root Cause:**  
Relay peers are being treated as regular mesh peers and auto-created as contacts during discovery.

**Fix Required:**  
Design decision + implementation to filter or tag relay peers appropriately.

---

### AND-PERMISSION-001: Permission Request Loop on App Startup

| Field | Value |
|-------|-------|
| **ID** | AND-PERMISSION-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Last Verified** | 2026-03-14 |
| **Source** | `REMAINING_WORK_TRACKING.md` |

**Symptom:**  
9+ rapid permission requests (location, BLE, notifications, nearby WiFi) in ~700ms on fresh app launch.

**Root Cause:**  
Multiple code paths requesting same permissions without deduplication.

**Fix Required:**
- Deduplicate permission requests in MainActivity
- Coordinate all permission sources into single request
- Add request state machine + backoff timer

---

### AND-STALE-PEER-001: Gratuitous Nearby Entries Persistence

| Field | Value |
|-------|-------|
| **ID** | AND-STALE-PEER-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Last Verified** | 2026-03-14 |
| **Source** | `REMAINING_WORK_TRACKING.md` |

**Symptom:**  
Discovered peer continues showing in UI for 6+ seconds after discovery is stopped.

**Root Cause:**  
Async discovery lifecycle or UI refresh batching.

**Fix Required:**
- Profile Nearby Discovery stop propagation timing
- Ensure immediate UI removal of peers when discovery stops
- Remove stale peer cache entries synchronously

---

### AND-CONTACT-DUP-001: Contact Duplication During Peer Discovery

| Field | Value |
|-------|-------|
| **ID** | AND-CONTACT-DUP-001 |
| **Status** | 🟡 In Progress |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | WS13.6 |
| **First Seen** | 2026-03-14 |
| **Last Verified** | 2026-03-14 |
| **Source** | `V0.2.0_RESIDUAL_RISK_REGISTER.md` (R-WS13.6-01) |

**Symptom:**  
Duplicate `onPeerIdentified` callbacks for same peer ID during discovery.

**Root Cause:**
- MeshEventBus or discovery stack may be emitting duplicate IdentityDiscovered events
- Contact creation callback not idempotent
- No deduplication logic on peer promotion callback

**Fix Required:**
- Implement idempotent contact upsert (not insert)
- Add unique constraint on peer_id in contacts table

---

### FIELD-BINARY-001: Field iOS Binary Version is Stale

| Field | Value |
|-------|-------|
| **ID** | FIELD-BINARY-001 |
| **Status** | 🔴 Open |
| **Priority** | P0 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Field iOS binary version is stale vs current source hardening (crash fix exists in source but not validated on deployed build).

**Fix Required:**  
Deploy latest iOS binary containing WS12.22+ fixes; capture post-deploy crash-free evidence.

---

### CROSS-PAIR-001: iOS/Android Cross-Device Continuity

| Field | Value |
|-------|-------|
| **ID** | CROSS-PAIR-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Cross-platform |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS physical device and Android cross-device continuity is still not closed with synchronized evidence.

**Fix Required:**  
Synchronized tri-platform artifact bundle shows discovery + bidirectional send + receipt convergence.

---

### IOS-DIAG-001: iOS Diagnostics File Extraction Unreliable

| Field | Value |
|-------|-------|
| **ID** | IOS-DIAG-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS diagnostics file extraction is unreliable (socket closes on large file transfer), reducing RCA speed.

**Fix Required:**  
Reliable pull path documented and validated for large diagnostics artifacts.

---

### OPS-ADB-001: Android Wireless ADB Endpoint Stability

| Field | Value |
|-------|-------|
| **ID** | OPS-ADB-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Android wireless ADB endpoint stability still drifts during reconnect cycles.

**Fix Required:**  
Stable reconnect behavior or scripted auto-recovery validated across multiple reconnects.

---

### TEST-ENV-001: Docker Simulation Verification Not Executed

| Field | Value |
|-------|-------|
| **ID** | TEST-ENV-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Infrastructure |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Docker simulation verification has not been executed on this host (environment debt).

**Fix Required:**  
Docker prerequisites resolved and simulation verification pass archived.

---

### VALIDATION-001: Required Closure Evidence Missing

| Field | Value |
|-------|-------|
| **ID** | VALIDATION-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Cross-platform |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Last Verified** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Required closure evidence remains missing (live network matrix, ACK-safe switch, reinstall continuity, iOS power profile evidence).

**Fix Required:**  
All closure artifacts captured and linked from canonical docs.

---

### AND-SEND-BTN-001: Send Button Not Responding

| Field | Value |
|-------|-------|
| **ID** | AND-SEND-BTN-001 |
| **Status** | 🔴 Open |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `ANDROID_DELIVERY_ISSUES_2026-03-10.md` |

**Symptom:**  
User reports clicking send button 100+ times with no response. No `SEND_BUTTON_CLICKED` log entries detected.

**Root Cause:**  
UI thread blocked/frozen, Compose recomposition issue, or coroutine scope cancellation.

**Fix Required:**
- Check for UI thread blocking
- Verify Compose button click handler
- Add defensive logging before/after sendMessage call

---

### AND-DELIVERY-001: Delivery State Tracking Broken

| Field | Value |
|-------|-------|
| **ID** | AND-DELIVERY-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `ANDROID_DELIVERY_ISSUES_2026-03-10.md` |

**Symptom:**  
Multiple log entries show `msg=unknown` instead of actual message IDs. Messages failing to send with "Network error". Delivery attempt count at 169 for one message.

**Fix Required:**
- Find why `msg=unknown` is appearing
- Ensure message ID is properly propagated
- Implement max retry limit

---

### AND-MSG-VIS-001: Message Visibility Issues

| Field | Value |
|-------|-------|
| **ID** | AND-MSG-VIS-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `ANDROID_DELIVERY_ISSUES_2026-03-10.md` |

**Symptom:**  
Messages may "disappear" from UI due to missing state updates. UI may only show "delivered" messages, hiding "pending" ones.

**Fix Required:**
- Ensure "pending" messages show in UI
- Add delivery state indicator to message list
- Fix filter logic that may hide unsent messages

---

### IOS-CONV-DEL-001: Conversation Deletion Not Persisting

| Field | Value |
|-------|-------|
| **ID** | IOS-CONV-DEL-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | iOS |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `IOS_ISSUES_2026-03-10.md` |

**Symptom:**  
User deletes a conversation in iOS app, but conversation reappears almost immediately. Deletion does not persist to storage.

**Root Cause:**  
History sync from other device restores messages. Deletion not calling `remove_conversation()` on history manager.

**Fix Required:**
- Verify it calls `historyManager.removeConversation(peerId)`
- Implement deletion marker or suppress deleted conversations

---

### IOS-FREEZE-001: App Freezing and Hanging

| Field | Value |
|-------|-------|
| **ID** | IOS-FREEZE-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | iOS |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `IOS_ISSUES_2026-03-10.md` |

**Symptom:**  
iOS app becomes unresponsive. UI freezes during operations, particularly during message sends, peer discovery, and contact operations.

**Root Cause:**  
Main thread blocking, excessive debug logging, or SwiftUI state thrashing.

**Fix Required:**
- Move heavy operations to background
- Reduce log verbosity
- Optimize SwiftUI state updates

---

### AND-CELLULAR-001: Android Cellular Cannot Send Messages

| Field | Value |
|-------|-------|
| **ID** | AND-CELLULAR-001 |
| **Status** | 🔴 Open |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-09 |
| **Last Verified** | 2026-03-09 |
| **Source** | `CELLULAR_NAT_SOLUTION.md` |

**Symptom:**  
Android device on cellular network cannot send messages to iOS device despite both apps running. All relay dials return "Network error".

**Root Cause:**  
Android's TCP transport cannot establish outbound connections to relay servers from cellular network. Carrier-level TCP port filtering.

**Fix Required:**
- Add UDP/QUIC transport fallback
- Implement aggressive relay bootstrap retry with exponential backoff

---

### CROSS-RELAY-001: Relay Circuit Delivery Failing

| Field | Value |
|-------|-------|
| **ID** | CROSS-RELAY-001 |
| **Status** | 🔴 Open |
| **Priority** | P0 |
| **Platform** | Cross-platform |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-09 |
| **Last Verified** | 2026-03-09 |
| **Source** | `MESSAGE_DELIVERY_RCA_2026-03-09.md` |

**Symptom:**  
Both devices cannot send messages via relay circuit despite iOS being connected to relay. IronCoreError error 4 (NetworkError).

**Fix Required:**  
Verify relay server is running and accepting circuit relay requests.

---

### AND-NICK-001: Nickname Display Showing IDs Instead of Names

| Field | Value |
|-------|-------|
| **ID** | AND-NICK-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `IMMEDIATE_ACTION_CHECKLIST.md` |

**Symptom:**  
Conversations show IDs like `f77690efd...` instead of "John".

**Fix Required:**  
Use Contact.displayName() in UI.

---

### AND-BLOCK-001: No Block Button for Contacts

| Field | Value |
|-------|-------|
| **ID** | AND-BLOCK-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `IMMEDIATE_ACTION_CHECKLIST.md` |

**Symptom:**  
Can't block annoying users.

**Fix Required:**  
Add menu item in ChatScreen.

---

### IOS-RESOLVER-001: iOS Missing ID Resolver

| Field | Value |
|-------|-------|
| **ID** | IOS-RESOLVER-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | iOS |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Last Verified** | 2026-03-10 |
| **Source** | `IMMEDIATE_ACTION_CHECKLIST.md` |

**Symptom:**  
iOS doesn't use new ID resolver yet.

**Fix Required:**  
Integrate resolve_identity() into iOS.

---

## Closed/Fixed Issues

### IOS-CRASH-001: iOS SIGTRAP Crash in BLE Peripheral Send Path

| Field | Value |
|-------|-------|
| **ID** | IOS-CRASH-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS app crashes during message send (`SIGTRAP`) in BLE peripheral send path.

**Resolution:**  
Added `peripheralManager.state == .poweredOn` guard before every `updateValue` call and in `processPendingNotifications`.

---

### IOS-PERF-001: iOS CPU Watchdog Kill Under Retry Pressure

| Field | Value |
|-------|-------|
| **ID** | IOS-PERF-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS process is killed by CPU resource watchdog under retry pressure.

**Resolution:**  
Added `Task.yield()` in outbox flush loop; CPU no longer saturates during retry storms.

---

### AND-ROUTE-001: Android Stale Route Peer ID Retries

| Field | Value |
|-------|-------|
| **ID** | AND-ROUTE-001 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Android repeatedly retries stale route peer ID.

**Resolution:**  
`appendRoutingHint` now replaces stale entries. One-time `migrateStaleRoutingHints()` migration strips all inherited stale `libp2p_peer_id`/`ble_peer_id` from contact notes on first launch.

---

### IOS-RELAY-001: iOS Relay Flapping Prevents Circuit Relay Path

| Field | Value |
|-------|-------|
| **ID** | IOS-RELAY-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS stuck in permanent `flapping` state (40-52 events/60s), never establishes relay reservations.

**Resolution:**  
Threshold raised 6→30, debounced events excluded from counter, backoff extended 12s→30s.

---

### AND-BLE-001: Android BLE Fallback Targets Stale BLE Peer

| Field | Value |
|-------|-------|
| **ID** | AND-BLE-001 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Android BLE fallback repeatedly targets stale/unavailable BLE peer (`65:99:F2:D9:77:01`).

**Resolution:**  
`appendRoutingHint` now replaces old BLE MAC with new one instead of appending duplicates.

---

### UX-IOS-002: iOS Message List Scrolling Issues

| Field | Value |
|-------|-------|
| **ID** | UX-IOS-002 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | iOS |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS message list keeps scrolling to the top of the window unprompted, and refreshes erratically.

**Resolution:**  
Changed scroll trigger from `messages.last?.id` to `messages.count` so delivery-state updates don't cause scroll jumps.

---

### MSG-ORDER-001: iOS and Android Disagree on Message Ordering

| Field | Value |
|-------|-------|
| **ID** | MSG-ORDER-001 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | Cross-platform |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
iOS and Android disagree on message ordering.

**Resolution:**  
All sort logic on both platforms now uses `senderTimestamp` as canonical sort key.

---

### AND-SCHEMA-001: Android Message Deserialization Crash

| Field | Value |
|-------|-------|
| **ID** | AND-SCHEMA-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Android messages disappear on send / "Failed to load messages: Internal error".

**Resolution:**  
Legacy `MessageRecord` deserialization crash due to missing `sender_timestamp` in Sled DB fixed with `#[serde(default)]` + `adjust_legacy_timestamps()`.

---

### RECEIPT-001: Delivery Receipts Falsely Rejected

| Field | Value |
|-------|-------|
| **ID** | RECEIPT-001 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | Cross-platform |
| **Phase** | WS12.29 |
| **First Seen** | 2026-03-04 |
| **Fixed** | 2026-03-04 |
| **Source** | `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` |

**Symptom:**  
Delivery receipts falsely rejected: "sender key does not match outbound recipient".

**Resolution:**  
Relaxed receipt filter to only require `direction == Sent`.

---

### AND-ID-UNIFY-001: Send Message Failure - Invalid Public Key

| Field | Value |
|-------|-------|
| **ID** | AND-ID-UNIFY-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Fixed** | 2026-03-14 |
| **Source** | `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` |

**Symptom:**  
User cannot send messages to saved contacts; `IronCoreException$InvalidInput` at `prepareMessageWithId()`.

**Root Cause:**  
ID Type Confusion - Android passing peer_id hash instead of Ed25519 public_key to encryption function.

**Resolution:**  
Added validation in MeshRepository.kt to check public key length (must be 64 hex chars), added recovery logic to fall back to discovered peers cache.

---

### AND-ID-UNIFY-002: Contact Recognition Failure in Chat

| Field | Value |
|-------|-------|
| **ID** | AND-ID-UNIFY-002 |
| **Status** | ✅ Closed |
| **Priority** | P1 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-14 |
| **Fixed** | 2026-03-14 |
| **Source** | `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` |

**Symptom:**  
Saved contacts show as "not found" (`contactFound=false`) in chat screen despite being in database.

**Root Cause:**  
ID Truncation/Normalization Mismatch between ContactsViewModel (uses 16-char prefix) and ChatScreen (uses full 64-char ID).

**Resolution:**  
Added canonicalContactId() normalization in MeshRepository.kt, implemented public-key-first matching.

---

### AND-INIT-001: IronCore Not Initialized on Startup

| Field | Value |
|-------|-------|
| **ID** | AND-INIT-001 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Fixed** | 2026-03-10 |
| **Source** | `PEER_ID_RESOLUTION_FIX.md` |

**Symptom:**  
`uniffi.api.IronCoreException$NotInitialized` exceptions. Send failures with no clear error to user. Messages appearing to "disappear".

**Resolution:**  
Added initialization checks to `sendHistorySyncIfNeeded()` and `sendIdentitySyncIfNeeded()`.

---

### AND-MSG-PERSIST-002: Case-Sensitive Peer ID Matching

| Field | Value |
|-------|-------|
| **ID** | AND-MSG-PERSIST-002 |
| **Status** | ✅ Closed |
| **Priority** | P0 |
| **Platform** | Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-10 |
| **Fixed** | 2026-03-10 |
| **Source** | `ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md` |

**Symptom:**  
History manager used `==` for peer ID comparison. Messages saved with one case variant, queried with another. Zero matches returned even though messages existed.

**Resolution:**  
Changed to `record.peer_id.eq_ignore_ascii_case(peer)` in `core/src/store/history.rs`.

---

## Global Viability Issues (from [`docs/global_viability_audit.md`](docs/global_viability_audit.md))

### P0 — Blocking Global Viability

| ID | Issue | Root Cause | Impact | First Seen | Last Verified | Status |
|----|-------|------------|--------|------------|---------------|--------|
| IOS-CRASH-001 | iOS BLE send path crashes with `SIGTRAP` in `BLEPeripheralManager.sendDataToCentral` | Remaining force-unwrap / assert in send path despite hotfix attempts | Device unreliable as sender; crashes lose messages | 2026-03-04 | 2026-03-04 | 🔴 Open |
| IOS-PERF-001 | iOS killed by CPU watchdog under retry pressure | Retry loop consumes ~99% CPU without yield | iOS effectively non-functional under load | 2026-03-04 | 2026-03-04 | 🔴 Open |
| CROSS-PAIR-001 | Android ↔ iOS bidirectional delivery not converging to `delivered` state end-to-end | BLE path mismatch, stale route hints, receipt emission gaps | Core use case (phone-to-phone) not proven | 2026-03-04 | 2026-03-04 | 🔴 Open |
| MSG-ORDER-001 | iOS and Android show different message ordering for same conversation | `sender_timestamp` not used consistently as authoritative sort key | Conversation incoherent; ordering is platform-dependent | 2026-03-04 | 2026-03-04 | 🔴 Open |

### P1 — Significant for Global Viability

| ID | Issue | Root Cause | Impact | First Seen | Last Verified | Status |
|----|-------|------------|--------|------------|---------------|--------|
| UX-IOS-002 | iOS message list scroll-to-top and erratic refresh during conversation | UI state not stable under concurrent SwiftUI update emissions | App feels buggy; hurts adoption | 2026-03-04 | 2026-03-04 | 🔴 Open |
| AND-ROUTE-001 | Android retries stale route peer IDs; 291 `Network error` events per session | Failed route IDs persisted back into `routePeerId` on failure | Messages stuck `stored`; delivery never converges | 2026-03-04 | 2026-03-04 | 🔴 Open |
| AND-BLE-001 | Android BLE fallback targets stale unavailable MAC (`65:99:F2:D9:77:01`) repeatedly | BLE cache not invalidated on disconnect; stale hint leaks | BLE path loops, never converges | 2026-03-04 | 2026-03-04 | 🔴 Open |
| IOS-DIAG-001 | iOS diagnostic file pull fails via socket (large files ~21MB) | `ios_diagnostics.log` grows to 10–21MB; socket closes mid-transfer | Slows debugging; can't get device-side ground truth | 2026-03-04 | 2026-03-04 | 🔴 Open |
| FIELD-BINARY-001 | Physical iOS device running stale build (v0.2.0 build 4) without latest hardening | Manual install required; no OTA push | All field evidence from an old binary | 2026-03-04 | 2026-03-04 | 🔴 Open |

### P2 — Quality / Completeness Gaps

| ID | Issue | Impact | First Seen | Last Verified | Status |
|----|-------|--------|------------|---------------|--------|
| OPS-ADB-001 | Android wireless ADB drifts; needs manual reconnect | Slows iteration speed; not user-facing | 2026-03-04 | 2026-03-04 | 🔴 Open |
| TEST-ENV-001 | Docker simulation not validated on this host | Integration test coverage gap for NAT/relay scenarios | 2026-03-04 | 2026-03-04 | 🔴 Open |
| VALIDATION-001 | Live field matrix not yet captured (CGNAT, captive portal, cross-region relay) | Can't prove global internet reachability beyond LAN+GCP | 2026-03-04 | 2026-03-04 | 🔴 Open |
| UX-IOS-001 | Contact deletion confirmation (implemented WS12.31) — keep under regression | Safety regression risk | 2026-03-04 | 2026-03-04 | 🟡 In Progress |
| EC-01 | Relay custody temp-dir → durable in all environments | Store-and-forward durability at risk on some devices | 2026-03-04 | 2026-03-04 | 🔴 Open |
| EC-02 | `DeviceStorageSnapshot` unavailability causes pressure policy to silently no-op | Message retention on low-storage devices unguarded | 2026-03-04 | 2026-03-04 | 🔴 Open |
| EC-03 | Stale local transport hints (WiFi Direct hints on Android, Multipeer on iOS) | Local fast-path hit rate degrades over time | 2026-03-04 | 2026-03-04 | 🔴 Open |
| WASM-GAP-001 | WASM/Web is internet-path only (no BLE/WiFi); marked experimental | Web clients can only participate as internet relay nodes | 2026-03-04 | 2026-03-04 | 🔴 Open |

---

## Edge-Case Readiness Gaps (from [`docs/EDGE_CASE_READINESS_MATRIX.md`](docs/EDGE_CASE_READINESS_MATRIX.md))

| Scenario | Readiness | Primary Gap | First Seen | Last Verified |
|----------|-----------|-------------|------------|---------------|
| Dense local outage (city blackout) | Medium | Encounter quality and stale local route hints | 2026-03-03 | 2026-03-03 |
| Sparse offline region (rural, disaster) | Medium | Delivery latency unbounded without encounter heuristics | 2026-03-03 | 2026-03-03 |
| Airplane mode / radios disabled | Medium | No user-visible "deferred by radio-off state" diagnostics | 2026-03-03 | 2026-03-03 |
| In-flight WiFi (restricted egress) | Low-Medium | No captive-portal/filtered-egress detection | 2026-03-03 | 2026-03-03 |
| Subway/tunnel commuting | Medium | Burst-window prioritization not tuned for short-lived opportunities | 2026-03-03 | 2026-03-03 |
| High-speed travel (train/car) | Medium | Route-recency quality decays under rapid topology churn | 2026-03-03 | 2026-03-03 |
| Carrier-grade NAT / symmetric NAT | Medium-High | Relay custody durability path should move from temp-dir to durable app data | 2026-03-03 | 2026-03-03 |
| Enterprise/school network filtering | Medium | No explicit transport-profile policy for restricted environments | 2026-03-03 | 2026-03-03 |
| IPv6-only / NAT64 transitions | Medium | Need explicit NAT64/IPv6-only validation matrix | 2026-03-03 | 2026-03-03 |
| Satellite / high-latency links | Medium | Retry/backoff not profile-tuned for high-latency links | 2026-03-03 | 2026-03-03 |
| OS background kill (iOS/Android) | Medium | Wake/reconnect reliability and delegate-style wake strategy | 2026-03-03 | 2026-03-03 |
| Battery-critical or thermal throttling | Medium | No explicit power-mode routing profile | 2026-03-03 | 2026-03-03 |
| Disk nearly full | Medium | Snapshot unavailability can disable pressure policy | 2026-03-03 | 2026-03-03 |
| Clock skew / wrong system time | Low-Medium | Need clock-skew-tolerant ordering/recency normalization | 2026-03-03 | 2026-03-03 |
| High churn crowd events | Medium | Convergence marker trust policy needs hardening | 2026-03-03 | 2026-03-03 |
| Recycled identity collisions | Low (today) | Requires WS13 implementation for robust resolution | 2026-03-03 | 2026-03-03 |
| Censored/hostile jurisdictions | Low | Needs explicit "restricted environment mode" guidance | 2026-03-03 | 2026-03-03 |

---

## iOS Simulator Launch Issues (2026-03-13)

| ID | Issue | Status | Severity | First Seen | Last Verified |
|----|-------|--------|----------|------------|---------------|
| SIM-LAUNCH-001 | Stale device-flavor SCMessenger bundle remains installed in iOS simulator | 🔴 Open | Medium | 2026-03-13 | 2026-03-13 |
| SIM-TRANSPORT-001 | iOS transport state appears healthier in system logs than app-level visibility | 🔴 Open | Medium | 2026-03-13 | 2026-03-13 |
| SIM-SEND-PATH-001 | iOS send-path behavior may diverge from store-and-forward-first model | 🔴 Open | Medium | 2026-03-13 | 2026-03-13 |

---

## Android BLE Freshness Issues (2026-03-13)

| ID | Issue | Status | Priority | First Seen | Last Verified |
|----|-------|--------|----------|------------|---------------|
| BLE-FRESH-001 | Android BLE fallback telemetry shows stale MAC while callback uses fresher GATT address | 🔴 Open | P2 | 2026-03-13 | 2026-03-13 |
| BLE-OWNID-001 | Physical iOS app-level own-ID/peer capture gaps in harness evidence | 🔴 Open | P2 | 2026-03-13 | 2026-03-13 |

---

## iOS Send-Path Issues (2026-03-13)

| ID | Issue | Status | Priority | First Seen | Last Verified |
|----|-------|--------|----------|------------|---------------|
| IOS-SEND-001 | iOS send-path may block on live transport attempts instead of store-and-forward-first | 🔴 Open | P2 | 2026-03-13 | 2026-03-13 |
| IOS-IDENTITY-001 | iOS peer-identify/identity-beacon event storms cause transient freeze/unfreeze | 🔴 Open | P2 | 2026-03-13 | 2026-03-13 |

---

## Summary Statistics

| Status | Count |
|--------|-------|
| 🔴 Open | 30 |
| 🟡 In Progress | 1 |
| ✅ Closed | 12 |
| **Total** | **43** |

| Priority | Open Count |
|----------|------------|
| P0 | 8 |
| P1 | 12 |
| P2 | 10 |

---

## Related Documentation

- [`REMAINING_WORK_TRACKING.md`](REMAINING_WORK_TRACKING.md) - Active implementation backlog
- [`docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`](docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md) - Historical issue burndown
- [`docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`](docs/V0.2.0_RESIDUAL_RISK_REGISTER.md) - v0.2.0 risk register
- [`docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`](docs/V0.2.1_RESIDUAL_RISK_REGISTER.md) - v0.2.1 risk register
- [`ANDROID_ID_UNIFICATION_BUG_2026-03-14.md`](ANDROID_ID_UNIFICATION_BUG_2026-03-14.md) - Android ID issues
- [`BLE_DEADOBJECT_BUG.md`](BLE_DEADOBJECT_BUG.md) - BLE DeadObject bug details
- [`BLE_FALSE_DELIVERY_BUG.md`](BLE_FALSE_DELIVERY_BUG.md) - BLE false delivery bug details
- [`PHANTOM_PEERS_BUG.md`](PHANTOM_PEERS_BUG.md) - Phantom peers bug details
- [`ANDROID_DISCOVERY_ISSUES.md`](ANDROID_DISCOVERY_ISSUES.md) - Android discovery issues
- [`IOS_ISSUES_2026-03-10.md`](IOS_ISSUES_2026-03-10.md) - iOS issues
- [`ANDROID_DELIVERY_ISSUES_2026-03-10.md`](ANDROID_DELIVERY_ISSUES_2026-03-10.md) - Android delivery issues
- [`CELLULAR_NAT_SOLUTION.md`](CELLULAR_NAT_SOLUTION.md) - Cellular NAT issues
- [`MESSAGE_DELIVERY_RCA_2026-03-09.md`](MESSAGE_DELIVERY_RCA_2026-03-09.md) - Message delivery RCA
- [`ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md`](ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md) - Message persistence issues

---

## New Issues Reported (2026-03-15)

### NICKNAME-CRASH-001: Nickname Update Causes Crashes Due to Real-Time Propagation

| Field | Value |
|-------|-------|
| **ID** | NICKNAME-CRASH-001 |
| **Status** | 🔴 Open |
| **Priority** | P1 |
| **Platform** | Android (likely iOS too) |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-15 |
| **Last Verified** | 2026-03-15 |
| **Source** | User report during live debugging session |

**Symptom:**
Updating nickname causes app crashes. Edits try to propagate in real time for each character edit, causing crashes.

**Root Cause:**
Real-time character-by-character propagation instead of debounced/batched updates. Each keystroke triggers a network sync operation.

**Fix Required:**
Implement debounced nickname update function that waits for user to finish typing before propagating. Use a delay (e.g., 500ms) after last keystroke before syncing.

**Verification:**
1. Open contact details
2. Edit nickname field
3. Type multiple characters quickly
4. Verify no crash occurs
5. Verify nickname syncs after typing stops

---

### CONTACT-STALE-001: Stale Contact Shows After Deletion on Android

| Field | Value |
|-------|-------|
| **ID** | CONTACT-STALE-001 |
| **Status** | 🔴 Open |
| **Priority** | P2 |
| **Platform** | Android |
| **Phase** | v0.2.1 |
| **First Seen** | 2026-03-15 |
| **Last Verified** | 2026-03-15 |
| **Source** | User report during live debugging session |

**Symptom:**
Stale contact showed up after being deleted on Android.

**Root Cause:**
Contact cache not properly invalidated after deletion. UI may be showing cached data.

**Fix Required:**
Ensure contact deletion:
1. Clears all in-memory caches
2. Removes from local database
3. Triggers UI refresh via LiveData/StateFlow update
4. Broadcasts deletion event to all observers

**Verification:**
1. Delete a contact
2. Verify contact disappears from contact list immediately
3. Restart app
4. Verify deleted contact does not reappear

---

### TRANSPORT-001: Message Delivery Failure Between iOS and Android

| Field | Value |
|-------|-------|
| **ID** | TRANSPORT-001 |
| **Status** | 🟡 In Progress |
| **Priority** | P0 |
| **Platform** | iOS + Android |
| **Phase** | v0.2.0 |
| **First Seen** | 2026-03-13 |
| **Last Verified** | 2026-03-15 |
| **Source** | `logs/5mesh/20260313_041301/`, `TRANSPORT_FAILURE_ANALYSIS_2026-03-15.md` |

**Symptom:**
Messages not being delivered between iOS and Android. Pending/forwarding messages on both sides. Both BLE on, same LAN, share node peers, many viable paths but transport is broken.

**Root Cause (from log analysis):**
1. iOS app was crashing (SIGKILL/SIGTERM) - now rebuilt and reinstalled
2. BLE fallback skipped with "stale_ble_hint_no_fresh_observation" - BLE hints were stale because iOS app wasn't running
3. Core and relay transports failing with "Network error" - iOS device unreachable

**Current Status:**
- Both apps rebuilt and reinstalled (2026-03-15)
- iOS: Release build installed successfully
- Android: Debug build installed successfully (release keystore missing)
- Both apps launched and running

**Verification Pending:**
1. Send test message from Android to iOS
2. Verify message delivery
3. Check BLE fallback works if core transport fails
4. Monitor logs for transport success
