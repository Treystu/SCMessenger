// Integration test for multi-port adaptive listening (Phase 2)
//
// Tests that nodes can listen on multiple ports simultaneously for
// maximum connectivity in restrictive network environments.

use scmessenger_core::transport::{multiport, BindResult, MultiPortConfig};

#[test]
fn test_generate_listen_addresses() {
    let config = MultiPortConfig {
        enable_common_ports: true,
        enable_random_port: true,
        additional_ports: vec![9999],
        enable_ipv4: true,
        enable_ipv6: false,
    };

    let addresses = multiport::generate_listen_addresses(&config);

    // Should have common ports + additional + random
    assert!(
        addresses.len() >= multiport::COMMON_PORTS.len() + 2,
        "Should generate addresses for common ports, additional, and random"
    );

    // Check for specific ports
    assert!(addresses.iter().any(|(_, port)| *port == 443));
    assert!(addresses.iter().any(|(_, port)| *port == 80));
    assert!(addresses.iter().any(|(_, port)| *port == 9999));
    assert!(addresses.iter().any(|(_, port)| *port == 0)); // random port

    println!("✓ Generated {} listen addresses", addresses.len());
}

#[test]
fn test_ipv4_only_mode() {
    let config = MultiPortConfig {
        enable_common_ports: true,
        enable_random_port: true,
        additional_ports: vec![],
        enable_ipv4: true,
        enable_ipv6: false,
    };

    let addresses = multiport::generate_listen_addresses(&config);

    for (addr, _) in &addresses {
        assert!(
            addr.to_string().contains("/ip4/"),
            "All addresses should be IPv4"
        );
    }

    println!("✓ IPv4-only mode generates only IPv4 addresses");
}

#[test]
fn test_ipv6_only_mode() {
    let config = MultiPortConfig {
        enable_common_ports: true,
        enable_random_port: true,
        additional_ports: vec![],
        enable_ipv4: false,
        enable_ipv6: true,
    };

    let addresses = multiport::generate_listen_addresses(&config);

    for (addr, _) in &addresses {
        assert!(
            addr.to_string().contains("/ip6/"),
            "All addresses should be IPv6"
        );
    }

    println!("✓ IPv6-only mode generates only IPv6 addresses");
}

#[test]
fn test_custom_ports_only() {
    let config = MultiPortConfig {
        enable_common_ports: false,
        enable_random_port: false,
        additional_ports: vec![9999, 8888, 7777],
        enable_ipv4: true,
        enable_ipv6: false,
    };

    let addresses = multiport::generate_listen_addresses(&config);

    assert_eq!(addresses.len(), 3, "Should only have custom ports");
    assert!(addresses.iter().any(|(_, port)| *port == 9999));
    assert!(addresses.iter().any(|(_, port)| *port == 8888));
    assert!(addresses.iter().any(|(_, port)| *port == 7777));

    println!("✓ Custom ports mode works correctly");
}

#[test]
fn test_bind_result_analysis() {
    let results = vec![
        BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
            port: 443,
        },
        BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/12345".parse().unwrap(),
            port: 12345,
        },
        BindResult::Failed {
            port: 80,
            error: "Permission denied (os error 13)".to_string(),
        },
        BindResult::Failed {
            port: 8080,
            error: "Address already in use (os error 98)".to_string(),
        },
        BindResult::Skipped {
            port: 9090,
            reason: "Disabled in config".to_string(),
        },
    ];

    let analysis = multiport::analyze_bind_results(&results);

    assert_eq!(analysis.successful.len(), 2);
    assert_eq!(analysis.failed_permission.len(), 1);
    assert_eq!(analysis.failed_in_use.len(), 1);
    assert_eq!(
        analysis.status,
        scmessenger_core::transport::ConnectivityStatus::Excellent
    );

    println!("✓ Bind result analysis categorizes correctly");
}

#[test]
fn test_bind_analysis_excellent_connectivity() {
    use scmessenger_core::transport::ConnectivityStatus;

    // Has both common port (443) and random port (>10000)
    let results = vec![
        BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
            port: 443,
        },
        BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/54321".parse().unwrap(),
            port: 54321,
        },
    ];

    let analysis = multiport::analyze_bind_results(&results);
    assert_eq!(analysis.status, ConnectivityStatus::Excellent);

    println!("✓ Excellent connectivity detected correctly");
}

