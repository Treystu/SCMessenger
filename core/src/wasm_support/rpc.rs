//! JSON-RPC shaped protocol for the local WASM thin client ↔ CLI daemon WebSocket bridge.
//!
//! Wire format: JSON-RPC 2.0 requests from the browser; responses and server-push notifications
//! use the same envelope (`result` or `method` + `params` for notifications).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Incoming JSON-RPC request from WASM.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC success or error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Server → client push (no `id`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

/// Intents: WASM → CLI (`method` names are snake_case).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum ClientIntent {
    SendMessage {
        recipient: String,
        message: String,
        #[serde(default)]
        id: Option<String>,
    },
    ScanPeers {},
    GetTopology {},
    GetIdentity {},
}

const ERR_PARSE: i32 = -32700;
const ERR_METHOD: i32 = -32601;
const ERR_PARAMS: i32 = -32602;

/// Map JSON-RPC `method` + `params` to a [`ClientIntent`].
pub fn parse_intent(req: &JsonRpcRequest) -> Result<ClientIntent, JsonRpcErrorBody> {
    if req.jsonrpc != "2.0" {
        return Err(JsonRpcErrorBody {
            code: ERR_PARSE,
            message: "jsonrpc must be \"2.0\"".to_string(),
            data: None,
        });
    }

    match req.method.as_str() {
        "send_message" => {
            let recipient = req
                .params
                .get("recipient")
                .and_then(|v| v.as_str())
                .ok_or_else(|| JsonRpcErrorBody {
                    code: ERR_PARAMS,
                    message: "missing recipient".to_string(),
                    data: None,
                })?
                .to_string();
            let message = req
                .params
                .get("message")
                .and_then(|v| v.as_str())
                .ok_or_else(|| JsonRpcErrorBody {
                    code: ERR_PARAMS,
                    message: "missing message".to_string(),
                    data: None,
                })?
                .to_string();
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(ClientIntent::SendMessage {
                recipient,
                message,
                id,
            })
        }
        "scan_peers" => Ok(ClientIntent::ScanPeers {}),
        "get_topology" => Ok(ClientIntent::GetTopology {}),
        "get_identity" => Ok(ClientIntent::GetIdentity {}),
        _ => Err(JsonRpcErrorBody {
            code: ERR_METHOD,
            message: format!("unknown method {}", req.method),
            data: None,
        }),
    }
}

pub fn rpc_result(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

pub fn rpc_error(id: Option<Value>, err: JsonRpcErrorBody) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(err),
    }
}

/// Notification method names (CLI → WASM).
pub mod notif {
    pub const MESSAGE_RECEIVED: &str = "message_received";
    pub const PEER_DISCOVERED: &str = "peer_discovered";
    pub const MESH_TOPOLOGY_UPDATE: &str = "mesh_topology_update";
    pub const DELIVERY_STATUS: &str = "delivery_status";
}

pub fn notification(method: impl Into<String>, params: Value) -> JsonRpcNotification {
    JsonRpcNotification {
        jsonrpc: "2.0".into(),
        method: method.into(),
        params,
    }
}

/// Typed event params for notifications (serde-compatible with plan).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageReceivedParams {
    pub from: String,
    pub content: String,
    pub timestamp: u64,
    #[serde(default)]
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PeerDiscoveredParams {
    pub peer_id: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MeshTopologyUpdateParams {
    pub peer_count: usize,
    pub known_peers: usize,
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusParams {
    pub message_id: String,
    pub status: String,
}

pub fn notif_message_received(p: MessageReceivedParams) -> JsonRpcNotification {
    notification(
        notif::MESSAGE_RECEIVED,
        serde_json::to_value(&p).unwrap_or_else(|_| json!({})),
    )
}

pub fn notif_peer_discovered(p: PeerDiscoveredParams) -> JsonRpcNotification {
    notification(
        notif::PEER_DISCOVERED,
        serde_json::to_value(&p).unwrap_or_else(|_| json!({})),
    )
}

pub fn notif_mesh_topology(p: MeshTopologyUpdateParams) -> JsonRpcNotification {
    notification(
        notif::MESH_TOPOLOGY_UPDATE,
        serde_json::to_value(&p).unwrap_or_else(|_| json!({})),
    )
}

pub fn notif_delivery_status(p: DeliveryStatusParams) -> JsonRpcNotification {
    notification(
        notif::DELIVERY_STATUS,
        serde_json::to_value(&p).unwrap_or_else(|_| json!({})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_send_message_roundtrip() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "send_message".to_string(),
            params: json!({
                "recipient": "12D3KooWTest",
                "message": "hi",
                "id": "mid-1"
            }),
        };
        let s = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, req);
        let intent = parse_intent(&back).unwrap();
        assert_eq!(
            intent,
            ClientIntent::SendMessage {
                recipient: "12D3KooWTest".into(),
                message: "hi".into(),
                id: Some("mid-1".into()),
            }
        );
    }

    #[test]
    fn jsonrpc_get_identity() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!("a")),
            method: "get_identity".to_string(),
            params: json!({}),
        };
        assert!(matches!(
            parse_intent(&req).unwrap(),
            ClientIntent::GetIdentity {}
        ));
    }

    #[test]
    fn notification_serialization() {
        let n = notif_message_received(MessageReceivedParams {
            from: "peer".into(),
            content: "x".into(),
            timestamp: 42,
            message_id: "m1".into(),
        });
        let s = serde_json::to_string(&n).unwrap();
        assert!(s.contains("message_received"));
        let back: JsonRpcNotification = serde_json::from_str(&s).unwrap();
        assert_eq!(back.method, notif::MESSAGE_RECEIVED);
    }

    #[test]
    fn unknown_method_error() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "nope".to_string(),
            params: json!({}),
        };
        assert!(parse_intent(&req).is_err());
    }
}
