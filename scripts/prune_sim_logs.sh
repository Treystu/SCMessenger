#!/usr/bin/env bash
# scripts/prune_sim_logs.sh
# Remove simulation logs older than 3 days to prevent disk bloat.

echo "🔍 Auditing logs for bloat..."
find ./logs -type f -name "*.log" -mtime +3 -exec du -h {} + | sort -rh | head -n 20
find ./logs -type f -name "*.txt" -mtime +3 -exec du -h {} + | sort -rh | head -n 20

echo "🧹 Pruning old simulation logs (older than 3 days)..."
find ./logs -type f -name "*.log" -mtime +3 -delete
find ./logs -type f -name "*.txt" -mtime +3 -delete

echo "📁 Removing empty log directories..."
find ./logs -type d -empty -delete

echo "✅ Log pruning complete."
