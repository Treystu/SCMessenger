# P1_CORE_002: Mycorrhizal Routing Activation

**Priority:** P1 (Core Functionality)
**Platform:** Core/Rust
**Status:** Dormant (Not Wired)
**Source:** PRODUCTION_ROADMAP.md - Module Status Matrix

## Problem Description
Mycorrhizal Routing (10 files in `core/src/routing/`) is fully unit-tested but completely dormant - not wired to production. Includes path selection, quality scoring, and transport optimization.

## Impact
- Missing intelligent path selection
- No transport quality optimization
- Reduced delivery efficiency
- Wasted routing capabilities

## Implementation Required
1. Wire routing modules into transport dispatch
2. Integrate path selection with message delivery
3. Activate transport quality scoring
4. Connect to swarm management

## Key Files
- `core/src/routing/mod.rs` - Main module wiring
- `core/src/transport/manager.rs` - Integration
- `core/src/swarm.rs` - Swarm integration

## Expected Outcome
- Intelligent path selection active
- Transport quality optimization
- Enhanced delivery efficiency
- Optimal route utilization