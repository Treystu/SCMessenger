#!/usr/bin/env python3
import argparse
import urllib.request
import json
import os
import re
import sys
import subprocess

QWEN_URL = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"
OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
OLLAMA_URL = "http://localhost:11434/api/chat"

QWEN_TIER_MAP = {
    "thinking": "qwen3-vl-235b-a22b-thinking",  # Architecture, security review, adversarial audit
    "max":      "qwen3-max",                      # Complex Rust impl, crypto, multi-file changes
    "standard": "qwen3.5-122b-a10b",              # Compile fixes, mechanical refactors, moderate tasks
    "plus":     "qwen-plus-2025-07-28",           # Docs, task file generation, planning
    "flash":    "qwen-max",                       # Simple fixes, small scoped changes, fallback
}

VALID_EXTENSIONS = ('.rs', '.toml', '.md', '.py', '.sh')

def _key_from_env_file(path, names):
    try:
        with open(os.path.expanduser(path), "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    if k.strip() in names and v.strip():
                        return v.strip().strip('"').strip("'")
    except OSError:
        pass
    return None

def get_api_key(provider):
    if provider == "qwen":
        return (os.environ.get("QWEN_API_KEY")
                or os.environ.get("DASHSCOPE_API_KEY")
                or _key_from_env_file("~/.config/scmorc/dashscope.env",
                                      ("QWEN_API_KEY", "DASHSCOPE_API_KEY")))
    elif provider == "openrouter":
        return (os.environ.get("OPENROUTER_API_KEY")
                or _key_from_env_file("~/.config/scmorc/openrouter.env",
                                      ("OPENROUTER_API_KEY",)))
    return None

def extract_file_blocks(content):
    """
    Extract (filename, file_content) pairs from model output.
    Handles two patterns:
      Pattern A: filename comment INSIDE code block as first line
        ```rust
        // path/to/file.rs
        ... code ...
        ```
      Pattern B: filename comment BEFORE code block (Qwen default)
        // path/to/file.rs
        ```python
        ... code ...
        ```
    """
    results = []

    # Pattern A: filename inside block (any language tag)
    for block in re.finditer(r"```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL):
        lines = block.group(1).split("\n")
        first = lines[0].strip()
        if first.startswith("// ") or first.startswith("# "):
            filename = first.replace("// ", "").replace("# ", "").replace("filename: ", "").strip()
            if filename.endswith(VALID_EXTENSIONS):
                results.append((filename, "\n".join(lines[1:])))

    # Pattern B: filename before block (e.g. "// scripts/foo.py\n```python\n...")
    for m in re.finditer(r"^(//\s*\S+\.\w+)\s*\n```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL | re.MULTILINE):
        filename = m.group(1).replace("// ", "").strip()
        if filename.endswith(VALID_EXTENSIONS) and not any(f == filename for f, _ in results):
            results.append((filename, m.group(2)))

    return results

def send_request(args, prompt, resolved_model, display_model, round_num=None):
    payload = {
        "model": resolved_model,
        "temperature": 0.1
    }

    if args.provider == "ollama":
        payload["messages"] = [
            {"role": "system", "content": "You are a senior Rust engineer. Strictly provide full file contents in code blocks with // filename as the first line inside the block."},
            {"role": "user", "content": prompt}
        ]
        payload["stream"] = False
        req_url = OLLAMA_URL
        headers = {"Content-Type": "application/json"}
    else:
        payload["messages"] = [
            {"role": "system", "content": "You are a senior Rust engineer. Strictly provide full file contents in code blocks with // filename as the first line inside the block."},
            {"role": "user", "content": prompt}
        ]
        req_url = QWEN_URL if args.provider == "qwen" else OPENROUTER_URL
        api_key = get_api_key(args.provider)
        if not api_key:
            print(f"Error: API key for {args.provider} is not set.")
            sys.exit(1)
        headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        }

    req = urllib.request.Request(req_url, headers=headers, data=json.dumps(payload).encode("utf-8"))

    try:
        with urllib.request.urlopen(req, timeout=600) as r:
            resp = json.loads(r.read().decode("utf-8"))

            if args.provider == "ollama":
                content = resp.get("message", {}).get("content", "")
            else:
                content = resp["choices"][0]["message"]["content"]

            os.makedirs("tmp", exist_ok=True)
            base_name = os.path.basename(args.task).split('.')[0]
            if round_num is not None:
                response_file = f"tmp/{base_name}_response_round{round_num}.md"
            else:
                response_file = f"tmp/{base_name}_response.md"
            with open(response_file, "w", encoding="utf-8") as f:
                f.write(content)

            return content, response_file

    except urllib.error.HTTPError as e:
        print(f"HTTP error: {e.code} - {e.read().decode('utf-8')}")
        sys.exit(1)
    except urllib.error.URLError as e:
        print(f"Network error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error processing request: {e}")
        sys.exit(1)

def apply_file_blocks(file_blocks):
    if not file_blocks:
        return False
    for filename, file_content in file_blocks:
        print(f"Applying updates to {filename}...")
        dir_name = os.path.dirname(filename)
        if dir_name:
            os.makedirs(dir_name, exist_ok=True)
        with open(filename, "w", encoding="utf-8") as f:
            f.write(file_content)
    return True

def main():
    parser = argparse.ArgumentParser(description="Universal Swarm Delegate Script")
    parser.add_argument("--task", required=True, help="Task markdown file path (e.g., HANDOFF/todo/PQC_07_PQ_RATCHET.md)")
    parser.add_argument("--provider", choices=["qwen", "openrouter", "ollama"], required=True, help="API provider to use")
    parser.add_argument("--model", help="Model name override (e.g., qwen-max, anthropic/claude-3.5-sonnet, llama3)")
    parser.add_argument("--tier", choices=["thinking", "max", "standard", "plus", "flash"],
                        help="Qwen tier for auto model selection: thinking > max > standard > plus > flash")
    parser.add_argument("--files", nargs="*", default=[], help="List of source files to include in context")
    parser.add_argument("--apply", action="store_true", help="Auto-apply the generated code blocks back into the files")
    parser.add_argument("--verify", type=str, help='Verification command to run after applying changes (e.g., "cargo check -p scmessenger-core")')
    parser.add_argument("--max-rounds", type=int, default=3, help="Maximum number of model calls including the first one (default: 3)")

    args = parser.parse_args()

    if args.verify and not args.apply:
        print("Warning: --verify is only meaningful with --apply; ignoring --verify.")

    if not os.path.exists(args.task):
        print(f"Error: Task file {args.task} not found.")
        sys.exit(1)

    with open(args.task, "r", encoding="utf-8") as f:
        task_content = f.read()

    prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement the following:

### Requirements:
{task_content}

Please provide the FULL, completely updated/new contents of the relevant files using standard Markdown code blocks.
The exact filename must be the first line inside the code block (e.g., `// core/src/crypto/session_manager.rs`).
DO NOT output partial files, snippets, or diffs. Output the ENTIRE file content. Preserve all existing logic unless it contradicts the requirements.
"""

    if args.files:
        prompt += "\n\n### Current Relevant File Contents:\n"
        for filepath in args.files:
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
            except Exception as e:
                print(f"Warning: Could not read {filepath}: {e}")

    # Resolve model
    if args.provider == "qwen":
        if args.tier:
            resolved_model = QWEN_TIER_MAP[args.tier]
        elif args.model:
            resolved_model = args.model
        else:
            resolved_model = QWEN_TIER_MAP["max"]
    else:
        if not args.model:
            print(f"Error: --model is required for provider '{args.provider}'.")
            sys.exit(1)
        resolved_model = args.model

    if args.provider == "qwen" and args.tier:
        display_model = f"{resolved_model} [tier: {args.tier}]"
    else:
        display_model = resolved_model

    print(f"Dispatching task {os.path.basename(args.task)} to {args.provider} ({display_model})...")

    content, response_file = send_request(args, prompt, resolved_model, display_model)
    print(f"Response received and saved to {response_file}!")

    if args.apply:
        file_blocks = extract_file_blocks(content)
        if not file_blocks:
            print("Warning: No properly formatted code blocks found to apply.")
        else:
            applied = apply_file_blocks(file_blocks)
            if applied:
                print(f"Successfully applied {len(file_blocks)} file(s).")

        # Verification loop
        if args.verify and args.apply:
            verify_env = os.environ.copy()
            verify_env["CARGO_INCREMENTAL"] = "0"

            current_round = 1
            while current_round <= args.max_rounds:
                print(f"[ROUND {current_round}] Running verification command: {args.verify}")
                result = subprocess.run(
                    args.verify,
                    shell=True,
                    capture_output=True,
                    text=True,
                    env=verify_env
                )

                if result.returncode == 0:
                    print(f"[OK] verify passed on round {current_round}")
                    sys.exit(0)
                else:
                    if current_round >= args.max_rounds:
                        last_output = (result.stdout + result.stderr)[-2000:]
                        print(f"[FAIL] verify still failing after {args.max_rounds} rounds")
                        print(last_output)
                        sys.exit(2)
                    else:
                        print(f"[ROUND {current_round}] verify failed (exit {result.returncode}); re-dispatching fix...")

                        # Build follow-up prompt
                        combined_output = (result.stdout + result.stderr)[-6000:]
                        follow_up_prompt = (
                            "Your previous attempt was applied but verification failed.\n"
                            f"Verification command: {args.verify}\n"
                            f"Last 6000 characters of output:\n```\n{combined_output}\n```\n\n"
                            "### Current Relevant File Contents:\n"
                        )
                        for filepath in args.files:
                            try:
                                with open(filepath, "r", encoding="utf-8") as f:
                                    follow_up_prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
                            except Exception as e:
                                print(f"Warning: Could not read {filepath}: {e}")

                        follow_up_prompt += (
                            "\nPlease provide the FULL, completely updated/new contents of the relevant files using standard Markdown code blocks.\n"
                            "The exact filename must be the first line inside the code block (e.g., `// core/src/crypto/session_manager.rs`).\n"
                            "DO NOT output partial files, snippets, or diffs. Output the ENTIRE file content. Preserve all existing logic unless it contradicts the requirements."
                        )

                        current_round += 1
                        content, response_file = send_request(args, follow_up_prompt, resolved_model, display_model, current_round)
                        print(f"Response received and saved to {response_file}!")

                        file_blocks = extract_file_blocks(content)
                        if not file_blocks:
                            print("[WARN] round response had no applicable file blocks; counting as a failed round")
                        else:
                            apply_file_blocks(file_blocks)

if __name__ == "__main__":
    main()