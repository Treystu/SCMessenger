// Default bootstrap nodes for SCMessenger network
//
// These nodes are embedded into binaries and Docker images at build time,
// providing automatic network connectivity without manual configuration.
//
// Build-time customization:
// - Set SCMESSENGER_BOOTSTRAP_NODES environment variable during build
// - Format: comma-separated multiaddrs
// - Example: export SCMESSENGER_BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo..."

/// Default bootstrap nodes - can be overridden at build time
///
/// Strategy: Multiple public relay nodes with varying availability
/// - Node 1: Primary GCP (high availability)
/// - Node 2: Secondary relay (geographic redundancy)
/// - Node 3: Tertiary relay (provider diversity)
/// - Node 7: Community relay (rotating availability)
///
/// All nodes relay for the mesh. Connection attempts fail over automatically.
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    // Node 1: Primary GCP bootstrap (North America) - High availability
    "/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",

    // Node 2: Secondary relay (add when deployed)
    // "/ip4/<IP>/tcp/9001/p2p/<PEER_ID>",

    // Node 3: Tertiary relay (add when deployed)
    // "/ip4/<IP>/tcp/9001/p2p/<PEER_ID>",

    // Node 7: Community relay (add when deployed)
    // "/ip4/<IP>/tcp/9001/p2p/<PEER_ID>",
];

/// Get default bootstrap nodes, with optional build-time override
pub fn default_bootstrap_nodes() -> Vec<String> {
    // Check for build-time override first
    let build_time_nodes = option_env!("SCMESSENGER_BOOTSTRAP_NODES");

    if let Some(nodes_str) = build_time_nodes {
        // Parse comma-separated multiaddrs
        nodes_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        // Use hardcoded defaults
        DEFAULT_BOOTSTRAP_NODES
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// Merge user-provided bootstrap nodes with defaults
/// Ensures defaults are preserved unless explicitly removed
pub fn merge_bootstrap_nodes(user_nodes: Vec<String>) -> Vec<String> {
    let mut merged = default_bootstrap_nodes();

    // Add user nodes that aren't already in the list
    for node in user_nodes {
        if !merged.contains(&node) {
            merged.push(node);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bootstrap_nodes() {
        let nodes = default_bootstrap_nodes();
        assert!(!nodes.is_empty(), "Should have at least one default bootstrap node");

        // Verify format
        for node in &nodes {
            assert!(node.starts_with("/ip4/"), "Bootstrap node should be multiaddr: {}", node);
            assert!(node.contains("/p2p/"), "Bootstrap node should include peer ID: {}", node);
        }
    }

    #[test]
    fn test_merge_bootstrap_nodes() {
        let user_nodes = vec![
            "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWTestPeerID".to_string(),
        ];

        let merged = merge_bootstrap_nodes(user_nodes.clone());

        // Should contain both defaults and user nodes
        assert!(merged.len() >= default_bootstrap_nodes().len() + 1);
        assert!(merged.contains(&user_nodes[0]));
    }

    #[test]
    fn test_merge_deduplicates() {
        let user_nodes = default_bootstrap_nodes(); // Same as defaults
        let merged = merge_bootstrap_nodes(user_nodes);

        // Should not duplicate
        assert_eq!(merged.len(), default_bootstrap_nodes().len());
    }
}
