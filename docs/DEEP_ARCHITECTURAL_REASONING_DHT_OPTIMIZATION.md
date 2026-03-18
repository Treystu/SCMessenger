# Deep Architectural Reasoning: DHT Peer Discovery Latency Optimization

**Status:** Active
**Created:** 2026-03-16
**Updated:** 2026-03-18
**Problem Domain:** Reducing route discovery latency for sleeping mobile peers from >2 seconds to <2-5 seconds (target: <2s) on self-hosted hardware

---

## Executive Summary

This document provides a comprehensive prompt for optimizing DHT-based peer discovery in SCMessenger, specifically targeting the "expensive query" of finding routes to sleeping mobile nodes. The goal is to reduce latency from >2 seconds to <2-5 seconds while maintaining the sovereign, decentralized architecture on self-hosted hardware.

### User Constraints (Interview Results)

| Constraint | Value | Implication |
|------------|-------|-------------|
| Infrastructure | Self-hosted hardware | No cloud costs, limited CPU/memory |
| Scale | Global, worldwide | Need efficient DHT routing for 10K+ peers |
| Cost concerns | All (bandwidth, compute, storage) | Bandwidth is primary constraint |
| Latency target | 2-5 seconds (best: <2s) | Aggressive optimization needed |
| Architecture | Headless nodes = relays/delegates | These are "always online" nodes |
| Wake mechanism | Push notifications | Can reduce DHT polling |
| Relay model | All phones relay for others | Sovereign, no-cost communication |

---

## The Expensive Query: Current State Analysis

### Current Discovery Path (O(log n) with timeouts)

```
User sends message to Peer X
    ↓
[1] Check Local Cell (< 10ms)
    ↓ miss
[2] Check Neighborhood Gateways (< 50ms)
    ↓ miss
[3] Check Global Routes (< 100ms)
    ↓ miss or stale
[4] Initiate Kademlia DHT Walk
    - Alpha=8 concurrent queries
    - Each query: 100-500ms timeout
    - Depth: O(log n) = 4-6 hops for 10K peers
    - Total: 400ms - 3000ms
    ↓
[5] Peer is sleeping → timeout
    ↓
[6] Query for delegate nodes (another DHT walk)
    ↓
[7] Send wake request to delegate
    ↓
[8] Wait for peer to wake and reconnect
    ↓
[9] Finally deliver message

**Total latency: 2-10 seconds in worst case**
```

### Why It's Expensive

| Cost Factor | Current Behavior | Impact |
|-------------|------------------|--------|
| Kademlia alpha | 8 concurrent queries | 8x message overhead per hop |
| Timeout per query | 500ms default | Cascading timeouts on sleeping peers |
| Route TTL | Fixed 3600s | Stale routes cause re-discovery |
| No predictive caching | Reactive only | Discovery triggered by first message |
| No negative cache | Re-query unknown peers | Wasted lookups on unreachable peers |
| No parallel paths | Sequential fallback | Each transport tried one at a time |

---

## Deep Architectural Reasoning Framework

### Constraint 1: Sovereignty Preservation (Non-Negotiable)

**PHIL-001, PHIL-003:** Identity and cryptographic authority must remain in Rust core. Any optimization cannot introduce centralized coordination or break the trust model.

**Implications:**
- Caches must be local-only (no shared cache servers)
- Predictions must be derived from local observation
- No central "peer location service"

### Constraint 2: Privacy Preservation (Non-Negotiable)

**PHIL-004:** First-run consent explains security/privacy boundaries.

**Implications:**
- Route caching reveals communication patterns to the local device
- Predictive prefetching must not leak metadata to the network
- Bloom filters for negative caches must not reveal who you're NOT messaging

### Constraint 3: Battery Efficiency (Negotiable with Trade-offs)

**PHIL-005:** Bounded retention prevents unbounded growth.

