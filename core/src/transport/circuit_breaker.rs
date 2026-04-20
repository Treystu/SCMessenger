// Circuit Breaker Pattern for Relay Connectivity
//
// Prevents hammering failed relays by tracking failure states and
// implementing half-open recovery probes. Three states:
// - Closed: relay is healthy, requests flow freely
// - Open: relay has failed too many times, requests are rejected
// - HalfOpen: probing relay to see if it has recovered

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use web_time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Relay is healthy — requests pass through
    Closed,
    /// Relay has failed too many times — requests are rejected
    Open,
    /// Probing relay to check if it has recovered
    HalfOpen,
}

/// Configuration for a circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Duration to keep the circuit open before attempting half-open probe
    pub open_timeout: Duration,
    /// Duration to wait in half-open before fully closing
    pub half_open_timeout: Duration,
    /// Number of successful probes in half-open to close the circuit
    pub success_threshold: u32,
    /// Maximum number of half-open probe attempts before going back to Open
    pub max_half_open_probes: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            open_timeout: Duration::from_secs(300), // 5 minutes
            half_open_timeout: Duration::from_secs(30),
            success_threshold: 2,
            max_half_open_probes: 3,
        }
    }
}

/// Tracks the state of a single relay's circuit breaker
#[derive(Debug)]
struct CircuitBreakerEntry {
    /// Current circuit state
    state: CircuitState,
    /// Consecutive failure count
    failure_count: u32,
    /// Consecutive success count (used in half-open)
    success_count: u32,
    /// Number of half-open probe attempts
    half_open_probes: u32,
    /// Timestamp when the circuit was opened
    opened_at: Option<SystemTime>,
    /// Timestamp when half-open probing started
    half_open_started_at: Option<SystemTime>,
    /// Last failure reason
    last_failure_reason: Option<String>,
    /// Last success timestamp
    last_success_at: Option<SystemTime>,
}

impl CircuitBreakerEntry {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            half_open_probes: 0,
            opened_at: None,
            half_open_started_at: None,
            last_failure_reason: None,
            last_success_at: None,
        }
    }
}

