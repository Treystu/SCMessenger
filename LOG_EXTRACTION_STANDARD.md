# Log Extraction Standard

**Status:** MANDATORY  
**Scope:** All AI Agents, Developers, QA, Support  
**Last Updated:** 2026-03-19  
**Version:** 1.1

---

## ⚠️ CRITICAL: READ THIS FIRST

This document establishes the **approved and standardized method** for extracting logs from SCMessenger mobile applications. All AI models, developers, and support personnel **MUST** use these scripts when working with iOS and Android platforms.

## 🔴 URGENT: CRITICAL ISSUES DISCOVERED 2026-03-19

**Log audit reveals severe reliability issues:** [LOG_AUDIT_REPORT_2026-03-19.md](LOG_AUDIT_REPORT_2026-03-19.md)

- **Message Delivery:** iOS 22.7% success rate, Android 34.1% success rate  
- **BLE Issues:** Connection timeouts and instability on both platforms
- **Relay Circuits:** iOS experiencing connection failures

**When extracting logs, specifically monitor for:**
- `delivery_attempt` success/failure patterns
- BLE connection stability (`ble_central_connected`, write timeouts)
- Relay circuit issues (`relay_dial_debounced`, `relay_state`)

---

## 🎯 Standard Log Extraction Methods

### iOS Platform

**APPROVED METHOD:**
```bash
python3 ios_extractor.py
```

**Script Location:** `/ios_extractor.py` (repository root)

**What It Does:**
- ✅ Auto-detects connected iOS device via CoreDevice
- ✅ Extracts diagnostic snapshot from app container
- ✅ Runs continuous monitoring until Ctrl+C
- ✅ Two modes: Real-time streaming (idevicesyslog) or intelligent polling
- ✅ Graceful cleanup on exit
- ✅ Repository-specific configuration (no hardcoded paths)

**Output Files:**
- `ios_diagnostic_snapshot.log` - Point-in-time structured diagnostics (PRIMARY)
- `live_ios_log.log` - Continuous capture (grows over time)

**Requirements:**
- iOS device connected via USB and trusted
- macOS with Xcode Command Line Tools
- Python 3.7+
- Optional: libimobiledevice for real-time streaming (`brew install libimobiledevice`)

---

### Android Platform

**APPROVED METHOD:**
```bash
python3 adb_extractor.py
```

**Script Location:** `/adb_extractor.py` (repository root)

**What It Does:**
- ✅ Auto-detects connected Android device via ADB
- ✅ Extracts diagnostic snapshots from app private storage
- ✅ Runs continuous logcat monitoring until Ctrl+C
- ✅ PID-filtered for app-specific logs
- ✅ Graceful cleanup on exit
- ✅ Repository-specific configuration (no hardcoded paths)

**Output Files:**
- `live_logcat.log` - Continuous logcat stream (PRIMARY)
- `diagnostic_snapshots/` - Structured diagnostic files

**Requirements:**
- Android device connected via USB with USB debugging enabled
- ADB installed and in PATH
- Python 3.7+

---

## 📋 Standard Operating Procedure

### For AI Models Working on iOS/Android Issues

**DO:**
1. ✅ Use `ios_extractor.py` for iOS log extraction
2. ✅ Use `adb_extractor.py` for Android log extraction
3. ✅ Reference these scripts when asking users for logs
4. ✅ Review captured logs from the standard output files
5. ✅ Mention the scripts in issue reports and debugging sessions

**DO NOT:**
1. ❌ Create ad-hoc log extraction commands
2. ❌ Ask users to manually run `adb logcat` or `idevicesyslog`
3. ❌ Use generic logging approaches without consulting these scripts
4. ❌ Hardcode device paths or bundle identifiers
5. ❌ Bypass the standardized extraction process

### For Developers and QA

**When Debugging:**
1. Start the appropriate extractor script
2. Reproduce the issue while logs are being captured
3. Press Ctrl+C when done
4. Attach the generated log files to bug reports
5. Reference script version in reports

**When Reporting Issues:**
- ✅ "Logs captured using ios_extractor.py v1.0"
- ✅ "Android logs from adb_extractor.py (464 lines captured)"
- ❌ "Here are some logs I grabbed with adb logcat"

