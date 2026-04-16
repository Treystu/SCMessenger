# P1_CORE_001: Drift Protocol Activation

**Priority:** P1 (Core Functionality)
**Platform:** Core/Rust
**Status:** Dormant (Not Wired)
**Source:** PRODUCTION_ROADMAP.md - Module Status Matrix

## Problem Description
Drift Protocol (8 files in `core/src/drift/`) is fully unit-tested but completely dormant - not wired to production path. This includes gossip protocols, store-and-relay, and encounter quality tracking.

## Impact
- Missing store-and-forward capabilities
- No encounter-based message delivery optimization
- Wasted development investment in tested code
- Reduced mesh resilience

## Implementation Required
1. Wire drift modules into `SwarmHandle` dispatch
2. Integrate with transport manager for message routing
3. Activate gossip protocols and encounter tracking
4. Connect to production message flow

## Key Files
- `core/src/drift/mod.rs` - Main module wiring
- `core/src/transport/manager.rs` - Integration point
- `cli/src/main.rs` - CLI activation

## Expected Outcome
- Drift protocol active in production
- Store-and-forward capabilities enabled
- Encounter quality optimization
- Enhanced mesh resilience