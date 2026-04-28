//! Local CLI daemon WebSocket bridge (thin client).
//!
//! The browser must open `ws://127.0.0.1:<port>/ws` with `Origin: http://127.0.0.1:<port>`
//! (or `http://localhost:<port>`). Wire format: JSON-RPC 2.0 as defined in
//! `scmessenger_core::wasm_support::rpc`.

use scmessenger_core::wasm_support::rpc::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

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

/// Parse a JSON-RPC response from the daemon.
pub fn parse_response(s: &str) -> Result<JsonRpcResponse, serde_json::Error> {
    serde_json::from_str(s)
}

/// Parse a JSON-RPC notification (server push).
pub fn parse_notification(s: &str) -> Result<JsonRpcNotification, serde_json::Error> {
    serde_json::from_str(s)
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
}
