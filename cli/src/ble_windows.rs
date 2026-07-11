// cli/src/ble_windows.rs
#![cfg(target_os = "windows")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, error, info, warn};

use scmessenger_core::transport::ble::{GattFragmentHeader, GattReassembler};
use scmessenger_core::IronCore;
use crate::server::UiOutbound;

use windows::core::{GUID, HSTRING, Result as WinResult};
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristicProperties, GattLocalCharacteristic, GattLocalCharacteristicParameters,
    GattProtectionLevel, GattServiceProvider, GattServiceProviderAdvertisingParameters,
    GattReadRequestedEventArgs, GattWriteRequestedEventArgs,
};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::{DataReader, DataWriter};

const GATT_SERVICE_UUID: u128 = 0x0000_DF01_0000_1000_8000_0080_5F9B_34FB;
const IDENTITY_CHAR_UUID: u128 = 0x0000_DF02_0000_1000_8000_0080_5F9B_34FB;
const MESSAGE_CHAR_UUID: u128 = 0x0000_DF03_0000_1000_8000_0080_5F9B_34FB;

struct ConnectionState {
    fragments: HashMap<u16, Vec<u8>>,
    total_fragments: Option<u16>,
}

static REASSEMBLY_BUFFERS: OnceLock<Mutex<HashMap<String, ConnectionState>>> = OnceLock::new();
static MESSAGE_CHAR: OnceLock<GattLocalCharacteristic> = OnceLock::new();

fn get_reassembly_buffers() -> &'static Mutex<HashMap<String, ConnectionState>> {
    REASSEMBLY_BUFFERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn set_message_characteristic(c: GattLocalCharacteristic) {
    let _ = MESSAGE_CHAR.set(c);
}

pub fn get_message_characteristic() -> Option<GattLocalCharacteristic> {
    MESSAGE_CHAR.get().cloned()
}

/// Send a message to all subscribed centrals over Windows BLE GATT notification.
pub async fn send_windows_ble_notification(data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(msg_char) = get_message_characteristic() else {
        return Err("Windows BLE: Message characteristic not initialized".into());
    };

    // Fragment using standard GattFragmenter
    let fragments = scmessenger_core::transport::ble::GattFragmenter::fragment(data)
        .map_err(|e| format!("GATT fragmentation error: {:?}", e))?;

    debug!("Windows BLE: sending {} fragments to subscribed clients", fragments.len());
    for fragment in fragments {
        let msg_char_clone = msg_char.clone();
        let fragment_clone = fragment.clone();

        // Spawn a background task to isolate non-Send WinRT types and await its JoinHandle (which is Send)
        tokio::spawn(async move {
            let op = {
                let writer = DataWriter::new()?;
                writer.WriteBytes(&fragment_clone)?;
                let buffer = writer.DetachBuffer()?;
                msg_char_clone.NotifyValueAsync(&buffer)?
            };
            op.await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }).await??;
    }

    Ok(())
}

