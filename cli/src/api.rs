// Control API for communicating with running SCMessenger node
//
// When `scm start` is running, it exposes a local HTTP API on localhost:9876
// Other CLI commands can send requests to this API instead of accessing the database directly

use anyhow::{Context, Result};
use axum::{
    extract::{Json as AxumJson, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub const API_PORT: u16 = 9876;
pub const API_ADDR: &str = "127.0.0.1:9876";

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub recipient: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddContactRequest {
    pub peer_id: String,
    pub public_key: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddContactResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: String,
    pub reputation: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPeersResponse {
    pub peers: Vec<PeerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConnectionStats {
    pub peer_id: String,
    pub state: String,
    pub duration_ms: u64,
    pub messages_sent: u64,
    pub message_failures: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: u64,
    pub last_activity: u64,
    pub connection_attempts: u32,
    pub successful_connections: u32,
    pub connection_failures: u32,
    pub current_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmStatsResponse {
    pub stats: Vec<ApiConnectionStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetHistoryRequest {
    pub peer_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryMessage {
    pub peer_id: String,
    pub content: String,
    pub direction: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetHistoryResponse {
    pub messages: Vec<HistoryMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetExternalAddressResponse {
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListenersResponse {
    pub listeners: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionPathStateResponse {
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriftStatusResponse {
    pub state: String,
    pub store_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryStatusResponse {
    pub mdns_enabled: bool,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub transport: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryPeersResponse {
    pub peers: Vec<DiscoveredPeer>,
}

// Check if API is available
pub async fn is_api_available() -> bool {
    tokio::net::TcpStream::connect(API_ADDR).await.is_ok()
}

// Client functions for CLI commands

pub async fn send_message_via_api(recipient: &str, message: &str) -> Result<()> {
    use http_body_util::{BodyExt, Empty, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = SendMessageRequest {
        recipient: recipient.to_string(),
        message: message.to_string(),
    };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/send", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: SendMessageResponse = serde_json::from_slice(&body_bytes)?;

    if response.success {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to send message: {}",
            response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )
    }
}

pub async fn add_contact_via_api(
    peer_id: &str,
    public_key: &str,
    name: Option<String>,
) -> Result<()> {
    use http_body_util::{BodyExt, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = AddContactRequest {
        peer_id: peer_id.to_string(),
        public_key: public_key.to_string(),
        name,
    };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/contacts", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: AddContactResponse = serde_json::from_slice(&body_bytes)?;

    if response.success {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to add contact: {}",
            response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )
    }
}

pub async fn get_peers_via_api() -> Result<Vec<PeerEntry>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/peers", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetPeersResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.peers)
}

pub async fn get_swarm_stats_via_api() -> Result<Vec<ApiConnectionStats>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/swarm/stats", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: SwarmStatsResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.stats)
}

/// NOTE: HANDOFF/done/task_wire_get_history_via_api.md claims this was wired,
/// but no caller exists anywhere in the repo as of this triage (2026-07-03) -
/// flagged for human follow-up.
#[allow(dead_code)]
pub async fn get_history_via_api(
    peer_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<HistoryMessage>> {
    use http_body_util::{BodyExt, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = GetHistoryRequest { peer_id, limit };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/history", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetHistoryResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.messages)
}

pub async fn get_external_address_via_api() -> Result<Vec<String>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/external-address", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;

    // Check HTTP status before attempting to parse
    let status = resp.status();
    let body_bytes = resp.into_body().collect().await?.to_bytes();

    if !status.is_success() {
        let error_body = String::from_utf8_lossy(&body_bytes);
        anyhow::bail!("API request failed with status {}: {}", status, error_body);
    }

    let response: GetExternalAddressResponse =
        serde_json::from_slice(&body_bytes).context("Failed to parse external address response")?;

    Ok(response.addresses)
}

pub async fn get_listeners_via_api() -> Result<Vec<String>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/listeners", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetListenersResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.listeners)
}

pub async fn get_connection_path_state_via_api() -> Result<String> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/connection-path-state", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: ConnectionPathStateResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.state)
}
pub async fn get_drift_state_via_api() -> Result<DriftStatusResponse> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/drift-status", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DriftStatusResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response)
}

pub async fn get_discovery_status() -> Result<DiscoveryStatusResponse> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/discovery/status", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DiscoveryStatusResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response)
}

pub async fn trigger_discovery_scan() -> Result<()> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/discovery/scan", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to trigger discovery scan: {}", resp.status());
    }
    Ok(())
}

pub async fn get_discovery_peers() -> Result<Vec<DiscoveredPeer>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/discovery/peers", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DiscoveryPeersResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.peers)
}

pub async fn export_diagnostics_via_api() -> Result<String> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/diagnostics", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    String::from_utf8(body_bytes.to_vec()).context("Diagnostics response was not UTF-8")
}

// Server implementation

#[derive(Clone)]
pub struct ApiContext {
    pub core: Arc<scmessenger_core::IronCore>,
    pub swarm_handle: Arc<scmessenger_core::transport::SwarmHandle>,
}

pub async fn stop_node_via_api() -> Result<()> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/shutdown", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let _res = client.request(req).await?;
    Ok(())
}

// Axum handler functions

async fn handle_send_message(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<SendMessageRequest>,
) -> Result<AxumJson<SendMessageResponse>, (StatusCode, String)> {
    let core = &ctx.core;
    let contacts = core.contacts_store_manager();

    let list = contacts.list().unwrap_or_default();
    let contact = list
        .into_iter()
        .find(|c| c.peer_id == request.recipient || c.nickname.as_ref() == Some(&request.recipient))
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Contact not found".to_string()))?;

    let peer_id = contact
        .peer_id
        .parse::<libp2p::PeerId>()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid peer ID: {}", e)))?;

    let prepared = core
        .prepare_message_with_id(
            contact.public_key.clone(),
            request.message.clone(),
            scmessenger_core::MessageType::Text,
            None,
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to prepare message: {:?}", e),
            )
        })?;

    let sent = crate::ble_mesh::send_ble_message(&peer_id.to_string(), &prepared.envelope_data).await.is_ok()
        || ctx.swarm_handle.send_message(peer_id, prepared.envelope_data, None, None).await.is_ok();

    if !sent {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send message via BLE and Swarm".to_string(),
        ));
    }

    Ok(AxumJson(SendMessageResponse {
        success: true,
        error: None,
    }))
}

