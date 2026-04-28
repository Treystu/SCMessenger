// SCMessenger Web UI — Complete Feature Parity Application Logic
// Mirrors Android: Conversations, Contacts, Mesh Dashboard, Settings, Chat
// Wires EVERY available WASM API: IronCore, ContactManager, HistoryManager

const DEFAULT_BOOTSTRAP = [
  "/ip4/127.0.0.1/tcp/9001/ws/p2p/12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag",
  "/ip4/34.135.34.73/tcp/9001/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
  "/ip4/104.28.216.43/tcp/9010/ws/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9",
];
const BS_KEY = "scm.desktop.bootstrap.v1";
const ONBOARD_KEY = "scm.desktop.onboard.v1";

// All MeshSettings fields (camelCase matches WasmMeshSettings)
const SETTING_KEYS = [
  "relayEnabled","maxRelayBudget","batteryFloor","bleEnabled","wifiAwareEnabled",
  "wifiDirectEnabled","internetEnabled","discoveryMode",
  "notificationsEnabled","notifyDmEnabled","notifyDmRequestEnabled",
  "notifyDmInForeground","notifyDmRequestInForeground","soundEnabled","badgeEnabled"
];

// Switch element IDs that map to settings keys
const SWITCH_IDS = {
  relayEnabled: "sw-relay",
  internetEnabled: "sw-internetEnabled"
};

