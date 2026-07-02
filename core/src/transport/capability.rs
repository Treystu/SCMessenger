// Transport capability checking
//
// Provides cross-platform parity for determining whether the transport layer
// can forward messages on behalf of thin clients (WASM, mobile). Core parity
// with the CLI `TransportBridge::can_forward_for_wasm()` method.

use crate::transport::health::TransportHealthMonitor;

/// Determine whether the local node can forward messages for WASM thin clients.
///
/// A node can forward for WASM if it has at least one active transport
/// capability — meaning it has at least one connected peer or an active relay
/// connection. This mirrors the CLI `TransportBridge::can_forward_for_wasm()`
/// check, which returns true when `cli_capabilities` is non-empty.
///
/// The WASM browser client connects to the CLI daemon via WebSocket and relies
/// on the daemon to relay messages over the mesh. This function tells the WASM
/// client whether the daemon is capable of doing so.
pub fn can_forward_for_wasm(monitor: &TransportHealthMonitor) -> bool {
    let metrics = monitor.get_global_metrics();
    // Can forward if we have any active connections or have sent messages
    metrics.current_active_connections > 0 || metrics.total_messages_sent > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_forward_for_wasm_returns_false_when_empty() {
        let monitor = TransportHealthMonitor::new();
        assert!(!can_forward_for_wasm(&monitor));
    }
}
