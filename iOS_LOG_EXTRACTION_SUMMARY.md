# iOS Log Extraction Summary

## Execution Date
2026-03-19 08:10 UTC

## Phase 1: Repository Analysis ✅

### Bundle Identifier
**Discovered:** `SovereignCommunications.SCMessenger`
- Source: `iOS/SCMessenger/SCMessenger.xcodeproj`

### OSLog Configuration
**Subsystem:** `com.scmessenger`

**Active Logging Categories:**
- `Multipeer` - Multipeer transport layer
- `BLE-L2CAP` - Bluetooth L2CAP connections
- `mDNS` - Service discovery
- `BLE-Peripheral` - Bluetooth peripheral mode
- `TransportRouter` - Smart transport routing
- `BLE-Central` - Bluetooth central mode
- `Repository` - Core data repository
- `Topics` - Topic management
- `ConversationList` - UI conversation list
- `Onboarding` - User onboarding flow
- `CoreDelegate` - Core delegate implementation
- `Background` - Background service operations
- `Notifications` - Notification management
- `Platform` - iOS platform bridge

### Diagnostic Log Configuration
**File Location:** `Documents/mesh_diagnostics.log`
- Stored in app's Documents directory
- Accessible via `UIFileSharingEnabled` (implicit via devicectl)
- Maximum 1000 lines in memory buffer
- Persistent file storage with append operations

## Phase 2: Extraction Results ✅

### 2A: Live Log Streaming
**Method Used:** Diagnostic snapshot polling (fallback method)
- **Output File:** `live_ios_log.log`
- **Size:** 8,367 bytes
- **Duration:** 15 seconds of monitoring
- **Format:** Timestamped diagnostic events

**Note:** For true real-time streaming, install libimobiledevice:
```bash
brew install libimobiledevice
# Then use: idevicesyslog -u <UDID> -m SCMessenger
```

### 2B: Diagnostic Snapshot
**Method Used:** `xcrun devicectl device copy from`
- **Output File:** `ios_diagnostic_snapshot.log`
- **Size:** 5,399 bytes
- **Lines:** 37 log entries
- **Status:** ✅ Successfully extracted

**Sample Diagnostic Events Captured:**
```
- delivery_attempt: Message delivery tracking
- delivery_state: Message state transitions (pending/stored/forwarding/delivered)
- ble_rx_start/complete: Bluetooth data reception
- peer_identified: Peer discovery and identification
- msg_rx: Message reception events
- connect_to_peer: Peer connection attempts
- relay_dial_debounced: Relay connection management
- relay_state: Relay circuit state tracking
- receipt_send: Delivery receipt transmission
- power_profile: Battery-aware power management
```

## Phase 3: Cleanup ✅
- All file operations completed successfully
- No zombie processes detected
- Temporary files stored in `tmp/` directory per repository guidelines

## Key Findings

### 1. Structured Diagnostic Logging
The app implements a custom diagnostic logging system that captures:
- Message delivery lifecycle (pending → stored → forwarding → delivered)
- Transport layer events (BLE, relay circuits, mDNS)
- Peer connection management
- Power profile adaptations
- Receipt acknowledgments

### 2. Multi-Transport Architecture
Evidence of sophisticated transport routing:
- BLE (Bluetooth Low Energy) for nearby direct communication
- Relay circuits for internet-roaming NAT traversal
- Smart router selecting optimal transport
- Connection debouncing and throttling

### 3. Delivery State Machine
Four-state delivery tracking:
- **pending**: Initial send attempt in progress
- **stored**: Queued for retry (recipient unreachable)
- **forwarding**: Active retry in progress
- **delivered**: Confirmed via receipt

### 4. Battery-Aware Operation
Power profile adaptation based on:
- Battery level (20% threshold observed)
- Motion detection
- Dynamic relay budget adjustment (100/hour in reduced mode)
- BLE scan interval scaling (5000ms in reduced mode)

## Device Information
- **Device Name:** christy's iPhone
- **Model:** iPhone 15 Pro Max (iPhone16,2)
- **UDID:** 4731D564-2F8F-5BC6-B713-D7774AF598F9
- **Connection:** CoreDevice (connected and trusted)
- **App Version:** 0.2.0 (build 4)

## Files Generated
1. `ios_extractor.py` - Python extraction script (repository-specific)
2. `live_ios_log.log` - Live diagnostic monitoring (polling mode)
3. `ios_diagnostic_snapshot.log` - Point-in-time diagnostic snapshot
4. `tmp/poll_snapshot_*.log` - Temporary polling snapshots

## Usage Instructions

### Run the Extractor
```bash
python3 ios_extractor.py
```

### For Real-Time Streaming (Optional Enhancement)
```bash
# Install libimobiledevice
brew install libimobiledevice

# Then re-run the script - it will auto-detect and use idevicesyslog
python3 ios_extractor.py
```

### Manual Diagnostic Export
Users can also export diagnostics directly from the app:
1. Open SCMessenger app
2. Navigate to Settings → Diagnostics
3. Tap "Export Diagnostics Bundle"
4. Share via AirDrop, Messages, or Files app

## Verification Status

| Component | Status | Notes |
|-----------|--------|-------|
| Bundle ID Discovery | ✅ | Found in Xcode project |
| OSLog Subsystem Discovery | ✅ | Found in 14 Swift files |
| Diagnostic Path Discovery | ✅ | Documents/mesh_diagnostics.log |
| Device Connection | ✅ | CoreDevice connected |
| Live Streaming | ⚠️ | Polling fallback used |
| Snapshot Extraction | ✅ | Full extraction successful |
| File Verification | ✅ | Non-zero bytes confirmed |
| Process Cleanup | ✅ | No zombie processes |

## Next Steps (Optional)
1. Install libimobiledevice for true real-time streaming
2. Enable verbose logging in app (DEBUG builds) for more detail
3. Use Xcode Console.app for graphical log inspection
4. Export full diagnostic bundle from app for comprehensive analysis

---
**Script:** `ios_extractor.py`
**Status:** Production Ready
**Maintenance:** Update DEVICE_UDID if device changes
