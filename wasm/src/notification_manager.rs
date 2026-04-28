// scmessenger-wasm — WebAssembly notification management for browser environments
// Simplified version using direct JavaScript API calls via js_sys

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{window, Storage, Node, Element, HtmlElement};

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
            // Check if Notification API is available
            let notification_available = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("Notification"),
            )
            .ok()
            .map(|v| v.is_object())
            .unwrap_or(false);

            if !notification_available {
                *self.permission.borrow_mut() = NotificationPermission::Denied;
                return false;
            }

            // Get the notification permission using navigator.permission API
            let _window = match window() {
                Some(w) => w,
                None => {
                    *self.permission.borrow_mut() = NotificationPermission::Default;
                    return false;
                }
            };

            // Get navigator.permission()
            let permission_result = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("navigator"),
            )
            .ok();

            if let Some(permission_obj) = permission_result {
                if permission_obj.is_object() {
                    let permission_fn = js_sys::Reflect::get(
                        &permission_obj,
                        &JsValue::from_str("permission"),
                    )
                    .ok();

                    if let Some(req_fn) = permission_fn {
                        if req_fn.is_function() {
                            let req_fn: js_sys::Function = req_fn.unchecked_into();
                            // Call navigator.permission() and await
                            let promise = js_sys::Promise::from(js_sys::Reflect::apply(
                                &req_fn,
                                &JsValue::UNDEFINED,
                                &js_sys::Array::new(),
                            ).unwrap());
                            let js_future = wasm_bindgen_futures::JsFuture::from(promise);
                            if let Ok(permission_val) = js_future.await {
                                let permission_str = permission_val.as_string().unwrap_or_default();
                                if permission_str == "granted" {
                                    *self.permission.borrow_mut() = NotificationPermission::Granted;
                                    save_notification_settings("granted");
                                    return true;
                                }
                                if permission_str == "denied" {
                                    *self.permission.borrow_mut() = NotificationPermission::Denied;
                                    save_notification_settings("denied");
                                    return false;
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: Request permission using window.Notification.requestPermission()
            let notification_obj = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("Notification"),
            )
            .ok();

            if let Some(notif_obj) = notification_obj {
                if notif_obj.is_object() {
                    let req_fn = js_sys::Reflect::get(
                        &notif_obj,
                        &JsValue::from_str("requestPermission"),
                    )
                    .ok();

                    if let Some(request_fn) = req_fn {
                        if request_fn.is_function() {
                            let request_fn: js_sys::Function = request_fn.unchecked_into();
                            let promise = js_sys::Promise::from(js_sys::Reflect::apply(
                                &request_fn,
                                &JsValue::UNDEFINED,
                                &js_sys::Array::new(),
                            ).unwrap());
                            let js_future = wasm_bindgen_futures::JsFuture::from(promise);
                            if let Ok(permission_val) = js_future.await {
                                let permission_str = permission_val.as_string().unwrap_or_default();
                                let granted = permission_str == "granted";
                                if granted {
                                    *self.permission.borrow_mut() = NotificationPermission::Granted;
                                    save_notification_settings("granted");
                                } else {
                                    *self.permission.borrow_mut() = NotificationPermission::Denied;
                                    save_notification_settings("denied");
                                }
                                return granted;
                            }
                        }
                    }
                }
            }

            false
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
            // Check if Notification constructor is available
            let notification_available = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("Notification"),
            )
            .ok()
            .map(|v| v.is_object())
            .unwrap_or(false);
            notification_available
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }

    /// Show a notification with the given options
    #[wasm_bindgen(js_name = showNotification)]
    pub async fn show_notification(&self, title: String, body: String, _options: JsValue) {
        #[cfg(target_arch = "wasm32")]
        {
            // Use the browser's Notification API directly
            let notification_result = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("Notification"),
            )
            .ok();

            if let Some(notification_obj) = notification_result {
                if notification_obj.is_object() {
                    let notification = js_sys::Reflect::construct(
                        &notification_obj.unchecked_into::<js_sys::Function>(),
                        &js_sys::Array::from_iter([
                            JsValue::from_str(title.as_str()),
                            JsValue::from_str(body.as_str()),
                        ]),
                    ).unwrap_or(JsValue::UNDEFINED);

                    if !notification.is_undefined() {
                        // Set click handler
                        if let Some(win) = web_sys::window() {
                            let notification_clone = notification.clone();
                            let click_handler = Closure::wrap(Box::new(move || {
                                web_sys::console::log_1(&JsValue::from_str(&format!("Notification clicked: {}", title)));
                                let _ = win.focus();
                            }) as Box<dyn FnMut()>);

                            let _ = js_sys::Reflect::set(
                                &notification_clone,
                                &JsValue::from_str("onclick"),
                                click_handler.as_ref().unchecked_ref(),
                            );
                            click_handler.forget();
                        }
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            web_sys::console::log_1(&JsValue::from_str("Notification not supported outside browser"));
        }
    }

    /// Close all notifications (placeholder for tracking)
    #[wasm_bindgen(js_name = closeAllNotifications)]
    pub fn close_all_notifications(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&JsValue::from_str("closeAllNotifications: Tracking notifications in real implementation"));
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
            let window = match window() {
                Some(w) => w,
                None => return,
            };

            let document = match window.document() {
                Some(d) => d,
                None => return,
            };

            // Build HTML for guidance overlay using string concatenation
            let guidance_html = String::from("<div id=\"scm_msg_guide\" style=\"position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); z-index: 9999; max-width: 400px;\"><h3 style=\"margin-top: 0;\">Notifications Disabled</h3><p style=\"margin-bottom: 1rem;\">Please enable notifications in your browser settings to receive messages.</p><div style=\"display: flex; gap: 10px;\"><a id=\"open_settings_btn\" href=\"#\" style=\"flex: 1; padding: 10px 20px; background: #007bff; color: white; text-decoration: none; border-radius: 4px; text-align: center;\">Open Settings</a><button id=\"close_guide_btn\" style=\"flex: 1; padding: 10px 20px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;\">Cancel</button></div></div>");

            // Create the div element using JavaScript
            let div = match js_sys::Reflect::get(&document, &JsValue::from_str("createElement")) {
                Ok(create_element_fn) if create_element_fn.is_function() => {
                    let create_element_fn: js_sys::Function = create_element_fn.unchecked_into();
                    let result = js_sys::Reflect::apply(&create_element_fn, &document, &js_sys::Array::from_iter([JsValue::from("div")]));
                    match result {
                        Ok(div) => div,
                        Err(_) => return,
                    }
                }
                _ => return,
            };

            // Set inner HTML
            let _ = js_sys::Reflect::set(&div, &JsValue::from_str("innerHTML"), &JsValue::from_str(&guidance_html));

            // Convert to Node for append_child
            let div_node: &Node = div.unchecked_ref();
            if let Some(body) = document.body() {
                let _ = body.append_child(div_node);
            }

            // Convert to Element for query_selector and set_attribute
            let div: &Element = div.unchecked_ref();
            if let Ok(Some(link)) = div.query_selector("#open_settings_btn") {
                let _ = link.set_attribute("href", "javascript:void(0)");
            }

            if let Ok(Some(button)) = div.query_selector("#close_guide_btn") {
                let div_clone = div.clone();
                let close_cb = Closure::wrap(Box::new(move || {
                    div_clone.parent_node().map(|p: Node| p.remove_child(div_clone.unchecked_ref()));
                }) as Box<dyn FnMut()>);

                // Convert button to HtmlElement for set_onclick
                let button: &HtmlElement = button.unchecked_ref();
                let _ = button.set_onclick(Some(close_cb.as_ref().unchecked_ref()));
                close_cb.forget();
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
        let window = match window() {
            Some(w) => w,
            None => return "Unknown".to_string(),
        };

        let navigator = match js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) {
            Ok(n) => n,
            Err(_) => return "Unknown".to_string(),
        };

        let user_agent = match js_sys::Reflect::get(&navigator, &JsValue::from_str("userAgent")) {
            Ok(ua) => ua.as_string(),
            Err(_) => None,
        };

        user_agent.unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "Unknown".to_string()
    }
}

fn get_notification_settings() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = window()?;
        let storage_value = match js_sys::Reflect::get(&window, &JsValue::from_str("localStorage")) {
            Ok(s) => s,
            Err(_) => return None,
        };
        let storage: Storage = storage_value.unchecked_into();
        match storage.get_item("notificationPermission") {
            Ok(Some(js_val)) => Some(js_val),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn save_notification_settings(_state: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let window = match window() {
            Some(w) => w,
            None => return,
        };

        let storage_value = match js_sys::Reflect::get(&window, &JsValue::from_str("localStorage")) {
            Ok(s) => s,
            Err(_) => return,
        };
        let storage: Storage = storage_value.unchecked_into();
        let _ = storage.set_item("notificationPermission", _state);
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
