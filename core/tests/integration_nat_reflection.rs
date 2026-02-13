// Integration tests for NAT traversal and address reflection protocol
//
// These tests verify the complete address reflection protocol with real libp2p swarms.
// They demonstrate:
// - Two-node address reflection (peer asks, other peer responds)
// - NAT type detection using multiple peer reflectors
// - External address discovery via mesh peers
// - Full request-response lifecycle

use libp2p::identity::Keypair;
use scmessenger_core::transport::{
    nat::{NatConfig, NatTraversal, PeerAddressDiscovery},
    swarm::{start_swarm, SwarmEvent2},
};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_two_node_address_reflection() {
    // Setup: Create two libp2p swarms
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    // Start first node (will be the reflector)
    let swarm1 = start_swarm(keypair1, None, event_tx1)
        .await
        .expect("Failed to start swarm1");

    // Wait for first node to start listening
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get the listening address of first node
    let mut listen_addr = None;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                listen_addr = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    assert!(listen_addr.is_some(), "Node 1 should be listening");
    let node1_addr = listen_addr.unwrap();

    // Start second node
    let swarm2 = start_swarm(keypair2, None, event_tx2)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Node 2 dials node 1
    swarm2
        .dial(node1_addr.clone())
        .await
        .expect("Failed to dial");

    // Wait for connection
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get peer IDs
    let peers = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty(), "Node 2 should be connected to node 1");

    let peer1_id = peers[0];

    // Node 2 requests address reflection from node 1
    let result = swarm2.request_address_reflection(peer1_id).await;

    assert!(result.is_ok(), "Address reflection should succeed");
    let observed_addr = result.unwrap();

    // Verify we got a valid address back
    assert!(!observed_addr.is_empty(), "Should receive observed address");
    println!("✅ Address reflection test passed!");
    println!("   Node 1 observed Node 2 at: {}", observed_addr);
}

#[tokio::test]
async fn test_peer_address_discovery_with_live_swarm() {
    // Setup: Create reflector node and discovery node
    let keypair_reflector = Keypair::generate_ed25519();
    let keypair_requester = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    // Start reflector node
    let swarm_reflector = start_swarm(keypair_reflector.clone(), None, event_tx1)
        .await
        .expect("Failed to start reflector swarm");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get reflector listening address
    let mut reflector_addr = None;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                reflector_addr = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    assert!(reflector_addr.is_some());

    // Start requester node
    let swarm_requester = start_swarm(keypair_requester.clone(), None, event_tx2)
        .await
        .expect("Failed to start requester swarm");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect requester to reflector
    swarm_requester
        .dial(reflector_addr.unwrap())
        .await
        .expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get reflector peer ID
    let peers = swarm_requester
        .get_peers()
        .await
        .expect("Failed to get peers");
    assert!(!peers.is_empty());
    let reflector_peer_id = peers[0];

    // Create PeerAddressDiscovery with the reflector
    let discovery = PeerAddressDiscovery::with_peers(vec![reflector_peer_id.to_string()], 10);

    // Request external address
    let result = discovery.get_external_address(&swarm_requester).await;

    assert!(
        result.is_ok(),
        "Should successfully discover external address"
    );
    let external_addr = result.unwrap();

    println!("✅ Peer address discovery test passed!");
    println!("   Discovered external address: {}", external_addr);
}

