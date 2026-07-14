#!/usr/bin/env bash
# Safely inject AWS credentials into ~/.config/scmorc/aws.env for the
# SCMessenger cloud relay rig (see infra/aws/README.md).
#
# - Prompts interactively; the secret key is never echoed to the terminal,
#   never written to shell history, and never appears in a process listing
#   (unlike passing it as a CLI argument would).
# - Region is fixed to us-east-1, matching the region lock in
#   iam-policy-scmessenger-relay.json - the scoped IAM policy only allows
#   EC2/security-group/key-pair actions in that region, so a different
#   region here would just fail silently at request time.
# - Restricts the file to owner-read/write only (chmod 600) where supported.
#
# Usage: bash infra/aws/set-aws-credentials.sh

set -euo pipefail

# Ensure the Git-for-Windows coreutils (mkdir, chmod, cat, grep) are on PATH.
# When this script is launched by calling usr/bin/bash.exe directly from
# PowerShell (rather than the bin/bash.exe login wrapper), PATH does not
# include /usr/bin, so `mkdir: command not found` etc. Prepend them here so
# the script works regardless of how bash was invoked.
export PATH="/usr/bin:/bin:$PATH"

ENV_DIR="$HOME/.config/scmorc"
ENV_FILE="$ENV_DIR/aws.env"
REGION="us-east-1"

mkdir -p "$ENV_DIR"

if [ -f "$ENV_FILE" ]; then
    echo "[WARNING] $ENV_FILE already exists."
    read -r -p "Overwrite it? [y/N] " confirm
    case "$confirm" in
        [yY]|[yY][eE][sS]) ;;
        *) echo "[INFO] Aborted, existing file left untouched."; exit 0 ;;
    esac
fi

echo "AWS credentials for the SCMessenger relay IAM user."
echo "Region is fixed to $REGION (matches the scoped IAM policy)."
echo

read -r -p "AWS Access Key ID: " access_key_id
if [ -z "$access_key_id" ]; then
    echo "[ERROR] Access Key ID cannot be empty."
    exit 1
fi

# -s = silent (no echo to terminal); -r = don't interpret backslashes.
read -r -s -p "AWS Secret Access Key (input hidden): " secret_access_key
echo
if [ -z "$secret_access_key" ]; then
    echo "[ERROR] Secret Access Key cannot be empty."
    exit 1
fi

umask 077
cat > "$ENV_FILE" <<EOF
AWS_ACCESS_KEY_ID=$access_key_id
AWS_SECRET_ACCESS_KEY=$secret_access_key
AWS_DEFAULT_REGION=$REGION
EOF

chmod 600 "$ENV_FILE" 2>/dev/null || true

echo
echo "[OK] Credentials written to $ENV_FILE (not printed, not committed - that"
echo "     path is outside the repo tree, same convention as the other lanes'"
echo "     keys in this session)."
echo "[OK] File permissions restricted to owner read/write where supported."
echo
echo "Verify without exposing the secret:"
echo "  grep -o '^AWS_ACCESS_KEY_ID=.*' \"$ENV_FILE\""
