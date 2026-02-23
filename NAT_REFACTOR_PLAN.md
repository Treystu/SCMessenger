> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# NAT Traversal Refactoring Plan
**Date:** 2026-02-09
**Issue:** External STUN server dependencies violate sovereign mesh architecture
**Solution:** Peer-assisted address discovery + in-mesh reflection service

---

## [Needs Revalidation] Problem Statement

Current `nat.rs` implementation (~884 LoC) includes hardcoded Google STUN servers:
- `stun.l.google.com:19302`
- `stun1.l.google.com:19302`
- `stun2.l.google.com:19302`

**This violates core principle #2:** "Every node IS the network. No third-party relays, no external infrastructure."

---

## [Needs Revalidation] Architecture Decision (User Input)

1. **Address Discovery:** Peer-assisted (mesh nodes report observed external address)
2. **NAT-to-NAT:** Try hole-punching first, fallback to relay circuits
3. **Implementation:** Convert to in-mesh address reflection service (~200 LoC refactor)
4. **Relay Strategy:** Web deploys are prime relay candidates (stable IP, better resources)

---

## [Needs Revalidation] Refactoring Approach

### [Needs Revalidation] Phase 1: Remove External Dependencies (~50 LoC changes)
**File:** `core/src/transport/nat.rs`

**Changes:**
1. Remove `NatProbe` struct entirely (lines 68-219)
   - **Delete:** ~150 LoC of STUN protocol code
   - **Reason:** No external STUN servers allowed

2. Update `NatConfig::default()` (lines 303-317)
   ```rust
   // REMOVE:
   stun_servers: vec![
       "stun.l.google.com:19302".to_string(),
       ...
   ],

   // REPLACE WITH:
   peer_address_reflectors: vec![],  // Populated dynamically from mesh
   ```

3. Remove `stun_servers` field from `NatConfig` struct (line 289)
   ```rust
   // REMOVE: pub stun_servers: Vec<String>,
   // ADD: pub peer_address_reflectors: Vec<PeerId>,
   ```

### [Needs Revalidation] Phase 2: Add Peer-Assisted Discovery (~150 LoC addition)
**File:** `core/src/transport/nat.rs`

**New struct:**
```rust
/// Peer-assisted address discovery
///
/// Uses other mesh nodes to discover external address.
/// Protocol:
/// 1. Send "what's my address?" request to mesh peers
/// 2. Peers respond with observed source IP:port
/// 3. Aggregate responses to determine external address
pub struct PeerAddressDiscovery {
    /// Known mesh peers who can report our address
    reflector_peers: Vec<PeerId>,
    /// Timeout for address discovery
    timeout_secs: u64,
    /// Minimum peer responses needed for consensus
    min_responses: u32,
}

impl PeerAddressDiscovery {
    /// Discover external address by asking mesh peers
    pub async fn discover_external_address(
        &self,
        swarm: &mut libp2p::Swarm<...>,
    ) -> Result<SocketAddr, NatTraversalError> {
        // Implementation:
        // 1. Send libp2p request-response to multiple peers
        // 2. Each peer responds with observed source address
        // 3. Aggregate responses (majority vote)
        // 4. Return consensus external address
    }
}
```

**Message protocol:**
```rust
/// Address reflection request (sent to peer)
#[derive(Serialize, Deserialize)]
pub struct AddressReflectionRequest {
    /// Request ID
    pub request_id: [u8; 16],
}

/// Address reflection response (from peer)
#[derive(Serialize, Deserialize)]
pub struct AddressReflectionResponse {
    /// Request ID (echo)
    pub request_id: [u8; 16],
    /// Observed source address of requester
    pub observed_address: SocketAddr,
}
```

### [Needs Revalidation] Phase 3: Update `NatTraversal` (~50 LoC changes)
**File:** `core/src/transport/nat.rs`

**Changes to `probe_nat()` method (lines 350-362):**
```rust
// OLD:
pub async fn probe_nat(&self) -> Result<NatType, NatTraversalError> {
    let probe = NatProbe::with_servers(self.config.stun_servers.clone(), ...);
    let nat_type = probe.detect_nat_type().await?;
    let external_addr = probe.get_external_address().await?;
    ...
}

// NEW:
pub async fn probe_nat(
    &self,
    swarm: &mut libp2p::Swarm<...>,
) -> Result<NatType, NatTraversalError> {
    let discovery = PeerAddressDiscovery::new(
        self.config.peer_address_reflectors.clone(),
        self.config.attempt_timeout,
    );
    let external_addr = discovery.discover_external_address(swarm).await?;

    // Determine NAT type by querying multiple peers
    // and checking for address/port consistency
    ...
}
```

### [Needs Revalidation] Phase 4: Add In-Mesh Reflection Service (~50 LoC addition)
**File:** `core/src/transport/internet.rs` or new `core/src/transport/reflection.rs`

