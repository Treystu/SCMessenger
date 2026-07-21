// Connection Ledger — Persistent peer discovery storage
//
// Philosophy: "A node is a node." IP is the source of truth.
//
// The ledger stores every successful IP:Port pair we've connected to.
// On startup, we load the ledger and attempt to reconnect to all known peers.
// If a peer presents a different PeerID (e.g., after restart), we accept it,
// update the ledger, and carry on. Unreachable peers enter exponential backoff
// but are never deleted — they may come back.

use anyhow::{Context, Result};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single entry in the connection ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// The IP:Port address (source of truth)
    pub address: String,

    /// The multiaddr we used to dial (without /p2p/ suffix)
    pub multiaddr: String,

    /// Last observed PeerID at this address (may change on restart)
    pub last_peer_id: Option<String>,

    /// All PeerIDs ever observed at this address
    pub observed_peer_ids: Vec<String>,

    /// Unix timestamp of last successful connection
    pub last_seen: u64,

    /// Unix timestamp of first discovery
    pub first_seen: u64,

    /// Number of consecutive failed connection attempts
    pub consecutive_failures: u32,

    /// Current backoff delay in seconds (doubles on each failure)
    pub backoff_seconds: u64,

    /// Unix timestamp of when we can next attempt connection
    pub next_attempt_after: u64,

    /// Whether this node has personally verified the address (successful
    /// local connection, or operator-trusted bootstrap). Defaults to false for
    /// entries loaded from disk that predate this field, so old peers.json
    /// entries classify as unknown until re-verified locally.
    #[serde(default)]
    pub locally_verified: bool,

    /// Whether this is a hardcoded bootstrap node (never remove)
    pub is_bootstrap: bool,

    /// Gossipsub topics this peer was subscribed to
    pub known_topics: Vec<String>,

    /// Human-readable label (e.g., "GCP Primary", "Community Relay")
    pub label: Option<String>,
}

impl LedgerEntry {
    /// Create a new entry for a discovered address
    pub fn new(multiaddr: String, is_bootstrap: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Extract IP:Port from multiaddr for the address field
        let address = extract_ip_port(&multiaddr).unwrap_or_else(|| multiaddr.clone());

        Self {
            address,
            multiaddr,
            last_peer_id: None,
            observed_peer_ids: Vec::new(),
            last_seen: now,
            first_seen: now,
            consecutive_failures: 0,
            backoff_seconds: 0,
            next_attempt_after: 0,
            locally_verified: false,
            is_bootstrap,
            known_topics: Vec::new(),
            label: None,
        }
    }

    /// Record a successful connection
    pub fn record_success(&mut self, peer_id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check if PeerID changed
        if let Some(ref old_id) = self.last_peer_id {
            if old_id != peer_id {
                tracing::warn!(
                    "[WARNING] PeerID changed at {}: {} -> {} (accepting new identity)",
                    self.address,
                    old_id,
                    peer_id
                );
            }
        }

        self.last_peer_id = Some(peer_id.to_string());

        // Track all observed PeerIDs
        if !self.observed_peer_ids.contains(&peer_id.to_string()) {
            self.observed_peer_ids.push(peer_id.to_string());
        }

        self.last_seen = now;
        self.consecutive_failures = 0;
        self.backoff_seconds = 0;
        self.next_attempt_after = 0;
        self.locally_verified = true;
    }

    /// Record a failed connection attempt with exponential backoff
    pub fn record_failure(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.consecutive_failures = self.consecutive_failures.saturating_add(1);

        // Exponential backoff: 5s, 10s, 20s, 40s, 80s, 160s, 300s (cap at 5 min).
        // Clamp exponent before shifting to avoid overflow under long-lived failure streaks.
        let exponent = self.consecutive_failures.saturating_sub(1).min(6);
        let uncapped_backoff = 5u64.saturating_mul(1u64 << exponent);
        self.backoff_seconds = std::cmp::min(uncapped_backoff, 300);

        self.next_attempt_after = now.saturating_add(self.backoff_seconds);
    }

    /// Check if we should attempt connection now
    pub fn should_attempt(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now >= self.next_attempt_after
    }

    /// Record a topic observed from this peer
    pub fn add_topic(&mut self, topic: &str) {
        if !self.known_topics.contains(&topic.to_string()) {
            self.known_topics.push(topic.to_string());
        }
    }
}

/// Key for per-peer dial state: PeerId when known, else the stripped
/// multiaddr (address-only dials must NEVER be dropped).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DialKey {
    Peer(PeerId),
    Addr(String),
}

impl DialKey {
    /// Build a key from a target multiaddr and optional known PeerId.
    pub fn for_target(multiaddr_str: &str, peer_id: Option<PeerId>) -> Self {
        if let Some(pid) = peer_id {
            return Self::Peer(pid);
        }

        if let Some(idx) = multiaddr_str.find("/p2p/") {
            let remainder = &multiaddr_str[idx + "/p2p/".len()..];
            if let Ok(pid) = PeerId::from_str(remainder) {
                return Self::Peer(pid);
            }
        }

        Self::Addr(strip_peer_id(multiaddr_str))
    }
}

