// scmessenger-wasm — WebAssembly notification management for browser environments

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Notification permission state
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotificationPermission {
    /// Permission not yet requested
    Default,
    /// Permission granted by user
    Granted,
    /// Permission denied by user
    Denied,
}

impl From<String> for NotificationPermission {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "granted" => NotificationPermission::Granted,
            "denied" => NotificationPermission::Denied,
            _ => NotificationPermission::Default,
        }
    }
}

impl From<NotificationPermission> for String {
    fn from(p: NotificationPermission) -> Self {
        match p {
            NotificationPermission::Default => "default".to_string(),
            NotificationPermission::Granted => "granted".to_string(),
            NotificationPermission::Denied => "denied".to_string(),
        }
    }
}

/// Browser type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Unknown,
}

impl Default for BrowserType {
    fn default() -> Self {
        BrowserType::Unknown
    }
}

/// Browser notification options with browser-specific configurations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserNotificationOptions {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub vibrate: Option<Vec<u32>>,
    pub tag: Option<String>,
    pub renotify: Option<bool>,
    pub require_interaction: Option<bool>,
}

impl Default for BrowserNotificationOptions {
    fn default() -> Self {
        Self {
            title: "SCMessenger".to_string(),
            body: "New message received".to_string(),
            icon: Some("/icon.png".to_string()),
            badge: Some("/badge.png".to_string()),
            vibrate: Some(vec![200, 100, 200]),
            tag: None,
            renotify: None,
            require_interaction: None,
        }
    }
}

/// Notification manager for WASM browser environment
#[wasm_bindgen]
pub struct NotificationManager {
    permission: Rc<RefCell<NotificationPermission>>,
}

#[wasm_bindgen]
impl NotificationManager {
    /// Create a new notification manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Check initial permission state
        let permission = if let Some(settings) = get_notification_settings() {
            NotificationPermission::from(settings)
        } else {
            NotificationPermission::Default
        };

