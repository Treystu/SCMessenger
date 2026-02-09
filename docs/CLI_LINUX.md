# SCMessenger CLI — Linux Setup Guide

Complete installation and usage guide for SCMessenger command-line interface on Linux distributions.

## System Requirements

- **Distribution**: Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux, or compatible
- **Kernel**: Linux 5.4 or later
- **Processor**: x86_64, ARM64, or ARMv7
- **RAM**: 256 MB minimum, 512 MB recommended
- **Disk Space**: 100 MB for application + storage for message history
- **Network**: Internet connection for P2P connectivity

## Installation

### Prerequisites

#### Ubuntu / Debian

```bash
# Update package list
sudo apt update

# Install build dependencies
sudo apt install -y build-essential curl git pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Fedora / RHEL / CentOS

```bash
# Install development tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git pkg-config openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Arch Linux / Manjaro

```bash
# Install dependencies (many are already included in base-devel)
sudo pacman -S base-devel rust cargo git openssl pkg-config

# Verify installation
rustc --version
cargo --version
```

#### Alpine Linux

```bash
# Install dependencies
sudo apk add rust cargo git openssl-dev build-base pkgconfig

# Verify installation
rustc --version
cargo --version
```

### Building SCMessenger CLI

#### Option 1: Build and Install System-Wide

```bash
# Clone the repository
git clone https://github.com/yourusername/SCMessenger.git
cd SCMessenger

# Build release version
cargo build --release -p scmessenger-cli

# Install to ~/.cargo/bin (in your PATH)
cargo install --path cli

# Verify installation
scm --version
```

#### Option 2: Install to /usr/local/bin

```bash
# Build release version
cargo build --release -p scmessenger-cli

# Copy to system directory (requires sudo)
sudo cp target/release/scmessenger-cli /usr/local/bin/scm

# Make executable
sudo chmod +x /usr/local/bin/scm

# Verify
scm --version
```

#### Option 3: Install to ~/bin (User-Only)

```bash
# Build release version
cargo build --release -p scmessenger-cli

# Create user bin directory if it doesn't exist
mkdir -p ~/bin

# Copy binary
cp target/release/scmessenger-cli ~/bin/scm

# Add to PATH if not already present
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify
scm --version
```

## First-Time Setup

### 1. Initialize Your Identity

```bash
scm init
```

**What this does:**
- Creates `~/.config/scmessenger/` (config directory)
- Creates `~/.local/share/scmessenger/` (data directory)
- Generates Ed25519 keypair
- Creates default configuration
- Initializes contact and message databases

**Output:**
```
✓ Configuration initialized
✓ Data directory: ~/.local/share/scmessenger
✓ Identity created successfully

Your Peer ID: 12D3KooWABC123...
```

### 2. View Your Identity

```bash
scm identity show
```

**Save this information** — your Peer ID is how others will contact you.

### 3. Configure Bootstrap Nodes (Optional)

```bash
scm config bootstrap add /dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN
scm config bootstrap add /dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa
```

## File Locations on Linux

SCMessenger follows the XDG Base Directory Specification:

| Component | Path | Environment Variable |
|-----------|------|---------------------|
| Configuration | `~/.config/scmessenger/config.json` | `$XDG_CONFIG_HOME` |
| Data Directory | `~/.local/share/scmessenger/` | `$XDG_DATA_HOME` |
| Contacts Database | `~/.local/share/scmessenger/contacts/` | |
| Message History | `~/.local/share/scmessenger/history/` | |

### Custom XDG Locations

```bash
# Use custom config directory
export XDG_CONFIG_HOME=~/my-config
export XDG_DATA_HOME=~/my-data

# Then run scm (it will use the custom locations)
scm init
```

### Viewing Files

```bash
# View config file
cat ~/.config/scmessenger/config.json

# View config with jq (pretty print)
sudo apt install jq  # or: sudo dnf install jq
jq . ~/.config/scmessenger/config.json

# Check data directory size
du -sh ~/.local/share/scmessenger

# List all files
find ~/.local/share/scmessenger -type f
```

### Backing Up Your Identity

⚠️ **IMPORTANT**: Back up your identity to prevent data loss!

