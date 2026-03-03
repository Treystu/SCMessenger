package com.scmessenger.android.ui.chat

data class PendingDeliverySnapshot(
    val attemptCount: Int,
    val nextAttemptAtEpochSec: Long
)

enum class DeliveryStateSurface(val label: String, val detail: String) {
    PENDING(
        label = "pending",
        detail = "Queued locally. First route attempt is still in progress."
    ),
    STORED(
        label = "stored",
        detail = "Stored for retry. The recipient is currently offline or unreachable."
    ),
    FORWARDING(
        label = "forwarding",
        detail = "Actively retrying through direct or relay paths."
    ),
    DELIVERED(
        label = "delivered",
        detail = "Delivery receipt confirmed by the recipient node."
    )
}

data class DeliveryStatePresentation(
    val state: DeliveryStateSurface,
    val label: String,
    val detail: String
)

object DeliveryStateMapper {
    fun resolve(
        delivered: Boolean,
        pending: PendingDeliverySnapshot?,
        nowEpochSec: Long
    ): DeliveryStatePresentation {
        val state = when {
            delivered -> DeliveryStateSurface.DELIVERED
            pending == null -> DeliveryStateSurface.PENDING
            pending.nextAttemptAtEpochSec <= nowEpochSec -> DeliveryStateSurface.FORWARDING
            else -> DeliveryStateSurface.STORED
        }
        return DeliveryStatePresentation(
            state = state,
            label = state.label,
            detail = state.detail
        )
    }
}
