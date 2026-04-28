import websocket
import json

def on_message(ws, message):
    print(f"Received: {message}")

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("Connection closed")

def on_open(ws):
    print("Connection opened")
    # Send status command
    status_cmd = {"cmd": "status"}
    ws.send(json.dumps(status_cmd))

# Connect to WebSocket
ws = websocket.WebSocketApp("ws://127.0.0.1:9000/ws",
                            on_open=on_open,
                            on_message=on_message,
                            on_error=on_error,
                            on_close=on_close)

ws.run_forever()