use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the current system time as milliseconds since the Unix epoch.
/// If the system clock is set before the Unix epoch (1970-01-01),
/// this function clamps to 0 rather than panicking.
pub fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

/// Returns the current system time as seconds since the Unix epoch.
/// If the system clock is set before the Unix epoch (1970-01-01),
/// this function clamps to 0 rather than panicking.
pub fn unix_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}
