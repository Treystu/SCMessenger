import re
import os

PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")

files = {
    'gcp': 'logs/5mesh/gcp.log',
    'osx': 'logs/5mesh/osx.log',
    'android': 'logs/5mesh/android.log',
    'ios_device': 'logs/5mesh/ios-device.log',
    'ios_sim': 'logs/5mesh/ios-sim-all.log'
}

peers_in_log = {}
counts = {}

for name, path in files.items():
    peers = set()
    node_counts = {}
    if os.path.exists(path):
        with open(path, 'r', errors='ignore') as f:
            for line in f:
                for p in PAT.findall(line):
                    peers.add(p)
                    node_counts[p] = node_counts.get(p, 0) + 1
    peers_in_log[name] = peers
    counts[name] = node_counts

# Determine Identities
identities = {}
for name, pcs in counts.items():
    if not pcs:
        identities[name] = None
        continue
    # Guess identity as the most frequent peer in its own log
    identities[name] = max(pcs, key=pcs.get)

# Reverse lookup for identities to names
peer_to_name = {pid: name for name, pid in identities.items() if pid}

print("=== Node Identities ===")
for name, pid in identities.items():
    print(f"{name:10}: {pid or 'UNKNOWN'} (from {counts[name].get(pid, 0)} logs)")

print("\n=== Visibility Matrix ===")
for name, peers in peers_in_log.items():
    own_id = identities.get(name)
    seen_labels = [peer_to_name.get(p, p[:8]+"...") for p in peers if p != own_id and p in peer_to_name]
    print(f"{name:10} sees: {', '.join(seen_labels) if seen_labels else 'None'}")
