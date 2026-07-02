use scmessenger_core::*;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct MockBridge;

impl PlatformBridge for MockBridge {
    fn on_battery_changed(&self, _battery_pct: u8, _is_charging: bool) {}
    fn on_network_changed(&self, _has_wifi: bool, _has_cellular: bool) {}
    fn on_motion_changed(&self, _motion: MotionState) {}
    fn on_ble_data_received(&self, _peer_id: String, _data: Vec<u8>) {}
    fn on_entering_background(&self) {}
    fn on_entering_foreground(&self) {}
    fn send_ble_packet(&self, _peer_id: String, _data: Vec<u8>) {}
    fn on_proximity_data_received(
        &self,
        _peer_id: String,
        _transport: ProximityTransport,
        _data: Vec<u8>,
    ) {
    }
    fn send_proximity_packet(
        &self,
        _peer_id: String,
        _transport: ProximityTransport,
        _data: Vec<u8>,
    ) {
    }
    fn wifi_aware_publish(&self, _service_name: String, _service_info: Vec<u8>) -> bool {
        true
    }
    fn wifi_aware_subscribe(&self, _service_name: String) -> bool {
        true
    }
    fn wifi_aware_create_data_path(&self, _peer_id: String, _pmk: Vec<u8>) -> bool {
        true
    }
    fn wifi_aware_stop(&self) {}
    fn wifi_direct_discover_peers(&self) -> bool {
        true
    }
    fn wifi_direct_stop_discovery(&self) {}
    fn wifi_direct_connect(&self, _device_address: String) -> bool {
        true
    }
    fn wifi_direct_create_group(&self, _group_name: String) -> bool {
        true
    }
    fn wifi_direct_remove_group(&self) {}
}

/// Poll a running swarm's listener list until it reports a plain loopback TCP
/// address (i.e. not the QUIC or `/ws` listeners also started by
/// `start_swarm`), returning the ephemeral port the OS actually assigned.
fn wait_for_loopback_tcp_port(
    rt: &tokio::runtime::Handle,
    handle: &transport::SwarmHandle,
) -> Option<u16> {
    use libp2p::multiaddr::Protocol;

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let listeners = rt.block_on(handle.get_listeners()).unwrap_or_default();
        for addr in listeners {
            let mut iter = addr.iter();
            let is_loopback_v4 = matches!(iter.next(), Some(Protocol::Ip4(ip)) if ip.is_loopback());
            if is_loopback_v4 {
                if let Some(Protocol::Tcp(port)) = iter.next() {
                    // Exclude the `/ws` listener: only the plain TCP one matches
                    // what MeshService's dial path constructs.
                    if iter.next().is_none() {
                        return Some(port);
                    }
                }
            }
        }
        sleep(Duration::from_millis(100));
    }
    None
}

#[test]
fn test_wifi_aware_peer_discovered_triggers_data_path_and_dial() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();

    // WiFi Aware is off by default (MeshSettings::default().wifi_aware_enabled
    // == false); MeshService::start() only constructs a WifiAwareTransport
    // (and thus only wires on_wifi_aware_peer_discovered's dial path) when
    // the setting is on, so persist it before starting.
    MeshSettingsManager::new(path.clone())
        .save(MeshSettings {
            wifi_aware_enabled: true,
            ..MeshSettings::default()
        })
        .expect("Failed to save mesh settings");

    let service = Arc::new(MeshService::with_storage(
        MeshServiceConfig {
            discovery_interval_ms: 100,
            battery_floor_pct: 10,
        },
        path,
    ));

    // Set mock platform bridge
    service.set_platform_bridge(Some(Box::new(MockBridge)));

    // Start service
    service.clone().start().expect("Failed to start service");

    // Retrieve the SwarmBridge and set a mock SwarmHandle
    let swarm_bridge = service.get_swarm_bridge();

    // We can get the runtime handle to block_on start_swarm
    let rt = swarm_bridge.get_runtime_handle();
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let initiator_peer_id = keypair.public().to_peer_id();
    let (event_tx, _event_rx) = tokio::sync::mpsc::channel(256);

    let swarm_handle = rt.block_on(async {
        transport::start_swarm(
            keypair,
            Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
            event_tx,
            None,
            false,
            // Disable mDNS/passive auto-discovery: both swarms run on the
            // same host, so with the default Open mode they'd find each
            // other on their own and the test would pass even if the
            // WiFi-Aware-triggered dial path were completely broken.
            Some(transport::DiscoveryConfig::new(
                transport::DiscoveryMode::Silent,
            )),
            scmessenger_core::transport::default_routing_engine_handle(),
        )
        .await
        .expect("Failed to start swarm")
    });

    swarm_bridge.set_handle(swarm_handle);

    // A second, independently-running libp2p node standing in for the peer
    // WiFi Aware "discovers". Using a real swarm (rather than only checking
    // that a callback fired) lets us assert genuine end-to-end connectivity:
    // dial + noise handshake + peer registration on both sides.
    let responder_keypair = libp2p::identity::Keypair::generate_ed25519();
    let responder_peer_id = responder_keypair.public().to_peer_id();
    let (responder_event_tx, _responder_event_rx) = tokio::sync::mpsc::channel(256);

    let responder_handle = rt.block_on(async {
        transport::start_swarm(
            responder_keypair,
            Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
            responder_event_tx,
            None,
            false,
            Some(transport::DiscoveryConfig::new(
                transport::DiscoveryMode::Silent,
            )),
            scmessenger_core::transport::default_routing_engine_handle(),
        )
        .await
        .expect("Failed to start responder swarm")
    });

    let responder_port = wait_for_loopback_tcp_port(&rt, &responder_handle)
        .expect("Responder swarm never reported a loopback TCP listener");

    // Simulate discovering the responder peer via Wi-Fi Aware. MeshService
    // spawns a task that calls into MockBridge (create_data_path succeeds
    // synchronously) and then awaits the platform's data-path confirmation
    // via a one-shot channel that's registered inside that spawned task —
    // which races with this thread. If the confirmation is delivered before
    // the spawned task registers its receiver, it's silently dropped (this
    // mirrors the real platform callback: delivery is best-effort, not
    // queued), so each retry re-fires peer-discovered too. That spawns a
    // fresh attempt with a fresh receiver rather than only re-sending a
    // confirmation into a channel that may never be listened on again.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut connected = false;
    while Instant::now() < deadline {
        service.on_wifi_aware_peer_discovered(responder_peer_id.to_string(), vec![1, 2, 3], -50);
        // Give the newly spawned task a chance to register its receiver.
        sleep(Duration::from_millis(50));
        service.on_wifi_aware_data_path_confirmed(
            responder_peer_id.to_string(),
            "127.0.0.1".to_string(),
            responder_port,
        );
        // Give the dial a moment to complete.
        sleep(Duration::from_millis(150));

        let peers = swarm_bridge.get_peers();
        if peers.contains(&responder_peer_id.to_string()) {
            connected = true;
            break;
        }
    }

    assert!(
        connected,
        "initiator swarm never dialed/connected to the WiFi-Aware-discovered peer \
         (expected {} in get_peers())",
        responder_peer_id
    );

    // Confirm it's a real, mutual libp2p connection rather than a one-sided
    // dial attempt: the responder's swarm should see the initiator too.
    let responder_peers = rt
        .block_on(responder_handle.get_peers())
        .expect("Failed to query responder peers");
    assert!(
        responder_peers.contains(&initiator_peer_id),
        "responder swarm never saw an inbound connection from the initiator \
         (expected {} in {:?})",
        initiator_peer_id,
        responder_peers
    );

    // Clean up
    service.stop();
}