---

## 🔍 Why These Scripts Are Mandatory

### 1. Repository-Specific Configuration
Both scripts are specifically configured for SCMessenger:
- **iOS:** `SovereignCommunications.SCMessenger` bundle ID, `com.scmessenger` subsystem
- **Android:** `com.scmessenger.android` package, specific Timber tags
- Diagnostic log paths extracted from actual codebase analysis

### 2. Comprehensive Coverage
- System logs (OSLog/Logcat)
- Application diagnostic logs (mesh_diagnostics.log)
- Structured event capture
- Transport layer debugging
- Delivery state tracking

### 3. Consistency Across Team
- Same log format for all users
- Predictable output locations
- Standardized troubleshooting workflow
- Easier collaboration and issue triage

### 4. Production-Ready Quality
- Graceful error handling
- Process cleanup (no zombies)
- Device verification
- Continuous monitoring
- Ctrl+C interrupt support

### 5. Delta Detection (iOS Polling Mode)
- Only captures NEW log entries
- Efficient storage usage
- Clear timeline of events

---

## 📊 Log File Priority

### iOS Diagnostics

**Primary Source:**
```
ios_diagnostic_snapshot.log
```
This contains SCMessenger's structured diagnostic events:
- Message delivery attempts and states
- BLE transport activity (rx/tx)
- Peer identification and connections
- Relay circuit management
- Power profile adaptations
- Receipt acknowledgments

**Secondary Source:**
```
live_ios_log.log
```
Continuous monitoring, useful for:
- Long-running sessions
- Timing correlation
- Event sequences

### Android Diagnostics

**Primary Source:**
```
live_logcat.log
```
Filtered logcat output containing:
- App lifecycle events
- Transport router decisions
- Storage operations
- Core messaging flow

**Secondary Source:**
```
diagnostic_snapshots/*.log
```
Structured diagnostic files:
- mesh_diagnostics.log (primary)
- Rotated backups (.log.1, .log.2, etc.)

---

## 🚀 Quick Reference

### iOS
```bash
# Standard extraction
python3 ios_extractor.py

# With real-time streaming (optional)
brew install libimobiledevice
python3 ios_extractor.py

# Stop capturing
[Ctrl+C]
```

### Android
```bash
# Standard extraction
python3 adb_extractor.py

# Stop capturing
[Ctrl+C]
```

### Both Platforms
- ✅ Scripts run continuously until Ctrl+C
- ✅ Safe to interrupt at any time
- ✅ Automatic cleanup on exit
- ✅ Works with device already connected and app running

---

## 🔧 Troubleshooting the Scripts

### Common Issues

**iOS: "Device not found"**
```bash
# Verify device connection
xcrun devicectl list devices

# If not listed, reconnect USB and trust the Mac
```

**iOS: "No logs captured"**
- App may be idle - trigger activity (send a message)
- Battery may be low (< 20%) - charge device
- Check app is actually running

**Android: "No authorized device"**
```bash
# Verify ADB connection
adb devices

# If "unauthorized", check device screen for prompt
```

**Android: "App may not be running"**
- Launch the app manually
- Script will continue monitoring when app starts

### Getting Help

1. Check `QUICKSTART_IOS_LOGS.md` for detailed iOS instructions
2. Check `iOS_LOG_EXTRACTION_SUMMARY.md` for technical details
3. Review script header comments for usage notes
4. Ask in team channels with specific error messages

---

## 📝 For Other Platforms

### Desktop (macOS, Linux, Windows)
Currently **ad-hoc** - no standardized extraction script yet.

**Recommended Approaches:**
- macOS: Use Console.app or `log show` commands
- Linux: Check system logs in `/var/log/` or use `journalctl`
- Windows: Event Viewer or application-specific logs

### Web/WASM
Currently **ad-hoc** - use browser DevTools console.

### Headless/CLI
Check application output and any file-based logging configured.

**Future Work:** Consider creating standardized scripts for these platforms following the iOS/Android model.

---

## 🎓 Script Implementation Details

### iOS (ios_extractor.py)

**Phase 1: Repository Analysis**
- Discovers bundle ID from Xcode project
- Identifies OSLog subsystem from Swift files
- Locates diagnostic log path from MeshRepository.swift

