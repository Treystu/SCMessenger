#!/usr/bin/env bash
# SCMessenger CLI — install locally built binary to ~/.local/bin and register a user daemon.
#
# Prerequisites: build first with `cargo build --release -p scmessenger-cli`
# Optional: SCMESSENGER_BIN=/path/to/scmessenger-cli to copy a specific artifact.
#
# Linux: writes ~/.config/systemd/user/scmessenger.service (enable manually).
# macOS: writes ~/Library/LaunchAgents/io.scmessenger.cli.plist (load manually).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_BIN="${ROOT}/target/release/scmessenger-cli"
BIN="${SCMESSENGER_BIN:-$DEFAULT_BIN}"
DEST_DIR="${HOME}/.local/bin"
DEST="${DEST_DIR}/scmessenger-cli"

if [[ ! -f "$BIN" ]]; then
  echo "error: CLI binary not found at: $BIN" >&2
  echo "  Build with: (cd repo && cargo build --release -p scmessenger-cli)" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
cp -f "$BIN" "$DEST"
chmod +x "$DEST"
echo "Installed: $DEST"
echo "Ensure PATH includes: $DEST_DIR"

OS="$(uname -s)"
if [[ "$OS" == "Linux" ]]; then
  UNIT_DIR="${HOME}/.config/systemd/user"
  mkdir -p "$UNIT_DIR"
  cat >"${UNIT_DIR}/scmessenger.service" <<EOF
[Unit]
Description=SCMessenger CLI daemon (local mesh + Web UI on 127.0.0.1)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${DEST} start
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
  echo "Wrote systemd user unit: ${UNIT_DIR}/scmessenger.service"
  echo "  systemctl --user daemon-reload"
  echo "  systemctl --user enable --now scmessenger.service"
elif [[ "$OS" == "Darwin" ]]; then
  PLIST="${HOME}/Library/LaunchAgents/io.scmessenger.cli.plist"
  mkdir -p "$(dirname "$PLIST")"
  cat >"$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>io.scmessenger.cli</string>
  <key>ProgramArguments</key>
  <array>
    <string>${DEST}</string>
    <string>start</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
</dict>
</plist>
EOF
  echo "Wrote launchd plist: $PLIST"
  echo "  launchctl load -w \"$PLIST\""
else
  echo "OS $OS: no automatic service file generated (use Task Scheduler on Windows; see scripts/install.ps1)."
fi