**Implications:**
- Prefetching must be batched and opportunistic
- Background DHT maintenance must respect OS power management
- Adaptive scan intervals based on device state

### Constraint 4: Cross-Platform Parity (Non-Negotiable)

**PHIL-006, PHIL-010:** Critical behavior must be identical across Android, iOS, Web.

**Implications:**
- Optimization logic must be in Rust core, not platform-specific
- Cache invalidation must be deterministic
- Timing-dependent behavior must be testable

---

## Self-Hosted Global Mesh Optimization Strategy

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    GLOBAL MESH NETWORK                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐                │
│   │ Headless │    │ Headless │    │ Headless │   Always Online │
│   │ Node A   │    │ Node B   │    │ Node C   │   (Delegates)  │
│   └────┬─────┘    └────┬─────┘    └────┬─────┘                │
│        │               │               │                       │
│        └───────────────┼───────────────┘                       │
│                        │                                       │
│              DHT + Push Notifications                          │
│                        │                                       │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐                │
│   │ Mobile   │    │ Mobile   │    │ Mobile   │   Sleeping     │
│   │ Phone 1  │    │ Phone 2  │    │ Phone 3  │   (Wakeable)   │
│   └──────────┘    └──────────┘    └──────────┘                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Key Insight: Headless Nodes as Shared Infrastructure

**Critical Architecture Detail (User Clarification):** Headless/no-identity nodes are NOT designated to any specific user. All users can use any node to relay, as all nodes are full relay nodes. Headless nodes are just presumed to be more viable (more reliable, always online).

**Implications:**
1. **Shared infrastructure**: Any user can use any headless node as a relay/delegate
2. **No assignment needed**: No need to "assign" delegates to specific users
3. **Natural DHT routing**: Headless nodes are just "better" relay nodes (more reliable)
4. **Simplified wake protocol**: Any headless node can wake any sleeping peer
5. **Democratic access**: All nodes (mobile and headless) are full relay nodes

**Why headless nodes are "more viable":**
- **Always online**: No battery constraints, no OS sleep
- **Cheap to run**: Self-hosted hardware, no cloud costs
- **No privacy concerns**: No user data, just routing
- **Predictable**: Stable network addresses for DHT routing
- **Higher reliability**: No app crashes, no user intervention

### Optimized Discovery Flow (Target: <2s)

```
User sends message to Sleeping Peer X
    ↓
[1] Check Local Cache (< 10ms)
    ↓ miss
[2] Query Headless Delegate Nodes (parallel, < 500ms)
    - Pre-registered delegates for Peer X
    - Known always-online nodes
    ↓ miss
[3] Send Push Notification to Peer X (< 100ms)
    - FCM/APNs wake signal
    - "You have a message waiting"
    ↓
[4] Peer X wakes, connects to nearest delegate (< 1s)
    ↓
[5] Delegate forwards message (< 500ms)

**Total latency: < 2 seconds**
```

### Strategy 1: Shared Headless Node Registry

**Concept:** Maintain a registry of headless nodes as shared infrastructure. Any user can use any headless node as a relay/delegate. No user-specific assignment needed.

**Key Insight:** Headless nodes are NOT assigned to specific users - they are shared infrastructure that any peer can use. All nodes (mobile and headless) are full relay nodes. Headless nodes are just more reliable (always online).

**Implementation:**

