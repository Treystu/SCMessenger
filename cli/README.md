# SCMessenger CLI

Cross-platform command-line interface for SCMessenger — sovereign, encrypted peer-to-peer messaging.

## Features

- **Sovereign Identity**: No accounts, no servers — your identity is your cryptographic keypair
- **End-to-End Encryption**: All messages encrypted with modern cryptography (X25519, ChaCha20-Poly1305)
- **Decentralized Network**: Peer-to-peer using libp2p with DHT routing and NAT traversal
- **Offline Queueing**: Send messages even when recipient is offline
- **Contact Management**: Persistent contact list with nicknames and search
- **Message History**: Full message history with search and filtering
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/SCMessenger.git
cd SCMessenger

# Build the CLI
cargo build --release -p scmessenger-cli

# Install (optional)
cargo install --path cli
```

### First Run

```bash
# Initialize your identity
scm init

# View your identity
scm identity show

# Check status
scm status
```

## Command Reference

### Identity Management

#### Initialize Identity
Creates a new cryptographic identity and configuration.

```bash
scm init
```

**Output:**
- Creates config directory (platform-specific)
- Generates Ed25519 keypair
- Sets up data directory for contacts and history

#### Show Identity
Display your current identity information.

```bash
scm identity show
```

**Output:**
```
Peer ID:   12D3KooWABC...
Public Key: 4f3a2b1c...
```

#### Export Identity
Export your keypair for backup or transfer.

```bash
scm identity export
```

**Output:**
```
Private Key: a1b2c3d4e5f6...
Public Key:  4f3a2b1c...
```

⚠️ **Warning**: Keep your private key secret! Anyone with your private key can impersonate you.

### Contact Management

#### Add Contact
Add a new contact to your list.

```bash
scm contact add <peer-id> <public-key> --name <nickname>
```

**Example:**
```bash
scm contact add 12D3KooWABC... 4f3a2b1c... --name Alice
```

#### List Contacts
Show all contacts.

```bash
scm contact list
```

**Output:**
```
Alice
  Peer ID:    12D3KooWABC...
  Public Key: 4f3a2b1c...
  Added:      2026-02-09 10:30:00
```

#### Show Contact Details
Display detailed information for a specific contact.

```bash
scm contact show <contact>
```

**Example:**
```bash
scm contact show Alice
```

#### Remove Contact
Remove a contact from your list.

```bash
scm contact remove <contact>
```

#### Search Contacts
Search contacts by name or peer ID.

```bash
scm contact search <query>
```

### Configuration

#### Set Configuration
Update a configuration value.

```bash
scm config set <key> <value>
```

**Available keys:**
- `listen_port` - Port for P2P network (default: 0 = random)
- `enable_mdns` - Enable mDNS for local peer discovery (true/false)
- `enable_dht` - Enable DHT for peer routing (true/false)
- `storage_path` - Custom path for message storage

**Examples:**
```bash
scm config set listen_port 9999
scm config set enable_mdns true
```

#### Get Configuration
Retrieve a configuration value.

```bash
scm config get <key>
```

#### List Configuration
Show all configuration settings.

```bash
scm config list
```

#### Manage Bootstrap Nodes
Bootstrap nodes help you connect to the P2P network.

```bash
# Add a bootstrap node
scm config bootstrap add <multiaddr>

# Remove a bootstrap node
scm config bootstrap remove <multiaddr>

# List bootstrap nodes
scm config bootstrap list
```

**Example:**
```bash
scm config bootstrap add /ip4/147.75.83.83/tcp/4001/p2p/QmW9m57aGDWiR1BaTEyUAzSK1dC4AaJCJmpKmJPYWEKrFY
```

### Messaging

#### Start P2P Node (Interactive Mode)
Start the messaging node and enter interactive mode for live chat.

```bash
scm start [--port <port>]
```

**Interactive Commands:**
- `send <contact> <message>` - Send a message to a contact
- `contacts` - List all contacts
- `peers` - Show connected peers
- `status` - Display network status
- `help` - Show available commands
- `quit` - Exit interactive mode

**Example Session:**
```bash
$ scm start
✓ Configuration loaded
✓ Identity: 12D3KooW...
✓ Network started on /ip4/0.0.0.0/tcp/51234

Type 'help' for commands, 'quit' to exit

> send Alice Hello!
✓ Message sent to Alice

> peers
Connected peers (1):
  12D3KooWABC... (Alice)

