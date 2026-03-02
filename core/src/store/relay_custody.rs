// Relay custody store — durable relay-side store-and-forward state.
//
// This stores messages accepted by a relay on behalf of offline recipients
// and records an auditable transition log for custody lifecycle changes.

#[cfg(not(target_arch = "wasm32"))]
use crate::store::backend::SledStorage;
use crate::store::backend::{MemoryStorage, StorageBackend};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::{ffi::CString, path::PathBuf};

const CUSTODY_MSG_PREFIX: &str = "relay_custody_msg_";
const CUSTODY_AUDIT_PREFIX: &str = "relay_custody_audit_";
const MAX_PENDING_PER_DESTINATION: usize = 10_000;
const DEVICE_USAGE_CEILING_PERCENT: u64 = 90;

static CUSTODY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePressureBand {
    UpTo20Pct,
    From20To50Pct,
    From50To70Pct,
    From70To80Pct,
    From80To90Pct,
    EmergencyOver90Pct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStorageSnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoragePressureState {
    pub band: StoragePressureBand,
    pub hard_ceiling_bytes: u64,
    pub target_quota_bytes: u64,
    pub scm_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StoragePressureReport {
    pub emergency_mode: bool,
    pub hard_ceiling_bytes: u64,
    pub target_quota_bytes: u64,
    pub scm_bytes_before: u64,
    pub scm_bytes_after: u64,
    pub purged_records: usize,
    pub purged_bytes: u64,
}

impl StoragePressureState {
    pub fn emergency_mode(self) -> bool {
        self.band == StoragePressureBand::EmergencyOver90Pct
    }
}

trait StoragePressureProbe: Send + Sync {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot>;
}

#[derive(Debug, Default)]
struct NoopStoragePressureProbe;

impl StoragePressureProbe for NoopStoragePressureProbe {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct FilesystemStoragePressureProbe {
    root: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl FilesystemStoragePressureProbe {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StoragePressureProbe for FilesystemStoragePressureProbe {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
        filesystem_usage_bytes(&self.root).map(|(total_bytes, used_bytes)| DeviceStorageSnapshot {
            total_bytes,
            used_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StoragePressureContext {
    total_bytes: u64,
    non_scm_used_bytes: u64,
}

impl StoragePressureContext {
    fn from_snapshot(snapshot: DeviceStorageSnapshot, scm_bytes: u64) -> Option<Self> {
        if snapshot.total_bytes == 0 {
            return None;
        }
        let used_bytes = snapshot.used_bytes.min(snapshot.total_bytes);
        let non_scm_used_bytes = used_bytes.saturating_sub(scm_bytes);
        Some(Self {
            total_bytes: snapshot.total_bytes,
            non_scm_used_bytes,
        })
    }

    fn state_for_scm_bytes(self, scm_bytes: u64) -> StoragePressureState {
        let used_bytes = self.non_scm_used_bytes.saturating_add(scm_bytes);
        let used_percent_basis_points =
            ((used_bytes as u128 * 10_000u128) / self.total_bytes as u128) as u64;
        let free_bytes = self.total_bytes.saturating_sub(used_bytes);
        let ninety_percent_total =
            ((self.total_bytes as u128 * DEVICE_USAGE_CEILING_PERCENT as u128) / 100u128) as u64;
        let hard_ceiling_bytes = ninety_percent_total.saturating_sub(self.non_scm_used_bytes);

        let (band, band_percent) = if used_percent_basis_points <= 2_000 {
            (StoragePressureBand::UpTo20Pct, 70u64)
        } else if used_percent_basis_points <= 5_000 {
            (StoragePressureBand::From20To50Pct, 45u64)
        } else if used_percent_basis_points <= 7_000 {
            (StoragePressureBand::From50To70Pct, 25u64)
        } else if used_percent_basis_points <= 8_000 {
            (StoragePressureBand::From70To80Pct, 10u64)
        } else if used_percent_basis_points <= 9_000 {
            (StoragePressureBand::From80To90Pct, 3u64)
        } else {
            (StoragePressureBand::EmergencyOver90Pct, 0u64)
        };

        let dynamic_target = if band == StoragePressureBand::EmergencyOver90Pct {
            hard_ceiling_bytes
        } else {
            ((free_bytes as u128 * band_percent as u128) / 100u128) as u64
        };
        let target_quota_bytes = dynamic_target.min(hard_ceiling_bytes);

        StoragePressureState {
            band,
            hard_ceiling_bytes,
            target_quota_bytes,
            scm_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyState {
    Accepted,
    Dispatching,
    Delivered,
}

impl CustodyState {
    fn as_str(self) -> &'static str {
        match self {
            CustodyState::Accepted => "accepted",
            CustodyState::Dispatching => "dispatching",
            CustodyState::Delivered => "delivered",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyMessage {
    pub custody_id: String,
    pub relay_message_id: String,
    pub source_peer_id: String,
    pub destination_peer_id: String,
    pub envelope_data: Vec<u8>,
    pub state: CustodyState,
    pub accepted_at_ms: u64,
    pub updated_at_ms: u64,
    pub delivery_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransition {
    pub custody_id: String,
    pub relay_message_id: String,
    pub source_peer_id: String,
    pub destination_peer_id: String,
    pub from_state: Option<CustodyState>,
    pub to_state: CustodyState,
    pub reason: String,
    pub at_ms: u64,
    #[serde(default)]
    pub sequence: u64,
}

#[derive(Clone)]
pub struct RelayCustodyStore {
    backend: Arc<dyn StorageBackend>,
    local_identity: Option<String>,
    pressure_probe: Arc<dyn StoragePressureProbe>,
}

impl RelayCustodyStore {
    pub fn in_memory() -> Self {
        Self::new_with_probe(
            Arc::new(MemoryStorage::new()),
            None,
            Arc::new(NoopStoragePressureProbe),
        )
    }

    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self::new_with_probe(backend, None, Arc::new(NoopStoragePressureProbe))
    }

    fn new_with_probe(
        backend: Arc<dyn StorageBackend>,
        local_identity: Option<String>,
        pressure_probe: Arc<dyn StoragePressureProbe>,
    ) -> Self {
        Self {
            backend,
            local_identity,
            pressure_probe,
        }
    }

    #[cfg(test)]
    fn in_memory_with_probe(
        local_identity: Option<String>,
        pressure_probe: Arc<dyn StoragePressureProbe>,
    ) -> Self {
        Self::new_with_probe(
            Arc::new(MemoryStorage::new()),
            local_identity,
            pressure_probe,
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_local_peer(local_peer_id: &str) -> Self {
        let base = std::env::var("SCM_RELAY_CUSTODY_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("scmessenger_relay_custody"));
        let dir = base.join(local_peer_id);
        let _ = std::fs::create_dir_all(&dir);

        let path = dir.to_string_lossy().to_string();
        match SledStorage::new(&path) {
            Ok(backend) => Self::new_with_probe(
                Arc::new(backend),
                Some(local_peer_id.to_string()),
                Arc::new(FilesystemStoragePressureProbe::new(dir)),
            ),
            Err(_) => Self::new_with_probe(
                Arc::new(MemoryStorage::new()),
                Some(local_peer_id.to_string()),
                Arc::new(NoopStoragePressureProbe),
            ),
        }
    }

    pub fn accept_custody(
        &self,
        source_peer_id: String,
        destination_peer_id: String,
        relay_message_id: String,
        envelope_data: Vec<u8>,
    ) -> Result<CustodyMessage, String> {
        if let Some(existing) = self.find_existing(&destination_peer_id, &relay_message_id)? {
            return Ok(existing);
        }

        let pending_count = self
            .pending_for_destination(&destination_peer_id, usize::MAX)
            .len();
        if pending_count >= MAX_PENDING_PER_DESTINATION {
            return Err(format!(
                "custody queue full for destination {}",
                destination_peer_id
            ));
        }

        let now_ms = now_ms();
        let sequence = CUSTODY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let custody_id = format!("{}-{}-{}", relay_message_id, now_ms, sequence);

        let message = CustodyMessage {
            custody_id,
            relay_message_id,
            source_peer_id,
            destination_peer_id,
            envelope_data,
            state: CustodyState::Accepted,
            accepted_at_ms: now_ms,
            updated_at_ms: now_ms,
            delivery_attempts: 0,
        };

        self.enforce_storage_pressure_for_write(&message)?;
        self.put_message(&message)?;
        self.record_transition(&message, None, CustodyState::Accepted, "custody_accepted")?;
        Ok(message)
    }

    pub fn storage_pressure_state(&self) -> Option<StoragePressureState> {
        let snapshot = self.pressure_probe.snapshot()?;
        let scm_bytes = self.current_scm_storage_bytes().ok()?;
        let context = StoragePressureContext::from_snapshot(snapshot, scm_bytes)?;
        Some(context.state_for_scm_bytes(scm_bytes))
    }

    pub fn enforce_storage_pressure(&self) -> Result<StoragePressureReport, String> {
        self.enforce_storage_pressure_internal(None)
    }

    pub fn pending_for_destination(
        &self,
        destination_peer_id: &str,
        limit: usize,
    ) -> Vec<CustodyMessage> {
        let prefix = destination_prefix(destination_peer_id);
        let mut records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(prefix.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.state == CustodyState::Accepted)
            .collect();
        records.sort_by_key(|record| (record.accepted_at_ms, record.custody_id.clone()));
        records.into_iter().take(limit).collect()
    }

    pub fn mark_dispatching(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Dispatching {
            return Ok(());
        }
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        let from_state = record.state;
        record.state = CustodyState::Dispatching;
        record.updated_at_ms = now_ms();
        record.delivery_attempts = record.delivery_attempts.saturating_add(1);
        self.put_message(&record)?;
        self.record_transition(&record, Some(from_state), CustodyState::Dispatching, reason)?;
        Ok(())
    }

    pub fn mark_dispatch_failed(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Accepted {
            return Ok(());
        }
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        let from_state = record.state;
        record.state = CustodyState::Accepted;
        record.updated_at_ms = now_ms();
        self.put_message(&record)?;
        self.record_transition(&record, Some(from_state), CustodyState::Accepted, reason)?;
        Ok(())
    }

    pub fn mark_delivered(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        let from_state = record.state;
        record.state = CustodyState::Delivered;
        record.updated_at_ms = now_ms();
        self.record_transition(&record, Some(from_state), CustodyState::Delivered, reason)?;
        self.remove_message(destination_peer_id, custody_id)?;
        Ok(())
    }

    /// Mark all duplicate custody records for a relay message as delivered and
    /// remove them from the pending queue.
    pub fn converge_delivered_for_message(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
        reason: &str,
    ) -> Result<usize, String> {
        let prefix = destination_prefix(destination_peer_id);
        let records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(prefix.as_bytes())?
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.relay_message_id == relay_message_id)
            .collect();

        if records.is_empty() {
            return Ok(0);
        }

        let mut converged = 0usize;
        for mut record in records {
            if record.state == CustodyState::Delivered {
                continue;
            }
            let from_state = record.state;
            record.state = CustodyState::Delivered;
            record.updated_at_ms = now_ms();
            self.record_transition(&record, Some(from_state), CustodyState::Delivered, reason)?;
            self.remove_message(destination_peer_id, &record.custody_id)?;
            converged += 1;
        }

        Ok(converged)
    }

    pub fn transitions_for_custody(&self, custody_id: &str) -> Vec<CustodyTransition> {
        let mut transitions: Vec<CustodyTransition> = self
            .backend
            .scan_prefix(CUSTODY_AUDIT_PREFIX.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyTransition>(&value).ok())
            .filter(|transition| transition.custody_id == custody_id)
            .collect();
        transitions.sort_by_key(|transition| (transition.at_ms, transition.sequence));
        transitions
    }

    pub fn audit_count(&self) -> usize {
        self.backend
            .count_prefix(CUSTODY_AUDIT_PREFIX.as_bytes())
            .unwrap_or(0)
    }

    fn enforce_storage_pressure_for_write(&self, incoming: &CustodyMessage) -> Result<(), String> {
        let report = self.enforce_storage_pressure_internal(Some(incoming))?;
        if let Some(snapshot) = self.pressure_probe.snapshot() {
            let write_bytes = serialized_record_bytes(incoming)?;
            let mut scm_bytes = report.scm_bytes_after;
            let context = StoragePressureContext::from_snapshot(snapshot, scm_bytes)
                .ok_or_else(|| "invalid_storage_snapshot".to_string())?;
            let mut state = context.state_for_scm_bytes(scm_bytes);

            if state.emergency_mode() && !self.is_identity_related_record(incoming) {
                return Err("emergency_mode_non_critical_rejected".to_string());
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.target_quota_bytes {
                let need = projected.saturating_sub(state.target_quota_bytes);
                let (_, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_target_quota")?;
                scm_bytes = scm_bytes.saturating_sub(purged_bytes);
                state = context.state_for_scm_bytes(scm_bytes);
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.hard_ceiling_bytes {
                let need = projected.saturating_sub(state.hard_ceiling_bytes);
                let (_, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_hard_ceiling")?;
                scm_bytes = scm_bytes.saturating_sub(purged_bytes);
                state = context.state_for_scm_bytes(scm_bytes);
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.target_quota_bytes || projected > state.hard_ceiling_bytes {
                return Err(format!(
                    "storage_pressure_capacity_exceeded: projected={} target={} hard_ceiling={}",
                    projected, state.target_quota_bytes, state.hard_ceiling_bytes
                ));
            }
        }
        Ok(())
    }

    fn enforce_storage_pressure_internal(
        &self,
        incoming: Option<&CustodyMessage>,
    ) -> Result<StoragePressureReport, String> {
        let scm_before = self.current_scm_storage_bytes()?;
        let Some(snapshot) = self.pressure_probe.snapshot() else {
            return Ok(StoragePressureReport {
                scm_bytes_before: scm_before,
                scm_bytes_after: scm_before,
                ..StoragePressureReport::default()
            });
        };

        let context = StoragePressureContext::from_snapshot(snapshot, scm_before)
            .ok_or_else(|| "invalid_storage_snapshot".to_string())?;
        let state = context.state_for_scm_bytes(scm_before);
        let mut report = StoragePressureReport {
            emergency_mode: state.emergency_mode(),
            hard_ceiling_bytes: state.hard_ceiling_bytes,
            target_quota_bytes: state.target_quota_bytes,
            scm_bytes_before: scm_before,
            scm_bytes_after: scm_before,
            ..StoragePressureReport::default()
        };

        if state.emergency_mode()
            && incoming
                .map(|record| !self.is_identity_related_record(record))
                .unwrap_or(false)
        {
            let need = report
                .scm_bytes_after
                .saturating_sub(state.hard_ceiling_bytes);
            if need > 0 {
                let (purged_records, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_emergency")?;
                report.purged_records += purged_records;
                report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
                report.scm_bytes_after = report.scm_bytes_after.saturating_sub(purged_bytes);
            }
            return Ok(report);
        }

        let mut scm_bytes = report.scm_bytes_after;
        let mut current_state = context.state_for_scm_bytes(scm_bytes);
        let mut required = scm_bytes.saturating_sub(current_state.target_quota_bytes);
        if required > 0 {
            let (purged_records, purged_bytes) =
                self.purge_oldest_by_policy(required, "storage_pressure_target_quota")?;
            scm_bytes = scm_bytes.saturating_sub(purged_bytes);
            report.purged_records += purged_records;
            report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
            current_state = context.state_for_scm_bytes(scm_bytes);
            required = scm_bytes.saturating_sub(current_state.hard_ceiling_bytes);
        } else {
            required = scm_bytes.saturating_sub(current_state.hard_ceiling_bytes);
        }

        if required > 0 {
            let (purged_records, purged_bytes) =
                self.purge_oldest_by_policy(required, "storage_pressure_hard_ceiling")?;
            scm_bytes = scm_bytes.saturating_sub(purged_bytes);
            report.purged_records += purged_records;
            report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
            current_state = context.state_for_scm_bytes(scm_bytes);
        }

        report.scm_bytes_after = scm_bytes;
        report.target_quota_bytes = current_state.target_quota_bytes;
        report.hard_ceiling_bytes = current_state.hard_ceiling_bytes;
        report.emergency_mode = current_state.emergency_mode();
        Ok(report)
    }

    fn current_scm_storage_bytes(&self) -> Result<u64, String> {
        let records = self.backend.scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?;
        Ok(records
            .into_iter()
            .map(|(_, value)| value.len() as u64)
            .sum::<u64>())
    }

    fn load_stored_records(&self) -> Result<Vec<StoredCustodyRecord>, String> {
        let records = self.backend.scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?;
        let mut parsed = Vec::with_capacity(records.len());
        for (_, value) in records {
            if let Ok(record) = bincode::deserialize::<CustodyMessage>(&value) {
                parsed.push(StoredCustodyRecord {
                    record,
                    serialized_bytes: value.len() as u64,
                });
            }
        }
        Ok(parsed)
    }

    fn purge_oldest_by_policy(
        &self,
        mut required_bytes: u64,
        reason: &str,
    ) -> Result<(usize, u64), String> {
        if required_bytes == 0 {
            return Ok((0, 0));
        }
        let mut candidates = self.load_stored_records()?;
        candidates.sort_by_key(|candidate| {
            (
                candidate.identity_related(self),
                candidate.delivery_priority(),
                candidate.record.accepted_at_ms,
                candidate.record.custody_id.clone(),
            )
        });

        let mut purged_records = 0usize;
        let mut purged_bytes = 0u64;
        for candidate in candidates {
            if required_bytes == 0 {
                break;
            }
            let purge_reason = format!("{}_purged", reason);
            self.record_transition(
                &candidate.record,
                Some(candidate.record.state),
                candidate.record.state,
                &purge_reason,
            )?;
            self.remove_message(
                &candidate.record.destination_peer_id,
                &candidate.record.custody_id,
            )?;
            purged_records += 1;
            purged_bytes = purged_bytes.saturating_add(candidate.serialized_bytes);
            required_bytes = required_bytes.saturating_sub(candidate.serialized_bytes);
            tracing::warn!(
                "purged custody {} due to {} ({} bytes)",
                candidate.record.custody_id,
                reason,
                candidate.serialized_bytes
            );
        }
        Ok((purged_records, purged_bytes))
    }

    fn is_identity_related_record(&self, record: &CustodyMessage) -> bool {
        self.is_identity_related_ids(&record.source_peer_id, &record.destination_peer_id)
    }

    fn is_identity_related_ids(&self, source_peer_id: &str, destination_peer_id: &str) -> bool {
        let Some(local_identity) = self.local_identity.as_deref() else {
            return false;
        };
        source_peer_id == local_identity || destination_peer_id == local_identity
    }

    fn find_existing(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
    ) -> Result<Option<CustodyMessage>, String> {
        let prefix = destination_prefix(destination_peer_id);
        for (_, value) in self.backend.scan_prefix(prefix.as_bytes())? {
            if let Ok(record) = bincode::deserialize::<CustodyMessage>(&value) {
                if record.relay_message_id == relay_message_id {
                    return Ok(Some(record));
                }
            }
        }
        Ok(None)
    }

    fn require_record(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
    ) -> Result<CustodyMessage, String> {
        self.get_message(destination_peer_id, custody_id)?
            .ok_or_else(|| format!("custody record not found: {}", custody_id))
    }

    fn get_message(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
    ) -> Result<Option<CustodyMessage>, String> {
        let key = message_key(destination_peer_id, custody_id);
        if let Some(bytes) = self.backend.get(key.as_bytes())? {
            let record = bincode::deserialize::<CustodyMessage>(&bytes)
                .map_err(|e| format!("deserialize custody record failed: {}", e))?;
            return Ok(Some(record));
        }
        Ok(None)
    }

    fn put_message(&self, record: &CustodyMessage) -> Result<(), String> {
        let key = message_key(&record.destination_peer_id, &record.custody_id);
        let bytes = bincode::serialize(record)
            .map_err(|e| format!("serialize custody record failed: {}", e))?;
        self.backend.put(key.as_bytes(), &bytes)?;
        self.backend.flush()?;
        Ok(())
    }

    fn remove_message(&self, destination_peer_id: &str, custody_id: &str) -> Result<(), String> {
        let key = message_key(destination_peer_id, custody_id);
        self.backend.remove(key.as_bytes())?;
        self.backend.flush()?;
        Ok(())
    }

    fn record_transition(
        &self,
        record: &CustodyMessage,
        from_state: Option<CustodyState>,
        to_state: CustodyState,
        reason: &str,
    ) -> Result<(), String> {
        let at_ms = now_ms();
        let sequence = CUSTODY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let key = format!(
            "{}{:020}_{:06}_{}_{}",
            CUSTODY_AUDIT_PREFIX,
            at_ms,
            sequence,
            record.custody_id,
            to_state.as_str()
        );
        let transition = CustodyTransition {
            custody_id: record.custody_id.clone(),
            relay_message_id: record.relay_message_id.clone(),
            source_peer_id: record.source_peer_id.clone(),
            destination_peer_id: record.destination_peer_id.clone(),
            from_state,
            to_state,
            reason: reason.to_string(),
            at_ms,
            sequence,
        };
        let bytes = bincode::serialize(&transition)
            .map_err(|e| format!("serialize custody transition failed: {}", e))?;
        self.backend.put(key.as_bytes(), &bytes)?;
        self.backend.flush()?;
        Ok(())
    }
}

impl Default for RelayCustodyStore {
    fn default() -> Self {
        Self::in_memory()
    }
}

#[derive(Debug, Clone)]
struct StoredCustodyRecord {
    record: CustodyMessage,
    serialized_bytes: u64,
}

impl StoredCustodyRecord {
    fn identity_related(&self, store: &RelayCustodyStore) -> bool {
        store.is_identity_related_record(&self.record)
    }

    fn delivery_priority(&self) -> u8 {
        if self.record.state == CustodyState::Delivered {
            0
        } else {
            1
        }
    }
}

fn serialized_record_bytes(record: &CustodyMessage) -> Result<u64, String> {
    let bytes = bincode::serialize(record)
        .map_err(|e| format!("serialize custody record failed: {}", e))?;
    Ok(bytes.len() as u64)
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(unix)]
fn filesystem_usage_bytes(path: &std::path::Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        return None;
    }

    let block_size = if stat.f_frsize > 0 {
        stat.f_frsize as u64
    } else {
        stat.f_bsize as u64
    };
    if block_size == 0 {
        return None;
    }

    let total_bytes = (stat.f_blocks as u128)
        .saturating_mul(block_size as u128)
        .min(u64::MAX as u128) as u64;
    let available_bytes = (stat.f_bavail as u128)
        .saturating_mul(block_size as u128)
        .min(u64::MAX as u128) as u64;
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    Some((total_bytes, used_bytes))
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(unix))]
fn filesystem_usage_bytes(_path: &std::path::Path) -> Option<(u64, u64)> {
    None
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn destination_prefix(destination_peer_id: &str) -> String {
    format!("{}{}_", CUSTODY_MSG_PREFIX, destination_peer_id)
}

fn message_key(destination_peer_id: &str, custody_id: &str) -> String {
    format!("{}{}", destination_prefix(destination_peer_id), custody_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    use std::time::Duration;

    #[derive(Default)]
    struct TestPressureProbe {
        snapshot: RwLock<Option<DeviceStorageSnapshot>>,
    }

    impl TestPressureProbe {
        fn set(&self, snapshot: DeviceStorageSnapshot) {
            *self.snapshot.write().expect("snapshot lock poisoned") = Some(snapshot);
        }
    }

    impl StoragePressureProbe for TestPressureProbe {
        fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
            *self.snapshot.read().expect("snapshot lock poisoned")
        }
    }

    #[test]
    fn custody_transitions_are_recorded() {
        let store = RelayCustodyStore::in_memory();
        let accepted = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-1".to_string(),
                vec![1, 2, 3],
            )
            .unwrap();

        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "reconnect_pull")
            .unwrap();
        store
            .mark_dispatch_failed(
                "destination-peer",
                &accepted.custody_id,
                "temporary_failure",
            )
            .unwrap();
        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "retry")
            .unwrap();
        store
            .mark_delivered("destination-peer", &accepted.custody_id, "recipient_ack")
            .unwrap();

        let transitions = store.transitions_for_custody(&accepted.custody_id);
        assert_eq!(transitions.len(), 5);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].to_state, CustodyState::Dispatching);
        assert_eq!(transitions[2].to_state, CustodyState::Accepted);
        assert_eq!(transitions[3].to_state, CustodyState::Dispatching);
        assert_eq!(transitions[4].to_state, CustodyState::Delivered);
        assert!(store
            .pending_for_destination("destination-peer", 100)
            .is_empty());
    }

    #[test]
    fn custody_deduplicates_same_destination_and_message_id() {
        let store = RelayCustodyStore::in_memory();
        let first = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-dedupe".to_string(),
                vec![9, 9, 9],
            )
            .unwrap();
        let second = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-dedupe".to_string(),
                vec![9, 9, 9],
            )
            .unwrap();

        assert_eq!(first.custody_id, second.custody_id);
        assert_eq!(
            store.pending_for_destination("destination-peer", 100).len(),
            1
        );
    }

    #[test]
    fn converge_delivered_for_message_removes_matching_pending_records() {
        let store = RelayCustodyStore::in_memory();
        let _ = store
            .accept_custody(
                "source-peer-a".to_string(),
                "destination-peer".to_string(),
                "relay-msg-converge".to_string(),
                vec![1, 2, 3],
            )
            .unwrap();
        let other = store
            .accept_custody(
                "source-peer-b".to_string(),
                "destination-peer".to_string(),
                "relay-msg-other".to_string(),
                vec![4, 5, 6],
            )
            .unwrap();

        let converged = store
            .converge_delivered_for_message(
                "destination-peer",
                "relay-msg-converge",
                "delivery_converged",
            )
            .unwrap();
        assert_eq!(converged, 1);

        let pending = store.pending_for_destination("destination-peer", 100);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].relay_message_id, "relay-msg-other");

        let transitions = store.transitions_for_custody(&other.custody_id);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
    }

    #[test]
    fn storage_pressure_quota_bands_follow_locked_policy() {
        let scm_bytes = 50_000u64;
        let ctx20 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 200_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s20 = ctx20.state_for_scm_bytes(scm_bytes);
        assert_eq!(s20.band, StoragePressureBand::UpTo20Pct);
        assert_eq!(s20.target_quota_bytes, 560_000);

        let ctx50 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 500_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s50 = ctx50.state_for_scm_bytes(scm_bytes);
        assert_eq!(s50.band, StoragePressureBand::From20To50Pct);
        assert_eq!(s50.target_quota_bytes, 225_000);

        let ctx70 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 700_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s70 = ctx70.state_for_scm_bytes(scm_bytes);
        assert_eq!(s70.band, StoragePressureBand::From50To70Pct);
        assert_eq!(s70.target_quota_bytes, 75_000);

        let ctx80 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 800_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s80 = ctx80.state_for_scm_bytes(scm_bytes);
        assert_eq!(s80.band, StoragePressureBand::From70To80Pct);
        assert_eq!(s80.target_quota_bytes, 20_000);

        let ctx90 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 900_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s90 = ctx90.state_for_scm_bytes(scm_bytes);
        assert_eq!(s90.band, StoragePressureBand::From80To90Pct);
        assert_eq!(s90.target_quota_bytes, 3_000);

        let ctx_over_90 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 910_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s_over_90 = ctx_over_90.state_for_scm_bytes(scm_bytes);
        assert_eq!(s_over_90.band, StoragePressureBand::EmergencyOver90Pct);
        assert!(s_over_90.emergency_mode());
        assert_eq!(s_over_90.target_quota_bytes, s_over_90.hard_ceiling_bytes);
    }

    fn seed_purge_order_records(store: &RelayCustodyStore) {
        let payload = vec![7u8; 256];
        let _ = store
            .accept_custody(
                "peer-non-old-src".to_string(),
                "peer-non-old-dst".to_string(),
                "msg-non-old".to_string(),
                payload.clone(),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "local-peer".to_string(),
                "peer-id-old-dst".to_string(),
                "msg-id-old".to_string(),
                payload.clone(),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "peer-non-new-src".to_string(),
                "peer-non-new-dst".to_string(),
                "msg-non-new".to_string(),
                payload.clone(),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "peer-id-new-src".to_string(),
                "local-peer".to_string(),
                "msg-id-new".to_string(),
                payload,
            )
            .unwrap();
    }

    #[test]
    fn storage_pressure_purge_prioritizes_non_identity_then_identity() {
        let store = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        seed_purge_order_records(&store);

        let records = store.load_stored_records().unwrap();
        assert_eq!(records.len(), 4);
        let non_identity_bytes = records
            .iter()
            .filter(|entry| !entry.identity_related(&store))
            .map(|entry| entry.serialized_bytes)
            .sum::<u64>();

        let (purged_records, _) = store
            .purge_oldest_by_policy(non_identity_bytes, "test_non_identity_first")
            .unwrap();
        assert_eq!(purged_records, 2);

        let mut remaining: Vec<String> = store
            .load_stored_records()
            .unwrap()
            .into_iter()
            .map(|entry| entry.record.relay_message_id)
            .collect();
        remaining.sort();
        assert_eq!(
            remaining,
            vec!["msg-id-new".to_string(), "msg-id-old".to_string()]
        );

        let store2 = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        seed_purge_order_records(&store2);

        let records2 = store2.load_stored_records().unwrap();
        let non_identity_bytes2 = records2
            .iter()
            .filter(|entry| !entry.identity_related(&store2))
            .map(|entry| entry.serialized_bytes)
            .sum::<u64>();

        let (purged_records2, _) = store2
            .purge_oldest_by_policy(non_identity_bytes2 + 1, "test_identity_when_required")
            .unwrap();
        assert_eq!(purged_records2, 3);

        let remaining2 = store2.load_stored_records().unwrap();
        assert_eq!(remaining2.len(), 1);
        assert_eq!(remaining2[0].record.relay_message_id, "msg-id-new");
    }

    #[test]
    fn storage_pressure_purge_records_audit_transition_before_delete() {
        let store = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        let accepted = store
            .accept_custody(
                "peer-src".to_string(),
                "peer-dst".to_string(),
                "relay-msg-audit-purge".to_string(),
                vec![9u8; 64],
            )
            .unwrap();

        let (purged_records, purged_bytes) = store
            .purge_oldest_by_policy(1, "test_pressure")
            .unwrap();
        assert_eq!(purged_records, 1);
        assert!(purged_bytes > 0);

        let transitions = store.transitions_for_custody(&accepted.custody_id);
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].from_state, Some(CustodyState::Accepted));
        assert_eq!(transitions[1].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].reason, "test_pressure_purged");
    }

    #[test]
    fn storage_pressure_emergency_mode_rejects_non_critical_and_recovers() {
        let probe = Arc::new(TestPressureProbe::default());
        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 50_000,
        });
        let store =
            RelayCustodyStore::in_memory_with_probe(Some("local-peer".to_string()), probe.clone());

        let _ = store
            .accept_custody(
                "peer-pre-emergency-src".to_string(),
                "peer-pre-emergency-dst".to_string(),
                "msg-pre-emergency".to_string(),
                vec![1u8; 256],
            )
            .unwrap();

        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 95_000,
        });
        let report = store.enforce_storage_pressure().unwrap();
        assert!(report.emergency_mode);
        assert_eq!(
            store.storage_pressure_state().unwrap().band,
            StoragePressureBand::EmergencyOver90Pct
        );

        let rejected = store.accept_custody(
            "peer-emergency-src".to_string(),
            "peer-emergency-dst".to_string(),
            "msg-rejected-emergency".to_string(),
            vec![2u8; 256],
        );
        assert!(rejected
            .unwrap_err()
            .contains("emergency_mode_non_critical_rejected"));

        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 85_000,
        });
        let accepted = store.accept_custody(
            "peer-post-emergency-src".to_string(),
            "peer-post-emergency-dst".to_string(),
            "msg-post-emergency".to_string(),
            vec![3u8; 256],
        );
        assert!(accepted.is_ok());
        assert_ne!(
            store.storage_pressure_state().unwrap().band,
            StoragePressureBand::EmergencyOver90Pct
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn custody_audit_persists_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("relay_custody_store");
        let path_str = path.to_string_lossy().to_string();

        let custody_id = {
            let backend = Arc::new(SledStorage::new(&path_str).unwrap());
            let store = RelayCustodyStore::persistent(backend);
            let accepted = store
                .accept_custody(
                    "source-peer".to_string(),
                    "destination-peer".to_string(),
                    "relay-msg-persist".to_string(),
                    vec![7, 7, 7],
                )
                .unwrap();
            store
                .mark_dispatching("destination-peer", &accepted.custody_id, "reconnect_pull")
                .unwrap();
            accepted.custody_id
        };

        let backend = Arc::new(SledStorage::new(&path_str).unwrap());
        let reloaded = RelayCustodyStore::persistent(backend);
        let transitions = reloaded.transitions_for_custody(&custody_id);
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].to_state, CustodyState::Dispatching);
        assert_eq!(
            reloaded
                .pending_for_destination("destination-peer", 100)
                .len(),
            0
        );
    }
}
