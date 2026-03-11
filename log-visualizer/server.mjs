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
    // Error/Success High-Level Intent
    match: /(failed to|error|exception|unexpected|succesful|successfully|completed)/i,
    map: (m) => [m[1].toUpperCase().includes("FAIL") ? "!!! FAILURE !!!" : "SUCCESS", m[0]],
  },
  {
    // MeshRepository: delivery_attempt msg=unknown medium=core phase=direct...
    match: /delivery_attempt\s+(.*)/,
    map: (m) => ["DELIVERY", "ATTEMPT", ...m[1].split(/\s+(?=\w+=)/)],
  },
  {
    // Transport routing/flow
    match: /Transport:\s+(.*)/,
    map: (m) => ["TRANSPORT", ...m[1].split(/\s+(?=\w+=)/)],
  },
  {
    // Peer/Identity Flow
    match: /Peer (identified|discovered|connected|disconnected):\s+([\w\d]+)/i,
    map: (m) => ["PEER_EVENT", m[1].toUpperCase(), "PEER_ID"],
  },
  {
    // BLE Logic
    match: /BLE (identity beacon set|characteristic write successful|scanning|discovered):\s*(.*)/i,
    map: (m) => ["BLE_OPS", m[1].toUpperCase(), m[2].includes(":") ? "MAC_ADDR" : "DETAIL"],
  },
  {
    // Sync Operations
    match: /Processed (identity|history) sync (request|data)?/i,
    map: (m) => ["SYNC_OPS", m[1].toUpperCase(), m[2] ? m[2].toUpperCase() : "SYNC"],
  },
  {
    // Dialing/Connection
    match: /Dialing\s+(.*)/,
    map: (m) => ["NETWORK", "DIALING", "PATH"],
  },
];

/**
 * Recursive Exploratory Extraction
 * Iteratively unpeels layers based on logical delineators to avoid static depth limits.
 */
function extractDynamicSections(text, depth = 0) {
  if (depth > 6 || !text || text.length < 2) return [];

  // 1. High-Value Mapping Priority
  for (const rule of LOG_MAPPINGS) {
    const m = text.match(rule.match);
    if (m) {
      const parts = rule.map(m).map(cleanSegment);
      // We take the mapped parts as the definitive extraction for this branch
      return parts;
    }
  }

  // 2. Delineator Identification (Colon, Verb, KV-bound, Punctuation)
  // We pivot on the FIRST structural break found
  const delims = [
    { regex: /:\s+/, weight: 1 },
    { regex: /\s+(?=\w+=)/, weight: 2 }, // KV boundary: "key=val"
    { regex: /\b(processing|handling|attempting|failed|success|sending|receiving|dialing|connected|disconnected)\b/i, weight: 0.5 },
    { regex: /[,;]\s+/, weight: 3 },
  ];

  let best = null;
  for (const d of delims) {
    const match = text.match(d.regex);
    if (match && (best === null || match.index < best.index)) {
      best = { index: match.index, length: match[0].length, content: match[0] };
    }
  }

  if (best) {
    const head = text.substring(0, best.index).trim();
    const tail = text.substring(best.index + best.length).trim();
    const results = [];
    if (head && head.length < 50) results.push(cleanSegment(head));
    return [...results, ...extractDynamicSections(tail, depth + 1)];
  }

  // 3. Leaf Node Processing (Clean variables)
  return [cleanSegment(text)];
}

function cleanSegment(s) {
  let cleaned = s.trim();
  
  // 1. Structural Normalization
  cleaned = cleaned.replace(/^\[|\]$/g, ""); // Strip brackets
  
  // 2. Variable Scrubbing (Aggressive)
  cleaned = cleaned.replace(/12D3KooW[a-zA-Z0-9]{32,}/g, "PEER_ID"); // Full PeerIDs
  cleaned = cleaned.replace(/[a-f0-9]{32,}/gi, "HASH"); // UUIDs/Hashes
  cleaned = cleaned.replace(/\b[0-9a-f]{8,}\b/gi, "HEX"); // Short Hex
  cleaned = cleaned.replace(/\b\d{4,}\b/g, "NUM"); // Any 4+ digit variable
  cleaned = cleaned.replace(/\d{2}:\d{2}:\d{2}(\.\d{3})?/, "TIME"); // Internal timestamps
  
  // 3. Network/Path Normalization
  cleaned = cleaned.replace(/\/ip[46]\/[\d\.]+/g, "/ipX/ADDR"); // IP addresses
  
  // 4. Semantic Short-circuit (Pull intent to front)
  const intentMap = [
    { match: /error|fail|err_|exception/i, label: "!!! ERROR !!!" },
    { match: /success|ok|completed|finished/i, label: "SUCCESS" },
    { match: /retry|retrying|backoff/i, label: "RETRY_FLOW" },
    { match: /timeout|timed out/i, label: "TIMEOUT" }
  ];
  for (const i of intentMap) {
    if (i.match.test(cleaned)) return i.label;
  }

  return cleaned.substring(0, 50); // Hard cap on label bloat
}

function processLine(rawLine, platform) {
  try {
    // Global Noise Stripper
    let line = rawLine
      .trim()
      .replace(/^(\d{2}-\d{2}\s+)?\d{2}:\d{2}:\d{2}(\.\d{3})?\s+/, "")
      .replace(/^\[\d{2}:\d{2}:\d{2}\]\s*/, "")
      .replace(/^System:\s*/i, "");

    let tag = "General";
    let msg = line;

    // Source Separation
    const adbMatch = line.match(/\b[VDIWEAF]\/([^\s\(:]+)\s*(?:\(\s*\d+\))?:\s*(.*)/);
    const scmMatch = line.match(/\(([^)]+)\)\s*\[([^\]]+)\]\s*(.*)/);
    const harnessStatMatch = line.match(/^\s*(GCP|OSX|Android|iOS Dev|iOS Sim):\s*(\d+)/i);
    if (scmMatch) {
      tag = scmMatch[2].split(":").pop().trim();
      msg = scmMatch[3].trim();
    } else if (harnessStatMatch) {
      tag = "Metrics";
      msg = `Status: ${harnessStatMatch[1]}`; // Group all "GCP: 123 lines" into "Metrics -> Status: GCP"
    } else if (platform === "Harness") {
      tag = "Harness";
      msg = line;
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

  const streamMesh = () => {
    const meshDir = path.join(__dirname, "..", "logs", "5mesh", "latest");
    // Check for common harness logs and stream them if they exist
    const sources = [
      { name: "GCP", file: "gcp.log" },
      { name: "OSX", file: "osx.log" },
      { name: "Harness", file: "harness.log" },
      { name: "Sim", file: "ios-sim.log" },
      { name: "iOS-Dev", file: "ios-device.log" },
      { name: "Android-Mesh", file: "android.log" },
    ];

    sources.forEach((src) => {
      const fullPath = path.join(meshDir, src.file);
      // Use tail -F to safely wait for file creation or rotation
      stream(src.name, "tail", ["-F", "-n", "100", fullPath]);
    });
  };

  streamMesh();
  streamAndroid();
  stream("iOS-Direct", "log", [
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
