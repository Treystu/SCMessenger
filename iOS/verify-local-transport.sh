#!/usr/bin/env bash
set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
MODULE_CACHE_DIR="$(mktemp -d /tmp/scmessenger-swift-module-cache.XXXXXX)"
TEST_BIN="$(mktemp /tmp/scmessenger-local-transport-tests.XXXXXX)"
trap 'rm -rf "$MODULE_CACHE_DIR"; rm -f "$TEST_BIN"' EXIT

SWIFT_MODULE_CACHE_PATH="$MODULE_CACHE_DIR" \
CLANG_MODULE_CACHE_PATH="$MODULE_CACHE_DIR" \
swiftc \
  -module-cache-path "$MODULE_CACHE_DIR" \
  "$DIR/SCMessenger/SCMessenger/Transport/LocalTransportFallback.swift" \
  "$DIR/tests/local_transport_fallback_tests.swift" \
  -o "$TEST_BIN"

"$TEST_BIN"
