import express from "express";
import { WebSocketServer } from "ws";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = express();
const port = 3001;

app.use(express.static(path.join(__dirname, "public")));

const server = app.listen(port, () => {
  console.log(`Server listening on http://localhost:${port}`);
});

const wss = new WebSocketServer({ server });

function parseLogcat(line) {
  // Typical logcat line: 03-09 15:16:45.123 D Tag: message...
  const match = line.match(
    /^(\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}\.\d{3})\s+([VDIWEAF])\s+([^:]+):\s+(.*)$/,
  );
  if (match) {
    const [, timestamp, level, tag, msg] = match;
    return { source: "Android", level, tag, msg };
  }
  return { source: "Android", level: "?", tag: "None", msg: line };
}

function parseXcode(line) {
  // Simple regex for Xcode/log stream
  const tagMatch = line.match(/([A-Za-z0-9_.-]+):\s+(.*)$/);
  if (tagMatch) {
    return { source: "iOS", level: "I", tag: tagMatch[1], msg: tagMatch[2] };
  }
  return { source: "iOS", level: "?", tag: "None", msg: line };
}

function parseKeyPairs(msg) {
  const pairs = {};
  const regex = /([a-zA-Z0-9_-]+)[=:]\s?(\"[^\"]*\"|[^\s]+)/g;
  let m;
  while ((m = regex.exec(msg)) !== null) {
    pairs[m[1]] = m[2].replace(/^\"|\"$/g, "");
  }
  return pairs;
}

wss.on("connection", (ws) => {
  console.log("Client connected");

  const adb = spawn("adb", ["logcat", "-v", "time"]);
  const xcode = spawn("log", [
    "stream",
    "--level",
    "debug",
    "--predicate",
    'process CONTAINS "SCMessenger" OR sender CONTAINS "SCMessenger"',
  ]);

  const relay = (source, proc, parser) => {
    proc.stdout.on("data", (data) => {
      const lines = data.toString().split("\n");
      for (const line of lines) {
        if (!line.trim()) continue;
        const parsed = parser(line);
        parsed.keyPairs = parseKeyPairs(parsed.msg);
        if (ws.readyState === 1 /* OPEN */) {
          ws.send(JSON.stringify(parsed));
        }
      }
    });
    proc.on("error", (err) => console.error(`${source} spawn error:`, err));
  };

  relay("Android", adb, parseLogcat);
  relay("iOS", xcode, parseXcode);

  ws.on("close", () => {
    adb.kill();
    xcode.kill();
    console.log("Client disconnected");
  });
});
