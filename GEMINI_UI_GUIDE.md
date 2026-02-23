> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# SCMessenger UI/UX Design Guide for Gemini 3.0 Pro

## [Needs Revalidation] Your Mission

You are tasked with designing and building a beautiful, intuitive user interface for **SCMessenger** — the world's first truly sovereign messenger. This is not just another chat app. This is communication infrastructure that works everywhere, is owned by no one, and is unstoppable by design.

Your challenge: Make cutting-edge cryptographic mesh networking feel as simple as sending a text message.

---

## [Needs Revalidation] Installation & Deployment Options

Your UI must support both local CLI installation and Docker deployment. Here are the two paths:

### [Needs Revalidation] Option 1: Local CLI Installation

**Prerequisites:**
- Rust 1.70+ (install via [rustup.rs](https://rustup.rs/))
- Git

**Installation Steps:**
```bash
# Clone the repository
git clone https://github.com/YourOrg/SCMessenger.git
cd SCMessenger

# Build the workspace
cargo build --workspace --release

# Install CLI globally
cargo install --path cli

# Or run directly
cargo run -p scmessenger-cli -- --help
```

**Quick Start:**
```bash
# Initialize identity
scm init

# Start the mesh node
scm start --port 9000

# In another terminal, check status
scm status

# Add a contact
scm contact add <peer-id> <public-key> --name "Alice"

# Send a message
scm send Alice "Hello, sovereign world!"
```

**UI Integration:**
Your web UI will communicate with the locally-running CLI via:
- **WebSocket** on `ws://localhost:9000/ws` (for real-time events)
- **HTTP API** on `http://localhost:9000/api` (for commands)
- Or **IPC** if wrapped in Electron/Tauri

### [Needs Revalidation] Option 2: Docker Deployment

**Prerequisites:**
- Docker 20.10+
- Docker Compose 2.0+

**Single Node (Quick Start):**
```bash
# Build and run
docker build -f docker/Dockerfile -t scmessenger:latest .
docker run -it -p 9000:9000 scmessenger:latest scm start --port 9000
```

**Multi-Node Network Simulation:**
```bash
# Build images and start 3-node network (Relay, Alice, Bob)
docker compose -f docker/docker-compose.yml up -d --build

# Verify connectivity
./verify_simulation.sh

# Check node status
docker exec -it scm-alice scm status
docker exec -it scm-bob scm status

# Send test message
docker exec scm-alice scm send <bob-id> "Hello Docker!"

# View logs
docker compose -f docker/docker-compose.yml logs -f

# Tear down
docker compose -f docker/docker-compose.yml down
```

**Production Docker Deployment:**
```dockerfile
# docker-compose.production.yml
version: "3.8"
services:
  scmessenger:
    image: scmessenger:latest
    container_name: scm-node
    environment:
      - RUST_LOG=info
      - LISTEN_PORT=9000
      - BOOTSTRAP_NODES=/ip4/bootstrap.example.com/tcp/4001
    ports:
      - "9000:9000"     # SCMessenger protocol
      - "8080:8080"     # Web UI (your HTML file served via nginx)
    volumes:
      - scm-data:/root/.config/scmessenger
      - ./ui/index.html:/usr/share/nginx/html/index.html
    restart: unless-stopped

volumes:
  scm-data:
```

**UI Deployment Notes:**
- Your single HTML file can be served by nginx/Apache/caddy alongside the SCMessenger node
- For Docker, mount the UI file into the container
- Ensure WebSocket connection points to the correct host (localhost vs container hostname)
- Support both scenarios in your JavaScript (auto-detect environment)

### [Needs Revalidation] Option 3: WASM (Browser-only, Future)

SCMessenger has WASM bindings for browser-native mesh participation:
```html
<script type="module">
  import init, { IronCore } from './pkg/scmessenger_wasm.js';
  await init();
  const core = new IronCore();
  // Your UI interacts directly with WASM, no backend needed
</script>
```

*Note: WASM support is experimental. Focus on CLI/Docker integration first.*

---

## [Needs Revalidation] What You're Building For

### [Needs Revalidation] The Application: SCMessenger

**Core Philosophy:**
- **Sovereign Communication** — No corporations, no servers, no surveillance
- **Works everywhere** — Internet, Bluetooth, WiFi Direct, mesh networks
- **Privacy-first** — End-to-end encrypted, no phone numbers, no accounts
- **Unstoppable** — Every node strengthens the network, self-healing mesh
- **Mass market UX** — Grandma should be able to use this

**Technical Reality:**
- **Backend:** Rust-based core (~53,000 lines of code)
- **Cryptography:** Ed25519 identity, XChaCha20-Poly1305 encryption, onion routing
- **Networking:** libp2p-based mesh with BLE, WiFi Aware, WiFi Direct, Internet transports
- **Storage:** Sled database for persistence, local-first architecture
- **Relay Model:** You cannot message without relaying. You cannot relay without messaging. (Non-negotiable coupling)

**Current Interface:** A command-line tool with these commands:
```bash
scm init                              # Create identity
scm identity show/export              # View identity
scm contact add/list/show/remove      # Manage contacts
scm config set/get/list               # Settings
scm history --peer <name> --limit 20  # View messages
scm start --port 9000                 # Start mesh node
scm send <contact> <message>          # Send message
scm status                            # Network status
scm test                              # Run self-tests
```

**Interactive Mode Commands** (when node is running):
```
send <contact> <message>    # Send message
contacts                    # List contacts
peers                       # Show connected peers
status                      # Network statistics
quit                        # Shutdown
```

---

## [Needs Revalidation] Design Requirements

### [Needs Revalidation] 1. Single-File Web Application
- **ONE HTML FILE** containing CSS and JavaScript inline
- No external dependencies, frameworks, or libraries (except what's in browser)
- Works offline once loaded
- Can be wrapped in Electron/Tauri for desktop deployment
- Communicates with SCMessenger backend via WebSocket/IPC

### [Needs Revalidation] 2. Visual Design Principles

**Aesthetic:**
- **Minimal, not minimalist** — Clean but warm
- **Dark mode first** — Light mode optional (most crypto users prefer dark)
- **Neumorphic or Glass morphism** — Modern, tactile, depth
- **Subtle animations** — Micro-interactions that feel alive (not distracting)
- **Color psychology:**
  - Primary: Deep cyan/teal (#00CED1, #20B2AA) — Trust, security, tech
  - Accent: Electric purple (#9D4EDD, #7B2CBF) — Innovation, sovereignty
  - Success: Vibrant green (#10B981, #059669) — Connection, relay active
  - Warning: Warm amber (#F59E0B, #D97706) — Attention needed
  - Danger: Coral red (#EF4444, #DC2626) — Disconnection, error
  - Background: Rich dark (#0F172A, #1E293B, #334155) — Depth, focus
  - Text: High contrast whites and grays (#F1F5F9, #CBD5E1, #64748B)

**Typography:**
- **Monospace for identity/keys** — `IBM Plex Mono`, `JetBrains Mono`, or `SF Mono`
- **Sans-serif for UI** — `Inter`, `SF Pro`, or system fonts
- **Variable font sizes** — Accessible, readable at all scales

### [Needs Revalidation] 3. Core User Flows

#### [Needs Revalidation] **First Launch (Onboarding)**
```
1. Splash screen with logo and tagline
2. "Welcome to Sovereign Communication" intro
3. Identity generation with visual feedback
   - Show cryptographic key generation (animated)
   - Display identity ID (colorful hash visualization)
   - Explain: "This is YOU. No password, no email, no phone number."
4. Optional: Share your identity (QR code, copy link)
5. "You're Ready" — Enter main interface
```

#### [Needs Revalidation] **Main Interface (Dashboard)**
```
┌────────────────────────────────────────────────────────┐
│  [≡ Menu]  SCMessenger        [Network: ● Online]  [⚙] │
├────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐  ┌──────────────────────────────────┐ │
│  │             │  │                                   │ │
│  │  CONTACTS   │  │      CONVERSATION AREA            │ │
│  │             │  │                                   │ │
│  │  • Alice    │  │  [Messages appear here]           │ │
│  │  • Bob      │  │                                   │ │
│  │  • Carol    │  │                                   │ │
│  │             │  │                                   │ │
│  │ [+ Add]     │  │  [Type message...]  [Send ➤]     │ │
│  └─────────────┘  └──────────────────────────────────┘ │
│                                                         │
├────────────────────────────────────────────────────────┤
│  Relaying for 12 peers • 47 messages today • ⚡ Online │
└────────────────────────────────────────────────────────┘
```

**Key Elements:**
- **Left Sidebar:** Contact list with status indicators
- **Center:** Message thread (Signal/Telegram style)
- **Right Panel (optional):** Contact details, settings, network graph
- **Bottom Status Bar:** Network status, relay activity, message stats
- **Floating Action Button:** Quick actions (new message, add contact, settings)

#### [Needs Revalidation] **Contact Management**
```
Add Contact Flow:
1. Click "+ Add Contact"
2. Modal appears with options:
   - Scan QR code (camera access)
   - Paste identity string
   - Import from file
   - Nearby discovery (show peers on network)
3. Add nickname (optional but encouraged)
4. Confirm and save
5. Contact appears in list with status indicator
```

**Contact Card:**
```
┌─────────────────────────────────┐
│  [Avatar]  Alice                │
│            alice.eth            │
├─────────────────────────────────┤
│  Status: ● Online (2 hops away) │
│  Peer ID: 12D3KooW...           │
│  Public Key: [Copy] [QR]        │
│  Added: 2 weeks ago             │
│  Messages: 143                  │
├─────────────────────────────────┤
│  [Send Message]  [Edit]  [⋮]    │
└─────────────────────────────────┘
```

#### [Needs Revalidation] **Messaging Interface**
```
Conversation View:
┌──────────────────────────────────────────┐
│  ← Alice                    [⋮ Options]  │
├──────────────────────────────────────────┤
│                                           │
│  ┌────────────────┐                      │
│  │ Hey! 👋        │  11:23 AM            │
│  └────────────────┘                      │
│                                           │
│                     ┌──────────────────┐ │
│        11:24 AM     │ Hi Alice! 🎉     │ │
│                     │ ✓✓ Delivered     │ │
│                     └──────────────────┘ │
│                                           │
│  ┌────────────────────────────────────┐  │
│  │ Relayed via 3 hops • Encrypted     │  │
│  └────────────────────────────────────┘  │
│                                           │
├──────────────────────────────────────────┤
│  [📎]  Type your message...    [Send ➤] │
└──────────────────────────────────────────┘
```

**Message States:**
- ◷ Queued (waiting to send)
- ➤ Sending (in transit)
- ✓ Delivered (acknowledged by recipient)
- ✓✓ Read (opened by recipient)
- ⚠ Failed (retry option)

#### [Needs Revalidation] **Network Status Visualization**
```
Network Graph (optional advanced view):
- Node graph showing YOU at center
- Connected peers as circles radiating outward
- Relay paths shown as animated lines
- Hover over nodes to see details
- Color-coded by connection strength/hops
```

**Simple Status Indicators:**
```
Network Status Bar:
┌────────────────────────────────────────────┐
│  ● Online  |  12 peers  |  Relaying  [▓▓▓] │
│  Internet: ✓  BLE: ✓  WiFi: ✓             │
└────────────────────────────────────────────┘
```

Status States:
- **🟢 Online (Internet)** — Connected to mesh via internet
- **🔵 Mesh (Local)** — Connected via BLE/WiFi only
- **🟡 Relaying** — Currently relaying messages for others
- **🟠 Limited** — Partial connectivity
- **🔴 Offline** — No connections (messages queued)

### [Needs Revalidation] 4. Key Features to Visualize

#### [Needs Revalidation] **Identity Display**
- QR code for sharing (animated generation)
- Copy buttons with feedback ("Copied!")
- Colorful hash visualization (identicon or gradient based on ID)
- Security indicator (lock icon, "Encrypted" badge)

#### [Needs Revalidation] **Relay Activity**
- **Live Feed** — Scrolling list of relay events (optional advanced mode)
- **Statistics Dashboard:**
  ```
  ┌─────────────────────────────────────┐
  │  Relay Statistics                   │
  ├─────────────────────────────────────┤
  │  Messages relayed today:      1,247 │
  │  Peers helped:                   34 │
  │  Bandwidth contributed:       2.3GB │
  │  Uptime:                   14h 23m  │
  └─────────────────────────────────────┘
  ```
- **Toggle:** "Relay ON/OFF" — But warn user: "Relaying is required to send messages"

#### [Needs Revalidation] **Settings Panel**
```
Categories:
├─ Identity
│  ├─ Show identity
│  ├─ Export backup
│  └─ QR code
├─ Network
│  ├─ Bootstrap nodes
│  ├─ Port configuration
│  ├─ Transport preferences (Internet, BLE, WiFi)
│  └─ Relay settings
├─ Privacy
│  ├─ Onion routing (default: ON)
│  ├─ Cover traffic (default: ON)
│  └─ Timing obfuscation
├─ Storage
│  ├─ Message retention (days)
│  ├─ Cache size
│  └─ Clear data
└─ About
   ├─ Version info
   ├─ Self-tests
   └─ License
```

#### [Needs Revalidation] **Notifications**
- **System Notifications:**
  - New message received
  - Peer connected/disconnected
  - Network status change
- **In-App Toasts:**
  - Message sent
  - Contact added
  - Settings saved
  - Error messages
- **Visual cues:**
  - Badge count on contact (unread messages)
  - Pulsing dot for activity
  - Color changes for status

### [Needs Revalidation] 5. Advanced Features (Power Users)

**Developer Console (hidden by default, accessible via keyboard shortcut):**
```
Press Ctrl+Shift+D to open:
┌─────────────────────────────────────────┐
│  Developer Console                      │
├─────────────────────────────────────────┤
│  > scm status                           │
│    Peers: 12                            │
│    Contacts: 8                          │
│    Messages: 1,247                      │
│                                         │
│  > scm config list                      │
│    relay_enabled: true                  │
│    listen_port: 9000                    │
│    ...                                  │
│                                         │
│  [Command input...]                     │
└─────────────────────────────────────────┘
```

**Network Diagnostics:**
- Latency graph
- Peer connection timeline
- Transport usage breakdown (Internet vs BLE vs WiFi)
- Message queue status

---

## [Needs Revalidation] Technical Integration Points

### [Needs Revalidation] Environment Detection
Your UI must auto-detect whether it's running locally or in Docker:

```javascript
// Auto-detect backend connection
function detectBackendUrl() {
  const isDocker = window.location.hostname !== 'localhost' &&
                   window.location.hostname !== '127.0.0.1';

  if (isDocker) {
    // Running in Docker, use container hostname
    return `ws://${window.location.hostname}:9000/ws`;
  } else {
    // Running locally, use localhost
    return 'ws://localhost:9000/ws';
  }
}

const WS_URL = detectBackendUrl();
const socket = new WebSocket(WS_URL);
```

### [Needs Revalidation] Backend Communication
Your UI needs to communicate with the SCMessenger Rust backend. Assume these interfaces:

**WebSocket Events** (from backend to UI):
```javascript
{
  type: "peer_discovered",
  peer_id: "12D3KooW...",
  transport: "internet" | "ble" | "wifi"
}

{
  type: "peer_disconnected",
  peer_id: "12D3KooW..."
}

{
  type: "message_received",
  from: "12D3KooW...",
  message_id: "abc123",
  content: "Hello!",
  timestamp: 1234567890
}

{
  type: "message_status",
  message_id: "abc123",
  status: "sent" | "delivered" | "read" | "failed"
}

{
  type: "network_status",
  status: "online" | "mesh" | "offline",
  peer_count: 12,
  relay_active: true
}
```

**Commands** (from UI to backend):
```javascript
// Send message
send_command({
  cmd: "send",
  recipient: "contact_name_or_peer_id",
  message: "Hello, world!"
})

// Add contact
send_command({
  cmd: "contact_add",
  peer_id: "12D3KooW...",
  public_key: "ed25519_hex...",
  name: "Alice"
})

// Get status
send_command({ cmd: "status" })

// Configuration
send_command({
  cmd: "config_set",
  key: "relay_enabled",
  value: "true"
})
```

### [Needs Revalidation] Data Structures

**Contact Object:**
```javascript
{
  peer_id: "12D3KooW...",
  public_key: "ed25519_hex_string",
  nickname: "Alice",
  display_name: "Alice",
  added_at: 1234567890,
  last_seen: 1234567890,
  status: "online" | "offline",
  message_count: 143
}
```

**Message Object:**
```javascript
{
  id: "msg_abc123",
  peer_id: "12D3KooW...",
  direction: "sent" | "received",
  content: "Message text",
  timestamp: 1234567890,
  status: "sent" | "delivered" | "read" | "failed",
  encrypted: true,
  relayed_via: ["peer1", "peer2"]  // Optional
}
```

---

## [Needs Revalidation] UX Principles

### [Needs Revalidation] 1. **Progressive Disclosure**
- Don't overwhelm users with technical details
- Hide advanced features behind "Advanced" toggles
- Show crypto concepts with friendly metaphors:
  - Identity = Your digital DNA
  - Public key = Your mailing address
  - Private key = Your house key
  - Relay = Being a good neighbor

### [Needs Revalidation] 2. **Zero Configuration Default**
- App should work out-of-the-box
- Sane defaults for everything
- Advanced users can tweak, beginners never need to

### [Needs Revalidation] 3. **Feedback Everywhere**
- Every action gets visual feedback
- Loading states for async operations
- Success/error messages that make sense
- Don't use crypto jargon in user-facing text

### [Needs Revalidation] 4. **Graceful Degradation**
- Works offline (queue messages)
- Works without internet (mesh only)
- Works without contacts (add contacts anytime)
- Works with slow connections (show progress)

### [Needs Revalidation] 5. **Privacy by Design**
- No telemetry, no analytics
- All data stored locally
- Clear indicators when data is being transmitted
- Option to verify encryption (advanced users)

---

## [Needs Revalidation] Example User Scenarios

### [Needs Revalidation] **Scenario 1: First-time user (Non-technical)**
```
1. Opens app
2. Sees: "Welcome to Sovereign Communication"
3. Clicks: "Get Started"
4. App generates identity (shows cool animation)
5. Sees: "You're ready! Share your identity to connect."
6. Clicks: "Share QR Code"
7. Friend scans QR code
8. Starts chatting
9. Never sees words like "Ed25519" or "libp2p"
```

### [Needs Revalidation] **Scenario 2: Power user (Technical)**
```
1. Opens app
2. Navigates to Settings → Network
3. Adds custom bootstrap nodes
4. Enables "Developer Console"
5. Monitors relay statistics
6. Runs self-tests
7. Exports identity backup
8. Verifies message encryption signatures
```

### [Needs Revalidation] **Scenario 3: Offline user (No internet)**
```
1. Opens app in area with no internet
2. Sees: "Mesh Mode" (blue indicator)
3. Discovers nearby peers via BLE
4. Sends message
5. Message relays through mesh
6. Receives delivery confirmation
7. Never notices internet was missing
```

---

## [Needs Revalidation] Deliverables

### [Needs Revalidation] **Phase 1: Core Interface**
- Single HTML file with inline CSS/JS
- Dashboard with messaging interface
- Contact management (add, list, view)
- Basic settings panel
- Network status indicator

### [Needs Revalidation] **Phase 2: Advanced Features**
- Network visualization
- Relay statistics dashboard
- Developer console
- QR code scanning
- Export/import functionality

### [Needs Revalidation] **Phase 3: Polish**
- Animations and transitions
- Dark/light theme toggle
- Keyboard shortcuts
- Accessibility (ARIA labels, screen reader support)
- Responsive design (mobile, tablet, desktop)

---

## [Needs Revalidation] Design Inspiration

Look at these for reference (but make it uniquely SCMessenger):
- **Signal** — Clean messaging, privacy-focused
- **Telegram** — Feature-rich, smooth animations
- **Discord** — Server/channel organization
- **Obsidian** — Dark theme, power user features
- **Linear** — Beautiful keyboard shortcuts, smooth UX
- **Raycast** — Command palette, fast interactions

**But remember:** This is not a chat app. This is *infrastructure*. Make it feel powerful yet approachable.

---

## [Needs Revalidation] Final Notes

**What makes this different:**
- NO servers (emphasize this in design)
- NO accounts (no login screen!)
- NO phone numbers/emails (identity = cryptographic keys)
- WORKS offline (make this obvious)
- YOU are the network (visualize relay activity)

**Success Criteria:**
- Grandma can send her first message in under 60 seconds
- Power users can access advanced features without friction
- The design makes encryption feel magical, not scary
- Users UNDERSTAND they're part of a mesh network (not just using one)
- The relay requirement feels like a feature, not a limitation

**Constraints:**
- One HTML file (CSS and JS inline)
- No external dependencies (vanilla JS, CSS)
- Must work offline once loaded
- Dark mode default, light mode optional
- Accessible (WCAG 2.1 AA minimum)

---

## [Needs Revalidation] Your Task

Build the **Phase 1: Core Interface** as a single HTML file. Include:
1. Dashboard layout (sidebar, message area, status bar)
2. Mock data for contacts and messages (hardcoded)
3. WebSocket interface (stubbed, shows how it would connect)
4. Basic interactions (click contact → show messages)
5. Settings panel (basic config)
6. Beautiful, modern design following the principles above

**Output:** One complete HTML file that can be opened in a browser and demonstrates the full interface with mock data. Include comments explaining where backend integration points are.

Make it beautiful. Make it intuitive. Make it *sovereign*.

Go build the future of communication. 🚀
