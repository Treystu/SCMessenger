// Transport Bridge API - Types module
//
// Provides transport capability discovery and peer registration types.
// The server uses inline implementations for direct integration with
// WebContext and TransportBridge.

/// Request payload for peer registration
#[derive(serde::Deserialize, Debug)]
pub struct RegisterPeerRequest {
    pub peer_id: String,
    pub capabilities: Vec<String>,
}

/// Custom error type for transport API
#[derive(Debug)]
pub enum TransportError {
    InvalidPeerId,
    #[allow(dead_code)]
    InvalidCapabilities,
}

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

impl warp::reject::Reject for TransportError {}