async fn handle_add_contact(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<AddContactRequest>,
) -> Result<AxumJson<AddContactResponse>, (StatusCode, String)> {
    let contacts = ctx.core.contacts_store_manager();

    let mut contact =
        scmessenger_core::store::Contact::new(request.peer_id.clone(), request.public_key);
    if let Some(name) = request.name {
        contact.nickname = Some(name);
    }

    contacts.add(contact).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add contact: {:?}", e),
        )
    })?;

    Ok(AxumJson(AddContactResponse {
        success: true,
        error: None,
    }))
}

async fn handle_get_peers(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetPeersResponse>, (StatusCode, String)> {
    let peers: Vec<PeerEntry> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            let pid = p.to_string();
            let reputation = ctx.core.get_peer_reputation(pid.clone());
            PeerEntry {
                peer_id: pid,
                reputation,
            }
        })
        .collect();

    Ok(AxumJson(GetPeersResponse { peers }))
}

async fn handle_get_swarm_stats(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<SwarmStatsResponse>, (StatusCode, String)> {
    let raw_stats = ctx.core.get_all_connection_stats();
    let stats = raw_stats
        .into_iter()
        .map(|(peer_id, stat)| {
            let state_str = match stat.state {
                scmessenger_core::transport::health::ConnectionState::Connecting => "Connecting",
                scmessenger_core::transport::health::ConnectionState::Connected => "Connected",
                scmessenger_core::transport::health::ConnectionState::Disconnecting => {
                    "Disconnecting"
                }
                scmessenger_core::transport::health::ConnectionState::Disconnected => {
                    "Disconnected"
                }
                scmessenger_core::transport::health::ConnectionState::Failed => "Failed",
            }
            .to_string();

            ApiConnectionStats {
                peer_id: peer_id.to_string(),
                state: state_str,
                duration_ms: stat.duration_ms,
                messages_sent: stat.messages_sent,
                message_failures: stat.message_failures,
                bytes_sent: stat.bytes_sent,
                bytes_received: stat.bytes_received,
                avg_latency_ms: stat.avg_latency_ms,
                last_activity: stat.last_activity,
                connection_attempts: stat.connection_attempts,
                successful_connections: stat.successful_connections,
                connection_failures: stat.connection_failures,
                current_address: stat.current_address.map(|addr| addr.to_string()),
            }
        })
        .collect();

    Ok(AxumJson(SwarmStatsResponse { stats }))
}

async fn handle_get_listeners(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetListenersResponse>, (StatusCode, String)> {
    let listeners = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();

    Ok(AxumJson(GetListenersResponse { listeners }))
}

async fn handle_get_history(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<GetHistoryRequest>,
) -> Result<AxumJson<GetHistoryResponse>, (StatusCode, String)> {
    let history = ctx.core.history_store_manager();

    let messages = if let Some(peer_id) = request.peer_id {
        history
            .conversation(peer_id, request.limit.unwrap_or(20) as u32)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get history: {:?}", e),
                )
            })?
    } else {
        history
            .recent(None, request.limit.unwrap_or(20) as u32)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get history: {:?}", e),
                )
            })?
    };

    let history_messages: Vec<HistoryMessage> = messages
        .into_iter()
        .map(|m| HistoryMessage {
            peer_id: m.peer_id,
            content: m.content,
            direction: match m.direction {
                scmessenger_core::store::MessageDirection::Sent => "sent".to_string(),
                scmessenger_core::store::MessageDirection::Received => "received".to_string(),
            },
            timestamp: m.timestamp,
        })
        .collect();

    Ok(AxumJson(GetHistoryResponse {
        messages: history_messages,
    }))
}

