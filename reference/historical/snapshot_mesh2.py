import re
import os

PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")

files = {
    'gcp': 'logs/5mesh/gcp.log',
    'osx': 'logs/5mesh/osx.log',
    'android': 'logs/5mesh/android.log'
}

ids_map = {
    '12D3KooWMyngfNZajWRNRPdtc32uxn1sBYZE126NDD4b547BAMLj': 'gcp',
    '12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9': 'osx',
    '12D3KooWK8tm9qspf8FZ4sr2VHR48azhYuxCsiu7Ee5yQVoChamU': 'android',
    '12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27': 'ios_sim',
    '12D3KooWAqrZFh84t7WbgkTcxUGesHxLUH1gTY4szfe4aEXXqvvg': 'ios_dev'
}

matrix = {}

for name, path in files.items():
    peers = set()
    node_counts = {}
    if os.path.exists(path):
        with open(path, 'r', errors='ignore') as f:
            for line in f:
                for p in PAT.findall(line):
                    peers.add(p)
                    node_counts[p] = node_counts.get(p, 0) + 1
    
    # map peer ID to actual name, defaulting to small ID if unknown map
    seen = []
    for p in peers:
        actual_name = ids_map.get(p)
        if actual_name and actual_name != name: # Don't list itself
            seen.append(f"{actual_name} ({node_counts[p]})")
    matrix[name] = seen

print("=== Visibility Matrix ===")
for name, seen_labels in matrix.items():
    print(f"{name:10} sees: {', '.join(seen_labels) if seen_labels else 'None'}")
