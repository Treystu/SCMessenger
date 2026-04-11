//! BLE GATT central path: discover SCMessenger peripherals, subscribe to notify, forward
//! decrypted payloads to the local Web UI as JSON-RPC `message_received`.
//!
//! **Advertising:** btleplug is central-oriented on desktop OSes; the CLI does not expose a
//! full peripheral GATT server here. Mobile/native peers remain peripherals; this node scans,
//! connects, and ingests notify payloads.

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, CentralEvent, CharPropFlags, Manager as _, Peripheral as PeripheralApi, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use futures_util::StreamExt;
use scmessenger_core::drift::frame::{DriftFrame, FrameType};
use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams};
use scmessenger_core::IronCore;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::server::{UiEvent, UiOutbound};

/// SCM GATT primary service UUID (must match `core/src/transport/ble/gatt.rs`).
const GATT_SERVICE_UUID: u128 = 0xDF01_0000_0000_1000_8000_00805F9B34FB;

fn scm_service_uuid() -> Uuid {
    Uuid::from_u128(GATT_SERVICE_UUID)
}

fn scm_notify_uuid() -> Uuid {
    uuid_from_u16(0xDF03)
}

/// Decode Drift-framed or raw envelope bytes; decrypt/verify via [`IronCore::receive_message`].
pub fn decode_ble_payload_for_ui(core: &IronCore, data: &[u8]) -> Option<MessageReceivedParams> {
    let payload: Vec<u8> = match DriftFrame::from_bytes(data) {
        Ok(f) => {
            if f.frame_type != FrameType::Data {
                return None;
            }
            f.payload
        }
        Err(_) => data.to_vec(),
    };
    let msg = core.receive_message(payload).ok()?;
    let from = msg.sender_id.clone();
    let content = msg.text_content().unwrap_or_default();
    let timestamp = msg.timestamp;
    let message_id = msg.id;
    Some(MessageReceivedParams {
        from,
        content,
        timestamp,
        message_id,
    })
}

fn push_message_to_ui(ui_tx: &tokio::sync::broadcast::Sender<UiOutbound>, p: MessageReceivedParams) {
    let legacy = UiEvent::MessageReceived {
        from: p.from.clone(),
        content: p.content.clone(),
        timestamp: p.timestamp,
        message_id: p.message_id.clone(),
    };
    let _ = ui_tx.send(UiOutbound::Legacy(legacy));
    let n = notif_message_received(p);
    if let Ok(v) = serde_json::to_value(&n) {
        let _ = ui_tx.send(UiOutbound::JsonRpc(v));
    }
}

async fn subscribe_ingress_for_peripheral(
    peripheral: Peripheral,
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    let addr = peripheral.address().to_string();
    if let Err(e) = peripheral.connect().await {
        tracing::debug!("BLE connect skipped/failed for {}: {}", addr, e);
        return;
    }
    if let Err(e) = peripheral.discover_services().await {
        tracing::warn!("BLE discover_services failed for {}: {}", addr, e);
        let _ = peripheral.disconnect().await;
        return;
    }
    let notify_uuid = scm_notify_uuid();
    let ch = peripheral
        .characteristics()
        .iter()
        .find(|c| c.uuid == notify_uuid && c.properties.contains(CharPropFlags::NOTIFY))
        .cloned();
    let Some(ch) = ch else {
        tracing::debug!("BLE no notify char {:} on {}", notify_uuid, addr);
        let _ = peripheral.disconnect().await;
        return;
    };
    if let Err(e) = peripheral.subscribe(&ch).await {
        tracing::warn!("BLE subscribe failed for {}: {}", addr, e);
        let _ = peripheral.disconnect().await;
        return;
    }
    tracing::info!(
        "BLE GATT notify subscribed on {} (SCM ingress for thin client WebSocket)",
        addr
    );

    let mut stream = match peripheral.notifications().await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("BLE notifications stream failed: {}", e);
            return;
        }
    };

    while let Some(note) = stream.next().await {
        if let Some(params) = decode_ble_payload_for_ui(core.as_ref(), &note.value) {
            push_message_to_ui(&ui_tx, params);
        }
    }
}

/// Run until process exit: scan for SCM service, connect + notify per peripheral.
pub async fn run_ble_central_ingress(
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        let _ = (core, ui_tx);
        tracing::debug!("BLE central ingress: unsupported OS");
        return;
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))]
    {
        tracing::info!(
            "BLE: CLI GATT central for service {:x} (peripheral advertising via btleplug not enabled).",
            GATT_SERVICE_UUID
        );

        let manager = match Manager::new().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("BLE Manager::new failed: {}", e);
                return;
            }
        };
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("BLE adapters() failed: {}", e);
                return;
            }
        };
        let Some(adapter) = adapters.first() else {
            tracing::warn!("BLE: no adapters");
            return;
        };

        let svc = scm_service_uuid();
        let filter = ScanFilter {
            services: vec![svc],
        };
        if let Err(e) = adapter.start_scan(filter).await {
            tracing::warn!("BLE start_scan failed: {}", e);
            return;
        }
        tracing::info!("BLE scan active (filtered to SCM service {})", svc);

        let mut events = match adapter.events().await {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("BLE events() failed: {}", e);
                return;
            }
        };

        let tracked: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        while let Some(evt) = events.next().await {
            let CentralEvent::ServicesAdvertisement { id, services } = evt else {
                continue;
            };
            if !services.contains(&svc) {
                continue;
            }

            let id_key = format!("{:?}", id);
            {
                let mut guard = tracked.lock().await;
                if guard.contains(&id_key) {
                    continue;
                }
                guard.insert(id_key.clone());
            }

            let peripheral = match adapter.peripheral(&id).await {
                Ok(p) => p,
                Err(_) => {
                    tracked.lock().await.remove(&id_key);
                    continue;
                }
            };

            let core_c = Arc::clone(&core);
            let ui_c = ui_tx.clone();
            let track = Arc::clone(&tracked);
            let key = id_key.clone();
            tokio::spawn(async move {
                subscribe_ingress_for_peripheral(peripheral, core_c, ui_c).await;
                track.lock().await.remove(&key);
            });
        }
    }
}

/// Run peripheral advertising.
/// Note: btleplug doesn't fully support advertising cross-platform native.
/// This logs intention for mobile peripheral discovery.
pub async fn run_ble_peripheral_advertising(_core: Arc<IronCore>) {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        tracing::info!(
            "BLE: GATT advertising stub started for service {:x} (Awaiting full platform advertising support).",
            GATT_SERVICE_UUID
        );
        
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmessenger_core::IronCore as CoreIron;

    #[test]
    fn decode_rejects_short_buffer() {
        let core = CoreIron::new();
        let _ = core.start();
        let junk = [0u8; 4];
        assert!(decode_ble_payload_for_ui(&core, &junk).is_none());
    }
}