async fn handle_get_external_address(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetExternalAddressResponse>, (StatusCode, String)> {
    let addresses = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get external addresses: {}", e),
            )
        })?;

    Ok(AxumJson(GetExternalAddressResponse {
        addresses: addresses.into_iter().map(|addr| addr.to_string()).collect(),
    }))
}

fn get_connection_path_state(
    peers: &[String],
    listeners: &[String],
    external_addrs: &[String],
) -> String {
    if peers.is_empty() {
        return "Bootstrapping".to_string();
    }
    if !listeners.is_empty() {
        return "DirectPreferred".to_string();
    }
    if !external_addrs.is_empty() {
        return "RelayFallback".to_string();
    }
    "RelayOnly".to_string()
}

fn export_diagnostics(
    peers: &[String],
    listeners: &[String],
    external_addrs: &[String],
    connection_path_state: &str,
    core: &scmessenger_core::IronCore,
) -> String {
    let history = core.history_store_manager();
    let stats = history.stats().ok();
    let mut payload = Map::new();
    payload.insert("running".to_string(), true.into());
    payload.insert(
        "connection_path_state".to_string(),
        connection_path_state.into(),
    );
    payload.insert("peers".to_string(), peers.into());
    payload.insert("listeners".to_string(), listeners.into());
    payload.insert("external_addrs".to_string(), external_addrs.into());
    payload.insert("inbox_count".to_string(), core.inbox_count().into());
    payload.insert("outbox_count".to_string(), core.outbox_count().into());
    payload.insert(
        "custody_audit_count".to_string(),
        core.custody_audit_count().into(),
    );

    let mut drift = Map::new();
    drift.insert("state".to_string(), core.drift_network_state().into());
    drift.insert("store_size".to_string(), core.drift_store_size().into());
    payload.insert("drift".to_string(), Value::Object(drift));

    payload.insert(
        "history_stats".to_string(),
        stats
            .as_ref()
            .map(|s| {
                let mut m = Map::new();
                m.insert("total_messages".to_string(), s.total_messages.into());
                m.insert("sent_count".to_string(), s.sent_count.into());
                m.insert("received_count".to_string(), s.received_count.into());
                m.insert("undelivered_count".to_string(), s.undelivered_count.into());
                Value::Object(m)
            })
            .into(),
    );
    payload.insert(
        "timestamp_ms".to_string(),
        chrono::Utc::now().timestamp_millis().into(),
    );
    Value::Object(payload).to_string()
}

