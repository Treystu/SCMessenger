import os
import glob
import subprocess

tasks = ['A-04', 'A-05', 'E-02', 'E-04', 'D-01', 'D-02', 'D-03', 'D-04', 'D-05', 'C-05', 'C-06', 'T-02', 'T-03', 'T-04']

def fix_task(task_prefix):
    matches = glob.glob(f"HANDOFF/todo/{task_prefix}_*.md")
    if not matches:
        return
    file_path = matches[0]
    
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    if "## Target Files" not in content:
        return
        
    print(f"Fixing {file_path}...")
    
    # Run deduce_files.py to get the actual target files
    try:
        result = subprocess.run(["python", "scripts/deduce_files.py", "--task", file_path], capture_output=True, text=True, check=True)
        files = [f for f in result.stdout.strip().split() if f]
    except Exception as e:
        print(f"Failed to deduce files for {file_path}: {e}")
        return
        
    # Replace the Target Files section
    new_content = content.split("## Target Files")[0].strip() + "\n\n## Target Files\n"
    for file in files:
        new_content += f"- {file}\n"
        
    with open(file_path, "w", encoding="utf-8") as f:
        f.write(new_content)
        
for task in tasks:
    fix_task(task)