```rust
// core/src/relay/shared_delegate_registry.rs

pub struct SharedDelegateRegistry {
    /// Known headless nodes (shared infrastructure, always online)
    headless_nodes: HashMap<PeerId, HeadlessNodeInfo>,
    /// Known mobile nodes (may be sleeping)
    mobile_nodes: HashMap<PeerId, MobileNodeInfo>,
    /// Push token registry for wakeup
    push_tokens: HashMap<PeerId, PushToken>,
}

pub struct HeadlessNodeInfo {
    pub peer_id: PeerId,
    pub multiaddr: Multiaddr,
    pub last_seen: Instant,
    pub reliability_score: f64,
    pub is_headless: bool,
    pub max_peers: usize,
    pub current_peers: usize,
}

pub struct MobileNodeInfo {
    pub peer_id: PeerId,
    pub last_seen: Instant,
    pub is_sleeping: bool,
    pub push_token: Option<PushToken>,
    pub preferred_delegates: Vec<PeerId>,
}

impl SharedDelegateRegistry {
    /// Register a headless node as shared infrastructure
    pub fn register_headless_node(&mut self, info: HeadlessNodeInfo) {
        self.headless_nodes.insert(info.peer_id, info);
    }
    
    /// Find best available headless nodes for any user
    /// No user-specific assignment - all users share the same pool
    pub fn find_best_delegates(&self, count: usize) -> Vec<PeerId> {
        // Select best delegates based on:
        // 1. Reliability score (headless nodes are more reliable)
        // 2. Available capacity
        // 3. Network proximity (latency)
        let candidates: Vec<_> = self.headless_nodes.values()
            .filter(|n| n.current_peers < n.max_peers)
            .sorted_by(|a, b| b.reliability_score.cmp(&a.reliability_score))
            .take(count)
            .map(|n| n.peer_id)
            .collect();
        
        candidates
    }
    
    /// Wake a sleeping peer via any available headless node
    pub async fn wake_peer_via_delegate(
        &mut self,
        target_peer: &PeerId,
        message_id: &str,
    ) -> Result<(), WakeError> {
        // 1. Find any available headless node
        let delegates = self.find_best_delegates(3);
        
        // 2. Send wake request to each delegate (parallel)
        let wake_futures: Vec<_> = delegates.iter()
            .map(|delegate| self.send_wake_request(delegate, target_peer, message_id))
            .collect();
        
        // 3. Return as soon as one delegate acknowledges
        let results = futures::future::select_all(wake_futures).await;
        Ok(())
    }
}
```

### Strategy 2: Push Notification Wakeup Integration

**Concept:** Use FCM/APNs to wake sleeping peers instead of waiting for DHT discovery.

**Implementation:**

```rust
// core/src/relay/push_wakeup.rs

pub struct PushWakeupManager {
    /// Peer -> Push token mapping
    push_tokens: HashMap<PeerId, PushToken>,
    /// Pending wake requests
    pending_wakes: HashMap<PeerId, WakeRequest>,
}

pub struct PushToken {
    pub peer_id: PeerId,
    pub token: String,
    pub platform: PushPlatform,
    pub last_updated: Instant,
}

pub enum PushPlatform {
    Fcm,    // Android
    Apns,   // iOS
    WebPush, // WASM
}

impl PushWakeupManager {
    /// Send wakeup notification to sleeping peer
    pub async fn wake_peer(&mut self, peer_id: &PeerId, message_id: &str) -> Result<(), PushError> {
        let token = self.push_tokens.get(peer_id)
            .ok_or(PushError::NoToken)?;
        
        // Send lightweight wake signal (no message content)
        let wake_signal = WakeSignal {
            action: "WAKE_P2P",
            message_id: message_id.to_string(),
            timestamp: current_timestamp(),
        };
        
        match token.platform {
            PushPlatform::Fcm => self.send_fcm(token, wake_signal).await,
            PushPlatform::Apns => self.send_apns(token, wake_signal).await,
            PushPlatform::WebPush => self.send_webpush(token, wake_signal).await,
        }
    }
    
    /// Register push token for a peer
    pub fn register_token(&mut self, peer_id: PeerId, token: String, platform: PushPlatform) {
        self.push_tokens.insert(peer_id, PushToken {
            peer_id,
            token,
            platform,
            last_updated: Instant::now(),
        });
    }
}
```

### Strategy 3: Shared Delegate Sleep Registration

**Concept:** Before going to sleep, mobile peers register with ANY available headless delegate (shared infrastructure) and share push tokens. No user-specific assignment needed.