/// Process-lifetime per-peer dial state (NOT serialized to peers.json).
#[derive(Debug, Clone, Default)]
pub struct PeerDialState {
    /// Consecutive dial failures this session (1st failure -> 5s delay).
    pub consecutive_failures: u32,

    /// Unix ts: next allowed dial attempt (0 = now).
    pub next_attempt_after: u64,

    /// A dial for this key is currently in flight.
    pub in_flight: bool,

    /// Has a successful connection history (seeded from ledger, set on success).
    pub is_known_good: bool,
}

impl PeerDialState {
    /// Backoff ladder in seconds: 5s, 30s, 2m, 5m, 30m.
    pub const BACKOFF_LADDER: [u64; 5] = [5, 30, 120, 300, 1800];

    /// Whether a new dial may be started now.
    pub fn ready(&self, now: u64) -> bool {
        now >= self.next_attempt_after && !self.in_flight
    }

    /// Reset state after a successful dial.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.next_attempt_after = 0;
        self.in_flight = false;
        self.is_known_good = true;
    }

    /// Back off after a failed dial.
    pub fn record_failure(&mut self, now: u64) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let idx = std::cmp::min(self.consecutive_failures.saturating_sub(1), 4) as usize;
        self.next_attempt_after = now.saturating_add(Self::BACKOFF_LADDER[idx]);
        self.in_flight = false;
    }
}

/// The Connection Ledger — persistent storage for all known peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLedger {
    /// All known peer entries, keyed by multiaddr (without /p2p/ suffix)
    pub entries: HashMap<String, LedgerEntry>,

    /// Version for future migrations
    pub version: u32,

    /// Last save timestamp
    pub last_saved: u64,

    /// Process-lifetime per-peer dial state. Never persisted to peers.json.
    #[serde(skip)]
    pub peer_dial_states: HashMap<DialKey, PeerDialState>,
}

impl Default for ConnectionLedger {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
            last_saved: 0,
            peer_dial_states: HashMap::new(),
        }
    }
}

impl ConnectionLedger {
    /// Load the ledger from disk, or create a new one
    pub fn load(data_dir: &Path) -> Result<Self> {
        let ledger_path = data_dir.join("peers.json");

        if ledger_path.exists() {
            let contents =
                std::fs::read_to_string(&ledger_path).context("Failed to read peers.json")?;
            let ledger: ConnectionLedger =
                serde_json::from_str(&contents).context("Failed to parse peers.json")?;
            tracing::info!(
                "[INFO] Loaded connection ledger: {} known peers",
                ledger.entries.len()
            );
            Ok(ledger)
        } else {
            tracing::info!("[INFO] No existing ledger found, starting fresh");
            Ok(Self::default())
        }
    }

    /// Save the ledger to disk
    pub fn save(&mut self, data_dir: &Path) -> Result<()> {
        let ledger_path = data_dir.join("peers.json");

        self.last_saved = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let contents = serde_json::to_string_pretty(self).context("Failed to serialize ledger")?;
        std::fs::write(&ledger_path, contents).context("Failed to write peers.json")?;

        tracing::debug!("[INFO] Saved ledger ({} entries)", self.entries.len());
        Ok(())
    }

    /// Add or update a peer entry from a bootstrap multiaddr
    pub fn add_bootstrap(&mut self, multiaddr: &str, local_peer_id: Option<&str>) {
        if let Some(local) = local_peer_id {
            if multiaddr.contains(local) {
                return;
            }
        }
        let stripped = strip_peer_id(multiaddr);
        let label = format!("Bootstrap {}", self.entries.len() + 1);

        self.entries
            .entry(stripped.clone())
            .and_modify(|e| {
                e.is_bootstrap = true;
                e.locally_verified = true;
            })
            .or_insert_with(|| {
                let mut entry = LedgerEntry::new(stripped.clone(), true);
                entry.label = Some(label);
                entry.locally_verified = true;
                entry
            });
    }

    /// Add or update a peer after successful connection
    pub fn record_connection(&mut self, multiaddr: &str, peer_id: &str) {
        let stripped = strip_peer_id(multiaddr);
        if !is_dialable_multiaddr(&stripped, NetworkMode::Local) {
            return;
        }

        self.entries
            .entry(stripped.clone())
            .and_modify(|e| {
                e.record_success(peer_id);
                e.locally_verified = true;
            })
            .or_insert_with(|| {
                let mut entry = LedgerEntry::new(stripped.clone(), false);
                entry.record_success(peer_id);
                entry.locally_verified = true;
                entry
            });
    }

    /// Record a topic observed from a peer
    pub fn record_topic(&mut self, multiaddr: &str, topic: &str) {
        let stripped = strip_peer_id(multiaddr);
        if let Some(entry) = self.entries.get_mut(&stripped) {
            entry.add_topic(topic);
        }
    }

