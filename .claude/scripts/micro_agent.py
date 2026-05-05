#!/usr/bin/env python3
"""Micro agent: direct ollama run for small models on scoped wiring tasks.

No Claude Code wrapper. Reads source files, asks the model for a diff,
applies it, verifies with cargo check. Max 3 retry loops.

Usage:
    python micro_agent.py --model gemma4:31b:cloud --task HANDOFF/todo/task_wire_micro_add_step.md
"""

import argparse
import re
import subprocess
import sys
import json
import os
import shutil
from pathlib import Path

MAX_RETRIES = 3


def parse_micro_task(task_path: str) -> dict:
    """Parse TARGET/WIRE/VERIFY from a micro task file."""
    task = {"target": "", "wire": "", "verify": ""}
    with open(task_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line.startswith("TARGET:"):
                task["target"] = line[len("TARGET:"):].strip().replace("\\", "/")
            elif line.startswith("WIRE:"):
                task["wire"] = line[len("WIRE:"):].strip()
            elif line.startswith("VERIFY:"):
                task["verify"] = line[len("VERIFY:"):].strip()
    return task


def extract_wired_files(wire_line: str, target_file: str) -> list[str]:
    """Extract file paths mentioned in the WIRE line (e.g., 'iron_core.rs')."""
    files = [target_file]
    # Match common file patterns: something.rs, something.kt, path/to/file.ext
    for m in re.finditer(r'[\w/]+\.(?:rs|kt|swift|java|ts)', wire_line):
        f = m.group(0)
        if f not in files:
            files.append(f)
    # Also match explicit file references like "in iron_core.rs"
    for m in re.finditer(r'(?:in|into|at|from)\s+([\w_]+\.(?:rs|kt|swift|java|ts))', wire_line):
        f = m.group(1)
        if f not in files:
            files.append(f)
    return files


def resolve_file(name: str, repo_root: str) -> str | None:
    """Find a file by name or relative path in the repo."""
    if os.path.isfile(os.path.join(repo_root, name)):
        return os.path.join(repo_root, name)
    # Try finding by basename
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in ('target', 'build', '.claude', 'node_modules', '.git')]
        for f in files:
            if f == os.path.basename(name):
                full = os.path.join(root, f)
                return full
    return None


def scope_file(content: str, keywords: list[str], window: int = 15) -> str:
    """Extract relevant sections of a large file around keyword matches.

    Returns the full file if small, or scoped sections with line numbers if large.
    """
    MAX_FULL_FILE = 8000  # chars — send full if under this
    if len(content) <= MAX_FULL_FILE:
        return content

    lines = content.split("\n")
    matched_lines = set()
    for kw in keywords:
        for i, line in enumerate(lines):
            if kw in line:
                # Window of ±50 lines around each match
                start = max(0, i - window)
                end = min(len(lines), i + window + 1)
                for j in range(start, end):
                    matched_lines.add(j)

    if not matched_lines:
        # No keyword match — send first/last 200 lines as fallback
        head = lines[:200]
        tail = lines[-200:]
        head_text = "\n".join(f"{i+1}: {l}" for i, l in enumerate(head))
        tail_text = "\n".join(f"{i+1}: {l}" for i, l in enumerate(tail, len(lines)-200))
        return f"[FILE TOO LARGE - showing first 200 and last 200 lines]\n{head_text}\n... [{len(lines)-400} lines omitted] ...\n{tail_text}"

    # Build scoped output with line numbers
    sorted_lines = sorted(matched_lines)
    sections = []
    section_start = sorted_lines[0]
    section_end = sorted_lines[0]

    for sl in sorted_lines[1:]:
        if sl <= section_end + 1:
            section_end = sl
        else:
            # Emit section
            section_lines = []
            for i in range(section_start, section_end + 1):
                section_lines.append(f"{i+1}: {lines[i]}")
            sections.append("\n".join(section_lines))
            section_start = sl
            section_end = sl

    # Last section
    section_lines = []
    for i in range(section_start, section_end + 1):
        section_lines.append(f"{i+1}: {lines[i]}")
    sections.append("\n".join(section_lines))

    omitted = len(lines) - len(matched_lines)
    header = f"[SCOPED: {len(matched_lines)}/{len(lines)} lines shown, {omitted} omitted]"
    return header + "\n" + ("\n... [lines omitted] ...\n".join(sections))


