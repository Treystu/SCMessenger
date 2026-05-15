//! Local CLI daemon WebSocket bridge (thin client).
//!
//! The browser opens `ws://127.0.0.1:<port>/ws` and exchanges JSON-RPC 2.0
//! frames as defined in `scmessenger_core::wasm_support::rpc`.
//!
//! ## Wire format
//!
//! **Request (browser → daemon):**
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"get_identity","params":{}}
//! ```
//!
//! **Response (daemon → browser):**
//! ```json
//! {"jsonrpc":"2.0","id":1,"result":{"identityId":"..."}}
//! ```
//!
//! **Notification (daemon → browser, server push, no `id`):**
//! ```json
//! {"jsonrpc":"2.0","method":"message_received","params":{"from":"...","content":"..."}}
//! ```

use scmessenger_core::wasm_support::rpc::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

// ── Request formatters ────────────────────────────────────────────────────

/// Build a `get_identity` JSON-RPC request string.
pub fn format_get_identity(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_identity".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

/// Build a `send_message` JSON-RPC request string.
pub fn format_send_message(
    id: impl serde::Serialize,
    recipient: &str,
    message: &str,
    msg_id: Option<&str>,
) -> Result<String, serde_json::Error> {
    let mut params = serde_json::json!({
        "recipient": recipient,
        "message": message,
    });
    if let Some(mid) = msg_id {
        params["id"] = serde_json::json!(mid);
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "send_message".into(),
        params,
    };
    serde_json::to_string(&req)
}

/// Build a `scan_peers` JSON-RPC request string.
pub fn format_scan_peers(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "scan_peers".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

/// Build a `get_topology` JSON-RPC request string.
pub fn format_get_topology(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_topology".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

// ── Contacts ──

/// Build a `get_contacts` JSON-RPC request string.
pub fn format_get_contacts(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_contacts".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

/// Build an `add_contact` JSON-RPC request string.
pub fn format_add_contact(
    id: impl serde::Serialize,
    peer_id: &str,
    nickname: Option<&str>,
) -> Result<String, serde_json::Error> {
    let mut params = serde_json::json!({"peer_id": peer_id});
    if let Some(n) = nickname {
        params["nickname"] = serde_json::json!(n);
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "add_contact".into(),
        params,
    };
    serde_json::to_string(&req)
}

/// Build a `remove_contact` JSON-RPC request string.
pub fn format_remove_contact(
    id: impl serde::Serialize,
    peer_id: &str,
) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "remove_contact".into(),
        params: serde_json::json!({"peer_id": peer_id}),
    };
    serde_json::to_string(&req)
}

// ── Settings ──

/// Build a `get_settings` JSON-RPC request string.
pub fn format_get_settings(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_settings".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

/// Build an `update_settings` JSON-RPC request string.
pub fn format_update_settings(
    id: impl serde::Serialize,
    key: &str,
    value: &serde_json::Value,
) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "update_settings".into(),
        params: serde_json::json!({"key": key, "value": value}),
    };
    serde_json::to_string(&req)
}

// ── History ──

/// Build a `get_history` JSON-RPC request string.
pub fn format_get_history(
    id: impl serde::Serialize,
    limit: Option<usize>,
) -> Result<String, serde_json::Error> {
    let mut params = serde_json::json!({});
    if let Some(n) = limit {
        params["limit"] = serde_json::json!(n);
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_history".into(),
        params,
    };
    serde_json::to_string(&req)
}

/// Build a `get_conversation` JSON-RPC request string.
pub fn format_get_conversation(
    id: impl serde::Serialize,
    peer_id: &str,
    limit: Option<usize>,
) -> Result<String, serde_json::Error> {
    let mut params = serde_json::json!({"peer_id": peer_id});
    if let Some(n) = limit {
        params["limit"] = serde_json::json!(n);
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "get_conversation".into(),
        params,
    };
    serde_json::to_string(&req)
}

/// Build a `clear_history` JSON-RPC request string.
pub fn format_clear_history(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "clear_history".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

// ── Blocking ──

/// Build a `list_blocked` JSON-RPC request string.
pub fn format_list_blocked(id: impl serde::Serialize) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "list_blocked".into(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req)
}

/// Build a `block_peer` JSON-RPC request string.
pub fn format_block_peer(
    id: impl serde::Serialize,
    peer_id: &str,
    reason: Option<&str>,
) -> Result<String, serde_json::Error> {
    let mut params = serde_json::json!({"peer_id": peer_id});
    if let Some(r) = reason {
        params["reason"] = serde_json::json!(r);
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "block_peer".into(),
        params,
    };
    serde_json::to_string(&req)
}

/// Build an `unblock_peer` JSON-RPC request string.
pub fn format_unblock_peer(
    id: impl serde::Serialize,
    peer_id: &str,
) -> Result<String, serde_json::Error> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(serde_json::to_value(id)?),
        method: "unblock_peer".into(),
        params: serde_json::json!({"peer_id": peer_id}),
    };
    serde_json::to_string(&req)
}