async fn handle_get_connection_path_state(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<ConnectionPathStateResponse>, (StatusCode, String)> {
    let peers: Vec<String> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let listeners: Vec<String> = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let external_addrs: Vec<String> = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();

    Ok(AxumJson(ConnectionPathStateResponse {
        state: get_connection_path_state(&peers, &listeners, &external_addrs),
    }))
}

async fn handle_export_diagnostics(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<String, (StatusCode, String)> {
    let peers: Vec<String> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let listeners: Vec<String> = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let external_addrs: Vec<String> = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let connection_path_state = get_connection_path_state(&peers, &listeners, &external_addrs);
    let diagnostics = export_diagnostics(
        &peers,
        &listeners,
        &external_addrs,
        &connection_path_state,
        &ctx.core,
    );

    Ok(diagnostics)
}

async fn handle_get_drift_status(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<DriftStatusResponse>, (StatusCode, String)> {
    Ok(AxumJson(DriftStatusResponse {
        state: ctx.core.drift_network_state(),
        store_size: ctx.core.drift_store_size(),
    }))
}

async fn handle_get_discovery_status(
) -> Result<AxumJson<DiscoveryStatusResponse>, (StatusCode, String)> {
    let cfg = crate::config::Config::load().unwrap_or_default();
    Ok(AxumJson(DiscoveryStatusResponse {
        mdns_enabled: cfg.enable_mdns,
        ble_enabled: cfg.enable_ble,
        wifi_aware_enabled: cfg.enable_wifi_aware,
    }))
}

async fn handle_trigger_discovery_scan() -> Result<String, (StatusCode, String)> {
    Ok("Scan triggered".to_string())
}

async fn handle_get_discovery_peers(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<DiscoveryPeersResponse>, (StatusCode, String)> {
    let mut discovered = Vec::new();

    if let Ok(peers) = ctx.swarm_handle.get_peers().await {
        for peer_id in peers {
            let pid_str = peer_id.to_string();
            let nickname = ctx
                .core
                .contacts_store_manager()
                .get(pid_str.clone())
                .ok()
                .flatten()
                .and_then(|c| c.nickname);

            discovered.push(DiscoveredPeer {
                peer_id: pid_str,
                transport: "tcp/lan".to_string(),
                nickname,
            });
        }
    }

    Ok(AxumJson(DiscoveryPeersResponse { peers: discovered }))
}

async fn handle_shutdown() -> impl IntoResponse {
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    (StatusCode::OK, "Stopping...")
}

pub async fn start_api_server(ctx: ApiContext) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    // Build router with all routes
    let app = Router::new()
        .route("/api/send", post(handle_send_message))
        .route("/api/contacts", post(handle_add_contact))
        .route("/api/peers", get(handle_get_peers))
        .route("/api/swarm/stats", get(handle_get_swarm_stats))
        .route("/api/listeners", get(handle_get_listeners))
        .route("/api/history", post(handle_get_history))
        .route("/api/external-address", get(handle_get_external_address))
        .route(
            "/api/connection-path-state",
            get(handle_get_connection_path_state),
        )
        .route("/api/diagnostics", get(handle_export_diagnostics))
        .route("/api/drift-status", get(handle_get_drift_status))
        .route("/api/discovery/status", get(handle_get_discovery_status))
        .route("/api/discovery/scan", post(handle_trigger_discovery_scan))
        .route("/api/discovery/peers", get(handle_get_discovery_peers))
        .route("/api/shutdown", post(handle_shutdown))
        .layer(cors)
        .with_state(ctx);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind API server")?;

    tracing::info!("Control API listening on {}", addr);

    // Serve with axum
    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}
