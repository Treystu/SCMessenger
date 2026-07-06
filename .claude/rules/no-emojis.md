# No Emojis Rule

**NEVER use emojis in any code, scripts, documentation, comments, logs, or
output — no exceptions, repo-wide.** Hook-enforced on every Edit/Write
(`.claude/hooks/check_no_emoji.py`).

Why: encoding breakage in PowerShell/terminals, script-parsing failures,
reduced portability, harder log parsing.

Instead use plain-text tags: `[OK]`, `[ERROR]`, `[WARNING]`, `[INFO]`,
`[DONE]`, `[FAIL]` — plus ANSI color codes for terminal output when useful.
When editing a file that already contains emoji, strip them as part of the
edit (the hook scans the whole file, not just your change).
