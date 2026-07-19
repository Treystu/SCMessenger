#!/usr/bin/env python3
"""
Morph Lite: Lightweight code transformation function using Morph V3 Fast.

Scope: Single-file code edits only (< 500 lines per file).
Cost cap: $0.01 default, $0.05 hard ceiling per invocation (never raised past that).
Purpose: Verify or apply tight-scoped Rust/Kotlin/TypeScript changes before orchestrator commit.

Usage:
  python morph_lite.py \
    --file <path> \
    --instruction <change-description> \
    --edit-snippet <desired-change-snippet> \
    [--verify-only] \
    [--max-cost 0.01]

Exit codes:
  0 = change applied successfully, within cost/scope limits
  1 = change rejected (out of scope, cost exceeded, or quality failed)
  2 = API error or misconfiguration
"""

import argparse
import json
import os
import sys
import subprocess
from pathlib import Path
from typing import NamedTuple, Optional

# --- Constants ---
OPENROUTER_API_URL = "https://openrouter.ai/api/v1/chat/completions"
MORPH_MODEL = "morph/morph-v3-fast"
# Default per-file line ceiling. Overridable per-invocation via --max-lines
# (fast-apply reliability/cost degrades on very large files, so keep the
# default conservative and raise deliberately for a specific cohesive file).
DEFAULT_MAX_FILE_LINES = 500
DEFAULT_MAX_COST = 0.01  # $0.01 default ceiling
HARD_MAX_COST = 0.05  # $0.05 hard ceiling, never raised past this via --max-cost
INSTRUCTION_MAX_CHARS = 1000
EDIT_SNIPPET_MAX_CHARS = 4000


class MorphConfig(NamedTuple):
    """Configuration for a Morph Lite invocation."""
    file_path: str
    instruction: str
    edit_snippet: str
    verify_only: bool
    max_cost: float
    api_key: str
    max_lines: int = DEFAULT_MAX_FILE_LINES


class MorphResult(NamedTuple):
    """Result of a Morph Lite transformation."""
    success: bool
    applied_code: Optional[str]
    error_msg: str
    cost_usd: float
    tokens_in: int
    tokens_out: int


def validate_config(cfg: MorphConfig) -> Optional[str]:
    """
    Validate configuration constraints.
    Returns error message if invalid, None if OK.
    """
    if not Path(cfg.file_path).exists():
        return f"File not found: {cfg.file_path}"

    file_lines = len(Path(cfg.file_path).read_text().splitlines())
    if file_lines > cfg.max_lines:
        return f"File too large: {file_lines} lines (max {cfg.max_lines})"

    if len(cfg.instruction) > INSTRUCTION_MAX_CHARS:
        return f"Instruction too long: {len(cfg.instruction)} chars (max {INSTRUCTION_MAX_CHARS})"

    if len(cfg.edit_snippet) > EDIT_SNIPPET_MAX_CHARS:
        return f"Edit snippet too long: {len(cfg.edit_snippet)} chars (max {EDIT_SNIPPET_MAX_CHARS})"

    if cfg.max_cost > HARD_MAX_COST:
        return f"Cost cap ${cfg.max_cost:.6f} exceeds hard limit ${HARD_MAX_COST:.6f}"

    return None


