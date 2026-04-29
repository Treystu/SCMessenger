use libp2p::{identity::Keypair, Multiaddr, PeerId};
use scmessenger_core::identity::IdentityKeys;
use scmessenger_core::transport::swarm::SwarmEvent2;
use scmessenger_core::transport::{start_swarm, DeregistrationRequest, RegistrationRequest};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

async fn wait_for_tcp_listener(
    rx: &mut mpsc::Receiver<SwarmEvent2>,
    max_wait: Duration,
) -> Multiaddr {
    timeout(max_wait, async {
        loop {
            match rx.recv().await {
                Some(SwarmEvent2::ListeningOn(addr)) if addr.to_string().contains("/tcp/") => {
                    return addr;
                }
                Some(_) => {}
                None => panic!("event channel closed while waiting for listener"),
            }
        }
    })
    .await
    .expect("timed out waiting for listener")
}

async fn wait_for_peer_ready(
    rx: &mut mpsc::Receiver<SwarmEvent2>,
    expected_peer: PeerId,
    max_wait: Duration,
) {
    timeout(max_wait, async {
        let mut discovered = false;
        let mut identified = false;
        loop {
            match rx.recv().await {
                Some(SwarmEvent2::PeerDiscovered(peer_id)) if peer_id == expected_peer => {
                    discovered = true;
                }
                Some(SwarmEvent2::PeerIdentified { peer_id, .. }) if peer_id == expected_peer => {
                    identified = true;
                }
                Some(_) => {}
                None => panic!("event channel closed while waiting for peer readiness"),
            }
            if discovered && identified {
                return;
            }
        }
    })
    .await
    .expect("timed out waiting for peer readiness");
}

#[tokio::test]
async fn registration_protocol_accepts_valid_signed_registration_request() {
    let receiver_keypair = Keypair::generate_ed25519();
    let receiver_peer_id = receiver_keypair.public().to_peer_id();

    let sender_identity = IdentityKeys::generate();
    let sender_keypair = sender_identity.to_libp2p_keypair().unwrap();
    let sender_peer_id = sender_keypair.public().to_peer_id();

    let (receiver_tx, mut receiver_rx) = mpsc::channel(256);
    let receiver_handle = start_swarm(
        receiver_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        receiver_tx,
        None,
        false,
    )
    .await
    .expect("failed to start receiver swarm");
    let receiver_addr = wait_for_tcp_listener(&mut receiver_rx, Duration::from_secs(10)).await;

    let (sender_tx, mut sender_rx) = mpsc::channel(256);
    let sender_handle = start_swarm(
        sender_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        sender_tx,
        None,
        false,
    )
    .await
    .expect("failed to start sender swarm");
    let _sender_addr = wait_for_tcp_listener(&mut sender_rx, Duration::from_secs(10)).await;

    sender_handle
        .dial(receiver_addr)
        .await
        .expect("sender failed to dial receiver");
    wait_for_peer_ready(&mut sender_rx, receiver_peer_id, Duration::from_secs(15)).await;
    wait_for_peer_ready(&mut receiver_rx, sender_peer_id, Duration::from_secs(15)).await;

    let request = RegistrationRequest::new_signed(
        &sender_identity,
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        1_731_000_000,
    )
    .expect("failed to sign registration request");

    sender_handle
        .register_identity(receiver_peer_id, request)
        .await
        .expect("valid registration request should be accepted");

    sender_handle.shutdown().await.ok();
    receiver_handle.shutdown().await.ok();
}

#[tokio::test]
async fn registration_protocol_rejects_malformed_identity_id_without_mutation() {
    let receiver_keypair = Keypair::generate_ed25519();
    let receiver_peer_id = receiver_keypair.public().to_peer_id();

    let sender_identity = IdentityKeys::generate();
    let sender_keypair = sender_identity.to_libp2p_keypair().unwrap();
    let sender_peer_id = sender_keypair.public().to_peer_id();

    let (receiver_tx, mut receiver_rx) = mpsc::channel(256);
    let receiver_handle = start_swarm(
        receiver_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        receiver_tx,
        None,
        false,
    )
    .await
    .expect("failed to start receiver swarm");
    let receiver_addr = wait_for_tcp_listener(&mut receiver_rx, Duration::from_secs(10)).await;

    let (sender_tx, mut sender_rx) = mpsc::channel(256);
    let sender_handle = start_swarm(
        sender_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        sender_tx,
        None,
        false,
    )
    .await
    .expect("failed to start sender swarm");
    let _sender_addr = wait_for_tcp_listener(&mut sender_rx, Duration::from_secs(10)).await;

    sender_handle
        .dial(receiver_addr)
        .await
        .expect("sender failed to dial receiver");
    wait_for_peer_ready(&mut sender_rx, receiver_peer_id, Duration::from_secs(15)).await;
    wait_for_peer_ready(&mut receiver_rx, sender_peer_id, Duration::from_secs(15)).await;

    let mut request = RegistrationRequest::new_signed(
        &sender_identity,
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        1_731_000_000,
    )
    .expect("failed to sign registration request");
    request.payload.identity_id = "not-a-valid-identity-id".to_string();

    let error = sender_handle
        .register_identity(receiver_peer_id, request)
        .await
        .expect_err("malformed registration request must be rejected")
        .to_string();
    assert!(
        error.contains("registration_identity_id_invalid"),
        "unexpected error: {}",
        error
    );

    sender_handle.shutdown().await.ok();
    receiver_handle.shutdown().await.ok();
}

#[tokio::test]
async fn registration_protocol_rejects_tampered_signed_deregistration_request() {
    let receiver_keypair = Keypair::generate_ed25519();
    let receiver_peer_id = receiver_keypair.public().to_peer_id();

    let sender_identity = IdentityKeys::generate();
    let sender_keypair = sender_identity.to_libp2p_keypair().unwrap();
    let sender_peer_id = sender_keypair.public().to_peer_id();

    let (receiver_tx, mut receiver_rx) = mpsc::channel(256);
    let receiver_handle = start_swarm(
        receiver_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        receiver_tx,
        None,
        false,
    )
    .await
    .expect("failed to start receiver swarm");
    let receiver_addr = wait_for_tcp_listener(&mut receiver_rx, Duration::from_secs(10)).await;

    let (sender_tx, mut sender_rx) = mpsc::channel(256);
    let sender_handle = start_swarm(
        sender_keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        sender_tx,
        None,
        false,
    )
    .await
    .expect("failed to start sender swarm");
    let _sender_addr = wait_for_tcp_listener(&mut sender_rx, Duration::from_secs(10)).await;

    sender_handle
        .dial(receiver_addr)
        .await
        .expect("sender failed to dial receiver");
    wait_for_peer_ready(&mut sender_rx, receiver_peer_id, Duration::from_secs(15)).await;
    wait_for_peer_ready(&mut receiver_rx, sender_peer_id, Duration::from_secs(15)).await;

    let mut request = DeregistrationRequest::new_signed(
        &sender_identity,
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        Some("550e8400-e29b-41d4-a716-446655440001".to_string()),
    )
    .expect("failed to sign deregistration request");
    request.payload.target_device_id = Some("550e8400-e29b-41d4-a716-446655440002".to_string());

    let error = sender_handle
        .deregister_identity(receiver_peer_id, request)
        .await
        .expect_err("tampered deregistration request must be rejected")
        .to_string();
    assert!(
        error.contains("deregistration_signature_invalid"),
        "unexpected error: {}",
        error
    );

    sender_handle.shutdown().await.ok();
    receiver_handle.shutdown().await.ok();
}
