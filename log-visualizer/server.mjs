import express from "express";
import { WebSocketServer } from "ws";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = express();
app.use(express.static(path.join(__dirname, "public")));
const server = app.listen(3001, () =>
  console.log("LogSankey Server v5.0 Dynamic Analysis @ http://localhost:3001"),
);
const wss = new WebSocketServer({ server });

/**
 * v5.0 Dynamic Section Analysis
 * Replaces static 7-level extraction with infinite-depth logical delineators.
 */

const LOG_MAPPINGS = [
  {
    // I/BugleRcsEngine: handleMessage processing message:[NOTIFY_UPTIME...] with...
    match: /handleMessage processing message:\[(.*?)\]\s*(.*)/,
    map: (m) => ["processing", m[1], m[2]],
  },
  {
    // MeshRepository: delivery_attempt msg=unknown medium=core phase=direct...
    match: /delivery_attempt\s+(.*)/,
    map: (m) => ["delivery_attempt", ...m[1].split(/\s+(?=\w+=)/)],
  },
  {
    // MeshRepository: Transport: route=... connected=...
    match:
      /(?:^|[\u2700-\u27BF]|[\uE000-\uF8FF]|\uD83C[\uDF00-\uDFFF]|\uD83D[\uDC00-\uDE4F]|\uD83D[\uDE80-\uDEFF]|[\u2011-\u26FF])*\s*Transport:\s+(.*)/,
    map: (m) => ["Transport", ...m[1].split(/\s+(?=\w+=)/)],
  },
  {
    // SCMessenger: Peer identified: 12D3K... with 13 addresses
    match: /Peer identified:\s+([\w\d]+)\s+with\s+(.*)/,
    map: (m) => ["Peer identified", m[1] + "...", "with " + m[2]],
  },
  {
    // BLE Identity Beacon
    match: /BLE identity beacon set:\s*([a-f0-9]+)\.\.\.\s*\((.*?)\)/,
    map: (m) => ["BLE identity beacon set", m[1], m[2]],
  },
  {
    // WiFi creating message
    match: /Creating message to (.*?);\s*iface\s*=\s*(\d+)/,
    map: (m) => ["Creating message", m[1], `iface=${m[2]}`],
  },
  {
    // Finsky Memory trim
    match: /Memory trim requested to level (\d+)/,
    map: (m) => ["Memory trim requested", `level ${m[1]}`],
  },
  {
    // Bluetooth Address mismatch
    match: /Address type mismatch for (.*?),\s*new type:\s*(\d+)/,
    map: (m) => ["Address type mismatch", m[1], `new type=${m[2]}`],
  },
  {
    // Mesh Receipt
    match: /Receipt for (.*?):\s*(.*)/,
    map: (m) => ["Receipt", m[1], m[2]],
  },
  {
    // MeshRepository: Dialing /ip4/104.28.216.43/tcp/9010/p2p/12D3K...
    match: /Dialing\s+(.*)/,
    map: (m) => {
      const path = m[1];
      if (path.includes("/p2p/")) {
        const parts = path.split("/p2p/");
        return [
          "Dialing",
          parts[0] + "/p2p/",
          parts[1].substring(0, 8) + "...",
        ];
      }
      return ["Dialing", path];
    },
  },
  {
    // Nickname generation
    match: /Generated default nickname '(.*?)' for peer (.*)/,
    map: (m) => ["Nickname Generated", m[1], m[2].substring(0, 8) + "..."],
  },
  {
    // BLE Characteristic Write
    match: /Characteristic write successful to (.*)/,
    map: (m) => ["BLE Write", "Successful", m[1]],
  },
  {
    // Raw Peer ID catcher
    match: /^(12D3K[a-zA-Z0-9]{30,})$/,
    map: (m) => ["Peer ID", m[1].substring(0, 10) + "..."],
  },
];

