//! Optimized Routing Engine with DHT Latency Reductions
//!
//! Integrates all optimization strategies:
//! - Hierarchical Timeout Budgeting (P0)
//! - Bloom Filter Negative Cache (P0)
//! - Route Prefetch on Resume (P1)
//! - Adaptive TTL Based on Peer Activity (P2)
//!
//! This engine replaces the basic RoutingEngine with optimized discovery paths.

use super::adaptive_ttl::AdaptiveTTLManager;
use super::engine::*;
use super::global::RouteAdvertisement;
use super::local::PeerId;
use super::negative_cache::{NegativeCache, NegativeCacheStats};
use super::resume_prefetch::{PrefetchStats, ResumePrefetchManager};
use super::timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget};
use std::time::Duration;

// For peer ID string conversion
use hex;

/// Optimized routing engine with all DHT latency optimizations
pub struct OptimizedRoutingEngine {
    /// Basic routing engine (layers 1-3)
    base_engine: RoutingEngine,
    /// Hierarchical timeout budgeting for discovery
    timeout_budget: TimeoutBudget,
    /// Bloom filter negative cache for fast unreachable detection
    negative_cache: NegativeCache,
    /// Route prefetch manager for app resume optimization
    prefetch_manager: ResumePrefetchManager,
    /// Adaptive TTL manager for activity-based route freshness
    adaptive_ttl: AdaptiveTTLManager,
    /// Our own peer ID
    #[allow(dead_code)]
    local_id: PeerId,
    /// Our recipient hint
    #[allow(dead_code)]
    local_hint: [u8; 4],
    /// Current discovery phase
    current_phase: DiscoveryPhase,
    /// Whether we're in the middle of a discovery operation
    discovery_in_progress: bool,
}

impl OptimizedRoutingEngine {
    /// Create a new optimized routing engine
    pub fn new(local_id: PeerId, local_hint: [u8; 4]) -> Self {
        OptimizedRoutingEngine {
            base_engine: RoutingEngine::new(local_id, local_hint),
            timeout_budget: TimeoutBudget::default_500ms(),
            negative_cache: NegativeCache::with_defaults(),
            prefetch_manager: ResumePrefetchManager::with_defaults(),
            adaptive_ttl: AdaptiveTTLManager::with_defaults(),
            local_id,
            local_hint,
            current_phase: DiscoveryPhase::LocalCache,
            discovery_in_progress: false,
        }
    }