**Key Insight:** Since all headless nodes are shared infrastructure, any headless node can wake any sleeping peer. The mobile peer just needs to register with 2-3 reliable headless nodes.

**Implementation:**

```rust
// core/src/relay/sleep_preparation.rs

pub struct SleepPreparationManager {
    /// Shared delegate registry (any user can use any headless node)
    delegate_registry: Arc<SharedDelegateRegistry>,
    /// Push wakeup manager
    push_manager: Arc<PushWakeupManager>,
}

impl SleepPreparationManager {
    /// Prepare for sleep: register with ANY available headless delegates
    pub async fn prepare_for_sleep(
        &mut self,
        swarm: &SwarmHandle,
        push_token: Option<String>,
        platform: PushPlatform,
    ) -> Result<(), SleepError> {
        // 1. Find 2-3 best available headless nodes (shared infrastructure)
        let delegates = self.delegate_registry.find_best_delegates(3);
        
        if delegates.is_empty() {
            tracing::warn!("No headless delegates available for sleep registration");
            return Err(SleepError::NoDelegatesAvailable);
        }
        
        // 2. Register with each delegate (parallel)
        let registration_futures: Vec<_> = delegates.iter()
            .map(|delegate_id| {
                let registration = SleepRegistration {
                    peer_id: swarm.local_peer_id(),
                    push_token: push_token.clone(),
                    platform: platform.clone(),
                    wakeup_protocol: "/sc/wakeup/1.0.0",
                };
                
                swarm.send_message(
                    delegate_id,
                    serialize_registration(registration)
                )
            })
            .collect();
        
        // Wait for at least one delegate to acknowledge
        let results = futures::future::join_all(registration_futures).await;
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        
        if success_count == 0 {
            return Err(SleepError::RegistrationFailed);
        }
        
        // 3. Update DHT provider records to point to delegates
        swarm.update_provider_record(
            &swarm.local_peer_id(),
            delegates.iter().map(|d| d.to_string()).collect()
        ).await?;
        
        tracing::info!("Registered with {}/{} delegates for sleep", success_count, delegates.len());
        Ok(())
    }
}
```

### Strategy 4: Bandwidth-Efficient DHT Configuration

**Concept:** Optimize DHT parameters for self-hosted hardware with limited bandwidth.

**Configuration:**

```rust
// core/src/transport/behaviour.rs

// Optimized for self-hosted hardware with limited bandwidth
let kad_config = kad::Config::default()
    .set_parallelism(4)  // Reduced from 8 to save bandwidth
    .set_replication_factor(3)  // Reduced from 5 to save bandwidth
    .set_query_timeout(Duration::from_secs(2))  // Reduced from 5s
    .set_record_ttl(Some(Duration::from_secs(3600)))  // 1 hour
    .set_publication_interval(Duration::from_secs(1800)); // 30 minutes
```

### Strategy 5: Hierarchical Discovery with Cost Awareness

**Concept:** Use progressively more expensive discovery methods only when needed.

**Implementation:**

