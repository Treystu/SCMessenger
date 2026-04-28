#!/usr/bin/env bash
# scripts/prune_sim_logs.sh — Safe log pruning with preservation guarantees
#
# Usage:
#   ./scripts/prune_sim_logs.sh
#   KEEP_HOURS=48 ./scripts/prune_sim_logs.sh
#   DRY_RUN=1 ./scripts/prune_sim_logs.sh
#
# Environment variables:
#   KEEP_HOURS  — preserve logs newer than this many hours (default: 24)
#   DRY_RUN     — preview what would be deleted without deleting (default: 0)
#
# Safety features:
#   - Preserves the 'latest' symlink and its target directory
#   - Preserves logs newer than KEEP_HOURS
#   - Dry-run mode for safe preview
#   - Reports what will be deleted before deletion
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

KEEP_HOURS="${KEEP_HOURS:-24}"
DRY_RUN="${DRY_RUN:-0}"
LOG_ROOT="$ROOT_DIR/logs"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Log Pruner                                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Log root:     $LOG_ROOT"
echo "Keep hours:   $KEEP_HOURS"
echo "Dry run:      $DRY_RUN"
echo ""

if [ ! -d "$LOG_ROOT" ]; then
  echo "No log directory found at $LOG_ROOT"
  exit 0
fi

# Identify directories to preserve
PRESERVE_DIRS=()

# Preserve 'latest' symlink targets
while IFS= read -r -d '' link; do
  target="$(readlink "$link" 2>/dev/null || true)"
  if [ -n "$target" ]; then
    link_dir="$(dirname "$link")"
    if [[ "$target" = /* ]]; then
      abs_target="$target"
    else
      abs_target="$link_dir/$target"
    fi
    if [ -d "$abs_target" ]; then
      PRESERVE_DIRS+=("$abs_target")
      echo "Preserving latest: $abs_target"
    fi
  fi
done < <(find "$LOG_ROOT" -name "latest" -type l -print0 2>/dev/null)

# Calculate minutes for find command
KEEP_MINUTES=$((KEEP_HOURS * 60))

# Show what would be deleted
echo ""
echo "Files older than ${KEEP_HOURS}h:"
OLD_FILES=$(find "$LOG_ROOT" -type f \( -name "*.log" -o -name "*.txt" \) -mmin +"$KEEP_MINUTES" 2>/dev/null || true)

if [ -z "$OLD_FILES" ]; then
  echo "  No old files to prune."
  echo ""
  echo "✅ Log pruning complete (nothing to do)."
  exit 0
fi

# Filter out files in preserve directories
FILES_TO_DELETE=""
while IFS= read -r file; do
  should_preserve=0
  for preserve_dir in "${PRESERVE_DIRS[@]}"; do
    if [[ "$file" == "$preserve_dir"* ]]; then
      should_preserve=1
      break
    fi
  done
  if [ "$should_preserve" -eq 0 ]; then
    FILES_TO_DELETE+="$file"$'\n'
  fi
done <<< "$OLD_FILES"

if [ -z "$FILES_TO_DELETE" ]; then
  echo "  All old files are in preserved directories."
  echo ""
  echo "✅ Log pruning complete (nothing to prune)."
  exit 0
fi

# Show summary
TOTAL_SIZE=$(echo "$FILES_TO_DELETE" | xargs du -ch 2>/dev/null | tail -1 | awk '{print $1}' || echo "unknown")
FILE_COUNT=$(echo "$FILES_TO_DELETE" | grep -c . || echo 0)

echo "  Files to delete: $FILE_COUNT"
echo "  Total size:      $TOTAL_SIZE"
echo ""

if [ "$DRY_RUN" = "1" ]; then
  echo "[DRY-RUN] Would delete:"
  echo "$FILES_TO_DELETE" | head -20
  if [ "$FILE_COUNT" -gt 20 ]; then
    echo "  ... and $((FILE_COUNT - 20)) more files"
  fi
  echo ""
  echo "[DRY-RUN] No files were deleted."
  exit 0
fi

# Actually delete
echo "Deleting $FILE_COUNT files..."
echo "$FILES_TO_DELETE" | while IFS= read -r file; do
  [ -n "$file" ] || continue
  rm -f "$file"
done

# Remove empty directories
echo "Removing empty log directories..."
find "$LOG_ROOT" -type d -empty -delete 2>/dev/null || true

echo ""
echo "✅ Log pruning complete."