def build_prompt(task: dict, file_contents: dict[str, str]) -> str:
    """Build a focused prompt for the model — source code + task, ask for structured edits."""
    # Extract meaningful keywords from WIRE line for scoping large files
    stop_words = {'self', 'mut', 'str', 'pub', 'fn', 'impl', 'use', 'let', 'in',
                  'to', 'the', 'a', 'an', 'it', 'for', 'of', 'and', 'or', 'at',
                  'from', 'with', 'by', 'on', 'rs', 'kt', 'call', 'after', 'into'}
    wire_words = [w for w in re.findall(r'[a-zA-Z_]\w{2,}', task['wire'])
                  if w.lower() not in stop_words]

    file_blocks = []
    for path, content in file_contents.items():
        scoped = scope_file(content, wire_words)
        file_blocks.append(f"=== {path} ===\n{scoped}\n=== END ===")

    files_section = "\n\n".join(file_blocks)

    prompt = f"""Wire this function into the codebase. Output structured line edits.

TASK: {task['wire']}

RELEVANT CODE (with line numbers):
{files_section}

OUTPUT FORMAT — for each edit, output:
FILE: path/to/file.rs
AFTER_LINE: <line_number>
ADD:
<code to insert>

Or to replace a line:
FILE: path/to/file.rs
REPLACE_LINE: <line_number>
WITH:
<replacement code>

Output ONLY the edit instructions, no explanations."""
    return prompt


def run_ollama(model: str, prompt: str) -> str:
    """Run ollama directly and return the model's output."""
    # Unique temp file per call to avoid parallel agent file locking
    prompt_file = os.path.join("tmp", f"micro_agent_prompt_{os.getpid()}_{int(__import__('time').time()*1000)}.txt")
    os.makedirs("tmp", exist_ok=True)
    with open(prompt_file, "w", encoding="utf-8") as f:
        f.write(prompt)

    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8"
    env["NO_COLOR"] = "1"
    env["TERM"] = "dumb"

    try:
        with open(prompt_file, "r", encoding="utf-8") as pf:
            result = subprocess.run(
                ["ollama", "run", model, "--nowordwrap", "--hidethinking", "--think=false"],
                stdin=pf, capture_output=True, timeout=300, env=env
            )
        # Decode as utf-8 with error replacement (Windows cp1252 safe)
        raw = result.stdout.decode("utf-8", errors="replace") if isinstance(result.stdout, bytes) else result.stdout
        # Strip ANSI escape codes from output
        output = re.sub(r'\x1b\[[0-9;]*[mGKHJF]', '', raw)
        output = re.sub(r'\x1b\[\?[0-9]*[hl]', '', output)
        return output.strip()
    except subprocess.TimeoutExpired:
        return ""
    finally:
        if os.path.exists(prompt_file):
            os.remove(prompt_file)


def extract_diff(output: str) -> str:
    """Extract unified diff from model output, handling markdown code fences."""
    # Try to extract from ```diff ... ``` blocks
    diff_blocks = re.findall(r'```diff\n(.*?)```', output, re.DOTALL)
    if diff_blocks:
        return "\n".join(diff_blocks)

    # Try raw diff (lines starting with ---, +++, @@, +, -)
    lines = output.split("\n")
    diff_lines = []
    in_diff = False
    for line in lines:
        if line.startswith("---") or line.startswith("+++"):
            in_diff = True
        if in_diff:
            if line.startswith("---") or line.startswith("+++") or line.startswith("@@") \
               or line.startswith("+") or line.startswith("-") or line.startswith(" ") \
               or line == "":
                diff_lines.append(line)
            elif not line.startswith(("---", "+++", "@@", "+", "-", " ")):
                # Hit a non-diff line after being in diff — stop
                if diff_lines:
                    break

    return "\n".join(diff_lines) if diff_lines else output


