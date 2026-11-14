use crate::store::backend::StorageBackend;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMemoryEntry {
    pub transport: String,
    pub port: u16,
    pub last_success_unix: u64,
    pub ladder_rank: u32,
}

pub struct TransportMemoryStore {
    backend: Arc<dyn StorageBackend>,
}

impl TransportMemoryStore {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    fn key(peer_id: &PeerId, network_fingerprint: &str) -> String {
        format!("tmem:{}:{}", peer_id, network_fingerprint)
    }

    pub fn record_success(
        &self,
        peer_id: &PeerId,
        network_fingerprint: &str,
        transport: String,
        port: u16,
        ladder_rank: u32,
    ) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = TransportMemoryEntry {
            transport,
            port,
            last_success_unix: now,
            ladder_rank,
        };
        let bytes = serde_json::to_vec(&entry).map_err(|e| e.to_string())?;
        self.backend
            .put(Self::key(peer_id, network_fingerprint).as_bytes(), &bytes)?;
        Ok(())
    }

    pub fn get_last_good(
        &self,
        peer_id: &PeerId,
        network_fingerprint: &str,
    ) -> Result<Option<TransportMemoryEntry>, String> {
        if let Some(bytes) = self
            .backend
            .get(Self::key(peer_id, network_fingerprint).as_bytes())?
        {
            let entry: TransportMemoryEntry =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}

pub fn get_network_fingerprint() -> String {
    // Placeholder network fingerprint function as requested.
    // In the future: Hash of MAC + Subnet /24
    "placeholder_network_fingerprint".to_string()
}