```rust
// core/src/routing/cost_aware_discovery.rs

pub struct CostAwareDiscovery {
    /// Local cache (cheapest)
    local_cache: RouteCache,
    /// Delegate queries (medium cost)
    delegate_queries: DelegateQueryManager,
    /// DHT walk (most expensive)
    dht_walk: DhtWalkManager,
    /// Push wakeup (fallback)
    push_wakeup: PushWakeupManager,
}

impl CostAwareDiscovery {
    /// Discover a peer with cost-aware strategy
    pub async fn discover_peer(
        &mut self,
        target: &PeerId,
        timeout: Duration,
    ) -> Result<DiscoveryResult, DiscoveryError> {
        let start = Instant::now();
        
        // Phase 1: Local cache (< 10ms)
        if let Some(route) = self.local_cache.get(target) {
            return Ok(DiscoveryResult::from_cache(route));
        }
        
        // Phase 2: Delegate queries (< 500ms)
        let remaining = timeout - start.elapsed();
        if remaining > Duration::from_millis(100) {
            if let Some(route) = self.delegate_queries.query(target, remaining).await? {
                return Ok(DiscoveryResult::from_delegate(route));
            }
        }
        
        // Phase 3: Push wakeup (< 100ms to send)
        let remaining = timeout - start.elapsed();
        if remaining > Duration::from_millis(50) {
            self.push_wakeup.wake_peer(target, "discovery").await?;
            // Don't wait for response - let delegate handle delivery
        }
        
        // Phase 4: DHT walk (last resort, expensive)
        let remaining = timeout - start.elapsed();
        if remaining > Duration::from_millis(200) {
            if let Some(route) = self.dht_walk.walk(target, remaining).await? {
                return Ok(DiscoveryResult::from_dht(route));
            }
        }
        
        Err(DiscoveryError::Timeout)
    }
}
```

## Optimization Strategies: Making the Expensive Query Cheap

### Strategy 1: Predictive Route Caching (Target: -500ms)

**Concept:** Pre-compute and cache routes during idle time, before they're needed.

**Implementation:**

```rust
// core/src/routing/predictive.rs

pub struct PredictiveRouteCache {
    /// Recently active conversations (who messages whom)
    conversation_graph: HashMap<PeerId, Vec<ConversationEdge>>,
    /// Pre-computed routes for predicted recipients
    warm_routes: LruCache<[u8; 4], Vec<RouteAdvertisement>>,
    /// Last time we refreshed each route
    route_freshness: HashMap<[u8; 4], Instant>,
    /// Maximum age before re-validation
    max_route_age: Duration,
}

impl PredictiveRouteCache {
    /// Called when a message is SENT - record the conversation pattern
    pub fn record_conversation(&mut self, recipient: PeerId, timestamp: Instant) {
        // Update conversation graph for future predictions
        // "User X frequently messages User Y at this time of day"
    }
    
    /// Called during idle time - refresh routes for predicted recipients
    pub fn prefetch_routes(&mut self, routing_engine: &RoutingEngine) {
        // Predict who will be messaged next based on:
        // - Recent conversation patterns
        // - Time of day
        // - App foreground/background state
        // Pre-fetch routes for top-N predicted recipients
    }
    
    /// Called when looking up a route - return cached if fresh
    pub fn get_route(&mut self, hint: &[u8; 4]) -> Option<&Vec<RouteAdvertisement>> {
        // Check if we have a fresh cached route
        // Return immediately if < max_route_age
        // Otherwise trigger background refresh
    }
}
```

**Latency Reduction:**
- Before: 2000ms (full DHT walk on first message)
- After: 10ms (cache hit) or 100ms (background refresh in progress)

### Strategy 2: Hierarchical Timeout Budgeting (Target: -300ms)

**Concept:** Instead of fixed 500ms timeouts, use a total time budget with progressive fallback.

**Implementation:**

```rust
// core/src/routing/timed_discovery.rs

pub struct TimeoutBudget {
    /// Total time budget for this discovery (e.g., 500ms)
    total_budget: Duration,
    /// Time spent so far
    elapsed: Duration,
    /// Current phase
    phase: DiscoveryPhase,
}

pub enum DiscoveryPhase {
    /// Phase 1: Local cache lookup (0-10ms budget)
    LocalCache,
    /// Phase 2: Neighborhood gossip query (10-50ms budget)
    NeighborhoodQuery,
    /// Phase 3: Targeted delegate query (50-200ms budget)
    DelegateQuery,
    /// Phase 4: Full DHT walk (200-500ms budget, last resort)
    FullDhtWalk,
}

impl TimeoutBudget {
    pub fn next_phase(&mut self) -> Option<DiscoveryPhase> {
        let remaining = self.total_budget.saturating_sub(self.elapsed);
        
        match self.phase {
            DiscoveryPhase::LocalCache if remaining.as_millis() > 50 => {
                self.phase = DiscoveryPhase::NeighborhoodQuery;
                Some(self.phase.clone())
            }
            DiscoveryPhase::NeighborhoodQuery if remaining.as_millis() > 200 => {
                self.phase = DiscoveryPhase::DelegateQuery;
                Some(self.phase.clone())
            }
            DiscoveryPhase::DelegateQuery if remaining.as_millis() > 100 => {
                self.phase = DiscoveryPhase::FullDhtWalk;
                Some(self.phase.clone())
            }
            _ => None, // Budget exhausted
        }
    }
}
```

