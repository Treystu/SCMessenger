# SwarmBridge Integration Guide

## Overview

The `SwarmBridge` in `core/src/mobile_bridge.rs` provides a synchronous wrapper around the async `SwarmHandle` for mobile platform integration via UniFFI.

**Status:** ✅ **WIRED** (as of Feb 2026)

## Architecture

```
Mobile App (Android/iOS)
    ↓ UniFFI
SwarmBridge (mobile_bridge.rs)
    ↓ parking_lot::Mutex + tokio::runtime::Handle
SwarmHandle (transport/swarm.rs)
    ↓ mpsc channel
Swarm Task (async event loop)
```

## Integration Steps

### 1. Create and Start Swarm in Rust

```rust
use scmessenger_core::transport::swarm::start_swarm;
use libp2p::identity::Keypair;
use tokio::sync::mpsc;

// In your async runtime
let keypair = Keypair::generate_ed25519();
let listen_addr = Some("/ip4/0.0.0.0/tcp/0".parse().unwrap());
let (event_tx, mut event_rx) = mpsc::channel(100);

// Start the swarm
let swarm_handle = start_swarm(keypair, listen_addr, event_tx).await?;
```

### 2. Wire SwarmBridge to SwarmHandle

```rust
use scmessenger_core::mobile_bridge::SwarmBridge;

// Create or get existing SwarmBridge
let bridge = SwarmBridge::new();

// Wire it to the SwarmHandle
bridge.set_handle(swarm_handle);
```

### 3. Use from Mobile Platform

#### Android/Kotlin Example

```kotlin
import uniffi.api.*

class MeshRepository(context: Context) {
    private var swarmBridge: SwarmBridge? = null
    
    fun startNetwork() {
        // SwarmBridge is created via UniFFI
        swarmBridge = SwarmBridge()
        
        // In Rust code, you would wire this to an actual SwarmHandle
        // For now, mobile platforms need to trigger Rust-side wiring
    }
    
    suspend fun sendMessage(peerId: String, data: ByteArray) = 
        withContext(Dispatchers.IO) {
            swarmBridge?.sendMessage(peerId, data)
        }
    
    suspend fun dialPeer(multiaddr: String) = 
        withContext(Dispatchers.IO) {
            swarmBridge?.dial(multiaddr)
        }
    
    fun getConnectedPeers(): List<String> {
        return swarmBridge?.getPeers() ?: emptyList()
    }
    
    fun getSubscribedTopics(): List<String> {
        return swarmBridge?.getTopics() ?: emptyList()
    }
    
    suspend fun subscribeToTopic(topic: String) = 
        withContext(Dispatchers.IO) {
            swarmBridge?.subscribeTopic(topic)
        }
    
    fun shutdown() {
        swarmBridge?.shutdown()
    }
}
```

#### iOS/Swift Example

```swift
import scmessenger_mobile

class MeshService {
    private var swarmBridge: SwarmBridge?
    
    func startNetwork() {
        swarmBridge = SwarmBridge()
        // In Rust code, you would wire this to an actual SwarmHandle
    }
    
    func sendMessage(peerId: String, data: Data) throws {
        try swarmBridge?.sendMessage(peerId: peerId, data: Array(data))
    }
    
    func dialPeer(multiaddr: String) throws {
        try swarmBridge?.dial(multiaddr: multiaddr)
    }
    
    func getConnectedPeers() -> [String] {
        return swarmBridge?.getPeers() ?? []
    }
    
    func getSubscribedTopics() -> [String] {
        return swarmBridge?.getTopics() ?? []
    }
    
    func subscribeToTopic(topic: String) throws {
        try swarmBridge?.subscribeTopic(topic: topic)
    }
    
    func shutdown() {
        swarmBridge?.shutdown()
    }
}
```

## Implementation Details

### Synchronous Bridge Pattern

SwarmBridge uses `tokio::runtime::Handle::block_on()` to bridge synchronous UniFFI calls to async SwarmHandle:

```rust
pub fn send_message(&self, peer_id: String, data: Vec<u8>) -> Result<(), IronCoreError> {
    let handle = self.handle.lock().as_ref()
        .ok_or(IronCoreError::NetworkError)?;
    
    let peer_id = PeerId::from_str(&peer_id)
        .map_err(|_| IronCoreError::InvalidInput)?;
    
    if let Some(rt) = &self.runtime_handle {
        rt.block_on(handle.send_message(peer_id, data))
            .map_err(|_| IronCoreError::NetworkError)
    } else {
        Err(IronCoreError::Internal)
    }
}
```