```bash
# Create backup directory
mkdir -p ~/scmessenger-backups

# Export identity
scm identity export > ~/scmessenger-backups/identity.txt

# Backup config and data directories
tar -czf ~/scmessenger-backups/scm-backup-$(date +%Y%m%d).tar.gz \
    ~/.config/scmessenger \
    ~/.local/share/scmessenger

# Create encrypted backup (recommended)
tar -czf - ~/.config/scmessenger ~/.local/share/scmessenger | \
    gpg --symmetric --cipher-algo AES256 \
    -o ~/scmessenger-backups/scm-backup-$(date +%Y%m%d).tar.gz.gpg

# Or use openssl
tar -czf - ~/.config/scmessenger ~/.local/share/scmessenger | \
    openssl enc -aes-256-cbc -salt -pbkdf2 \
    -out ~/scmessenger-backups/scm-backup-$(date +%Y%m%d).tar.gz.enc
```

## Network Configuration

### Firewall Settings

SCMessenger needs to accept incoming P2P connections.

#### UFW (Ubuntu/Debian)

```bash
# Allow specific port (if you set a fixed listen_port)
scm config set listen_port 9999
sudo ufw allow 9999/tcp
sudo ufw allow 9999/udp

# Or allow the application
sudo ufw allow from any to any app scm

# Check status
sudo ufw status
```

#### firewalld (Fedora/RHEL/CentOS)

```bash
# Allow specific port
scm config set listen_port 9999
sudo firewall-cmd --permanent --add-port=9999/tcp
sudo firewall-cmd --permanent --add-port=9999/udp
sudo firewall-cmd --reload

# Check status
sudo firewall-cmd --list-all
```

#### iptables (Manual)

```bash
# Allow specific port
sudo iptables -A INPUT -p tcp --dport 9999 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 9999 -j ACCEPT

# Save rules (Ubuntu/Debian)
sudo netfilter-persistent save

# Save rules (RHEL/CentOS)
sudo service iptables save
```

### Local Network Discovery (mDNS)

For peer discovery on your local network:

```bash
# Install Avahi (if not already installed)
# Ubuntu/Debian
sudo apt install avahi-daemon

# Fedora/RHEL
sudo dnf install avahi

# Arch Linux
sudo pacman -S avahi

# Enable and start
sudo systemctl enable avahi-daemon
sudo systemctl start avahi-daemon

# Enable mDNS in SCMessenger
scm config set enable_mdns true
```

## Running as a System Service

### systemd Service (Recommended)

Create a systemd user service to run SCMessenger automatically:

```bash
# Create user systemd directory
mkdir -p ~/.config/systemd/user

# Create service file
cat > ~/.config/systemd/user/scmessenger.service << 'EOF'
[Unit]
Description=SCMessenger P2P Messaging Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/scm start
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
EOF

# Reload systemd
systemctl --user daemon-reload

# Enable service (start on boot)
systemctl --user enable scmessenger.service

# Start service now
systemctl --user start scmessenger.service

# Check status
systemctl --user status scmessenger.service

# View logs
journalctl --user -u scmessenger.service -f

# Stop service
systemctl --user stop scmessenger.service

# Disable service (don't start on boot)
systemctl --user disable scmessenger.service
```

### Enable User Services on Boot

To start user services before login:

```bash
# Enable lingering for your user
sudo loginctl enable-linger $USER

# Verify
loginctl show-user $USER | grep Linger
```

## Shell Integration

### Bash Completion

```bash
# Create completion directory
mkdir -p ~/.local/share/bash-completion/completions

# Generate completion script
scm --generate-completion bash > ~/.local/share/bash-completion/completions/scm

# Add to ~/.bashrc if not already present
cat >> ~/.bashrc << 'EOF'
# Load bash completion
if [ -f ~/.local/share/bash-completion/completions/scm ]; then
    . ~/.local/share/bash-completion/completions/scm
fi
EOF

# Reload shell
source ~/.bashrc
```

### Zsh Completion

```bash
# Create completion directory
mkdir -p ~/.zsh/completions

# Generate completion script
scm --generate-completion zsh > ~/.zsh/completions/_scm

# Add to ~/.zshrc
cat >> ~/.zshrc << 'EOF'
# Load zsh completions
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
EOF

# Reload shell
source ~/.zshrc
```

### Fish Completion

```bash
# Create completion directory
mkdir -p ~/.config/fish/completions

# Generate completion script
scm --generate-completion fish > ~/.config/fish/completions/scm.fish

# Reload fish
fish
```

