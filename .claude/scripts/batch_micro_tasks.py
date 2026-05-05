#!/usr/bin/env python3
"""
Batch micro-tasks for efficient processing.
This script consolidates small tasks (under 50 LOC) into batch files
to reduce API consumption overhead.
"""

import os
import json
import glob
from pathlib import Path

def is_micro_task(task_file):
    """Determine if a task is a micro-task based on filename patterns."""
    micro_patterns = ['TRIAGE', 'QUICK', 'LINT', 'FMT', 'MICRO']
    task_name = os.path.basename(task_file)
    return any(pattern in task_name.upper() for pattern in micro_patterns)

def get_task_loc(task_file):
    """Estimate lines of code in a task file."""
    try:
        with open(task_file, 'r', encoding='utf-8') as f:
            return len(f.readlines())
    except:
        return 0

def batch_micro_tasks():
    """Consolidate micro-tasks into batch files."""
    todo_dir = Path('HANDOFF/todo')
    if not todo_dir.exists():
        print("No todo directory found")
        return

    # Find all micro-tasks
    micro_tasks = []
    for task_file in todo_dir.glob('*.md'):
        if is_micro_task(str(task_file)):
            loc = get_task_loc(task_file)
            if loc < 50:  # Less than 50 lines of code
                micro_tasks.append((str(task_file), loc))

    # Sort by estimated LOC
    micro_tasks.sort(key=lambda x: x[1])

    # Batch tasks (max 5 tasks per batch or 200 LOC total)
    batches = []
    current_batch = []
    current_loc = 0

    for task_file, loc in micro_tasks:
        if len(current_batch) >= 5 or current_loc + loc > 200:
            if current_batch:
                batches.append(current_batch)
            current_batch = [(task_file, loc)]
            current_loc = loc
        else:
            current_batch.append((task_file, loc))
            current_loc += loc

    if current_batch:
        batches.append(current_batch)

    # Create batch files
    batch_dir = Path('HANDOFF/batches')
    batch_dir.mkdir(exist_ok=True)

    for i, batch in enumerate(batches):
        batch_filename = batch_dir / f'micro_batch_{i+1:03d}.md'

        # Combine task contents
        combined_content = "# MICRO-TASK BATCH\n\n"
        combined_content += "This batch contains the following micro-tasks:\n\n"

        task_list = []
        for task_file, loc in batch:
            task_name = os.path.basename(task_file)
            task_list.append(f"- {task_name} ({loc} LOC)")

        combined_content += "\n".join(task_list)
        combined_content += "\n\n## Combined Implementation\n\n"
        combined_content += "Implement all the above micro-tasks in a single pass.\n"
        combined_content += "CRITICAL: Process each task individually and ensure all are completed.\n"

        # Write batch file
        with open(batch_filename, 'w', encoding='utf-8') as f:
            f.write(combined_content)

        # Move original tasks to batched directory
        batched_dir = Path('HANDOFF/batched')
        batched_dir.mkdir(exist_ok=True)

        for task_file, _ in batch:
            task_path = Path(task_file)
            new_path = batched_dir / task_path.name
            task_path.rename(new_path)

        print(f"Created batch {batch_filename} with {len(batch)} tasks")

    if not batches:
        print("No micro-tasks found to batch")
    else:
        print(f"Created {len(batches)} batch files")

if __name__ == "__main__":
    batch_micro_tasks()