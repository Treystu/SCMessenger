# P0_ANDROID_003: BLE Scan Stabilization

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** Open
**Routing Tags:** [REQUIRES: TECH_DEBT] [REQUIRES: FINALIZATION]

## Objective
Stabilize BLE scanning operations that are failing due to Android scan quota limitations and restart loops. BLE failures break local peer discovery and transport fallback capabilities.

## Root Cause
From ANDROID_PIXEL_6A_AUDIT_2026-04-17:
- BLE scanner retry logic present but ineffective
- Android 12+ scan quota limitations (5 starts in 30s)
- Continuous scan restart loops causing `SCAN_FAILED_ALREADY_STARTED` errors
- Missing proper scan session management
- No exponential backoff implementation

## Implementation Plan

### 1. Scan Session Reuse and Management
**File:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`
```kotlin
class BleScanner @Inject constructor(
    private val bluetoothAdapter: BluetoothAdapter
) {
    private var currentScanSession: BluetoothLeScanner? = null
    private var isScanning = false
    private val scanLock = Mutex()
    
    suspend fun startScanning(): Boolean = scanLock.withLock {
        if (isScanning) {
            Timber.d("Scan already in progress, reusing existing session")
            return true // Reuse existing scan
        }
        
        if (currentScanSession == null) {
            currentScanSession = bluetoothAdapter.bluetoothLeScanner
        }
        
        return try {
            val settings = ScanSettings.Builder()
                .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
                .build()
            
            currentScanSession?.startScan(emptyList(), settings, scanCallback)
            isScanning = true
            Timber.d("BLE scanning started successfully")
            true
        } catch (e: Exception) {
            Timber.e("BLE scan start failed: ${e.message}")
            handleScanFailure(e)
            false
        }
    }
    
    suspend fun stopScanning() = scanLock.withLock {
        if (isScanning) {
            try {
                currentScanSession?.stopScan(scanCallback)
                isScanning = false
                Timber.d("BLE scanning stopped")
            } catch (e: Exception) {
                Timber.e("BLE scan stop failed: ${e.message}")
            }
        }
    }
}
```

### 2. Scan Quota Management
**File:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt` (NEW)
```kotlin
class BleQuotaManager {
    private val scanAttempts = mutableMapOf<String, Int>()
    private val lastScanTime = mutableMapOf<String, Long>()
    private val quotaLock = Mutex()
    
    suspend fun canStartScan(context: String): Boolean = quotaLock.withLock {
        val now = System.currentTimeMillis()
        val lastAttempt = lastScanTime[context] ?: 0
        val attempts = scanAttempts[context] ?: 0
        
        // Android quota: max 5 starts in 30 seconds
        if (now - lastAttempt < 30000 && attempts >= 5) {
            Timber.w("BLE scan quota exceeded for $context: $attempts attempts in 30s")
            return false
        }
        
        // Reset counter if outside 30s window
        if (now - lastAttempt >= 30000) {
            scanAttempts[context] = 0
        }
        
        scanAttempts[context] = attempts + 1
        lastScanTime[context] = now
        return true
    }
    
    fun recordScanSuccess(context: String) {
        // Successful scans don't count against quota
        lastScanTime[context] = System.currentTimeMillis()
    }
}
```

### 3. Exponential Backoff with Jitter
**File:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleBackoffStrategy.kt` (NEW)
```kotlin
class BleBackoffStrategy(
    private val initialDelayMs: Long = 1000,
    private val maxDelayMs: Long = 30000,
    private val multiplier: Double = 2.0
) {
    private var currentDelay = initialDelayMs
    private var attemptCount = 0
    
    fun nextDelay(): Long {
        attemptCount++
        
        // Exponential backoff with jitter
        val delay = minOf(
            (initialDelayMs * Math.pow(multiplier, (attemptCount - 1).toDouble())).toLong(),
            maxDelayMs
        )
        
        // Add jitter (±20%) to avoid synchronized retry storms
        val jitter = (delay * 0.2).toLong()
        val jitteredDelay = delay - jitter + (Random.nextLong() % (2 * jitter + 1))
        
        currentDelay = jitteredDelay
        return jitteredDelay
    }
    
    fun reset() {
        currentDelay = initialDelayMs
        attemptCount = 0
    }
    
    fun getCurrentDelay(): Long = currentDelay
}
```

### 4. Transport Health Monitoring
**File:** `android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt` (NEW)
```kotlin
class TransportHealthMonitor {
    private val transportHealth = mutableMapOf<String, TransportHealth>()
    
    fun recordTransportEvent(transport: String, success: Boolean, latency: Long? = null) {
        val health = transportHealth.getOrPut(transport) { TransportHealth() }
        
        if (success) {
            health.successCount++
            latency?.let { health.totalLatency += it }
        } else {
            health.failureCount++
        }
        
        health.lastUpdated = System.currentTimeMillis()
    }
    
    fun getTransportHealth(transport: String): TransportHealth {
        return transportHealth[transport] ?: TransportHealth()
    }
    
    fun shouldUseTransport(transport: String): Boolean {
        val health = getTransportHealth(transport)
        val totalAttempts = health.successCount + health.failureCount
        
        if (totalAttempts < 5) {
            return true // Not enough data, try anyway
        }
        
        val successRate = health.successCount.toDouble() / totalAttempts.toDouble()
        return successRate > 0.3 // Use if success rate > 30%
    }
}

data class TransportHealth(
    var successCount: Int = 0,
    var failureCount: Int = 0,
    var totalLatency: Long = 0,
    var lastUpdated: Long = System.currentTimeMillis()
)
```

### 5. Graceful Degradation
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun handleBleFailure() {
    Timber.w("BLE transport failing, initiating graceful degradation")
    
    // Reduce BLE usage
    reduceBleScanFrequency()
    
    // Prioritize other transports
 prioritizeWifiDirect()
    prioritizeRelayTransport()
    
    // Update UI to reflect degraded state
    updateTransportStatus("BLE degraded, using fallbacks")
    
    // Schedule BLE recovery attempt
    scheduleBleRecovery()
}
```

## Files to Modify/Create
1. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` - Scan session management
2. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt` (NEW) - Quota enforcement
3. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleBackoffStrategy.kt` (NEW) - Backoff strategy
4. `android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt` (NEW) - Health monitoring
5. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Graceful degradation
6. `android/app/src/main/java/com/scmessenger/android/utils/Random.kt` - Jitter utility

## Test Plan
1. **Quota Testing**: Force scan quota exhaustion and verify enforcement
2. **Backoff Verification**: Test exponential backoff with jitter
3. **Session Reuse**: Verify scan sessions are reused instead of restarted
4. **Graceful Degradation**: Test fallback to other transports when BLE fails
5. **Health Monitoring**: Verify transport health metrics are collected

## Success Criteria
- ✅ BLE scanning operates within Android quotas
- ✅ Scan sessions reused instead of restarted
- ✅ Exponential backoff with jitter implemented
- ✅ Graceful degradation when BLE fails
- ✅ Transport health monitoring working

## Priority: URGENT
BLE failures break local peer discovery and transport fallback, severely limiting app functionality.

**Estimated LOC:** ~250-300 LOC across 6 files
**Time Estimate:** 2-3 hours implementation + 1 hour testing