function extractDynamicSections(msg) {
  // 1. Check known explicit mappings
  for (const rule of LOG_MAPPINGS) {
    const m = msg.match(rule.match);
    if (m) {
      return rule
        .map(m)
        .map((s) => s.trim())
        .filter((s) => s);
    }
  }

  // 2. Generic KV Extractor (Action key=val key2=val2)
  if (msg.includes("=")) {
    const parts = msg.split(/\s+(?=\w+=)/);
    if (parts.length > 1) {
      return parts.map((p) => p.trim()).filter((s) => s);
    }
  }

  // 3. Status/Colon Breakout Extractor
  if (msg.includes(": ")) {
    const parts = msg.split(/:\s+/);
    if (parts.length > 1 && parts[0].length < 40) {
      // Keep the first part as the pivot, the rest as the detail
      return [parts[0].trim(), parts.slice(1).join(": ").trim()].filter(
        (s) => s,
      );
    }
  }

  // 4. Fallback: Split by punctuation (, or ;)
  const chunks = msg.split(/[,;]\s+/);
  if (chunks.length > 1) {
    return chunks.map((c) => c.trim()).filter((s) => s);
  }

  // 5. Raw string default
  return [msg.trim()];
}

function processLine(rawLine, platform) {
  try {
    let line = rawLine.trim().replace(/^System:\s*/i, "");
    let tag = "General";
    let msg = line;

    // Pattern extraction mapping
    const adbMatch = line.match(
      /\b[VDIWEAF]\/([^\s\(:]+)\s*(?:\(\s*\d+\))?:\s*(.*)/,
    );
    const scmMatch = line.match(/\(([^)]+)\)\s*\[([^\]]+)\]\s*(.*)/);

    if (scmMatch) {
      tag = scmMatch[2].split(":").pop().trim();
      msg = scmMatch[3].trim();
    } else if (adbMatch) {
      tag = adbMatch[1].trim();
      msg = adbMatch[2].trim();
    } else {
      const genericMatch = line.match(/^([A-Za-z0-9_.-]+):\s*(.*)$/);
      if (genericMatch && genericMatch[1].length < 32) {
        tag = genericMatch[1].trim();
        msg = genericMatch[2].trim();
      }
    }

    // Strip bracketed components from the tag (e.g. D/Repository[MyThread] -> Repository)
    tag = tag.replace(/\[.*?\]/g, "").trim();

    // Dynamically iterate and extract based on log type sections
    const dynamicSections = extractDynamicSections(msg);
    const segments = [platform, tag, ...dynamicSections];

    return {
      source: platform,
      levelTag: tag,
      msg: msg,
      segments: segments,
    };
  } catch (e) {
    console.error("Dynamic Processor Error:", e);
    return {
      source: platform,
      levelTag: "Error",
      msg: rawLine,
      segments: [platform, "Error", rawLine],
    };
  }
}

wss.on("connection", (ws) => {
  let isAlive = true;
  const procs = [];

  const stream = (name, cmd, args) => {
    const p = spawn(cmd, args);
    procs.push(p);
    let buffer = "";
    p.stdout.on("data", (d) => {
      if (!isAlive) return;
      buffer += d.toString();
      const lines = buffer.split("\n");
      buffer = lines.pop();
      for (const l of lines) {
        if (!l.trim()) continue;
        const parsed = processLine(l, name);
        if (ws.readyState === 1 && parsed.segments.length > 1)
          ws.send(JSON.stringify(parsed));
      }
    });
    p.on("exit", () => {
      if (isAlive) setTimeout(() => stream(name, cmd, args), 3000);
    });
  };

  // Emulate "package:mine" by dynamically fetching the SCMessenger PID
  const streamAndroid = () => {
    const getPid = spawn("adb", [
      "shell",
      "pidof",
      "-s",
      "com.scmessenger.android",
    ]);
    let pidBuffer = "";
    getPid.stdout.on("data", (d) => (pidBuffer += d.toString()));
    getPid.on("close", () => {
      const pid = pidBuffer.trim();
      if (pid && isAlive) {
        stream("Android", "adb", ["logcat", "-v", "time", "--pid", pid]);
      } else if (isAlive) {
        setTimeout(streamAndroid, 3000); // Polling until the app is launched
      }
    });
  };

  streamAndroid();
  stream("iOS", "log", [
    "stream",
    "--level",
    "debug",
    "--predicate",
    'process CONTAINS "SCMessenger"',
  ]);

  ws.on("close", () => {
    isAlive = false;
    procs.forEach((p) => p.kill("SIGTERM"));
  });
});