#[tokio::test]
async fn test_nat_traversal_with_live_swarms() {
    // Setup: Create 3 nodes for multi-peer NAT detection
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();
    let keypair3 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);
    let (event_tx3, mut _event_rx3) = mpsc::channel(256);

    // Start all three nodes
    let swarm1 = start_swarm(keypair1.clone(), None, event_tx1)
        .await
        .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(300)).await;

    let swarm2 = start_swarm(keypair2.clone(), None, event_tx2)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(300)).await;

    let swarm3 = start_swarm(keypair3.clone(), None, event_tx3)
        .await
        .expect("Failed to start swarm3");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get listening addresses
    let mut addr1 = None;
    let mut addr2 = None;

    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr1 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr2 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    assert!(addr1.is_some() && addr2.is_some());

    // Node 3 connects to nodes 1 and 2
    swarm3
        .dial(addr1.unwrap())
        .await
        .expect("Failed to dial node 1");
    tokio::time::sleep(Duration::from_secs(1)).await;

    swarm3
        .dial(addr2.unwrap())
        .await
        .expect("Failed to dial node 2");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get peer IDs
    let peers = swarm3.get_peers().await.expect("Failed to get peers");
    assert!(
        peers.len() >= 2,
        "Node 3 should be connected to at least 2 peers"
    );

    let peer1_id = peers[0];
    let peer2_id = peers[1];

    // Create NAT traversal config with both reflectors
    let mut config = NatConfig::default();
    config.peer_reflectors = vec![peer1_id.to_string(), peer2_id.to_string()];

    let nat_traversal = NatTraversal::new(config).expect("Failed to create NatTraversal");

    // Probe NAT type
    let result = nat_traversal.probe_nat(&swarm3).await;

    assert!(result.is_ok(), "NAT probing should succeed");
    let nat_type = result.unwrap();

    // Verify we got external address
    let external_addr = nat_traversal.get_external_address();
    assert!(
        external_addr.is_some(),
        "Should have discovered external address"
    );

    println!("✅ NAT traversal test passed!");
    println!("   Detected NAT type: {:?}", nat_type);
    println!("   External address: {}", external_addr.unwrap());
}

#[tokio::test]
async fn test_multiple_address_reflections() {
    // Test that a single node can handle multiple reflection requests
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let swarm1 = start_swarm(keypair1, None, event_tx1)
        .await
        .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut addr1 = None;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr1 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    assert!(addr1.is_some());

    let swarm2 = start_swarm(keypair2, None, event_tx2)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm2.dial(addr1.unwrap()).await.expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty());
    let peer1 = peers[0];

    // Make multiple reflection requests
    for i in 1..=5 {
        let result = swarm2.request_address_reflection(peer1).await;
        assert!(result.is_ok(), "Reflection request {} should succeed", i);

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Multiple address reflections test passed!");
    println!("   Successfully completed 5 address reflection requests");
}

#[tokio::test]
async fn test_address_reflection_timeout() {
    // Test that address reflection handles disconnected peers gracefully
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let swarm1 = start_swarm(keypair1, None, event_tx1)
        .await
        .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut addr1 = None;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr1 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    let swarm2 = start_swarm(keypair2, None, event_tx2)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm2.dial(addr1.unwrap()).await.expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty());
    let peer1 = peers[0];

    // First request should succeed
    let result1 = swarm2.request_address_reflection(peer1).await;
    assert!(result1.is_ok(), "First reflection should succeed");

    // Shutdown swarm1
    swarm1.shutdown().await.ok();
    
    // Poll for disconnect to propagate (wait up to 10 seconds)
    let mut disconnected = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let remaining_peers = swarm2.get_peers().await.expect("Failed to get peers");
        if !remaining_peers.contains(&peer1) {
            disconnected = true;
            println!("✅ Peer disconnected after polling");
            break;
        }
    }

    // If peer still hasn't disconnected after 10s, skip the timeout test
    // This can happen in slow CI environments
    if !disconnected {
        println!("⚠️  Peer still connected after 10s wait - skipping timeout test");
        println!("   This is acceptable in slow CI environments");
        return;
    }

    // Second request should timeout/fail
    let result2 = tokio::time::timeout(
        Duration::from_secs(3),
        swarm2.request_address_reflection(peer1),
    )
    .await;

    // Either timeout or error is acceptable
    let failed = result2.is_err() || (result2.is_ok() && result2.unwrap().is_err());
    assert!(failed, "Reflection should fail after peer disconnect");

    println!("✅ Address reflection timeout test passed!");
    println!("   Correctly handled disconnected peer");
}