    /// Record a failed connection attempt
    pub fn record_failure(&mut self, multiaddr: &str) {
        let stripped = strip_peer_id(multiaddr);
        if let Some(entry) = self.entries.get_mut(&stripped) {
            entry.record_failure();
            tracing::warn!(
                "[WARNING] Connection failed to {} (attempt #{}, backoff {}s)",
                stripped,
                entry.consecutive_failures,
                entry.backoff_seconds
            );
        }
    }

    /// Get all addresses that should be dialed now, excluding the local node
    pub fn dialable_addresses(&self, local_peer_id: Option<&str>) -> Vec<(String, Option<String>)> {
        self.entries
            .values()
            .filter(|e| e.should_attempt())
            .filter(|e| is_dialable_multiaddr(&e.multiaddr, NetworkMode::Local))
            .filter(|e| {
                if let (Some(local), Some(last)) = (local_peer_id, &e.last_peer_id) {
                    local != last
                } else {
                    true
                }
            })
            .map(|e| (e.multiaddr.clone(), e.last_peer_id.clone()))
            .collect()
    }

    /// Get all known topics from connected peers
    pub fn all_known_topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = self
            .entries
            .values()
            .flat_map(|e| e.known_topics.clone())
            .collect();
        topics.sort();
        topics.dedup();
        topics
    }

    /// Find entry by PeerID (lookup across all entries)
    pub fn find_by_peer_id(&self, peer_id: &str) -> Option<&LedgerEntry> {
        self.entries.values().find(|e| {
            e.last_peer_id.as_deref() == Some(peer_id)
                || e.observed_peer_ids.contains(&peer_id.to_string())
        })
    }

    /// Convert ledger entries to wire-format for sharing with peers.
    ///
    /// Only shares peers seen in the last 7 days — no point advertising
    /// stale addresses. Private backoff data is stripped.
    pub fn to_shared_entries(&self) -> Vec<scmessenger_core::transport::SharedPeerEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let seven_days_ago = now.saturating_sub(7 * 24 * 3600);

        self.entries
            .values()
            .filter(|e| e.last_seen >= seven_days_ago || e.is_bootstrap)
            .map(|e| scmessenger_core::transport::SharedPeerEntry {
                multiaddr: e.multiaddr.clone(),
                last_peer_id: e.last_peer_id.clone(),
                last_seen: e.last_seen,
                known_topics: e.known_topics.clone(),
            })
            .collect()
    }

    /// Merge peer entries received from a remote peer.
    ///
    /// New addresses are added with is_bootstrap=false.
    /// Existing addresses get their last_seen updated if the remote has
    /// a more recent timestamp. Returns the number of new peers learned.
    pub fn merge_shared_entries(
        &mut self,
        entries: &[scmessenger_core::transport::SharedPeerEntry],
    ) -> usize {
        let mut new_count = 0;

        for entry in entries {
            let stripped = strip_peer_id(&entry.multiaddr);

            if !is_dialable_multiaddr(&stripped, NetworkMode::Local) {
                continue;
            }

            if let Some(existing) = self.entries.get_mut(&stripped) {
                // Update last_seen if the remote has fresher data
                if entry.last_seen > existing.last_seen {
                    existing.last_seen = entry.last_seen;
                }
                // Update PeerID if we didn't have one
                if existing.last_peer_id.is_none() {
                    existing.last_peer_id = entry.last_peer_id.clone();
                }
                // Merge topics
                for topic in &entry.known_topics {
                    existing.add_topic(topic);
                }
            } else {
                // Brand new peer — add it
                let mut new_entry = LedgerEntry::new(stripped.clone(), false);
                new_entry.last_peer_id = entry.last_peer_id.clone();
                new_entry.last_seen = entry.last_seen;
                new_entry.known_topics = entry.known_topics.clone();
                new_entry.label = Some("Discovered via peer".to_string());

                // Track the PeerID in observed list
                if let Some(ref pid) = entry.last_peer_id {
                    if !new_entry.observed_peer_ids.contains(pid) {
                        new_entry.observed_peer_ids.push(pid.clone());
                    }
                }

                self.entries.insert(stripped, new_entry);
                new_count += 1;
            }
        }

        if new_count > 0 {
            tracing::info!(
                "[INFO] Merged {} new peers from ledger exchange (total: {})",
                new_count,
                self.entries.len()
            );
        }

        new_count
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        let total = self.entries.len();
        let bootstrap = self.entries.values().filter(|e| e.is_bootstrap).count();
        let reachable = self
            .entries
            .values()
            .filter(|e| e.consecutive_failures == 0)
            .count();
        let backoff = self
            .entries
            .values()
            .filter(|e| e.consecutive_failures > 0)
            .count();

        format!(
            "Ledger: {} peers ({} bootstrap, {} reachable, {} in backoff)",
            total, bootstrap, reachable, backoff
        )
    }

    /// Decide whether a dial may be started for `key` right now.
    ///
    /// Returns true only when the key is ready and the dial is not suppressed
    /// by a healthy relay path. When the key is new, it is seeded from the
    /// persistent ledger so known-good peers are never suppressed.
    pub fn try_begin_dial(&mut self, key: DialKey, now: u64, relay_healthy: bool) -> bool {
        let is_circuit = Self::is_circuit_key(&key);
        let is_bootstrap = self.is_bootstrap_key(&key);

        if let Some(state) = self.peer_dial_states.get(&key) {
            if !state.ready(now) {
                return false;
            }
            if relay_healthy && !state.is_known_good && !is_circuit && !is_bootstrap {
                return false;
            }
        } else {
            let is_known_good = self.is_known_good_key(&key);
            if relay_healthy && !is_known_good && !is_circuit && !is_bootstrap {
                return false;
            }

            // Cap process-lifetime dial state at 4096 keys. Drop the entry
            // with the smallest next_attempt_after (least urgent) in a single
            // pass.
            if self.peer_dial_states.len() >= 4096 {
                if let Some(evict_key) = self
                    .peer_dial_states
                    .iter()
                    .min_by_key(|(_, state)| state.next_attempt_after)
                    .map(|(k, _)| k.clone())
                {
                    self.peer_dial_states.remove(&evict_key);
                }
            }

            let state = PeerDialState {
                is_known_good,
                ..Default::default()
            };
            self.peer_dial_states.insert(key.clone(), state);
        }

        self.peer_dial_states
            .get_mut(&key)
            .expect("key was just inserted or already present")
            .in_flight = true;
        true
    }

    /// Record the outcome of a dial previously started with `try_begin_dial`.
    pub fn complete_dial(
        &mut self,
        key: &DialKey,
        success: bool,
        now: u64,
        learned_peer_id: Option<PeerId>,
    ) {
        if success {
            let mut state = self
                .peer_dial_states
                .remove(key)
                .unwrap_or_default();
            state.record_success();

            if let DialKey::Addr(_) = key {
                if let Some(pid) = learned_peer_id {
                    let peer_key = DialKey::Peer(pid);
                    self.peer_dial_states.entry(peer_key).or_insert(state);
                    return;
                }
            }

            self.peer_dial_states.insert(key.clone(), state);
        } else if let Some(state) = self.peer_dial_states.get_mut(key) {
            state.record_failure(now);
        }
    }

    /// Borrow a tracked dial state, if any.
    pub fn dial_state(&self, key: &DialKey) -> Option<&PeerDialState> {
        self.peer_dial_states.get(key)
    }

    fn is_circuit_key(key: &DialKey) -> bool {
        matches!(key, DialKey::Addr(addr) if addr.contains("/p2p-circuit"))
    }

    fn is_bootstrap_key(&self, key: &DialKey) -> bool {
        match key {
            DialKey::Peer(pid) => self
                .find_by_peer_id(&pid.to_string())
                .map(|e| e.is_bootstrap)
                .unwrap_or(false),
            DialKey::Addr(addr) => self
                .entries
                .get(addr)
                .map(|e| e.is_bootstrap)
                .unwrap_or(false),
        }
    }

    fn is_known_good_key(&self, key: &DialKey) -> bool {
        match key {
            DialKey::Peer(pid) => self
                .find_by_peer_id(&pid.to_string())
                .is_some_and(|e| e.locally_verified && e.last_peer_id.is_some() && e.consecutive_failures == 0),
            DialKey::Addr(addr) => self
                .entries
                .get(addr)
                .is_some_and(|e| e.locally_verified && e.last_peer_id.is_some() && e.consecutive_failures == 0),
        }
    }
}

