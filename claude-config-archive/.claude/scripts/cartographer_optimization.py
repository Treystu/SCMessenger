#!/usr/bin/env python3
"""Cartographer Optimization Harness

Tests different ollama models, context sizes, and chunk sizes against known-answer
source files to find the optimal configuration for REPO_MAP generation.

Outputs a scored comparison table.
"""

import json
import time
import argparse
import urllib.request
import urllib.error
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
OLLAMA_URL = "http://127.0.0.1:11434/api/generate"

# ── Test files with known ground truth ──────────────────────────────────────

TEST_CASES = {
    "adaptive_ttl": {
        "file": "core/src/routing/adaptive_ttl.rs",
        "ground_truth": {
            "structs": ["ActivityHistory", "AdaptiveTTLManager"],
            "key_funcs": ["new", "record_message", "calculate_ttl", "decay",
                          "record_activity", "get_activity", "cleanup",
                          "calculate_dynamic_ttl"],
            "imports_count_min": 2,
            "line_count": 250,
        }
    },
    "iron_core": {
        "file": "core/src/iron_core.rs",
        "ground_truth": {
            "structs": ["IronCore"],
            "key_funcs": ["new", "prepare_message", "send_message", "receive_message",
                          "mark_message_sent", "is_peer_blocked"],
            "imports_count_min": 10,
            "line_count": 2849,
        }
    },
    "mesh_repository": {
        "file": "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt",
        "ground_truth": {
            "structs": ["MeshRepository"],
            "key_funcs": ["connect", "disconnect", "sendMessage", "getPeerList"],
            "imports_count_min": 5,
            "line_count": 1800,
        }
    },
}

# ── Prompt templates ───────────────────────────────────────────────────────

PROMPT_TEMPLATES = {
    "v1_basic": """You are an expert code cartographer. Analyze this source code and extract its architecture.

Output RAW JSON ONLY. No markdown. No backticks. No explanation.

Schema:
{{
  "file": "RELATIVE_PATH",
  "chunk": CHUNK_NUM,
  "summary": "One sentence summary",
  "structs_or_classes": ["list of struct/class/enum/trait names"],
  "imports": ["list of import statements"],
  "funcs": [
    {{"name": "func_name", "line": LINE_NUM, "calls_out_to": ["called_funcs"]}}
  ]
}}

CODE (Lines {start}-{end}):
{code}""",

    "v2_structured": """Analyze this code chunk and produce a JSON architecture map.

RULES:
- Use EXACT line numbers from the code (the number before the colon)
- Include ALL functions defined in this chunk
- Include ALL structs, enums, traits, classes, interfaces, objects
- List ALL imports/use statements
- For calls_out_to, list functions this function calls (not just local calls, also external)
- summary must be at least 20 characters
- Output ONLY the JSON object, no markdown fences

Schema: {{"file": str, "chunk": int, "summary": str, "structs_or_classes": [str], "imports": [str], "funcs": [{{"name": str, "line": int, "calls_out_to": [str]}}]}}

File: {file_path}
Chunk: {chunk_num}
Lines {start}-{end}:

{code}""",

    "v3_minimal": """Extract architecture from this code. JSON only, no prose.

{{
  "file": "{file_path}",
  "chunk": {chunk_num},
  "summary": "...",
  "structs_or_classes": [...],
  "imports": [...],
  "funcs": [{{"name": "...", "line": 0, "calls_out_to": [...]}}]
}}

Lines {start}-{end}:
{code}"""
}


def call_ollama(prompt, model, num_ctx=4096, temperature=0.1, timeout=300):
    """Call ollama and return (response_text, latency_seconds, token_counts)."""
    body = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {
            "num_ctx": num_ctx,
            "temperature": temperature,
            "num_thread": 4
        }
    })

    start = time.time()
    try:
        req = urllib.request.Request(
            OLLAMA_URL,
            data=body.encode(),
            headers={"Content-Type": "application/json"}
        )
        resp = urllib.request.urlopen(req, timeout=timeout)
        result = json.loads(resp.read().decode())
        latency = time.time() - start
        return (
            result.get("response", ""),
            latency,
            result.get("prompt_eval_count", 0),
            result.get("eval_count", 0),
            result.get("eval_count", 0) / max(latency, 0.001)  # tokens/sec
        )
    except Exception as e:
        latency = time.time() - start
        return (f"ERROR: {e}", latency, 0, 0, 0)


