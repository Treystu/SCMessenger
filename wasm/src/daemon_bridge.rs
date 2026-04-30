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
use std::sync::Arc;

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

// ── Blocking ──

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
pub struct DaemonBridge {
    url: String,
    next_id: std::sync::atomic::AtomicU64,
    /// Pending requests keyed by JSON-RPC id, awaiting a response.
    #[cfg(target_arch = "wasm32")]
    pending: Arc<parking_lot::Mutex<std::collections::HashMap<u64, ResponseCallback>>>,
    /// Notification callback invoked on each server push.
    #[cfg(target_arch = "wasm32")]
    on_notification: Arc<parking_lot::Mutex<Option<NotificationCallback>>>,
    /// Live WebSocket handle (held so the browser object is not GC'd).
    #[cfg(target_arch = "wasm32")]
    socket: Arc<parking_lot::Mutex<Option<web_sys::WebSocket>>>,
    #[cfg(target_arch = "wasm32")]
    url_clone: String,
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
            pending: Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
            #[cfg(target_arch = "wasm32")]
            on_notification: Arc::new(parking_lot::Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            socket: Arc::new(parking_lot::Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            url_clone: url,
        }
    }

    /// Register a callback for server-push notifications.
    ///
    /// The callback receives `(method_name, params_object)` for each
    /// notification frame from the daemon. Only one callback is active at
    /// a time — calling this again replaces the previous one.
    #[cfg(target_arch = "wasm32")]
    pub fn on_notification(&self, cb: NotificationCallback) {
        *self.on_notification.lock() = Some(cb);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_notification(&self, _cb: NotificationCallback) {}

    /// Open the WebSocket connection to the daemon.
    ///
    /// Registers `onopen`, `onmessage`, `onerror`, and `onclose` event
    /// handlers. Inbound text frames are parsed as JSON-RPC responses or
    /// notifications and dispatched to the registered callbacks.
    #[cfg(target_arch = "wasm32")]
    pub fn connect(&self) -> Result<(), String> {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

        let ws = WebSocket::new(&self.url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

        // ── onopen ──
        let url_log = self.url.clone();
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            tracing::info!("Daemon bridge connected to {}", url_log);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // ── onmessage ──
        let pending_map = Arc::clone(&self.pending);
        let notif_cb = Arc::clone(&self.on_notification);
        let onmessage =
            Closure::wrap(Box::new(move |event: MessageEvent| {
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
                            let cb = pending_map.lock().remove(&id_num);
                            if let Some(cb) = cb {
                                cb(*id_val, resp.result, resp.error.map(|e| {
                                serde_json::json!({"code": e.code, "message": e.message})
                            }));
                                return;
                            }
                        }
                    }
                }

                // Try parsing as a JSON-RPC notification (no `id`).
                if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&text) {
                    if let Some(ref cb) = *notif_cb.lock() {
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
        let url_err = self.url.clone();
        let onerror = Closure::wrap(Box::new(move |event: ErrorEvent| {
            tracing::error!("Daemon bridge error for {}: {}", url_err, event.message());
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // ── onclose ──
        let url_close = self.url.clone();
        let onclose = Closure::wrap(Box::new(move |event: CloseEvent| {
            tracing::info!(
                "Daemon bridge closed for {}: code={} reason={}",
                url_close,
                event.code(),
                event.reason()
            );
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        *self.socket.lock() = Some(ws);
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

        self.pending.lock().insert(id, cb);

        let guard = self.socket.lock();
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
        self.pending.lock().remove(&id);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn cancel_request(&self, _id: u64) {}

    /// Close the WebSocket and drop all pending callbacks.
    #[cfg(target_arch = "wasm32")]
    pub fn disconnect(&self) {
        self.pending.lock().clear();
        let mut guard = self.socket.lock();
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

    /// True if a WebSocket handle is held (connected or connecting).
    #[cfg(target_arch = "wasm32")]
    pub fn is_connected(&self) -> bool {
        self.socket.lock().is_some()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_connected(&self) -> bool {
        true
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
