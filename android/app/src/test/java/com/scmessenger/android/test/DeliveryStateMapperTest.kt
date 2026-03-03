package com.scmessenger.android.test

import com.scmessenger.android.ui.chat.DeliveryStateMapper
import com.scmessenger.android.ui.chat.DeliveryStateSurface
import com.scmessenger.android.ui.chat.PendingDeliverySnapshot
import org.junit.Assert.assertEquals
import org.junit.Test

class DeliveryStateMapperTest {

    @Test
    fun `maps delivered flag to delivered state`() {
        val state = DeliveryStateMapper.resolve(
            delivered = true,
            pending = PendingDeliverySnapshot(attemptCount = 3, nextAttemptAtEpochSec = 1000),
            nowEpochSec = 1000
        )

        assertEquals(DeliveryStateSurface.DELIVERED, state.state)
        assertEquals("delivered", state.label)
    }

    @Test
    fun `maps missing pending snapshot to pending state`() {
        val state = DeliveryStateMapper.resolve(
            delivered = false,
            pending = null,
            nowEpochSec = 1000
        )

        assertEquals(DeliveryStateSurface.PENDING, state.state)
        assertEquals("pending", state.label)
    }

    @Test
    fun `maps future retry time to stored state`() {
        val state = DeliveryStateMapper.resolve(
            delivered = false,
            pending = PendingDeliverySnapshot(attemptCount = 2, nextAttemptAtEpochSec = 3000),
            nowEpochSec = 1000
        )

        assertEquals(DeliveryStateSurface.STORED, state.state)
        assertEquals("stored", state.label)
    }

    @Test
    fun `maps due retry time to forwarding state`() {
        val state = DeliveryStateMapper.resolve(
            delivered = false,
            pending = PendingDeliverySnapshot(attemptCount = 2, nextAttemptAtEpochSec = 1000),
            nowEpochSec = 1000
        )

        assertEquals(DeliveryStateSurface.FORWARDING, state.state)
        assertEquals("forwarding", state.label)
    }
}
