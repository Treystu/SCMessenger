// Network diagnostics reporting
//
// Provides a cross-platform parity type for network diagnostic information.
// Core parity with the Android `DiagnosticsReporter.NetworkDiagnosticsReport`
// and iOS equivalent. Aggregates data from TransportHealthMonitor into a
// single report.

use crate::transport::health::{ConnectionState, ConnectionStats, TransportHealthMonitor};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// Summary of network diagnostics for the mesh node.
///
/// Aggregates connection statistics, transport metrics, and relay health into a
/// single reportable structure. Used by platform layers (Android, iOS, WASM)
/// to surface network state to users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnosticsReport {
    pub connected_peer_count: usize,
    pub total_messages_sent: u64,
    pub total_messages_failed: u64,
    pub avg_latency_ms: u64,
    pub active_connections: u32,
    pub connection_summary: Vec<PeerConnectionSummary>,
    pub generated_at_ms: u64,
}

/// Per-peer connection summary included in diagnostics reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionSummary {
    pub peer_id: String,
    pub state: String,
    pub messages_sent: u64,
    pub messages_failed: u64,
    pub avg_latency_ms: u64,
    pub last_activity_ms: u64,
}

/// Build a network diagnostics report from the transport health monitor.
///
/// Aggregates all peer connection statistics and message metrics into a single
/// report. This is the core parity of the Android
/// `SettingsViewModel.getNetworkDiagnosticsReport()` method.
pub fn get_network_diagnostics_report(
    monitor: &TransportHealthMonitor,
) -> NetworkDiagnosticsReport {
    let metrics = monitor.get_global_metrics();
    let peers = monitor.get_all_connection_stats();

    let connection_summary: Vec<PeerConnectionSummary> = peers
        .values()
        .map(|stats| PeerConnectionSummary {
            peer_id: stats.peer_id.to_string(),
            state: format!("{:?}", stats.state),
            messages_sent: stats.messages_sent,
            messages_failed: stats.message_failures,
            avg_latency_ms: stats.avg_latency_ms,
            last_activity_ms: stats.last_activity,
        })
        .collect();

    let connected_count = peers
        .values()
        .filter(|p| p.state == ConnectionState::Connected)
        .count();

    NetworkDiagnosticsReport {
        connected_peer_count: connected_count,
        total_messages_sent: metrics.total_messages_sent,
        total_messages_failed: metrics.total_message_failures,
        avg_latency_ms: compute_avg_latency(&peers),
        active_connections: metrics.current_active_connections,
        connection_summary,
        generated_at_ms: now_ms(),
    }
}

fn compute_avg_latency(peers: &std::collections::HashMap<PeerId, ConnectionStats>) -> u64 {
    let latencies: Vec<u64> = peers
        .values()
        .filter(|p| p.state == ConnectionState::Connected)
        .map(|p| p.avg_latency_ms)
        .filter(|l| *l > 0)
        .collect();
    if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    }
}

fn now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_diagnostics_report_serializes() {
        let report = NetworkDiagnosticsReport {
            connected_peer_count: 3,
            total_messages_sent: 100,
            total_messages_failed: 2,
            avg_latency_ms: 50,
            active_connections: 3,
            connection_summary: vec![],
            generated_at_ms: 1000,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("connected_peer_count"));
        assert!(json.contains("total_messages_sent"));
    }

    #[test]
    fn peer_connection_summary_formats_state() {
        let summary = PeerConnectionSummary {
            peer_id: "peer-1".to_string(),
            state: "Connected".to_string(),
            messages_sent: 10,
            messages_failed: 1,
            avg_latency_ms: 30,
            last_activity_ms: 500,
        };
        assert_eq!(summary.peer_id, "peer-1");
    }

    #[test]
    fn get_network_diagnostics_report_from_empty_monitor() {
        let monitor = TransportHealthMonitor::new();
        let report = get_network_diagnostics_report(&monitor);
        assert_eq!(report.connected_peer_count, 0);
        assert_eq!(report.total_messages_sent, 0);
        assert!(report.connection_summary.is_empty());
    }
}