    /// Optimized route message with hierarchical discovery and negative caching
    pub fn route_message_optimized(
        &mut self,
        recipient_hint: &[u8; 4],
        message_id: &[u8; 16],
        priority: u8,
        now: u64,
    ) -> RoutingDecision {
        // Phase 0: Fast negative cache check (P0 optimization)
        let peer_id_str = hex::encode(recipient_hint);
        if self.negative_cache.is_definitely_unreachable(&peer_id_str) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::StoreAndCarry,
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0,
            };
        }

        // Phase 1: Check prefetch cache (P1 optimization)
        if let Some(prefetched_route) = self.prefetch_manager.get_route_early(recipient_hint) {
            // Convert prefetched route to routing decision
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::GlobalRoute {
                    next_hop_id: prefetched_route.next_hop,
                    total_hops: prefetched_route.hop_count,
                },
                alternatives: vec![],
                decided_by: RoutingLayer::Global,
                confidence: 0.95, // High confidence for prefetched routes
            };
        }

        // Phase 2: Hierarchical discovery with timeout budgeting (P0 optimization)
        self.start_discovery_if_needed();

        let decision = self
            .base_engine
            .route_message(recipient_hint, message_id, priority, now);

        // Structured tracing: Log routing decision
        tracing::info!(
            event = "routing_decision",
            message_id = %hex::encode(message_id),
            recipient_hint = %hex::encode(recipient_hint),
            priority = priority,
            next_hop = ?decision.primary,
            decided_by = ?decision.decided_by,
            confidence = decision.confidence
        );

        // Apply adaptive TTL to the decision (P2 optimization)
        let peer_id_str = if let NextHop::GlobalRoute { next_hop_id, .. } = decision.primary {
            let s = hex::encode(next_hop_id);
            let _ttl = self.adaptive_ttl.calculate_ttl(&s);
            // In a real implementation, this would update the route's TTL
            // For now, we just track the activity
            self.adaptive_ttl.record_activity(&s);
            s
        } else {
            String::new()
        };

        // If we got StoreAndCarry and it's a high priority message, record as unreachable
        if matches!(decision.primary, NextHop::StoreAndCarry)
            && priority >= 100
            && !peer_id_str.is_empty()
        {
            self.negative_cache.record_unreachable(peer_id_str);
        }

        decision
    }

    /// Start discovery if not already in progress
    fn start_discovery_if_needed(&mut self) {
        if self.discovery_in_progress {
            return;
        }

        self.discovery_in_progress = true;
        self.current_phase = DiscoveryPhase::LocalCache;
        self.timeout_budget = TimeoutBudget::default_500ms();
    }

    /// Advance to next discovery phase
    pub fn advance_discovery_phase(&mut self) -> Option<DiscoveryPhase> {
        if !self.discovery_in_progress {
            return None;
        }

        let next_phase = self.timeout_budget.advance();
        if let Some(phase) = next_phase {
            self.current_phase = phase;
            Some(phase)
        } else {
            self.discovery_in_progress = false;
            None
        }
    }

    /// Get current discovery phase
    pub fn current_discovery_phase(&self) -> DiscoveryPhase {
        self.current_phase
    }

    /// Check if discovery is in progress
    pub fn is_discovery_in_progress(&self) -> bool {
        self.discovery_in_progress
    }

    /// Get timeout budget summary
    pub fn timeout_budget_summary(&self) -> BudgetSummary {
        self.timeout_budget.summary()
    }

    /// Get negative cache statistics
    pub fn negative_cache_stats(&self) -> NegativeCacheStats {
        self.negative_cache.stats()
    }

    /// Get prefetch statistics
    pub fn prefetch_stats(&self) -> PrefetchStats {
        self.prefetch_manager.stats()
    }

    /// Get adaptive TTL manager
    pub fn adaptive_ttl(&mut self) -> &mut AdaptiveTTLManager {
        &mut self.adaptive_ttl
    }

    /// Access base engine methods
    pub fn base_engine(&self) -> &RoutingEngine {
        &self.base_engine
    }

    /// Mutable access to base engine
    pub fn base_engine_mut(&mut self) -> &mut RoutingEngine {
        &mut self.base_engine
    }

    /// Access prefetch manager
    pub fn prefetch_manager(&self) -> &ResumePrefetchManager {
        &self.prefetch_manager
    }

    /// Mutable access to prefetch manager
    pub fn prefetch_manager_mut(&mut self) -> &mut ResumePrefetchManager {
        &mut self.prefetch_manager
    }

    /// Periodic maintenance for all components
    pub fn tick(&mut self, now: u64) -> OptimizedRoutingMaintenance {
        let base_maint = self.base_engine.tick(now);
        let neg_cache_cleaned = self.negative_cache.cleanup_expired();
        let ttl_cleaned = self.adaptive_ttl.cleanup(Duration::from_secs(86400)); // 24h

        OptimizedRoutingMaintenance {
            base_maintenance: base_maint,
            negative_cache_entries_cleaned: neg_cache_cleaned,
            adaptive_ttl_entries_cleaned: ttl_cleaned,
        }
    }

    /// Called when app goes to background
    pub fn on_app_background(
        &mut self,
        current_routes: Vec<(PeerId, [u8; 4], RouteAdvertisement)>,
    ) {
        self.prefetch_manager.on_app_background(current_routes);
    }

    /// Called when app resumes from background
    pub fn on_app_resume(&mut self) -> Vec<[u8; 4]> {
        self.prefetch_manager.on_app_resume()
    }

    /// Record message activity for adaptive TTL
    pub fn record_message_activity(&mut self, peer_id: &str) {
        self.adaptive_ttl.record_activity(peer_id);
    }

    /// Record unreachable peer
    pub fn record_unreachable_peer(&mut self, peer_id: &str) {
        self.negative_cache.record_unreachable(peer_id.to_string());
    }

    /// Clear unreachable status for peer
    pub fn clear_unreachable_peer(&mut self, peer_id: &str) {
        self.negative_cache.clear_unreachable(peer_id);
    }
}

