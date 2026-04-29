use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMode {
    Normal,
    Cautious,
    Paranoid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MeshSettings {
    pub relay_enabled: bool,
    pub max_relay_budget: u32,
    pub battery_floor: u8,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
    pub wifi_direct_enabled: bool,
    pub internet_enabled: bool,
    pub discovery_mode: DiscoveryMode,
    pub onion_routing: bool,
    pub cover_traffic_enabled: bool,
    pub message_padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
    pub notifications_enabled: bool,
    pub notify_dm_enabled: bool,
    pub notify_dm_request_enabled: bool,
    pub notify_dm_in_foreground: bool,
    pub notify_dm_request_in_foreground: bool,
    pub sound_enabled: bool,
    pub badge_enabled: bool,
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: true,
            max_relay_budget: 200,
            battery_floor: 20,
            ble_enabled: true,
            wifi_aware_enabled: true,
            wifi_direct_enabled: true,
            internet_enabled: true,
            discovery_mode: DiscoveryMode::Normal,
            onion_routing: false,
            cover_traffic_enabled: false,
            message_padding_enabled: false,
            timing_obfuscation_enabled: false,
            notifications_enabled: crate::notification_defaults::notifications_enabled(),
            notify_dm_enabled: crate::notification_defaults::notify_dm_enabled(),
            notify_dm_request_enabled: crate::notification_defaults::notify_dm_request_enabled(),
            notify_dm_in_foreground: crate::notification_defaults::notify_dm_in_foreground(),
            notify_dm_request_in_foreground:
                crate::notification_defaults::notify_dm_request_in_foreground(),
            sound_enabled: crate::notification_defaults::sound_enabled(),
            badge_enabled: crate::notification_defaults::badge_enabled(),
        }
    }
}
