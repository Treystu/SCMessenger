// Desktop notification module.
//
// On Linux: sends notifications via the D-Bus Desktop Notifications spec
//           (org.freedesktop.Notifications).
// On other targets: stub implementation that returns "unsupported".

use crate::NotificationResult;

/// Send a desktop notification using the Freedesktop Notifications D-Bus interface.
///
/// # Arguments
/// * `title` — Notification title
/// * `body` — Notification body
/// * `urgency` — Notification urgency level
///
/// # Returns
/// NotificationResult with compositor-assigned ID, or error.
pub fn send_notification(
    title: String,
    body: String,
    urgency: crate::NotificationUrgency,
) -> NotificationResult {
    #[cfg(target_os = "linux")]
    {
        // On Linux, we delegate to the D-Bus implementation at runtime.
        // This synchronous wrapper is for the UniFFI boundary.
        // The actual D-Bus call is async internally but we block for the FFI boundary.
        match send_notification_dbus(&title, &body, &urgency) {
            Ok(id) => NotificationResult {
                notification_id: id,
                shown: true,
                error_message: None,
            },
            Err(e) => NotificationResult {
                notification_id: 0,
                shown: false,
                error_message: Some(e),
            },
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        NotificationResult {
            notification_id: 0,
            shown: false,
            error_message: Some("Desktop notifications are only supported on Linux".to_string()),
        }
    }
}

#[cfg(target_os = "linux")]
fn send_notification_dbus(
    title: &str,
    body: &str,
    _urgency: &crate::NotificationUrgency,
) -> Result<u32, String> {
    // Use a dedicated async runtime thread for the D-Bus call.
    // This avoids interfering with any existing tokio runtime the caller may have.
    // `title`/`body` are owned before crossing the thread boundary since
    // `std::thread::spawn` requires a `'static` closure.
    let title = title.to_string();
    let body = body.to_string();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;

        rt.block_on(async { dbus_notify(&title, &body).await })
    })
    .join()
    .map_err(|_| "D-Bus notification thread panicked".to_string())?
}

#[cfg(target_os = "linux")]
async fn dbus_notify(title: &str, body: &str) -> Result<u32, String> {
    use zbus::Connection;

    let connection = Connection::session()
        .await
        .map_err(|e| format!("D-Bus session connection failed: {e}"))?;

    let reply = connection
        .call_method(
            Some("org.freedesktop.Notifications"), // destination
            "/org/freedesktop/Notifications",      // path
            Some("org.freedesktop.Notifications"), // interface
            "Notify",                              // method
            &(
                "scmessenger", // app_name
                0u32,          // replaces_id (0 = new notification)
                "",            // app_icon
                title,
                body,
                Vec::<String>::new(), // actions
                std::collections::HashMap::<&str, zbus::zvariant::Value<'_>>::new(), // hints
                5000i32,              // expire_timeout (ms, 5s)
            ),
        )
        .await
        .map_err(|e| format!("D-Bus Notify call failed: {e}"))?;

    let (notification_id,): (u32,) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Failed to deserialize D-Bus reply: {e}"))?;

    Ok(notification_id)
}
