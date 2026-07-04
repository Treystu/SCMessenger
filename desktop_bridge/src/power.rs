// Power management module.
//
// Provides power state detection and management hints for the desktop session.
// Detects battery/AC status and suspend/resume events.
//
// On Linux: reads from /sys/class/power_supply/ or UPower D-Bus.
// On other targets: returns AC profile with unknown battery.

use crate::PowerState;

/// Detect the current power state.
///
/// # Returns
/// PowerState with the detected profile, battery percentage, and source.
pub fn detect_power_state() -> PowerState {
    #[cfg(target_os = "linux")]
    {
        detect_linux_power_state()
    }

    #[cfg(not(target_os = "linux"))]
    {
        PowerState {
            profile: crate::PowerProfile::AC,
            battery_pct: 255,
            on_battery: false,
            idle_inhibited: false,
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_power_state() -> PowerState {
    // Try to read battery info from /sys/class/power_supply/
    let mut on_battery = false;
    let mut battery_pct: u8 = 255;

    // Check if any power supply is a battery and currently discharging
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let path = entry.path();
            let type_path = path.join("type");
            if let Ok(ps_type) = std::fs::read_to_string(&type_path) {
                let ps_type = ps_type.trim();
                if ps_type == "Battery" {
                    // Read status (Charging/Discharging/Full/Unknown)
                    let status_path = path.join("status");
                    if let Ok(status) = std::fs::read_to_string(&status_path) {
                        let status = status.trim();
                        if status == "Discharging" {
                            on_battery = true;
                        }
                    }

                    // Read capacity (percentage)
                    let capacity_path = path.join("capacity");
                    if let Ok(capacity_str) = std::fs::read_to_string(&capacity_path) {
                        if let Ok(cap) = capacity_str.trim().parse::<u8>() {
                            battery_pct = cap;
                        }
                    }
                    // Use the first battery found
                    break;
                }
            }
        }
    }

    // If no battery was found at all, assume AC (desktop machine)
    if !on_battery && battery_pct == 255 {
        battery_pct = 100;
    }

    let profile = if on_battery {
        crate::PowerProfile::Battery
    } else {
        crate::PowerProfile::AC
    };

    PowerState {
        profile,
        battery_pct,
        on_battery,
        idle_inhibited: false, // Set by desktop client via set_idle_inhibited
    }
}

/// Create a PowerState with the given profile (for suspend/resume events).
pub fn power_state_for_event(event: crate::PowerProfile) -> PowerState {
    let mut state = detect_power_state();
    state.profile = event;
    state
}

/// Set idle inhibition state.
/// Called by the desktop client when it acquires/releases an idle inhibitor.
pub fn set_idle_inhibited(state: &mut PowerState, inhibited: bool) {
    state.idle_inhibited = inhibited;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_state() {
        let state = detect_power_state();
        // Should always return a valid state (even on non-Linux in CI)
        assert!(state.battery_pct <= 100 || state.battery_pct == 255);
    }

    #[test]
    fn test_power_state_for_suspend() {
        let state = power_state_for_event(crate::PowerProfile::SuspendImminent);
        assert!(matches!(
            state.profile,
            crate::PowerProfile::SuspendImminent
        ));
    }

    #[test]
    fn test_power_state_for_resume() {
        let state = power_state_for_event(crate::PowerProfile::Resumed);
        assert!(matches!(state.profile, crate::PowerProfile::Resumed));
    }

    #[test]
    fn test_set_idle_inhibited() {
        let mut state = detect_power_state();
        set_idle_inhibited(&mut state, true);
        assert!(state.idle_inhibited);
        set_idle_inhibited(&mut state, false);
        assert!(!state.idle_inhibited);
    }
}
