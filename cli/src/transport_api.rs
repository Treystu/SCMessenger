// Transport Bridge API - Minimal working version
//
// Simplified to avoid complex warp filter issues while maintaining functionality
// NOTE: This module contains unused code that will be activated when warp filter
// chaining issues are resolved. The structures and functions are kept for future use.
use scmessenger_core::transport::abstraction::TransportType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[allow(dead_code)]
use warp::Filter;
use warp::Rejection;

/// Request payload for peer registration
#[derive(Deserialize, Debug)]
pub struct RegisterPeerRequest {
    pub peer_id: String,
    pub capabilities: Vec<String>,
}

/// Response payload for transport capabilities
#[derive(Serialize, Debug)]
pub struct TransportCapabilitiesResponse {
    pub cli_capabilities: Vec<String>,
    pub peer_capabilities: std::collections::HashMap<String, Vec<String>>,
}

/// Response payload for transport paths
#[derive(Serialize, Debug)]
pub struct TransportPathsResponse {
    pub paths: Vec<crate::transport_bridge::TransportRoute>,
}

/// Custom error type for transport API
#[derive(Debug)]
pub enum TransportError {
    InvalidPeerId,
    InvalidCapabilities,
}

impl warp::reject::Reject for TransportError {}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::InvalidPeerId => write!(f, "Invalid peer ID format"),
            TransportError::InvalidCapabilities => {
                write!(f, "No valid transport capabilities provided")
            }
        }
    }
}

pub fn transport_routes(
    web_ctx: Arc<crate::server::WebContext>,
) -> impl Filter<Extract = (impl warp::Reply,)> + Clone {
    // Clone once for each route to avoid move issues
    let ctx1 = web_ctx.clone();
    let ctx2 = web_ctx.clone();
    let ctx3 = web_ctx.clone();

    // Transport capabilities endpoint
    let capabilities_route = warp::path!("api" / "transport" / "capabilities")
        .and(warp::get())
        .and(warp::any().map(move || ctx1.clone()))
        .and_then(handle_transport_capabilities)
        .boxed();

    // Transport paths endpoint
    let paths_route = warp::path!("api" / "transport" / "paths" / String)
        .and(warp::get())
        .and(warp::any().map(move || ctx2.clone()))
        .and_then(handle_transport_paths)
        .boxed();

    // Register peer endpoint
    let register_route = warp::path!("api" / "transport" / "register")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || ctx3.clone()))
        .and_then(handle_register_peer)
        .boxed();

    capabilities_route.or(paths_route).or(register_route)
}

async fn handle_transport_capabilities(
    ctx: Arc<crate::server::WebContext>,
) -> Result<impl warp::Reply, Rejection> {
    let bridge = ctx.transport_bridge.lock().await;

    let cli_caps = bridge
        .get_cli_capabilities()
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>();

    let mut peer_caps = std::collections::HashMap::new();
    for (peer_id, caps) in bridge.get_available_paths() {
        let cap_strings = caps
            .iter()
            .map(|path| path.destination.to_string())
            .collect::<Vec<String>>();
        peer_caps.insert(peer_id.to_string(), cap_strings);
    }

    let response = TransportCapabilitiesResponse {
        cli_capabilities: cli_caps,
        peer_capabilities: peer_caps,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_transport_paths(
    peer_id: String,
    ctx: Arc<crate::server::WebContext>,
) -> Result<impl warp::Reply, Rejection> {
    let peer_id = match peer_id.parse::<libp2p::PeerId>() {
        Ok(id) => id,
        Err(_) => return Err(warp::reject::custom(TransportError::InvalidPeerId)),
    };

    let bridge = ctx.transport_bridge.lock().await;
    let paths = bridge.find_all_paths(&peer_id);

    let routes = paths.iter().map(|path| path.into()).collect::<Vec<_>>();

    let response = TransportPathsResponse { paths: routes };

    Ok(warp::reply::json(&response))
}

async fn handle_register_peer(
    request: RegisterPeerRequest,
    ctx: Arc<crate::server::WebContext>,
) -> Result<impl warp::Reply, Rejection> {
    let peer_id = match request.peer_id.parse::<libp2p::PeerId>() {
        Ok(id) => id,
        Err(_) => return Err(warp::reject::custom(TransportError::InvalidPeerId)),
    };

    // Convert string capabilities to TransportType enum
    let capabilities: Vec<TransportType> = request
        .capabilities
        .iter()
        .filter_map(|s| match s.as_str() {
            "BLE" => Some(TransportType::BLE),
            "WiFiAware" => Some(TransportType::WiFiAware),
            "WiFiDirect" => Some(TransportType::WiFiDirect),
            "Internet" => Some(TransportType::Internet),
            "Local" => Some(TransportType::Local),
            _ => None,
        })
        .collect();

    if capabilities.is_empty() {
        return Err(warp::reject::custom(TransportError::InvalidCapabilities));
    }

    let mut bridge = ctx.transport_bridge.lock().await;
    bridge.register_peer(peer_id, capabilities);

    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Peer registered"
    })))
}
