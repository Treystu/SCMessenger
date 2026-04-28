# Log Rotation and Retention Policy

**Status:** Active  
**Applies to:** iOS and Android log extractors  
**Last Updated:** 2026-03-19

## Overview

Both `ios_extractor.py` and `adb_extractor.py` now implement automatic log rotation and retention management to prevent unbounded disk usage while maintaining recent debugging history.

## Retention Policy

**Per Platform (iOS and Android are separate):**
- **Age Limit:** 24 hours
- **Size Limit:** 100 MB total
- **Policy:** Whichever limit is reached first triggers cleanup

### How It Works

1. **On Startup:** 
   - Previous logs are archived with timestamp
   - Old logs are evaluated for deletion
   - New capture session begins with clean files

2. **Age-Based Cleanup:**
   - Any log older than 24 hours is deleted
   - Evaluated before size-based cleanup

3. **Size-Based Cleanup:**
   - If total archived logs exceed 100 MB
   - Oldest logs deleted first until under limit
   - Applies separately to iOS and Android archives

## Archive Directories

```
ios_logs_archive/
  ├── live_ios_log_20260319_083027.log
  ├── ios_diagnostic_snapshot_20260319_083027.log
  ├── live_ios_log_20260318_143522.log
  └── ios_diagnostic_snapshot_20260318_143522.log

android_logs_archive/
  ├── live_logcat_20260319_083045.log
  ├── diagnostic_snapshots_20260319_083045/
  ├── live_logcat_20260318_143610.log
  └── diagnostic_snapshots_20260318_143610/
```

## File Naming Convention

### iOS
- Live logs: `live_ios_log_YYYYMMDD_HHMMSS.log`
- Snapshots: `ios_diagnostic_snapshot_YYYYMMDD_HHMMSS.log`

### Android
- Live logs: `live_logcat_YYYYMMDD_HHMMSS.log`
- Snapshot dirs: `diagnostic_snapshots_YYYYMMDD_HHMMSS/`

Timestamp format: `YYYYMMDD_HHMMSS` (e.g., `20260319_083027`)

## Behavior Examples

### Example 1: Age-Based Cleanup
```
Day 1 10:00 - Capture 50 MB of logs
Day 2 11:00 - Run script again
Result: Day 1 logs deleted (>24 hours old)
```

### Example 2: Size-Based Cleanup
```
Session 1: 30 MB captured
Session 2: 35 MB captured  
Session 3: 40 MB captured
Total: 105 MB

On Session 4 startup:
  → Session 1 deleted (oldest)
  → Total now 75 MB (under limit)
  → Session 4 begins fresh
```

### Example 3: Mixed Cleanup
```
Day 1 08:00: 20 MB captured
Day 1 14:00: 25 MB captured
Day 2 10:00: 30 MB captured
Day 2 16:00: 40 MB captured
Day 3 09:00: Run script

Age-based: Day 1 logs deleted (both sessions)
Size-based: Still under 100 MB
Result: Only Day 2 logs remain (70 MB total)
```

## Separate Limits for iOS and Android

Each platform has its own 100 MB budget:
- iOS can use up to 100 MB
- Android can use up to 100 MB
- Total possible: 200 MB (if both platforms used)

This separation ensures:
- Platform-specific debugging isn't impacted by the other
- Cross-platform development has sufficient history
- Each platform's logs are independently managed

## Manual Cleanup

To manually clean up all archived logs:

```bash
# iOS only
rm -rf ios_logs_archive/*

# Android only
rm -rf android_logs_archive/*

# Both platforms
rm -rf ios_logs_archive/* android_logs_archive/*
```

## Bypassing Rotation

If you need to preserve specific logs:

1. **Before running script:**
   ```bash
   mv ios_logs_archive ios_logs_backup
   mv android_logs_archive android_logs_backup
   ```

2. **Run your capture session**

3. **Restore if needed:**
   ```bash
   cp -r ios_logs_backup/* ios_logs_archive/
   cp -r android_logs_backup/* android_logs_archive/
   ```

## Configuration

To adjust limits, edit the respective script:

### iOS (`ios_extractor.py`)
```python
MAX_LOG_AGE_HOURS = 24      # Hours
MAX_TOTAL_SIZE_MB = 100     # Megabytes
```

### Android (`adb_extractor.py`)
```python
MAX_LOG_AGE_HOURS = 24      # Hours
MAX_TOTAL_SIZE_MB = 100     # Megabytes
```

**Note:** Changes require script restart to take effect.

## Monitoring Disk Usage

Check current archive sizes:

```bash
# iOS
du -sh ios_logs_archive/

# Android
du -sh android_logs_archive/

# Both
du -sh *_logs_archive/
```

List archived logs with details:

```bash
# iOS
ls -lht ios_logs_archive/

# Android
ls -lht android_logs_archive/
```

## Troubleshooting

### "Permission denied" when deleting
- Check file permissions
- Ensure no other process has files open
- Try running with appropriate permissions

### Cleanup not triggering
- Verify archive directory exists
- Check script has write permissions
- Review console output for cleanup messages

### Excessive disk usage
- Manually review archive directories
- Check for orphaned files outside archive dirs
- Verify cleanup thresholds are set correctly

## Integration with CI/CD

For automated testing environments:

```bash
# Before test run
rm -rf ios_logs_archive/* android_logs_archive/*

# Run tests with log capture
python3 ios_extractor.py &
iOS_PID=$!

# ... run tests ...

# Stop capture
kill -INT $iOS_PID

# Archive for CI artifacts
tar -czf ci-logs-${BUILD_NUMBER}.tar.gz \
  live_ios_log.log \
  ios_diagnostic_snapshot.log \
  ios_logs_archive/
```

## Best Practices

1. **Regular Cleanup:** Scripts handle this automatically
2. **Long-term Storage:** Archive important logs externally
3. **Disk Monitoring:** Set alerts at 80% disk usage
4. **Documentation:** Keep notes on which logs correspond to which issues
5. **Backup:** Critical debugging sessions should be backed up immediately

## Related Documentation

- [LOG_EXTRACTION_STANDARD.md](LOG_EXTRACTION_STANDARD.md) - Mandatory standard
- [QUICKSTART_IOS_LOGS.md](QUICKSTART_IOS_LOGS.md) - iOS quick start
- [ios_extractor.py](ios_extractor.py) - iOS implementation
- [adb_extractor.py](adb_extractor.py) - Android implementation

---

**Questions?** See [LOG_EXTRACTION_STANDARD.md](LOG_EXTRACTION_STANDARD.md) for full documentation.