**Latency Reduction:**
- Before: 500ms per phase × 4 phases = 2000ms worst case
- After: 500ms total budget with early termination on success

### Strategy 3: Bloom Filter Negative Cache (Target: -200ms wasted lookups)

**Concept:** Quickly know which peers are DEFINITELY unreachable without doing a DHT walk.

**Implementation:**

```rust
// core/src/routing/negative_cache.rs

pub struct NegativeCache {
    /// Bloom filter of peers we've confirmed are unreachable
    unreachable_filter: BloomFilter,
    /// Timestamps when we last confirmed unreachability
    unreachable_times: HashMap<PeerId, Instant>,
    /// How long to trust a negative result
    negative_ttl: Duration,
}

impl NegativeCache {
    /// Check if we recently confirmed this peer is unreachable
    pub fn is_definitely_unreachable(&self, peer: &PeerId) -> bool {
        if self.unreachable_filter.contains(peer) {
            // Check if the negative result is still fresh
            if let Some(confirmed_at) = self.unreachable_times.get(peer) {
                return confirmed_at.elapsed() < self.negative_ttl;
            }
        }
        false
    }
    
    /// Record that we confirmed a peer is unreachable
    pub fn record_unreachable(&mut self, peer: PeerId) {
        self.unreachable_filter.insert(&peer);
        self.unreachable_times.insert(peer, Instant::now());
    }
    
    /// Record that a peer became reachable (clear negative)
    pub fn clear_unreachable(&mut self, peer: &PeerId) {
        // Note: Bloom filters can't truly delete, but we update the timestamp
        // so is_definitely_unreachable returns false
        self.unreachable_times.remove(peer);
    }
}
```

**Latency Reduction:**
- Before: 2000ms DHT walk for unreachable peer
- After: 1ms bloom filter check → return StoreAndCarry immediately

### Strategy 4: Speculative Delegate Pre-warming (Target: -1000ms)

**Concept:** Register with delegates BEFORE going to sleep, and pre-warm delegate connections.

**Implementation:**

```rust
// core/src/relay/delegate_prewarm.rs

pub struct DelegatePrewarmManager {
    /// Current delegate assignments
    delegates: Vec<DelegateInfo>,
    /// Pre-warmed connections to delegates
    warm_connections: HashMap<PeerId, WarmConnection>,
    /// When to next refresh delegate connections
    next_refresh: Instant,
}

pub struct DelegateInfo {
    pub peer_id: PeerId,
    pub multiaddr: Multiaddr,
    pub reliability_score: f64,
    pub last_confirmed: Instant,
}

impl DelegatePrewarmManager {
    /// Called before app goes to background (iOS/Android lifecycle)
    pub async fn prewarm_for_background(&mut self, swarm: &mut SwarmHandle) {
        // 1. Select top-N reliable delegates
        // 2. Establish connections to them NOW
        // 3. Send "I'm going to sleep, please wake me" registration
        // 4. Keep connections warm with periodic pings
    }
    
    /// Called when app comes to foreground
    pub async fn refresh_delegate_routes(&mut self, routing_engine: &mut RoutingEngine) {
        // 1. Re-validate all delegate routes
        // 2. Update global route advertisements
        // 3. Warm the predictive cache with delegate routes
    }
}
```

