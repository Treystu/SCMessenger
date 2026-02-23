> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# SCMessenger CLI — macOS Setup Guide

> Legacy OS-specific guide. Prefer `cli/README.md` for the current command surface.

Complete installation and usage guide for SCMessenger command-line interface on macOS.

## [Needs Revalidation] System Requirements

- **macOS Version**: 10.15 (Catalina) or later
- **Processor**: Intel or Apple Silicon (M1/M2/M3)
- **RAM**: 512 MB minimum, 1 GB recommended
- **Disk Space**: 100 MB for application + storage for message history
- **Network**: Internet connection for P2P connectivity

## [Needs Revalidation] Installation

### [Needs Revalidation] Prerequisites

#### [Needs Revalidation] 1. Install Homebrew (if not already installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### [Needs Revalidation] 2. Install Rust

```bash
# Install via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### [Needs Revalidation] Building SCMessenger CLI

#### [Needs Revalidation] Option 1: Build and Install System-Wide

```bash
# Clone the repository
git clone https://github.com/treystu/SCMessenger.git
cd SCMessenger

# Build release version
cargo build --release -p scmessenger-cli

# Install to ~/.cargo/bin (in your PATH)
cargo install --path cli

# Verify installation
scm --version
```

#### [Needs Revalidation] Option 2: Build for Local Development

```bash
# Clone the repository
git clone https://github.com/treystu/SCMessenger.git
cd SCMessenger

# Build debug version (faster compilation, slower runtime)
cargo build -p scmessenger-cli

# Run directly
./target/debug/scmessenger-cli --version
```

#### [Needs Revalidation] Option 3: Create Standalone Binary

```bash
# Build optimized release binary
cargo build --release -p scmessenger-cli

# Copy to /usr/local/bin for system-wide access
sudo cp target/release/scmessenger-cli /usr/local/bin/scm

# Or copy to ~/bin for user-only access
mkdir -p ~/bin
cp target/release/scmessenger-cli ~/bin/scm
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Verify
scm --version
```

## [Needs Revalidation] First-Time Setup

### [Needs Revalidation] 1. Initialize Your Identity

```bash
scm init
```

**What this does:**
- Creates `~/Library/Application Support/scmessenger/`
- Generates Ed25519 keypair
- Creates default configuration
- Initializes contact and message databases

**Output:**
```
✓ Configuration initialized
✓ Data directory: ~/Library/Application Support/scmessenger
✓ Identity created successfully

Your Peer ID: 12D3KooWABC123...
```

### [Needs Revalidation] 2. View Your Identity

```bash
scm identity show
```

**Save this information** — your Peer ID is how others will contact you.

### [Needs Revalidation] 3. Configure Bootstrap Nodes (Optional)

For better network connectivity, add reliable bootstrap nodes:

```bash
scm config bootstrap add /dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN
scm config bootstrap add /dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa
```

## [Needs Revalidation] File Locations on macOS

SCMessenger uses standard macOS directories:

| Component | Path |
|-----------|------|
| Configuration | `~/Library/Application Support/scmessenger/config.json` |
| Contacts Database | `~/Library/Application Support/scmessenger/contacts/` |
| Message History | `~/Library/Application Support/scmessenger/history/` |
| Identity Keypair | Stored in config.json |

### [Needs Revalidation] Viewing Files

```bash
# Open config directory in Finder
open ~/Library/Application\ Support/scmessenger

# View config file
cat ~/Library/Application\ Support/scmessenger/config.json

# Check data directory size
du -sh ~/Library/Application\ Support/scmessenger
```

### [Needs Revalidation] Backing Up Your Identity

⚠️ **IMPORTANT**: Back up your identity to prevent data loss!

```bash
# Create backup directory
mkdir -p ~/Documents/SCMessenger-Backup

# Export identity
scm identity export > ~/Documents/SCMessenger-Backup/identity.txt

# Backup entire config directory
cp -r ~/Library/Application\ Support/scmessenger ~/Documents/SCMessenger-Backup/

# Create encrypted backup (recommended)
tar -czf - ~/Library/Application\ Support/scmessenger | \
  openssl enc -aes-256-cbc -salt -out ~/Documents/SCMessenger-Backup/scm-backup.tar.gz.enc
```

## [Needs Revalidation] Network Configuration

### [Needs Revalidation] Firewall Settings

SCMessenger needs to accept incoming P2P connections.

#### [Needs Revalidation] Configure macOS Firewall

1. Open **System Settings** → **Network** → **Firewall**
2. Click **Options**
3. Click **+** to add an application
4. Navigate to where you installed `scm` (e.g., `/usr/local/bin/scm`)
5. Set to **Allow incoming connections**

#### [Needs Revalidation] Alternative: Allow specific port

```bash
# If you set a specific listen port
scm config set listen_port 9999

