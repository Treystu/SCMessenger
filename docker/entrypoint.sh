#!/bin/bash
set -e

# Initialize config directory
mkdir -p "$SCM_CONFIG_DIR"
CONFIG_FILE="$SCM_CONFIG_DIR/config.json"

# Create default config if it doesn't exist
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default configuration..."
    echo '{}' > "$CONFIG_FILE"
fi

# Function to update nested JSON fields using temporary files and grep/sed/awk
# Note: In a real production environment, jq is preferred, but we want to minimize dependencies
# or we can install jq in the Dockerfile. For simplicity, we'll use a basic approach
# or just rely on 'scm config set' if the CLI supports it.

# Let's use the CLI itself to configure, which is robust
# Wait, 'scm config set' might expect 'scm init' to have run.
# 'scm init' is interactive. We need a way to init non-interactively or just start with defaults.
# The 'scm start' command handles initialization if identity is missing.

# Set listen port if provided
if [ ! -z "$LISTEN_PORT" ]; then
    echo "Setting listen port to $LISTEN_PORT"
    # We can use sed to patch config.json since it's simple JSON
    # Or better, run scm commands if they support non-interactive setup
fi

# Actually, the CLI's `config::Config::load()` creates a default config if missing.
# We can manipulate the file directly or via CLI commands IF we modify the CLI to support non-interactive init.
# For now, let's inject a basic valid config.json using jq (we should add jq to Dockerfile for ease)

# Check if jq is installed, if not, warn or fail
if ! command -v jq &> /dev/null; then
    echo "jq is not installed. Installing..."
    apt-get update && apt-get install -y jq
fi

# Base config structure
if [ ! -s "$CONFIG_FILE" ] || [ "$(cat "$CONFIG_FILE")" = "{}" ]; then
    # Create valid initial config
    cat > "$CONFIG_FILE" <<EOF
{
  "bootstrap_nodes": [],
  "listen_port": ${LISTEN_PORT:-0},
  "enable_mdns": true,
  "enable_dht": true,
  "storage_path": null,
  "network": {
    "max_peers": 50,
    "connection_timeout": 30,
    "enable_nat_traversal": true,
    "enable_relay": true
  }
}
EOF
fi

# Add bootstrap nodes from environment variable (comma separated)
if [ ! -z "$BOOTSTRAP_NODES" ]; then
    echo "Configuring bootstrap nodes..."
    # Split by comma and add each
    IFS=',' read -ra ADDR <<< "$BOOTSTRAP_NODES"
    for i in "${ADDR[@]}"; do
        # Use jq to add to array if not present
        tmp=$(mktemp)
        jq --arg node "$i" '.bootstrap_nodes += [$node] | .bootstrap_nodes |= unique' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
    done
fi

# Update listen port if specified
if [ ! -z "$LISTEN_PORT" ]; then
    tmp=$(mktemp)
    jq --arg port "$LISTEN_PORT" '.listen_port = ($port | tonumber)' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
fi

echo "Configuration complete:"
cat "$CONFIG_FILE"

# Run the command passed to arguments
exec "$@"
