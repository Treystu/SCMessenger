# T2.1 — Complete the SwarmCommand async-command seam (G5)

**Status:** completed
**Track:** 2 (Asynchronous Storage & Delay-Tolerant Networking)
**Dependencies:** T5.4
**Blocks:** T2.2

## Technical Context
- `transport/swarm.rs:1238` (`SwarmCommand` enum), `:1394` (`SwarmHandle`); stub at `iron_core.rs:3314-3318` (`update_keepalive`)
- `SwarmBridge` (`mobile_bridge.rs:2245+`) already wraps handle ops sync-over-async

## Implementation
1. Add `SwarmCommand::UpdateKeepalive { peer_id, interval }` (and audit the enum for other externally-needed-but-missing variants: per-peer disconnect, transport-pref hint from T1.7)
2. Plumb `IronCore::update_keepalive` through a held `SwarmHandle` (IronCore currently has no handle field — inject via the same wiring `SwarmBridge::set_handle` uses, `mobile_bridge.rs:2566`)
3. Remove the TODO at `iron_core.rs:3314-3318`

## Edge Cases
- Command channel full/closed (swarm shut down) must return `Err`, not block — use `try_send` with explicit error mapping
- WASM target excludes this path (`#[cfg(not(target_arch = "wasm32"))]` already present)

## Verification
- [x] `grep -n "TODO" core/src/iron_core.rs` empty
- [x] New integration test: start swarm, issue `update_keepalive`, observe keepalive change via swarm event inspection
- [x] The 7 `#[ignore]`d NAT tests in `transport/nat.rs:634-699` get a live-SwarmHandle harness and are un-ignored (or moved to the CI `--ignored` network job from T5.4 with the harness)