// ── Response / notification parsers ───────────────────────────────────────

/// Parse a JSON-RPC response from the daemon.
pub fn parse_response(s: &str) -> Result<JsonRpcResponse, serde_json::Error> {
    serde_json::from_str(s)
}

/// Parse a JSON-RPC notification (server push).
pub fn parse_notification(s: &str) -> Result<JsonRpcNotification, serde_json::Error> {
    serde_json::from_str(s)
}

// ── WebSocket bridge ──────────────────────────────────────────────────────

/// Callback invoked when a JSON-RPC response arrives matching a pending
/// request ID. The `id` value is the original request ID; `result` and
/// `error` correspond to the JSON-RPC response fields.
pub type ResponseCallback =
    Box<dyn Fn(serde_json::Value, Option<serde_json::Value>, Option<serde_json::Value>)>;

/// Callback invoked for each server-push notification.
pub type NotificationCallback = Box<dyn Fn(String, serde_json::Value)>;

/// Manages the browser WebSocket connection to the local CLI daemon.
///
/// ## Usage (JavaScript)
///
/// ```js
/// const bridge = DaemonBridge.new("ws://127.0.0.1:9000/ws");
/// bridge.on_notification((method, params) => {
///     if (method === "message_received") {
///         showNotification(params.from, params.content);
///     }
/// });
/// bridge.connect();
/// const resp = await bridge.request("get_identity", {});
/// console.log(resp.result);
/// ```
/// Connection state for the daemon bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(target_arch = "wasm32")]
pub enum BridgeState {
    /// Not yet connected
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Connected and operational
    Connected,
    /// Connection lost, reconnecting
    Reconnecting,
    /// Permanently closed
    Closed,
}

pub struct DaemonBridge {
    url: String,
    next_id: std::sync::atomic::AtomicU64,
    /// Pending requests keyed by JSON-RPC id, awaiting a response.
    #[cfg(target_arch = "wasm32")]
    pending: Rc<RefCell<std::collections::HashMap<u64, ResponseCallback>>>,
    /// Notification callback invoked on each server push.
    #[cfg(target_arch = "wasm32")]
    on_notification: Rc<RefCell<Option<NotificationCallback>>>,
    /// Live WebSocket handle (held so the browser object is not GC'd).
    #[cfg(target_arch = "wasm32")]
    socket: Rc<RefCell<Option<web_sys::WebSocket>>>,
    #[cfg(target_arch = "wasm32")]
    url_clone: String,
    /// Reconnection state and configuration
    #[cfg(target_arch = "wasm32")]
    reconnection_state: Rc<RefCell<BridgeState>>,
    /// Maximum reconnection attempts before giving up
    #[cfg(target_arch = "wasm32")]
    max_reconnect_attempts: std::sync::atomic::AtomicU32,
    /// Current reconnection attempt count
    #[cfg(target_arch = "wasm32")]
    reconnect_attempts: std::sync::atomic::AtomicU32,
    /// Reconnection interval in milliseconds (exponential backoff base)
    #[cfg(target_arch = "wasm32")]
    reconnect_interval_ms: std::sync::atomic::AtomicU64,
}