const SCM = {
  state: {
    wasm: null, core: null, contactMgr: null, historyMgr: null,
    identity: null, role: "relay", settings: {},
    swarmRunning: false, bootstrapAddrs: [],
    contacts: {}, messages: {}, peers: [],
    activeChat: null, dashTimer: null, inboxTimer: null, startTime: Date.now(),
    cliBridgeAvailable: true, // Assume CLI is available until proven otherwise
    transportMode: "standalone" // "standalone", "cli-bridge", or "headless-node"
  },

  async init() {
    this.state.bootstrapAddrs = this.loadBootstrap();
    const bsEl = document.getElementById("setting-bootstrap");
    if (bsEl) bsEl.value = this.state.bootstrapAddrs.join("\n");

    try {
      await this.loadWasm();
      await this.initCore();
      
      // Sync with CLI node info
      await this.syncWithCliNode();
      
      // Connect specifically to Daemon Bridge JSON-RPC
      await this.connectDaemonBridge();
      
      // Fetch identity from CLI JSON-RPC instead of local WebRTC swarm
      await this.refreshContacts();
      this.bindSettingsListeners();
      this.startPolling();
      await this.refreshDashboard();
    } catch (e) {
      this.showSnackbar("Runtime error: " + e.message);
      console.error(e);
    }
  },

  // ===== WASM =====
  async loadWasm() {
    const paths = ["/wasm/pkg/scmessenger_wasm.js", "../wasm/pkg/scmessenger_wasm.js", "./wasm/pkg/scmessenger_wasm.js"];
    let err;
    for (const p of paths) {
      try {
        const mod = await import(p);
        if (typeof mod.default === "function") await mod.default();
        if (!mod.IronCore) throw new Error("IronCore missing");
        this.state.wasm = mod;
        return;
      } catch (e) { err = e; }
    }
    throw new Error("Failed to load WASM: " + err);
  },

  async initCore() {
    const { IronCore } = this.state.wasm;
    let core;
    if (typeof IronCore.withStorageAsync === "function") {
      core = await IronCore.withStorageAsync("scm_desktop");
    } else if (typeof IronCore.withStorage === "function") {
      core = IronCore.withStorage("scm_desktop");
    } else {
      core = new IronCore();
    }
    core.start();
    this.state.core = core;
    this.state.contactMgr = core.getContactManager();
    this.state.historyMgr = core.getHistoryManager();

    // Load settings from core
    const s = core.getSettings();
    this.state.settings = {};
    SETTING_KEYS.forEach(k => { this.state.settings[k] = s[k]; });
  },

  async startSwarm() {
    if (this.state.swarmRunning) return;
    try {
      console.log("Starting swarm...");
      await this.state.core.startSwarm(this.state.bootstrapAddrs);
      this.state.swarmRunning = true;
      this.state.startTime = Date.now();
      
      // Proactively try to discover and sync with the local CLI node
      await this.syncWithCliNode();

      this.showSnackbar("Mesh active");
      this.refreshDashboard();
    } catch (e) {
      this.showSnackbar("Swarm error: " + e.message);
    }
  },

  async syncWithCliNode() {
    try {
      const resp = await fetch("/api/network-info");
      if (resp.ok) {
        const info = await resp.json();
        const cliPeerId = info.node.peer_id;
        console.log("Found local CLI headless node:", cliPeerId);
        
        // Check if CLI advertises itself as headless node with forwarding support
        const isHeadlessNode = info.transport?.is_headless_node || false;
        const supportsForwarding = info.transport?.supports_forwarding || false;
        const wsBridgePort = info.transport?.ws_bridge_port || 9002;
        const cliCapabilities = info.transport?.capabilities || ["Internet", "Local"];
        
        // Set CLI bridge availability based on headless node status
        this.state.cliBridgeAvailable = isHeadlessNode && supportsForwarding;
        this.state.transportMode = isHeadlessNode ? "cli-bridge" : "standalone";
        
        if (this.state.cliBridgeAvailable) {
          console.log("CLI is configured as headless node with forwarding support");
          console.log("Available transport capabilities:", cliCapabilities);
          
          // Register CLI's transport capabilities with the bridge
          await this.registerPeerWithTransportBridge(cliPeerId, cliCapabilities);
          
          const bridgeHost = window.location.hostname || "127.0.0.1";
          // Use the advertised WebSocket bridge port
          const localWsAddr = `/ip4/${bridgeHost}/tcp/${wsBridgePort}/ws/p2p/${cliPeerId}`;
          console.log("Connecting to CLI WebSocket bridge:", localWsAddr);
          
          try {
            if (this.state.core.dial) {
              await this.state.core.dial(localWsAddr);
              console.log("Successfully connected to CLI bridge via dial()");
            } else {
              console.warn("WASM core.dial() is not available! This usually means the WASM package is stale and needs to be rebuilt with 'wasm-pack build'.");
              console.warn("Attempting to add bridge to bootstrap list instead...");
              
              if (!this.state.bootstrapAddrs.includes(localWsAddr)) {
                this.state.bootstrapAddrs.push(localWsAddr);
                // If the swarm is already running, we might still be disconnected from bridge
                // but at least it will be tried on next restart.
              }
            }
            
            this.state.cliBridgeAvailable = true; // Optimistic if dial didn't throw
            
            // Listen on a relay circuit to ensure we are visible
            // This makes the WASM node reachable via the CLI's public/mDNS addresses
            try {
              if (this.state.core.listenOn) {
                await this.state.core.listenOn("/p2p-circuit");
                console.log("Reserved relay circuit on CLI bridge - WASM is now reachable via CLI");
              }
            } catch(e) {
              console.warn("Relay circuit reservation skipped:", e);
            }
            
          } catch (dialError) {
            console.error("Failed to connect to CLI WebSocket bridge:", dialError);
            this.state.cliBridgeAvailable = false;
            this.state.transportMode = "standalone";
            // Don't rethrow, just continue in standalone mode
          }
        } else {
          console.log("CLI detected but not configured as headless node with forwarding support");
          this.state.cliBridgeAvailable = false;
          this.state.transportMode = "standalone";
        }
      }
    } catch (e) {
      console.log("CLI node not detected locally:", e.message);
      // Set standalone mode flag for UI feedback
      this.state.cliBridgeAvailable = false;
      this.state.transportMode = "standalone";
      
      // Fallback: Try to connect to public bootstrap nodes if available
      if (this.state.bootstrapAddrs.length > 0) {
        console.log("Falling back to public bootstrap nodes - running in standalone mode");
        // WASM will work in standalone mode with internet relay
      } else {
        console.warn("No bootstrap nodes available - WASM will work in isolated mode");
      }
    }
  },

  async stopSwarm() {
    if (!this.state.swarmRunning) return;
    try { await this.state.core.stopSwarm(); } catch(_) {}
    this.state.swarmRunning = false;
  },

  async connectDaemonBridge() {
    return new Promise((resolve, reject) => {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = protocol + '//' + window.location.host + '/ws';
      this.state.bridgeWs = new WebSocket(wsUrl);

      this.state.bridgeWs.onopen = () => {
        console.log("Daemon Bridge connected.");
        
        // Request identity blocking UI
        const id = "req_" + Date.now();
        const handler = (e) => {
          try {
            const data = JSON.parse(e.data);
            if (data.id === id) {
              this.state.bridgeWs.removeEventListener('message', handler);
              if (data.error) {
                console.error("Identity error:", data.error);
                return reject(new Error(data.error.message));
              }
              
              this.state.identity = {
                initialized: true,
                identityId: data.result.peer_id,
                publicKeyHex: data.result.public_key,
                deviceId: "daemon",
                libp2pPeerId: data.result.peer_id,
              };
              this.state.role = "full";
              this.ui.updateIdentityUI(this.state.identity);
              this.ui.applyRoleGating();
              
              const obs = document.getElementById("modal-onboarding");
              if (obs) obs.classList.remove("visible");
              
              resolve();
            }
          } catch(err) {}
        };
        this.state.bridgeWs.addEventListener('message', handler);
        this.state.bridgeWs.send(JSON.stringify({ jsonrpc: "2.0", id, method: "get_identity", params: {} }));
      };

      this.state.bridgeWs.onmessage = (event) => {
        try {
          const payload = JSON.parse(event.data);
          if (payload.method === "message_received") {
             const p = payload.params;
             const ts = p.timestamp || Date.now();
             const msgId = p.message_id || "in-" + Date.now() + "-" + Math.random().toString(36).slice(2,8);
             
             this.ensureContact(p.from);
             const msg = { id: msgId, direction: "received", content: p.content, timestamp: ts, status: "read" };
             this.storeMsg(p.from, msg);
             this.addHistory({ id: msgId, direction: "Received", peer_id: p.from, content: p.content, timestamp: Math.floor(ts/1000), sender_timestamp: Math.floor(ts/1000), delivered: true, hidden: false });
             
             if (this.state.activeChat === p.from) this.ui.renderChatMessages(p.from);
             this.ui.renderConversations();
          } else if (payload.method === "peer_discovered") {
             this.ensureContact(payload.params.peer_id);
          }
        } catch(_) {}
      };

      this.state.bridgeWs.onerror = (e) => reject(new Error("Daemon Bridge WebSocket error"));
    });
  },

  // ===== BOOTSTRAP =====
  loadBootstrap() {
    try {
      const r = localStorage.getItem(BS_KEY);
      if (r) { const a = JSON.parse(r); if (Array.isArray(a) && a.length) return a; }
    } catch (_) {}
    return [...DEFAULT_BOOTSTRAP];
  },

  persistBootstrap(nodes) { localStorage.setItem(BS_KEY, JSON.stringify(nodes)); },

  // ===== POLLING =====
  startPolling() {
    clearInterval(this.state.dashTimer);
    clearInterval(this.state.inboxTimer);
    this.state.dashTimer = setInterval(() => this.refreshDashboard().catch(console.warn), 2500);
    this.state.inboxTimer = setInterval(() => this.pollInbox().catch(console.warn), 900);
  },

  async refreshIdentity() {
    const info = this.state.core.getIdentityInfo();
    this.state.identity = info;
    this.state.role = info.initialized ? "full" : "relay";
    this.ui.updateIdentityUI(info);
    this.ui.applyRoleGating();
  },

  async refreshDashboard() {
    if (!this.state.core) return;
    const el = id => document.getElementById(id);
    const peers = await this.state.core.getPeers().catch(() => []);
    this.state.peers = Array.isArray(peers) ? peers : [];

    // Mesh tab updates - filter to only legitimate peers (with topics or contacts)
    const legitimatePeers = this.state.peers.filter(peerId => {
      const c = this.state.contacts[peerId];
      return c || (this.state.knownTopics?.[peerId]?.length > 0);
    });
    
    el("mesh-peer-count").textContent = legitimatePeers.length;
    el("mesh-peer-label").textContent = legitimatePeers.length === 1 ? "Peer" : "Peers";

    const path = await this.state.core.getConnectionPathState().catch(() => "Disconnected");
    el("mesh-conn-path").textContent = path;

    el("mesh-nat").textContent = this.state.core.getNatStatus ? this.state.core.getNatStatus() : "unknown";
    el("mesh-inbox-outbox").textContent = (this.state.core.inboxCount() || 0) + " / " + (this.state.core.outboxCount() || 0);

    const running = this.state.swarmRunning;
    el("mesh-status-card").className = running ? "status-card" : "status-card inactive";
    el("mesh-status-title").textContent = running ? "Mesh Active" : "Mesh Stopped";
    el("mesh-status-sub").textContent = "State: " + (running ? "Running" : "Stopped");
    el("svc-status-text").textContent = "Status: " + (running ? "Running" : "Stopped");
    el("svc-active-text").textContent = running ? "Active" : "";
    el("btn-toggle-service").textContent = running ? "Stop" : "Start";

    const uptimeSec = Math.floor((Date.now() - this.state.startTime) / 1000);
    el("mesh-uptime").textContent = Math.floor(uptimeSec/3600) + "h " + Math.floor((uptimeSec%3600)/60) + "m";

    // Contact online status
    const onlineSet = new Set(this.state.peers);
    Object.values(this.state.contacts).forEach(c => {
      const wasOnline = c.status;
      c.status = onlineSet.has(c.peer_id) ? "online" : "offline";
      if (c.status === "online" && wasOnline !== "online") {
        try { this.state.contactMgr.updateLastSeen(c.peer_id); } catch(_) {}
      }
    });

    // Stats from history manager
    try {
      const stats = this.state.historyMgr.stats();
      const total = stats.total_messages ?? stats.totalMessages ?? 0;
      const sent = stats.sent_count ?? stats.sentCount ?? 0;
      const recv = stats.received_count ?? stats.receivedCount ?? 0;
      const undel = stats.undelivered_count ?? stats.undeliveredCount ?? 0;
      el("conv-stats").classList.remove("hidden");
      el("st-total").textContent = total;
      el("st-sent").textContent = sent;
      el("st-recv").textContent = recv;
      el("st-deliv").textContent = Math.max(0, sent - undel);
      el("info-messages").textContent = total;
      el("info-undelivered").textContent = undel;
    } catch (_) {}

    el("info-contacts").textContent = Object.keys(this.state.contacts).length;
    el("info-inbox-outbox").textContent = (this.state.core.inboxCount() || 0) + " / " + (this.state.core.outboxCount() || 0);

    // Blocked count
    try { el("blocked-count").textContent = this.state.core.blockedCount(); } catch(_) {}

    this.ui.renderMeshPeers();
    this.ui.renderConversations();
  },

  async pollInbox() {
    if (!this.state.core) return;
    const drained = Array.from(this.state.core.drainReceivedMessages());
    for (const inc of drained) {
      const peerId = inc.senderPeerId || inc.sender_peer_id || inc.senderId || inc.sender_id;
      if (!peerId) continue;

      // Check if peer is blocked before processing
      try { if (this.state.core.isPeerBlocked(peerId)) continue; } catch(_) {}

      await this.ensureContact(peerId);
      const ts = Number(inc.timestamp || 0) * 1000 || Date.now();
      const text = inc.text || "";
      const msgId = inc.id || "in-" + Date.now() + "-" + Math.random().toString(36).slice(2,8);

      const msg = { id: msgId, direction: "received", content: text, timestamp: ts, status: "read" };
      this.storeMsg(peerId, msg);
      this.addHistory({ id: msgId, direction: "Received", peer_id: peerId, content: text, timestamp: Math.floor(ts/1000), sender_timestamp: Math.floor(ts/1000), delivered: true, hidden: false });

      // Update UI to show new message
      if (this.state.activeChat === peerId) {
        this.ui.renderChatMessages(peerId);
      }
      this.ui.renderConversations();

      // Send delivery receipt back
      const c = this.state.contacts[peerId];
      if (c?.public_key) {
        try {
          const receipt = this.state.core.prepareReceipt(c.public_key, msgId);
          await this.state.core.sendPreparedEnvelope(peerId, receipt).catch(() => {});
        } catch(_) {}
      }
    }
  },

  async ensureContact(peerId) {
    if (this.state.contacts[peerId]) return;
    let pk = "";
    try { pk = this.state.core.extractPublicKeyFromPeerId(peerId); } catch (_) { pk = "00".repeat(32); }
    const now = Math.floor(Date.now()/1000);
    const c = { peer_id: peerId, nickname: null, local_nickname: "Peer " + peerId.slice(0,8), public_key: pk, added_at: now, last_seen: now, notes: null, status: "online" };
    try { this.state.contactMgr.add(c); } catch(_) {}
    this.state.contacts[peerId] = { ...c, displayName: c.local_nickname };
    this.ui.renderContacts();
  },

  // ===== TRANSPORT BRIDGE INTEGRATION =====
  
  async registerPeerWithTransportBridge(peerId, capabilities) {
    try {
      const response = await fetch("/api/transport/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ peer_id: peerId, capabilities })
      });
      
      if (!response.ok) {
        console.warn("Failed to register peer transport capabilities:", await response.text());
      } else {
        console.log("Registered transport capabilities for", peerId);
      }
    } catch (e) {
      console.warn("Transport registration failed:", e.message);
    }
  },
  
  async fetchTransportCapabilities() {
    try {
      const response = await fetch("/api/transport/capabilities");
      if (response.ok) {
        const data = await response.json();
        console.log("Transport capabilities:", data);
        return data;
      }
    } catch (e) {
      console.warn("Failed to fetch transport capabilities:", e.message);
    }
    return { cli_capabilities: [], peer_capabilities: {} };
  },
  
  async fetchTransportPaths(peerId) {
    try {
      const response = await fetch(`/api/transport/paths/${peerId}`);
      if (response.ok) {
        const data = await response.json();
        console.log("Available transport paths to", peerId, ":", data);
        return data.paths || [];
      }
    } catch (e) {
      console.warn("Failed to fetch transport paths:", e.message);
    }
    return [];
  },
  
  async selectBestTransportPath(peerId) {
    try {
      // Check if we have CLI bridge available
      if (this.state.cliBridgeAvailable) {
        const paths = await this.fetchTransportPaths(peerId);
        if (paths.length === 0) return null;
        
        // Return the highest reliability path
        return paths.reduce((best, current) => 
          current.reliability > best.reliability ? current : best
        );
      } else {
        // Standalone mode - use direct connection or relay
        console.log("Using standalone transport mode for", peerId);
        return null; // Let core handle direct connection
      }
    } catch (e) {
      console.warn("Failed to fetch transport paths:", e);
      // Fallback to standalone mode
      this.state.cliBridgeAvailable = false;
      this.state.transportMode = "standalone";
      return null;
    }
  },

  async checkCliForwardingCapabilities() {
    try {
      const resp = await fetch("/api/network-info");
      if (resp.ok) {
        const info = await resp.json();
        const transportInfo = info.transport;
        
        if (transportInfo) {
          const canForward = transportInfo.supports_forwarding || false;
          const capabilities = transportInfo.capabilities || [];
          
          console.log("CLI Forwarding Capabilities:", {
            canForward,
            capabilities,
            isHeadlessNode: transportInfo.is_headless_node
          });
          
          return {
            canForward,
            capabilities,
            wsBridgePort: transportInfo.ws_bridge_port || 9002,
            isHeadlessNode: transportInfo.is_headless_node || false
          };
        }
      }
      return null;
    } catch (e) {
      console.warn("Failed to check CLI forwarding capabilities:", e);
      return null;
    }
  },

  async ensureCliBridgeConnected() {
    if (!this.state.cliBridgeAvailable) {
      console.log("CLI bridge not available - attempting to connect...");
      await this.syncWithCliNode();
    }
    
    if (this.state.cliBridgeAvailable) {
      console.log("CLI bridge is connected and ready for forwarding");
      return true;
    } else {
      console.log("No CLI bridge available - running in standalone mode");
      return false;
    }
  },

  storeMsg(peerId, msg) {
    if (!this.state.messages[peerId]) this.state.messages[peerId] = [];
    if (this.state.messages[peerId].find(m => m.id === msg.id)) return;
    this.state.messages[peerId].push(msg);
    this.state.messages[peerId].sort((a,b) => a.timestamp - b.timestamp);
    if (this.state.activeChat === peerId) this.ui.renderChatMessages(peerId);
  },

  addHistory(rec) {
    if (!this.state.historyMgr) return;
    try { this.state.historyMgr.add(rec); } catch (_) {}
  },

  async refreshContacts() {
    try {
      const list = Array.from(this.state.contactMgr.list());
      this.state.contacts = {};
      for (const c of list) {
        this.state.contacts[c.peer_id] = {
          ...c,
          displayName: c.local_nickname || c.nickname || "Peer " + c.peer_id.slice(0,8),
          status: "offline",
        };
      }
    } catch(_) {}
    this.ui.renderContacts();
  },

  showSnackbar(text, duration = 3000) {
    const el = document.getElementById("snackbar");
    if (!el) return;
    el.textContent = text;
    el.classList.add("show");
    setTimeout(() => el.classList.remove("show"), duration);
  },

  async maybeOnboard() {
    if (this.state.identity?.initialized) { localStorage.setItem(ONBOARD_KEY, "done"); return; }
    if (localStorage.getItem(ONBOARD_KEY) === "done") return;
    
    // Try to pre-fill nickname from CLI node for "perfect merge" feel
    try {
      const resp = await fetch("/api/network-info");
      if (resp.ok) {
        const info = await resp.json();
        const input = document.getElementById("onboard-nickname");
        if (input && info.node.nickname) {
          input.value = info.node.nickname + " (Web)";
        }
      }
    } catch(e) {}

    document.getElementById("modal-onboarding")?.classList.add("visible");
  },

  // Sync all setting switches to match core settings
  syncSettingsUI() {
    Object.entries(SWITCH_IDS).forEach(([key, elId]) => {
      const sw = document.getElementById(elId);
      if (sw) sw.classList.toggle("on", !!this.state.settings[key]);
    });
    const sel = document.getElementById("sel-discoveryMode");
    if (sel) sel.value = this.state.settings.discoveryMode || "normal";
  },
  bindSettingsListeners() {
    Object.entries(SWITCH_IDS).forEach(([key, elId]) => {
      const sw = document.getElementById(elId);
      if (!sw) return;
      sw.onclick = () => {
        const newVal = !sw.classList.contains("on");
        sw.classList.toggle("on", newVal);
        this.state.settings[key] = newVal;
        this.saveSettings();
      };
    });

    const sel = document.getElementById("sel-discoveryMode");
    if (sel) {
      sel.onchange = () => {
        this.state.settings.discoveryMode = sel.value;
        this.saveSettings();
      };
    }

    document.getElementById("setting-bootstrap-save")?.addEventListener("click", () => {
      const val = document.getElementById("setting-bootstrap")?.value || "";
      const addrs = val.split("\n").map(s => s.trim()).filter(Boolean);
      this.state.bootstrapAddrs = addrs;
      localStorage.setItem(BS_KEY, JSON.stringify(addrs));
      this.showSnackbar("Bootstrap nodes saved. Restart swarm to apply.");
    });
  },

  saveSettings() {
    if (!this.state.core) return;
    try {
      this.state.core.updateSettings(this.state.settings);
      this.showSnackbar("Settings saved.");
      // If relay toggled, we might need to restart swarm or just notify core
    } catch (e) {
      this.showSnackbar("Failed to save settings: " + e.message);
    }
  },

  // ===== ACTIONS =====
  actions: {
    async initializeIdentity() {
      try {
        SCM.state.core.initializeIdentity();
        const nick = document.getElementById("id-nickname")?.value?.trim() ||
                     document.getElementById("onboard-nickname")?.value?.trim();
        if (nick) SCM.state.core.setNickname(nick);
        await SCM.refreshIdentity();
        await SCM.stopSwarm();
        await SCM.startSwarm();
        await SCM.refreshContacts();
        localStorage.setItem(ONBOARD_KEY, "done");
        SCM.ui.closeModals();
        SCM.showSnackbar("Identity created!");
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    async onboardInit() {
      const nick = document.getElementById("onboard-nickname")?.value?.trim();
      if (nick) {
        const el = document.getElementById("id-nickname");
        if (el) el.value = nick;
      }
      await SCM.actions.initializeIdentity();
    },

    skipOnboarding() {
      localStorage.setItem(ONBOARD_KEY, "done");
      SCM.ui.closeModals();
    },

    async importIdentity() {
      const input = document.getElementById("import-backup-input");
      const backup = input?.value?.trim();
      if (!backup) { SCM.showSnackbar("Paste a backup string first."); return; }
      try {
        SCM.state.core.importIdentityBackup(backup);
        await SCM.refreshIdentity();
        await SCM.stopSwarm();
        await SCM.startSwarm();
        await SCM.refreshContacts();
        localStorage.setItem(ONBOARD_KEY, "done");
        input.value = "";
        SCM.showSnackbar("Identity imported!");
      } catch (e) { SCM.showSnackbar("Import failed: " + e); }
    },

    async toggleMeshService() {
      try {
        if (SCM.state.swarmRunning) { await SCM.stopSwarm(); } else { await SCM.startSwarm(); }
      } catch(e) { SCM.showSnackbar("Error: " + e); }
      await SCM.refreshDashboard();
    },

    // Generic settings toggle for any boolean setting
    toggleSetting(key) {
      const s = SCM.state.core.getSettings();
      s[key] = !s[key];
      try {
        SCM.state.core.updateSettings(s);
        SCM.state.settings[key] = s[key];
        // Update switch UI
        const elId = SWITCH_IDS[key];
        if (elId) document.getElementById(elId)?.classList.toggle("on", s[key]);
        SCM.showSnackbar(key + " " + (s[key] ? "enabled" : "disabled"));
      } catch(e) { SCM.showSnackbar("Settings error: " + e); }
    },

    resetSettingsToDefault() {
      if (!confirm("Reset mesh network settings to defaults?")) return;
      try {
        const defaults = SCM.state.core.getDefaultSettings();
        SCM.state.core.updateSettings(defaults);
        SCM.showSnackbar("Settings reset. Reloading...");
        setTimeout(() => location.reload(), 1000);
      } catch (e) { SCM.showSnackbar("Reset failed: " + e); }
    },

    setDiscoveryMode(mode) {
      const s = SCM.state.core.getSettings();
      s.discoveryMode = mode;
      try {
        SCM.state.core.updateSettings(s);
        SCM.state.settings.discoveryMode = mode;
        SCM.showSnackbar("Discovery mode: " + mode);
      } catch(e) { SCM.showSnackbar("Settings error: " + e); }
    },

    saveBootstrap() {
      const raw = document.getElementById("setting-bootstrap")?.value || "";
      const nodes = [...new Set(raw.split("\n").map(l=>l.trim()).filter(Boolean))];
      SCM.state.bootstrapAddrs = nodes.length ? nodes : [...DEFAULT_BOOTSTRAP];
      SCM.persistBootstrap(SCM.state.bootstrapAddrs);
      SCM.showSnackbar("Bootstrap saved. Restart mesh to apply.");
    },

    async dialPeer() {
      const addr = document.getElementById("dial-multiaddr")?.value?.trim();
      if (!addr) { SCM.showSnackbar("Enter a multiaddr."); return; }
      try {
        await SCM.state.core.dial(addr);
        SCM.showSnackbar("Dialing " + addr.slice(0,30) + "...");
      } catch(e) { SCM.showSnackbar("Dial failed: " + e); }
    },

    pasteIdentityExport() {
      navigator.clipboard.readText().then(text => {
        try {
          const parsed = JSON.parse(text);
          if (parsed.peerId) document.getElementById("add-peer-id").value = parsed.peerId;
          else if (parsed.libp2pPeerId) document.getElementById("add-peer-id").value = parsed.libp2pPeerId;
          if (parsed.nickname) document.getElementById("add-name").value = parsed.nickname;
          document.getElementById("add-contact-error").style.display = "none";
          SCM.showSnackbar("Identity export parsed.");
        } catch(_) {
          // Not JSON, try raw peer ID
          document.getElementById("add-peer-id").value = text.trim();
          SCM.showSnackbar("Pasted as raw Peer ID.");
        }
      }).catch(() => SCM.showSnackbar("Clipboard access denied."));
    },

    async addContact() {
      if (SCM.state.role !== "full") { SCM.showSnackbar("Initialize identity first."); return; }
      const peerId = document.getElementById("add-peer-id")?.value?.trim();
      if (!peerId) { SCM.showSnackbar("Peer ID required."); return; }
      const name = document.getElementById("add-name")?.value?.trim();
      try {
        let pk;
        try { pk = SCM.state.core.extractPublicKeyFromPeerId(peerId); }
        catch(_) { pk = "00".repeat(32); }
        const now = Math.floor(Date.now()/1000);
        const c = { peer_id: peerId, nickname: null, local_nickname: name || null, public_key: pk, added_at: now, last_seen: now, notes: null };
        SCM.state.contactMgr.add(c);
        SCM.state.contacts[peerId] = { ...c, displayName: name || "Peer " + peerId.slice(0,8), status: "offline" };
        SCM.state.messages[peerId] = SCM.state.messages[peerId] || [];
        SCM.ui.renderContacts();
        SCM.ui.closeModals();
        document.getElementById("add-peer-id").value = "";
        document.getElementById("add-name").value = "";
        SCM.showSnackbar("Contact added.");
      } catch (e) {
        const errEl = document.getElementById("add-contact-error");
        if (errEl) { errEl.textContent = String(e); errEl.style.display = "block"; }
        SCM.showSnackbar("Failed: " + e);
      }
    },

    async addContactAndChat() {
      const peerId = document.getElementById("add-peer-id")?.value?.trim();
      await SCM.actions.addContact();
      if (peerId && SCM.state.contacts[peerId]) SCM.ui.openChat(peerId);
    },

    quickAddFromChat() {
      const peerId = SCM.state.activeChat;
      if (!peerId) return;
      document.getElementById("add-peer-id").value = peerId;
      document.getElementById("modal-add-contact")?.classList.add("visible");
    },

    removeContact(peerId) {
      if (!confirm("Delete " + (SCM.state.contacts[peerId]?.displayName || "contact") + "?")) return;
      try {
        SCM.state.contactMgr.remove(peerId);
        delete SCM.state.contacts[peerId];
        delete SCM.state.messages[peerId];
        if (SCM.state.activeChat === peerId) SCM.ui.closeChat();
        SCM.ui.renderContacts();
        SCM.ui.renderConversations();
        SCM.showSnackbar("Contact removed.");
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    editNickname(peerId) {
      const c = SCM.state.contacts[peerId];
      document.getElementById("edit-nick-peer-id").value = peerId;
      document.getElementById("edit-nick-input").value = c?.local_nickname || c?.nickname || "";
      document.getElementById("edit-nick-info").textContent = "Set a local nickname for " + peerId.slice(0,16) + "...";
      if (c?.nickname) {
        document.getElementById("edit-nick-info").textContent += "\nFederated nickname: @" + c.nickname;
      }
      document.getElementById("modal-edit-nickname")?.classList.add("visible");
    },

    saveNickname() {
      const peerId = document.getElementById("edit-nick-peer-id").value;
      const name = document.getElementById("edit-nick-input").value.trim();
      try {
        SCM.state.contactMgr.setLocalNickname(peerId, name || null);
        SCM.refreshContacts();
        SCM.ui.closeModals();
        SCM.showSnackbar("Nickname saved.");
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    blockPeer(peerId) {
      if (!confirm("Block this peer?")) return;
      try {
        SCM.state.core.blockPeer(peerId, null);
        SCM.showSnackbar("Peer blocked.");
        SCM.ui.renderBlockedPeers();
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    unblockPeer(peerId) {
      try {
        SCM.state.core.unblockPeer(peerId);
        SCM.showSnackbar("Peer unblocked.");
        SCM.ui.renderBlockedPeers();
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    toggleBlockChatPeer() {
      const peerId = SCM.state.activeChat;
      if (!peerId) return;
      try {
        const blocked = SCM.state.core.isPeerBlocked(peerId);
        if (blocked) { SCM.state.core.unblockPeer(peerId); SCM.showSnackbar("Unblocked."); }
        else { SCM.state.core.blockPeer(peerId, null); SCM.showSnackbar("Blocked."); }
        SCM.ui.updateChatBlockBtn(peerId);
        SCM.ui.renderBlockedPeers();
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    blockAndDeleteChatPeer() {
      const peerId = SCM.state.activeChat;
      if (!peerId) return;
      if (!confirm("Block AND delete all messages from this peer? This cannot be undone.")) return;
      try {
        SCM.state.core.blockAndDeletePeer(peerId, null);
        SCM.state.messages[peerId] = [];
        try { SCM.state.historyMgr.clearConversation(peerId); } catch(_) {}
        SCM.showSnackbar("Peer blocked & messages deleted.");
        SCM.ui.updateChatBlockBtn(peerId);
        SCM.ui.renderBlockedPeers();
        SCM.ui.renderConversations();
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    requestDeleteConversation(peerId) {
      const c = SCM.state.contacts[peerId];
      document.getElementById("delete-conv-text").textContent = "Delete all messages with " + (c?.displayName || peerId.slice(0,12)) + "? This cannot be undone.";
      document.getElementById("delete-conv-peer-id").value = peerId;
      document.getElementById("modal-delete-conv")?.classList.add("visible");
    },

    confirmDeleteConversation() {
      const peerId = document.getElementById("delete-conv-peer-id").value;
      try {
        SCM.state.historyMgr.clearConversation(peerId);
        SCM.state.messages[peerId] = [];
        SCM.ui.closeModals();
        SCM.ui.renderConversations();
        SCM.showSnackbar("Conversation deleted.");
      } catch (e) { SCM.showSnackbar("Failed: " + e); }
    },

    async sendMessage() {
      if (SCM.state.role !== "full") { SCM.showSnackbar("Initialize identity first."); return; }
      const input = document.getElementById("chat-input");
      const text = input.value.trim();
      const peerId = SCM.state.activeChat;
      if (!text || !peerId) return;
      const c = SCM.state.contacts[peerId];
      if (!c?.public_key) { SCM.showSnackbar("Contact missing public key."); return; }
      try {
        const prep = SCM.state.core.prepareMessageWithId(c.public_key, text);
        
        // Ensure CLI bridge is connected for optimal forwarding
        const bridgeConnected = await SCM.ensureCliBridgeConnected();
        
        if (this.state.cliBridgeAvailable && this.state.bridgeWs && this.state.bridgeWs.readyState === WebSocket.OPEN) {
          const rpcId = "send_" + Date.now();
          this.state.bridgeWs.send(JSON.stringify({
            jsonrpc: "2.0",
            id: rpcId,
            method: "send_message",
            params: {
              recipient: peerId,
              message: text,
              id: prep.messageId
            }
          }));
          console.log(`Message forwarded via Daemon bridge to ${peerId}`);
        } else {
          await SCM.state.core.sendPreparedEnvelope(peerId, prep.envelopeData);
          console.log("Using direct connection to", peerId);
        }
        
        // Mark sent in core outbox
        try { SCM.state.core.markMessageSent(prep.messageId); } catch(_) {}
        const nowMs = Date.now();
        SCM.storeMsg(peerId, { id: prep.messageId, direction: "sent", content: text, timestamp: nowMs, status: "sent" });
        SCM.addHistory({ id: prep.messageId, direction: "Sent", peer_id: peerId, content: text, timestamp: Math.floor(nowMs/1000), sender_timestamp: Math.floor(nowMs/1000), delivered: false, hidden: false });
        input.value = "";
        document.getElementById("chat-send-btn").classList.remove("ready");
      } catch (e) { SCM.showSnackbar("Send failed: " + e); }
    },

    exportIdentity() {
      try {
        if (!SCM.state.identity?.initialized) { SCM.showSnackbar("No identity."); return; }
        const backup = SCM.state.core.exportIdentityBackup();
        document.getElementById("export-pub").value = SCM.state.identity.publicKeyHex || "N/A";
        document.getElementById("export-priv").value = backup;
        document.getElementById("modal-export")?.classList.add("visible");
      } catch (e) { SCM.showSnackbar("Export failed: " + e); }
    },

    copyIdentityExport() {
      try {
        const info = SCM.state.identity;
        const payload = JSON.stringify({
          peerId: info.identityId,
          publicKey: info.publicKeyHex,
          nickname: info.nickname,
          libp2pPeerId: info.libp2pPeerId,
          deviceId: info.deviceId,
        });
        navigator.clipboard.writeText(payload);
        SCM.showSnackbar("Identity export copied to clipboard.");
      } catch (e) { SCM.showSnackbar("Copy failed: " + e); }
    },

    async exportDiagnostics() {
      try {
        const diag = await SCM.state.core.exportDiagnostics();
        document.getElementById("diag-output").value = diag;
        document.getElementById("modal-diagnostics")?.classList.add("visible");
      } catch(e) { SCM.showSnackbar("Diagnostics failed: " + e); }
    },

    // ===== QR CODE GENERATION =====
    generateQRCode(text, elementId) {
      try {
        // Use QRCode.js library if available, otherwise provide fallback
        if (typeof QRCode !== 'undefined') {
          const qrElement = document.getElementById(elementId);
          qrElement.innerHTML = "";
          new QRCode(qrElement, {
            text: text,
            width: 200,
            height: 200,
            colorDark: "#000000",
            colorLight: "#ffffff",
            correctLevel: QRCode.CorrectLevel.H,
            // Ensure compatibility with jsQR scanner
            format: 'UTF-8',
            margin: 2
          });
          return true;
        } else {
          // Fallback: Display text if QR library not available
          const fallbackElement = document.getElementById(elementId);
          fallbackElement.textContent = "QR Code Library Not Loaded\n" + text;
          fallbackElement.style.whiteSpace = "pre";
          return false;
        }
      } catch (e) {
        console.error("QR generation failed:", e);
        return false;
      }
    },

    showPeerQRCode() {
      try {
        const info = this.state.core.getIdentityInfo();
        if (!info?.peerId) {
          SCM.showSnackbar("Identity not initialized");
          return;
        }
        
        // Format: SCM:peerId:publicKey
        const qrData = `SCM:${info.peerId}:${info.publicKeyHex || ''}`;
        this.generateQRCode(qrData, "peer-qr-code");
        
        // Also show peer ID for manual copy
        document.getElementById("peer-id-text").textContent = info.peerId;
        document.getElementById("peer-id-text").setAttribute("data-peer-id", info.peerId);
        
        // Show modal
        document.getElementById("modal-peer-qr").classList.add("visible");
        
        // Copy to clipboard button
        document.getElementById("btn-copy-peer-id").onclick = () => {
          navigator.clipboard.writeText(info.peerId);
          SCM.showSnackbar("Peer ID copied to clipboard!");
        };
      } catch (e) {
        SCM.showSnackbar("Failed to generate QR: " + e.message);
      }
    },

    async scanQRCode() {
      try {
        // Check if browser supports camera access
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
          SCM.showSnackbar("Camera access not supported in this browser");
          return;
        }
        
        // Check if we're on HTTPS (required for camera access)
        if (window.location.protocol !== 'https:' && window.location.hostname !== 'localhost' && window.location.hostname !== '127.0.0.1') {
          SCM.showSnackbar("Camera access requires HTTPS or localhost for security");
          return;
        }
        
        // Load QR libraries if not already loaded
        await this.loadQRLibraries();
        
        // Use jsQR or similar library for scanning
        const video = document.getElementById("qr-video");
        const canvas = document.getElementById("qr-canvas");
        const context = canvas.getContext("2d");
        
        // Show scanner modal first
        document.getElementById("modal-qr-scanner").classList.add("visible");
        
        try {
          const stream = await navigator.mediaDevices.getUserMedia({ 
            video: { 
              facingMode: "environment",
              width: { ideal: 1920 },
              height: { ideal: 1080 }
            } 
          });
          
          video.srcObject = stream;
          video.setAttribute("playsinline", true);
          video.play();
          
          // Add error handler for video
          video.onerror = () => {
            SCM.showSnackbar("Camera video error occurred");
            this.cancelQRScan();
          };
          
          let scanInterval = null;
          let scanAttempts = 0;
          const maxAttempts = 100; // 50 seconds max scanning time
          
          scanInterval = setInterval(() => {
            scanAttempts++;
            
            if (scanAttempts >= maxAttempts) {
              SCM.showSnackbar("QR scanning timed out");
              this.cancelQRScan();
              return;
            }
            
            if (video.readyState === video.HAVE_ENOUGH_DATA) {
              try {
                canvas.height = video.videoHeight;
                canvas.width = video.videoWidth;
                context.drawImage(video, 0, 0, canvas.width, canvas.height);
                const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
                
                // Use jsQR to decode
                if (typeof jsQR !== 'undefined') {
                  const code = jsQR(imageData.data, imageData.width, imageData.height, {
                    inversionAttempts: 'attemptBoth'
                  });
                  
                  if (code) {
                    clearInterval(scanInterval);
                    stream.getTracks().forEach(track => track.stop());
                    document.getElementById("modal-qr-scanner").classList.remove("visible");
                    this.handleScannedQR(code.data);
                  }
                } else {
                  if (scanAttempts % 10 === 0) {
                    console.warn("jsQR library not loaded");
                  }
                }
              } catch (scanError) {
                console.error("QR scan error:", scanError);
                if (scanAttempts % 20 === 0) {
                  SCM.showSnackbar("QR scanning error - trying again...");
                }
              }
            }
          }, 500);
          
          // Cancel button
          document.getElementById("btn-cancel-qr").onclick = () => {
            this.cancelQRScan();
          };
          
        } catch (err) {
          console.error("Camera access error:", err);
          SCM.showSnackbar("Camera access denied: " + (err.message || String(err)));
          this.cancelQRScan();
        }
      } catch (e) {
        console.error("QR scan initialization failed:", e);
        SCM.showSnackbar("QR scanning failed: " + e.message);
      }
    },

    async loadQRLibraries() {
      return new Promise((resolve, reject) => {
        // Check if libraries are already loaded
        if (typeof jsQR !== 'undefined') {
          resolve();
          return;
        }
        
        // Load jsQR library dynamically
        const script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/jqrcode@1.0.4/dist/jqrcode.min.js';
        script.onload = () => {
          console.log("jsQR library loaded successfully");
          resolve();
        };
        script.onerror = () => {
          console.error("Failed to load jsQR library");
          reject(new Error("Failed to load QR scanning library"));
        };
        document.head.appendChild(script);
      });
    },

    async handleScannedQR(qrData) {
      try {
        // Parse SCM QR format: SCM:peerId:publicKey
        if (qrData.startsWith("SCM:")) {
          const parts = qrData.split(":");
          if (parts.length >= 3) {
            const peerId = parts[1];
            const publicKey = parts[2];
            
            // Check if we already have this contact
            if (this.state.contacts[peerId]) {
              SCM.showSnackbar("Contact already exists!");
              return;
            }
            
            // Add the contact
            await this.addContact(peerId, publicKey, "Scanned Contact");
            SCM.showSnackbar("Contact added successfully!");
            
            // Switch to chat with new contact
            this.ui.showChat(peerId);
          }
        } else {
          SCM.showSnackbar("Invalid QR code format");
        }
      } catch (e) {
        SCM.showSnackbar("Failed to process QR: " + e.message);
      }
    },

    cancelQRScan() {
      const video = document.getElementById("qr-video");
      if (video && video.srcObject) {
        video.srcObject.getTracks().forEach(track => track.stop());
        video.srcObject = null;
      }
      document.getElementById("modal-qr-scanner").classList.remove("visible");
    },

    exportLogs() {
      try {
        const logs = SCM.state.core.exportLogs();
        document.getElementById("diag-output").value = logs;
        document.getElementById("modal-diagnostics")?.classList.add("visible");
      } catch(e) { SCM.showSnackbar("Log export failed: " + e); }
    },

    runMaintenance() {
      try {
        SCM.state.core.performMaintenance();
        SCM.showSnackbar("Maintenance completed.");
      } catch(e) { SCM.showSnackbar("Maintenance failed: " + e); }
    },

    factoryReset() {
      if (!confirm("DELETE ALL DATA & RESET? This cannot be undone.")) return;
      try {
        try { SCM.state.historyMgr.clear(); } catch(_) {}
        localStorage.clear();
        SCM.showSnackbar("All data deleted. Reloading...");
        setTimeout(() => location.reload(), 1500);
      } catch (e) { SCM.showSnackbar("Reset failed: " + e); }
    },

    async checkHeadlessNodeCompatibility() {
      try {
        // Check if we can detect any headless node (CLI or other)
        const response = await fetch("/api/network-info");
        if (response.ok) {
          const info = await response.json();
          this.state.transportMode = "cli-bridge";
          this.state.cliBridgeAvailable = true;
          console.log("Headless node detected:", info.node.peer_id);
          return true;
        }
      } catch (e) {
        // No headless node detected
        console.log("No headless node detected - running in standalone mode");
        this.state.transportMode = "standalone";
        this.state.cliBridgeAvailable = false;
        return false;
      }
      
      // Try to detect any other headless nodes via transport API
      try {
        const capsResponse = await fetch("/api/transport/capabilities");
        if (capsResponse.ok) {
          const caps = await capsResponse.json();
          if (caps.cli_capabilities.length > 0) {
            console.log("Headless node capabilities:", caps.cli_capabilities);
            return true;
          }
        }
      } catch (e) {
        console.log("No additional headless nodes detected");
      }
      
      return false;
    },
  },

  // ===== UI =====
  ui: {
    switchTab(tab) {
      document.querySelectorAll(".tab-page").forEach(p => p.classList.remove("active"));
      document.querySelectorAll(".nav-item").forEach(n => n.classList.remove("active"));
      document.getElementById("page-" + tab)?.classList.add("active");
      document.querySelector(`.nav-item[data-tab="${tab}"]`)?.classList.add("active");
      document.getElementById("chat-view")?.classList.remove("active");
      document.getElementById("bottom-nav")?.classList.remove("hidden");

      if (tab === "settings") { SCM.ui.renderBlockedPeers(); SCM.ui.updateDiagnostics(); }
      if (tab === "contacts") SCM.ui.renderContacts();
      if (tab === "conversations") SCM.ui.renderConversations();
    },

    applyRoleGating() {
      const full = SCM.state.role === "full";
      document.querySelectorAll(".nav-item").forEach(n => {
        const tab = n.dataset.tab;
        if (tab === "conversations" || tab === "contacts") n.classList.toggle("hidden", !full);
      });
      if (!full) {
        const activePage = document.querySelector(".tab-page.active");
        if (activePage && (activePage.id === "page-conversations" || activePage.id === "page-contacts")) {
          SCM.ui.switchTab("mesh");
        }
      }
      document.getElementById("identity-initialized")?.classList.toggle("hidden", !full);
      document.getElementById("identity-uninitialized")?.classList.toggle("hidden", full);
    },

    updateIdentityUI(info) {
      const el = id => document.getElementById(id);
      if (info.initialized) {
        if (el("id-nickname")) el("id-nickname").value = info.nickname || "";
        if (el("id-identity-id")) el("id-identity-id").textContent = (info.identityId || "").slice(0,8) || "—";
        if (el("id-pub-key")) el("id-pub-key").textContent = (info.publicKeyHex || "").slice(0,8) || "—";
        if (el("id-device-id")) el("id-device-id").textContent = (info.deviceId || SCM.state.core.getDeviceId?.() || "").slice(0,8) || "—";
        if (el("id-libp2p-peer")) el("id-libp2p-peer").textContent = (info.libp2pPeerId || "").slice(0,12) || "—";
      }

      // Bind nickname change
      const nickInput = el("id-nickname");
      if (nickInput && !nickInput._bound) {
        nickInput._bound = true;
        nickInput.addEventListener("change", () => {
          try {
            SCM.state.core.setNickname(nickInput.value.trim());
            SCM.showSnackbar("Nickname updated.");
          } catch (_) {}
        });
      }
    },

    async updateDiagnostics() {
      const el = id => document.getElementById(id);
      try {
        const listeners = await SCM.state.core.getListeners();
        el("diag-listeners").textContent = listeners.length ? listeners.join(", ") : "None";
      } catch(_) { el("diag-listeners").textContent = "N/A"; }
      try {
        const externals = await SCM.state.core.getExternalAddresses();
        el("diag-external").textContent = externals.length ? externals.join(", ") : "None";
      } catch(_) { el("diag-external").textContent = "N/A"; }
    },

    renderConversations() {
      const list = document.getElementById("conv-list");
      const empty = document.getElementById("conv-empty");
      if (!list) return;

      const convMap = {};
      Object.keys(SCM.state.contacts).forEach(peerId => {
        try {
          const records = Array.from(SCM.state.historyMgr.conversation(peerId, 200));
          if (records.length > 0) convMap[peerId] = records;
        } catch(_) {}
      });

      const convIds = Object.keys(convMap).sort((a,b) => {
        const la = convMap[a][convMap[a].length-1];
        const lb = convMap[b][convMap[b].length-1];
        return Number(lb.timestamp) - Number(la.timestamp);
      });

      if (convIds.length === 0) {
        list.innerHTML = "";
        empty?.classList.remove("hidden");
        return;
      }
      empty?.classList.add("hidden");

      list.innerHTML = convIds.map(peerId => {
        const msgs = convMap[peerId];
        const last = msgs[msgs.length - 1];
        const c = SCM.state.contacts[peerId] || {};
        const name = c.displayName || peerId.slice(0,8);
        const dir = last.direction === "Sent" || last.direction?.toLowerCase() === "sent";
        const lastText = dir ? "You: " + last.content : last.content;
        const time = SCM.utils.fmtTime(Number(last.timestamp));
        const online = c.status === "online";

        return `<div class="list-item" onclick="SCM.ui.openChat('${peerId}')">
          <div class="list-avatar" style="background:${SCM.utils.strColor(peerId)}">${(name[0]||"?").toUpperCase()}</div>
          <div class="list-content">
            <div style="display:flex;justify-content:space-between;align-items:center"><span class="list-title">${SCM.utils.esc(name)}</span><span class="list-meta">${time}</span></div>
            <div class="list-subtitle">${SCM.utils.esc(lastText.slice(0,60))}</div>
          </div>
          <div class="list-trailing">
            ${online ? '<div style="width:8px;height:8px;border-radius:50%;background:#4caf50;margin-bottom:4px"></div>' : ''}
            <button class="icon-btn" style="font-size:14px" onclick="event.stopPropagation();SCM.actions.requestDeleteConversation('${peerId}')">🗑️</button>
          </div>
        </div>`;
      }).join("");
    },

    renderContacts() {
      const list = document.getElementById("contacts-list");
      const empty = document.getElementById("contacts-empty");
      const title = document.getElementById("contacts-title");
      if (!list) return;

      const q = document.getElementById("contact-search")?.value?.toLowerCase() || "";
      let contacts = Object.values(SCM.state.contacts);
      if (q) contacts = contacts.filter(c => (c.displayName || "").toLowerCase().includes(q) || c.peer_id.includes(q));

      title.textContent = "Contacts (" + Object.keys(SCM.state.contacts).length + ")";

      if (contacts.length === 0) {
        list.innerHTML = "";
        empty?.classList.remove("hidden");
        return;
      }
      empty?.classList.add("hidden");

      list.innerHTML = contacts.map(c => {
        const name = c.displayName || c.peer_id.slice(0,8);
        const online = c.status === "online";
        const lastSeen = c.last_seen ? SCM.utils.fmtTime(Number(c.last_seen)) : "";
        return `<div class="list-item" onclick="SCM.ui.openChat('${c.peer_id}')">
          <div class="list-avatar" style="background:${SCM.utils.strColor(c.peer_id)}">${(name[0]||"?").toUpperCase()}</div>
          <div class="list-content">
            <div class="list-title">${SCM.utils.esc(name)}${online ? ' <span style="color:#4caf50">●</span>' : ''}</div>
            ${c.nickname && c.local_nickname ? `<div class="list-meta">@${SCM.utils.esc(c.nickname)}</div>` : ""}
            <div class="list-meta">ID: ${c.peer_id.slice(0,16)}...${lastSeen ? ' • Last: ' + lastSeen : ''}</div>
          </div>
          <div class="list-trailing" style="flex-direction:row;gap:0">
            <button class="icon-btn" style="font-size:14px" onclick="event.stopPropagation();SCM.actions.editNickname('${c.peer_id}')" title="Edit Nickname">✏️</button>
            <button class="icon-btn" style="font-size:14px" onclick="event.stopPropagation();SCM.actions.removeContact('${c.peer_id}')" title="Delete">🗑️</button>
          </div>
        </div>`;
      }).join("");
    },

    renderMeshPeers() {
      const list = document.getElementById("mesh-peers-list");
      const empty = document.getElementById("mesh-peers-empty");
      if (!list) return;

      // Filter peers to only show those with known topics (legitimate nodes)
      const legitimatePeers = SCM.state.peers.filter(peerId => {
        const c = SCM.state.contacts[peerId];
        // Show if it's a contact or if we have topic information
        return c || SCM.state.knownTopics?.[peerId]?.length > 0;
      });

      if (legitimatePeers.length === 0) {
        list.innerHTML = "";
        empty?.classList.remove("hidden");
        return;
      }
      empty?.classList.add("hidden");

      // Update peer count to show only legitimate peers
      const peerCountEl = document.getElementById("mesh-peer-count");
      if (peerCountEl) {
        peerCountEl.textContent = legitimatePeers.length;
      }

      list.innerHTML = legitimatePeers.map(peerId => {
        const c = SCM.state.contacts[peerId];
        const name = c?.displayName || "Node";
        const isContact = !!c;
        const statusIndicator = isContact ? ' • Contact' : ' • Unknown';
        return `<div class="list-item">
          <div class="list-avatar" style="background:var(--md-primary-container)">${(name[0]||"N").toUpperCase()}</div>
          <div class="list-content">
            <div class="list-title">${SCM.utils.esc(name)}</div>
            <div class="list-meta">ID: ${peerId.slice(0,12)}... • WebSocket${statusIndicator}</div>
          </div>
          <div style="width:8px;height:8px;border-radius:50%;background:#4caf50"></div>
        </div>`;
      }).join("");
    },

    renderBlockedPeers() {
      const list = document.getElementById("blocked-list");
      const empty = document.getElementById("blocked-empty");
      if (!list || !SCM.state.core) return;
      try {
        const blocked = Array.from(SCM.state.core.listBlockedPeers());
        document.getElementById("blocked-count").textContent = blocked.length;
        if (blocked.length === 0) { list.innerHTML = ""; empty?.classList.remove("hidden"); return; }
        empty?.classList.add("hidden");
        list.innerHTML = blocked.map(b => {
          const pid = b.peerId || b.peer_id || "unknown";
          const reason = b.reason ? ` (${b.reason})` : "";
          const blockedAt = b.blockedAt ? new Date(Number(b.blockedAt) * 1000).toLocaleDateString() : "";
          return `<div style="display:flex;justify-content:space-between;align-items:center;padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.06)">
            <div>
              <span class="mono" style="font-size:12px">${pid.slice(0,16)}...</span>${reason}
              ${blockedAt ? `<div class="list-meta">Blocked: ${blockedAt}</div>` : ""}
            </div>
            <button class="btn btn-text" style="color:var(--md-error);font-size:12px" onclick="SCM.actions.unblockPeer('${pid}')">Unblock</button>
          </div>`;
        }).join("");
      } catch (_) {}
    },

    openAddContact() {
      document.getElementById("modal-add-contact").classList.add("visible");
    },

    openChat(peerId) {
      if (SCM.state.role !== "full") { SCM.showSnackbar("Initialize identity first."); return; }
      SCM.state.activeChat = peerId;
      const c = SCM.state.contacts[peerId];
      document.getElementById("chat-title").textContent = c?.displayName || peerId.slice(0,12);
      document.getElementById("chat-peer-id-short").textContent = peerId.slice(0,20) + "...";

      // Load history from core
      try {
        const records = Array.from(SCM.state.historyMgr.conversation(peerId, 500)).sort((a,b) => Number(a.timestamp) - Number(b.timestamp));
        SCM.state.messages[peerId] = records.map(r => ({
          id: r.id,
          direction: (r.direction === "Sent" || r.direction?.toLowerCase() === "sent") ? "sent" : "received",
          content: r.content,
          timestamp: Number(r.timestamp) * 1000,
          status: r.delivered ? "delivered" : "sent",
        }));
      } catch (_) {}

      SCM.ui.renderChatMessages(peerId);
      SCM.ui.updateChatBlockBtn(peerId);

      document.querySelectorAll(".tab-page").forEach(p => p.classList.remove("active"));
      document.getElementById("chat-view")?.classList.add("active");
      document.getElementById("bottom-nav")?.classList.add("hidden");
      document.getElementById("chat-input")?.focus();
    },

    closeChat() {
      SCM.state.activeChat = null;
      document.getElementById("chat-view")?.classList.remove("active");
      document.getElementById("bottom-nav")?.classList.remove("hidden");
      const activeNav = document.querySelector(".nav-item.active");
      if (activeNav) SCM.ui.switchTab(activeNav.dataset.tab);
      else SCM.ui.switchTab("conversations");
    },

    renderChatMessages(peerId) {
      const container = document.getElementById("chat-messages");
      if (!container) return;
      const msgs = SCM.state.messages[peerId] || [];
      container.innerHTML = msgs.map(m => {
        const mine = m.direction === "sent";
        const time = new Date(m.timestamp).toLocaleTimeString([], {hour: "2-digit", minute: "2-digit"});
        const statusIcon = mine ? (m.status === "delivered" ? " ✓✓" : " ✓") : "";
        return `<div class="msg-row ${mine ? "mine" : "theirs"}">
          <div class="msg-bubble">${SCM.utils.esc(m.content)}</div>
          <div class="msg-time">${time}${statusIcon}</div>
        </div>`;
      }).join("");
      container.scrollTop = container.scrollHeight;
    },

    updateChatBlockBtn(peerId) {
      try {
        const blocked = SCM.state.core.isPeerBlocked(peerId);
        const btn = document.getElementById("chat-block-btn");
        if (btn) { btn.textContent = blocked ? "✅" : "🛡️"; btn.title = blocked ? "Unblock" : "Block"; }
      } catch (_) {}
    },

    closeModals() {
      document.querySelectorAll(".modal-overlay").forEach(m => m.classList.remove("visible"));
    },
  },

  // ===== UTILS =====
  utils: {
    esc(text) {
      const d = document.createElement("div"); d.textContent = text || ""; return d.innerHTML;
    },
    strColor(str) {
      let h = 0;
      for (let i = 0; i < str.length; i++) h = str.charCodeAt(i) + ((h << 5) - h);
      return `hsl(${h % 360}, 40%, 35%)`;
    },
    fmtTime(seconds) {
      if (!seconds) return "";
      const ts = Number(seconds);
      // Handle both seconds and milliseconds
      const d = new Date(ts > 1e12 ? ts : ts * 1000);
      const now = new Date();
      const diff = now - d;
      if (diff < 60000) return "Just now";
      if (diff < 3600000) return Math.floor(diff/60000) + "m ago";
      if (d.toDateString() === now.toDateString()) return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
      if (diff < 86400000) return Math.floor(diff/3600000) + "h ago";
      return d.toLocaleDateString([], { month: "short", day: "numeric" });
    },
  },
};

window.SCM = SCM;

window.addEventListener("DOMContentLoaded", () => {
  // Bind core events
  document.getElementById("chat-send-btn")?.addEventListener("click", () => SCM.actions.sendMessage());
  document.getElementById("chat-input")?.addEventListener("keypress", e => { if (e.key === "Enter") SCM.actions.sendMessage(); });
  document.getElementById("chat-input")?.addEventListener("input", e => {
    document.getElementById("chat-send-btn")?.classList.toggle("ready", e.target.value.trim().length > 0);
  });
  document.getElementById("fab-add-contact")?.addEventListener("click", () => {
    document.getElementById("modal-add-contact")?.classList.add("visible");
  });
  document.getElementById("contact-search")?.addEventListener("input", () => SCM.ui.renderContacts());

  // Close modals on overlay click
  document.querySelectorAll(".modal-overlay").forEach(overlay => {
    overlay.addEventListener("click", e => { if (e.target === overlay) SCM.ui.closeModals(); });
  });

  SCM.init().catch(e => { console.error(e); SCM.showSnackbar("Fatal: " + e.message); });
});