**Latency Reduction:**
- Before: 1000ms (discover delegate + connect + register)
- After: 0ms (already connected and registered)

### Strategy 5: Route Prefetch on App Resume (Target: -500ms)

**Concept:** When app wakes from background, immediately refresh known routes instead of waiting for first message.

**Implementation:**

```rust
// core/src/routing/resume_prefetch.rs

pub struct ResumePrefetchManager {
    /// Routes that were valid before background
    cached_routes: HashMap<[u8; 4], RouteAdvertisement>,
    /// Peers we frequently message (candidates for prefetch)
    frequent_peers: Vec<PeerId>,
    /// Whether prefetch is in progress
    prefetch_in_progress: bool,
}

impl ResumePrefetchManager {
    /// Called on app resume (iOS/Android lifecycle event)
    pub async fn on_app_resume(&mut self, routing_engine: &mut RoutingEngine) {
        self.prefetch_in_progress = true;
        
        // 1. Immediately re-validate cached routes (parallel)
        let validation_futures: Vec<_> = self.cached_routes.iter()
            .map(|(hint, route)| self.validate_route(hint, route))
            .collect();
        
        // 2. Wait for first N validations, then mark ready
        // 3. Continue background refresh for remaining routes
    }
    
    /// Get a route immediately if available (even if slightly stale)
    pub fn get_route_early(&self, hint: &[u8; 4]) -> Option<&RouteAdvertisement> {
        // Return cached route even if > max_route_age
        // Better to deliver with slight delay than wait for fresh route
    }
}
```

**Latency Reduction:**
- Before: 2000ms on first message after resume
- After: 10ms (route already refreshed during resume)

### Strategy 6: Adaptive TTL Based on Peer Activity (Target: -300ms re-discovery)

**Concept:** Keep routes fresh longer for active peers, expire faster for inactive peers.

**Implementation:**

```rust
// core/src/routing/adaptive_ttl.rs

pub struct AdaptiveTTLManager {
    /// Activity history per peer
    peer_activity: HashMap<PeerId, ActivityHistory>,
    /// Base TTL for inactive peers
    base_ttl: Duration,
    /// Maximum TTL for very active peers
    max_ttl: Duration,
}

pub struct ActivityHistory {
    /// Messages exchanged in last hour
    recent_messages: u32,
    /// Last message timestamp
    last_message: Instant,
    /// Calculated adaptive TTL
    adaptive_ttl: Duration,
}

impl AdaptiveTTLManager {
    /// Calculate TTL for a peer based on activity
    pub fn calculate_ttl(&self, peer: &PeerId) -> Duration {
        if let Some(activity) = self.peer_activity.get(peer) {
            // Active peer: longer TTL (route stays fresh)
            if activity.recent_messages > 10 {
                return self.max_ttl; // e.g., 7200s
            }
            // Moderate activity: medium TTL
            if activity.recent_messages > 2 {
                return self.base_ttl * 2; // e.g., 7200s
            }
        }
        // Inactive peer: short TTL (re-discover if needed)
        self.base_ttl // e.g., 1800s
    }
    
    /// Record message activity
    pub fn record_activity(&mut self, peer: PeerId) {
        // Update recent message count
        // Recalculate adaptive TTL
    }
}
```

**Latency Reduction:**
- Before: 2000ms re-discovery when stale route expires
- After: Route stays fresh for active peers, no re-discovery needed

---

## Implementation Priority Matrix

| Strategy | Latency Reduction | Implementation Effort | Risk | Priority |
|----------|-------------------|----------------------|------|----------|
| Hierarchical Timeout Budgeting | -300ms | Low (~100 LOC) | Low | **P0** |
| Bloom Filter Negative Cache | -200ms | Medium (~200 LOC) | Low | **P0** |
| Route Prefetch on Resume | -500ms | Medium (~150 LOC) | Medium | **P1** |
| Predictive Route Caching | -500ms | High (~300 LOC) | Medium | **P1** |
| Adaptive TTL | -300ms | Low (~100 LOC) | Low | **P2** |
| Speculative Delegate Pre-warming | -1000ms | High (~250 LOC) | High | **P2** |

