# P0_WASM_002: Thin Client Completion - Peripheral BLE & UI Lobotomy

**Priority:** P0 CRITICAL
**Platform:** WASM
**Status:** Open
**Source:** REMAINING_WORK_TRACKING.md - WASM thin client section
**Routing Tags:** [REQUIRES: BLE_INTEGRATION] [REQUIRES: FINALIZATION]
**[NATIVE_SUB_AGENT: RESEARCH]** [NATIVE_SUB_AGENT: LINT_FORMAT]

## Objective
Complete the WASM thin client implementation with peripheral BLE advertising and full UI lobotomy (daemon-driven operation).

## Current State (From REMAINING_WORK_TRACKING.md)
✅ **Partial landing 2026-04-11**

### 1. BLE GATT Peripheral Implementation
- btleplug-based advertising for SCM service UUID
- Mobile discovery of desktop daemon capability
- Central scan + Drift notify → `message_received` implemented
- Located in `cli/src/ble_mesh.rs` (notify char `0xDF03`)

### 2. Browser IronCore (Missing)
- Optional runtime mode that refuses local `initialize_identity`
- Requires `get_identity` over daemon socket first
- Routes `prepare_message` / send exclusively through JSON-RPC
- Partial implementation: `wasm/src/daemon_bridge.rs` wire helpers + `transport.rs` direction comments
- **Full gating not landed**

### 3. Verification (Missing)
- `wasm-pack test --headless` browser harness
- Daemon-driven UI state testing
- Deferred where CI lacks headless browser

## Implementation Plan

### 1. Complete Browser IronCore Mode
**File:** `wasm/src/lib.rs`
```rust
pub fn initialize_ironcore_mode(daemon_socket_url: String) -> Result<(), JsError> {
    // Refuse local identity initialization
    // Require identity retrieval from daemon
    // Route all operations through JSON-RPC
}
```

### 2. Finish Daemon Bridge Integration
**File:** `wasm/src/daemon_bridge.rs`
- Complete JSON-RPC client implementation
- Add proper error handling and reconnection logic
- Implement message routing gating

### 3. Enhance Transport Layer
**File:** `wasm/src/transport.rs`
- Add IronCore mode detection and routing
- Implement fallback to local mode when daemon unavailable
- Ensure proper WebSocket connection management

### 4. Add Verification Tests
**File:** `tests/daemon_integration_test.rs`
- Create headless browser test harness
- Test daemon-driven operation scenarios
- Verify JSON-RPC message routing

## Success Criteria
- ✅ WASM client can operate in daemon-only mode
- ✅ BLE peripheral advertising works for mobile discovery
- ✅ All message operations route through daemon when in IronCore mode
- ✅ Proper error handling for daemon connectivity issues
- ✅ Headless test verification working

## Priority: CRITICAL
Required for complete WASM transport parity and production readiness.