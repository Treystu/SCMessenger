# 📱 Mobile Log Extraction - Quick Reference

**⚠️ MANDATORY: All AI agents and developers must use these standardized scripts**

Full documentation: [LOG_EXTRACTION_STANDARD.md](LOG_EXTRACTION_STANDARD.md)

---

## iOS

```bash
python3 ios_extractor.py        # Start extraction
python3 ios_extractor.py -h     # Show help
python3 ios_extractor.py -v     # Show version
# Runs continuously until Ctrl+C
```

**Output:**
- `ios_diagnostic_snapshot.log` - Structured diagnostics (PRIMARY)
- `live_ios_log.log` - Continuous capture

**Quick Start:** [QUICKSTART_IOS_LOGS.md](QUICKSTART_IOS_LOGS.md)

---

## Android

```bash
python3 adb_extractor.py        # Start extraction
python3 adb_extractor.py -h     # Show help
python3 adb_extractor.py -v     # Show version
# Runs continuously until Ctrl+C
```

**Output:**
- `live_logcat.log` - Filtered logcat stream (PRIMARY)
- `diagnostic_snapshots/` - Diagnostic files

---

## ⚠️ Do NOT

- ❌ Create ad-hoc log extraction commands
- ❌ Ask users to run `adb logcat` or `idevicesyslog` manually
- ❌ Use generic logging without consulting these scripts
- ❌ Hardcode device paths or identifiers

---

## ✅ Always

- ✅ Use these standardized scripts for iOS/Android
- ✅ Reference script version in bug reports
- ✅ Let scripts run until Ctrl+C
- ✅ Review captured logs before sharing externally

---

**Read the full standard:** [LOG_EXTRACTION_STANDARD.md](LOG_EXTRACTION_STANDARD.md)
