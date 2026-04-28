/**
 * SCMessenger Headless Node
 * Minimal JS to drive the libp2p swarm in the browser.
 */

const DEFAULT_BOOTSTRAP_NODES = [
  "/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
  "/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9",
];

const UI = {
  statusText: document.getElementById("status-text"),
  statusDot: document.getElementById("status-dot"),
  peerId: document.getElementById("peer-id"),
  log: document.getElementById("log"),

  updateStatus(text, isActive = false) {
    this.statusText.textContent = text;
    if (isActive) {
      this.statusDot.classList.add("active");
    } else {
      this.statusDot.classList.remove("active");
    }
  },

  setPeerId(id) {
    this.peerId.textContent = `Local Peer ID: ${id}`;
  },

  addLog(msg) {
    const time = new Date().toLocaleTimeString();
    const line = document.createElement("div");
    line.textContent = `[${time}] ${msg}`;
    this.log.prepend(line);
    if (this.log.children.length > 50) {
      this.log.lastChild.remove();
    }
    console.log(`[SCM] ${msg}`);
  },
};

async function init() {
  UI.addLog("Initializing headless node...");

  try {
    // 1. Load WASM Module
    UI.addLog("Loading WASM module...");
    const wasmPath = "./wasm/scmessenger_wasm.js";
    const module = await import(wasmPath);

    // Initialize wasm-bindgen if it's a function
    if (typeof module.default === "function") {
      await module.default();
    }

    const { IronCore } = module;
    if (!IronCore) {
      throw new Error("IronCore export missing from WASM module");
    }

    // 2. Initialize Core
    UI.addLog("Creating IronCore instance...");
    // Use a dedicated storage name for the headless node
    const core =
      typeof IronCore.withStorage === "function"
        ? IronCore.withStorage("scm_headless")
        : new IronCore();

    core.start();
    UI.addLog("Core started.");

    // 3. Get Identity
    const identity = core.getIdentityInfo();
    UI.setPeerId(identity.peerId);
    UI.addLog(`Identity loaded: ${identity.peerId}`);

    // 4. Start Swarm
    UI.addLog("Starting libp2p swarm...");
    UI.updateStatus("Connecting to mesh...");

    // Use any bootstrap nodes passed in URL, or defaults
    const urlParams = new URLSearchParams(window.location.search);
    let bootstrapAddrs = DEFAULT_BOOTSTRAP_NODES;
    if (urlParams.has("bootstrap")) {
      bootstrapAddrs = urlParams.get("bootstrap").split(",");
      UI.addLog("Using custom bootstrap nodes from URL.");
    }

    await core.startSwarm(bootstrapAddrs);
    UI.updateStatus("Active in Mesh", true);
    UI.addLog("Swarm active.");

    // 5. Polling Loop for Diagnostics
    setInterval(async () => {
      try {
        const peers = await core.getPeers();
        if (peers.length > 0) {
          UI.updateStatus(`${peers.length} peers connected`, true);
        } else {
          UI.updateStatus("Searching for peers...", false);
        }
      } catch (err) {
        console.warn("Status poll failed:", err);
      }
    }, 5000);
  } catch (error) {
    UI.addLog(`FATAL ERROR: ${error.message}`);
    UI.updateStatus("Error", false);
    console.error("Headless init failed:", error);
  }
}

window.addEventListener("DOMContentLoaded", init);
