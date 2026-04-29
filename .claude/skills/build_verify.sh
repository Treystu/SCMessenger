#!/usr/bin/env bash
set -euo pipefail

COMMAND="${1:-full}"
PASSED=0
FAILED=0

run_gate() {
  local name="$1"
  shift
  echo "=== GATE: ${name} ==="
  if "$@" 2>&1; then
    echo "✅ ${name}: PASS"
    ((PASSED++))
  else
    echo "❌ ${name}: FAIL"
    ((FAILED++))
  fi
  echo ""
}

case "$COMMAND" in
  rust|full)
    run_gate "cargo_build" cargo build --workspace
    run_gate "cargo_check" cargo check --workspace
    run_gate "cargo_clippy" cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
    run_gate "cargo_fmt" cargo fmt --all -- --check
    run_gate "compile_gate" cargo test --workspace --no-run
    ;;
esac

case "$COMMAND" in
  android|full)
    if [ -d "android" ]; then
      run_gate "gradle_assembleDebug" bash -c 'cd android && ./gradlew assembleDebug -x lint --quiet'
      run_gate "role_nav_test" bash -c 'cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"'
    else
      echo "⚠️  Android directory not found, skipping Android gates"
    fi
    ;;
esac

case "$COMMAND" in
  wasm)
    run_gate "wasm_build" cargo build -p scmessenger-wasm --target wasm32-unknown-unknown
    run_gate "wasm_check" cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
    ;;
esac

case "$COMMAND" in
  compile_gate)
    run_gate "compile_gate" cargo test --workspace --no-run
    ;;
esac

echo ""
echo "=== BUILD VERIFICATION SUMMARY ==="
echo "Passed: ${PASSED}"
echo "Failed: ${FAILED}"
echo "==================================="

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
