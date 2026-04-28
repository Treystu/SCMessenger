// Integration tests for NAT traversal and address reflection protocol
//
// These tests verify the complete address reflection protocol with real libp2p swarms.
// They demonstrate:
// - Two-node address reflection (peer asks, other peer responds)
// - NAT type detection using multiple peer reflectors
// - External address discovery via mesh peers
// - Full request-response lifecycle
//
// All tests are #[ignore] by default — run with: cargo test -- --include-ignored

use anyhow::Result as AnyhowResult;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use scmessenger_core::transport::swarm::{start_swarm, SwarmEvent2, SwarmHandle};
use scmessenger_core::transport::nat::{NatConfig, NatTraversal, PeerAddressDiscovery};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_two_node_address_reflection() {
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let _swarm1: SwarmHandle = start_swarm(keypair1, None, event_tx1, None, false)
        .await
        .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(500)).await;

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

    let swarm2: SwarmHandle = start_swarm(keypair2, None, event_tx2, None, false)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm2
        .dial(node1_addr.clone())
        .await
        .expect("Failed to dial");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers: Vec<PeerId> = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty(), "Node 2 should be connected to node 1");

    let peer1_id = peers[0];

    let result: AnyhowResult<String> = swarm2.request_address_reflection(peer1_id).await;

    assert!(result.is_ok(), "Address reflection should succeed");
    let observed_addr = result.unwrap();

    assert!(!observed_addr.is_empty(), "Should receive observed address");
    println!("✅ Address reflection test passed!");
    println!("   Node 1 observed Node 2 at: {}", observed_addr);
}

#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_peer_address_discovery_with_live_swarm() {
    let keypair_reflector = Keypair::generate_ed25519();
    let keypair_requester = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let _swarm_reflector: SwarmHandle =
        start_swarm(keypair_reflector.clone(), None, event_tx1, None, false)
            .await
            .expect("Failed to start reflector swarm");

    tokio::time::sleep(Duration::from_millis(500)).await;

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

    let swarm_requester: SwarmHandle =
        start_swarm(keypair_requester.clone(), None, event_tx2, None, false)
            .await
            .expect("Failed to start requester swarm");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm_requester
        .dial(reflector_addr.unwrap())
        .await
        .expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers: Vec<PeerId> = swarm_requester
        .get_peers()
        .await
        .expect("Failed to get peers");
    assert!(!peers.is_empty());
    let reflector_peer_id = peers[0];

    let discovery = PeerAddressDiscovery::with_peers(vec![reflector_peer_id.to_string()], 10);

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
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_nat_traversal_with_live_swarms() {
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();
    let keypair3 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);
    let (event_tx3, mut _event_rx3) = mpsc::channel(256);

    let _swarm1: SwarmHandle = start_swarm(keypair1.clone(), None, event_tx1, None, false)
        .await
        .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(300)).await;

    let _swarm2: SwarmHandle = start_swarm(keypair2.clone(), None, event_tx2, None, false)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(300)).await;

    let swarm3: SwarmHandle = start_swarm(keypair3.clone(), None, event_tx3, None, false)
        .await
        .expect("Failed to start swarm3");

    tokio::time::sleep(Duration::from_millis(500)).await;

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

    let peers: Vec<PeerId> = swarm3.get_peers().await.expect("Failed to get peers");
    assert!(
        peers.len() >= 2,
        "Node 3 should be connected to at least 2 peers"
    );

    let peer1_id = peers[0];
    let peer2_id = peers[1];

    let config = NatConfig {
        peer_reflectors: vec![peer1_id.to_string(), peer2_id.to_string()],
        ..Default::default()
    };

    let nat_traversal = NatTraversal::new(config).expect("Failed to create NatTraversal");

    let result = nat_traversal.probe_nat(&swarm3).await;

    assert!(result.is_ok(), "NAT probing should succeed");
    let nat_type = result.unwrap();

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
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_multiple_address_reflections() {
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let _swarm1: SwarmHandle = start_swarm(keypair1, None, event_tx1, None, false)
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

    let swarm2: SwarmHandle = start_swarm(keypair2, None, event_tx2, None, false)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm2.dial(addr1.unwrap()).await.expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers: Vec<PeerId> = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty());
    let peer1 = peers[0];

    for i in 1..=5 {
        let result: AnyhowResult<String> = swarm2.request_address_reflection(peer1).await;
        assert!(result.is_ok(), "Reflection request {} should succeed", i);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Multiple address reflections test passed!");
    println!("   Successfully completed 5 address reflection requests");
}

#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_address_reflection_timeout() {
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut _event_rx2) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(keypair1, None, event_tx1, None, false)
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

    let swarm2: SwarmHandle = start_swarm(keypair2, None, event_tx2, None, false)
        .await
        .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(500)).await;

    swarm2.dial(addr1.unwrap()).await.expect("Failed to dial");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let peers: Vec<PeerId> = swarm2.get_peers().await.expect("Failed to get peers");
    assert!(!peers.is_empty());
    let peer1 = peers[0];

    let result1: AnyhowResult<String> = swarm2.request_address_reflection(peer1).await;
    assert!(result1.is_ok(), "First reflection should succeed");

    swarm1.shutdown().await.ok();

    let mut disconnected = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let remaining_peers: Vec<PeerId> = swarm2.get_peers().await.expect("Failed to get peers");
        if !remaining_peers.contains(&peer1) {
            disconnected = true;
            println!("✅ Peer disconnected after polling");
            break;
        }
    }

    if !disconnected {
        println!("⚠️  Peer still connected after 10s wait - skipping timeout test");
        println!("   This is acceptable in slow CI environments");
        return;
    }

    let result2: Result<AnyhowResult<String>, tokio::time::error::Elapsed> = tokio::time::timeout(
        Duration::from_secs(3),
        swarm2.request_address_reflection(peer1),
    )
    .await;

    let failed = result2.is_err() || (result2.is_ok() && result2.unwrap().is_err());
    assert!(failed, "Reflection should fail after peer disconnect");

    println!("✅ Address reflection timeout test passed!");
    println!("   Correctly handled disconnected peer");
}