        Self {
            permission: Rc::new(RefCell::new(permission)),
        }
    }

    /// Request notification permission from the user
    #[wasm_bindgen(js_name = requestPermission)]
    pub async fn request_permission(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            // Use web_sys directly for browser APIs
            use wasm_bindgen::JsCast;
            use web_sys::{window, Notification};

            let window = match window() {
                Some(w) => w,
                None => return false,
            };

            // Check if notifications are supported
            if !Notification::supported() {
                *self.permission.borrow_mut() = NotificationPermission::Denied;
                return false;
            }

            // Check current permission
            if Notification::permission() == NotificationPermission::Granted.into() {
                *self.permission.borrow_mut() = NotificationPermission::Granted;
                return true;
            }

            // Request permission
            match Notification::request_permission().await {
                Ok(permission) => {
                    let granted = permission == NotificationPermission::Granted.into();
                    if granted {
                        *self.permission.borrow_mut() = NotificationPermission::Granted;
                        save_notification_settings("granted");
                    } else {
                        *self.permission.borrow_mut() = NotificationPermission::Denied;
                        save_notification_settings("denied");
                    }
                    granted
                }
                Err(_) => {
                    *self.permission.borrow_mut() = NotificationPermission::Denied;
                    save_notification_settings("denied");
                    false
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }

    /// Check if notification permission is granted
    #[wasm_bindgen(js_name = isPermissionGranted)]
    pub fn is_permission_granted(&self) -> bool {
        matches!(*self.permission.borrow(), NotificationPermission::Granted)
    }

    /// Check if notifications are supported in the current browser
    #[wasm_bindgen(js_name = isSupported)]
    pub fn is_supported(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::Notification;
            Notification::supported()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }

    /// Show a notification with the given options
    #[wasm_bindgen(js_name = showNotification)]
    pub async fn show_notification(&self, title: String, body: String, options: JsValue) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::{window, Notification, NotificationOptions};

            // Parse default options
            let _ = options; // placeholder - would parse if needed

            let notification_options = NotificationOptions::new();
            notification_options.set_body(body.as_str());

            if let Some(icon) = BrowserNotificationOptions::default().icon {
                notification_options.set_icon(icon.as_str());
            }

            if let Ok(notification) = Notification::new_with_options(title.as_str(), &notification_options) {
                // Set click handler
                if let Some(win) = window() {
                    let notification_clone = notification.clone();
                    let click_handler = Closure::wrap(Box::new(move || {
                        console_log!("Notification clicked: {}", title);
                        win.focus();
                    }) as Box<dyn FnMut()>);

                    notification_clone.set_onclick(Some(click_handler.as_ref().unchecked_ref()));
                    click_handler.forget();
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            console_log("Notification not supported outside browser");
        }
    }

    /// Close all notifications (placeholder for tracking)
    #[wasm_bindgen(js_name = closeAllNotifications)]
    pub fn close_all_notifications(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            console_log("closeAllNotifications: Tracking notifications in real implementation");
        }
    }

    /// Get the current notification permission
    #[wasm_bindgen(js_name = getPermission)]
    pub fn get_permission(&self) -> String {
        (*self.permission.borrow()).into()
    }

    /// Detect the current browser type
    #[wasm_bindgen(js_name = detectBrowser)]
    pub fn detect_browser(&self) -> String {
        BrowserType::from_user_agent(&get_user_agent()).to_string()
    }

    /// Get browser-specific notification options as JSON
    #[wasm_bindgen(js_name = getBrowserOptions)]
    pub fn get_browser_options(&self) -> JsValue {
        let browser = BrowserType::from_user_agent(&get_user_agent());
        let mut options = BrowserNotificationOptions::default();

        // Apply browser-specific settings
        match browser {
            BrowserType::Chrome => {
                options.require_interaction = Some(false);
                options.vibrate = Some(vec![200, 100, 200]);
            }
            BrowserType::Firefox => {
                options.require_interaction = Some(true);
                options.tag = Some("scmessenger".to_string());
            }
            BrowserType::Safari => {
                options.tag = Some("scmessenger".to_string());
                options.renotify = Some(true);
            }
            _ => {}
        }

        serde_wasm_bindgen::to_value(&options).unwrap_or(JsValue::UNDEFINED)
    }

    /// Show permission guidance UI when permission is denied
    #[wasm_bindgen(js_name = showPermissionGuidance)]
    pub fn show_permission_guidance(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            let window = match window() {
                Some(w) => w,
                None => return,
            };

            // Build HTML for guidance overlay using string concatenation
            let guidance_html = format!(
                r#"<div id="scm_msg_guide" style="position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); z-index: 9999; max-width: 400px;"><h3 style="margin-top: 0;">Notifications Disabled</h3><p style="margin-bottom: 1rem;">Please enable notifications in your browser settings to receive messages.</p><div style="display: flex; gap: 10px;"><a id="open_settings_btn" href="#" style="flex: 1; padding: 10px 20px; background: #007bff; color: white; text-decoration: none; border-radius: 4px; text-align: center;">Open Settings</a><button id="close_guide_btn" style="flex: 1; padding: 10px 20px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;">Cancel</button></div></div>"#
            );

            if let Ok(div) = window.document().unwrap().create_element("div") {
                div.set_inner_html(&guidance_html);

                if let Some(body) = window.document().unwrap().body() {
                    body.append_child(&div).ok();

                    // Setup event handlers
                    if let Ok(Some(link)) = div.query_selector("#open_settings_btn") {
                        link.set_attribute("href", "javascript:void(0);").ok();
                    }

                    if let Ok(Some(button)) = div.query_selector("#close-guide-btn") {
                        let div_clone = div.clone();
                        let close_cb = Closure::wrap(Box::new(move || {
                            div_clone.parent_node().map(|p| p.remove_child(&div_clone));
                        }) as Box<dyn FnMut()>);

                        button.set_onclick(Some(close_cb.as_ref().unchecked_ref()));
                        close_cb.forget();
                    }
                }
            }
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn get_user_agent() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        window()
            .and_then(|w| w.navigator().ok())
            .and_then(|n| n.user_agent().ok())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "Unknown".to_string()
    }
}

fn get_notification_settings() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        window()
            .and_then(|w| w.local_storage().ok())
            .and_then(|s| s.get_item("notificationPermission").ok())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn save_notification_settings(_state: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()) {
            storage.set_item("notificationPermission", _state).ok();
        }
    }
}

fn console_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&JsValue::from_str(msg));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", msg);
    }
}

impl BrowserType {
    fn from_user_agent(ua: &str) -> Self {
        let ua_lower = ua.to_lowercase();

        if ua_lower.contains("edg") || ua_lower.contains("edge") {
            BrowserType::Edge
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            BrowserType::Safari
        } else if ua_lower.contains("firefox") || ua_lower.contains("fxios") {
            BrowserType::Firefox
        } else if ua_lower.contains("chrome") || ua_lower.contains("chromium") {
            BrowserType::Chrome
        } else {
            BrowserType::Unknown
        }
    }
}

impl std::fmt::Display for BrowserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserType::Chrome => write!(f, "Chrome"),
            BrowserType::Firefox => write!(f, "Firefox"),
            BrowserType::Safari => write!(f, "Safari"),
            BrowserType::Edge => write!(f, "Edge"),
            BrowserType::Unknown => write!(f, "Unknown"),
        }
    }
}