/// Run Windows WinRT BLE Peripheral GATT Server and Advertising.
pub async fn run_windows_ble_peripheral(
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Windows BLE: initializing GATT Service Provider...");

    let service_uuid = GUID::from_u128(GATT_SERVICE_UUID);
    let provider_result = GattServiceProvider::CreateAsync(service_uuid)?.await?;
    let provider = provider_result.ServiceProvider()?;

    let service = provider.Service()?;

    // 1. Identity Characteristic (0xDF02)
    let identity_uuid = GUID::from_u128(IDENTITY_CHAR_UUID);
    let mut parameters = GattLocalCharacteristicParameters::new()?;
    parameters.SetReadProtectionLevel(GattProtectionLevel::Plain)?;
    parameters.SetWriteProtectionLevel(GattProtectionLevel::Plain)?;
    parameters.SetCharacteristicProperties(GattCharacteristicProperties::Read)?;

    let identity_result = service.CreateCharacteristicAsync(identity_uuid, &parameters)?.await?;
    let identity_char = identity_result.Characteristic()?;

    // Setup ReadRequested handler for Identity
    let read_core = Arc::clone(&core);
    identity_char.ReadRequested(&TypedEventHandler::new(move |_sender: windows::core::Ref<'_, GattLocalCharacteristic>, args: windows::core::Ref<'_, GattReadRequestedEventArgs>| {
        let args_ref = args.ok()?;
        let deferral = args_ref.GetDeferral()?;
        let args_clone = args_ref.clone();
        let read_core_clone = read_core.clone();
        tokio::spawn(async move {
            if let Ok(request_op) = args_clone.GetRequestAsync() {
                if let Ok(request) = request_op.await {
                    let info = read_core_clone.get_identity_info();
                    let peer_id_str = info.libp2p_peer_id.unwrap_or_default();
                    let response_json = serde_json::json!({ "peer_id": peer_id_str }).to_string();
                    let response_bytes = response_json.as_bytes().to_vec();

                    if let Ok(writer) = DataWriter::new() {
                        let _ = writer.WriteBytes(&response_bytes);
                        if let Ok(buffer) = writer.DetachBuffer() {
                            let _ = request.RespondWithValue(&buffer);
                        }
                    }
                }
            }
            let _ = deferral.Complete();
        });
        Ok(())
    }))?;

    // 2. Message Characteristic (0xDF03)
    let message_uuid = GUID::from_u128(MESSAGE_CHAR_UUID);
    let mut parameters = GattLocalCharacteristicParameters::new()?;
    parameters.SetReadProtectionLevel(GattProtectionLevel::Plain)?;
    parameters.SetWriteProtectionLevel(GattProtectionLevel::Plain)?;
    parameters.SetCharacteristicProperties(
        GattCharacteristicProperties::Write
            | GattCharacteristicProperties::WriteWithoutResponse
            | GattCharacteristicProperties::Notify,
    )?;

    let message_result = service.CreateCharacteristicAsync(message_uuid, &parameters)?.await?;
    let message_char = message_result.Characteristic()?;

    // Store for outbound notification writes
    set_message_characteristic(message_char.clone());

    // Setup WriteRequested handler for Message Characteristic
    let write_core = Arc::clone(&core);
    let write_ui_tx = ui_tx.clone();
    message_char.WriteRequested(&TypedEventHandler::new(move |_sender: windows::core::Ref<'_, GattLocalCharacteristic>, args: windows::core::Ref<'_, GattWriteRequestedEventArgs>| {
        let args_ref = args.ok()?;
        let deferral = args_ref.GetDeferral()?;
        let args_clone = args_ref.clone();
        let write_core_clone = write_core.clone();
        let write_ui_tx_clone = write_ui_tx.clone();
        tokio::spawn(async move {
            if let Ok(request_op) = args_clone.GetRequestAsync() {
                if let Ok(request) = request_op.await {
                    if let Ok(value) = request.Value() {
                        if let Ok(reader) = DataReader::FromBuffer(&value) {
                            if let Ok(len) = reader.UnconsumedBufferLength() {
                                let mut bytes = vec![0u8; len as usize];
                                if reader.ReadBytes(&mut bytes).is_ok() {
                                    if let Ok(session) = args_clone.Session() {
                                        if let Ok(device_id) = session.DeviceId() {
                                            if let Ok(device_id_str) = device_id.Id() {
                                                let device_id_string = device_id_str.to_string();
                                                if let Err(e) = handle_write_request(&write_core_clone, &write_ui_tx_clone, &device_id_string, &bytes) {
                                                    warn!("Windows BLE: write request handling error: {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let _ = request.Respond();
                }
            }
            let _ = deferral.Complete();
        });
        Ok(())
    }))?;

    // 3. Start Advertising
    let adv_parameters = GattServiceProviderAdvertisingParameters::new()?;
    adv_parameters.SetIsConnectable(true)?;
    adv_parameters.SetIsDiscoverable(true)?;

    info!("Windows BLE: starting peripheral LE advertisement...");
    provider.StartAdvertisingWithParameters(&adv_parameters)?;

    // Keep active to prevent local characteristics / provider from dropping
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

fn handle_write_request(
    core: &IronCore,
    ui_tx: &tokio::sync::broadcast::Sender<UiOutbound>,
    device_id: &str,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let header = GattFragmentHeader::from_bytes(bytes)
        .map_err(|e| format!("Invalid fragment header: {:?}", e))?;

    let total = header.total_fragments;
    let index = header.fragment_index;

    let payload = if bytes.len() > 4 {
        bytes[4..].to_vec()
    } else {
        Vec::new()
    };

    let mut buffers = get_reassembly_buffers().lock().unwrap();
    let state = buffers.entry(device_id.to_string()).or_insert_with(|| ConnectionState {
        fragments: HashMap::new(),
        total_fragments: Some(total),
    });

    state.fragments.insert(index, payload);

    if state.fragments.len() == total as usize {
        let mut sorted_fragments = Vec::new();
        for i in 0..total {
            if let Some(frag_payload) = state.fragments.get(&i) {
                let h = GattFragmentHeader::new(total, i)?;
                let mut full_frag = h.to_bytes().to_vec();
                full_frag.extend_from_slice(frag_payload);
                sorted_fragments.push(full_frag);
            } else {
                return Err("Missing fragment in buffer".into());
            }
        }

        buffers.remove(device_id);
        drop(buffers);

        let message = GattReassembler::reassemble(&sorted_fragments)
            .map_err(|e| format!("Reassembly failed: {:?}", e))?;

        crate::ble_mesh::handle_incoming_ble_payload(core, ui_tx, &message);
    }

    Ok(())
}