# Then allow that port through firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/local/bin/scm
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /usr/local/bin/scm
```

### [Needs Revalidation] Local Network Discovery (mDNS)

For peer discovery on your local network (home/office):

```bash
# Enable mDNS
scm config set enable_mdns true

# Start node
scm start
```

This allows automatic discovery of other SCMessenger users on the same WiFi/Ethernet network.

## [Needs Revalidation] Common Usage Examples

### [Needs Revalidation] Example 1: Send a Message to a Friend

```bash
# 1. Add your friend as a contact
scm contact add 12D3KooWXYZ... abc123publickey... --name Bob

# 2. Start the messaging node
scm start

# 3. In the interactive prompt, send a message
> send Bob Hey, want to grab coffee?
✓ Message sent to Bob

# 4. Check for replies
> # Messages appear automatically when received
📨 Bob: Sure! How about 3pm?

> send Bob Sounds good!

> quit
```

### [Needs Revalidation] Example 2: Queue Messages for Later Delivery

```bash
# Send a message even when recipient is offline
scm send Alice "Check out this article: https://..."

# Message is queued and will be delivered when Alice comes online
✓ Message queued for delivery to Alice
```

### [Needs Revalidation] Example 3: Search Your Message History

```bash
# Search for messages containing "meeting"
scm history --search meeting

# View conversation with specific person
scm history --peer Bob --limit 50

# View all recent messages
scm history
```

## [Needs Revalidation] Advanced Configuration

### [Needs Revalidation] Custom Storage Location

```bash
# Use an external drive for message storage
scm config set storage_path /Volumes/External/SCMessenger

# Or use iCloud Drive for sync (not recommended for privacy)
scm config set storage_path ~/Library/Mobile\ Documents/com~apple~CloudDocs/SCMessenger
```

### [Needs Revalidation] Performance Tuning

```bash
# Increase connection limit for better connectivity
scm config set max_connections 200

# Set specific listen port (useful for port forwarding)
scm config set listen_port 9999

# Disable DHT for faster startup (but fewer connections)
scm config set enable_dht false
```

### [Needs Revalidation] Running as Background Service

Create a Launch Agent to run SCMessenger in the background:

```bash
# Create the plist file
cat > ~/Library/LaunchAgents/com.scmessenger.cli.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.scmessenger.cli</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/scm</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/scmessenger.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/scmessenger.error.log</string>
</dict>
</plist>
EOF

# Load the service
launchctl load ~/Library/LaunchAgents/com.scmessenger.cli.plist

# Check if it's running
launchctl list | grep scmessenger

# View logs
tail -f /tmp/scmessenger.log

# Stop the service
launchctl unload ~/Library/LaunchAgents/com.scmessenger.cli.plist
```

## [Needs Revalidation] Shell Integration

### [Needs Revalidation] Zsh Completion (macOS default shell)

```bash
# Generate completion script
scm --generate-completion zsh > ~/.zsh/completions/_scm

# Add to ~/.zshrc if not already present
mkdir -p ~/.zsh/completions
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Reload shell
source ~/.zshrc
```

### [Needs Revalidation] Bash Completion

```bash
# Generate completion script
scm --generate-completion bash > /usr/local/etc/bash_completion.d/scm

# Add to ~/.bash_profile
echo 'source /usr/local/etc/bash_completion.d/scm' >> ~/.bash_profile

# Reload shell
source ~/.bash_profile
```

### [Needs Revalidation] Aliases

Add convenient aliases to your shell config:

```bash
# Add to ~/.zshrc or ~/.bash_profile
cat >> ~/.zshrc << 'EOF'

# SCMessenger aliases
alias scmsg='scm start'                    # Quick start
alias scms='scm status'                    # Check status
alias scmh='scm history'                   # View history
alias scmc='scm contact'                   # Manage contacts
alias scmconf='scm config'                 # Config shortcuts

# Quick message function
scmsend() {
    scm send "$1" "$2"
}

EOF

# Reload
source ~/.zshrc
```

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] Issue: Command not found

```bash
# Check if scm is in your PATH
which scm

# If not, add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Or use full path
~/.cargo/bin/scm --version
```

### [Needs Revalidation] Issue: Permission denied on config directory

```bash
# Fix permissions
chmod 700 ~/Library/Application\ Support/scmessenger
chmod 600 ~/Library/Application\ Support/scmessenger/config.json
```

### [Needs Revalidation] Issue: Cannot connect to any peers

```bash
# Check network connectivity
ping 8.8.8.8

