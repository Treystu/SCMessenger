package com.scmessenger.android.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import com.scmessenger.android.data.MeshRepository
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlin.system.measureTimeMillis
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Regression test for P0-ANDROID-ANR: registerBatteryMonitor()'s onReceive must
 * dispatch the (potentially slow) FFI-backed device-state update to a background
 * dispatcher instead of running it synchronously on the calling (main) thread.
 */
class AndroidPlatformBridgeTest {

    @Test
    fun `battery broadcast onReceive returns promptly despite slow FFI call`() {
        val context = mockk<Context>(relaxed = true)
        val meshRepository = mockk<MeshRepository>(relaxed = true)

        // Simulate the real ANR cause: a slow synchronous FFI call into the Rust core.
        every { meshRepository.updateDeviceState(any()) } answers {
            Thread.sleep(300)
        }

        val receiverSlot = slot<BroadcastReceiver>()
        every { context.registerReceiver(capture(receiverSlot), any()) } returns null

        val bridge = AndroidPlatformBridge(context, meshRepository)

        val registerBatteryMonitor =
            AndroidPlatformBridge::class.java.getDeclaredMethod("registerBatteryMonitor")
        registerBatteryMonitor.isAccessible = true
        registerBatteryMonitor.invoke(bridge)

        assertTrue("battery receiver was not registered", receiverSlot.isCaptured)

        // Now that the real receiver is registered, stub the sticky-intent query that
        // updateBatteryState() performs (context.registerReceiver(null, filter)).
        val stickyIntent = mockk<Intent>(relaxed = true)
        every { stickyIntent.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) } returns 80
        every { stickyIntent.getIntExtra(BatteryManager.EXTRA_SCALE, -1) } returns 100
        every { stickyIntent.getIntExtra(BatteryManager.EXTRA_STATUS, -1) } returns
            BatteryManager.BATTERY_STATUS_CHARGING
        every { context.registerReceiver(isNull(), any<IntentFilter>()) } returns stickyIntent

        val incomingIntent = mockk<Intent>(relaxed = true)
        val elapsedMs = measureTimeMillis {
            receiverSlot.captured.onReceive(context, incomingIntent)
        }

        assertTrue(
            "onReceive blocked the calling thread for ${elapsedMs}ms -- it must dispatch " +
                "the device-state update to a background dispatcher instead of running it inline",
            elapsedMs < 250
        )
    }
}