def call_morph(cfg: MorphConfig) -> MorphResult:
    """
    Call Morph V3 Fast via OpenRouter for code transformation.

    Returns MorphResult with applied code, cost, and token counts.
    """
    initial_code = Path(cfg.file_path).read_text()

    # Build the prompt in Morph's required format
    prompt = (
        f"<instruction>{cfg.instruction}</instruction>\n"
        f"<code>{initial_code}</code>\n"
        f"<update>{cfg.edit_snippet}</update>"
    )

    headers = {
        "Authorization": f"Bearer {cfg.api_key}",
        "Content-Type": "application/json",
        "HTTP-Referer": "https://github.com/SCMessenger/scmessenger",
        "X-Title": "morph_lite",
    }

    payload = {
        "model": MORPH_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.1,  # Low temp for deterministic code edits
        "max_tokens": 8192,  # Morph handles code efficiently
    }

    try:
        response = subprocess.run(
            [
                "curl",
                "-s",
                "-X", "POST",
                OPENROUTER_API_URL,
                "-H", f"Authorization: Bearer {cfg.api_key}",
                "-H", "Content-Type: application/json",
                "-d", json.dumps(payload),
            ],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=30,
        )

        if response.returncode != 0:
            return MorphResult(
                success=False,
                applied_code=None,
                error_msg=f"curl failed: {response.stderr}",
                cost_usd=0.0,
                tokens_in=0,
                tokens_out=0,
            )

        result = json.loads(response.stdout)

        # Check for API errors
        if "error" in result:
            return MorphResult(
                success=False,
                applied_code=None,
                error_msg=result["error"].get("message", "Unknown API error"),
                cost_usd=0.0,
                tokens_in=0,
                tokens_out=0,
            )

        # Extract response
        if "choices" not in result or not result["choices"]:
            return MorphResult(
                success=False,
                applied_code=None,
                error_msg="No choices in API response",
                cost_usd=0.0,
                tokens_in=0,
                tokens_out=0,
            )

        applied_code = result["choices"][0]["message"]["content"].strip()
        tokens_in = result.get("usage", {}).get("prompt_tokens", 0)
        tokens_out = result.get("usage", {}).get("completion_tokens", 0)

        # Calculate cost
        # Morph V3 Fast: $0.80/M input, $1.20/M output
        cost_usd = (tokens_in * 0.80 + tokens_out * 1.20) / 1_000_000

        # Hard cost gate
        if cost_usd > cfg.max_cost:
            return MorphResult(
                success=False,
                applied_code=None,
                error_msg=f"Cost ${cost_usd:.6f} exceeds limit ${cfg.max_cost:.6f}",
                cost_usd=cost_usd,
                tokens_in=tokens_in,
                tokens_out=tokens_out,
            )

        return MorphResult(
            success=True,
            applied_code=applied_code,
            error_msg="",
            cost_usd=cost_usd,
            tokens_in=tokens_in,
            tokens_out=tokens_out,
        )

    except subprocess.TimeoutExpired:
        return MorphResult(
            success=False,
            applied_code=None,
            error_msg="API call timeout (30s)",
            cost_usd=0.0,
            tokens_in=0,
            tokens_out=0,
        )
    except json.JSONDecodeError as e:
        return MorphResult(
            success=False,
            applied_code=None,
            error_msg=f"Invalid JSON response: {e}",
            cost_usd=0.0,
            tokens_in=0,
            tokens_out=0,
        )
    except Exception as e:
        return MorphResult(
            success=False,
            applied_code=None,
            error_msg=f"Unexpected error: {e}",
            cost_usd=0.0,
            tokens_in=0,
            tokens_out=0,
        )


