#!/usr/bin/env bash
# Reports disk usage for the repo's known build-cache hotspots plus overall
# free space. Read-only / non-blocking: never deletes anything itself.
# Warns (does not fail) when free space on the repo's drive drops below 15%.
#
# Usage: bash scripts/disk_hygiene.sh
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR" || exit 0

echo "=== Disk hygiene report ==="

df_line="$(df -h / 2>/dev/null | tail -1)"
echo "root filesystem: ${df_line}"

used_pct="$(echo "${df_line}" | awk '{print $5}' | tr -d '%')"
if [ -n "${used_pct}" ] && [ "${used_pct}" -ge 85 ] 2>/dev/null; then
  echo "[WARNING] Root filesystem is ${used_pct}% used (>= 85%) -- consider running:"
  echo "  cargo clean                 (workspace target/, safe, regenerable)"
  echo "  cd android && ./gradlew clean   (Android build outputs, safe, regenerable)"
fi

echo "--- build-cache hotspots ---"
for dir in target core/target android/app/build; do
  if [ -d "${dir}" ]; then
    du -sh "${dir}" 2>/dev/null | awk -v d="${dir}" '{print $1"\t"d}'
  fi
done

if [ -d "${HOME}/.cargo/registry" ]; then
  du -sh "${HOME}/.cargo/registry" 2>/dev/null | awk '{print $1"\t~/.cargo/registry (network re-download cost if cleared)"}'
fi
if [ -d "${HOME}/.gradle/caches" ]; then
  du -sh "${HOME}/.gradle/caches" 2>/dev/null | awk '{print $1"\t~/.gradle/caches (network re-download cost if cleared)"}'
fi

echo "==========================="
exit 0
