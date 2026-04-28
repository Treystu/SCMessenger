// WASM / browser daemon bridge shared types (serde JSON-RPC).
//
// Historical mesh/storage/transport helpers live under `mesh.rs`, `storage.rs`, and
// `transport.rs` but are not wired into `lib.rs` until they are updated for current libp2p.

pub mod rpc;
