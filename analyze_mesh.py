import time
import re
import os
import sys

# Peer identifiers: we assume they start with 12D3KooW and are ~52 chars long
PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")

logs = {
    'gcp': 'logs/5mesh/gcp.log',
    'osx': 'logs/5mesh/osx.log',
    'android': 'logs/5mesh/android.log',
    'ios_dev': 'logs/5mesh/ios-device.log',
    'ios_sim': 'logs/5mesh/ios-sim.log'
}

NODE_TYPES = {
    'gcp': 'Headless',
    'osx': 'Headless',
    'android': 'Full',
    'ios_dev': 'Full',
    'ios_sim': 'Full'
}

# Known/Target IDs
TARGET_IDS = {
    '12D3KooWMyngfNZajWRNRPdtc32uxn1sBYZE126NDD4b547BAMLj': 'gcp',
    '12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9': 'osx',
    '12D3KooWK8tm9qspf8FZ4sr2VHR48azhYuxCsiu7Ee5yQVoChamU': 'android',
    '12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27': 'ios_sim',
    '12D3KooWAqrZFh84t7WbgkTcxUGesHxLUH1gTY4szfe4aEXXqvvg': 'ios_dev'
}

matrix = {k: set() for k in logs}
file_to_id = {}
id_to_label = TARGET_IDS.copy()

def get_node_type(name_or_id):
    label = id_to_label.get(name_or_id, name_or_id)
    return NODE_TYPES.get(label, "Unknown")

print("Tracking Mesh Interconnectivity... Waiting for logs to populate...")

try:
    while True:
        # Read files
        for name, path in logs.items():
            if not os.path.exists(path):
                continue
            try:
                with open(path, 'r', errors='ignore') as f:
                    content = f.read()
            except Exception:
                continue
                
            # Find all mentioned peers in this log
            peers_in_log = set(PAT.findall(content))
            matrix[name].update(peers_in_log)
            
            # Simple heuristic for "own" ID
            if name not in file_to_id or file_to_id[name] not in peers_in_log:
                clean_content = re.sub(r'\x1b\[.*?m', '', content)
                local_match = re.search(r"(?:local_peer_id=|Peer ID:\s*|SwarmBridge with peer id:?\s*|Local Peer ID:\s*)(12D3KooW[a-zA-Z0-9]{44,})", clean_content)
                if local_match:
                    owner_id = local_match.group(1)
                    file_to_id[name] = owner_id
                    id_to_label[owner_id] = name
                else:
                    for pid, label in TARGET_IDS.items():
                        if label == name and pid in peers_in_log:
                            file_to_id[name] = pid

        # Clear screen
        os.system('clear' if os.name == 'posix' else 'cls')
        
        print("======== LIVE MESH STATUS ========")
        print(f"{'Node':<10} | {'Type':<10} | {'Peer ID'}")
        print("-" * 60)
        for name in logs:
            pid = file_to_id.get(name, "Unknown")
            ntype = NODE_TYPES.get(name, "???")
            print(f"{name:<10} | {ntype:<10} | {pid}")
            
        print("\nVisibility Matrix (Targeting 4 peers each):")
        print(f"{'Node':<10} | {'Full Sees':<10} | {'Headless Sees':<15} | {'Missing'}")
        print("-" * 80)
        
        for name in logs:
            own_id = file_to_id.get(name)
            seen_ids = matrix[name]
            
            full_seen = []
            headless_seen = []
            missing = []
            
            for other_name, other_type in NODE_TYPES.items():
                if other_name == name: continue
                other_id = file_to_id.get(other_name)
                
                if other_id and other_id in seen_ids:
                    if other_type == 'Full':
                        full_seen.append(other_name)
                    else:
                        headless_seen.append(other_name)
                elif other_id:
                    missing.append(other_name)
                else:
                    # ID still unknown, but we know it's a target
                    missing.append(f"{other_name}?")

            full_str = f"{len(full_seen)}/2 ({', '.join(full_seen)})"
            head_str = f"{len(headless_seen)}/2 ({', '.join(headless_seen)})"
            miss_str = ", ".join(missing) if missing else "None! (Perfect)"
            
            print(f"{name:<10} | {full_str:<10} | {head_str:<15} | {miss_str}")

        print("\nOther Peers detected (External/Stray):")
        for name in logs:
            stray = [p[:12] + "..." for p in matrix[name] if p not in id_to_label and p != file_to_id.get(name)]
            if stray:
                print(f"  {name:<10} sees {len(stray)} others: {', '.join(stray[:5])}{'...' if len(stray) > 5 else ''}")
                
        print("\n(Press Ctrl+C to stop)")
        time.sleep(3)
except KeyboardInterrupt:
    print("\nStopped.")
