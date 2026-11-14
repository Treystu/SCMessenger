/**
 * SCMessenger Headless Node
 * Minimal JS to drive the libp2p swarm in the browser.
 *
 * WS14: Includes browser notification support with DM vs DM Request classification.
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

/**
 * WS14: Browser notification helper for WASM.
 * Uses the Notification API with DM vs DM Request classification.
 */
const NotificationHelper = {
  permissionGranted: false,
  settings: {
    notificationsEnabled: true,
    notifyDmEnabled: true,
    notifyDmRequestEnabled: true,
    notifyDmInForeground: false,
    notifyDmRequestInForeground: true,
    soundEnabled: true,
    badgeEnabled: true,
  },

  async requestPermission() {
    if (!("Notification" in window)) {
      console.warn("Browser does not support notifications");
      return false;
    }
    if (Notification.permission === "granted") {
      this.permissionGranted = true;
      return true;
    }
    if (Notification.permission !== "denied") {
      const permission = await Notification.requestPermission();
      this.permissionGranted = permission === "granted";
      return this.permissionGranted;
    }
    return false;
  },

  /**
   * Show a browser notification based on classification.
   * @param {Object} decision - NotificationDecision from classifyNotification
   * @param {string} content - Message content
   * @param {string} nickname - Sender nickname
   */
  showNotification(decision, content, nickname) {
    if (!this.permissionGranted) {
      console.warn("Notification permission not granted");
      return;
    }

    if (!this.settings.notificationsEnabled) {
      return;
    }

    // Check per-kind settings
    if (decision.kind === "directMessage" && !this.settings.notifyDmEnabled) {
      return;
    }
    if (decision.kind === "directMessageRequest" && !this.settings.notifyDmRequestEnabled) {
      return;
    }

    if (!decision.should_alert) {
      console.log(`Notification suppressed: ${decision.suppression_reason}`);
      return;
    }

    const displayName = nickname || decision.sender_peer_id.substring(0, 8);
    const isRequest = decision.kind === "directMessageRequest";
    
    const title = isRequest ? `Message Request from ${displayName}` : `Message from ${displayName}`;
    const options = {
      body: content,
      icon: "/icon-192.png",
      badge: "/badge-72.png",
      tag: decision.message_id,
      renotify: true,
      data: {
        conversationId: decision.conversation_id,
        senderPeerId: decision.sender_peer_id,
        messageId: decision.message_id,
        isRequest: isRequest,
      },
    };

    if (this.settings.soundEnabled) {
      options.silent = false;
    }

    const notification = new Notification(title, options);

    notification.onclick = () => {
      window.focus();
      notification.close();
      
      // Route based on classification
      if (isRequest) {
        // Navigate to requests inbox
        console.log(`Navigate to requests inbox for ${decision.sender_peer_id}`);
        // In production: window.location.hash = `#requests?sender=${decision.sender_peer_id}`;
      } else {
        // Navigate to conversation
        console.log(`Navigate to conversation ${decision.conversation_id}`);
        // In production: window.location.hash = `#chat/${decision.conversation_id}`;
      }
    };

    console.log(`WS14: ${isRequest ? "DM Request" : "DM"} notification shown for ${displayName}`);
  },

  updateSettings(newSettings) {
    this.settings = { ...this.settings, ...newSettings };
    console.log("Notification settings updated:", this.settings);
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

    // WS14: Request notification permission
    UI.addLog("Requesting notification permission...");
    await NotificationHelper.requestPermission();
    if (NotificationHelper.permissionGranted) {
      UI.addLog("Notification permission granted.");
    } else {
      UI.addLog("Notification permission denied or not supported.");
    }

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

    // WS14: Set up message notification listener
    // In production, this would be driven by incoming message events
    // For now, expose a global function for testing
    window.showTestNotification = function(senderPeerId, messageId, content, nickname, isKnownContact, hasExistingConversation) {
      const messageContext = {
        conversation_id: null,
        sender_peer_id: senderPeerId,
        message_id: messageId,
        explicit_dm_request: null,
        sender_is_known_contact: isKnownContact || false,
        has_existing_conversation: hasExistingConversation || false,
        is_self_originated: false,
        is_duplicate: false,
        already_seen: false,
        is_blocked: false,
      };

      const uiState = {
        app_in_foreground: document.hasFocus(),
        active_conversation_id: null,
      };

      try {
        const decision = core.classifyNotification(messageContext, uiState);
        NotificationHelper.showNotification(decision, content, nickname);
        UI.addLog(`Notification classified as: ${decision.kind}`);
      } catch (err) {
        console.error("Notification classification failed:", err);
      }
    };

    UI.addLog("WS14: Notification system ready. Use window.showTestNotification() to test.");

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
