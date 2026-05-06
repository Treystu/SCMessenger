# REPO_MAP System Documentation

## Overview

The REPO_MAP system provides a structured, indexed representation of the codebase to enable efficient agentic workflows. It reduces token usage by allowing agents to quickly locate and understand code structure without reading entire files.

## Architecture

```
HANDOFF_AUDIT/
├── repo_map_index.json          # Main index with metadata
├── REPO_MAP.jsonl                # Combined JSONL entries
├── REPO_MAP_android.jsonl        # Android-specific entries
├── REPO_MAP_other.jsonl          # Other platform entries
├── done/                         # Processed ticket files
│   └── *.txt                     # File path and chunk info
├── output/                       # Indexed JSONL data
│   └── *.jsonl                   # Structured code metadata
└── .context_cache/               # Cached context data

```

## Index Structure

### repo_map_index.json

```json
{
  "version": "1.0",
  "generated_at": "2026-05-06T06:51:32.006215Z",
  "files": {
    "relative/path/to/file.rs": {
      "indexed_at": "2026-05-04T18:18:34Z",
      "file_modified_at": "2026-05-03T10:30:00Z",
      "chunks": [1, 2, 3],
      "total_lines": 450,
      "status": "complete"
    }
  }
}
```

### JSONL Entry Format

Each `.jsonl` file in `output/` contains structured metadata:

```json
{
  "file": "MeshApplication.kt",
  "chunk": 1,
  "summary": "SCMessenger Application class with Hilt dependency injection.",
  "structs_or_classes": ["MeshApplication"],
  "imports": ["android.app.Application", "dagger.hilt.android.HiltAndroidApp"],
  "funcs": [
    {
      "name": "onCreate",
      "line": 18,
      "calls_out_to": ["Timber.plant(Timber.DebugTree())"]
    }
  ]
}
```

## Scripts

### 1. build_repo_index.py

**Purpose**: Build or update the REPO_MAP index from processed tickets.

**Usage**:
```bash
# Full rebuild
python .claude/scripts/build_repo_index.py --full-rebuild

# Incremental update for specific files
python .claude/scripts/build_repo_index.py --incremental --files "file1.rs,file2.kt"
```

**What it does**:
- Reads ticket files from `done/`
- Extracts file paths and chunk numbers
- Reads corresponding JSONL from `output/`
- Updates `repo_map_index.json` with metadata
- Tracks file modification times for staleness detection

### 2. verify_and_fix_repo_map.py

**Purpose**: Verify REPO_MAP health and automatically fix issues.

**Usage**:
```bash
# Verification only
python .claude/scripts/verify_and_fix_repo_map.py

# Verification + automatic fixes
python .claude/scripts/verify_and_fix_repo_map.py --fix
```

**Checks performed**:
- ✅ Missing `file_modified_at` timestamps
- ✅ Stale files (modified after indexing)
- ✅ Missing files (indexed but deleted)
- ✅ Invalid paths or timestamps
- ✅ Orphaned entries

**Fixes applied**:
- Populates missing timestamps from filesystem
- Updates stale entry timestamps
- Generates detailed verification report

### 3. repo_map_health_check.sh / .ps1

**Purpose**: Automated health check wrapper for CI/CD integration.

**Usage**:
```bash
# Bash (Linux/Mac)
./claude/scripts/repo_map_health_check.sh [--fix] [--strict]

# PowerShell (Windows)
.\.claude\scripts\repo_map_health_check.ps1 [-Fix] [-Strict]
```

**Options**:
- `--fix` / `-Fix`: Automatically fix issues
- `--strict` / `-Strict`: Exit with error if issues found (for CI/CD gates)

### 4. context_extractor.py

**Purpose**: Extract relevant context from REPO_MAP for agent queries.

**Usage**:
```bash
python .claude/scripts/context_extractor.py --query "authentication" --max-tokens 5000
```

### 5. freshness_gate.py

**Purpose**: Check if files need re-indexing before agent handoff.

**Usage**:
```bash
python .claude/scripts/freshness_gate.py --files "core/src/auth.rs,android/app/src/MainActivity.kt"
```

## Workflow Integration

### For Agents

1. **Query Phase**: Use `context_extractor.py` to find relevant files
2. **Freshness Check**: Run `freshness_gate.py` to ensure data is current
3. **Context Loading**: Read JSONL entries for selected files
4. **Implementation**: Use extracted metadata to guide code changes

### For Developers

1. **Pre-commit Hook**: Run health check to ensure index is current
2. **CI/CD Pipeline**: Use `--strict` mode to gate merges
3. **Periodic Maintenance**: Schedule full rebuilds weekly

## Maintenance

### Daily

```bash
# Quick health check
python .claude/scripts/verify_and_fix_repo_map.py
```

### Weekly

```bash
# Full rebuild to catch any drift
python .claude/scripts/build_repo_index.py --full-rebuild

# Verify health
python .claude/scripts/verify_and_fix_repo_map.py --fix
```

### On File Changes

```bash
# Incremental update for changed files
python .claude/scripts/build_repo_index.py --incremental --files "changed_file1.rs,changed_file2.kt"
```

## Metrics

### Current Status (as of 2026-05-06)

- **Total Indexed Files**: 212
- **Missing Timestamps**: 0 (fixed)
- **Stale Files**: 2 (>24h old)
- **Index Health**: ✅ Healthy

### Staleness Thresholds

- **Fresh**: Modified < 24h ago
- **Stale**: Modified 24h-7d ago (warning)
- **Very Stale**: Modified > 7d ago (requires re-indexing)

## Troubleshooting

### Issue: Missing Timestamps

**Symptom**: `file_modified_at` is empty or null

**Fix**:
```bash
python .claude/scripts/verify_and_fix_repo_map.py --fix
```

### Issue: Stale Files

**Symptom**: Files modified after indexing

**Fix**:
```bash
# Re-index specific files
python .claude/scripts/build_repo_index.py --incremental --files "stale_file.rs"

# Or full rebuild
python .claude/scripts/build_repo_index.py --full-rebuild
```

### Issue: Orphaned Entries

**Symptom**: Index contains files that no longer exist

**Fix**: Manual cleanup required - remove entries from `repo_map_index.json`

## Best Practices

1. **Run health checks before agent handoffs**
2. **Keep index fresh** - rebuild after major refactors
3. **Monitor staleness** - files >7d old should be re-indexed
4. **Use incremental updates** for small changes
5. **Full rebuild weekly** to prevent drift
6. **Integrate with CI/CD** to prevent regressions

## Token Efficiency

### Without REPO_MAP

- Agent reads entire files to understand structure
- Average: 500-2000 tokens per file
- For 212 files: ~100,000-400,000 tokens

### With REPO_MAP

- Agent reads structured metadata only
- Average: 50-200 tokens per file
- For 212 files: ~10,000-40,000 tokens

**Savings**: 90% reduction in token usage for code discovery

## Future Enhancements

- [ ] Automatic re-indexing on file save
- [ ] Semantic search across REPO_MAP
- [ ] Dependency graph generation
- [ ] Integration with IDE extensions
- [ ] Real-time staleness monitoring
- [ ] Automated ticket generation for stale files

## Contributing

When adding new scripts or modifying the REPO_MAP system:

1. Update this README
2. Add verification tests
3. Ensure backward compatibility
4. Document breaking changes
5. Update health check scripts

## Support

For issues or questions:
1. Check verification report: `HANDOFF_AUDIT/repo_map_verification_report.txt`
2. Run health check with verbose output
3. Review script logs in `.claude/logs/`
