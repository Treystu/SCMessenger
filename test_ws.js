const { WebSocket } = require("ws");

const ws = new WebSocket("ws://localhost:3001");

ws.on("open", () => {
  console.log("Connected");
  setTimeout(() => process.exit(0), 1000);
});

ws.on("message", (data) => console.log("msg:", data.toString()));
ws.on("error", (e) => console.error("Error:", e));
