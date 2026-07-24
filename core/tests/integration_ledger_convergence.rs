// Integration test for ledger convergence between two nodes (FARM WS-FARM-F1)
//
// Proves two in-process nodes converge their peer ledgers via ledger_exchange
// after connecting: a pre-existing entry in node 1's ledger, never directly
// dialed or discovered by node 2, still ends up in node 2's ledger purely via
// the ledger_exchange protocol.
//
// Test is #[ignore] by default (real networking) - run with:
//   cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use scmessenger_core::mobile_bridge::LedgerManager;
use scmessenger_core::transport::behaviour::SharedPeerEntry;
use scmessenger_core::transport::swarm::{start_swarm, SwarmEvent2, SwarmHandle};
use std::sync::Arc;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::sync::mpsc;

#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_ledger_convergence_between_nodes() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init()
        .ok();

    let keypair1 = Keypair::generate_ed25519();
    let peer_id1 = libp2p::PeerId::from(keypair1.public());
    let keypair2 = Keypair::generate_ed25519();
    let peer_id2 = libp2p::PeerId::from(keypair2.public());

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(
        keypair1,
        None,
        event_tx1,
        None,
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm1");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Collect all ListeningOn events in a bounded window so we can pick the
    // plain, directly-dialable TCP listen address. We must be careful because
    // multiple ListeningOn events arrive. We need to select a plain TCP
    // listener, preferring localhost (127.0.0.1 or ::1), and avoiding port 9002 (fixed WS port)
    // or any address containing /ws, /quic-v1, /p2p-circuit.
    let mut all_addrs: Vec<libp2p::Multiaddr> = Vec::new();
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                all_addrs.push(addr);
                let has_loopback_tcp = all_addrs.iter().any(|a| {
                    let s = a.to_string();
                    s.contains("/127.0.0.1/")
                        && s.contains("/tcp/")
                        && !s.contains("/ws")
                        && !s.contains("/quic")
                });
                if has_loopback_tcp {
                    break;
                }
            }
        }
    })
    .await
    .ok();

    assert!(
        !all_addrs.is_empty(),
        "Node 1 should have at least one listen address"
    );

    let node1_addr = select_dialable_tcp_loopback(&all_addrs)
        .expect("No suitable plain TCP loopback address found among node1 listeners");

    let swarm2: SwarmHandle = start_swarm(
        keypair2,
        None,
        event_tx2,
        None,
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm2");

    tokio::time::sleep(Duration::from_millis(1500)).await;

    // LedgerManager derives uniffi::Object, not Clone - Arc-wrap so both the
    // spawned event-loop task and this function's final assertion can share it.
    let ledger_file1 = NamedTempFile::new().expect("Failed to create temp file for ledger1");
    let ledger_file2 = NamedTempFile::new().expect("Failed to create temp file for ledger2");
    let ledger1 = Arc::new(LedgerManager::new(
        ledger_file1.path().to_str().unwrap().to_string(),
    ));
    let ledger2 = Arc::new(LedgerManager::new(
        ledger_file2.path().to_str().unwrap().to_string(),
    ));

    // Seed node 1's ledger with an entry node 2 never learns any other way.
    ledger1.record_connection(
        "/ip4/1.2.3.4/tcp/9000".to_string(),
        "QmFakePeerXYZ".to_string(),
    );

    // Node 2's event loop: record whatever ledger entries it receives.
    let ledger2_for_task = ledger2.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                for entry in entries {
                    if let Some(peer_id) = entry.last_peer_id {
                        ledger2_for_task.record_connection(entry.multiaddr, peer_id);
                    }
                }
            }
        }
    });

    // Append /p2p/<peer_id1> so libp2p can associate the dial with a known PeerId.
    // Without this suffix, dial() succeeds but libp2p reports "no addresses for peer"
    // because it cannot track the connection against a specific PeerId.
    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    swarm2.dial(dial_addr).await.expect("Failed to dial");

    // Wait for connection handshake and protocols to negotiate
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Trigger the ledger share directly from Node 1 to Node 2 now that they are connected
    let entries = ledger1.dialable_addresses();
    let shared_entries: Vec<SharedPeerEntry> = entries
        .into_iter()
        .map(|entry| SharedPeerEntry {
            multiaddr: entry.multiaddr,
            last_peer_id: entry.peer_id,
            last_seen: entry.last_seen.unwrap_or(0),
            known_topics: entry.topics,
        })
        .collect();
    swarm1
        .share_ledger(peer_id2, shared_entries)
        .await
        .expect("Failed to share ledger");

    // Let the test wait for 3 seconds so the ledger is received on Node 2
    tokio::time::sleep(Duration::from_secs(3)).await;

    let dialable_addresses = ledger2.dialable_addresses();
    let has_converged_entry = dialable_addresses
        .iter()
        .any(|entry| entry.multiaddr == "/ip4/1.2.3.4/tcp/9000");

    assert!(
        has_converged_entry,
        "Node 2's ledger should contain node 1's pre-existing entry via ledger_exchange"
    );
}

/// Select a plain TCP loopback address from a list of ListeningOn multiaddrs.
///
/// Picks the first address matching ALL of:
///   - contains /ip4/127.0.0.1  (loopback -- this is an in-process localhost test)
///   - contains /tcp/<port>     (plain TCP, not QUIC)
///   - port != 9002             (hardcoded WS listener port shared by both nodes)
///   - no /ws, /wss, /quic-v1, /p2p-circuit protocol components
///
/// Falls back to any 127.0.0.1/tcp address if the port-9002 filter is too
/// aggressive (should not happen in practice), then to any /ip4 + /tcp addr.
fn select_dialable_tcp_loopback(addrs: &[Multiaddr]) -> Option<Multiaddr> {
    // Classify each address.
    let mut loopback_ephemeral: Option<Multiaddr> = None;
    let mut loopback_any_tcp: Option<Multiaddr> = None;
    let mut any_plain_tcp: Option<Multiaddr> = None;

    for addr in addrs {
        let s = addr.to_string();

        // Reject non-plain-TCP transports.
        if s.contains("/ws")
            || s.contains("/wss")
            || s.contains("/quic")
            || s.contains("/p2p-circuit")
        {
            continue;
        }

        // Must have /tcp.
        let mut has_tcp = false;
        let mut tcp_port: u16 = 0;
        let mut is_loopback = false;

        for proto in addr.iter() {
            match proto {
                libp2p::multiaddr::Protocol::Ip4(ip) => {
                    if ip == std::net::Ipv4Addr::LOCALHOST {
                        is_loopback = true;
                    }
                }
                libp2p::multiaddr::Protocol::Tcp(p) => {
                    has_tcp = true;
                    tcp_port = p;
                }
                _ => {}
            }
        }

        if !has_tcp {
            continue;
        }

        if any_plain_tcp.is_none() {
            any_plain_tcp = Some(addr.clone());
        }

        if is_loopback {
            if loopback_any_tcp.is_none() {
                loopback_any_tcp = Some(addr.clone());
            }
            // Prefer ephemeral port (not the hardcoded WS port 9002).
            if tcp_port != 9002 && loopback_ephemeral.is_none() {
                loopback_ephemeral = Some(addr.clone());
            }
        }
    }

    loopback_ephemeral.or(loopback_any_tcp).or(any_plain_tcp)
}
