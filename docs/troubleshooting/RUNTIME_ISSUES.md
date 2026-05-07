# Runtime Issues Troubleshooting Guide

Status: Active  
Last updated: 2026-03-07

This guide covers common runtime issues and their solutions across all SCMessenger platforms.

## Table of Contents

- [General Runtime Issues](#general-runtime-issues)
- [Connection Issues](#connection-issues)
- [Message Delivery Issues](#message-delivery-issues)
- [Performance Issues](#performance-issues)
- [Crash Issues](#crash-issues)
- [Platform-Specific Runtime Issues](#platform-specific-runtime-issues)
- [Debugging Runtime Issues](#debugging-runtime-issues)

## General Runtime Issues

### Issue: "Application won't start"

**Symptoms:**
- App crashes immediately on launch
- No error message displayed

**Solution:**
1. Check logs for error messages
2. Verify all dependencies are installed
3. Clear application data/cache
4. Reinstall application

**Platform-specific:**
```bash
# Android
adb logcat | grep SCMessenger

# iOS
xcrun simctl spawn booted log stream --predicate 'processImagePath contains "SCMessenger"'

# CLI
RUST_LOG=debug ./scmessenger-cli start
```

### Issue: "Identity not found"

**Symptoms:**
```
Error: No identity found. Please create an identity first.
```

**Solution:**
```bash
# CLI
./scmessenger-cli identity create

# Or restore from backup
./scmessenger-cli identity import --file backup.json
```

### Issue: "Database corruption"

**Symptoms:**
```
Error: Database is corrupted or incompatible
```

**Solution:**
```bash
# Backup existing data
cp -r ~/.scmessenger ~/.scmessenger.backup

# Clear database
rm -rf ~/.scmessenger/db

# Restart application (will recreate database)
./scmessenger-cli start
```

## Connection Issues

### Issue: "No peers found"

**Symptoms:**
- Peer list is empty
- Cannot discover other nodes

**Solution:**
1. Check network connectivity
2. Verify firewall settings
3. Ensure bootstrap nodes are reachable
4. Check transport configuration

```bash
# Test network connectivity
ping 8.8.8.8

# Check bootstrap nodes
./scmessenger-cli peers bootstrap

# Enable all transports
RUST_LOG=debug ./scmessenger-cli start
```

### Issue: "Cannot connect to peer"

**Symptoms:**
```
Error: Failed to connect to peer <peer-id>
```

**Solution:**
1. Verify peer is online
2. Check NAT traversal configuration
3. Try different transport (BLE, WiFi, Internet)
4. Check firewall rules

```bash
# Check peer status
./scmessenger-cli peers status <peer-id>

# Test connectivity
./scmessenger-cli peers ping <peer-id>

# Enable verbose logging
RUST_LOG=scmessenger_core::transport=debug ./scmessenger-cli start
```

### Issue: "Relay connection failed"

**Symptoms:**
```
Error: Failed to establish relay circuit
```

**Solution:**
1. Check relay node availability
2. Verify relay configuration
3. Check network connectivity
4. Try alternative relay nodes

```bash
# List relay nodes
./scmessenger-cli relay list

# Test relay connectivity
./scmessenger-cli relay test <relay-id>

# Enable relay logging
RUST_LOG=scmessenger_core::relay=debug ./scmessenger-cli start
```

### Issue: "BLE not working"

**Symptoms:**
- BLE peers not discovered
- BLE connection fails

**Solution:**
1. Verify Bluetooth is enabled
2. Check Bluetooth permissions
3. Ensure BLE is supported
4. Restart Bluetooth service

**Android:**
```bash
# Check Bluetooth status
adb shell dumpsys bluetooth_manager

# Grant Bluetooth permissions
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_CONNECT
```

**iOS:**
- Settings → Privacy → Bluetooth → SCMessenger → Enable

## Message Delivery Issues

### Issue: "Messages not sending"

**Symptoms:**
- Messages stuck in "Sending" state
- Error: "Failed to send message"

**Solution:**
1. Check network connectivity
2. Verify peer is online
3. Check message queue
4. Retry sending

```bash
# Check outbox
./scmessenger-cli messages outbox

# Retry failed messages
./scmessenger-cli messages retry

# Clear stuck messages
./scmessenger-cli messages clear-outbox
```

### Issue: "Messages not received"

**Symptoms:**
- Peer sent message but not received
- Message delivery delayed

**Solution:**
1. Check inbox
2. Verify peer connection
3. Check message sync status
4. Force sync

```bash
# Check inbox
./scmessenger-cli messages inbox

# Force sync
./scmessenger-cli sync

# Check sync status
./scmessenger-cli sync status
```

### Issue: "Duplicate messages"

**Symptoms:**
- Same message appears multiple times
- Message ID collision

**Solution:**
1. Check message deduplication
2. Verify message IDs are unique
3. Clear duplicate messages

```bash
# Check for duplicates
./scmessenger-cli messages check-duplicates

# Remove duplicates
./scmessenger-cli messages deduplicate
```

### Issue: "Message delivery receipts not working"

**Symptoms:**
- No delivery confirmation
- Receipt status stuck

**Solution:**
1. Verify receipt protocol is enabled
2. Check peer supports receipts
3. Check network connectivity

```bash
# Check receipt status
./scmessenger-cli messages receipts <message-id>

# Enable receipt logging
RUST_LOG=scmessenger_core::message::receipts=debug ./scmessenger-cli start
```

## Performance Issues

### Issue: "High CPU usage"

**Symptoms:**
- CPU usage consistently high
- Device overheating

**Solution:**
1. Check for infinite loops in logs
2. Reduce peer discovery frequency
3. Limit concurrent connections
4. Optimize transport selection

```bash
# Check CPU usage
top -p $(pgrep scmessenger-cli)

# Reduce discovery frequency
./scmessenger-cli config set discovery.interval 60

# Limit connections
./scmessenger-cli config set max_connections 10
```

### Issue: "High memory usage"

**Symptoms:**
- Memory usage growing over time
- Out of memory errors

**Solution:**
1. Check for memory leaks
2. Clear message cache
3. Reduce cache size
4. Restart application

```bash
# Check memory usage
ps aux | grep scmessenger-cli

# Clear cache
./scmessenger-cli cache clear

# Reduce cache size
./scmessenger-cli config set cache.max_size 100MB
```

### Issue: "Slow message delivery"

**Symptoms:**
- Messages take long time to deliver
- High latency

**Solution:**
1. Check network latency
2. Optimize routing
3. Use direct connections when possible
4. Check relay performance

```bash
# Measure latency
./scmessenger-cli peers ping <peer-id>

# Check routing table
./scmessenger-cli routing table

# Force direct connection
./scmessenger-cli peers connect-direct <peer-id>
```

### Issue: "Battery drain (mobile)"

**Symptoms:**
- Rapid battery consumption
- Device warm/hot

**Solution:**
1. Reduce background activity
2. Optimize transport usage
3. Adjust sync frequency
4. Enable battery optimization

**Android:**
```bash
# Check battery usage
adb shell dumpsys batterystats | grep SCMessenger

# Enable battery optimization
# Settings → Apps → SCMessenger → Battery → Optimize battery usage
```

**iOS:**
- Settings → Battery → Battery Health → Optimize Battery Charging

## Crash Issues

### Issue: "Application crashes on startup"

**Symptoms:**
- App crashes immediately after launch
- No UI displayed

**Solution:**
1. Check crash logs
2. Verify dependencies
3. Clear app data
4. Reinstall app

**Android:**
```bash
# Get crash logs
adb logcat -b crash

# Clear app data
adb shell pm clear com.scmessenger.android

# Reinstall
adb uninstall com.scmessenger.android
adb install app-debug.apk
```

**iOS:**
```bash
# Get crash logs
xcrun simctl spawn booted log show --predicate 'processImagePath contains "SCMessenger"' --last 1h

# Reset app
xcrun simctl uninstall booted com.scmessenger.ios
xcrun simctl install booted SCMessenger.app
```

### Issue: "Panic: thread panicked"

**Symptoms:**
```
thread 'main' panicked at 'assertion failed: ...'
```

**Solution:**
1. Note panic message and location
2. Check for known issues
3. Report bug with backtrace

```bash
# Get full backtrace
RUST_BACKTRACE=full ./scmessenger-cli start

# Save backtrace to file
RUST_BACKTRACE=full ./scmessenger-cli start 2> crash.log
```

### Issue: "Segmentation fault"

**Symptoms:**
```
Segmentation fault (core dumped)
```

**Solution:**
1. Enable core dumps
2. Analyze with debugger
3. Report bug with core dump

```bash
# Enable core dumps
ulimit -c unlimited

# Run and capture core dump
./scmessenger-cli start

# Analyze with gdb
gdb ./scmessenger-cli core
```

## Platform-Specific Runtime Issues

### Android Runtime Issues

**Issue: "App killed by system"**

**Symptoms:**
- App stops running in background
- No crash log

**Solution:**
1. Disable battery optimization
2. Add to protected apps list
3. Enable autostart

```bash
# Check if app is optimized
adb shell dumpsys deviceidle whitelist | grep scmessenger

# Disable optimization
# Settings → Apps → SCMessenger → Battery → Don't optimize
```

**Issue: "BLE permissions denied"**

**Symptoms:**
```
Error: Bluetooth permission denied
```

**Solution:**
```bash
# Grant permissions
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_CONNECT
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_SCAN
adb shell pm grant com.scmessenger.android android.permission.ACCESS_FINE_LOCATION
```

### iOS Runtime Issues

**Issue: "App suspended in background"**

**Symptoms:**
- App stops working when in background
- No background activity

**Solution:**
1. Enable background modes in Xcode
2. Use background tasks API
3. Check background app refresh settings

**Issue: "Network extension not working"**

**Symptoms:**
```
Error: Network extension failed to start
```

**Solution:**
1. Check entitlements
2. Verify provisioning profile
3. Enable network extension capability

### CLI Runtime Issues

**Issue: "Port already in use"**

**Symptoms:**
```
Error: Address already in use (os error 98)
```

**Solution:**
```bash
# Find process using port
lsof -i :9002

# Kill process
kill -9 <pid>

# Or use different port
./scmessenger-cli start --port 9003
```

**Issue: "Permission denied (bind)"**

**Symptoms:**
```
Error: Permission denied (os error 13)
```

**Solution:**
```bash
# Use port > 1024 (no root required)
./scmessenger-cli start --port 9002

# Or run with sudo (not recommended)
sudo ./scmessenger-cli start --port 80
```

## Debugging Runtime Issues

### Enable Debug Logging

```bash
# CLI
RUST_LOG=debug ./scmessenger-cli start

# Specific modules
RUST_LOG=scmessenger_core::transport=debug,scmessenger_core::relay=info ./scmessenger-cli start

# Trace level (very verbose)
RUST_LOG=trace ./scmessenger-cli start
```

### Capture Logs

**Android:**
```bash
# Capture logs to file
adb logcat -d > android.log

# Filter by tag
adb logcat -s SCMessenger > android.log

# Clear and capture
adb logcat -c && adb logcat > android.log
```

**iOS:**
```bash
# Capture logs
xcrun simctl spawn booted log stream --predicate 'processImagePath contains "SCMessenger"' > ios.log

# Or use Console.app on macOS
```

**CLI:**
```bash
# Redirect to file
RUST_LOG=debug ./scmessenger-cli start 2>&1 | tee cli.log
```

### Use Diagnostic Tools

```bash
# Check system resources
top
htop
vmstat

# Check network
netstat -an | grep 9002
ss -tulpn | grep scmessenger

# Check disk space
df -h

# Check file descriptors
lsof -p $(pgrep scmessenger-cli)
```

### Profiling

```bash
# CPU profiling (Linux)
perf record -g ./scmessenger-cli start
perf report

# Memory profiling
valgrind --leak-check=full ./scmessenger-cli start

# Heap profiling
heaptrack ./scmessenger-cli start
```

## Getting Help

If runtime issues persist:

1. **Collect logs**: Capture detailed logs with debug logging enabled
2. **Reproduce**: Document steps to reproduce the issue
3. **Search issues**: https://github.com/Treystu/SCMessenger/issues
4. **Report bug**: Create new issue with logs and reproduction steps
5. **Ask for help**: https://github.com/Treystu/SCMessenger/discussions

### Bug Report Template

```markdown
## Description
[Clear description of the issue]

## Platform
- OS: [e.g., Ubuntu 22.04, macOS 13, Android 13]
- Version: [e.g., v0.2.1]
- Architecture: [e.g., x86_64, aarch64]

## Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Observe issue]

## Expected Behavior
[What you expected to happen]

## Actual Behavior
[What actually happened]

## Logs
```
[Paste relevant logs here]
```

## Additional Context
[Any other relevant information]
```

---

**Related Guides:**
- [Build Issues Guide](BUILD_ISSUES.md)
- [CI Failures Guide](CI_FAILURES.md)
- [Testing Guide](../TESTING_GUIDE.md)
- [Architecture Overview](../ARCHITECTURE.md)
