# SCMessenger CLI Reference Guide

Use this guide to look up exact command patterns. When driving the CLI via `core_cli_driver.py`, use the `raw` command for any function not explicitly listed in your primary command table.

## 1. Identity Management
- **Initialize**: `scm init [--name <nickname>]`
- **Show Identity**: `scm identity show`
- **Set Nickname**: `scm identity set-name <name>`
- **Show Device ID**: `scm identity device-id`
- **Export Backup**: `scm identity export --passphrase <secret> [--output <file>]`
- **Import Backup**: `scm identity import --passphrase <secret> [--backup <payload> | --input <file>]`

## 2. Contact Management
- **Add Contact**: `scm contact add <peer_id> <public_key> [--name <nickname>]`
  - *Note: PeerID starts with 12D3Koo. Public Key is 64 hex chars.*
- **List Contacts**: `scm contact list`
- **Remove Contact**: `scm contact remove <contact_name_or_id>`
- **Search**: `scm contact search <query>`
- **Set Nickname**: `scm contact set-nickname <contact> <nickname>`

## 3. Node Control
- **Start Node**: `scm start [--port <p2p_port>]`
- **Stop Node**: `scm stop`
- **Status**: `scm status`
- **Headless Relay**: `scm relay [--listen <multiaddr>] [--http-port <port>] [--name <name>]`

## 4. Messaging & History
- **Send Message**: `scm send <recipient_id> "<message>"`
- **View History**: `scm history [--peer <peer_id>] [--limit <n>] [--search <query>]`
- **Clear History**: `scm history-clear --yes`
- **Prune History**: `scm history-prune-before <unix_timestamp>`
- **Delete Conversation**: `scm history-clear-conversation <peer_id>`

## 5. Network & Security
- **Swarm Stats**: `scm swarm stats`
- **Block Peer**: `scm block add <peer_id> [--reason <text>]`
- **Unblock Peer**: `scm block remove <peer_id>`
- **Sign Data**: `scm identity sign-data <hex_data>`
- **Verify Signature**: `scm identity verify-signature --data-hex <hex> --signature-hex <hex> --public-key-hex <hex>`

## 6. Configuration
- **Set Config**: `scm config set <key> <value>`
- **Get Config**: `scm config get <key>`
- **List Config**: `scm config list`

## 7. Discovery & Roadmap Gaps (Expected Soon)
The CLI currently performs most discovery in the **background**. Manual triggers are not yet implemented in the `scm` binary but are planned for Parity with Android.

- **mDNS / LAN**: Auto-starts with the node. No manual scan command yet.
- **Bluetooth / BLE**: Auto-starts with the node. No manual scan command yet.
- **WiFi-Aware**: Currently **Android-only**. Planned for CLI integration in the next update.
- **Universal Discovery Command**: `scm discovery scan` is **planned** but not yet available.

---
### Driver Usage via Ollama
To run these, use:
`python scripts/core_cli_driver.py raw <command_without_scm_prefix>`

*Note: If a user asks to "Scan for Bluetooth nodes", explain that Bluetooth discovery is currently automatic and running in the background while the node is started.*
