import os
import re

log_dir = 'logs/5mesh'
nodes = {}

# Regex patterns
peer_id_pat = re.compile(r'local_peer_id=([1-9A-HJ-NP-Za-km-z]{46,})|Peer ID:\s*([1-9A-HJ-NP-Za-km-z]{46,})|Swarm initialized with local peer id:?\s*([1-9A-HJ-NP-Za-km-z]{46,})', re.IGNORECASE)
conn_pat = re.compile(r'established connection to ([1-9A-HJ-NP-Za-km-z]{46,})|Connected to peer: ([1-9A-HJ-NP-Za-km-z]{46,})|ConnectionEstablished.*?peer_id=([1-9A-HJ-NP-Za-km-z]{46,})', re.IGNORECASE)

for filename in os.listdir(log_dir):
    if not filename.endswith('.log'): continue
    filepath = os.path.join(log_dir, filename)
    name = filename.replace('.log', '')
    node_info = {'id': None, 'connected_to': set()}
    
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            # Find local peer id
            if node_info['id'] is None:
                m = peer_id_pat.search(line)
                if m:
                    node_info['id'] = m.group(1) or m.group(2) or m.group(3)
            
            # Find connections
            m2 = conn_pat.search(line)
            if m2:
                peer = m2.group(1) or m2.group(2) or m2.group(3)
                if peer:
                    node_info['connected_to'].add(peer)

    nodes[name] = node_info

print("=== Node Identities ===")
for name, info in nodes.items():
    print(f"{name:12}: {info['id'] or 'UNKNOWN'}")

print("\n=== Connectivity Matrix ===")
for name, info in nodes.items():
    print(f"{name:12} ({info['id']}) connected to:")
    if not info['connected_to']:
        print("    (none)")
    for target in info['connected_to']:
        target_name = "UNKNOWN"
        for n, i in nodes.items():
            if i['id'] == target:
                target_name = n
                break
        print(f"    - {target_name} ({target})")