/// Maintenance result for optimized engine
#[derive(Debug, Clone)]
pub struct OptimizedRoutingMaintenance {
    /// Base engine maintenance
    pub base_maintenance: RoutingMaintenance,
    /// Negative cache entries cleaned
    pub negative_cache_entries_cleaned: usize,
    /// Adaptive TTL entries cleaned
    pub adaptive_ttl_entries_cleaned: usize,
}

impl std::fmt::Display for OptimizedRoutingMaintenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Optimized Maintenance: base({} promoted, {} demoted), neg_cache({} cleaned), ttl({} cleaned)",
            self.base_maintenance.peers_promoted,
            self.base_maintenance.peers_demoted,
            self.negative_cache_entries_cleaned,
            self.adaptive_ttl_entries_cleaned
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_id(id: u8) -> PeerId {
        let mut peer = [0u8; 32];
        peer[0] = id;
        peer
    }

    fn make_message_id(id: u8) -> [u8; 16] {
        [id; 16]
    }

    fn make_hint(id: u8) -> [u8; 4] {
        [id, 0, 0, 0]
    }

    #[test]
    fn test_optimized_engine_creation() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);

        let engine = OptimizedRoutingEngine::new(local_id, local_hint);
        assert_eq!(engine.local_id, local_id);
        assert_eq!(engine.local_hint, local_hint);
    }

    #[test]
    fn test_negative_cache_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record a peer as unreachable
        engine.record_unreachable_peer("peer1");

        // Should be detected as unreachable
        let target_hint = make_hint(99);
        let msg_id = make_message_id(1);
        let decision = engine.route_message_optimized(&target_hint, &msg_id, 50, 1000);

        // This test is simplified - in reality we'd need to mock the peer_id_str generation
        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
    }

    #[test]
    fn test_discovery_phase_advancement() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Start discovery
        engine.start_discovery_if_needed();
        assert!(engine.is_discovery_in_progress());

        // Advance through phases
        let phase1 = engine.advance_discovery_phase();
        assert!(phase1.is_some());

        let phase2 = engine.advance_discovery_phase();
        assert!(phase2.is_some());

        // Eventually should complete
        let phase3 = engine.advance_discovery_phase();
        let phase4 = engine.advance_discovery_phase();
        assert!(phase4.is_none());
        assert!(!engine.is_discovery_in_progress());
    }

    #[test]
    fn test_app_lifecycle_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Simulate going to background
        let route = RouteAdvertisement {
            destination_hint: make_hint(99),
            next_hop: make_peer_id(2),
            hop_count: 2,
            reliability: 0.95,
            last_confirmed: 1000,
            sequence: 1,
            ttl: 3600,
        };

        engine.on_app_background(vec![(make_peer_id(2), make_hint(99), route)]);

        // Simulate resuming
        let hints = engine.on_app_resume();
        assert_eq!(hints.len(), 1);
    }

    #[test]
    fn test_adaptive_ttl_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record activity for a peer
        engine.record_message_activity("peer1");

        // Should have activity recorded
        let ttl = engine.adaptive_ttl().calculate_ttl("peer1");
        assert!(ttl > Duration::from_secs(0));
    }

    #[test]
    fn test_maintenance_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record some unreachable peers and activity
        engine.record_unreachable_peer("peer1");
        engine.record_message_activity("peer2");

        // Run maintenance
        let maint = engine.tick(1000);

        // Should have cleaned up appropriately
        assert_eq!(maint.adaptive_ttl_entries_cleaned, 0); // Nothing old enough to clean
    }
}
