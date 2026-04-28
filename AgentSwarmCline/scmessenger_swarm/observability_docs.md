# Observability Module Documentation - SCMessenger v0.2.1

## Module-Level Documentation (`//!`)

```rust
//! SCMessenger v0.2.1 Observability Component
//! 
//! This module implements the structured tracing payload for mandatory relay protocol events.
//! It provides the `RelayTracePayload` struct that captures essential telemetry data about
//! message processing through relay nodes.
//!
//! ### Architecture Position
//! The observability module acts as a data contract between the relay nodes and the 
//! telemetry pipeline. By providing a strictly validated structure, it ensures that 
//! downstream aggregators (Loki, Elasticsearch) receive consistent schema data for 
//! latency analysis and message routing audits.
//!
//! To avoid circular dependencies, this module provides the *data structures* for 
//! observability but does not implement the *transport* (e.g., it does not call 
//! `tracing::info!`). Integration with the `tracing` ecosystem is handled by the 
//! consuming subsystems.
```

## Item-Level Documentation (`///`)

### `RelayTracePayload`
```rust
/// Represents a structured tracing payload for mandatory relay protocol events.
/// 
/// This struct captures key telemetry information about how messages traverse the SCMessenger
/// relay network, including identification, node information, and performance metrics.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayTracePayload {
    /// The unique identifier of the SCMessenger payload traversing the relay.
    /// Must not be empty.
    pub message_id: String,
    
    /// The cryptographic hash identifying the specific relay node processing the message.
    /// Must be a 64-character SHA-256 hex string.
    pub relay_node_hash: String,
    
    /// The processing latency introduced by the relay node, measured in milliseconds.
    /// `0` is permissible for local loopback scenarios.
    pub latency_ms: u64,
}
```

#### Methods & Implementations
- **`validate()`**: 
  - `/// Validates the integrity and correctness of the relay trace payload.`
  - Returns `Ok(())` if validation rules pass, otherwise returns a `RelayTraceError`.
  - **Validation Rules**: 
    1. `message_id` must not be empty.
    2. `relay_node_hash` must be exactly 64 hexadecimal characters.
- **`fmt::Display`**: 
  - Formats the payload as: `[RelayTrace] msg_id={message_id}, node={relay_node_hash}, latency={latency_ms}ms`.

### `RelayTraceError`
```rust
/// Error types that can occur during validation of relay trace payloads.
#[derive(Debug)]
pub enum RelayTraceError {
    /// The message ID field was empty.
    EmptyMessageId,
    
    /// The relay node hash was invalid (empty, wrong length, or non-hex characters).
    InvalidNodeHash,
}
```

---

## Usage Examples

### Basic Construction and Validation
```rust
use crate::observability::{RelayTracePayload, RelayTraceError};

fn main() -> Result<(), RelayTraceError> {
    let payload = RelayTracePayload {
        message_id: "msg_12345".to_string(),
        relay_node_hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
        latency_ms: 12,
    };

    // Validate payload before transmission
    payload.validate()?;
    
    // Print via Display implementation
    println!("{}", payload); 
    // Output: [RelayTrace] msg_id=msg_12345, node=a1b2..., latency=12ms
    
    Ok(())
}
```

---

## JSON Serialization Examples

Because of the `#[serde(rename_all = "snake_case")]` attribute, the fields are serialized to JSON using snake_case.

### Example JSON Output
```json
{
  "message_id": "msg_xyz_999",
  "relay_node_hash": "ef797c811a3937bac572a227750d23586d8f7a8f17386e04f871ae9e7b08996d",
  "latency_ms": 45
}
```

### Round-trip Serialization
```rust
let payload = RelayTracePayload { /* ... */ };
let json = serde_json::to_string(&payload).unwrap();
let deserialized: RelayTracePayload = serde_json::from_str(&json).unwrap();
assert_eq!(payload, deserialized);
```

---

## Integration Guidance

### 1. Tracing Crate Integration
To emit these payloads as structured events, use the `tracing` crate in combination with `serde_json`.

**Option A: Structured Fields (Recommended)**
```rust
use tracing::info;

let payload = RelayTracePayload { ... };
info!(
    message_id = %payload.message_id,
    relay_node_hash = %payload.relay_node_hash,
    latency_ms = payload.latency_ms,
    "Relay single-hop processing complete"
);
```

**Option B: JSON Blob**
```rust
let payload_json = serde_json::to_string(&payload).unwrap();
tracing::info!(payload = %payload_json, "Relay trace emitted");
```

### 2. Grafana / Loki Integration
Loki captures the log lines. Since the payload is serialized as JSON, use the `json` parser in LogQL.

**Filter by high latency (> 100ms):**
```logql
{app="scm-relay"} | json | latency_ms > 100
```

**Search for a specific message journey:**
```logql
{app="scm-relay"} | json | message_id = "msg_12345"
```

**Aggregate average latency per node:**
```logql
avg_over_time({app="scm-relay"} | json | unwrap latency_ms [5m])
```

### 3. Elasticsearch Integration
To ensure efficient querying, use the following index mapping template:

| Field | Elasticsearch Type | Reason |
| :--- | :--- | :--- |
| `message_id` | `keyword` | Exact match lookups for message auditing. |
| `relay_node_hash` | `keyword` | Exact match for node performance tracking. |
| `latency_ms` | `unsigned_long` | Enables range queries and aggregations. |

**Example Kibana Query (KQL):**
`relay_node_hash: "ef79..." AND latency_ms > 50`
