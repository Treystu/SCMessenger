// BlueZ BLE D-Bus integration for Linux desktop.
//
// Communicates with BlueZ (the Linux Bluetooth stack) via D-Bus
// using the zbus crate (pure Rust, no C dependencies).
//
// D-Bus interfaces implemented:
//   - org.bluez.Adapter1 — adapter power/scan control
//   - org.bluez.Device1 — discovered device info
//   - org.freedesktop.DBus.ObjectManager — device discovery
//
// Only compiled on Linux: `#[cfg(target_os = "linux")]`.

use crate::{BleAdapterInfo, BleAdapterState, BlePeer};

/// Default BlueZ D-Bus service name.
const BLUEZ_SERVICE: &str = "org.bluez";

/// Default adapter object path.
const DEFAULT_ADAPTER_PATH: &str = "/org/bluez/hci0";

/// D-Bus interface names.
const IF_ADAPTER: &str = "org.bluez.Adapter1";
const IF_DEVICE: &str = "org.bluez.Device1";
const IF_OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";
const IF_PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// Get information about the default BLE adapter.
///
/// Connects to the system D-Bus and queries org.bluez.Adapter1
/// for the default adapter at /org/bluez/hci0.
pub async fn get_adapter_info() -> Result<BleAdapterInfo, String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    let proxy = zbus::Proxy::new(
        &connection,
        BLUEZ_SERVICE,
        DEFAULT_ADAPTER_PATH,
        IF_PROPERTIES,
    )
    .await
    .map_err(|e| format!("Failed to create D-Bus proxy: {e}"))?;

    // Read Powered property
    let powered: bool = read_property(&proxy, "Powered").await?;

    // Read Address property
    let address: String = read_property(&proxy, "Address").await?;

    // Read Name property
    let name: String = read_property(&proxy, "Name").await?;

    // Read Discovering property
    let scanning: bool = read_property(&proxy, "Discovering").await?;

    // Read UUIDs to check if advertising is active
    let _uuids: Vec<String> = read_property(&proxy, "UUIDs").await.unwrap_or_default();
    // If LEAdvertisingManager is registered, advertising is available
    let advertising = false; // Would need to check Advertising property on LEAdvertisingManager1

    let state = if !powered {
        BleAdapterState::PoweredOff
    } else if scanning {
        BleAdapterState::Scanning
    } else {
        BleAdapterState::PoweredOn
    };

    Ok(BleAdapterInfo {
        dbus_path: DEFAULT_ADAPTER_PATH.to_string(),
        name,
        address,
        powered,
        scanning,
        advertising,
        state,
    })
}

/// Start BLE scan on the default adapter.
pub async fn start_scan() -> Result<(), String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    let proxy = zbus::Proxy::new(&connection, BLUEZ_SERVICE, DEFAULT_ADAPTER_PATH, IF_ADAPTER)
        .await
        .map_err(|e| format!("Failed to create D-Bus proxy for adapter: {e}"))?;

    proxy
        .call_method("StartDiscovery", &())
        .await
        .map_err(|e| format!("BlueZ StartDiscovery failed: {e}"))?;

    Ok(())
}

/// Stop BLE scan on the default adapter.
pub async fn stop_scan() -> Result<(), String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    let proxy = zbus::Proxy::new(&connection, BLUEZ_SERVICE, DEFAULT_ADAPTER_PATH, IF_ADAPTER)
        .await
        .map_err(|e| format!("Failed to create D-Bus proxy for adapter: {e}"))?;

    proxy
        .call_method("StopDiscovery", &())
        .await
        .map_err(|e| format!("BlueZ StopDiscovery failed: {e}"))?;

    Ok(())
}

