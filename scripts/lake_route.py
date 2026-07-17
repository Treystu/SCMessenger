#!/usr/bin/env python3
"""
lake_route.py -- quota-aware lake/model router for SCMessenger orchestration.

Reads tmp/lakes/registry.json (lake capabilities) + tmp/lakes/ledger.jsonl
(observed 429s / cooldowns) + tmp/lakes/round_robin_state.json (rotation
counters) and prints the best available "provider model" for a requested tier.

USAGE:
  python scripts/lake_route.py --tier FLASH
  python scripts/lake_route.py --tier CODER
  python scripts/lake_route.py --tier THINK
  python scripts/lake_route.py --tier MAX
  python scripts/lake_route.py --tier MORPH
  python scripts/lake_route.py --probe-groq   # print Groq rate-limit headroom

The "probe-groq" mode makes a real tiny API call to Groq and prints the
x-ratelimit-remaining-tokens header so the orchestrator knows whether a
full-size prompt or a micro-chunk is needed.

Output (normal mode): exactly two space-separated tokens on stdout:
  <provider> <model>
e.g.:
  groq llama-3.1-8b-instant

Exit codes: 0 = found, 1 = no lake available for this tier.
"""
import argparse
import json
import os
import sys
import urllib.request
import urllib.error
from datetime import datetime, timezone

REGISTRY_PATH = "tmp/lakes/registry.json"
LEDGER_PATH = "tmp/lakes/ledger.jsonl"
RR_STATE_PATH = "tmp/lakes/round_robin_state.json"

# Tier preference ladders (first lake with quota wins)
TIER_LADDERS = {
    "FLASH":  ["groq", "qwen", "openrouter", "gemini", "ollama"],
    "CODER":  ["qwen", "groq", "openrouter", "gemini", "ollama"],
    "THINK":  ["qwen", "gemini", "openrouter", "groq"],
    "MAX":    ["qwen", "gemini", "openrouter"],
    "MORPH":  ["openrouter"],
}

def _load_json(path, default):
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (OSError, json.JSONDecodeError):
        return default

def _load_ledger(path):
    """Return dict: lake -> dict: model -> cooldown_until ISO string (or None)."""
    cooldowns = {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("{\"comment\""):
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                lake = entry.get("lake")
                model = entry.get("model")
                cu = entry.get("cooldown_until")
                if lake and model and cu:
                    cooldowns.setdefault(lake, {})[model] = cu
    except OSError:
        pass
    return cooldowns

def _is_cooled_down(cooldown_until_str):
    """True if the cooldown has expired (safe to use), False if still in cooldown."""
    if not cooldown_until_str:
        return True
    try:
        # Parse ISO 8601 with optional Z suffix
        s = cooldown_until_str.replace("Z", "+00:00")
        until = datetime.fromisoformat(s)
        return datetime.now(timezone.utc) >= until
    except ValueError:
        return True  # Unparseable -> assume expired

def _rr_advance(rr_state, lake, tier, model_list):
    """Return next model from rotation, advance counter in rr_state."""
    key = rr_state.setdefault(lake, {})
    idx = key.get(tier, 0)
    model = model_list[idx % len(model_list)]
    key[tier] = (idx + 1) % len(model_list)
    return model

def _save_rr(rr_state):
    os.makedirs(os.path.dirname(RR_STATE_PATH), exist_ok=True)
    with open(RR_STATE_PATH, "w", encoding="utf-8") as f:
        json.dump(rr_state, f)

def route(tier):
    """Return (lake, model) for the given tier, or (None, None) if nothing available."""
    ladder = TIER_LADDERS.get(tier.upper())
    if not ladder:
        print(f"[ERROR] unknown tier '{tier}'. Valid: {list(TIER_LADDERS)}", file=sys.stderr)
        return None, None

    registry = _load_json(REGISTRY_PATH, {})
    cooldowns = _load_ledger(LEDGER_PATH)
    rr_state = _load_json(RR_STATE_PATH, {})

    for lake in ladder:
        lake_cfg = registry.get("lakes", {}).get(lake)
        if not lake_cfg:
            # Not in registry; fall through
            continue

        tiers_cfg = lake_cfg.get("tiers", {})
        model_list = tiers_cfg.get(tier.upper())
        if not model_list:
            continue

        lake_cooldowns = cooldowns.get(lake, {})

        # Find first model in round-robin order that is not in cooldown
        start_idx = rr_state.get(lake, {}).get(tier.upper(), 0)
        n = len(model_list)
        for offset in range(n):
            idx = (start_idx + offset) % n
            model = model_list[idx]
            cu = lake_cooldowns.get(model)
            if _is_cooled_down(cu):
                # Advance round-robin to this model's next position
                rr_state.setdefault(lake, {})[tier.upper()] = (idx + 1) % n
                _save_rr(rr_state)
                return lake, model

    return None, None


def probe_groq():
    """Make a minimal Groq call and print rate-limit headers to stdout."""
    # Load key
    def _key():
        k = os.environ.get("GROQ_API_KEY")
        if k:
            return k
        path = os.path.expanduser("~/.config/scmorc/groq.env")
        try:
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if "=" in line and not line.startswith("#"):
                        k2, v = line.split("=", 1)
                        if k2.strip() == "GROQ_API_KEY" and v.strip():
                            return v.strip().strip('"').strip("'")
        except OSError:
            pass
        return None

    api_key = _key()
    if not api_key:
        print("[ERROR] GROQ_API_KEY not found. Set ~/.config/scmorc/groq.env", file=sys.stderr)
        sys.exit(1)

    payload = json.dumps({
        "model": "llama-3.1-8b-instant",
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    }).encode("utf-8")
    req = urllib.request.Request(
        "https://api.groq.com/openai/v1/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "User-Agent": "curl/8.5.0"
        }
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            headers = dict(r.headers)
            # Groq returns x-ratelimit-remaining-tokens and x-ratelimit-reset-tokens
            remaining = headers.get("x-ratelimit-remaining-tokens", "unknown")
            remaining_req = headers.get("x-ratelimit-remaining-requests", "unknown")
            reset = headers.get("x-ratelimit-reset-tokens", "unknown")
            limit = headers.get("x-ratelimit-limit-tokens", "unknown")
            print(f"[Groq probe] limit={limit} remaining_tokens={remaining} remaining_requests={remaining_req} reset={reset}")
            print(f"[Groq probe] GROQ_PROMPT_TOKEN_CEILING=6000 {'OK' if remaining == 'unknown' or int(remaining) > 6000 else 'LOW -- use micro-chunk'}")
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8")
        print(f"[Groq probe] HTTP {e.code}: {body[:300]}", file=sys.stderr)
        # Still print headers if available
        print(f"[Groq probe] headers: {dict(e.headers)}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"[Groq probe] error: {e}", file=sys.stderr)
        sys.exit(1)


def main():
    ap = argparse.ArgumentParser(description="Quota-aware lake/model router.")
    ap.add_argument("--tier", help="Dispatch tier: FLASH|CODER|THINK|MAX|MORPH")
    ap.add_argument("--probe-groq", action="store_true",
                    help="Make a minimal Groq API call and print rate-limit headers.")
    args = ap.parse_args()

    if args.probe_groq:
        probe_groq()
        return

    if not args.tier:
        ap.print_help()
        sys.exit(1)

    lake, model = route(args.tier)
    if lake is None:
        print(f"[ERROR] no lake available for tier {args.tier}", file=sys.stderr)
        sys.exit(1)

    print(f"{lake} {model}")


if __name__ == "__main__":
    main()