### Useful Aliases

Add to your `~/.bashrc` or `~/.zshrc`:

```bash
# SCMessenger aliases
alias scmsg='scm start'                    # Quick start
alias scms='scm status'                    # Check status
alias scmh='scm history'                   # View history
alias scmc='scm contact'                   # Manage contacts

# Quick send function
scmsend() {
    scm send "$1" "$2"
}

# Backup function
scmbackup() {
    tar -czf ~/scmessenger-backups/scm-backup-$(date +%Y%m%d).tar.gz \
        ~/.config/scmessenger ~/.local/share/scmessenger
    echo "Backup created: ~/scmessenger-backups/scm-backup-$(date +%Y%m%d).tar.gz"
}
```

## Common Usage Examples

### Example 1: Basic Messaging

```bash
# Add a contact
scm contact add 12D3KooWXYZ... abc123... --name Alice

# Start messaging node
scm start

# In interactive mode:
> send Alice Hello!
✓ Message sent to Alice

# Wait for reply
📨 Alice: Hi! How are you?

> send Alice I'm great, thanks!

> quit
```

### Example 2: Offline Messaging

```bash
# Send message when recipient is offline
scm send Bob "I'll be at the cafe at 3pm"

# Message is queued for delivery
✓ Message queued for delivery to Bob
```

### Example 3: Search Message History

```bash
# Search messages
scm history --search "meeting"

# View conversation with Alice
scm history --peer Alice --limit 50

# View recent messages
scm history --limit 100
```

## Advanced Configuration

### Custom Port Configuration

```bash
# Set specific port (useful for port forwarding)
scm config set listen_port 9999

# Verify
scm config get listen_port
```

### Performance Tuning

```bash
# Increase max connections
scm config set max_connections 200

# Adjust connection timeout
scm config set connection_timeout_secs 60
```

### Storage on Separate Partition

```bash
# Mount external drive
sudo mkdir -p /mnt/scmessenger
sudo mount /dev/sdb1 /mnt/scmessenger
sudo chown $USER:$USER /mnt/scmessenger

# Configure SCMessenger to use it
scm config set storage_path /mnt/scmessenger

# Make mount permanent (add to /etc/fstab)
echo "/dev/sdb1 /mnt/scmessenger ext4 defaults 0 2" | sudo tee -a /etc/fstab
```

## Troubleshooting

### Issue: Command not found

```bash
# Check if scm is in PATH
which scm

# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or use full path
~/.cargo/bin/scm --version
```

### Issue: Permission denied

```bash
# Fix permissions
chmod 700 ~/.config/scmessenger
chmod 700 ~/.local/share/scmessenger
chmod 600 ~/.config/scmessenger/config.json
```

### Issue: Cannot connect to peers

```bash
# Check network
ping 8.8.8.8

# Check firewall
sudo ufw status
# or
sudo firewall-cmd --list-all

# Enable verbose logging
RUST_LOG=debug scm start

# Try local discovery
scm config set enable_mdns true
scm start
```

### Issue: Port already in use

```bash
# Find what's using the port
sudo lsof -i :9999
# or
sudo netstat -tulpn | grep 9999

# Kill the process or use a different port
scm config set listen_port 0  # Random port
```

### Issue: Build failures

```bash
# Update Rust
rustup update stable

# Install missing dependencies
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel

# Clean and rebuild
cargo clean
cargo build --release -p scmessenger-cli
```

### Issue: Database corruption

```bash
# Backup current data
cp -r ~/.local/share/scmessenger ~/.local/share/scmessenger.backup

# Remove corrupted databases
rm -rf ~/.local/share/scmessenger/contacts
rm -rf ~/.local/share/scmessenger/history

# Restart to reinitialize
scm init
```

## Headless Server Setup

Running SCMessenger on a headless server (no GUI):

### 1. Install and Configure

```bash
# Install as described above
cargo install --path cli

# Initialize
scm init

# Configure
scm config set listen_port 9999
scm config bootstrap add /dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7...
```

### 2. Setup systemd Service

```bash
# Create service (as shown above)
systemctl --user enable --now scmessenger.service
```

### 3. Remote Management via SSH

