//! Best-effort Bluetooth adapter discovery via btleplug (desktop CLI only).
//! Full GATT advertising/scanning and Drift→RPC proxy are follow-on work.

use btleplug::api::Manager as _;

/// Log whether the local Bluetooth stack exposes at least one adapter.
pub async fn probe_and_log() {
    #[cfg(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))]
    {
        match btleplug::platform::Manager::new().await {
            Ok(manager) => match manager.adapters().await {
                Ok(adapters) => {
                    tracing::info!(
                        "btleplug: acquired Bluetooth manager; {} adapter(s) visible",
                        adapters.len()
                    );
                    for a in adapters.iter().take(3) {
                        tracing::debug!("btleplug adapter: {:?}", a);
                    }
                }
                Err(e) => tracing::warn!("btleplug: failed to list adapters: {}", e),
            },
            Err(e) => tracing::warn!("btleplug: failed to create manager: {}", e),
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        tracing::debug!("btleplug: BLE probe skipped on this target OS");
    }
}