### Thread Safety

- `SwarmHandle` is wrapped in `Arc<Mutex<Option<SwarmHandle>>>`
- Multiple mobile threads can safely call SwarmBridge methods
- Internal `tokio::runtime::Handle` ensures async operations run in correct context

### Error Handling

SwarmBridge returns `IronCoreError` variants:
- `NotInitialized` - SwarmBridge created but handle not set
- `NetworkError` - Network operation failed
- `InvalidInput` - Invalid peer ID or multiaddr format
- `Internal` - Runtime handle not available

## Current Limitations

1. **No Event Callbacks**: SwarmBridge doesn't yet expose incoming message events to mobile. Events are currently only sent via `event_tx` channel, which is Rust-side only.

2. **Blocking Semantics**: Using `block_on()` means mobile UI threads should wrap calls in background tasks.

3. **Single Swarm**: Currently assumes one swarm per process. Multi-swarm support would require indexed bridges.

## Recommended Mobile Architecture

```
┌─────────────────────────────────────────┐
│         Mobile UI Layer                 │
│  (Activities, ViewModels, Composables)  │
└──────────────┬──────────────────────────┘
               │
┌──────────────┴──────────────────────────┐
│       MeshRepository                    │
│  (Kotlin/Swift - manages UniFFI)        │
│                                          │
│  - SwarmBridge (network ops)            │
│  - IronCore (crypto/identity)           │
│  - HistoryManager (message storage)     │
│  - ContactManager (contacts)            │
└──────────────┬──────────────────────────┘
               │ UniFFI boundary
┌──────────────┴──────────────────────────┐
│         Rust Core                       │
│  - SwarmBridge → SwarmHandle            │
│  - libp2p network stack                 │
│  - Drift protocol                       │
│  - Encryption/signatures                │
└─────────────────────────────────────────┘
```

## Testing

### Unit Tests

```rust
#[test]
fn test_swarm_bridge_creation() {
    let bridge = SwarmBridge::new();
    assert_eq!(bridge.get_peers().len(), 0);
    assert_eq!(bridge.get_topics().len(), 0);
    bridge.shutdown(); // Should not panic
}
```

### Integration Tests

For full integration testing, you need:
1. A tokio runtime
2. An actual SwarmHandle from `start_swarm()`
3. SwarmBridge wired to that handle

Example:

```rust
#[tokio::test]
async fn test_swarm_bridge_integration() {
    let keypair = Keypair::generate_ed25519();
    let (tx, _rx) = mpsc::channel(10);
    
    let handle = start_swarm(keypair, None, tx).await.unwrap();
    let bridge = SwarmBridge::new();
    bridge.set_handle(handle);
    
    // Now bridge methods should work
    let peers = bridge.get_peers();
    assert_eq!(peers.len(), 0); // No peers connected yet
}
```

## Migration Path for Existing Android Code

The Android app currently creates `SwarmBridge()` but it was a stub. With the new implementation:

**Before (stub):**
```kotlin
swarmBridge = uniffi.api.SwarmBridge()
swarmBridge?.sendMessage(peerId, data) // Did nothing
```

**After (wired):**
```kotlin
// Same Kotlin code, but now it actually works!
swarmBridge = uniffi.api.SwarmBridge()
// Rust side needs to call bridge.set_handle() to wire it up
swarmBridge?.sendMessage(peerId, data) // Actually sends via libp2p
```

## Next Steps

1. ✅ SwarmBridge basic wiring (DONE)
2. 🔲 Add MeshService integration to automatically wire SwarmBridge on start
3. 🔲 Expose incoming message events via CoreDelegate callbacks
4. 🔲 Add peer discovery events to mobile platforms
5. 🔲 Implement bandwidth/relay statistics tracking in SwarmBridge
6. 🔲 Add connection quality metrics for mobile UI

## See Also

- `core/src/mobile_bridge.rs` - SwarmBridge implementation
- `core/src/transport/swarm.rs` - SwarmHandle async implementation
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Android integration
- `cli/src/api.rs` - Reference implementation for CLI integration