**New service:**
```rust
/// Address reflection service
///
/// Each node provides this service to help other nodes
/// discover their external address. This is the in-mesh
/// equivalent of a STUN server.
pub struct AddressReflectionService {
    /// How many reflection requests served
    pub requests_served: Arc<AtomicU64>,
}

impl AddressReflectionService {
    /// Handle address reflection request
    ///
    /// Simply tells the requester what their observed
    /// source address is from our perspective.
    pub fn handle_request(
        &self,
        request: AddressReflectionRequest,
        observed_addr: SocketAddr,
    ) -> AddressReflectionResponse {
        self.requests_served.fetch_add(1, Ordering::Relaxed);

        AddressReflectionResponse {
            request_id: request.request_id,
            observed_address: observed_addr,
        }
    }
}
```

### [Needs Revalidation] Phase 5: Update Documentation (~100 LoC changes)

**Files to update:**
1. `ENHANCEMENTS_SUMMARY.md` - Remove STUN server references
2. `VERIFICATION_REPORT.md` - Update production readiness assessment
3. `NAT_REFACTOR_PLAN.md` (this file) - Document final implementation
4. Comments in `nat.rs` - Remove RFC 5389 references, add peer protocol docs

---

## [Needs Revalidation] Implementation Estimates

| Task | LoC Change | Complexity |
|------|-----------|------------|
| Remove external STUN code | -150 LoC | Low |
| Remove STUN config | -20 LoC | Low |
| Add peer discovery struct | +150 LoC | Medium |
| Add reflection service | +50 LoC | Low |
| Update NatTraversal | +50 LoC | Medium |
| Add message protocol types | +30 LoC | Low |
| Update documentation | ~100 LoC | Low |
| **NET CHANGE** | **~+110 LoC** | **Medium** |

**Effort:** ~200 LoC of changes (some additions, some deletions, some modifications)

---

## [Needs Revalidation] Testing Strategy

### [Needs Revalidation] Unit Tests to Update
1. `test_nat_probe_creation()` → Replace with `test_peer_discovery_creation()`
2. `test_nat_probe_custom_servers()` → Replace with `test_peer_discovery_custom_reflectors()`
3. `test_detect_nat_type()` → Update to use mock peer responses
4. `test_get_external_address()` → Update to use peer-assisted discovery

### [Needs Revalidation] New Integration Tests Needed
1. Test address reflection service (request → response)
2. Test peer discovery with multiple reflectors
3. Test consensus algorithm (majority vote)
4. Test fallback when insufficient peer responses

**Test LoC:** ~150 LoC of test updates/additions

---

## [Needs Revalidation] Migration Notes

### [Needs Revalidation] For Existing Code
- Any code calling `probe_nat()` needs to pass `swarm` handle
- `NatConfig` construction needs peer IDs instead of STUN servers
- Address discovery requires at least 3 connected mesh peers

### [Needs Revalidation] For Deployment
- Bootstrap nodes should be configured as address reflectors
- Web deploys (browser nodes) are excellent relay candidates
- Mobile nodes can also serve as reflectors when online

---

## [Needs Revalidation] Benefits of Peer-Assisted Approach

1. **Zero external dependencies** - Fully sovereign
2. **More resilient** - Not dependent on single STUN server infrastructure
3. **Privacy-preserving** - Address discovery stays within mesh
4. **Faster** - No DNS resolution, direct peer communication
5. **More accurate** - Peers see actual routed address
6. **Distributed load** - Every node can provide reflection service

---

## [Needs Revalidation] Open Questions

1. **Bootstrap problem:** How does first node discover its address?
   - **Answer:** First node assumes it's publicly reachable OR waits for second node to join mesh

2. **Consensus algorithm:** What if peers report different addresses?
   - **Answer:** Majority vote. If symmetric NAT, different peers may see different ports - use most common address

3. **Reflection service abuse:** Can attackers spam reflection requests?
   - **Answer:** Rate limit per peer ID. Relay policy already handles abuse prevention.

---

## [Needs Revalidation] Implementation Priority

**Priority 1 (Critical):** Remove external dependencies
- Delete Google STUN server references
- Update NatConfig to not default to external servers

**Priority 2 (High):** Add peer discovery
- Implement PeerAddressDiscovery
- Update probe_nat() method

**Priority 3 (Medium):** Add reflection service
- Implement AddressReflectionService
- Wire into swarm behavior

**Priority 4 (Low):** Update tests and docs
- Update all test cases
- Clean up documentation

---

## [Needs Revalidation] Next Steps

1. ✅ Create this refactoring plan
2. ⏳ Remove external STUN dependencies (Priority 1)
3. ⏳ Implement peer-assisted discovery (Priority 2)
4. ⏳ Add reflection service (Priority 3)
5. ⏳ Update tests and documentation (Priority 4)
6. ⏳ Commit all changes with clear description

---

**Status:** Planning Complete - Ready for Implementation
**Estimated Total:** ~200 LoC refactor + ~150 LoC tests = ~350 LoC