def parse_json_response(response_text):
    """Parse JSON from model response, handling artifacts."""
    text = response_text.strip()
    if text.startswith("```json"):
        text = text[7:]
    elif text.startswith("```"):
        text = text[3:]
    if text.endswith("```"):
        text = text[:-3]
    text = text.strip()

    start = text.find("{")
    end = text.rfind("}")
    if start >= 0 and end > start:
        text = text[start:end + 1]

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return None


def score_entry(entry, ground_truth):
    """Score a parsed entry against ground truth. Returns (score, details)."""
    if entry is None:
        return (0, {"error": "Failed to parse JSON"})

    score = 0
    max_score = 100
    details = {}

    # File path correct (10 pts)
    if entry.get("file", "") == ground_truth.get("file", "MISSING"):
        score += 10
    details["file_correct"] = entry.get("file", "") == ground_truth.get("file", "")

    # Summary quality (10 pts) — must be >20 chars and not placeholder
    summary = entry.get("summary", "")
    if summary and len(summary) > 20 and "REPLACE" not in summary:
        score += 10
    details["summary_quality"] = "good" if len(summary) > 20 else "poor"

    # Structs/classes (25 pts) — fraction of ground truth found
    gt_structs = set(ground_truth.get("structs", []))
    found_structs = set(entry.get("structs_or_classes", []))
    if gt_structs:
        recall = len(gt_structs & found_structs) / len(gt_structs)
        score += int(25 * recall)
    details["structs_recall"] = f"{len(gt_structs & found_structs)}/{len(gt_structs)}"

    # Key functions (25 pts) — fraction found
    gt_funcs = set(ground_truth.get("key_funcs", []))
    found_funcs = set(f.get("name", "") for f in entry.get("funcs", []))
    if gt_funcs:
        recall = len(gt_funcs & found_funcs) / len(gt_funcs)
        score += int(25 * recall)
    details["funcs_recall"] = f"{len(gt_funcs & found_funcs)}/{len(gt_funcs)}"

    # Line numbers valid (15 pts) — must be numeric and > 0
    funcs = entry.get("funcs", [])
    valid_lines = sum(1 for f in funcs if isinstance(f.get("line"), (int, float)) and f.get("line", 0) > 0)
    if funcs:
        score += int(15 * valid_lines / len(funcs))
    details["valid_lines"] = f"{valid_lines}/{len(funcs)}"

    # No placeholders (15 pts)
    has_placeholder = any(
        "REPLACE" in f.get("name", "") or f.get("name", "") == "function_name"
        for f in funcs
    )
    if not has_placeholder and summary and "REPLACE" not in summary:
        score += 15
    details["no_placeholders"] = not has_placeholder

    return (score, details)


def chunk_file(filepath, chunk_size, start_line=1):
    """Read a file and return a chunk of the given size."""
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        lines = f.readlines()

    end_line = min(start_line + chunk_size - 1, len(lines))
    chunk_lines = lines[start_line - 1:end_line]
    numbered = "\n".join(f"{start_line + i}: {line.rstrip()}" for i, line in enumerate(chunk_lines))
    return numbered, start_line, end_line


