#!/usr/bin/env python3
"""Split oversized batch files into sub-batches of at most N tasks each.

Usage: batch_splitter.py <batch_file> <max_tasks>

Reads a BATCH_*.md file, counts numbered task items (lines matching
'N. **func_name**'), and splits into sub-batch files if the count
exceeds max_tasks. Sub-batches are written to HANDOFF/todo/ as
BATCH_<original>_SUB01.md, _SUB02.md, etc.
"""

import re
import sys
from pathlib import Path


def split_batch(batch_file, max_tasks):
    with open(batch_file, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')

    # Find the header (everything before the first numbered task item)
    header_lines = []
    first_task_idx = None
    for i, line in enumerate(lines):
        if re.match(r'^\d+\.\s+\*\*', line):
            first_task_idx = i
            break
        header_lines.append(line)

    if first_task_idx is None:
        print(f"No numbered task items found in {batch_file}")
        return

    header = '\n'.join(header_lines) + '\n'

    # Extract task groups (numbered items + their continuation lines)
    tasks = []
    current_task = []
    for line in lines[first_task_idx:]:
        if re.match(r'^\d+\.\s+\*\*', line) and current_task:
            tasks.append('\n'.join(current_task))
            current_task = [line]
        else:
            current_task.append(line)
    if current_task:
        tasks.append('\n'.join(current_task))

    # Split into sub-batches
    num_sub_batches = (len(tasks) + max_tasks - 1) // max_tasks
    handoff_todo = Path('HANDOFF/todo')
    batch_stem = Path(batch_file).stem

    for i in range(num_sub_batches):
        start = i * max_tasks
        end = min(start + max_tasks, len(tasks))
        chunk = tasks[start:end]

        # Renumber tasks in this sub-batch
        renumbered = []
        for j, task_text in enumerate(chunk):
            renumbered.append(re.sub(r'^\d+\.', str(j + 1) + '.', task_text, count=1))

        sub_name = f"{batch_stem}_SUB{i+1:02d}.md"
        sub_path = handoff_todo / sub_name

        with open(sub_path, 'w', encoding='utf-8') as f:
            f.write(header)
            f.write(f"\n## Sub-batch {i+1} of {num_sub_batches}\n\n")
            f.write('\n'.join(renumbered))

        print(f"Created: {sub_path} ({end - start} tasks)")

    print(f"Split {len(tasks)} tasks into {num_sub_batches} sub-batches")


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: batch_splitter.py <batch_file> <max_tasks>")
        sys.exit(1)

    split_batch(sys.argv[1], int(sys.argv[2]))