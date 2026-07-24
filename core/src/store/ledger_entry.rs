use libp2p::Multiaddr;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// CONNECTION LEDGER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub multiaddr: String,
    pub peer_id: Option<String>,
    pub public_key: Option<String>,
    pub nickname: Option<String>,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_seen: Option<u64>,
    pub topics: Vec<String>,
}

fn is_dns_multiaddr(addr_str: &str) -> bool {
    addr_str.contains("/dns/")
        || addr_str.contains("/dns4/")
        || addr_str.contains("/dns6/")
        || addr_str.contains("/dnsaddr/")
}

fn get_multiaddr_port(addr_str: &str) -> Option<u16> {
    if let Ok(addr) = addr_str.parse::<Multiaddr>() {
        for proto in addr.iter() {
            match proto {
                libp2p::multiaddr::Protocol::Tcp(port) => return Some(port),
                libp2p::multiaddr::Protocol::Udp(port) => return Some(port),
                _ => {}
            }
        }
    }
    None
}

#[derive(uniffi::Object)]
pub struct LedgerManager {
    storage_path: std::path::PathBuf,
    entries: Arc<Mutex<Vec<LedgerEntry>>>,
}

#[uniffi::export]
impl LedgerManager {
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path: std::path::PathBuf::from(storage_path),
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn load(&self) -> Result<(), crate::IronCoreError> {
        let ledger_file = self.storage_path.join("ledger.json");
        if ledger_file.exists() {
            let data = std::fs::read_to_string(&ledger_file)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            let entries: Vec<LedgerEntry> =
                serde_json::from_str(&data).map_err(|_| crate::IronCoreError::Internal)?;
            *self.entries.lock() = entries;
        }
        Ok(())
    }

    fn save_with_entries(&self, entries: &[LedgerEntry]) -> Result<(), crate::IronCoreError> {
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|_| crate::IronCoreError::StorageError)?;

        let ledger_file = self.storage_path.join("ledger.json");
        let data =
            serde_json::to_string_pretty(entries).map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&ledger_file, data).map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), crate::IronCoreError> {
        let entries = self.entries.lock();
        self.save_with_entries(&entries)
    }

    pub fn record_connection(&self, multiaddr: String, peer_id: String) {
        let mut entries = self.entries.lock();
        let target_port = get_multiaddr_port(&multiaddr);
        let mut found_dns_idx = None;
        for (idx, entry) in entries.iter().enumerate() {
            if entry.peer_id.as_deref() == Some(&peer_id)
                && is_dns_multiaddr(&entry.multiaddr)
                && (target_port.is_none() || get_multiaddr_port(&entry.multiaddr) == target_port)
            {
                found_dns_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = found_dns_idx {
            let entry = &mut entries[idx];
            entry.success_count += 1;
            entry.last_seen = Some(current_timestamp());
        } else if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.success_count += 1;
            entry.peer_id = Some(peer_id);
            entry.last_seen = Some(current_timestamp());
        } else {
            entries.push(LedgerEntry {
                multiaddr,
                peer_id: Some(peer_id),
                public_key: None,
                nickname: None,
                success_count: 1,
                failure_count: 0,
                last_seen: Some(current_timestamp()),
                topics: Vec::new(),
            });
        }
        let _ = self.save_with_entries(&entries);
    }

    pub fn record_failure(&self, multiaddr: String) {
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.failure_count += 1;
        }
        let _ = self.save_with_entries(&entries);
    }

    pub fn annotate_identity(
        &self,
        multiaddr: String,
        peer_id: String,
        public_key: Option<String>,
        nickname: Option<String>,
    ) {
        let normalized_public_key = public_key.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let normalized_nickname = nickname.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let mut entries = self.entries.lock();
        let target_port = get_multiaddr_port(&multiaddr);
        let mut found_dns_idx = None;
        for (idx, entry) in entries.iter().enumerate() {
            if entry.peer_id.as_deref() == Some(&peer_id)
                && is_dns_multiaddr(&entry.multiaddr)
                && (target_port.is_none() || get_multiaddr_port(&entry.multiaddr) == target_port)
            {
                found_dns_idx = Some(idx);
                break;
            }
        }

        let is_new = if let Some(idx) = found_dns_idx {
            let entry = &mut entries[idx];
            if normalized_public_key.is_some() {
                entry.public_key = normalized_public_key;
            }
            if normalized_nickname.is_some() {
                entry.nickname = normalized_nickname;
            }
            entry.last_seen = Some(current_timestamp());
            false
        } else if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.peer_id = Some(peer_id);
            if normalized_public_key.is_some() {
                entry.public_key = normalized_public_key;
            }
            if normalized_nickname.is_some() {
                entry.nickname = normalized_nickname;
            }
            entry.last_seen = Some(current_timestamp());
            false
        } else {
            entries.push(LedgerEntry {
                multiaddr,
                peer_id: Some(peer_id),
                public_key: normalized_public_key,
                nickname: normalized_nickname,
                success_count: 0,
                failure_count: 0,
                last_seen: Some(current_timestamp()),
                topics: Vec::new(),
            });
            true
        };
        let _ = self.save_with_entries(&entries);
        let _ = is_new;
    }

    pub fn dialable_addresses(&self) -> Vec<LedgerEntry> {
        let entries = self.entries.lock();
        entries
            .iter()
            .filter(|e| e.success_count > 0 && e.failure_count < 5)
            .cloned()
            .collect()
    }

    pub fn get_preferred_relays(&self, limit: u32) -> Vec<LedgerEntry> {
        let entries = self.entries.lock();
        let mut preferred: Vec<LedgerEntry> = entries
            .iter()
            .filter(|e| e.success_count > 0)
            .cloned() // Clone now so we can sort
            .collect();
        // Sort by last_seen descending
        preferred.sort_by_key(|b| std::cmp::Reverse(b.last_seen.unwrap_or(0)));
        preferred.truncate(limit as usize);
        preferred
    }

    pub fn all_known_topics(&self) -> Vec<String> {
        let entries = self.entries.lock();
        let mut topics: Vec<String> = entries.iter().flat_map(|e| e.topics.clone()).collect();
        topics.sort();
        topics.dedup();
        topics
    }
}

/// A shared peer entry for ledger exchange.
/// Stripped-down version of ledger data suitable for wire transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SharedPeerEntry {
    /// The multiaddr (transport address only, no /p2p/ suffix)
    pub multiaddr: String,
    /// Last known PeerID at this address (if any)
    pub last_peer_id: Option<String>,
    /// Unix timestamp of last successful connection
    pub last_seen: u64,
    /// Gossipsub topics this peer was subscribed to
    pub known_topics: Vec<String>,
}

fn default_version() -> u8 {
    1
}

/// Ledger exchange request — sent automatically on new connection.
/// "Here are all the peers I know about. Tell me yours."
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeRequest {
    /// Explicit version tag for bincode wire format
    #[serde(default = "default_version")]
    pub version_tag: u8,
    /// Our known peers (shared generously)
    pub peers: Vec<SharedPeerEntry>,
    /// Our own PeerID (so the remote can record us)
    pub sender_peer_id: String,
    /// Protocol version for forward compatibility
    pub version: u32,
}

/// Ledger exchange response — reciprocal sharing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeResponse {
    /// Explicit version tag for bincode wire format
    #[serde(default = "default_version")]
    pub version_tag: u8,
    /// Their known peers (shared back)
    pub peers: Vec<SharedPeerEntry>,
    /// Number of new peers they learned from our request
    pub new_peers_learned: u32,
    /// Protocol version
    pub version: u32,
}