def main():
    parser = argparse.ArgumentParser(description="Cartographer Optimization Harness")
    parser.add_argument("--models", type=str, default="qwen2.5-coder:7b,qwen2.5-coder:3b,qwen2.5-coder:1.5b,qwen2.5-coder:0.5b",
                        help="Comma-separated list of models to test")
    parser.add_argument("--ctx-sizes", type=str, default="2048,4096,8192",
                        help="Comma-separated list of num_ctx values to test")
    parser.add_argument("--chunk-sizes", type=str, default="200,400,700",
                        help="Comma-separated list of chunk sizes to test")
    parser.add_argument("--prompts", type=str, default="v1_basic,v2_structured,v3_minimal",
                        help="Comma-separated list of prompt template versions")
    parser.add_argument("--test-case", type=str, default="adaptive_ttl",
                        help="Which test case to use")
    parser.add_argument("--timeout", type=int, default=300, help="Per-request timeout in seconds")
    args = parser.parse_args()

    models = [m.strip() for m in args.models.split(",")]
    ctx_sizes = [int(c.strip()) for c in args.ctx_sizes.split(",")]
    chunk_sizes = [int(s.strip()) for s in args.chunk_sizes.split(",")]
    prompt_versions = [p.strip() for p in args.prompts.split(",")]

    test_case = TEST_CASES.get(args.test_case)
    if not test_case:
        print(f"ERROR: Unknown test case '{args.test_case}'. Available: {list(TEST_CASES.keys())}")
        sys.exit(1)

    filepath = str(REPO_ROOT / test_case["file"])
    ground_truth = test_case["ground_truth"]

    print(f"\n{'='*100}")
    print(f"CARTOGRAPHER OPTIMIZATION HARNESS")
    print(f"Test file: {test_case['file']}")
    print(f"Ground truth: {len(ground_truth['structs'])} structs, {len(ground_truth['key_funcs'])} key funcs")
    print(f"{'='*100}\n")

    results = []

    for model in models:
        for ctx_size in ctx_sizes:
            for chunk_size in chunk_sizes:
                for prompt_ver in prompt_versions:
                    template = PROMPT_TEMPLATES[prompt_ver]

                    # Prepare the chunk
                    numbered, start, end = chunk_file(filepath, chunk_size)

                    prompt = template.format(
                        file_path=test_case["file"],
                        chunk_num=1,
                        start=start,
                        end=end,
                        code=numbered
                    )

                    config_name = f"{model.split(':')[0]}|ctx{ctx_size}|chk{chunk_size}|{prompt_ver}"
                    print(f"Testing: {config_name}...", end=" ", flush=True)

                    response, latency, prompt_tokens, eval_tokens, tps = call_ollama(
                        prompt, model, num_ctx=ctx_size, timeout=args.timeout
                    )

                    parsed = parse_json_response(response)
                    score, details = score_entry(parsed, ground_truth)

                    result = {
                        "config": config_name,
                        "model": model,
                        "ctx_size": ctx_size,
                        "chunk_size": chunk_size,
                        "prompt_ver": prompt_ver,
                        "score": score,
                        "latency": round(latency, 2),
                        "prompt_tokens": prompt_tokens,
                        "eval_tokens": eval_tokens,
                        "tokens_per_sec": round(tps, 1),
                        "details": details,
                    }
                    results.append(result)

                    print(f"Score: {score}/100  Latency: {latency:.1f}s  TPS: {tps:.0f}")

    # Sort by score desc, then latency asc
    results.sort(key=lambda r: (-r["score"], r["latency"]))

    print(f"\n{'='*100}")
    print(f"RESULTS (sorted by score desc, latency asc)")
    print(f"{'='*100}")
    print(f"{'Config':<45} {'Score':>5} {'Lat(s)':>7} {'TPS':>8} {'Details':>30}")
    print(f"{'-'*45} {'-'*5} {'-'*7} {'-'*8} {'-'*30}")

    for r in results:
        detail_str = f"structs:{r['details'].get('structs_recall','?')} funcs:{r['details'].get('funcs_recall','?')}"
        print(f"{r['config']:<45} {r['score']:>5} {r['latency']:>7.1f} {r['tokens_per_sec']:>8.0f} {detail_str:>30}")

    # Find the optimal: best score, then fastest
    if results:
        best = results[0]
        print(f"\n{'='*100}")
        print(f"RECOMMENDED CONFIG:")
        print(f"  Model:        {best['model']}")
        print(f"  num_ctx:      {best['ctx_size']}")
        print(f"  chunk_size:   {best['chunk_size']}")
        print(f"  prompt:       {best['prompt_ver']}")
        print(f"  Score:        {best['score']}/100")
        print(f"  Latency:      {best['latency']}s")
        print(f"  Throughput:   {best['tokens_per_sec']} tok/s")
        print(f"{'='*100}")

    # Save raw results
    output_path = REPO_ROOT / "tmp" / "cartographer_optimization_results.json"
    output_path.parent.mkdir(exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nRaw results saved to {output_path}")


if __name__ == "__main__":
    main()