> quit
Shutting down...
```

#### Send Message (Offline Mode)
Queue a message for delivery when the recipient comes online.

```bash
scm send <recipient> <message>
```

**Example:**
```bash
scm send Alice "Hey, check this out!"
```

### Message History

#### View History
Display recent messages.

```bash
scm history [--peer <contact>] [--search <query>] [--limit <num>]
```

**Options:**
- `--peer, -p` - Filter by specific contact
- `--search, -s` - Search message content
- `--limit, -l` - Number of messages to show (default: 20)

**Examples:**
```bash
# Show recent 20 messages
scm history

# Show messages with Alice
scm history --peer Alice

# Search all messages
scm history --search "meeting"

# Show last 50 messages
scm history --limit 50
```

### Status and Testing

#### Network Status
Show current network and storage statistics.

```bash
scm status
```

**Output:**
```
Identity:  12D3KooW...
Contacts:  5
Messages:  47 (sent: 23, received: 24)
Config:    ~/.config/scmessenger/config.json
Data:      ~/.local/share/scmessenger
```

#### Self-Test
Run diagnostic tests to verify the system is working.

```bash
scm test
```

**Output:**
```
✓ Identity generation
✓ Message encryption (356 bytes)
✓ Message decryption
✓ Encryption security
All tests passed!
```

## File Locations

### macOS
- **Config**: `~/Library/Application Support/scmessenger/config.json`
- **Data**: `~/Library/Application Support/scmessenger/`
- **Contacts**: `~/Library/Application Support/scmessenger/contacts/`
- **History**: `~/Library/Application Support/scmessenger/history/`

### Linux
- **Config**: `~/.config/scmessenger/config.json`
- **Data**: `~/.local/share/scmessenger/`
- **Contacts**: `~/.local/share/scmessenger/contacts/`
- **History**: `~/.local/share/scmessenger/history/`

### Windows
- **Config**: `%APPDATA%\scmessenger\config.json`
- **Data**: `%APPDATA%\scmessenger\`
- **Contacts**: `%APPDATA%\scmessenger\contacts\`
- **History**: `%APPDATA%\scmessenger\history\`

## Configuration File Format

The configuration is stored as JSON:

```json
{
  "bootstrap_nodes": [
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"
  ],
  "listen_port": 0,
  "enable_mdns": true,
  "enable_dht": true,
  "storage_path": null,
  "network": {
    "connection_timeout_secs": 30,
    "max_connections": 100
  }
}
```

## Platform-Specific Guides

For detailed platform-specific instructions, see:
- [macOS Setup Guide](docs/CLI_MACOS.md)
- [Linux Setup Guide](docs/CLI_LINUX.md)
- [Windows Setup Guide](docs/CLI_WINDOWS.md)

## Architecture

The CLI is built with:
- **clap** - Command-line argument parsing
- **sled** - Embedded database for contacts and history
- **dirs** - Cross-platform directory paths
- **colored** - Terminal output styling
- **tokio** - Async runtime
- **scmessenger-core** - Core cryptography and networking

## Security Notes

1. **Private Key Protection**: Your private key is stored in the config directory. Protect it with appropriate filesystem permissions.

2. **Message Encryption**: All messages are encrypted end-to-end using X25519 key exchange and ChaCha20-Poly1305 AEAD.

3. **Identity Verification**: Always verify public keys through a trusted side channel before adding contacts.

4. **Network Privacy**: By default, the CLI connects to public bootstrap nodes. For maximum privacy, run your own bootstrap nodes.

## Troubleshooting

### Cannot connect to peers
1. Check your firewall settings
2. Verify bootstrap nodes are reachable: `scm config bootstrap list`
3. Try enabling mDNS for local discovery: `scm config set enable_mdns true`

### Identity not found
Run `scm init` to create a new identity.

### Permission denied errors
Ensure the data directory is writable:
```bash
# macOS/Linux
chmod 700 ~/.local/share/scmessenger

# Or check the actual path
scm status
```

### Build errors
Ensure you have the latest Rust toolchain:
```bash
rustup update stable
```

## Development

### Building from Source

```bash
# Debug build
cargo build -p scmessenger-cli

# Release build (optimized)
cargo build --release -p scmessenger-cli

# Run without installing
cargo run -p scmessenger-cli -- <command>
```

### Running Tests

```bash
# Unit tests
cargo test -p scmessenger-cli

# Integration tests
cargo test -p scmessenger-cli --test '*'
```

### Code Quality

```bash
# Check for warnings
cargo clippy -p scmessenger-cli

# Format code
cargo fmt -p scmessenger-cli

# Security audit
cargo audit
```

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

See [LICENSE](../LICENSE) for details.

## Support

- **Issues**: https://github.com/yourusername/SCMessenger/issues
- **Discussions**: https://github.com/yourusername/SCMessenger/discussions
- **Matrix**: #scmessenger:matrix.org
