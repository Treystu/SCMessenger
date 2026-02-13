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

#[derive(Debug, Serialize, Deserialize)]
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

// Server implementation

pub struct ApiContext {
    pub core: Arc<scmessenger_core::IronCore>,
    pub contacts: Arc<crate::contacts::ContactList>,
    pub history: Arc<crate::history::MessageHistory>,
    pub swarm_handle: Arc<scmessenger_core::transport::SwarmHandle>,
    pub peers: Arc<tokio::sync::Mutex<std::collections::HashMap<libp2p::PeerId, Option<String>>>>,
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
        (&Method::POST, "/api/history") => handle_get_history(req, ctx).await,
        (&Method::GET, "/api/external-address") => handle_get_external_address(req, ctx).await,
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

    // Find contact
    let contact = crate::find_contact(&ctx.contacts, &request.recipient)?;

    // Parse peer ID
    let peer_id = contact.peer_id.parse::<libp2p::PeerId>()?;

    // Prepare and send message
    let envelope_bytes = ctx
        .core
        .prepare_message(contact.public_key.clone(), request.message.clone())?;
    ctx.swarm_handle
        .send_message(peer_id, envelope_bytes)
        .await?;

    // Record in history
    let record = crate::history::MessageRecord::new_sent(contact.peer_id.clone(), request.message);
    ctx.history.add(record)?;

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

    // Validate public key format
    if let Err(e) = scmessenger_core::crypto::validate_ed25519_public_key(&request.public_key) {
        let response = AddContactResponse {
            success: false,
            error: Some(format!("Invalid public key: {}", e)),
        };
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&response)?))?);
    }

    let contact = crate::contacts::Contact::new(request.peer_id.clone(), request.public_key)
        .with_nickname(request.name.unwrap_or_else(|| request.peer_id.clone()));

    ctx.contacts.add(contact)?;

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
    let peers = ctx.peers.lock().await;
    let peer_ids: Vec<String> = peers.keys().map(|p| p.to_string()).collect();

    let response = GetPeersResponse { peers: peer_ids };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&response)?))?)
}

async fn handle_get_history(req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: GetHistoryRequest = serde_json::from_slice(&body_bytes)?;

    let messages = if let Some(peer_id) = request.peer_id {
        ctx.history
            .conversation(&peer_id, request.limit.unwrap_or(20))?
    } else {
        ctx.history.recent(None, request.limit.unwrap_or(20))?
    };

    let history_messages: Vec<HistoryMessage> = messages
        .into_iter()
        .map(|m| HistoryMessage {
            peer_id: m.peer().to_string(),
            content: m.content,
            direction: match m.direction {
                crate::history::Direction::Sent => "sent".to_string(),
                crate::history::Direction::Received => "received".to_string(),
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