# Verify bootstrap nodes
scm config bootstrap list

# Enable verbose logging (if available)
RUST_LOG=debug scm start

# Try local discovery
scm config set enable_mdns true
scm start
```

### [Needs Revalidation] Issue: Port already in use

```bash
# Find what's using the port
lsof -i :<port>

# Use a different port
scm config set listen_port 0  # Random port

# Or specify a specific unused port
scm config set listen_port 9999
```

### [Needs Revalidation] Issue: Build failures

```bash
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cargo clean

# Rebuild
cargo build --release -p scmessenger-cli

# If still failing, check for system dependencies
brew install cmake pkg-config openssl
```

## [Needs Revalidation] Uninstallation

To completely remove SCMessenger:

```bash
# Remove binary
rm /usr/local/bin/scm
# Or if installed via cargo install
cargo uninstall scmessenger-cli

# Remove data directory
rm -rf ~/Library/Application\ Support/scmessenger

# Remove launch agent (if created)
launchctl unload ~/Library/LaunchAgents/com.scmessenger.cli.plist
rm ~/Library/LaunchAgents/com.scmessenger.cli.plist

# Remove source code (if desired)
rm -rf ~/path/to/SCMessenger
```

## [Needs Revalidation] Security Best Practices

### [Needs Revalidation] File Permissions

Ensure your identity is protected:

```bash
# Restrict access to config directory
chmod 700 ~/Library/Application\ Support/scmessenger

# Restrict access to config file
chmod 600 ~/Library/Application\ Support/scmessenger/config.json
```

### [Needs Revalidation] FileVault Encryption

Enable FileVault for full-disk encryption:

1. **System Settings** → **Privacy & Security** → **FileVault**
2. Click **Turn On FileVault**
3. Save your recovery key securely

This protects your SCMessenger identity even if your Mac is stolen.

### [Needs Revalidation] Firewall Configuration

Enable macOS Firewall:

1. **System Settings** → **Network** → **Firewall**
2. Toggle **On**
3. Enable **Block all incoming connections** (SCMessenger can still make outgoing connections)

### [Needs Revalidation] Regular Backups

```bash
# Use Time Machine for automatic backups
# SCMessenger data is included in your user directory backups

# Or use a script for encrypted backups
#!/bin/bash
BACKUP_DIR=~/Documents/SCMessenger-Backups
DATE=$(date +%Y%m%d)
mkdir -p "$BACKUP_DIR"

tar -czf - ~/Library/Application\ Support/scmessenger | \
  openssl enc -aes-256-cbc -salt -out "$BACKUP_DIR/scm-$DATE.tar.gz.enc"

echo "Backup created: $BACKUP_DIR/scm-$DATE.tar.gz.enc"
```

## [Needs Revalidation] Performance Notes

### [Needs Revalidation] Apple Silicon (M1/M2/M3) Optimization

The CLI is compiled for your architecture automatically:

```bash
# Verify native compilation
file $(which scm)

# Should show:
# /usr/local/bin/scm: Mach-O 64-bit executable arm64

# For Intel Macs, it will show:
# /usr/local/bin/scm: Mach-O 64-bit executable x86_64
```

### [Needs Revalidation] Memory Usage

Typical memory usage:
- **Idle**: 20-40 MB
- **Active**: 50-100 MB
- **With large history**: 100-200 MB

### [Needs Revalidation] Network Bandwidth

- **Idle**: < 1 KB/s (keepalives)
- **Active messaging**: 1-10 KB/s
- **Initial DHT sync**: 100-500 KB burst

## [Needs Revalidation] Getting Help

```bash
# Built-in help
scm --help
scm <command> --help

# Run self-tests
scm test

# Check status
scm status
```

For more help:
- **GitHub Issues**: https://github.com/treystu/SCMessenger/issues
- **Documentation**: https://scmessenger.org/docs
- **Community**: #scmessenger on Matrix

## [Needs Revalidation] Updates

```bash
# Update to latest version
cd ~/path/to/SCMessenger
git pull
cargo install --path cli --force

# Verify new version
scm --version
```

## [Needs Revalidation] Next Steps

Now that you have SCMessenger installed:

1. Share your Peer ID with contacts: `scm identity show`
2. Add contacts: `scm contact add <peer-id> <public-key> --name <nickname>`
3. Start messaging: `scm start`

For more advanced usage, see the [main CLI README](../cli/README.md).
