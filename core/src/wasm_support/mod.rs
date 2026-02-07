// WASM Client Support — Phase 8
//
// Portable browser-compatible mesh node implementation for WASM environments.
// Includes WebRTC/WebSocket transport, in-memory storage, and mesh synchronization.

pub mod mesh;
pub mod storage;
pub mod transport;

pub use mesh::{WasmMeshConfig, WasmMeshNode, WasmMeshState};
pub use storage::{EvictionStrategy, WasmStore, WasmStoreConfig};
pub use transport::{WebSocketRelay, WebTransportConfig, WebTransportManager, WebTransportType};
