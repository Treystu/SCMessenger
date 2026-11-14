#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[WS12] Rust workspace compile gate"
cargo test --workspace --no-run

echo "[WS12] Rust workspace test gate"
cargo test --workspace

echo "[WS12] Deterministic offline/partition suites"
cargo test -p scmessenger-core --test integration_offline_partition_matrix
cargo test -p scmessenger-core --test integration_retry_lifecycle
cargo test -p scmessenger-core --test integration_receipt_convergence
cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored

echo "[WS12] Desktop role parity checks (WASM adapter)"
cargo test -p scmessenger-wasm test_desktop_role_resolution_defaults_to_relay_only_without_identity
cargo test -p scmessenger-wasm test_desktop_relay_only_flow_blocks_outbound_message_prepare

if [[ "${SCM_SKIP_ANDROID:-0}" != "1" ]]; then
  if [[ -z "${ANDROID_HOME:-}" ]]; then
    echo "ANDROID_HOME must be set for Android parity checks (or set SCM_SKIP_ANDROID=1)." >&2
    exit 1
  fi
  echo "[WS12] Android role/fallback parity checks"
  (
    cd android
    ./gradlew :app:testDebugUnitTest \
      --tests com.scmessenger.android.test.RoleNavigationPolicyTest \
      --tests com.scmessenger.android.data.MeshRepositoryTest
  )
fi

if [[ "${SCM_SKIP_IOS:-0}" != "1" ]]; then
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "iOS parity checks require macOS (or set SCM_SKIP_IOS=1)." >&2
    exit 1
  fi
  echo "[WS12] iOS transport/role parity checks"
  bash ./iOS/verify-local-transport.sh
  bash ./iOS/verify-role-mode.sh
fi
