# iOS Log Extraction - Quick Start

## Fastest Way to Get Logs

```bash
python3 ios_extractor.py
```

**Need help?** Run `python3 ios_extractor.py -h` for detailed usage information.

The script will:
1. ✅ Auto-detect your connected iOS device
2. ✅ Extract the diagnostic snapshot (most useful logs)
3. ✅ Start continuous monitoring (runs until Ctrl+C)
4. ✅ Clean up automatically when stopped

**Press Ctrl+C to stop capturing and cleanup**

## Command Line Options

```bash
python3 ios_extractor.py       # Start log extraction
python3 ios_extractor.py -h    # Show detailed help
python3 ios_extractor.py -v    # Show version
```

## Output Files

- **`ios_diagnostic_snapshot.log`** ← Start here! Contains structured diagnostics
- **`live_ios_log.log`** ← Continuous live monitoring (grows over time)
- **`iOS_LOG_EXTRACTION_SUMMARY.md`** ← Full technical details

## What You'll See in the Logs

```
2026-03-19T08:05:01.033Z delivery_attempt msg=<uuid> medium=relay-circuit phase=retry outcome=success
2026-03-19T08:05:01.093Z ble_rx_start total=13 from=013CAD23
2026-03-19T08:05:01.392Z peer_identified transport=<peer-id> agent=scmessenger/0.2.0
```

**Key Events:**
- `delivery_attempt` - Message send tracking
- `delivery_state` - State changes (pending/stored/forwarding/delivered)
- `ble_rx_*` - Bluetooth activity
- `peer_identified` - Device discovery
- `relay_state` - Internet relay connections
- `power_profile` - Battery-aware behavior

## Optional: Real-Time Streaming

For true real-time OSLog streaming (not just polling):

```bash
brew install libimobiledevice
python3 ios_extractor.py
```

The script auto-detects `idevicesyslog` and uses it automatically for live streaming.

## Manual Export from App

1. Open SCMessenger app
2. Go to Settings → Diagnostics
3. Tap "Export Diagnostics Bundle"
4. Share to your Mac

This gives you an even more comprehensive diagnostic bundle!

## Continuous Monitoring

The script runs continuously like Android's `adb_extractor.py`:

- **Polling Mode** (default): Checks for new diagnostic entries every 5 seconds
- **Streaming Mode** (with libimobiledevice): Captures OSLog output in real-time

Both modes continue until you press **Ctrl+C**.

### Recommended Workflow

1. Start the extractor: `python3 ios_extractor.py`
2. Use the app (send messages, trigger the behavior you want to debug)
3. Watch the terminal for activity updates
4. Press **Ctrl+C** when done
5. Review `ios_diagnostic_snapshot.log` and `live_ios_log.log`

## Troubleshooting

**"Device not found"**
- Connect iPhone via USB
- Trust the Mac (tap "Trust" on iPhone)
- Check: `xcrun devicectl list devices`

**"No logs captured"**
- Make sure the app is running
- Send a test message to generate activity
- Check battery isn't too low (< 20%)

**"Stream started but no data captured"**
- This is normal if the app is idle
- Trigger some activity (send a message, open app)
- The stream is ready and will capture when events occur

**"Permission denied"**
- Make sure script is executable: `chmod +x ios_extractor.py`
- Or run with: `python3 ios_extractor.py`

## Understanding Delivery States

The most important debugging info:

1. **pending** → First send attempt in progress
2. **stored** → Queued (recipient offline or unreachable)
3. **forwarding** → Active retry happening now
4. **delivered** → Success! Receipt confirmed

If a message gets stuck in `stored` for a long time, that's what to investigate.

---

**Need help?** Check `iOS_LOG_EXTRACTION_SUMMARY.md` for full technical details.
