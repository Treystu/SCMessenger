# Relay Peer Discovery - Integration Status
**Date:** March 10, 2026
**Status:** PROTOCOL IMPLEMENTED, NEEDS MESSAGE HANDLING

## Current Situation

### ✅ What's Working
1. **Protocol** - New message types added and compiling
2. **PeerBroadcaster** - Tracking peers and generating messages  
3. **Swarm Integration** - Broadcasting on connect/disconnect
4. **Builds** - Core, iOS, Android all build successfully

### ⚠️ What's Not Working Yet
**Peer discovery messages are being SENT but not RECEIVED/PROCESSED**

The issue: We're sending peer discovery messages via the `messaging` protocol (which is for encrypted user messages), but these are internal protocol messages that need different handling.

## The Problem

Current flow:
```
Peer A connects → OSX Relay
  → peer_broadcaster.peer_connected(A)
  → Creates PeerJoined message
  → Sends via messaging.send_request() to all other peers
  
Peer B receives → MessageRequest with peer discovery bytes
  → Treats it as encrypted user message
  → Fails to decrypt / ignores it
  → ❌ Doesn't process peer announcement
```

## The Solution

We need to either:

### Option 1: Add Special Handling in Message Receipt
When receiving a MessageRequest, check if it's a relay protocol message:
```rust
// In message handler
if let Ok(relay_msg) = RelayMessage::from_bytes(&envelope_data) {
    // Handle peer discovery
    match relay_msg {
        RelayMessage::PeerJoined { peer_info } => {
            // Dial the new peer
        }
        RelayMessage::PeerListResponse { peers } => {
            // Dial all peers
        }
        _ => {}
    }
} else {
    // Normal encrypted message handling
}
```

### Option 2: Use Separate Request/Response Protocol  
Create dedicated peer_discovery protocol in behaviour.rs similar to how ledger_exchange works.

## Recommended Fix: Option 1 (Faster)

Add peer discovery message handling where encrypted messages are received. This is in the message response handler in swarm.rs.

**Location:** Where `MessageResponse` is handled (around line ~1500-1600 in swarm.rs)

**Code to add:**
```rust
// Try to parse as relay protocol message first
if let Ok(relay_msg) = crate::relay::protocol::RelayMessage::from_bytes(&response.envelope_data) {
    match relay_msg {
        crate::relay::protocol::RelayMessage::PeerJoined { peer_info } => {
            tracing::info!("📢 Received PeerJoined: {}", peer_info.peer_id);
            for addr_str in &peer_info.addresses {
                if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                    let _ = swarm.dial(addr);
                }
            }
        }
        crate::relay::protocol::RelayMessage::PeerListResponse { peers } => {
            tracing::info!("📋 Received peer list: {} peers", peers.len());
            for peer_info in peers {
                for addr_str in &peer_info.addresses {
                    if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                        let _ = swarm.dial(addr);
                    }
                }
            }
        }
        crate::relay::protocol::RelayMessage::PeerLeft { peer_id } => {
            tracing::info!("📢 Peer left: {}", peer_id);
        }
        _ => {}
    }
    continue; // Don't process as normal message
}
// Otherwise, handle as encrypted user message...
```

## Next Steps

1. Find MessageResponse handler in swarm.rs
2. Add relay message parsing/handling  
3. Test that peers now discover each other
4. Verify cross-network messaging works