def apply_code(file_path: str, new_code: str) -> bool:
    """Write the transformed code to the file. Returns True on success."""
    try:
        Path(file_path).write_text(new_code)
        return True
    except Exception as e:
        print(f"[ERROR] Failed to write file: {e}", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Morph Lite: Scoped code transformation with cost ceiling.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
REQUIRED:
  --file <path>          File to transform (max 500 lines)
  --instruction <text>   Change description (max 1000 chars)
  --edit-snippet <text>  Desired edit snippet (max 2000 chars)

OPTIONS:
  --verify-only          Fetch transformation but don't apply (dry run)
  --max-cost <usd>       Cost ceiling; default $0.01, hard max $0.05 (never raised past that)

ENVIRONMENT:
  OPENROUTER_API_KEY     API key for OpenRouter (required)

EXIT CODES:
  0 = success
  1 = validation/cost/quality gate failed
  2 = API error or misconfiguration
        """,
    )

    parser.add_argument("--file", required=True, help="File path to transform")
    parser.add_argument("--instruction", required=True, help="Change instruction")
    parser.add_argument("--edit-snippet", required=True, help="Edit snippet")
    parser.add_argument("--verify-only", action="store_true", help="Dry run (no write)")
    parser.add_argument(
        "--max-cost",
        type=float,
        default=DEFAULT_MAX_COST,
        help=f"Cost ceiling (default: ${DEFAULT_MAX_COST:.6f}, hard max ${HARD_MAX_COST:.6f})",
    )
    parser.add_argument(
        "--max-lines",
        type=int,
        default=DEFAULT_MAX_FILE_LINES,
        help=f"Per-file line ceiling (default: {DEFAULT_MAX_FILE_LINES}). Raise deliberately "
        "for a specific cohesive file; fast-apply reliability/cost degrades on large files.",
    )

    args = parser.parse_args()

    # Key policy (operator, 2026-07-17): Morph runs on the PAID budget lane.
    # The paid key is shared with Fusion Lite at openrouter_fusion.env
    # ($0.50 cap -- combined spend across both tools). The general
    # openrouter.env key is FREE-MODELS-ONLY and must never be used here:
    # morph/morph-v3-fast is a paid model and would silently spend on the
    # no-paid-usage key.
    api_key = os.environ.get("OPENROUTER_API_KEY")
    key_source = "env OPENROUTER_API_KEY" if api_key else None
    if not api_key:
        key_file = os.path.expanduser("~/.config/scmorc/openrouter_fusion.env")
        try:
            with open(key_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("OPENROUTER_API_KEY=") and "=" in line:
                        api_key = line.split("=", 1)[1].strip().strip('"').strip("'")
                        key_source = key_file
                        break
        except OSError:
            pass
    if not api_key:
        print("[ERROR] No PAID OpenRouter key found. Morph Lite requires the paid lane:", file=sys.stderr)
        print("  set OPENROUTER_API_KEY, or create ~/.config/scmorc/openrouter_fusion.env", file=sys.stderr)
        print("  (shared Fusion+Morph paid budget, $0.50 cap). The free-only openrouter.env", file=sys.stderr)
        print("  key is refused for paid morph calls.", file=sys.stderr)
        sys.exit(2)
    print(f"[INFO] Morph key source: {key_source} (shared Fusion+Morph paid budget)", file=sys.stderr)

    cfg = MorphConfig(
        file_path=args.file,
        instruction=args.instruction,
        edit_snippet=args.edit_snippet,
        verify_only=args.verify_only,
        max_cost=args.max_cost,
        api_key=api_key,
        max_lines=args.max_lines,
    )

    # Validate upfront
    error = validate_config(cfg)
    if error:
        print(f"[ERROR] {error}", file=sys.stderr)
        sys.exit(1)

    # Call Morph
    result = call_morph(cfg)

    # Report
    print(
        f"[RESULT] {'PASS' if result.success else 'FAIL'} | "
        f"cost=${result.cost_usd:.6f} | "
        f"tokens_in={result.tokens_in} tokens_out={result.tokens_out}"
    )

    if not result.success:
        print(f"[ERROR] {result.error_msg}", file=sys.stderr)
        sys.exit(1)

    if not result.applied_code:
        print("[ERROR] No code returned from Morph", file=sys.stderr)
        sys.exit(2)

    if not args.verify_only:
        if not apply_code(cfg.file_path, result.applied_code):
            sys.exit(1)
        print(f"[OK] Applied to {cfg.file_path}")
    else:
        print(f"[DRY-RUN] Transformation ready (not applied)")
        print("--- PROPOSED ---")
        print(result.applied_code[:500])  # Preview first 500 chars
        print("--- END PREVIEW ---")

    sys.exit(0)


if __name__ == "__main__":
    main()