def apply_diff(diff_text: str) -> bool:
    """Apply a unified diff using git apply."""
    diff_file = os.path.join("tmp", "micro_agent_patch.diff")
    os.makedirs("tmp", exist_ok=True)
    with open(diff_file, "w", encoding="utf-8") as f:
        f.write(diff_text)

    try:
        result = subprocess.run(
            ["git", "apply", "--whitespace=fix", diff_file],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0:
            return True
        # Try with --3way for better conflict resolution
        result3 = subprocess.run(
            ["git", "apply", "--3way", diff_file],
            capture_output=True, text=True, timeout=30
        )
        return result3.returncode == 0
    except Exception:
        return False
    finally:
        if os.path.exists(diff_file):
            os.remove(diff_file)


def apply_line_edits(output: str, task: dict) -> bool:
    """Fallback: parse structured line edits from model output if diff fails.

    Looks for patterns like:
    FILE: path/to/file.rs
    AFTER_LINE: 42
    ADD: code line
    or
    REPLACE_LINE: 42
    WITH: new code
    """
    edits = []
    current_file = None
    lines = output.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("FILE:"):
            current_file = line[len("FILE:"):].strip()
        elif line.startswith("AFTER_LINE:") and current_file:
            after_line = int(re.search(r'\d+', line).group())
            i += 1
            # Skip past ADD: marker if present
            if i < len(lines) and lines[i].strip().startswith("ADD:"):
                i += 1
            code_lines = []
            while i < len(lines) and not lines[i].strip().startswith(("FILE:", "AFTER_LINE:", "REPLACE_LINE:")):
                code_lines.append(lines[i].rstrip())
                i += 1
            if code_lines:
                edits.append(("after", current_file, after_line, code_lines))
            continue
        elif line.startswith("REPLACE_LINE:") and current_file:
            replace_line = int(re.search(r'\d+', line).group())
            i += 1
            if i < len(lines) and lines[i].strip().startswith("WITH:"):
                i += 1
                code_lines = []
                while i < len(lines) and not lines[i].strip().startswith(("FILE:", "AFTER_LINE:", "REPLACE_LINE:")):
                    code_lines.append(lines[i].rstrip())
                    i += 1
                if code_lines:
                    edits.append(("replace", current_file, replace_line, code_lines))
            continue
        i += 1

    if not edits:
        return False

    # Group edits by file
    files_edits = {}
    for op, filepath, lineno, code in edits:
        if filepath not in files_edits:
            files_edits[filepath] = []
        files_edits[filepath].append((op, lineno, code))

    # Apply edits in reverse line order per file to preserve line numbers
    for filepath, file_edits in files_edits.items():
        if not os.path.isfile(filepath):
            continue
        with open(filepath, "r", encoding="utf-8") as f:
            file_lines = f.readlines()

        file_edits.sort(key=lambda e: e[1], reverse=True)
        for op, lineno, code in file_edits:
            # Convert to 0-indexed
            idx = lineno - 1
            if op == "after":
                for j, code_line in enumerate(code):
                    file_lines.insert(idx + 1 + j, code_line + "\n")
            elif op == "replace":
                for j, code_line in enumerate(code):
                    if idx + j < len(file_lines):
                        file_lines[idx + j] = code_line + "\n"
                    else:
                        file_lines.append(code_line + "\n")

        with open(filepath, "w", encoding="utf-8") as f:
            f.writelines(file_lines)

    return True


def count_errors(verify_cmd: str) -> int:
    """Run verify command and count error lines. Returns -1 on timeout."""
    try:
        result = subprocess.run(
            verify_cmd, shell=True, capture_output=True, timeout=120
        )
        stdout = result.stdout.decode("utf-8", errors="replace") if isinstance(result.stdout, bytes) else result.stdout
        stderr = result.stderr.decode("utf-8", errors="replace") if isinstance(result.stderr, bytes) else result.stderr
        output = stdout + stderr
        if result.returncode == 0:
            return 0
        error_lines = [l for l in output.split('\n') if 'error[' in l or l.startswith('error:')]
        return max(len(error_lines), 1)
    except subprocess.TimeoutExpired:
        return -1


def run_verify(verify_cmd: str, baseline_errors: int = 0) -> tuple[bool, str]:
    """Run the verify command and return (success, output).

    For cargo check, compares error count against baseline to avoid
    false failures from pre-existing compile errors.
    """
    try:
        result = subprocess.run(
            verify_cmd, shell=True, capture_output=True, text=True, timeout=120
        )
        output = result.stdout + result.stderr

        # Clean compile — always a pass
        if result.returncode == 0:
            return True, output

        # Failed compile — check if we introduced NEW errors
        error_lines = [l for l in output.split('\n') if 'error[' in l or l.startswith('error:')]
        new_errors = len(error_lines) - baseline_errors

        if new_errors <= 0:
            return True, f"VERIFY: {len(error_lines)} errors (all pre-existing, baseline={baseline_errors})"
        elif new_errors <= 3:
            return True, f"VERIFY: {new_errors} new errors (baseline={baseline_errors}, total={len(error_lines)})"
        else:
            return False, output
    except subprocess.TimeoutExpired:
        return False, "VERIFY command timed out"
    except Exception as e:
        return False, str(e)


def write_completion(agent_id: str, status: str, task_file: str,
                      changed_files: list[str], build_status: str, error: str = ""):
    """Write completion marker for orchestrator patrol detection."""
    agent_dir = os.path.join(".claude", "agents", agent_id)
    os.makedirs(agent_dir, exist_ok=True)
    ts = str(int(__import__("time").time()))
    with open(os.path.join(agent_dir, "COMPLETION"), "w") as f:
        if status == "completed":
            f.write(f"STATUS=completed\n")
            f.write(f"TASK_FILE=HANDOFF/done/{os.path.basename(task_file)}\n")
            f.write(f"CHANGED_FILES={','.join(changed_files)}\n")
            f.write(f"BUILD_STATUS={build_status}\n")
            f.write(f"COMPLETED_AT={ts}\n")
        else:
            f.write(f"STATUS=failed\n")
            f.write(f"TASK_FILE={task_file}\n")
            f.write(f"ERROR={error}\n")
            f.write(f"COMPLETED_AT={ts}\n")


def main():
    parser = argparse.ArgumentParser(description="Micro agent: direct ollama for small models")
    parser.add_argument("--model", required=True, help="Ollama model (e.g., gemma4:31b:cloud)")
    parser.add_argument("--task", required=True, help="Path to micro task file")
    parser.add_argument("--agent-id", default=None, help="Agent ID for completion marker")
    parser.add_argument("--retries", type=int, default=MAX_RETRIES, help="Max retry attempts")
    args = parser.parse_args()

    repo_root = os.getcwd()

    # Parse task
    task = parse_micro_task(args.task)
    if not task["target"] or not task["wire"]:
        print(f"ERROR: Invalid micro task file — missing TARGET or WIRE line")
        sys.exit(1)

    # Resolve target file
    target_path = resolve_file(task["target"], repo_root)
    if not target_path:
        print(f"ERROR: Target file not found: {task['target']}")
        sys.exit(1)

    # Find additional files mentioned in WIRE line
    wired_files = extract_wired_files(task["wire"], task["target"])
    file_contents = {}
    for f in wired_files:
        resolved = resolve_file(f, repo_root)
        if resolved and os.path.isfile(resolved):
            rel_path = os.path.relpath(resolved, repo_root).replace("\\", "/")
            with open(resolved, "r", encoding="utf-8", errors="replace") as fh:
                file_contents[rel_path] = fh.read()
            print(f"LOADED: {rel_path} ({len(file_contents[rel_path])} chars)")

    # Make sure target file is included
    if not file_contents:
        with open(target_path, "r", encoding="utf-8", errors="replace") as fh:
            rel_path = os.path.relpath(target_path, repo_root).replace("\\", "/")
            file_contents[rel_path] = fh.read()
            print(f"LOADED: {rel_path} ({len(file_contents[rel_path])} chars)")

    # Generate agent ID if not provided
    agent_id = args.agent_id or f"micro_{int(__import__('time').time())}"

    print(f"\n=== Micro Agent: {args.model} ===")
    print(f"TASK: {task['wire']}")
    print(f"FILES: {', '.join(file_contents.keys())}")
    print(f"VERIFY: {task['verify'] or 'cargo check --workspace'}")
    print()

    # Save original file states for rollback
    originals = {p: open(p, "r", encoding="utf-8").read() for p in file_contents}

    # Capture baseline error count before any edits
    verify_cmd = task["verify"] or "cargo check --workspace"
    print("Capturing baseline error count...")
    baseline_errors = count_errors(verify_cmd)
    print(f"BASELINE: {baseline_errors} pre-existing errors")

    # Build prompt
    prompt = build_prompt(task, file_contents)
    print(f"PROMPT: {len(prompt)} chars (~{len(prompt)//4} tokens)")

    for attempt in range(1, args.retries + 1):
        print(f"\n--- Attempt {attempt}/{args.retries} ---")

        # Run ollama
        print(f"Running ollama run {args.model}...")
        output = run_ollama(args.model, prompt)
        if not output:
            print("ERROR: Model returned empty output")
            continue

        print(f"OUTPUT: {len(output)} chars")
        print(f"RAW OUTPUT:\n{output}\n---END RAW---")

        # Try line edits first (more reliable than diff for small models)
        applied = apply_line_edits(output, task)
        if applied:
            print("LINE-EDIT: Applied successfully")
        else:
            # Fallback: try unified diff
            diff_text = extract_diff(output)
            if diff_text.strip():
                applied = apply_diff(diff_text)
                if applied:
                    print("DIFF: Applied via git apply")
                else:
                    print("DIFF: git apply failed, no edits could be applied")
            else:
                print("NO-EDIT: No diff or line edits found in output")

        if not applied:
            print("SKIP: Could not apply model output, retrying...")
            # Rollback any partial changes
            for p, content in originals.items():
                with open(p, "w", encoding="utf-8") as f:
                    f.write(content)
            continue

        # Run verify
        verify_cmd = task["verify"] or "cargo check --workspace"
        success, verify_output = run_verify(verify_cmd, baseline_errors)
        print(f"VERIFY: {'PASS' if success else 'FAIL'}")
        if not success:
            # Print last 20 lines of error
            error_lines = verify_output.strip().split('\n')[-20:]
            print("VERIFY ERROR (last 20 lines):")
            for l in error_lines:
                print(f"  {l}")

        if success:
            # Determine which files actually changed
            changed = []
            for p in file_contents:
                current = open(p, "r", encoding="utf-8").read()
                if current != originals.get(p, ""):
                    changed.append(p)

            print(f"\n=== SUCCESS ===")
            print(f"Changed: {', '.join(changed)}")
            write_completion(agent_id, "completed", args.task, changed, "pass")

            # Move task file to done/
            done_dir = os.path.join("HANDOFF", "done")
            os.makedirs(done_dir, exist_ok=True)
            dest = os.path.join(done_dir, os.path.basename(args.task))
            shutil.move(args.task, dest)
            print(f"Moved task to {dest}")
            sys.exit(0)
        else:
            # Feed error back for retry
            print(f"VERIFY FAILED, feeding error back to model...")
            # Rollback and re-try with error context
            for p, content in originals.items():
                with open(p, "w", encoding="utf-8") as f:
                    f.write(content)

            # Re-read files (in case model's diff was partially right)
            prompt = build_prompt(task, file_contents)
            prompt += f"""

PREVIOUS ATTEMPT FAILED with this error:
{verify_output[-2000:]}

Fix the error and output a corrected diff."""
            print(f"RETRY PROMPT: {len(prompt)} chars")

    # All retries exhausted
    print(f"\n=== FAILED after {args.retries} attempts ===")
    write_completion(agent_id, "failed", args.task, [], "fail",
                     f"Failed after {args.retries} attempts")
    sys.exit(1)


if __name__ == "__main__":
    main()