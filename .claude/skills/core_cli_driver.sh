#!/usr/bin/env bash
set -euo pipefail

# OS Detection
OS=$(uname -s)

# Logging directory
LOG_DIR=".claude"
LOG_FILE="${LOG_DIR}/core_daemon.log"
mkdir -p "${LOG_DIR}"

start_node() {
    echo "Starting SCMessenger CLI daemon (release build)..."
    nohup cargo run --package scmessenger-cli --release -- start > "${LOG_FILE}" 2>&1 &
    echo "Daemon started with PID $!"
}

rpc_call() {
    local payload="$1"
    /c/Users/kanal/AppData/Local/Programs/Python/Python312/python.exe -c "
import socket, base64, hashlib, struct, json, os, sys

payload = '''${payload}'''
host = '127.0.0.1'
port = 9002

sock = socket.create_connection((host, port))
key = base64.b64encode(os.urandom(16)).decode()
upgrade = (
    f'GET / HTTP/1.1\\r\\n'
    f'Host: {host}:{port}\\r\\n'
    f'Upgrade: websocket\\r\\n'
    f'Connection: Upgrade\\r\\n'
    f'Sec-WebSocket-Key: {key}\\r\\n'
    f'Sec-WebSocket-Version: 13\\r\\n'
    f'\\r\\n'
)
sock.sendall(upgrade.encode())

# Read HTTP response headers
resp = b''
while b'\\r\\n\\r\\n' not in resp:
    chunk = sock.recv(1024)
    if not chunk:
        print('Connection closed during handshake', file=sys.stderr)
        sys.exit(1)
    resp += chunk

# Build masked text frame (opcode 0x81)
data = payload.encode()
mask = os.urandom(4)
masked = bytes(b ^ mask[i % 4] for i, b in enumerate(data))
frame = struct.pack('>BB', 0x81, 0x80 | len(data)) + mask + masked
sock.sendall(frame)

# Read response frame
resp_frame = b''
while len(resp_frame) < 2:
    chunk = sock.recv(1)
    if not chunk:
        print('Connection closed waiting for frame header', file=sys.stderr)
        sys.exit(1)
    resp_frame += chunk

opcode = resp_frame[0]
if (opcode & 0x0F) == 0x08:
    print('Server sent close frame', file=sys.stderr)
    sys.exit(1)

payload_len = resp_frame[1] & 0x7F
offset = 2
if payload_len == 126:
    while len(resp_frame) < 4:
        resp_frame += sock.recv(1)
    payload_len = struct.unpack('>H', resp_frame[2:4])[0]
    offset = 4
elif payload_len == 127:
    while len(resp_frame) < 10:
        resp_frame += sock.recv(1)
    payload_len = struct.unpack('>Q', resp_frame[2:10])[0]
    offset = 10

masked_flag = (resp_frame[1] & 0x80) != 0
mask_key = b''
if masked_flag:
    while len(resp_frame) < offset + 4:
        resp_frame += sock.recv(1)
    mask_key = resp_frame[offset:offset+4]
    offset += 4

while len(resp_frame) < offset + payload_len:
    chunk = sock.recv(max(4096, (offset + payload_len) - len(resp_frame)))
    if not chunk:
        print('Connection closed waiting for payload', file=sys.stderr)
        sys.exit(1)
    resp_frame += chunk

payload_bytes = resp_frame[offset:offset+payload_len]
if masked_flag:
    payload_bytes = bytes(b ^ mask_key[i % 4] for i, b in enumerate(payload_bytes))

print(payload_bytes.decode('utf-8', errors='replace'))
sock.close()
"
}

dump_logs() {
    case "${OS}" in
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            powershell.exe -NoProfile -Command "Get-Content ${LOG_FILE} -Tail 100"
            ;;
        Darwin|Linux)
            if [[ -f "${LOG_FILE}" ]]; then
                tail -n 100 "${LOG_FILE}"
            else
                echo "Log file not found: ${LOG_FILE}"
            fi
            ;;
        *)
            echo "Unsupported OS: ${OS}"
            exit 1
            ;;
    esac
}

stop_node() {
    case "${OS}" in
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            powershell.exe -NoProfile -Command "Stop-Process -Name scmessenger-cli -Force -ErrorAction SilentlyContinue"
            echo "Stop command issued for scmessenger-cli (Windows)."
            ;;
        Darwin|Linux)
            pkill -f "scmessenger-cli" || true
            echo "Stop command issued for scmessenger-cli (Unix)."
            ;;
        *)
            echo "Unsupported OS: ${OS}"
            exit 1
            ;;
    esac
}

# Main dispatch
case "${1:-}" in
    start)
        start_node
        ;;
    rpc)
        if [[ $# -lt 2 ]]; then
            echo "Usage: $0 rpc <json_payload>"
            exit 1
        fi
        rpc_call "$2"
        ;;
    dump_logs)
        dump_logs
        ;;
    stop)
        stop_node
        ;;
    *)
        echo "Usage: $0 {start|rpc <json_payload>|dump_logs|stop}"
        exit 1
        ;;
esac
