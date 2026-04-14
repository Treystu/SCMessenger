// Control API for communicating with running SCMessenger node
//
// When `scm start` is running, it exposes a local HTTP API on localhost:9876
// Other CLI commands can send requests to this API instead of accessing the database directly

use anyhow::{Context, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

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
pub struct GetPeersResponse {
    pub peers: Vec<String>,
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

// Check if API is available
pub async fn is_api_available() -> bool {
    tokio::net::TcpStream::connect(API_ADDR).await.is_ok()
}

// Client functions for CLI commands

pub async fn send_message_via_api(recipient: &str, message: &str) -> Result<()> {
    let client = hyper::Client::new();
    let req_body = SendMessageRequest {
        recipient: recipient.to_string(),
        message: message.to_string(),
    };

    let json = serde_json::to_string(&req_body)?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/send", API_ADDR))
        .header("content-type", "application/json")
        .body(Body::from(json))?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
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
    let client = hyper::Client::new();
    let req_body = AddContactRequest {
        peer_id: peer_id.to_string(),
        public_key: public_key.to_string(),
        name,
    };

    let json = serde_json::to_string(&req_body)?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/contacts", API_ADDR))
        .header("content-type", "application/json")
        .body(Body::from(json))?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
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

#[allow(dead_code)]
pub async fn get_peers_via_api() -> Result<Vec<String>> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/peers", API_ADDR))
        .body(Body::empty())?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let response: GetPeersResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.peers)
}

#[allow(dead_code)]
pub async fn get_history_via_api(
    peer_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<HistoryMessage>> {
    let client = hyper::Client::new();
    let req_body = GetHistoryRequest { peer_id, limit };

    let json = serde_json::to_string(&req_body)?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/history", API_ADDR))
        .header("content-type", "application/json")
        .body(Body::from(json))?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let response: GetHistoryResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.messages)
}

#[allow(dead_code)]
pub async fn get_external_address_via_api() -> Result<Vec<String>> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/external-address", API_ADDR))
        .body(Body::empty())?;

    let resp = client.request(req).await?;

    // Check HTTP status before attempting to parse
    let status = resp.status();
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;

    if !status.is_success() {
        let error_body = String::from_utf8_lossy(&body_bytes);
        anyhow::bail!("API request failed with status {}: {}", status, error_body);
    }

    let response: GetExternalAddressResponse =
        serde_json::from_slice(&body_bytes).context("Failed to parse external address response")?;

    Ok(response.addresses)
}

pub async fn get_listeners_via_api() -> Result<Vec<String>> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/listeners", API_ADDR))
        .body(Body::empty())?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let response: GetListenersResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.listeners)
}

pub async fn get_connection_path_state_via_api() -> Result<String> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/connection-path-state", API_ADDR))
        .body(Body::empty())?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let response: ConnectionPathStateResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.state)
}

pub async fn export_diagnostics_via_api() -> Result<String> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/diagnostics", API_ADDR))
        .body(Body::empty())?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    String::from_utf8(body_bytes.to_vec()).context("Diagnostics response was not UTF-8")
}

// Server implementation

pub struct ApiContext {
    pub core: Arc<scmessenger_core::IronCore>,
    pub swarm_handle: Arc<scmessenger_core::transport::SwarmHandle>,
}

pub async fn stop_node_via_api() -> Result<()> {
    let client = hyper::Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/shutdown", API_ADDR))
        .body(Body::empty())?;

    let _res = client.request(req).await?;
    Ok(())
}

async fn handle_request(
    req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>, Infallible> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/send") => handle_send_message(req, ctx).await,
        (&Method::POST, "/api/contacts") => handle_add_contact(req, ctx).await,
        (&Method::GET, "/api/peers") => handle_get_peers(req, ctx).await,
        (&Method::GET, "/api/listeners") => handle_get_listeners(req, ctx).await,
        (&Method::POST, "/api/history") => handle_get_history(req, ctx).await,
        (&Method::GET, "/api/external-address") => handle_get_external_address(req, ctx).await,
        (&Method::GET, "/api/connection-path-state") => {
            handle_get_connection_path_state(req, ctx).await
        }
        (&Method::GET, "/api/diagnostics") => handle_export_diagnostics(req, ctx).await,
        (&Method::POST, "/api/shutdown") => {
            // Spawn a task to exit after a brief delay to allow response to send
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                std::process::exit(0);
            });
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from("Stopping..."))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    };

    Ok(response.unwrap_or_else(|e| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Error: {}", e)))
            .unwrap()
    }))
}

