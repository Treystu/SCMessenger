#!/usr/bin/env bash
set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
MAIN_TAB="$DIR/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift"

require_pattern() {
  local pattern="$1"
  local message="$2"
  if ! rg -q "$pattern" "$MAIN_TAB"; then
    echo "Role-mode parity check failed: $message"
    exit 1
  fi
}

require_pattern "if identityInitialized \\{" "identity gate for Messages/Contacts tabs is missing"
require_pattern "Label\\(\"Messages\"" "Messages tab declaration is missing"
require_pattern "Label\\(\"Contacts\"" "Contacts tab declaration is missing"
require_pattern "if !identityInitialized && \\(selectedTab == \\.messages \\|\\| selectedTab == \\.contacts\\)" \
  "relay-only tab reset guard is missing"
require_pattern "selectedTab = \\.mesh" "relay-only fallback to Mesh tab is missing"

echo "PASS: iOS role-mode parity checks"
