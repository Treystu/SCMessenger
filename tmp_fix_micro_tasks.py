import re
import os
import glob

todo_dir = "HANDOFF/todo"
files = glob.glob(os.path.join(todo_dir, "MICRO_*.md"))

for fpath in files:
    with open(fpath, "r") as fh:
        content = fh.read()

    # Extract model from YAML frontmatter
    m = re.search(r'^model:\s*"?([^"\n]+)"?', content, re.MULTILINE)
    model = m.group(1).strip() if m else "gemini-3-flash-preview:cloud"

    # Extract time_limit_ms and convert to seconds budget (capped at 300)
    m2 = re.search(r'^time_limit_ms:\s*(\d+)', content, re.MULTILINE)
    budget = min(int(m2.group(1)) // 1000, 300) if m2 else 120

    # Check if already has MODEL header
    if re.search(r'^#\s*MODEL\s*:', content, re.MULTILINE):
        print(f"Skipping {fpath} - already has MODEL header")
        continue

    # Find end of frontmatter and insert headers
    lines = content.split("\n")
    new_lines = []
    in_frontmatter = False
    frontmatter_done = False
    for line in lines:
        if line.strip() == "---" and not frontmatter_done:
            in_frontmatter = not in_frontmatter
            new_lines.append(line)
            if not in_frontmatter:
                frontmatter_done = True
                new_lines.append("")
                new_lines.append(f"# MODEL: {model}")
                new_lines.append(f"# BUDGET: {budget}")
        else:
            new_lines.append(line)

    with open(fpath, "w") as fh:
        fh.write("\n".join(new_lines))

    print(f"Fixed {fpath}: MODEL={model} BUDGET={budget}")
