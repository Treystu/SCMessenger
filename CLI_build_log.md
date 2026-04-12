PS C:\Users\kanal\Documents\SCMessenger\SCMessenger> cargo run -p scmessenger-cli -- start
   Compiling scmessenger-core v0.2.1 (C:\Users\kanal\Documents\SCMessenger\SCMessenger\core)
   Compiling scmessenger-cli v0.2.1 (C:\Users\kanal\Documents\SCMessenger\SCMessenger\cli)
warning: unused variable: `cli_cap`
   --> cli\src\transport_bridge.rs:338:51
    |
338 |                 self.cli_capabilities.iter().any(|cli_cap| {
    |                                                   ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_cli_cap`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `dest_cap`
   --> cli\src\transport_bridge.rs:337:30
    |
337 |             caps.iter().any(|dest_cap| {
    |                              ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_dest_cap`

warning: method `transport_bridge` is never used
   --> cli\src\server.rs:150:12
    |
148 | impl WebContext {
    | --------------- method in this implementation
149 |     /// Get transport bridge reference (for future API activation)
150 |     pub fn transport_bridge(&self) -> &Arc<tokio::sync::Mutex<crate::transport_bridge::TransportBridge>> {
    |            ^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `TransportCapabilitiesResponse` is never constructed
  --> cli\src\transport_api.rs:23:12
   |
23 | pub struct TransportCapabilitiesResponse {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `TransportPathsResponse` is never constructed
  --> cli\src\transport_api.rs:30:12
   |
30 | pub struct TransportPathsResponse {
   |            ^^^^^^^^^^^^^^^^^^^^^^

warning: variant `InvalidCapabilities` is never constructed
  --> cli\src\transport_api.rs:38:5
   |
36 | pub enum TransportError {
   |          -------------- variant in this enum
37 |     InvalidPeerId,
38 |     InvalidCapabilities,
   |     ^^^^^^^^^^^^^^^^^^^
   |
   = note: `TransportError` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: function `transport_routes` is never used
  --> cli\src\transport_api.rs:52:8
   |
52 | pub fn transport_routes(
   |        ^^^^^^^^^^^^^^^^

warning: function `handle_transport_capabilities` is never used
  --> cli\src\transport_api.rs:87:10
   |
87 | async fn handle_transport_capabilities(
   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `handle_transport_paths` is never used
   --> cli\src\transport_api.rs:113:10
    |
113 | async fn handle_transport_paths(
    |          ^^^^^^^^^^^^^^^^^^^^^^

warning: function `handle_register_peer` is never used
   --> cli\src\transport_api.rs:134:10
    |
134 | async fn handle_register_peer(
    |          ^^^^^^^^^^^^^^^^^^^^

warning: fields `wasm_peer_id`, `active_paths`, and `api_context` are never read
  --> cli\src\transport_bridge.rs:52:5
   |
50 | pub struct TransportBridge {
   |            --------------- fields in this struct
51 |     /// WASM peer ID (browser client)
52 |     wasm_peer_id: Option<PeerId>,
   |     ^^^^^^^^^^^^
...
58 |     active_paths: HashMap<PeerId, TransportPath>,
   |     ^^^^^^^^^^^^
...
62 |     api_context: Option<Arc<Mutex<ApiContext>>>,
   |     ^^^^^^^^^^^

warning: field `failure_count` is never read
  --> cli\src\transport_bridge.rs:69:5
   |
67 | struct PathStatistics {
   |        -------------- field in this struct
68 |     success_count: u32,
69 |     failure_count: u32,
   |     ^^^^^^^^^^^^^
   |
   = note: `PathStatistics` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: multiple methods are never used
   --> cli\src\transport_bridge.rs:106:12
    |
 92 | impl TransportBridge {
    | -------------------- methods in this implementation
...
106 |     pub fn with_api_context(mut self, ctx: Arc<Mutex<ApiContext>>) -> Self {
    |            ^^^^^^^^^^^^^^^^
...
112 |     pub fn set_wasm_peer(&mut self, peer_id: PeerId) {
    |            ^^^^^^^^^^^^^
...
182 |     pub fn find_best_path(&self, peer_id: &PeerId) -> Option<TransportPath> {
    |            ^^^^^^^^^^^^^^
...
256 |     pub fn update_path_stats(&mut self, path: &TransportPath, success: bool, latency: u32) {
    |            ^^^^^^^^^^^^^^^^^
...
277 |     pub fn get_available_paths(&self) -> HashMap<PeerId, Vec<TransportPath>> {
    |            ^^^^^^^^^^^^^^^^^^^
...
296 |     pub fn get_peer_capabilities(&self, peer_id: &PeerId) -> Option<&[TransportType]> {
    |            ^^^^^^^^^^^^^^^^^^^^^
...
316 |     pub fn can_forward_for_wasm(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
...
322 |     pub fn get_forwarding_capability(&self, request_type: &str) -> Option<TransportType> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
...
334 |     pub fn can_reach_destination(&self, peer_id: &PeerId) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^
...
349 |     pub fn get_best_forwarding_path(&self, peer_id: &PeerId) -> Option<TransportPath> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `TransportRoute` is never constructed
   --> cli\src\transport_bridge.rs:361:12
    |
361 | pub struct TransportRoute {
    |            ^^^^^^^^^^^^^^

warning: `scmessenger-cli` (bin "scmessenger-cli") generated 14 warnings (run `cargo fix --bin "scmessenger-cli" -p scmessenger-cli` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 56.23s