#!/usr/bin/env python3
from __future__ import annotations
import json,re
from pathlib import Path
from collections import defaultdict,Counter

ROOT=Path(__file__).resolve().parents[1]
TODO_DIR=ROOT/'HANDOFF'/'todo'
SCAN_EXCLUDE={'target','build','.gradle','node_modules','.git'}
TOKEN_RE=re.compile(r'\b[A-Za-z_][A-Za-z0-9_]*\b')

FN_DECL_PATTERNS=[
 re.compile(r'\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b'),
 re.compile(r'\bfun\s+([A-Za-z_][A-Za-z0-9_]*)\b'),
 re.compile(r'\bfunction\s+([A-Za-z_][A-Za-z0-9_]*)\b'),
 re.compile(r'\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\('),
 re.compile(r'\b(?:private\s+)?func\s+([A-Za-z_][A-Za-z0-9_]*)\b'),
]

def extract_target(p:Path):
 for l in p.read_text(encoding='utf-8',errors='ignore').splitlines():
  if l.startswith('TARGET:'): return l.split('TARGET:',1)[1].strip().replace('\\','/')
 return ''

def extract_fn(p:Path):
 txt=p.read_text(encoding='utf-8',errors='ignore')
 m=re.search(r"The function '([^']+)'",txt)
 return m.group(1) if m else p.stem.replace('task_wire_','')

def find_def_line(target:Path,fn:str):
 if not target.exists(): return None
 for i,l in enumerate(target.read_text(encoding='utf-8',errors='ignore').splitlines(),1):
  if fn not in l: continue
  for pat in FN_DECL_PATTERNS:
   m=pat.search(l)
   if m and m.group(1)==fn: return i
 token=re.compile(rf'\b{re.escape(fn)}\b')
 for i,l in enumerate(target.read_text(encoding='utf-8',errors='ignore').splitlines(),1):
  if token.search(l): return i
 return None

def variant(path:str):
 for pre,name in [('android/','Android'),('core/','Core'),('wasm/','WASM'),('cli/','CLI'),('iOS/','iOS')]:
  if path.startswith(pre): return name
 return 'Other'

def batch(path:str):
 if path.startswith('core/src/lib.rs') or path.startswith('core/src/mobile_bridge.rs'): return 'B1-core-entrypoints'
 if path.startswith('core/src/transport') or path.startswith('core/src/store/relay_custody') or path.startswith('core/src/routing'): return 'B2-core-transport-routing'
 if path.startswith('android/app/src/main/java/com/scmessenger/android/data'): return 'B3-android-repository'
 if '/ui/' in path: return 'B4-android-ui'
 if path.startswith('android/app/src/main/java/com/scmessenger/android/transport') or path.startswith('android/app/src/main/java/com/scmessenger/android/service'): return 'B5-android-transport-service'
 if path.startswith('wasm/'): return 'B6-wasm'
 if path.startswith('cli/'): return 'B7-cli'
 return 'B8-cross-cutting'

# Build global token refs once
ref_counts=Counter()
for p in ROOT.rglob('*'):
 if not p.is_file(): continue
 rel=p.relative_to(ROOT)
 if any(part in SCAN_EXCLUDE for part in rel.parts): continue
 if rel.as_posix().startswith('HANDOFF/done/'): continue
 txt=p.read_text(encoding='utf-8',errors='ignore')
 ref_counts.update(TOKEN_RE.findall(txt))

tasks=[]
for tf in sorted(TODO_DIR.glob('task_wire_*.md')):
 t=extract_target(tf)
 fn=extract_fn(tf)
 d=find_def_line(ROOT/t,fn)
 ext=max(0, ref_counts[fn]-1)
 tasks.append({
  'task_file':tf.relative_to(ROOT).as_posix(),
  'task_name':tf.stem.replace('task_wire_',''),
  'function':fn,
  'target':t,
  'variant':variant(t),
  'batch':batch(t),
  'definition_line':d,
  'external_reference_hits':ext,
  'implementation_patch_template':{
   'file':t,'anchor_line':d,
   'required_change':f'Wire `{fn}` into production call path(s) and add parity-safe tests'
  }
 })

(ROOT/'HANDOFF'/'WIRING_PATCH_MANIFEST.json').write_text(json.dumps({'total_tasks':len(tasks),'tasks':tasks},indent=2),encoding='utf-8')
by=defaultdict(list)
for x in tasks: by[x['batch']].append(x)
lines=['# Wiring Patch Manifest (Pre-Implementation)','',
'This file provides exact edit coordinates and patch templates for each wiring task without implementing runtime logic.','',
f'Total tasks: **{len(tasks)}**','']
for b in sorted(by):
 lines += [f'## {b}','', '| Task | Function | Target | Definition line | External refs | Patch template |','|---|---|---|---:|---:|---|']
 for x in by[b]:
  lines.append(f"| `{x['task_name']}` | `{x['function']}` | `{x['target']}` | {x['definition_line'] or 'N/A'} | {x['external_reference_hits']} | `WIRE {x['function']} call path + tests` |")
 lines.append('')
(ROOT/'HANDOFF'/'WIRING_PATCH_MANIFEST.md').write_text('\n'.join(lines)+'\n',encoding='utf-8')
print('generated',len(tasks))