/// Network context for address filtering. `Local` (WiFi/LAN) keeps private/LAN
/// ranges dialable for local mesh discovery; `Public` (cellular / public-only)
/// additionally drops private ranges since a public-only node cannot reach
/// anyone's LAN. Defaults to the conservative `Local`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NetworkMode {
    #[default]
    Local,
    Public,
}

/// Returns true iff `multiaddr` is worth dialing. Always rejects non-routable
/// addresses that a remote node can never reach: loopback, unspecified, IPv4
/// link-local (169.254/16), IPv6 link-local (fe80::/10) and IPv6 site-local
/// (fec0::/10). `/p2p-circuit` addresses are always allowed (the only way to
/// reach a relayed peer). Private/LAN IPv4 ranges are rejected only when
/// `mode == NetworkMode::Public`.
pub fn is_dialable_multiaddr(multiaddr: &str, mode: NetworkMode) -> bool {
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut i = 0;
    while i + 1 < parts.len() {
        match parts[i] {
            "p2p-circuit" => return true,
            "ip4" => {
                if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv4Addr>() {
                    if ip.is_loopback() || ip.is_unspecified() || ip.is_link_local() {
                        return false;
                    }
                    if mode == NetworkMode::Public && ip.is_private() {
                        return false;
                    }
                }
            }
            "ip6" => {
                if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv6Addr>() {
                    if ip.is_loopback() || ip.is_unspecified() {
                        return false;
                    }
                    // fe80::/10 link-local and fec0::/10 site-local: check the
                    // top 10 bits of the first 16-bit segment (std lacks stable
                    // helpers for these on this toolchain).
                    let top10 = ip.segments()[0] & 0xffc0;
                    if top10 == 0xfe80 || top10 == 0xfec0 {
                        return false;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    true
}

/// Extract the first `/ip4/x.x.x.x/` component of a multiaddr, if any.
fn extract_ipv4(multiaddr: &str) -> Option<std::net::Ipv4Addr> {
    let parts: Vec<&str> = multiaddr.split('/').collect();
    for i in 0..parts.len() {
        if parts[i] == "ip4" && i + 1 < parts.len() {
            if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv4Addr>() {
                return Some(ip);
            }
        }
    }
    None
}

/// Which RFC1918 private-address class an IPv4 address falls in, if any.
/// `None` means the address is not a private (RFC1918) address at all.
fn rfc1918_class(ip: &std::net::Ipv4Addr) -> Option<u8> {
    let o = ip.octets();
    if o[0] == 10 {
        Some(0) // 10.0.0.0/8
    } else if o[0] == 172 && (16..=31).contains(&o[1]) {
        Some(1) // 172.16.0.0/12
    } else if o[0] == 192 && o[1] == 168 {
        Some(2) // 192.168.0.0/16
    } else {
        None
    }
}

/// Returns true iff `candidate` is one of this node's own known addresses
/// (listen or external) -- i.e. dialing it would be a self-dial. Compares
/// the transport address only (strips any `/p2p/` peer-id suffix on both
/// sides), since the same node can be observed with or without its own
/// peer-id attached depending on which ledger entry produced it.
pub fn is_self_address(candidate: &str, my_addrs: &[String]) -> bool {
    let stripped_candidate = strip_peer_id(candidate);
    my_addrs
        .iter()
        .any(|a| strip_peer_id(a) == stripped_candidate)
}

/// Returns true iff `candidate` is worth dialing given this node's own known
/// addresses: rejects self-dials outright, and (in `NetworkMode::Local`)
/// rejects a private-range (RFC1918) address unless this node itself holds
/// an address in the SAME private-range class -- e.g. a node on
/// `192.168.0.121` should not promiscuously dial an advertised
/// `10.0.2.16` (a different private class it has no route to), but should
/// still dial other `192.168.x.x` peers on its own LAN. This does not
/// replace `is_dialable_multiaddr` -- callers should still apply that
/// filter first (it rejects unconditionally-unroutable things like
/// loopback/link-local); this is an additional, node-aware layer on top.
pub fn is_dialable_for_this_node(multiaddr: &str, mode: NetworkMode, my_addrs: &[String]) -> bool {
    if is_self_address(multiaddr, my_addrs) {
        return false;
    }
    // A /p2p-circuit address's leading /ip4/.../ component is the RELAY
    // hop's address, not the final target peer's -- applying RFC1918
    // class-awareness to the relay's own address would incorrectly reject
    // the only path to a NAT'd peer whenever the relay's IP happens to
    // differ in private-range class from this node's own address. Mirrors
    // the same unconditional-allow exemption is_dialable_multiaddr already
    // gives circuit addresses.
    if multiaddr.contains("/p2p-circuit") {
        return true;
    }
    if mode == NetworkMode::Local {
        if let Some(candidate_ip) = extract_ipv4(multiaddr) {
            if let Some(candidate_class) = rfc1918_class(&candidate_ip) {
                let my_ipv4s: Vec<std::net::Ipv4Addr> =
                    my_addrs.iter().filter_map(|a| extract_ipv4(a)).collect();
                let on_same_range = my_ipv4s
                    .iter()
                    .any(|m| rfc1918_class(m) == Some(candidate_class));
                if !on_same_range {
                    return false;
                }
            }
        }
    }
    true
}

/// Strip the /p2p/PeerID suffix from a multiaddr string, leaving just the transport address.
/// This is the core of "promiscuous" dialing — we dial the IP, not the identity.
pub fn strip_peer_id(multiaddr: &str) -> String {
    if let Some(idx) = multiaddr.find("/p2p/") {
        multiaddr[..idx].to_string()
    } else {
        multiaddr.to_string()
    }
}

/// Extract IP:Port from a multiaddr string for human-readable display
pub fn extract_ip_port(multiaddr: &str) -> Option<String> {
    // Parse /ip4/1.2.3.4/tcp/9001 -> 1.2.3.4:9001
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut ip = None;
    let mut port = None;

    for i in 0..parts.len() {
        if (parts[i] == "ip4" || parts[i] == "ip6") && i + 1 < parts.len() {
            ip = Some(parts[i + 1]);
        }
        if (parts[i] == "tcp" || parts[i] == "udp") && i + 1 < parts.len() {
            port = Some(parts[i + 1]);
        }
    }

    match (ip, port) {
        (Some(ip), Some(port)) => Some(format!("{}:{}", ip, port)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_peer_id() {
        assert_eq!(
            strip_peer_id("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWGGdvGNJb3Jw"),
            "/ip4/1.2.3.4/tcp/9001"
        );
        assert_eq!(
            strip_peer_id("/ip4/1.2.3.4/tcp/9001"),
            "/ip4/1.2.3.4/tcp/9001"
        );
    }

    #[test]
    fn test_extract_ip_port() {
        assert_eq!(
            extract_ip_port("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW"),
            Some("1.2.3.4:9001".to_string())
        );
        assert_eq!(
            extract_ip_port("/ip4/10.0.0.1/tcp/4001"),
            Some("10.0.0.1:4001".to_string())
        );
    }

    #[test]
    fn test_ledger_entry_backoff() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), false);
        assert!(entry.should_attempt());

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 1);
        assert_eq!(entry.backoff_seconds, 5);

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 2);
        assert_eq!(entry.backoff_seconds, 10);

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 3);
        assert_eq!(entry.backoff_seconds, 20);

        // Success resets everything
        entry.record_success("12D3KooWTest");
        assert_eq!(entry.consecutive_failures, 0);
        assert_eq!(entry.backoff_seconds, 0);
        assert_eq!(entry.last_peer_id, Some("12D3KooWTest".to_string()));
    }

    #[test]
    fn test_ledger_entry_backoff_overflow_safety() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), false);
        entry.consecutive_failures = u32::MAX;

        entry.record_failure();

        assert_eq!(entry.consecutive_failures, u32::MAX);
        assert_eq!(entry.backoff_seconds, 300);
        assert!(entry.next_attempt_after >= entry.backoff_seconds);
    }

    #[test]
    fn test_ledger_entry_peer_id_tracking() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), true);

        entry.record_success("PeerA");
        assert_eq!(entry.last_peer_id, Some("PeerA".to_string()));
        assert_eq!(entry.observed_peer_ids, vec!["PeerA".to_string()]);

        // Peer restarts with new key
        entry.record_success("PeerB");
        assert_eq!(entry.last_peer_id, Some("PeerB".to_string()));
        assert_eq!(
            entry.observed_peer_ids,
            vec!["PeerA".to_string(), "PeerB".to_string()]
        );
    }

    #[test]
    fn test_ledger_crud() {
        let mut ledger = ConnectionLedger::default();

        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW", None);
        assert_eq!(ledger.entries.len(), 1);

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.is_bootstrap);

        ledger.record_connection("/ip4/1.2.3.4/tcp/9001", "NewPeerId");
        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert_eq!(entry.last_peer_id, Some("NewPeerId".to_string()));
    }

    #[test]
    fn test_ledger_topic_tracking() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001", None);
        ledger.record_topic("/ip4/1.2.3.4/tcp/9001", "sc-mesh");
        ledger.record_topic("/ip4/1.2.3.4/tcp/9001", "sc-lobby");

        let topics = ledger.all_known_topics();
        assert!(topics.contains(&"sc-mesh".to_string()));
        assert!(topics.contains(&"sc-lobby".to_string()));
    }

    #[test]
    fn test_is_dialable_multiaddr() {
        use NetworkMode::{Local, Public};
        // Non-routable: rejected regardless of mode.
        assert!(!is_dialable_multiaddr("/ip4/127.0.0.1/tcp/9001", Local));
        assert!(!is_dialable_multiaddr("/ip4/0.0.0.0/tcp/9001", Local));
        assert!(!is_dialable_multiaddr("/ip4/169.254.1.2/tcp/9001", Local));
        assert!(!is_dialable_multiaddr("/ip6/::1/tcp/9001", Local));
        assert!(!is_dialable_multiaddr(
            "/ip6/fe80::1897:a8ff:fec5:3d16/tcp/443",
            Local
        ));
        assert!(!is_dialable_multiaddr("/ip6/fec0::1/tcp/9001", Local));
        // Globally routable: accepted.
        assert!(is_dialable_multiaddr("/ip4/1.2.3.4/tcp/9001", Local));
        assert!(is_dialable_multiaddr(
            "/ip6/2606:4700:4700::1111/tcp/9001",
            Local
        ));
        // Private/LAN: kept in Local, dropped in Public.
        assert!(is_dialable_multiaddr("/ip4/10.0.2.16/tcp/9001", Local));
        assert!(is_dialable_multiaddr("/ip4/192.168.1.5/tcp/9001", Local));
        assert!(!is_dialable_multiaddr("/ip4/10.0.2.16/tcp/9001", Public));
        assert!(!is_dialable_multiaddr("/ip4/192.168.1.5/tcp/9001", Public));
        // p2p-circuit always allowed (relay path).
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/9001/p2p-circuit",
            Local
        ));
    }

    #[test]
    fn test_is_self_address() {
        let my_addrs = vec![
            "/ip4/192.168.0.121/tcp/9001".to_string(),
            "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWExample".to_string(),
        ];
        // Exact match (own LAN address) -> self-dial.
        assert!(is_self_address("/ip4/192.168.0.121/tcp/9001", &my_addrs));
        // Own address with a peer-id suffix attached still matches after stripping.
        assert!(is_self_address(
            "/ip4/192.168.0.121/tcp/9001/p2p/12D3KooWOther",
            &my_addrs
        ));
        // Own public address matches regardless of which side carries the peer-id.
        assert!(is_self_address("/ip4/1.2.3.4/tcp/9001", &my_addrs));
        // A different address is not a self-dial.
        assert!(!is_self_address("/ip4/10.0.2.16/tcp/9001", &my_addrs));
    }

    #[test]
    fn test_is_dialable_for_this_node() {
        use NetworkMode::Local;
        // Node is on a 192.168.x.x home LAN.
        let my_addrs = vec!["/ip4/192.168.0.121/tcp/9001".to_string()];

        // Self-dial rejected even though it would otherwise be dialable.
        assert!(!is_dialable_for_this_node(
            "/ip4/192.168.0.121/tcp/9001",
            Local,
            &my_addrs
        ));
        // Another peer on the SAME private range (192.168.x.x) is fine.
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.0.55/tcp/9001",
            Local,
            &my_addrs
        ));
        // A DIFFERENT private range (10.x.x.x, e.g. an emulator's internal
        // address) is not reachable from a 192.168.x.x-only node.
        assert!(!is_dialable_for_this_node(
            "/ip4/10.0.2.16/tcp/9001",
            Local,
            &my_addrs
        ));
        // Globally routable addresses are unaffected by range-awareness.
        assert!(is_dialable_for_this_node(
            "/ip4/1.2.3.4/tcp/9001",
            Local,
            &my_addrs
        ));

        // A node with no private addresses of its own (e.g. cellular-only)
        // should not dial ANY private-range address.
        let public_only: Vec<String> = vec!["/ip4/1.2.3.4/tcp/9001".to_string()];
        assert!(!is_dialable_for_this_node(
            "/ip4/192.168.1.5/tcp/9001",
            Local,
            &public_only
        ));

        // Dual-homed node (has addresses in TWO different private classes):
        // both classes should be dialable, not just the first one found.
        let dual_homed = vec![
            "/ip4/192.168.0.121/tcp/9001".to_string(),
            "/ip4/10.5.5.5/tcp/9001".to_string(),
        ];
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.1.5/tcp/9001",
            Local,
            &dual_homed
        ));
        assert!(is_dialable_for_this_node(
            "/ip4/10.9.9.9/tcp/9001",
            Local,
            &dual_homed
        ));
        // Still not the third RFC1918 class (172.16.0.0/12).
        assert!(!is_dialable_for_this_node(
            "/ip4/172.16.0.5/tcp/9001",
            Local,
            &dual_homed
        ));

        // A relay-circuit address's leading /ip4/.../ is the RELAY hop, not
        // the final target -- it must NOT be subject to RFC1918
        // class-matching against that hop's own address, or the only path
        // to a NAT'd peer behind a cross-class relay would be silently
        // dropped. Regression test for the exact shape used by this
        // project's own test fixtures (core/src/transport/swarm.rs).
        let my_addrs = vec!["/ip4/192.168.0.121/tcp/9001".to_string()];
        assert!(is_dialable_for_this_node(
            "/ip4/172.26.144.1/tcp/9101/p2p/12D3KooWRelay/p2p-circuit/p2p/12D3KooWTarget",
            Local,
            &my_addrs
        ));
        // A circuit address whose relay hop happens to share this node's IP
        // is NOT treated as a self-dial: is_self_address does an exact
        // string match after stripping at the first "/p2p/", and the
        // "/p2p-circuit" suffix makes that stripped string differ from a
        // plain "/ip4/.../tcp/9001" self-address, so this is correctly
        // treated as "unconditionally allowed circuit address", not "self".
        // (Genuinely self-targeted circuit dials are a degenerate case the
        // ledger shouldn't produce in practice; libp2p itself also rejects
        // dialing one's own PeerId at the connection layer as a backstop.)
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.0.121/tcp/9001/p2p-circuit/p2p/12D3KooWTarget",
            Local,
            &my_addrs
        ));
    }

    #[test]
    fn test_dial_key_for_target() {
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_id_str = peer_id.to_string();

        // Explicit peer id wins.
        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", Some(peer_id));
        assert_eq!(key, DialKey::Peer(peer_id));

        // Parsed from /p2p/ suffix.
        let addr_with_p2p = format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", peer_id_str);
        let key = DialKey::for_target(&addr_with_p2p, None);
        assert_eq!(key, DialKey::Peer(peer_id));

        // Address-only falls back to stripped multiaddr.
        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", None);
        assert_eq!(key, DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string()));
    }

    #[test]
    fn test_peer_dial_state_backoff_ladder() {
        let mut state = PeerDialState::default();
        let now = 1_000_000;
        let expected = [5, 30, 120, 300, 1800, 1800];

        for (i, &delay) in expected.iter().enumerate() {
            state.record_failure(now);
            assert_eq!(state.consecutive_failures, (i + 1) as u32);
            assert_eq!(state.next_attempt_after.saturating_sub(now), delay);
            assert!(!state.ready(now + delay - 1));
            assert!(state.ready(now + delay));
        }
    }

    #[test]
    fn test_peer_dial_state_success_reset() {
        let mut state = PeerDialState::default();
        let now = 1_000_000;

        state.record_failure(now);
        state.record_failure(now);
        assert!(!state.ready(now));

        state.record_success();
        assert!(state.ready(now));
        assert_eq!(state.consecutive_failures, 0);
        assert_eq!(state.next_attempt_after, 0);
        assert!(state.is_known_good);
    }

    #[test]
    fn test_try_begin_dial_blocks_in_flight_reuse() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key.clone(), 0, false));
        assert!(!ledger.try_begin_dial(key.clone(), 0, false));
    }

    #[test]
    fn test_try_begin_dial_suppresses_unknown_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(!ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_circuit_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001/p2p-circuit".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_bootstrap_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001", None);
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_known_good_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        ledger.record_connection("/ip4/1.2.3.4/tcp/9001", "12D3KooWTestPeerId");
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_complete_dial_failure_enforces_backoff() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key.clone(), 0, false));
        ledger.complete_dial(&key, false, 0, None);

        let state = ledger.dial_state(&key).unwrap();
        assert!(!state.ready(4));
        assert!(state.ready(5));
    }

    #[test]
    fn test_complete_dial_migrates_addr_to_peer() {
        let mut ledger = ConnectionLedger::default();
        let addr_key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        assert!(ledger.try_begin_dial(addr_key.clone(), 0, false));
        ledger.complete_dial(&addr_key, true, 0, Some(peer_id));

        assert!(ledger.dial_state(&addr_key).is_none());
        let peer_key = DialKey::Peer(peer_id);
        let state = ledger.dial_state(&peer_key).unwrap();
        assert!(state.is_known_good);
    }

    #[test]
    fn test_peer_dial_states_eviction_caps_at_4096() {
        let mut ledger = ConnectionLedger::default();

        for i in 0..4096u64 {
            let key = DialKey::Addr(format!("/ip4/1.2.3.4/tcp/{}", i));
            assert!(ledger.try_begin_dial(key.clone(), i, false));
            ledger.complete_dial(&key, false, i, None);
        }
        assert_eq!(ledger.peer_dial_states.len(), 4096);

        let new_key = DialKey::Addr("/ip4/9.9.9.9/tcp/9999".to_string());
        assert!(ledger.try_begin_dial(new_key.clone(), 5000, false));
        assert_eq!(ledger.peer_dial_states.len(), 4096);
        assert!(ledger.peer_dial_states.contains_key(&new_key));

        let evicted_key = DialKey::Addr("/ip4/1.2.3.4/tcp/0".to_string());
        assert!(!ledger.peer_dial_states.contains_key(&evicted_key));
    }

    #[test]
    fn test_shared_entry_does_not_seed_known_good_until_locally_verified() {
        let mut ledger = ConnectionLedger::default();
        let shared = scmessenger_core::transport::SharedPeerEntry {
            multiaddr: "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWSpoof".to_string(),
            last_peer_id: Some("12D3KooWSpoof".to_string()),
            last_seen: 1_700_000_000,
            known_topics: vec![],
        };
        ledger.merge_shared_entries(&[shared]);

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.last_peer_id.is_some());
        assert_eq!(entry.consecutive_failures, 0);
        assert!(!entry.locally_verified);

        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        assert!(!ledger.try_begin_dial(key.clone(), 0, true));

        ledger.record_connection("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWSpoof", "12D3KooWSpoof");
        assert!(ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap().locally_verified);
        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_add_bootstrap_seeds_known_good() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWBootstrap", None);

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.locally_verified);
        assert!(entry.is_bootstrap);

        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_locally_verified_defaults_false_on_deserialize() {
        let json = r#"{
            "address": "1.2.3.4:9001",
            "multiaddr": "/ip4/1.2.3.4/tcp/9001",
            "last_peer_id": "12D3KooWTest",
            "observed_peer_ids": [],
            "last_seen": 1700000000,
            "first_seen": 1700000000,
            "consecutive_failures": 0,
            "backoff_seconds": 0,
            "next_attempt_after": 0,
            "is_bootstrap": false,
            "known_topics": [],
            "label": null
        }"#;
        let entry: LedgerEntry = serde_json::from_str(json).unwrap();
        assert!(!entry.locally_verified);
    }
}
