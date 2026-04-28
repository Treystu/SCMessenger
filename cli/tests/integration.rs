use scmessenger_cli::ble_daemon::{BleConfig, BleDaemon, BleError, BleStatus};
use scmessenger_cli::cli::{Cli, Commands, ContactAction};
use clap::Parser;

#[test]
fn test_cli_parse_init_with_nickname() {
    let cli = Cli::parse_from(["scm", "init", "--name", "Alice"]);
    assert!(matches!(cli.command, Commands::Init { name: Some(ref n) } if n == "Alice"));
}

#[test]
fn test_cli_parse_identity_export() {
    let cli = Cli::parse_from(["scm", "identity", "export", "--passphrase", "secret"]);
    assert!(matches!(cli.command, Commands::Identity { action: Some(_) }));
}

#[test]
fn test_cli_parse_contact_add() {
    let cli = Cli::parse_from([
        "scm", "contact", "add",
        "12D3KooTest12345678901234567890123456789012345678901234",
        "abcd1234efgh5678",
        "--name", "Bob",
    ]);
    assert!(matches!(
        cli.command,
        Commands::Contact { action: ContactAction::Add { ref peer_id, name: Some(ref n), .. } }
        if peer_id.starts_with("12D3Koo") && n == "Bob"
    ));
}

#[test]
fn test_cli_parse_contact_search() {
    let cli = Cli::parse_from(["scm", "contact", "search", "alice"]);
    assert!(matches!(
        cli.command,
        Commands::Contact { action: ContactAction::Search { ref query } } if query == "alice"
    ));
}

#[test]
fn test_cli_parse_block_add() {
    let cli = Cli::parse_from(["scm", "block", "add", "test-peer-id"]);
    assert!(matches!(cli.command, Commands::Block { .. }));
}

#[test]
fn test_cli_parse_relay() {
    let cli = Cli::parse_from(["scm", "relay", "--listen", "/ip4/127.0.0.1/tcp/9001"]);
    assert!(matches!(
        cli.command,
        Commands::Relay { ref listen, .. } if listen == "/ip4/127.0.0.1/tcp/9001"
    ));
}

#[test]
fn test_cli_parse_send() {
    let cli = Cli::parse_from(["scm", "send", "recipient-id", "hello world"]);
    assert!(matches!(
        cli.command,
        Commands::Send { ref recipient, ref message }
        if recipient == "recipient-id" && message == "hello world"
    ));
}

#[test]
fn test_cli_parse_status() {
    let cli = Cli::parse_from(["scm", "status"]);
    assert!(matches!(cli.command, Commands::Status));
}

#[test]
fn test_cli_parse_config_list() {
    let cli = Cli::parse_from(["scm", "config", "list"]);
    assert!(matches!(cli.command, Commands::Config { .. }));
}

#[test]
fn test_ble_daemon_default_unavailable() {
    let daemon = BleDaemon::default();
    assert!(!daemon.is_available());
}

#[test]
fn test_ble_config_default_values() {
    let config = BleConfig::default();
    assert_eq!(config.scan_interval_ms, 1000);
    assert_eq!(config.max_retry_attempts, 3);
    assert_eq!(config.advertisement_timeout_ms, 5000);
}

#[test]
fn test_ble_error_display_messages() {
    assert_eq!(format!("{}", BleError::NoAdapter), "No Bluetooth adapter found");
    assert_eq!(format!("{}", BleError::PermissionDenied), "Bluetooth permission denied");
    assert_eq!(format!("{}", BleError::Timeout), "BLE operation timed out");
}

#[test]
fn test_ble_status_variants() {
    let status = BleStatus::Unavailable(BleError::NoAdapter);
    assert!(!matches!(status, BleStatus::Available(_)));

    let status = BleStatus::Disabled;
    assert!(!matches!(status, BleStatus::Available(_)));
}

#[test]
fn test_config_default_values() {
    let config = scmessenger_cli::config::Config::default();
    assert_eq!(config.listen_port, 9000);
    assert!(config.enable_mdns);
    assert!(config.enable_dht);
}

#[test]
fn test_cli_parse_history_stats() {
    let cli = Cli::parse_from(["scm", "history-stats"]);
    assert!(matches!(cli.command, Commands::HistoryStats));
}

#[test]
fn test_cli_parse_history_count() {
    let cli = Cli::parse_from(["scm", "history-count"]);
    assert!(matches!(cli.command, Commands::HistoryCount));
}

#[test]
fn test_identity_backup_restore_roundtrip_logic() {
    use scmessenger_cli::cli::IdentityAction;

    // Simulate export
    let export_cli = Cli::parse_from(["scm", "identity", "export", "--passphrase", "mypass", "--output", "backup.json"]);
    if let Commands::Identity { action: Some(IdentityAction::Export { passphrase, output }) } = export_cli.command {
        assert_eq!(passphrase, "mypass");
        assert_eq!(output, Some("backup.json".to_string()));
    } else {
        panic!("Failed to parse export command");
    }

    // Simulate import
    let import_cli = Cli::parse_from(["scm", "identity", "import", "--passphrase", "mypass", "--input", "backup.json"]);
    if let Commands::Identity { action: Some(IdentityAction::Import { passphrase, input, .. }) } = import_cli.command {
        assert_eq!(passphrase, "mypass");
        assert_eq!(input, Some("backup.json".to_string()));
    } else {
        panic!("Failed to parse import command");
    }
}

#[test]
fn test_block_unblock_cascade_logic() {
    use scmessenger_cli::cli::BlockAction;

    // Block
    let block_cli = Cli::parse_from(["scm", "block", "add", "peer-123", "--reason", "spam"]);
    if let Commands::Block { action: BlockAction::Add { peer_id, reason, .. } } = block_cli.command {
        assert_eq!(peer_id, "peer-123");
        assert_eq!(reason, Some("spam".to_string()));
    } else {
        panic!("Failed to parse block command");
    }

    // Unblock
    let unblock_cli = Cli::parse_from(["scm", "block", "remove", "peer-123"]);
    if let Commands::Block { action: BlockAction::Remove { peer_id, .. } } = unblock_cli.command {
        assert_eq!(peer_id, "peer-123");
    } else {
        panic!("Failed to parse unblock command");
    }

    // Cascade delete
    let delete_cli = Cli::parse_from(["scm", "block", "delete", "peer-123"]);
    if let Commands::Block { action: BlockAction::Delete { peer_id, .. } } = delete_cli.command {
        assert_eq!(peer_id, "peer-123");
    } else {
        panic!("Failed to parse block delete command");
    }
}

#[test]
fn test_cli_parse_mark_sent() {
    let cli = Cli::parse_from(["scm", "mark-sent", "msg-123"]);
    assert!(matches!(cli.command, Commands::MarkSent { ref message_id } if message_id == "msg-123"));
}