/// Circuit breaker manager for relay connections
///
/// Each relay address gets its own circuit breaker instance.
/// The manager provides a unified interface to check and update
/// circuit states across all known relays.
pub struct CircuitBreakerManager {
    config: CircuitBreakerConfig,
    entries: Arc<RwLock<HashMap<String, CircuitBreakerEntry>>>,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager with the given config
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if a request to the given relay address should be allowed
    ///
    /// Returns `true` if the circuit is Closed or HalfOpen (probing allowed),
    /// and `false` if the circuit is Open (requests should be rejected).
    pub fn allow_request(&self, relay_address: &str) -> bool {
        let entries = self.entries.read();
        if let Some(entry) = entries.get(relay_address) {
            match entry.state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    // Check if we should transition to half-open
                    if let Some(opened_at) = entry.opened_at {
                        if opened_at.elapsed().unwrap_or_default() >= self.config.open_timeout {
                            // Time to probe — drop the read lock and transition
                            drop(entries);
                            self.transition_to_half_open(relay_address);
                            return true;
                        }
                    }
                    debug!("Circuit breaker OPEN for relay {}, skipping attempt", relay_address);
                    false
                }
                CircuitState::HalfOpen => {
                    // Allow limited probes in half-open state
                    if entry.half_open_probes < self.config.max_half_open_probes {
                        true
                    } else {
                        debug!(
                            "Circuit breaker HALF-OPEN max probes reached for relay {}",
                            relay_address
                        );
                        false
                    }
                }
            }
        } else {
            true // No entry means circuit is implicitly closed
        }
    }

    /// Record a successful connection to a relay
    ///
    /// In HalfOpen state, accumulates successes toward closing the circuit.
    /// In Closed state, resets the failure counter.
    pub fn record_success(&self, relay_address: &str) {
        let mut entries = self.entries.write();
        let entry = entries.entry(relay_address.to_string()).or_insert_with(CircuitBreakerEntry::new);

        entry.last_success_at = Some(SystemTime::now());
        entry.last_failure_reason = None;

        match entry.state {
            CircuitState::Closed => {
                entry.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                entry.success_count += 1;
                if entry.success_count >= self.config.success_threshold {
                    info!("Circuit breaker CLOSING for relay {} after {} successful probes",
                          relay_address, entry.success_count);
                    entry.state = CircuitState::Closed;
                    entry.failure_count = 0;
                    entry.success_count = 0;
                    entry.half_open_probes = 0;
                    entry.opened_at = None;
                    entry.half_open_started_at = None;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                debug!("Unexpected success on OPEN circuit for relay {}", relay_address);
            }
        }
    }

    /// Record a failed connection to a relay
    ///
    /// In Closed state, accumulates failures toward opening the circuit.
    /// In HalfOpen state, reopens the circuit immediately.
    pub fn record_failure(&self, relay_address: &str, reason: &str) {
        let mut entries = self.entries.write();
        let entry = entries.entry(relay_address.to_string()).or_insert_with(CircuitBreakerEntry::new);

        entry.failure_count += 1;
        entry.last_failure_reason = Some(reason.to_string());

        match entry.state {
            CircuitState::Closed => {
                if entry.failure_count >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker OPENING for relay {} after {} failures: {}",
                        relay_address, entry.failure_count, reason
                    );
                    entry.state = CircuitState::Open;
                    entry.opened_at = Some(SystemTime::now());
                    entry.success_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                warn!(
                    "Circuit breaker RE-OPENING for relay {} after failed probe: {}",
                    relay_address, reason
                );
                entry.state = CircuitState::Open;
                entry.opened_at = Some(SystemTime::now());
                entry.success_count = 0;
                entry.half_open_probes = 0;
                entry.half_open_started_at = None;
            }
            CircuitState::Open => {
                // Already open, just update the failure reason
                entry.opened_at = Some(SystemTime::now());
            }
        }
    }

    /// Get the current circuit state for a relay
    pub fn get_state(&self, relay_address: &str) -> CircuitState {
        let entries = self.entries.read();
        entries.get(relay_address).map(|e| e.state).unwrap_or(CircuitState::Closed)
    }

    /// Get the failure count for a relay
    pub fn get_failure_count(&self, relay_address: &str) -> u32 {
        let entries = self.entries.read();
        entries.get(relay_address).map(|e| e.failure_count).unwrap_or(0)
    }

    /// Get the last failure reason for a relay
    pub fn get_last_failure_reason(&self, relay_address: &str) -> Option<String> {
        let entries = self.entries.read();
        entries.get(relay_address).and_then(|e| e.last_failure_reason.clone())
    }

    /// Reset the circuit breaker for a specific relay
    pub fn reset(&self, relay_address: &str) {
        let mut entries = self.entries.write();
        entries.remove(relay_address);
    }

    /// Reset all circuit breakers (e.g., on network change)
    pub fn reset_all(&self) {
        let mut entries = self.entries.write();
        entries.clear();
        info!("All circuit breakers reset");
    }

    /// Get all relay addresses currently in an open state
    pub fn get_open_circuits(&self) -> Vec<String> {
        let entries = self.entries.read();
        entries.iter()
            .filter(|(_, e)| e.state == CircuitState::Open)
            .map(|(addr, _)| addr.clone())
            .collect()
    }

    /// Get all relay addresses with healthy (closed) circuits
    pub fn get_healthy_relays(&self) -> Vec<String> {
        let entries = self.entries.read();
        entries.iter()
            .filter(|(_, e)| e.state == CircuitState::Closed)
            .map(|(addr, _)| addr.clone())
            .collect()
    }

    /// Get the number of relays in each circuit state
    pub fn get_stats(&self) -> CircuitBreakerStats {
        let entries = self.entries.read();
        let mut stats = CircuitBreakerStats::default();
        for entry in entries.values() {
            match entry.state {
                CircuitState::Closed => stats.closed_count += 1,
                CircuitState::Open => stats.open_count += 1,
                CircuitState::HalfOpen => stats.half_open_count += 1,
            }
        }
        stats.total = entries.len();
        stats
    }

    /// Transition a circuit to half-open state
    fn transition_to_half_open(&self, relay_address: &str) {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(relay_address) {
            info!("Circuit breaker transitioning to HALF-OPEN for relay {}", relay_address);
            entry.state = CircuitState::HalfOpen;
            entry.half_open_started_at = Some(SystemTime::now());
            entry.half_open_probes += 1;
            entry.success_count = 0;
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerStats {
    pub total: usize,
    pub closed_count: usize,
    pub open_count: usize,
    pub half_open_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_default_config() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.open_timeout, Duration::from_secs(300));
        assert_eq!(config.half_open_timeout, Duration::from_secs(30));
        assert_eq!(config.success_threshold, 2);
    }

    #[test]
    fn test_circuit_starts_closed() {
        let mgr = CircuitBreakerManager::with_defaults();
        assert_eq!(mgr.get_state("relay1.example.com"), CircuitState::Closed);
        assert!(mgr.allow_request("relay1.example.com"));
    }

    #[test]
    fn test_circuit_opens_after_failures() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        // Record failures up to threshold
        for i in 0..3 {
            mgr.record_failure(relay, &format!("Connection refused {}", i));
            assert_eq!(mgr.get_failure_count(relay), i + 1);
        }

        // Circuit should be open now
        assert_eq!(mgr.get_state(relay), CircuitState::Open);
        assert!(!mgr.allow_request(relay));
    }

    #[test]
    fn test_circuit_does_not_open_before_threshold() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        mgr.record_failure(relay, "timeout");
        mgr.record_failure(relay, "timeout");

        assert_eq!(mgr.get_state(relay), CircuitState::Closed);
        assert!(mgr.allow_request(relay));
    }

    #[test]
    fn test_success_resets_closed_circuit() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        mgr.record_failure(relay, "timeout");
        mgr.record_failure(relay, "timeout");
        mgr.record_success(relay);

        assert_eq!(mgr.get_failure_count(relay), 0);
        assert_eq!(mgr.get_state(relay), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_success_closes_circuit() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        // Open the circuit
        for _ in 0..3 {
            mgr.record_failure(relay, "connection failed");
        }
        assert_eq!(mgr.get_state(relay), CircuitState::Open);

        // Manually transition to half-open (simulating open_timeout elapsed)
        mgr.transition_to_half_open(relay);
        assert_eq!(mgr.get_state(relay), CircuitState::HalfOpen);

        // Successful probes should close the circuit
        mgr.record_success(relay);
        assert_eq!(mgr.get_state(relay), CircuitState::HalfOpen); // Need 2 successes

        mgr.record_success(relay);
        assert_eq!(mgr.get_state(relay), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_failure_reopens_circuit() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        // Open the circuit
        for _ in 0..3 {
            mgr.record_failure(relay, "connection failed");
        }

        // Transition to half-open
        mgr.transition_to_half_open(relay);
        assert_eq!(mgr.get_state(relay), CircuitState::HalfOpen);

        // Failure in half-open should re-open
        mgr.record_failure(relay, "still failing");
        assert_eq!(mgr.get_state(relay), CircuitState::Open);
    }

    #[test]
    fn test_reset_specific_relay() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        for _ in 0..3 {
            mgr.record_failure(relay, "connection failed");
        }
        assert_eq!(mgr.get_state(relay), CircuitState::Open);

        mgr.reset(relay);
        assert_eq!(mgr.get_state(relay), CircuitState::Closed);
    }

    #[test]
    fn test_reset_all() {
        let mgr = CircuitBreakerManager::with_defaults();

        for relay in ["r1", "r2", "r3"] {
            for _ in 0..3 {
                mgr.record_failure(relay, "failed");
            }
        }

        mgr.reset_all();
        for relay in ["r1", "r2", "r3"] {
            assert_eq!(mgr.get_state(relay), CircuitState::Closed);
        }
    }

    #[test]
    fn test_get_stats() {
        let mgr = CircuitBreakerManager::with_defaults();

        // 2 failures on r1 (still closed)
        mgr.record_failure("r1", "fail");
        mgr.record_failure("r1", "fail");

        // 3 failures on r2 (open)
        for _ in 0..3 {
            mgr.record_failure("r2", "fail");
        }

        // r3 stays closed (no entries)

        let stats = mgr.get_stats();
        assert_eq!(stats.closed_count, 1); // r1
        assert_eq!(stats.open_count, 1); // r2
        assert_eq!(stats.total, 2);
    }

    #[test]
    fn test_get_open_circuits() {
        let mgr = CircuitBreakerManager::with_defaults();

        for _ in 0..3 {
            mgr.record_failure("open-relay", "fail");
        }
        mgr.record_failure("closed-relay", "fail"); // Only 1 failure

        let open = mgr.get_open_circuits();
        assert_eq!(open.len(), 1);
        assert!(open.contains(&"open-relay".to_string()));
    }

    #[test]
    fn test_last_failure_reason() {
        let mgr = CircuitBreakerManager::with_defaults();
        let relay = "relay1.example.com";

        mgr.record_failure(relay, "DNS resolution failed");
        assert_eq!(mgr.get_last_failure_reason(relay), Some("DNS resolution failed".to_string()));
    }
}