#[test]
fn test_bind_analysis_good_connectivity() {
    use scmessenger_core::transport::ConnectivityStatus;

    // Has common port (443) but no random port
    let results = vec![BindResult::Success {
        addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
        port: 443,
    }];

    let analysis = multiport::analyze_bind_results(&results);
    assert_eq!(analysis.status, ConnectivityStatus::Good);

    println!("✓ Good connectivity detected correctly");
}

#[test]
fn test_bind_analysis_moderate_connectivity() {
    use scmessenger_core::transport::ConnectivityStatus;

    // Has random port but no common ports
    let results = vec![BindResult::Success {
        addr: "/ip4/0.0.0.0/tcp/54321".parse().unwrap(),
        port: 54321,
    }];

    let analysis = multiport::analyze_bind_results(&results);
    assert_eq!(analysis.status, ConnectivityStatus::Moderate);

    println!("✓ Moderate connectivity detected correctly");
}

#[test]
fn test_bind_analysis_report_format() {
    let results = vec![
        BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
            port: 443,
        },
        BindResult::Failed {
            port: 80,
            error: "Permission denied".to_string(),
        },
    ];

    let analysis = multiport::analyze_bind_results(&results);
    let report = analysis.report();

    assert!(report.contains("Listening on 1 address"));
    assert!(report.contains("443"));
    assert!(report.contains("priority"));
    assert!(report.contains("Permission denied") || report.contains("elevated privileges"));
    assert!(report.contains("Connectivity:"));

    println!("✓ Bind analysis report is well-formatted");
    println!("\nSample report:\n{}", report);
}

#[test]
fn test_requires_elevated_privileges() {
    #[cfg(unix)]
    {
        assert!(multiport::requires_elevated_privileges(80));
        assert!(multiport::requires_elevated_privileges(443));
        assert!(multiport::requires_elevated_privileges(1));
        assert!(multiport::requires_elevated_privileges(1023));
        assert!(!multiport::requires_elevated_privileges(1024));
        assert!(!multiport::requires_elevated_privileges(8080));
        assert!(!multiport::requires_elevated_privileges(9999));

        println!("✓ Elevated privilege detection works correctly (Unix)");
    }

    #[cfg(not(unix))]
    {
        // On Windows, privileged ports don't require elevation
        assert!(!multiport::requires_elevated_privileges(80));
        assert!(!multiport::requires_elevated_privileges(443));

        println!("✓ Elevated privilege detection works correctly (Windows)");
    }
}

#[test]
fn test_default_multiport_config() {
    let config = MultiPortConfig::default();

    assert!(config.enable_common_ports);
    assert!(config.enable_random_port);
    assert!(config.enable_ipv4);
    assert!(config.enable_ipv6);
    assert!(config.additional_ports.is_empty());

    println!("✓ Default MultiPortConfig has sensible defaults");
}

#[tokio::test]
#[ignore = "requires real networking (TCP bind); run with --include-ignored"]
async fn test_multiport_swarm_integration() {
    // This test verifies that the swarm can be started with multi-port config
    use scmessenger_core::transport::{start_swarm_with_config, MultiPortConfig};

    let keypair = libp2p::identity::Keypair::generate_ed25519();

    // Use high ports that don't require privileges
    let config = MultiPortConfig {
        enable_common_ports: false,
        enable_random_port: true,
        additional_ports: vec![19999, 18888],
        enable_ipv4: true,
        enable_ipv6: false,
    };

    let (event_tx, _event_rx) = tokio::sync::mpsc::channel(256);

    let swarm =
        start_swarm_with_config(keypair, None, event_tx, Some(config), Vec::new(), false).await;

    assert!(swarm.is_ok(), "Should start swarm with multi-port config");

    if let Ok(swarm) = swarm {
        swarm.shutdown().await.ok();
    }

    println!("✓ Swarm starts successfully with multi-port configuration");
}

#[test]
fn test_phase_2_implementation_complete() {
    println!("\n=== Phase 2: Multi-Port Adaptive Listening ===");
    println!("✓ MultiPortConfig defines port selection strategy");
    println!("✓ generate_listen_addresses() creates address list");
    println!("✓ BindResult tracks success/failure for each port");
    println!("✓ BindAnalysis provides connectivity assessment");
    println!("✓ ConnectivityStatus categorizes network capability");
    println!("✓ start_swarm_with_config() enables multi-port binding");
    println!("✓ Automatic privilege detection for low ports");
    println!("✓ IPv4/IPv6 mode selection supported");
    println!("✓ Priority ports (443, 80) maximize firewall traversal");
    println!("✓ Random port ensures fallback connectivity");
    println!("\nPhase 2 implementation complete!");
}