---

## Verification Strategy

### Unit Tests Required

```rust
// core/tests/test_predictive_cache.rs
#[tokio::test]
async fn test_predictive_cache_hit() { ... }

#[tokio::test]
async fn test_predictive_cache_miss_triggers_refresh() { ... }

// core/tests/test_timeout_budget.rs
#[tokio::test]
async fn test_timeout_budget_phase_transitions() { ... }

#[tokio::test]
async fn test_timeout_budget_exhaustion() { ... }

// core/tests/test_negative_cache.rs
#[tokio::test]
async fn test_bloom_filter_false_positive_rate() { ... }

#[tokio::test]
async fn test_negative_cache_expiry() { ... }
```

### Integration Tests Required

```rust
// core/tests/test_dht_latency_optimization.rs
#[tokio::test]
async fn test_cold_start_latency_under_500ms() { ... }

#[tokio::test]
async fn test_warm_cache_latency_under_50ms() { ... }

#[tokio::test]
async fn test_app_resume_prefetch_completes() { ... }
```

### Performance Benchmarks

```bash
# Run before optimization (baseline)
cargo bench --bench dht_discovery -- --save-baseline before

# Run after optimization
cargo bench --bench dht_discovery -- --save-baseline after

# Compare
cargo bench --bench dht_discovery -- --baseline before --baseline after
```

---

## Cross-Platform Considerations

### iOS Specific
- Background fetch window: ~30 seconds for prefetch
- Push notification delegate: Pre-register before sleep
- BLE background modes: Limited to specific use cases

### Android Specific
- Doze mode: Batch DHT maintenance during maintenance windows
- Foreground service: Can run continuous discovery
- Battery optimization: Respect system power hints

### WASM/Web Specific
- Service Worker: Cache routes in browser storage
- Page lifecycle: Prefetch on page visibility change
- IndexedDB: Persistent route cache across sessions

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Predictive cache wrong | Medium | Low (fallback to DHT) | Confidence threshold + fallback |
| Bloom filter false positives | Low | Medium (miss reachable peer) | Conservative TTL + periodic clear |
| Prefetch wastes bandwidth | Medium | Low (background, rate-limited) | Adaptive based on network state |
| Delegate pre-warming fails | Low | High (no wake path) | Multiple delegate redundancy |

---

## Success Criteria

1. **Cold start discovery:** < 500ms (from 2000ms)
2. **Warm cache hit:** < 50ms (from 2000ms)
3. **App resume to ready:** < 200ms (from 2000ms)
4. **Unreachable peer detection:** < 10ms (from 2000ms)
5. **Network overhead:** < 2x current (acceptable for 4x latency improvement)

---

## Next Steps

1. Implement P0 items (Hierarchical Timeout Budgeting + Bloom Filter Negative Cache)
2. Benchmark cold start and warm cache latencies
3. Implement P1 items if P0 achieves < 1000ms target
4. Iterate based on real-world testing on physical devices

---

## Appendix: Current Kademlia Configuration

From `core/src/transport/behaviour.rs`:

```rust
// Apply DHT Hyper-Optimization (Alpha 8, Replication 5)
let kad_config = kad::Config::default()
    .set_parallelism(8)  // alpha = 8
    .set_replication_factor(5)
    .set_query_timeout(Duration::from_secs(5));
```

**Analysis:**
- Alpha=8 is aggressive (good for latency) but high bandwidth
- 5-second query timeout is too long for mobile (should be 500ms total budget)
- Replication factor=5 is reasonable for redundancy

---

*Document generated for Deep Architectural Reasoning prompt - SCMessenger DHT Optimization*
