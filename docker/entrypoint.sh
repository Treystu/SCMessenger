#!/bin/bash
set -e

# Initialize config and data directories
mkdir -p "$SCM_CONFIG_DIR"
mkdir -p "$SCM_DATA_DIR"
CONFIG_FILE="$SCM_CONFIG_DIR/config.json"

# Determine the port to use (priority: LISTEN_PORT > PORT > default 9000)
FINAL_PORT="${LISTEN_PORT:-${PORT:-9000}}"

# Note: Bootstrap nodes are now embedded in the binary at build time
# The SCM CLI will automatically load these defaults when creating new config
# This entrypoint only needs to handle port configuration and environment overrides

# Create initial config if it doesn't exist (CLI will populate with defaults)
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Initializing configuration with embedded bootstrap nodes..."
    # Let the CLI create the default config with embedded bootstraps
    scm config list > /dev/null 2>&1 || true
fi

# Update port from environment if specified
if [ -f "$CONFIG_FILE" ]; then
    tmp=$(mktemp)
    jq --arg port "$FINAL_PORT" '.listen_port = ($port | tonumber)' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
fi

# Add additional bootstrap nodes from environment variable (merged with defaults)
if [ ! -z "$BOOTSTRAP_NODES" ]; then
    echo "Adding environment bootstrap nodes to defaults..."
    IFS=',' read -ra ADDR <<< "$BOOTSTRAP_NODES"
    for i in "${ADDR[@]}"; do
        node=$(echo "$i" | xargs)  # Trim whitespace
        if [ ! -z "$node" ]; then
            tmp=$(mktemp)
            jq --arg node "$node" '.bootstrap_nodes += [$node] | .bootstrap_nodes |= unique' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
        fi
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