/// List discovered BLE peers (devices known to BlueZ).
///
/// Returns devices that BlueZ currently tracks, filtering for
/// potential SCMessenger peers by checking service UUIDs.
pub async fn list_discovered_peers() -> Result<Vec<BlePeer>, String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    // Use ObjectManager to list all managed objects
    let proxy = zbus::Proxy::new(&connection, BLUEZ_SERVICE, "/", IF_OBJECT_MANAGER)
        .await
        .map_err(|e| format!("Failed to create ObjectManager proxy: {e}"))?;

    let (objects,): (
        std::collections::HashMap<
            zbus::zvariant::OwnedObjectPath,
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            >,
        >,
    ) = proxy
        .call_method("GetManagedObjects", &())
        .await
        .map_err(|e| format!("GetManagedObjects failed: {e}"))?
        .body()
        .deserialize()
        .map_err(|e| format!("Failed to deserialize GetManagedObjects reply: {e}"))?;

    let mut peers = Vec::new();

    // SCMessenger BLE service UUID (16-bit or 128-bit)
    const SCM_SERVICE_UUID_16: &str = "0000fe9f-0000-1000-8000-00805f9b34fb"; // Example Nordic NUS-like

    for (path, interfaces) in &objects {
        if let Some(device_props) = interfaces.get(IF_DEVICE) {
            let peer_id = path.to_string();

            // Extract properties
            let display_name = device_props
                .get("Name")
                .and_then(|v| v.downcast_ref::<String>().ok());

            let rssi = device_props
                .get("RSSI")
                .and_then(|v| v.downcast_ref::<i16>().ok())
                .unwrap_or(-100);

            let _is_connected = device_props
                .get("Connected")
                .and_then(|v| v.downcast_ref::<bool>().ok())
                .unwrap_or(false);

            // Check if this device advertises an SCMessenger UUID
            let uuids: Vec<String> = device_props
                .get("UUIDs")
                .and_then(|v| {
                    let array: zbus::zvariant::Array = v.downcast_ref().ok()?;
                    array
                        .iter()
                        .map(|item| String::try_from(item).ok())
                        .collect::<Option<Vec<String>>>()
                })
                .unwrap_or_default();

            let is_scmessenger = uuids.iter().any(|u| {
                u.eq_ignore_ascii_case(SCM_SERVICE_UUID_16) || u.contains("fe9f")
                // Match the short UUID
            });

            let last_seen = web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            peers.push(BlePeer {
                peer_id,
                display_name,
                rssi,
                is_scmessenger_node: is_scmessenger,
                last_seen_secs: last_seen,
            });
        }
    }

    // Sort: SCMessenger nodes first, then by signal strength
    peers.sort_by(|a, b| {
        b.is_scmessenger_node
            .cmp(&a.is_scmessenger_node)
            .then_with(|| b.rssi.cmp(&a.rssi))
    });

    Ok(peers)
}

/// Power the BLE adapter on.
pub async fn power_on_adapter() -> Result<(), String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    let proxy = zbus::Proxy::new(
        &connection,
        BLUEZ_SERVICE,
        DEFAULT_ADAPTER_PATH,
        IF_PROPERTIES,
    )
    .await
    .map_err(|e| format!("Failed to create D-Bus proxy: {e}"))?;

    proxy
        .call_method(
            "Set",
            &(IF_ADAPTER, "Powered", zbus::zvariant::Value::Bool(true)),
        )
        .await
        .map_err(|e| format!("BlueZ Power On failed: {e}"))?;

    Ok(())
}

/// Power the BLE adapter off.
pub async fn power_off_adapter() -> Result<(), String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {e}"))?;

    let proxy = zbus::Proxy::new(
        &connection,
        BLUEZ_SERVICE,
        DEFAULT_ADAPTER_PATH,
        IF_PROPERTIES,
    )
    .await
    .map_err(|e| format!("Failed to create D-Bus proxy: {e}"))?;

    proxy
        .call_method(
            "Set",
            &(IF_ADAPTER, "Powered", zbus::zvariant::Value::Bool(false)),
        )
        .await
        .map_err(|e| format!("BlueZ Power Off failed: {e}"))?;

    Ok(())
}

// ===========================================================================
// Helpers
// ===========================================================================

async fn read_property<'a, T>(proxy: &zbus::Proxy<'a>, property_name: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned + zbus::zvariant::Type + 'static,
{
    proxy
        .call_method("Get", &(IF_ADAPTER, property_name))
        .await
        .map_err(|e| format!("Failed to read property '{property_name}': {e}"))?
        .body()
        .deserialize::<T>()
        .map_err(|e| format!("Failed to deserialize property '{property_name}': {e}"))
}

/// Synchronous wrapper for get_adapter_info (for UniFFI boundary).
pub fn get_adapter_info_sync() -> Result<BleAdapterInfo, String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Tokio runtime creation failed: {e}"))?;
        rt.block_on(get_adapter_info())
    })
    .join()
    .map_err(|_| "BLE thread panicked".to_string())?
}

/// Synchronous wrapper for list_discovered_peers.
pub fn list_discovered_peers_sync() -> Result<Vec<BlePeer>, String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Tokio runtime creation failed: {e}"))?;
        rt.block_on(list_discovered_peers())
    })
    .join()
    .map_err(|_| "BLE thread panicked".to_string())?
}

/// Synchronous wrapper for start_scan.
pub fn start_scan_sync() -> Result<(), String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Tokio runtime creation failed: {e}"))?;
        rt.block_on(start_scan())
    })
    .join()
    .map_err(|_| "BLE thread panicked".to_string())?
}

/// Synchronous wrapper for stop_scan.
pub fn stop_scan_sync() -> Result<(), String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Tokio runtime creation failed: {e}"))?;
        rt.block_on(stop_scan())
    })
    .join()
    .map_err(|_| "BLE thread panicked".to_string())?
}