```bash
# Connect via SSH
ssh user@server

# Check status
scm status
systemctl --user status scmessenger.service

# Send messages
scm send Alice "Message from server"

# View history
scm history --limit 50

# View logs
journalctl --user -u scmessenger.service -f
```

### 4. Monitoring Script

```bash
#!/bin/bash
# ~/bin/scm-monitor.sh

while true; do
    if ! systemctl --user is-active --quiet scmessenger.service; then
        echo "$(date): SCMessenger is down, restarting..."
        systemctl --user restart scmessenger.service
    fi
    sleep 60
done
```

## Security Best Practices

### File Permissions

```bash
# Restrict config directory
chmod 700 ~/.config/scmessenger
chmod 600 ~/.config/scmessenger/config.json

# Restrict data directory
chmod 700 ~/.local/share/scmessenger
```

### SELinux (RHEL/Fedora/CentOS)

```bash
# Check SELinux status
getenforce

# If enforcing, you may need to create a policy
# (Usually not required for user-space applications)

# Or run in permissive mode for testing
sudo setenforce 0
```

### AppArmor (Ubuntu/Debian)

```bash
# SCMessenger runs in user space and typically doesn't need AppArmor profiles
# Check AppArmor status
sudo aa-status
```

### Regular Security Updates

```bash
# Update system packages
sudo apt update && sudo apt upgrade  # Ubuntu/Debian
sudo dnf update  # Fedora
sudo pacman -Syu  # Arch Linux

# Update SCMessenger
cd ~/SCMessenger
git pull
cargo install --path cli --force
```

## Performance Notes

### Resource Usage

Typical resource usage on Linux:
- **Memory**: 20-50 MB idle, 50-150 MB active
- **CPU**: < 1% idle, 2-5% active messaging
- **Disk I/O**: Minimal (database writes on message send/receive)
- **Network**: < 1 KB/s idle, 1-10 KB/s active

### Monitoring

```bash
# Monitor resource usage
top -p $(pgrep -f scmessenger-cli)

# Or with htop
htop -p $(pgrep -f scmessenger-cli)

# Network connections
sudo netstat -tulpn | grep scm
```

## Uninstallation

```bash
# Stop service (if running)
systemctl --user stop scmessenger.service
systemctl --user disable scmessenger.service
rm ~/.config/systemd/user/scmessenger.service

# Remove binary
rm ~/.cargo/bin/scm
# Or if installed to /usr/local/bin
sudo rm /usr/local/bin/scm

# Remove data (backup first!)
rm -rf ~/.config/scmessenger
rm -rf ~/.local/share/scmessenger

# Remove source code
rm -rf ~/SCMessenger
```

## Distribution-Specific Notes

### Ubuntu/Debian

- Snap packages: Not currently available
- AppImage: Not currently available
- Use cargo build method as documented above

### Fedora

- RPM packages: Not currently available
- Use cargo build method as documented above

### Arch Linux

- AUR package: Not currently available (contributions welcome!)
- Use cargo build or PKGBUILD method

Creating a PKGBUILD:

```bash
# Example PKGBUILD for Arch Linux
cat > PKGBUILD << 'EOF'
# Maintainer: Your Name <your.email@example.com>
pkgname=scmessenger-cli
pkgver=0.1.0
pkgrel=1
pkgdesc="Sovereign encrypted P2P messaging CLI"
arch=('x86_64')
url="https://github.com/yourusername/SCMessenger"
license=('MIT')
depends=('openssl')
makedepends=('rust' 'cargo')
source=("git+https://github.com/yourusername/SCMessenger.git")
sha256sums=('SKIP')

build() {
    cd "$srcdir/SCMessenger"
    cargo build --release -p scmessenger-cli
}

package() {
    cd "$srcdir/SCMessenger"
    install -Dm755 "target/release/scmessenger-cli" "$pkgdir/usr/bin/scm"
}
EOF

# Build and install
makepkg -si
```

## Getting Help

```bash
# Built-in help
scm --help
scm <command> --help

# Run diagnostics
scm test

# Check status
scm status

# View logs (if running as service)
journalctl --user -u scmessenger.service -f
```

## Next Steps

1. Share your Peer ID: `scm identity show`
2. Add contacts: `scm contact add <peer-id> <public-key> --name <nickname>`
3. Start messaging: `scm start`

For more information, see the [main CLI README](../cli/README.md).
