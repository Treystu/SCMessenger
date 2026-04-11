# SCMessenger

**SCMessenger** (Sovereign Encrypted Messaging) is a highly resilient, cross-platform decentralized messaging mesh handling secure communications. Built with cutting-edge peer-to-peer technologies, it bypasses traditional central servers, utilizing BLE, mDNS, LAN, and Quic/TCP Relay circuits to ensure instantaneous delivery under any network condition.

## Overview
SCMessenger is architected for total sovereignty and uncompromised privacy. It features:
- **Rust Core**: A highly secure, multi-transport headless mesh daemon using `libp2p`.
- **WASM Thin-Client Web UI**: A stunning browser-based interface running over a strict `localhost` multiplexed JSON-RPC WebSocket Bridge to perfectly sandbox cryptographic isolation.
- **Native Android & iOS Clients**: Full-featured smart transport routers that elegantly fallback through Multipeer, Wi-Fi Direct, BLE, mDNS, and Internet Relay based on real-time sub-500ms connectivity races.
- **Resilient Transport Matrix**: Automatic Transport path determination delivering messages whether on an airplane, in a crowded stadium, or on a cellular network traversing strict NATs using resilient UDP/QUIC forwarding.



## Getting Started

### 1. The Headless Daemon / Desktop CLI
The daemon handles all identity custody, transport logic, and cryptographic signing.
```sh
# Clone the repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger/cli

# Run the local daemon & bridge server
cargo run --release -- start
```
*The bridge binds strictly to localhost (127.0.0.1:9002) for security.*

### 2. Browser WebUI (WASM)
Connects directly to the Daemon via JSON-RPC.
```sh
# Build the UI
cd SCMessenger/wasm
wasm-pack build --target web
```
Open `http://localhost:9000` via the CLI embedded server to launch the unified Material 3 dashboard.

### 3. Native Mobile (iOS & Android)
- **Android:** Open the `android/` directory in Android Studio and run.
- **iOS:** Open `iOS/SCMessenger/SCMessenger.xcworkspace` in Xcode (requires macOS).

## Architecture & Documentation
For complete details on SCMessenger's architecture, peer block state-machines, ID management, and Smart Transport routing, review the `docs/` directory.

- `AGENTS.md` and `DOCUMENTATION.md` for contributor guidelines.
- `docs/CURRENT_STATE.md` for live node tracking and engineering status.

## Contributing
Please refer to the documentation to ensure you align with our required repository-scoped extraction and synchronization standards. Build verifications are strictly enforced across Android, iOS, WASM, and CLI boundaries.

## License
The Unlicense