async fn handle_send_message(req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: SendMessageRequest = serde_json::from_slice(&body_bytes)?;

    // Core handle
    let core = &ctx.core;
    let contacts = core.contacts_store_manager();

    // Find contact
    // Note: find_contact needs the contact list, but it's a CLI-specific helper.
    // We'll update it or do a manual lookup.
    let list = contacts.list().unwrap_or_default();
    let contact = list
        .into_iter()
        .find(|c| c.peer_id == request.recipient || c.nickname.as_ref() == Some(&request.recipient))
        .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;

    // Parse peer ID
    let peer_id = contact.peer_id.parse::<libp2p::PeerId>()?;

    // Prepare and send message.
    // Uses prepare_message_with_id to trigger Core's auto-save to history.
    let prepared =
        core.prepare_message_with_id(contact.public_key.clone(), request.message.clone(), None)?;

    ctx.swarm_handle
        .send_message(peer_id, prepared.envelope_data, None, None)
        .await?;

    let response = SendMessageResponse {
        success: true,
        error: None,
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_add_contact(req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: AddContactRequest = serde_json::from_slice(&body_bytes)?;

    let contacts = ctx.core.contacts_store_manager();

    let mut contact =
        scmessenger_core::store::Contact::new(request.peer_id.clone(), request.public_key);
    if let Some(name) = request.name {
        contact.nickname = Some(name);
    }

    contacts
        .add(contact)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let response = AddContactResponse {
        success: true,
        error: None,
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_get_peers(_req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let peer_ids: Vec<String> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();

    let response = GetPeersResponse { peers: peer_ids };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_get_listeners(_req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let listeners = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();

    let response = GetListenersResponse { listeners };
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_get_history(req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: GetHistoryRequest = serde_json::from_slice(&body_bytes)?;

    let history = ctx.core.history_store_manager();

    let messages = if let Some(peer_id) = request.peer_id {
        history
            .conversation(peer_id, request.limit.unwrap_or(20) as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    } else {
        history
            .recent(None, request.limit.unwrap_or(20) as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
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

    let response = GetHistoryResponse {
        messages: history_messages,
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_get_external_address(
    _req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>> {
    // Get external addresses from swarm
    let addresses = match ctx.swarm_handle.get_external_addresses().await {
        Ok(addresses) => addresses,
        Err(e) => {
            let body = format!("Failed to get external addresses: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Body::from(body))?);
        }
    };

    let response = GetExternalAddressResponse {
        addresses: addresses.into_iter().map(|addr| addr.to_string()).collect(),
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
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
    let payload = serde_json::json!({
        "running": true,
        "connection_path_state": connection_path_state,
        "peers": peers,
        "listeners": listeners,
        "external_addrs": external_addrs,
        "inbox_count": core.inbox_count(),
        "outbox_count": core.outbox_count(),
        "history_stats": stats.as_ref().map(|s| serde_json::json!({
            "total_messages": s.total_messages,
            "sent_count": s.sent_count,
            "received_count": s.received_count,
            "undelivered_count": s.undelivered_count,
        })),
        "timestamp_ms": chrono::Utc::now().timestamp_millis(),
    });
    payload.to_string()
}

async fn handle_get_connection_path_state(
    _req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>> {
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

    let response = ConnectionPathStateResponse {
        state: get_connection_path_state(&peers, &listeners, &external_addrs),
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_export_diagnostics(
    _req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>> {
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(diagnostics))?)
}

pub async fn start_api_server(ctx: ApiContext) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    let make_svc = make_service_fn(move |_conn| {
        let ctx = ctx.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, ctx.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    tracing::info!("Control API listening on {}", addr);

    server.await.context("API server error")?;

    Ok(())
}
