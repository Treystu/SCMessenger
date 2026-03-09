# BLE Connection Failure - Root Cause Analysis
**Date**: 2026-03-09 14:22 UTC  
**Status**: 🔴 CRITICAL BUG - DeadObjectException in GATT Server  
**Priority**: P0 - Blocking BLE Messaging

## Problem Statement

After successful messaging over cellular/WiFi, switching to Bluetooth-only mode fails with:
```
android.os.DeadObjectException
at android.bluetooth.BluetoothGattServer.notifyCharacteristicChanged
```

## Root Cause

### Issue 1: No Subscription Tracking ❌
**File**: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`

The GATT server tracks `connectedDevices` but **NOT** which devices have subscribed to notifications.

**Problem Flow**:
1. iOS central connects → added to `connectedDevices`
2. iOS central subscribes to MESSAGE characteristic → **NOT TRACKED**
3. Messages sent successfully via cellular
4. iOS central disconnects → removed from `connectedDevices`
5. iOS central reconnects → added to `connectedDevices` again
6. iOS central does **NOT** re-subscribe (thinks it's still subscribed)
7. Android tries to send notification → **DeadObjectException** (no subscription)

### Issue 2: Missing Descriptor Write Handler ❌
The GATT server has **NO** `onDescriptorWriteRequest` handler. This means:
- Client subscriptions are never acknowledged
- Server doesn't know which clients want notifications
- Server sends to all "connected" devices, not just "subscribed" devices

### Issue 3: Stale Connection Tracking ❌
Connections show as `connected=1` but the actual GATT connection is dead:
```
connected=5F:87:A7:15:B0:A6  # Map says connected
server_sendData_false        # But sending fails
DeadObjectException          # Connection is actually dead
```

## Evidence from Logs

### Android GATT Server Logs
```
E BleGattServer: Failed to send data to 5F:87:A7:15:B0:A6
E BleGattServer: java.lang.RuntimeException: android.os.DeadObjectException
E BleGattServer: at android.bluetooth.BluetoothGattServer.notifyCharacteristicChanged
E BleGattServer: Caused by: android.os.DeadObjectException
E BleGattServer: at android.os.BinderProxy.transactNative(Native Method)
E BleGattServer: at android.bluetooth.IBluetoothGatt$Stub$Proxy.sendNotification
```

### Delivery Failures
```
delivery_attempt medium=ble outcome=failed reason=server_sendData_false:5F:87:A7:15:B0:A6 connected=1
```

Device shows as connected but send fails because subscription is missing.

### Connection Timeouts
```
W BLE connection timeout for 5F:87:A7:15:B0:A6 (stuck in CONNECTING), disconnecting
```

Connection attempts get stuck because old state isn't cleaned up.

## Solution

### Fix 1: Add Subscription Tracking
Track which devices have actually subscribed to notifications:

```kotlin
// Add to BleGattServer class
private val subscribedDevices = ConcurrentHashMap<String, MutableSet<UUID>>()

override fun onDescriptorWriteRequest(
    device: BluetoothDevice,
    requestId: Int,
    descriptor: BluetoothGattDescriptor,
    preparedWrite: Boolean,
    responseNeeded: Boolean,
    offset: Int,
    value: ByteArray
) {
    val characteristic = descriptor.characteristic
    
    if (descriptor.uuid == CLIENT_CHARACTERISTIC_CONFIG_UUID) {
        val isSubscribing = value.contentEquals(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
        
        if (isSubscribing) {
            // Track subscription
            subscribedDevices.getOrPut(device.address) { mutableSetOf() }
                .add(characteristic.uuid)
            Timber.d("Device ${device.address} subscribed to ${characteristic.uuid}")
        } else {
            // Remove subscription
            subscribedDevices[device.address]?.remove(characteristic.uuid)
            Timber.d("Device ${device.address} unsubscribed from ${characteristic.uuid}")
        }
        
        if (responseNeeded) {
            gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, 0, value)
        }
    }
}
```

### Fix 2: Clean Subscriptions on Disconnect
```kotlin
override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
    when (newState) {
        BluetoothProfile.STATE_DISCONNECTED -> {
            connectedDevices.remove(device.address)
            subscribedDevices.remove(device.address)  // ✅ ADD THIS
            reassemblyBuffers.remove(device.address)
            expectedFragments.remove(device.address)
            Timber.d("GATT client disconnected: ${device.address}")
        }
    }
}
```

### Fix 3: Only Send to Subscribed Devices
```kotlin
fun sendData(device: BluetoothDevice, data: ByteArray): Boolean {
    // Check if device is subscribed, not just connected
    val isSubscribed = subscribedDevices[device.address]?.contains(MESSAGE_CHAR_UUID) == true
    if (!isSubscribed) {
        Timber.w("Device ${device.address} not subscribed to MESSAGE characteristic")
        return false
    }
    
    // Wrap in try-catch to handle DeadObjectException
    return try {
        sendFragmented(device, data, MESSAGE_CHAR_UUID)
    } catch (e: DeadObjectException) {
        Timber.e("GATT connection dead for ${device.address}, cleaning up")
        // Force cleanup
        connectedDevices.remove(device.address)
        subscribedDevices.remove(device.address)
        false
    }
}
```

### Fix 4: Add Companion Object Definition
```kotlin
companion object {
    // ... existing UUIDs ...
    
    // Client Characteristic Configuration Descriptor
    val CLIENT_CHARACTERISTIC_CONFIG_UUID: UUID = 
        UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
}
```

## Expected Behavior After Fix

### Connection Flow
1. iOS central connects → `connectedDevices` updated
2. iOS central subscribes → `subscribedDevices` updated
3. Messages can now be sent successfully
4. iOS central disconnects → **BOTH** maps cleaned
5. iOS central reconnects → added to `connectedDevices`
6. iOS central re-subscribes → added to `subscribedDevices`
7. Messages flow correctly

### Error Handling
- `DeadObjectException` caught and connection cleaned up
- Stale connections automatically removed
- Delivery failures logged with correct reason

## Testing Plan

1. **Test BLE-only messaging**
   - Turn off WiFi/cellular on both devices
   - Send messages via BLE
   - Verify delivery

2. **Test reconnection**
   - Connect via BLE
   - Send message
   - Disconnect Bluetooth
   - Reconnect Bluetooth
   - Send message again

3. **Test network switching**
   - Send via cellular
   - Disable cellular
   - Send via BLE
   - Verify no DeadObjectException

## Files to Modify

1. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`
   - Add `subscribedDevices` map
   - Add `onDescriptorWriteRequest` handler
   - Update `onConnectionStateChange` to clean subscriptions
   - Add DeadObjectException handling in `sendData`
   - Add CLIENT_CHARACTERISTIC_CONFIG_UUID constant

## Impact

- **Critical**: BLE messaging completely broken after network switch
- **User Experience**: Messages appear stuck, no feedback
- **Workaround**: Restart app to clear stale connection state

## Priority

**P0 - Must fix before release**

BLE is a core fallback mechanism when cellular/WiFi unavailable. This bug makes BLE unreliable and breaks the app's offline-first promise.

---

**Next**: Implement fix and test BLE reconnection flow.
