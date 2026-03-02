use libp2p::{identity::Keypair, Multiaddr, PeerId};
use scmessenger_core::transport::{start_swarm, SwarmEvent};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

async fn wait_for_tcp_listener(
    rx: &mut mpsc::Receiver<SwarmEvent>,
    max_wait: Duration,
) -> Multiaddr {
    timeout(max_wait, async {
        loop {
            match rx.recv().await {
                Some(SwarmEvent::ListeningOn(addr)) if addr.to_string().contains("/tcp/") => {
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

async fn wait_for_peer_discovered(
    rx: &mut mpsc::Receiver<SwarmEvent>,
    expected_peer: PeerId,
    max_wait: Duration,
) {
    timeout(max_wait, async {
        loop {
            match rx.recv().await {
                Some(SwarmEvent::PeerDiscovered(peer_id)) if peer_id == expected_peer => return,
                Some(_) => {}
                None => panic!("event channel closed while waiting for peer discovery"),
            }
        }
    })
    .await
    .expect("timed out waiting for peer discovery");
}

async fn wait_for_envelope(
    rx: &mut mpsc::Receiver<SwarmEvent>,
    expected_payload: &[u8],
    max_wait: Duration,
) -> PeerId {
    timeout(max_wait, async {
        loop {
            match rx.recv().await {
                Some(SwarmEvent::MessageReceived {
                    peer_id,
                    envelope_data,
                }) if envelope_data == expected_payload => return peer_id,
                Some(_) => {}
                None => panic!("event channel closed while waiting for message"),
            }
        }
    })
    .await
    .expect("timed out waiting for message delivery")
}

#[tokio::test]
#[ignore = "requires libp2p socket permissions; run with --include-ignored"]
async fn offline_recipient_receives_after_reconnect_without_sender_resend() {
    let relay_key = Keypair::generate_ed25519();
    let sender_key = Keypair::generate_ed25519();
    let recipient_key = Keypair::generate_ed25519();

    let relay_peer_id = relay_key.public().to_peer_id();
    let recipient_peer_id = recipient_key.public().to_peer_id();

    let (relay_tx, mut relay_rx) = mpsc::channel(256);
    let relay_handle = start_swarm(relay_key, None, relay_tx, false)
        .await
        .expect("failed to start relay");
    let relay_addr = wait_for_tcp_listener(&mut relay_rx, Duration::from_secs(10)).await;

    let (sender_tx, mut sender_rx) = mpsc::channel(256);
    let sender_handle = start_swarm(sender_key, None, sender_tx, false)
        .await
        .expect("failed to start sender");
    let _sender_addr = wait_for_tcp_listener(&mut sender_rx, Duration::from_secs(10)).await;

    let relay_full_addr: Multiaddr = format!("{}/p2p/{}", relay_addr, relay_peer_id)
        .parse()
        .expect("invalid relay multiaddr");
    sender_handle
        .dial(relay_full_addr.clone())
        .await
        .expect("sender failed to dial relay");
    wait_for_peer_discovered(&mut sender_rx, relay_peer_id, Duration::from_secs(15)).await;

    let payload = b"ws3-relay-custody-offline-to-reconnect".to_vec();

    // Recipient is offline here. WS3 requires relay custody acceptance instead of
    // rejecting purely because the destination is disconnected.
    let send_result = timeout(
        Duration::from_secs(25),
        sender_handle.send_message(recipient_peer_id, payload.clone()),
    )
    .await
    .expect("sender send timed out");
    assert!(
        send_result.is_ok(),
        "relay should accept custody while recipient is offline"
    );

    // Recipient reconnects later and should receive the message without sender resend.
    let (recipient_tx, mut recipient_rx) = mpsc::channel(256);
    let recipient_handle = start_swarm(recipient_key, None, recipient_tx, false)
        .await
        .expect("failed to start recipient");
    let _recipient_addr = wait_for_tcp_listener(&mut recipient_rx, Duration::from_secs(10)).await;

    recipient_handle
        .dial(relay_full_addr.clone())
        .await
        .expect("recipient failed to dial relay");
    wait_for_peer_discovered(&mut recipient_rx, relay_peer_id, Duration::from_secs(15)).await;

    let delivered_from =
        wait_for_envelope(&mut recipient_rx, &payload, Duration::from_secs(30)).await;
    assert_eq!(delivered_from, relay_peer_id);

    sender_handle.shutdown().await.ok();
    relay_handle.shutdown().await.ok();
    recipient_handle.shutdown().await.ok();
}
