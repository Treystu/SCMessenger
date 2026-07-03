#!/usr/bin/env bash
# SessionStart hook: print a quick backlog/state orientation so a fresh
# session doesn't have to re-derive it via several manual lookups.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR" || exit 0

echo "=== SCMessenger session orientation ==="

echo "--- git status (short) ---"
git status --short 2>/dev/null | head -20

echo "--- HANDOFF backlog ---"
if [ -d HANDOFF/todo ]; then
  echo "todo: $(ls HANDOFF/todo 2>/dev/null | wc -l | tr -d ' ') files"
fi
if [ -d HANDOFF/IN_PROGRESS ]; then
  echo "in_progress: $(ls HANDOFF/IN_PROGRESS 2>/dev/null | wc -l | tr -d ' ') files"
fi

echo "--- REMAINING_WORK_TRACKING.md header ---"
if [ -f REMAINING_WORK_TRACKING.md ]; then
  head -6 REMAINING_WORK_TRACKING.md
fi

echo "========================================"
exit 0