impl DaemonBridge {
    /// Create a new bridge targeting a daemon WebSocket URL.
    ///
    /// The URL must be a full `ws://127.0.0.1:<port>/ws` string. The
    /// connection is not opened until `connect()` is called.
    pub fn new(url: String) -> Self {
        Self {
            url: url.clone(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            #[cfg(target_arch = "wasm32")]
            pending: Rc::new(RefCell::new(std::collections::HashMap::new())),
            #[cfg(target_arch = "wasm32")]
            on_notification: Rc::new(RefCell::new(None)),
            #[cfg(target_arch = "wasm32")]
            socket: Rc::new(RefCell::new(None)),
            #[cfg(target_arch = "wasm32")]
            url_clone: url,
            #[cfg(target_arch = "wasm32")]
            reconnection_state: Rc::new(RefCell::new(BridgeState::Disconnected)),
            #[cfg(target_arch = "wasm32")]
            max_reconnect_attempts: std::sync::atomic::AtomicU32::new(5),
            #[cfg(target_arch = "wasm32")]
            reconnect_attempts: std::sync::atomic::AtomicU32::new(0),
            #[cfg(target_arch = "wasm32")]
            reconnect_interval_ms: std::sync::atomic::AtomicU64::new(1000),
        }
    }

    /// Configure maximum reconnection attempts.
    #[cfg(target_arch = "wasm32")]
    pub fn set_max_reconnect_attempts(&self, max: u32) {
        self.max_reconnect_attempts.store(max, std::sync::atomic::Ordering::SeqCst);
    }

    /// Configure base reconnection interval (milliseconds, exponential backoff).
    #[cfg(target_arch = "wasm32")]
    pub fn set_reconnect_interval_ms(&self, interval_ms: u64) {
        self.reconnect_interval_ms.store(interval_ms, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get current reconnection state.
    #[cfg(target_arch = "wasm32")]
    pub fn state(&self) -> BridgeState {
        *self.reconnection_state.borrow()
    }

    /// True if the bridge is connected and operational.
    #[cfg(target_arch = "wasm32")]
    pub fn is_connected(&self) -> bool {
        self.state() == BridgeState::Connected
    }

    /// True if the bridge is connected and operational (desktop stub).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_connected(&self) -> bool {
        false
    }

    /// Register a callback for server-push notifications.
    ///
    /// The callback receives `(method_name, params_object)` for each
    /// notification frame from the daemon. Only one callback is active at
    /// a time — calling this again replaces the previous one.
    #[cfg(target_arch = "wasm32")]
    pub fn on_notification(&self, cb: NotificationCallback) {
        *self.on_notification.borrow_mut() = Some(cb);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_notification(&self, _cb: NotificationCallback) {}

    /// Open the WebSocket connection to the daemon.
    ///
    /// Registers `onopen`, `onmessage`, `onerror`, and `onclose` event
    /// handlers. Inbound text frames are parsed as JSON-RPC responses or
    /// notifications and dispatched to the registered callbacks.
    /// On connection loss, automatically attempts reconnection with exponential backoff.
    #[cfg(target_arch = "wasm32")]
    pub fn connect(&self) -> Result<(), String> {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

        // Set connecting state
        *self.reconnection_state.borrow_mut() = BridgeState::Connecting;

        // Get reconnect config values to avoid borrowing issues in closures
        let reconnect_interval = self.reconnect_interval_ms.load(std::sync::atomic::Ordering::SeqCst);
        let max_reconnect_attempts = self.max_reconnect_attempts.load(std::sync::atomic::Ordering::SeqCst);

        let ws = WebSocket::new(&self.url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

        let state_for_handlers = Rc::clone(&self.reconnection_state);
        let socket_for_handlers = Rc::clone(&self.socket);
        let reconnect_attempts = Rc::clone(&self.reconnect_attempts);
        let max_attempts = self.max_reconnect_attempts.clone();
        let interval = self.reconnect_interval_ms.clone();
        let pending_for_handlers = Rc::clone(&self.pending);
        let notif_cb_for_handlers = Rc::clone(&self.on_notification);
        let url_clone_for_handlers = self.url_clone.clone();
        let url_for_log = self.url.clone();

        // ── onopen ──
        let onopen_url = url_for_log.clone();
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            tracing::info!("Daemon bridge connected to {}", onopen_url);
            *state_for_handlers.borrow_mut() = BridgeState::Connected;
            reconnect_attempts.store(0, std::sync::atomic::Ordering::SeqCst);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // ── onmessage ──
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let text = match event.data().as_string() {
                Some(s) => s,
                None => {
                    tracing::warn!("Daemon bridge received non-text frame; ignored");
                    return;
                }
            };

            // Try parsing as a JSON-RPC response (has `id`).
            if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&text) {
                if let Some(ref id_val) = resp.id {
                    if let Some(id_num) = id_val.as_u64() {
                        let cb = pending_for_handlers.borrow_mut().remove(&id_num);
                        if let Some(cb) = cb {
                            cb(
                                id_val.clone(),
                                resp.result,
                                resp.error.map(
                                    |e| serde_json::json!({"code": e.code, "message": e.message}),
                                ),
                            );
                            return;
                        }
                    }
                }
            }

            // Try parsing as a JSON-RPC notification (no `id`).
            if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&text) {
                if let Some(ref cb) = *notif_cb_for_handlers.borrow() {
                    cb(notif.method, notif.params);
                }
                return;
            }

            tracing::warn!(
                "Daemon bridge received unrecognized frame: {}",
                &text[..text.len().min(200)]
            );
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // ── onerror ──
        let onerror_url = url_for_log.clone();
        let onerror_reconnect_state = Rc::clone(&state_for_handlers);
        let onerror = Closure::wrap(Box::new(move |event: ErrorEvent| {
            let msg = event.message();
            tracing::error!("Daemon bridge error for {}: {}", onerror_url, msg);
            // Set reconnecting state to trigger reconnection on close
            *onerror_reconnect_state.borrow_mut() = BridgeState::Reconnecting;
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // ── onclose ──
        let onclose_url = url_for_log.clone();
        let onclose_reconnect_state = Rc::clone(&state_for_handlers);
        let onclose_socket = Rc::clone(&socket_for_handlers);
        let onclose_reconnect_attempts = Rc::clone(&reconnect_attempts);
        let onclose_max_attempts = max_attempts;
        let onclose_interval = interval;
        let onclose = Closure::wrap(Box::new(move |event: CloseEvent| {
            let code = event.code();
            let reason = event.reason();
            tracing::info!(
                "Daemon bridge closed for {}: code={} reason={}",
                onclose_url,
                code,
                reason
            );

            // Try to reconnect if not permanently closed and within attempts
            let current_state = *onclose_reconnect_state.borrow();
            if current_state != BridgeState::Closed {
                let attempts = onclose_reconnect_attempts.load(std::sync::atomic::Ordering::SeqCst);
                let max = onclose_max_attempts.load(std::sync::atomic::Ordering::SeqCst);

                if attempts < max {
                    let next_interval = onclose_interval.load(std::sync::atomic::Ordering::SeqCst);
                    let next_interval = next_interval * (2u64.pow(attempts)); // exponential backoff

                    *onclose_reconnect_state.borrow_mut() = BridgeState::Reconnecting;
                    onclose_reconnect_attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    // Schedule reconnection after delay
                    let url_for_reconnect = onclose_url.clone();
                    let socket_for_reconnect = Rc::clone(&onclose_socket);
                    let state_for_reconnect = Rc::clone(&onclose_reconnect_state);
                    let attempts_for_reconnect = onclose_reconnect_attempts.clone();
                    let max_for_reconnect = onclose_max_attempts.clone();
                    let interval_for_reconnect = onclose_interval.clone();
                    let pending_for_reconnect = Rc::clone(&pending_for_handlers);
                    let notif_cb_for_reconnect = Rc::clone(&notif_cb_for_handlers);

                    spawn_local(async move {
                        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
                            let timeout_id = wasm_bindgen::JsValue::from(
                                wasm_bindgen::JsValue::UNDEFINED,
                            );
                            // Use setTimeout via js_sys
                            let window = wasm_bindgen::JsValue::UNDEFINED;
                            let _ = wasm_bindgen::JsValue::from(
                                wasm_bindgen::JsCast::dyn_ref::<wasm_bindgen::JsCast>(&window)
                                    .unwrap()
                                    .dyn_into::<wasm_bindgen::js_sys::global::Window>()
                                    .unwrap(),
                            );
                        })
                        .map_err(|_| {
                            tracing::error!("Failed to schedule reconnection");
                        });

                        // Simple setTimeout via js_sys
                        let window = js_sys::global();
                        let cb = Closure::wrap(Box::new(move || {
                            tracing::info!(
                                "Attempting reconnection to {} (attempt {}/{})",
                                url_for_reconnect,
                                attempts + 1,
                                max
                            );
                            *state_for_reconnect.borrow_mut() = BridgeState::Connecting;

                            if let Ok(ws) = WebSocket::new(&url_for_reconnect) {
                                // onopen
                                let onopen_reconnect = Closure::wrap(Box::new(move |_| {
                                    tracing::info!("Reconnection successful");
                                    *state_for_reconnect.borrow_mut() = BridgeState::Connected;
                                    attempts_for_reconnect.store(0, std::sync::atomic::Ordering::SeqCst);
                                }) as Box<dyn FnMut(web_sys::Event)>);
                                ws.set_onopen(Some(onopen_reconnect.as_ref().unchecked_ref()));
                                onopen_reconnect.forget();

                                // onmessage
                                let onmessage_reconnect = Closure::wrap(Box::new(move |event: MessageEvent| {
                                    let text = match event.data().as_string() {
                                        Some(s) => s,
                                        None => return,
                                    };
                                    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&text) {
                                        if let Some(ref id_val) = resp.id {
                                            if let Some(id_num) = id_val.as_u64() {
                                                let cb = pending_for_reconnect.borrow_mut().remove(&id_num);
                                                if let Some(cb) = cb {
                                                    cb(id_val.clone(), resp.result, resp.error.map(|e| serde_json::json!({"code": e.code, "message": e.message})));
                                                }
                                                return;
                                            }
                                        }
                                    }
                                    if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&text) {
                                        if let Some(ref cb) = *notif_cb_for_reconnect.borrow() {
                                            cb(notif.method, notif.params);
                                        }
                                    }
                                }) as Box<dyn FnMut(MessageEvent)>);
                                ws.set_onmessage(Some(onmessage_reconnect.as_ref().unchecked_ref()));
                                onmessage_reconnect.forget();

                                // onerror
                                let onerror_reconnect = Closure::wrap(Box::new(move |_: ErrorEvent| {
                                    tracing::error!("Reconnection error");
                                    *state_for_reconnect.borrow_mut() = BridgeState::Reconnecting;
                                }) as Box<dyn FnMut(ErrorEvent)>);
                                ws.set_onerror(Some(onerror_reconnect.as_ref().unchecked_ref()));
                                onerror_reconnect.forget();

                                // onclose
                                let onclose_reconnect = Closure::wrap(Box::new(move |event: CloseEvent| {
                                    tracing::info!(
                                        "Reconnection closed: code={} reason={}",
                                        event.code(),
                                        event.reason()
                                    );
                                    *state_for_reconnect.borrow_mut() = BridgeState::Reconnecting;
                                }) as Box<dyn FnMut(CloseEvent)>);
                                ws.set_onclose(Some(onclose_reconnect.as_ref().unchecked_ref()));
                                onclose_reconnect.forget();

                                *socket_for_reconnect.borrow_mut() = Some(ws);
                            } else {
                                tracing::error!("Failed to create WebSocket for reconnection");
                                // Schedule another reconnect attempt
                            }
                        }) as Box<dyn FnMut()>);

                        // Schedule the callback with setTimeout
                        let _ = window
                            .dyn_ref::<wasm_bindgen::js_sys::global::Window>()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                next_interval as i32,
                            );
                        cb.forget();
                    });
                } else {
                    tracing::warn!(
                        "Max reconnection attempts ({}) reached for {}",
                        max,
                        onclose_url
                    );
                    *onclose_reconnect_state.borrow_mut() = BridgeState::Closed;
                }
            }
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        *self.socket.borrow_mut() = Some(ws);
        tracing::info!("Daemon bridge connecting to {}", self.url);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn connect(&self) -> Result<(), String> {
        tracing::info!("Daemon bridge simulation: connected to {}", self.url);
        Ok(())
    }

    /// Send a JSON-RPC request and register a callback for the response.
    ///
    /// Returns the numeric request ID that was assigned. The `cb` will be
    /// invoked when a response with a matching `id` arrives.
    #[cfg(target_arch = "wasm32")]
    pub fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
        cb: ResponseCallback,
    ) -> Result<u64, String> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(id)),
            method: method.to_string(),
            params,
        };

        let payload = serde_json::to_string(&req)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        self.pending.borrow_mut().insert(id, cb);

        let guard = self.socket.borrow();
        let sock = guard
            .as_ref()
            .ok_or_else(|| "WebSocket not connected — call connect() first".to_string())?;

        sock.send_with_str(&payload)
            .map_err(|e| format!("WebSocket send failed: {:?}", e))?;

        tracing::info!("Daemon bridge sent request id={} method={}", id, method);
        Ok(id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn send_request(
        &self,
        method: &str,
        _params: serde_json::Value,
        _cb: ResponseCallback,
    ) -> Result<u64, String> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tracing::info!(
            "Daemon bridge simulation: sent request id={} method={}",
            id,
            method
        );
        Ok(id)
    }

    /// Cancel a pending request by ID (the callback will not fire).
    #[cfg(target_arch = "wasm32")]
    pub fn cancel_request(&self, id: u64) {
        self.pending.borrow_mut().remove(&id);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn cancel_request(&self, _id: u64) {}

    /// Close the WebSocket and drop all pending callbacks.
    #[cfg(target_arch = "wasm32")]
    pub fn disconnect(&self) {
        *self.reconnection_state.borrow_mut() = BridgeState::Closed;
        self.pending.borrow_mut().clear();
        let mut guard = self.socket.borrow_mut();
        if let Some(ws) = guard.take() {
            if let Err(e) = ws.close() {
                tracing::warn!("Daemon bridge close error: {:?}", e);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn disconnect(&self) {
        tracing::info!("Daemon bridge simulation: disconnected");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams};

    #[test]
    fn get_identity_wire_shape() {
        let s = format_get_identity(1u32).unwrap();
        assert!(s.contains("get_identity"));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "get_identity");
    }

    #[test]
    fn send_message_wire_shape() {
        let s = format_send_message(1u32, "12D3KooWTest", "hello", Some("mid-1")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "send_message");
        assert_eq!(v["params"]["recipient"], "12D3KooWTest");
        assert_eq!(v["params"]["message"], "hello");
        assert_eq!(v["params"]["id"], "mid-1");
    }

    #[test]
    fn send_message_wire_shape_no_msg_id() {
        let s = format_send_message(2u32, "peer", "hi", None).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["params"]["recipient"], "peer");
        assert_eq!(v["params"]["message"], "hi");
        assert!(v["params"]["id"].is_null() || v["params"].get("id").is_none());
    }

    #[test]
    fn scan_peers_wire_shape() {
        let s = format_scan_peers(1u32).unwrap();
        assert!(s.contains("scan_peers"));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "scan_peers");
    }

    #[test]
    fn get_topology_wire_shape() {
        let s = format_get_topology(1u32).unwrap();
        assert!(s.contains("get_topology"));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "get_topology");
    }

    #[test]
    fn notification_roundtrip_for_ui_state() {
        let n = notif_message_received(MessageReceivedParams {
            from: "12D3KooW".into(),
            content: "hello".into(),
            timestamp: 99,
            message_id: "mid".into(),
        });
        let s = serde_json::to_string(&n).unwrap();
        let back = parse_notification(&s).unwrap();
        assert_eq!(back.method, "message_received");
    }

    #[test]
    fn response_roundtrip() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            result: Some(serde_json::json!({"ok": true})),
            error: None,
        };
        let s = serde_json::to_string(&resp).unwrap();
        let back = parse_response(&s).unwrap();
        assert_eq!(back.id, Some(serde_json::json!(1)));
        assert_eq!(back.result, Some(serde_json::json!({"ok": true})));
    }

    #[test]
    fn daemon_bridge_creation() {
        let bridge = DaemonBridge::new("ws://127.0.0.1:9000/ws".to_string());
        assert!(!bridge.is_connected());
    }

    #[test]
    fn daemon_bridge_connect_simulation() {
        let bridge = DaemonBridge::new("ws://127.0.0.1:9000/ws".to_string());
        assert!(bridge.connect().is_ok());
        assert!(bridge.is_connected());
    }

    #[test]
    fn daemon_bridge_send_request_simulation() {
        let bridge = DaemonBridge::new("ws://127.0.0.1:9000/ws".to_string());
        bridge.connect().unwrap();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let id = bridge
            .send_request(
                "get_identity",
                serde_json::json!({}),
                Box::new(move |id, result, error| {
                    called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    let _ = (id, result, error);
                }),
            )
            .unwrap();
        assert_eq!(id, 1);
        bridge.disconnect();
    }
}
