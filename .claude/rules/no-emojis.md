# No Emojis Rule

## Rule

**NEVER use emojis in any code, scripts, documentation, or output.**

## Rationale

- Emojis cause encoding issues across different platforms and shells
- PowerShell and some terminals don't handle Unicode emojis correctly
- Breaks script parsing and execution
- Reduces portability and compatibility
- Makes logs harder to parse programmatically

## Instead Use

- Plain text symbols: `[OK]`, `[ERROR]`, `[WARNING]`, `[INFO]`
- ASCII art for visual separation
- Clear descriptive text
- ANSI color codes for terminal output (when appropriate)

## Examples

### ❌ BAD (with emojis)
```python
print("✅ Success!")
print("❌ Error occurred")
print("🔍 Searching...")
```

### ✅ GOOD (without emojis)
```python
print("[SUCCESS] Operation completed")
print("[ERROR] Operation failed")
print("[INFO] Searching...")
```

### ❌ BAD (with emojis)
```bash
echo "🚀 Starting deployment..."
echo "✨ Done!"
```

### ✅ GOOD (without emojis)
```bash
echo "[DEPLOY] Starting deployment..."
echo "[DONE] Deployment complete!"
```

## Applies To

- Python scripts
- Bash scripts
- PowerShell scripts
- Documentation (README, comments)
- Log messages
- Error messages
- User-facing output
- Code comments

## Exceptions

None. This rule applies universally across the entire repository.