**Phase 2: Extraction**
- Attempts idevicesyslog for real-time streaming
- Falls back to intelligent polling (5-second intervals)
- Extracts diagnostic snapshot via `devicectl copy`

**Phase 3: Monitoring**
- Continuous until Ctrl+C
- Delta detection in polling mode
- Periodic status updates

**Phase 4: Cleanup**
- SIGTERM → SIGKILL escalation
- Zombie process detection
- File handle closure

### Android (adb_extractor.py)

**Phase 1: Verification**
- Checks ADB device connection
- Verifies app is running (PID detection)
- Clears old logcat buffer

**Phase 2: Extraction**
- Starts PID-filtered logcat stream
- Extracts diagnostic files via `run-as`
- Pulls rotated log backups

**Phase 3: Monitoring**
- Continuous until Ctrl+C
- Live logcat stream to file
- Buffer verification

**Phase 4: Cleanup**
- SIGTERM to logcat process
- Process group termination
- File handle closure

---

## 🔐 Security and Privacy

### What Gets Captured

**Safe to capture:**
- ✅ Message IDs (UUIDs, no content)
- ✅ Peer IDs (libp2p identifiers)
- ✅ Transport metadata (BLE, relay, mDNS)
- ✅ Delivery states and timestamps
- ✅ Connection events and routing decisions

**NOT captured:**
- ❌ Message content/bodies
- ❌ User identities or contact names
- ❌ Private keys or secrets
- ❌ Personal information

### Sharing Logs

**Safe to share:**
- Log files from these scripts (diagnostic focus)
- Sanitized excerpts showing specific issues

**Use caution:**
- Review logs before sharing externally
- Redact peer IDs if publicly posting
- Don't commit log files to version control

---

## 📅 Maintenance and Updates

### Script Versioning
- Scripts follow semantic versioning
- Version documented in script headers
- Breaking changes communicated via this document

### Update Triggers
- Bundle ID or package name changes
- New diagnostic log locations
- Platform tool updates (devicectl, ADB)
- New logging categories or subsystems

### Who Can Modify
- Changes require review by maintainer
- Test with real devices before committing
- Update this document with any changes
- Document breaking changes in CHANGELOG

---

## ✅ Compliance Checklist

When working with iOS/Android logs, ensure:

- [ ] Using official extraction scripts (not ad-hoc commands)
- [ ] Device properly connected and authorized
- [ ] App running during extraction
- [ ] Ctrl+C used to stop (not kill -9)
- [ ] Output files reviewed for sensitive data
- [ ] Script version noted in bug reports
- [ ] Logs attached to issues in proper format

---

## 🔗 Related Documentation

- **Quick Start:** `QUICKSTART_IOS_LOGS.md`
- **iOS Technical Details:** `iOS_LOG_EXTRACTION_SUMMARY.md`
- **Repository Layout:** `REPO_LAYOUT.md`
- **Contribution Guide:** `CONTRIBUTING.md`
- **Agent Instructions:** `.github/COPILOT_AGENT_INSTRUCTIONS.md`

---

## 📞 Support and Questions

### For AI Models
- Follow this standard in all log extraction scenarios
- Reference these scripts when guiding users
- Do not deviate without explicit user request

### For Developers
- Use these scripts as the primary debugging tool
- Report issues with the scripts via GitHub Issues
- Suggest improvements via Pull Requests

### For QA/Support
- Train on these scripts for log collection
- Use standardized output in bug reports
- Share this document with external QA vendors

---

## 🎯 Key Takeaways

1. **iOS logs:** Use `ios_extractor.py` (mandatory)
2. **Android logs:** Use `adb_extractor.py` (mandatory)
3. **Both scripts:** Run until Ctrl+C (continuous monitoring)
4. **Output files:** Repository-standardized locations
5. **Other platforms:** Ad-hoc until standardized scripts exist

---

**This is the approved standard. Deviation requires explicit justification and maintainer approval.**

---

**Document Owner:** Repository Maintainers  
**Review Cycle:** Quarterly or on major platform updates  
**Status:** ✅ ACTIVE AND MANDATORY
