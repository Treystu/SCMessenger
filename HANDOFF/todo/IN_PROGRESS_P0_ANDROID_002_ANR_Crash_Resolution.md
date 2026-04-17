# IN_PROGRESS_P0_ANDROID_002: Android ANR Crash Resolution

## Status: 🔴 P0 BLOCKER - App Freezes Requiring Force-Kill
**Source:** MASTER_BUG_TRACKER.md (ANR-001, AUDIT-006), User crash reports

## Problem Statement
Android app experiences frequent ANRs (Application Not Responding):
- MeshForegroundService blocking main thread >20s
- Complete app freeze requiring force-kill
- ErrorId: d9404a9e-b3a8-4d8d-94b4-7fd53b1ded69
- User frustration and data loss risk

## Root Cause Analysis
Main thread blocking from:
- Network operations on UI thread
- Synchronous database operations  
- Coroutine scope mismanagement
- Heavy computation in foreground service

## Implementation Targets

### 1. Threading Audit & Fix (~400 LoC)
**Files:** `android/app/src/main/java/com/scmessenger/android/`
- Identify all main thread blocking operations
- Move network I/O to background threads
- Implement proper coroutine scopes

### 2. Foreground Service Optimization (~300 LoC)
**Files:** `MeshForegroundService.kt`, `MeshRepository.kt`
- Async operation patterns for service
- WorkManager integration for heavy tasks
- Priority-based task scheduling

### 3. ANR Detection & Recovery (~300 LoC)
**Files:** `ANRMonitor.kt`, `CrashRecovery.kt`
- ANR detection with watchdog timer
- Graceful service restart mechanisms
- User notification before force-kill scenarios

### 4. Performance Monitoring (~200 LoC)
**Files:** `PerformanceMonitor.kt`, analytics integration
- Real-time performance metrics
- ANR rate tracking and reporting
- Bottleneck identification tools

## Total Estimate: ~1,200 LoC

## Success Criteria
1. ✅ No more ANR events requiring force-kill
2. ✅ Main thread remains responsive during heavy operations
3. ✅ Foreground service handles load without blocking
4. ✅ Graceful recovery from near-ANR states
5. ✅ Performance monitoring with actionable metrics

## Priority: URGENT
Critical user experience issue causing app unusability.