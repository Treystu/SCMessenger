#!/bin/bash
set -e

# Initialize config and data directories
mkdir -p "$SCM_CONFIG_DIR"
mkdir -p "$SCM_DATA_DIR"
CONFIG_FILE="$SCM_CONFIG_DIR/config.json"

# Determine the port to use (priority: LISTEN_PORT > PORT > default 9000)
FINAL_PORT="${LISTEN_PORT:-${PORT:-9000}}"

# Create or update config if needed
if [ ! -f "$CONFIG_FILE" ] || [ ! -s "$CONFIG_FILE" ] || [ "$(cat "$CONFIG_FILE")" = "{}" ]; then
    echo "Creating default configuration..."
    cat > "$CONFIG_FILE" <<EOF
{
  "bootstrap_nodes": [],
  "listen_port": ${FINAL_PORT},
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
else
    # Update existing config with environment variables
    tmp=$(mktemp)
    jq --arg port "$FINAL_PORT" '.listen_port = ($port | tonumber)' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
fi

# Add bootstrap nodes from environment variable (comma separated)
if [ ! -z "$BOOTSTRAP_NODES" ]; then
    echo "Configuring bootstrap nodes..."
    IFS=',' read -ra ADDR <<< "$BOOTSTRAP_NODES"
    for i in "${ADDR[@]}"; do
        tmp=$(mktemp)
        jq --arg node "$i" '.bootstrap_nodes += [$node] | .bootstrap_nodes |= unique' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
    done
fi

echo "✓ Configuration ready (port: $FINAL_PORT)"

# If no port argument is provided to scm start, add it
if [ "$1" = "scm" ] && [ "$2" = "start" ]; then
    # Check if --port is already in arguments
    if ! echo "$@" | grep -q "\-\-port"; then
        set -- "$1" "$2" "--port" "$FINAL_PORT" "${@:3}"
    fi
fi

# Run the command
exec "$@"
