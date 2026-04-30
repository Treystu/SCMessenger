// scmessenger-cli — Complete Desktop CLI
//
// Cross-platform (macOS, Linux, Windows) command-line interface for SCMessenger.

#![allow(dead_code, unused)]

mod api;
mod ble_daemon;
mod ble_mesh;
mod config;
mod ledger;
mod server;
mod transport_api;
mod transport_bridge;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use libp2p::{Multiaddr, PeerId};
use scmessenger_core::message::{decode_envelope, MessageType};
use scmessenger_core::store::{Contact, ContactManager, MessageDirection, Outbox, QueuedMessage};
use scmessenger_core::transport::abstraction::TransportType;
use scmessenger_core::transport::{self, SwarmEvent};
use scmessenger_core::wasm_support::rpc::{
    notif_delivery_status, notif_message_received, notif_peer_discovered, rpc_error, rpc_result,
    ClientIntent, DeliveryStatusParams, MeshTopologyUpdateParams, MessageReceivedParams,
    PeerDiscoveredParams,
};
use scmessenger_core::IronCore;
use std::collections::HashMap;
use std::sync::Arc;

fn load_or_create_headless_network_keypair(
    storage_path: &std::path::Path,
    core: &IronCore,
) -> Result<libp2p::identity::Keypair> {
    std::fs::create_dir_all(storage_path).context("Failed to create relay storage directory")?;
    let key_path = storage_path.join("relay_network_key.pb");

    if key_path.exists() {
        let bytes = std::fs::read(&key_path).context("Failed to read relay network key file")?;
        match libp2p::identity::Keypair::from_protobuf_encoding(&bytes) {
            Ok(keypair) => return Ok(keypair),
            Err(err) => {
                tracing::warn!(
                    "Relay network key decode failed ({}); rotating key file: {}",
                    err,
                    key_path.display()
                );
            }
        }
    }

    // Key file absent or corrupt — try migrating from IronCore identity to
    // preserve the relay PeerId across upgrades.
    if let Ok(keypair) = core.get_libp2p_keypair() {
        tracing::info!("Migrating relay network key from existing IronCore identity");
        if let Ok(encoded) = keypair.to_protobuf_encoding() {
            if let Err(e) = std::fs::write(&key_path, &encoded) {
                tracing::warn!("Failed to persist migrated relay key: {}", e);
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ =
                        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
                }
            }
        }
        return Ok(keypair);
    }

    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let encoded = keypair
        .to_protobuf_encoding()
        .context("Failed to encode relay network key")?;
    std::fs::write(&key_path, &encoded).context("Failed to persist relay network key")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(keypair)
}

#[derive(Parser)]
#[command(name = "scm")]
#[command(about = "SCMessenger — Sovereign Encrypted Messaging", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize new identity
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show identity information
    Identity {
        #[command(subcommand)]
        action: Option<IdentityAction>,
    },
    /// Manage contacts
    Contact {
        #[command(subcommand)]
        action: ContactAction,
    },
    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// View message history
    History {
        #[arg(short, long)]
        peer: Option<String>,
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Start P2P messaging node
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Run headless relay node (no interactive console)
    Relay {
        /// P2P listen multiaddr (default: /ip4/0.0.0.0/tcp/9001)
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/9001")]
        listen: String,
        /// HTTP status/landing page port (default: 9000)
        #[arg(long, default_value = "9000")]
        http_port: u16,
        /// Node name for logging/status (default: auto from peer ID)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Send a message (offline mode)
    Send { recipient: String, message: String },
    /// Show network status
    Status,
    /// Mark an outbox message as delivered/sent
    MarkSent { message_id: String },
    /// Clear all local history records
    HistoryClear {
        /// Required confirmation flag
        #[arg(long)]
        yes: bool,
    },
    /// Keep only the newest N history messages
    HistoryEnforceRetention { max_messages: u32 },
    /// Remove history older than a unix timestamp (seconds)
    HistoryPruneBefore { before_timestamp: u64 },
    /// Stop the running node
    Stop,
    /// Manage peer blocking
    Block {
        #[command(subcommand)]
        action: BlockAction,
    },
    /// Get a single history message by ID
    HistoryGet { id: String },
    /// Show history statistics
    HistoryStats,
    /// Show total history message count
    HistoryCount,
    /// Mark a history message as delivered
    HistoryMarkDelivered { id: String },
    /// Clear conversation history with a specific peer
    HistoryClearConversation { peer: String },
    /// Remove an entire conversation (alias for clear-conversation)
    HistoryRemoveConversation { peer: String },
    /// Delete a single history message by ID
    HistoryDelete { id: String },
    /// Run self-tests
    Test,
    /// Manage audit log
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    /// Swarm management commands
    Swarm {
        #[command(subcommand)]
        action: SwarmAction,
    },
}

#[derive(Subcommand)]
enum AuditAction {
    /// Export the entire audit log as JSON
    Export {
        /// Optional output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Verify the cryptographic integrity of the audit log
    Verify,
    /// Show a summary of audit log statistics
    Stats,
}

#[derive(Subcommand)]
enum BlockAction {
    /// Block a peer
    Add {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Unblock a peer
    Remove {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Block a peer AND delete all their stored messages (cascade purge)
    Delete {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// List all blocked peers
    List,
    /// Check if a peer is blocked
    Check {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Show total blocked count
    Count,
}

#[derive(Subcommand)]
enum IdentityAction {
    Show,
    Export {
        /// Passphrase to encrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Optional output file path for backup payload
        #[arg(short, long)]
        output: Option<String>,
    },
    Import {
        /// Passphrase to decrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Backup payload string
        #[arg(long, conflicts_with = "input")]
        backup: Option<String>,
        /// Read backup payload from file
        #[arg(short = 'i', long)]
        input: Option<String>,
    },
    SetName {
        name: String,
    },
    /// Show the local device ID (WS13)
    DeviceId,
    /// Show the seniority timestamp (WS13)
    Seniority,
    /// Show registration state for an identity
    RegistrationState {
        identity_id: String,
    },
    /// Sign arbitrary data with the local identity key
    SignData {
        /// Hex-encoded data to sign
        data_hex: String,
    },
    /// Verify a signature against data and a public key
    VerifySignature {
        /// Hex-encoded data
        data_hex: String,
        /// Hex-encoded signature
        signature_hex: String,
        /// Hex-encoded Ed25519 public key
        public_key_hex: String,
    },
}

#[derive(Subcommand)]
enum ContactAction {
    Add {
        peer_id: String,
        public_key: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    List,
    Show {
        contact: String,
    },
    Remove {
        contact: String,
    },
    Search {
        query: String,
    },
    SetLocalNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Set the federated (broadcast) nickname for a contact
    SetNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    List,
    /// Manage privacy-enhancing features (onion routing, padding, etc.)
    Privacy {
        /// Enable/disable message padding
        #[arg(short, long)]
        padding: Option<bool>,
        /// Enable/disable onion routing
        #[arg(short, long)]
        onion: Option<bool>,
        /// Enable/disable cover traffic
        #[arg(short, long)]
        cover: Option<bool>,
        /// Enable/disable timing obfuscation (jitter)
        #[arg(short, long)]
        timing: Option<bool>,
    },
}

#[derive(Subcommand)]
enum SwarmAction {
    /// Show swarm connection statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Determine data directory early for logging
    let data_dir = config::Config::data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    // 2. Setup file logging (rolling hourly, keeping last 24h)
    let file_appender = tracing_appender::rolling::hourly(&log_dir, "scm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 3. Initialize tracing with both stdout (fmt) and file (appender)
    use tracing_subscriber::prelude::*;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .init();

    tracing::info!("SCMessenger CLI starting up...");
    tracing::info!("Log directory: {}", log_dir.display());

    // 4. Prune old logs (keep last 7 days)
    if let Err(e) = prune_logs(&log_dir, 7) {
        tracing::warn!("Failed to prune old logs: {}", e);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => cmd_init(name).await,
        Commands::Identity { action } => cmd_identity(action).await,
        Commands::Contact { action } => cmd_contact(action).await,
        Commands::Config { action } => cmd_config(action).await,
        Commands::History {
            peer,
            search,
            limit,
        } => cmd_history(peer, search, limit).await,
        Commands::Start { port } => cmd_start(port).await,
        Commands::Relay {
            listen,
            http_port,
            name,
        } => cmd_relay(listen, http_port, name).await,
        Commands::Stop => cmd_stop().await,
        Commands::Send { recipient, message } => cmd_send_offline(recipient, message).await,
        Commands::Status => cmd_status().await,
        Commands::MarkSent { message_id } => cmd_mark_sent(message_id).await,
        Commands::HistoryClear { yes } => cmd_history_clear(yes).await,
        Commands::HistoryEnforceRetention { max_messages } => {
            cmd_history_enforce_retention(max_messages).await
        }
        Commands::HistoryPruneBefore { before_timestamp } => {
            cmd_history_prune_before(before_timestamp).await
        }
        Commands::Block { action } => cmd_block(action).await,
        Commands::HistoryGet { id } => cmd_history_get(id).await,
        Commands::HistoryStats => cmd_history_stats().await,
        Commands::HistoryCount => cmd_history_count().await,
        Commands::HistoryMarkDelivered { id } => cmd_history_mark_delivered(id).await,
        Commands::HistoryClearConversation { peer }
        | Commands::HistoryRemoveConversation { peer } => {
            cmd_history_clear_conversation(peer).await
        }
        Commands::HistoryDelete { id } => cmd_history_delete(id).await,
        Commands::Test => cmd_test().await,
        Commands::Audit { action } => cmd_audit(action).await,
        Commands::Swarm { action } => cmd_swarm(action).await,
    }
}

async fn cmd_stop() -> Result<()> {
    if !api::is_api_available().await {
        println!("{}", "No SCMessenger node is running.".yellow());
        return Ok(());
    }

    print!("Stopping SCMessenger node... ");
    match api::stop_node_via_api().await {
        Ok(_) => println!("{}", "Done.".green()),
        Err(e) => println!("{} {}", "Error:".red(), e),
    }
    Ok(())
}

async fn cmd_init(name: Option<String>) -> Result<()> {
    println!("{}", "Initializing SCMessenger...".bold());
    println!();

    let config = config::Config::load()?;
    println!("  {} Configuration", "✓".green());

    let data_dir = config::Config::data_dir()?;
    println!("  {} Data directory: {}", "✓".green(), data_dir.display());

    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to initialize identity")?;

    // Set nickname if provided
    if let Some(nickname) = name {
        core.set_nickname(nickname)
            .context("Failed to set nickname")?;
        println!("  {} Nickname set", "✓".green());
    }

    println!("  {} Identity created", "✓".green());
    println!();

    print_full_identity(&core, &config)?;

    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  • Add contacts: {}",
        "scm contact add <peer-id> <public-key> --name <nickname>".bright_green()
    );
    println!("  • Start node:   {}", "scm start".bright_green());

    Ok(())
}

async fn cmd_identity(action: Option<IdentityAction>) -> Result<()> {
    let config = config::Config::load()?;
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to load identity")?;

    match action {
        None | Some(IdentityAction::Show) => {
            print_full_identity(&core, &config)?;
        }
        Some(IdentityAction::SetName { name }) => {
            core.set_nickname(name.clone())
                .context("Failed to set nickname")?;
            println!(
                "{} Nickname updated to: {}",
                "✓".green(),
                name.bright_cyan()
            );
        }
        Some(IdentityAction::Export { passphrase, output }) => {
            let backup = core
                .export_identity_backup(passphrase)
                .context("Failed to export identity backup")?;
            let info = core.get_identity_info();

            println!("{}", "Export Identity Backup".bold());
            println!();
            println!(
                "{}",
                "⚠️  WARNING: backup payload contains private key material."
                    .bright_red()
                    .bold()
            );
            println!("Identity ID: {}", info.identity_id.unwrap_or_default());
            println!("Public Key:  {}", info.public_key_hex.unwrap_or_default());
            println!("Payload size: {} bytes", backup.len());
            println!();

            if let Some(path) = output {
                std::fs::write(&path, &backup)
                    .with_context(|| format!("Failed to write backup file: {}", path))?;
                println!("{} Backup written to {}", "✓".green(), path.bright_cyan());
            } else {
                println!("{}", "Backup payload:".bold());
                println!("{}", backup);
            }
        }
        Some(IdentityAction::Import {
            passphrase,
            backup,
            input,
        }) => {
            let payload = if let Some(path) = input {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read backup file: {}", path))?
            } else if let Some(raw) = backup {
                raw
            } else {
                anyhow::bail!("Provide --backup <payload> or --input <file>");
            };

            core.import_identity_backup(payload, passphrase)
                .context("Failed to import identity backup")?;
            let info = core.get_identity_info();
            println!("{}", "✓ Identity backup imported".green());
            println!(
                "  Identity ID: {}",
                info.identity_id.unwrap_or_default().bright_cyan()
            );
            println!(
                "  Public Key:  {}",
                info.public_key_hex.unwrap_or_default().bright_yellow()
            );
        }
        Some(IdentityAction::DeviceId) => match core.get_device_id() {
            Some(id) => println!("Device ID: {}", id.bright_cyan()),
            None => println!(
                "{}",
                "No device ID available (identity not initialized?)".dimmed()
            ),
        },
        Some(IdentityAction::Seniority) => match core.get_seniority_timestamp() {
            Some(ts) => println!("Seniority Timestamp: {} ({})", ts, format_timestamp(ts)),
            None => println!("{}", "No seniority timestamp available".dimmed()),
        },
        Some(IdentityAction::RegistrationState { identity_id }) => {
            let state = core.get_registration_state(identity_id.clone());
            println!("{}", "Registration State".bold());
            println!("  Identity:   {}", identity_id.bright_cyan());
            println!("  State:      {}", state.state);
            if let Some(device_id) = state.device_id {
                println!("  Device ID:  {}", device_id);
            }
            if let Some(ts) = state.seniority_timestamp {
                println!("  Seniority:  {} ({})", ts, format_timestamp(ts));
            }
        }
        Some(IdentityAction::SignData { data_hex }) => {
            let data = hex::decode(&data_hex).context("Invalid hex data")?;
            let result = core.sign_data(data).context("Failed to sign data")?;
            println!("{}", "Signature Result".bold());
            println!(
                "  Signature:  {}",
                hex::encode(&result.signature).bright_yellow()
            );
            println!("  Public Key: {}", result.public_key_hex.bright_cyan());
        }
        Some(IdentityAction::VerifySignature {
            data_hex,
            signature_hex,
            public_key_hex,
        }) => {
            let data = hex::decode(&data_hex).context("Invalid hex data")?;
            let signature = hex::decode(&signature_hex).context("Invalid hex signature")?;
            let valid = core
                .verify_signature(data, signature, public_key_hex)
                .context("Failed to verify signature")?;
            if valid {
                println!("{} Signature is valid", "✓".green());
            } else {
                println!("{} Signature is INVALID", "✗".red());
            }
        }
    }

    Ok(())
}

fn print_full_identity(core: &IronCore, config: &config::Config) -> Result<()> {
    let info = core.get_identity_info();
    // Derive PeerId from identity
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair")?;
    let local_peer_id = network_keypair.public().to_peer_id();

    println!("{}", "Identity Information".bold());
    println!(
        "  ID:                     {}",
        info.identity_id.unwrap().bright_cyan()
    );
    println!(
        "  Peer ID (Network):      {}",
        local_peer_id.to_string().bright_cyan()
    );
    println!(
        "  Nickname:               {}",
        info.nickname
            .as_deref()
            .unwrap_or("(not set)")
            .bright_cyan()
    );
    println!(
        "  Public Key:             {}",
        info.public_key_hex.unwrap().bright_yellow()
    );
    println!();

    println!("{}", "Direct Connection Info".bold());
    let ws_port = if config.listen_port == 0 {
        9000
    } else {
        config.listen_port
    };
    let p2p_port = ws_port + 1;

    // Show P2P listening address
    println!(
        "  P2P Listener:           {}",
        format!("/ip4/0.0.0.0/tcp/{}", p2p_port).green()
    );

    // Show examples for LAN/Localhost
    println!("  Local Address:          /ip4/127.0.0.1/tcp/{}", p2p_port);

    // Attempt to show LAN IP if possible (simple heuristic or just mention it)
    println!(
        "  LAN Address:            /ip4/<YOUR_LAN_IP>/tcp/{}",
        p2p_port
    );

    println!();

    Ok(())
}

async fn cmd_contact(action: ContactAction) -> Result<()> {
    match action {
        ContactAction::Add {
            peer_id,
            public_key,
            name,
        } => {
            // Validate public key format before adding
            scmessenger_core::crypto::validate_ed25519_public_key(&public_key)
                .context("Invalid public key")?;

            // Guard: reject public key / identity_id where a libp2p Peer ID is required
            if looks_like_blake3_id(&peer_id) {
                eprintln!(
                    "{} That looks like a public key or identity ID (64 hex chars), not a libp2p Peer ID.",
                    "⚠ Error:".red()
                );
                eprintln!("  Use the 'Peer ID (Network)' shown by: scm identity");
                eprintln!("  It starts with '12D3Koo...' and is ~52 characters.");
                return Ok(());
            }
            if !looks_like_libp2p_peer_id(&peer_id) {
                eprintln!(
                    "{} '{}' is not a valid libp2p Peer ID.",
                    "⚠ Error:".red(),
                    peer_id
                );
                eprintln!("  Use the 'Peer ID (Network)' shown by: scm identity");
                eprintln!("  It starts with '12D3Koo...' and is ~52 characters.");
                return Ok(());
            }

            // Try to use API if a node is running
            if api::is_api_available().await {
                api::add_contact_via_api(&peer_id, &public_key, name.clone())
                    .await
                    .context("Failed to add contact via API")?;
                println!("{} Contact added:", "✓".green());
                if let Some(nickname) = &name {
                    println!("  Name: {}", nickname.bright_cyan());
                }
                println!("  Peer ID: {}", peer_id);
                return Ok(());
            }

            // Fallback to direct database access
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
            let contacts = core.contacts_store_manager();

            // UNIFIED ID FIX: derive canonical public_key_hex from libp2p Peer ID
            // and verify it matches the user-supplied public key.
            let canonical_pk = core
                .extract_public_key_from_peer_id(peer_id.clone())
                .context(
                    "Failed to derive public key from Peer ID — is it a valid libp2p Peer ID?",
                )?;
            if canonical_pk.to_lowercase() != public_key.to_lowercase() {
                eprintln!(
                    "{} The provided public key does not match the Peer ID.",
                    "⚠ Error:".red()
                );
                eprintln!(
                    "  Peer ID {} resolves to public key: {}",
                    peer_id.dimmed(),
                    canonical_pk.yellow()
                );
                eprintln!("  You provided public key: {}", public_key.dimmed());
                return Ok(());
            }

            let mut contact = Contact::new(canonical_pk.clone(), public_key);
            if let Some(nickname) = name.clone() {
                contact.nickname = Some(nickname);
            }

            contacts
                .add(contact)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            println!("{} Contact added:", "✓".green());
            if let Some(nickname) = name {
                println!("  Name: {}", nickname.bright_cyan());
            }
            println!("  Canonical ID: {}", canonical_pk.yellow());
            println!("  Peer ID (Network): {}", peer_id.dimmed());
        }

        _ => {
            // For other contact operations, use direct database access
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
            let contacts = core.contacts_store_manager();

            match action {
                ContactAction::List => {
                    let list = contacts.list().unwrap_or_default();

                    if list.is_empty() {
                        println!("{}", "No contacts yet.".dimmed());
                    } else {
                        println!("{} ({} total)", "Contacts".bold(), list.len());
                        println!();

                        for contact in list {
                            let display = contact
                                .nickname
                                .clone()
                                .unwrap_or_else(|| contact.peer_id.clone());
                            println!("  {} {}", "•".bright_green(), display.bright_cyan());
                            println!("    Peer ID: {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::Show { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;

                    let display = contact
                        .nickname
                        .clone()
                        .unwrap_or_else(|| contact.peer_id.clone());
                    println!("{}", "Contact Details".bold());
                    println!("  Name:       {}", display.bright_cyan());
                    println!("  Peer ID:    {}", contact.peer_id);
                    println!("  Public Key: {}", contact.public_key.bright_yellow());
                    println!("  Added:      {}", format_timestamp(contact.added_at));
                }

                ContactAction::Remove { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;
                    let name = contact
                        .nickname
                        .clone()
                        .unwrap_or_else(|| contact.peer_id.clone());

                    contacts
                        .remove(contact.peer_id)
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                    println!("{} Removed contact: {}", "✓".green(), name.bright_cyan());
                }

                ContactAction::Search { query } => {
                    let results = contacts.search(query).unwrap_or_default();

                    if results.is_empty() {
                        println!("{}", "No matching contacts.".dimmed());
                    } else {
                        println!("{} ({} matches)", "Search Results".bold(), results.len());
                        println!();

                        for contact in results {
                            let display = contact
                                .nickname
                                .clone()
                                .unwrap_or_else(|| contact.peer_id.clone());
                            println!("  {} {}", "•".bright_green(), display.bright_cyan());
                            println!("    {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::SetLocalNickname {
                    contact: query,
                    nickname,
                    clear,
                } => {
                    if clear && nickname.is_some() {
                        anyhow::bail!("Use either <nickname> or --clear, not both");
                    }

                    let contact = find_contact(&contacts, &query)?;
                    let local = if clear { None } else { nickname };
                    contacts
                        .set_local_nickname(contact.peer_id.clone(), local.clone())
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    match local {
                        Some(name) => {
                            println!(
                                "{} Local nickname set for {} -> {}",
                                "✓".green(),
                                contact.peer_id.dimmed(),
                                name.bright_cyan()
                            );
                        }
                        None => {
                            println!(
                                "{} Local nickname cleared for {}",
                                "✓".green(),
                                contact.peer_id.dimmed()
                            );
                        }
                    }
                }

                ContactAction::Add { .. } => unreachable!(),
                ContactAction::SetNickname {
                    contact: query,
                    nickname,
                    clear,
                } => {
                    if clear && nickname.is_some() {
                        anyhow::bail!("Use either <nickname> or --clear, not both");
                    }

                    let contact = find_contact(&contacts, &query)?;
                    let nick = if clear { None } else { nickname };
                    contacts
                        .set_nickname(contact.peer_id.clone(), nick.clone())
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    match nick {
                        Some(name) => {
                            println!(
                                "{} Federated nickname set for {} -> {}",
                                "✓".green(),
                                contact.peer_id.dimmed(),
                                name.bright_cyan()
                            );
                        }
                        None => {
                            println!(
                                "{} Federated nickname cleared for {}",
                                "✓".green(),
                                contact.peer_id.dimmed()
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cmd_config(action: ConfigAction) -> Result<()> {
    let mut config = config::Config::load()?;

    match action {
        ConfigAction::Set { key, value } => {
            config.set(&key, &value)?;
            println!("{} Set {} = {}", "✓".green(), key.bright_cyan(), value);
        }

        ConfigAction::Get { key } => {
            if let Some(value) = config.get(&key) {
                println!("{} = {}", key.bright_cyan(), value);
            } else {
                anyhow::bail!("Unknown config key: {}", key);
            }
        }

        ConfigAction::List => {
            println!("{}", "Configuration".bold());
            println!();

            for (key, value) in config.list() {
                println!("  {:<20} {}", key.bright_cyan(), value);
            }
        }

        ConfigAction::Privacy {
            padding,
            onion,
            cover,
            timing,
        } => {
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());

            let mut p: scmessenger_core::privacy::PrivacyConfig =
                serde_json::from_str(&core.get_privacy_config())?;

            if padding.is_none() && onion.is_none() && cover.is_none() && timing.is_none() {
                // Just show current config if no flags provided
                println!("{}", "Privacy Configuration".bold());
                println!(
                    "  Message Padding:   {}",
                    if p.message_padding_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Onion Routing:      {}",
                    if p.onion_routing_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Cover Traffic:      {}",
                    if p.cover_traffic_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Timing Obfuscation: {}",
                    if p.timing_obfuscation_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                return Ok(());
            }

            if let Some(v) = padding {
                p.message_padding_enabled = v;
            }
            if let Some(v) = onion {
                p.onion_routing_enabled = v;
            }
            if let Some(v) = cover {
                p.cover_traffic_enabled = v;
            }
            if let Some(v) = timing {
                p.timing_obfuscation_enabled = v;
            }

            core.set_privacy_config(serde_json::to_string(&p)?)?;
            println!("{} Privacy configuration updated.", "✓".green());
        }
    }

    Ok(())
}

async fn cmd_history(
    peer_filter: Option<String>,
    search_query: Option<String>,
    limit: usize,
) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();

    let messages = if let Some(query) = search_query {
        history
            .search(query, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    } else if let Some(peer) = peer_filter {
        let contacts = core.contacts_store_manager();
        let peer_id = if let Ok(contact) = find_contact(&contacts, &peer) {
            contact.peer_id
        } else {
            peer
        };

        history
            .conversation(peer_id, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    } else {
        history
            .recent(None, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    };

    if messages.is_empty() {
        println!("{}", "No messages found.".dimmed());
        return Ok(());
    }

    println!("{} ({} messages)", "Message History".bold(), messages.len());
    println!();

    for msg in messages {
        let direction = match msg.direction {
            MessageDirection::Sent => "→".bright_green(),
            MessageDirection::Received => "←".bright_blue(),
        };

        let time = format_timestamp(msg.timestamp).dimmed();
        let peer = msg.peer_id;

        println!("{} {} [{}]", direction, peer.bright_cyan(), time);
        println!("   {}", msg.content);
        println!();
    }

    Ok(())
}

async fn cmd_start(port: Option<u16>) -> Result<()> {
    let config = config::Config::load()?;
    let ws_port = port.unwrap_or({
        if config.listen_port == 0 {
            9000 // Default to 9000 if config has random port
        } else {
            config.listen_port
        }
    });

    // 1. Check if SCMessenger is already running via Control API
    if api::is_api_available().await {
        println!("{}", "SCMessenger is already running!".yellow());
        println!(
            "Run {} to stop the existing node first.",
            "scm stop".bright_green()
        );
        return Ok(());
    }

    // 2. Check if ports are occupied by something else (v4, v6, and localhost)
    let p2p_port = ws_port + 1;
    let check_ports = [ws_port, p2p_port];
    for p in check_ports {
        let addrs = [
            std::net::SocketAddr::from(([127, 0, 0, 1], p)),
            std::net::SocketAddr::from(([0, 0, 0, 0], p)),
        ];
        for addr in addrs {
            if std::net::TcpListener::bind(addr).is_err() {
                println!("{} Port {} is already in use.", "Error:".red(), p);
                println!(
                    "Try running {} or checking for other processes on this port.",
                    "scm stop".bright_green()
                );
                return Ok(());
            }
        }
    }

    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to load identity")?;

    let info = core.get_identity_info();

    let contacts = core.contacts_store_manager();
    let history = core.history_store_manager();
    let _ = history.enforce_retention(10000); // Limit history to 10k messages

    // ── Outbox — persistent store-and-forward queue ──────────────────────
    // Messages sent to offline peers are queued here and flushed automatically
    // when those peers come online (see PeerDiscovered handler below).
    let outbox_path = data_dir.join("outbox");
    let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
    let outbox = match scmessenger_core::store::backend::SledStorage::new(&outbox_path_str) {
        Ok(backend) => Arc::new(tokio::sync::Mutex::new(Outbox::persistent(Arc::new(
            backend,
        )))),
        Err(e) => {
            tracing::warn!(
                "Failed to open persistent outbox, falling back to in-memory: {}",
                e
            );
            Arc::new(tokio::sync::Mutex::new(Outbox::new()))
        }
    };

    // ── Connection Ledger — persistent peer memory ──────────────────────
    let connection_ledger = ledger::ConnectionLedger::load(&data_dir)?;

    // Subscribe to any topics discovered in the ledger from past sessions
    let known_topics = connection_ledger.all_known_topics();

    println!("{}", "SCMessenger — Starting...".bold());
    println!();
    println!(
        "Identity: {}",
        info.identity_id.clone().unwrap().bright_cyan()
    );
    println!(
        "Public Key: {}",
        info.public_key_hex
            .as_deref()
            .unwrap_or("(not initialized)")
    );
    println!("Landing Page:  http://127.0.0.1:{}", ws_port);
    println!("WebSocket:     ws://127.0.0.1:{}/ws", ws_port);
    println!("P2P Listener:  /ip4/0.0.0.0/tcp/{}", p2p_port);
    println!("WASM Bridge:   /ip4/0.0.0.0/tcp/{}/ws", p2p_port + 1);
    println!("📒 {}", connection_ledger.summary());
    println!();

    // Wrap core in Arc early so WebContext and later tasks can share it.
    let core = Arc::new(core);

    // Use identity keypair for network (unified ID)
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair from identity")?;
    let local_peer_id = network_keypair.public().to_peer_id();

    // NOTE: PeerId is now derived from identity keys. Existing installations that
    // had a separate network_keypair.dat will see their PeerId change. This is
    // intentional to unify identity and network IDs, but may require updating
    // peer expectations/ledgers on migration.

    println!("{} Peer ID: {}", "✓".green(), local_peer_id);
    println!();

    // Create shared state BEFORE server start so landing page has access
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let ledger = Arc::new(tokio::sync::Mutex::new(connection_ledger));

    // Create transport bridge
    let transport_bridge = Arc::new(tokio::sync::Mutex::new(
        transport_bridge::TransportBridge::new(),
    ));

    // Build web context for landing page + public APIs
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: info.public_key_hex.clone().unwrap_or_default(),
        bootstrap_nodes: config.bootstrap_nodes.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
        transport_bridge: transport_bridge.clone(),
        ui_port: ws_port,
        core: Some(Arc::clone(&core)),
    });

    // Start WebSocket + HTTP Server (serves landing page at /)
    let (ui_broadcast, mut ui_cmd_rx) = server::start(ws_port, web_ctx.clone()).await?;

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse()?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle =
        transport::start_swarm(network_keypair, Some(listen_addr), event_tx, None, false).await?;

    // ── WebSocket P2P Bridge for WASM ────────────────────────────────────
    let ws_p2p_port = p2p_port + 1;
    let ws_listen_addr: libp2p::Multiaddr =
        format!("/ip4/0.0.0.0/tcp/{}/ws", ws_p2p_port).parse()?;
    match swarm_handle.listen(ws_listen_addr.clone()).await {
        Ok(_) => println!(
            "{} WebSocket P2P Bridge started on {}",
            "✓".green(),
            ws_listen_addr
        ),
        Err(e) => tracing::warn!("Failed to start WebSocket P2P bridge: {}", e),
    }

    println!("{} Network started", "✓".green());

    tokio::spawn(async move {
        ble_daemon::probe_and_log().await;
    });

    // Subscribe to any topics from the ledger
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }

    // Subscribe to default topics
    for topic in ["sc-lobby", "sc-mesh"] {
        let _ = swarm_handle.subscribe_topic(topic.to_string()).await;
    }

    println!();
    println!("{}", "Commands:".bold());
    println!("  {} <contact> <message>", "send".bright_green());
    println!("  {}                      ", "contacts".bright_green());
    println!("  {}                       ", "peers".bright_green());
    println!("  {}                      ", "status".bright_green());
    println!("  {}                        ", "quit".bright_green());
    println!();

    // Note: core was wrapped in Arc above before WebContext creation;
    // peers and ledger Arc<Mutex> were created above before server::start
    // so the landing page and API endpoints have access to them.

    let core_ble = Arc::clone(&core);
    let ui_ble = ui_broadcast.clone();
    tokio::spawn(async move {
        ble_mesh::run_ble_central_ingress(core_ble, ui_ble).await;
    });

    let core_ble_adv = Arc::clone(&core);
    tokio::spawn(async move {
        ble_mesh::run_ble_peripheral_advertising(core_ble_adv).await;
    });

    // ── Dial known peers from persistent ledger ──────────────────────────
    // Dial any peers from the persistent ledger that pass backoff.
    {
        println!();
        println!(
            "{} Aggressive Discovery — dialing known peers...",
            "⚙".yellow()
        );
        let swarm_clone = swarm_handle.clone();
        let ledger_clone = ledger.clone();

        tokio::spawn(async move {
            let addrs = {
                let l = ledger_clone.lock().await;
                l.dialable_addresses(Some(&local_peer_id.to_string()))
            };

            // Dial all known addresses (bootstrap + discovered)
            for (i, (multiaddr_str, _peer_id_opt)) in addrs.iter().enumerate() {
                let stripped = ledger::strip_peer_id(multiaddr_str);
                match stripped.parse::<Multiaddr>() {
                    Ok(addr) => {
                        let label = ledger::extract_ip_port(multiaddr_str)
                            .unwrap_or_else(|| multiaddr_str.clone());
                        println!("  {}. 📞 Dialing {} (promiscuous)...", i + 1, label);

                        // Single attempt — the swarm will handle retries
                        match swarm_clone.dial(addr).await {
                            Ok(_) => {
                                println!("  {} Dial initiated to {}", "✓".green(), label);
                            }
                            Err(e) => {
                                tracing::warn!("Dial failed to {}: {}", label, e);
                                let mut l = ledger_clone.lock().await;
                                l.record_failure(multiaddr_str);
                            }
                        }

                        // Brief pause between dials to avoid overwhelming
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                    Err(e) => {
                        tracing::error!("Invalid multiaddr: {} - {}", stripped, e);
                    }
                }
            }
        });
    }

    // Broadcast status loop for WebSocket UI
    let ui_broadcast_clone = ui_broadcast.clone();
    let peers_clone_status = peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let count = peers_clone_status.lock().await.len();
            // Don't crash if no subscribers
            let _ = ui_broadcast_clone.send(server::UiOutbound::Legacy(
                server::UiEvent::NetworkStatus {
                    status: "online".to_string(),
                    peer_count: count,
                },
            ));
        }
    });

    // Periodic ledger save (every 60 seconds)
    let ledger_save_clone = ledger.clone();
    let data_dir_save = data_dir.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let mut l = ledger_save_clone.lock().await;
            if let Err(e) = l.save(&data_dir_save) {
                tracing::error!("Failed to save ledger: {}", e);
            }
        }
    });

    // P0_TRANSPORT_001: Periodic address refresh - before dialing from ledger,
    // send an Identify probe to refresh peer addresses. This ensures we have
    // current listen addresses even if peers restarted with new ports.
    let swarm_refresh_clone = swarm_handle.clone();
    let ledger_refresh_clone = ledger.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

            let addrs = {
                let l = ledger_refresh_clone.lock().await;
                l.dialable_addresses(Some(&local_peer_id.to_string()))
            };

            // For each peer, try to refresh their address via Identify
            // This asks the peer: "What is your current listening address?"
            for (_multiaddr_str, peer_id_opt) in &addrs {
                if let Some(ref peer_id_str) = peer_id_opt {
                    if let Ok(peer_id) = peer_id_str.parse::<PeerId>() {
                        // Attempt to request address reflection
                        // If this succeeds, we'll get the current address
                        // If it fails (no connection), we'll still dial with current address
                        let _ = swarm_refresh_clone
                            .request_address_reflection(peer_id)
                            .await;
                    }
                }
            }
        }
    });

    // Start control API server
    let api_ctx = api::ApiContext {
        core: core.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
    };

    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_ctx).await {
            tracing::error!("API server error: {}", e);
        }
    });

    println!(
        "{} Control API: {}",
        "✓".green(),
        format!("http://127.0.0.1:{}", api::API_PORT).dimmed()
    );

    let core_rx = core.clone();
    let contacts_rx = contacts.clone();
    let history_rx = history.clone();
    let peers_rx = peers.clone();
    let ledger_rx = ledger.clone();
    let outbox_rx = outbox.clone();

    // Stdin handling
    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = tokio::io::AsyncBufReadExt::lines(stdin);

    loop {
        tokio::select! {
                    // 1. Swarm Events (Network -> App -> UI)
                    Some(event) = event_rx.recv() => {
                        match event {
                            SwarmEvent::PeerDiscovered(peer_id) => {
                                 let mut p = peers_rx.lock().await;
                                 if let std::collections::hash_map::Entry::Vacant(e) = p.entry(peer_id) {
                                     e.insert(None);
                                     println!("\n{} Peer: {}", "✓".green(), peer_id);
                                     print!("> ");
                                     let _ = std::io::Write::flush(&mut std::io::stdout());
                                     let _ = contacts_rx.update_last_seen(peer_id.to_string());

                                     // Try to get public key from existing contact, if available
                                     let (public_key, identity) = contacts_rx.get(peer_id.to_string())
                                         .ok().flatten()
                                         .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                         .unwrap_or((None, None));

                                     let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::PeerDiscovered {
                                         peer_id: peer_id.to_string(),
                                         transport: "tcp".to_string(),
                                         public_key: public_key.unwrap_or_default(),
                                         identity: identity.unwrap_or_default(),
                                     }));
                                     let n = notif_peer_discovered(PeerDiscoveredParams {
                                         peer_id: peer_id.to_string(),
                                         transport: "tcp".to_string(),
                                         public_key: contacts_rx
                                             .get(peer_id.to_string())
                                             .ok()
                                             .flatten()
                                             .map(|c| c.public_key),
                                     });
                                     if let Ok(v) = serde_json::to_value(&n) {
                                         let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                     }

                                     // Register peer with transport bridge using default capabilities
                                     let capabilities = vec![TransportType::Internet, TransportType::Local];
                                     let capabilities_clone = capabilities.clone();
                                     let mut bridge = transport_bridge.lock().await;
                                     bridge.register_peer(peer_id, capabilities);
                                     tracing::info!("Registered transport capabilities for {}: {:?}", peer_id, capabilities_clone);

                                     // AUTO LEDGER EXCHANGE: Share our known peers with the new connection
                                     let entries = {
                                         let l = ledger_rx.lock().await;
                                         l.to_shared_entries()
                                     };
                                     if let Err(e) = swarm_handle.share_ledger(peer_id, entries).await {
                                         tracing::warn!("Failed to share ledger with {}: {}", peer_id, e);
                                     }

                                     // OUTBOX FLUSH: Deliver any queued messages for this peer now
                                     // that they are online. We drain (remove-and-return) the queue
                                     // atomically; failed sends are re-enqueued so they retry on the
                                     // next connection.
                                     //
                                     // KEY MATCHING: `peer_id.to_string()` here is the libp2p PeerId
                                     // (a base58-encoded multihash derived from the peer's Ed25519 key,
                                     // e.g. "12D3Koo..."). The outbox stores messages keyed by
                                     // `QueuedMessage::recipient_id`, which is set to `contact.peer_id`
                                     // in `cmd_send_offline`. `Contact::peer_id` is documented and
                                     // populated as the libp2p PeerId string — users supply it via
                                     // `scm contact add <peer-id> <public-key>`. The `scm identity`
                                     // display shows both "ID" (Blake3 identity_id) and "Peer ID
                                     // (Network)" (the libp2p PeerId); contacts must use the *Peer ID
                                     // (Network)* value for this flush to match. The two identifiers
                                     // are distinct strings; using the Blake3 identity_id as the
                                     // contact peer_id would silently break outbox delivery.
                                     let queued = {
                                         let mut ob = outbox_rx.lock().await;
                                         ob.drain_for_peer(&peer_id.to_string())
                                     };

                                     if !queued.is_empty() {
                                         tracing::info!(
                                             "Flushing {} queued message(s) to newly-connected peer {}",
                                             queued.len(),
                                             peer_id
                                         );
                                     }

                                     for msg in queued {
                                         let msg_id = msg.message_id.clone();
                                         match swarm_handle.send_message(peer_id, msg.envelope_data.clone(), None, None).await {
                                             Ok(()) => {
                                                 tracing::info!(
                                                     "Flushed queued message {} to {}",
                                                     msg_id,
                                                     peer_id
                                                 );
                                             }
                                             Err(e) => {
                                                 // Re-enqueue on failure so it is retried next connect.
                                                 tracing::warn!(
                                                     "Failed to flush queued message {} to {}: {} — re-enqueuing",
                                                     msg_id,
                                                     peer_id,
                                                     e
                                                 );
                                                 let mut ob = outbox_rx.lock().await;
                                                 if let Err(eq_err) = ob.enqueue(msg) {
                                                     tracing::error!(
                                                         "Failed to re-enqueue message {}: {}",
                                                         msg_id,
                                                         eq_err
                                                     );
                                                 }
                                             }
                                         }
                                     }
                                 }
                            }
                            SwarmEvent::PeerDisconnected(peer_id) => {
                                peers_rx.lock().await.remove(&peer_id);

                                // Record disconnect in ledger (useful for backoff tracking)
                                // We find the entry by PeerID and record failure
                                let mut l = ledger_rx.lock().await;
                                if let Some(entry) = l.find_by_peer_id(&peer_id.to_string()) {
                                    let multiaddr = entry.multiaddr.clone();
                                    l.record_failure(&multiaddr);
                                }
                            }

                            // LEDGER EXCHANGE: Received peer list from a connected peer
                            SwarmEvent::LedgerReceived { from_peer, entries } => {
                                let mut l = ledger_rx.lock().await;
                                let new_count = l.merge_shared_entries(&entries);

                                if new_count > 0 {
                                    println!(
                                        "\n{} 📒 Learned {} new peers from {}",
                                        "✓".green(),
                                        new_count,
                                        from_peer
                                    );
                                    print!("> ");
                                    let _ = std::io::Write::flush(&mut std::io::stdout());

                                    // Save immediately after learning new peers
                                    if let Err(e) = l.save(&data_dir) {
                                        tracing::error!("Failed to save ledger: {}", e);
                                    }

                                    // Dial newly discovered peers
                                    let new_addrs: Vec<String> = entries.iter()
                                        .map(|e| ledger::strip_peer_id(&e.multiaddr))
                                        .collect();
                                    drop(l); // release lock before dialing

                                    for addr_str in new_addrs {
                                        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                            let _ = swarm_handle.dial(addr).await;
                                        }
                                    }
                                }
                            }

                            // IDENTIFY: Peer identity confirmed — update ledger
                            SwarmEvent::PeerIdentified { peer_id, listen_addrs, .. } => {
                                let mut l = ledger_rx.lock().await;
                                for addr in &listen_addrs {
                                    l.record_connection(&addr.to_string(), &peer_id.to_string());
                                }
                            }

                            // GOSSIPSUB: New topic discovered
                            SwarmEvent::TopicDiscovered { peer_id, topic } => {
                                tracing::info!("Topic discovered from {}: {}", peer_id, topic);
                                // Record the topic in the ledger for this peer
                                let mut l = ledger_rx.lock().await;
                                if let Some(entry) = l.find_by_peer_id(&peer_id.to_string()) {
                                    let multiaddr = entry.multiaddr.clone();
                                    l.record_topic(&multiaddr, &topic);
                                }
                            }

                            SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                                // Extract sender's Ed25519 public key from the envelope before decryption.
                                // We need it to encrypt the delivery receipt back to them.
                                let sender_public_key_hex = decode_envelope(&envelope_data)
                                    .ok()
                                    .map(|env| hex::encode(&env.sender_public_key));

                                if let Ok(msg) = core_rx.receive_message(envelope_data) {
                                    match msg.message_type {
                                        MessageType::OnionRelay => {
                                            // Forward onion-routed packet to next hop
                                            let next_hop_hex = msg.recipient_id.clone();
                                            let payload = msg.payload.clone();

                                            if let Ok(next_hop_bytes) = hex::decode(&next_hop_hex) {
                                                // Convert Ed25519 PK to libp2p PeerId
                                                if let Ok(keypair) = libp2p::identity::ed25519::Keypair::try_from_bytes(&mut next_hop_bytes[..32].to_vec()) {
                                                    let next_peer_id = libp2p::PeerId::from_public_key(&keypair.public().into());

                                                    tracing::info!("Relaying onion packet from {} to next hop {}", peer_id, next_peer_id);
                                                    let swarm_clone = swarm_handle.clone();
                                                    tokio::spawn(async move {
                                                        if let Err(e) = swarm_clone.send_message(next_peer_id, payload, None, None).await {
                                                            tracing::warn!("Failed to relay onion packet to {}: {}", next_peer_id, e);
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                        MessageType::Text => {
                                            let text = msg.text_content().unwrap_or_else(|| "<binary>".into());
                                            let sender_name = contacts_rx.get(peer_id.to_string())
                                                .ok().flatten()
                                                .map(|c| c.display_name().to_string())
                                                .unwrap_or_else(|| peer_id.to_string());

                                            println!("\n{} {}: {}", "←".bright_blue(), sender_name.bright_cyan(), text);
                                            print!("> ");
                                            let _ = std::io::Write::flush(&mut std::io::stdout());


                                            let ts = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();
                                            let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::MessageReceived {
                                                from: peer_id.to_string(),
                                                content: text.clone(),
                                                timestamp: ts,
                                                message_id: msg.id.clone(),
                                            }));
                                            let mn = notif_message_received(MessageReceivedParams {
                                                from: peer_id.to_string(),
                                                content: text,
                                                timestamp: ts,
                                                message_id: msg.id.clone(),
                                            });
                                            if let Ok(v) = serde_json::to_value(&mn) {
                                                let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                            }

                                            // Send delivery receipt back to sender.
                                            if let Some(ref pk_hex) = sender_public_key_hex {
                                                match core_rx.prepare_receipt(pk_hex.clone(), msg.id.clone()) {
                                                    Ok(ack_bytes) => {
                                                        tracing::debug!("Sending delivery ACK for {} to {}", msg.id, peer_id);
                                                        if let Err(e) = swarm_handle.send_message(peer_id, ack_bytes, None, None).await {
                                                            tracing::debug!("Failed to send delivery ACK to {}: {}", peer_id, e);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        tracing::debug!("Failed to prepare delivery ACK: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        MessageType::Receipt => {
                                            // Received a delivery receipt — the remote peer confirmed delivery.
                                            if let Ok(receipt) = bincode::deserialize::<scmessenger_core::Receipt>(&msg.payload) {
                                                let short_id = receipt.message_id.get(..8).unwrap_or(&receipt.message_id);
                                                println!("\n{} Delivered: {}", "✓✓".green(), short_id);
                                                print!("> ");
                                                let _ = std::io::Write::flush(&mut std::io::stdout());
                                                tracing::debug!("Delivery ACK received from {}: msg_id={}", peer_id, receipt.message_id);

                                                // Mark the message as delivered in history
                                                if let Err(e) = history_rx.mark_delivered(receipt.message_id.clone()) {
                                                    tracing::warn!("Failed to mark message {} as delivered: {}", receipt.message_id, e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::ListeningOn(addr) => {
                                println!("{} Listening on {}", "✓".green(), addr);
                            }
                            _ => {}
                        }
                    }



                    // 2. UI Commands (UI -> App -> Network)
                    Some(cmd) = ui_cmd_rx.recv() => {
                        match cmd {
                            server::UiCommand::IdentityShow => {
                                let i = core_rx.get_identity_info();
                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::IdentityInfo {
                                    peer_id: i.identity_id.unwrap_or_default(),
                                    public_key: i.public_key_hex.unwrap_or_default(),
                                    nickname: i.nickname,
                                    libp2p_peer_id: i.libp2p_peer_id,
                                }));
                            }
                            server::UiCommand::IdentityExport => {
                                let i = core_rx.get_identity_info();
                                let data_dir = config::Config::data_dir().unwrap_or_default();
                                let storage_path = data_dir.join("storage");

                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::IdentityExportData {
                                    identity_id: i.identity_id.unwrap_or_default(),
                                    public_key: i.public_key_hex.unwrap_or_default(),
                                    private_key: "Keys are stored securely in the data directory.".to_string(),
                                    storage_path: storage_path.display().to_string(),
                                }));
                            }
                            server::UiCommand::ContactList => {
                                if let Ok(list) = contacts_rx.list() {
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                }
                            }
                            server::UiCommand::HistoryList { peer_id, limit } => {
                                let l = limit.unwrap_or(50);
                                if let Ok(messages) = history_rx.conversation(peer_id.clone(), l) {
                                    let history_messages = messages.into_iter().map(|m| {
                                        crate::api::HistoryMessage {
                                            peer_id: m.peer_id,
                                            content: m.content,
                                            direction: match m.direction {
                                                MessageDirection::Sent => "sent".to_string(),
                                                MessageDirection::Received => "received".to_string(),
                                            },
                                            timestamp: m.timestamp,
                                        }
                                    }).collect::<Vec<_>>();
                                    let history_messages: Vec<serde_json::Value> = history_messages.into_iter().map(|m| serde_json::to_value(m).unwrap_or_default()).collect();
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::HistoryList {
                                        peer_id,
                                        messages: history_messages
                                    }));
                                }
                            }
                            server::UiCommand::Status => {
                                let count = peers_rx.lock().await.len();
                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::NetworkStatus {
                                    status: "online".to_string(),
                                    peer_count: count
                                }));
                            }
                            server::UiCommand::Send { recipient, message, id } => {
                                // Resolve recipient to PeerID and PublicKey
                                let peer_id_res = recipient.parse::<libp2p::PeerId>();
                                let contact_res = contacts_rx.get(recipient.clone());

                                let target_peer = if let Ok(pid) = peer_id_res {
                                    Some(pid)
                                } else if let Ok(Some(contact)) = contact_res {
                                    contact.peer_id.parse().ok()
                                } else {
                                    None
                                };

                                if let Some(target) = target_peer {
                                     // Try to find public key
                                     let pk_opt = if let Ok(Some(c)) = contacts_rx.get(target.to_string()) {
                                         Some(c.public_key)
                                     } else { None };

                                     if let Some(pk) = pk_opt {
                                         // prepare_message_with_id automatically saves outgoing history
        if let Ok(prep) = core_rx.prepare_message_with_id(pk.clone(), message.clone(), scmessenger_core::MessageType::Text, None) {
                                             if swarm_handle.send_message(target, prep.envelope_data, None, None).await.is_ok() {
                                                 let mid = id.clone().unwrap_or_default();
                                                 let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::MessageStatus {
                                                     message_id: mid.clone(),
                                                     status: "sent".to_string()
                                                 }));
                                                 let dn = notif_delivery_status(DeliveryStatusParams {
                                                     message_id: mid,
                                                     status: "sent".to_string(),
                                                 });
                                                 if let Ok(v) = serde_json::to_value(&dn) {
                                                     let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                                 }
                                             }
                                         }
                                     }
                                }
                            }
                            server::UiCommand::ContactAdd { peer_id, name, public_key } => {
                                // Assuming public key is provided or we can fetch it? MVP assumes provided.
                                if let Some(pk) = public_key {
                                    // Validate public key before adding
                                    if let Err(e) = scmessenger_core::crypto::validate_ed25519_public_key(&pk) {
                                        tracing::warn!("Failed to add contact {}: invalid public key - {}", peer_id, e);
                                        let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::Error {
                                            message: format!("Invalid public key: {}", e)
                                        }));
                                        continue;
                                    }

                                    let contact = Contact::new(peer_id.clone(), pk)
                                        .with_nickname(name.unwrap_or(peer_id));
                                    let _ = contacts_rx.add(contact);
                                    if let Ok(list) = contacts_rx.list() {
                                        let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                    }
                                }
                            }
                            server::UiCommand::ContactRemove { contact } => {
                                 // remove by peer_id (assuming contact arg is peer_id for now, or resolving nickname)
                                 // contacts.remove takes peer_id string
                                 if contacts_rx.remove(contact).is_ok() {
                                     if let Ok(list) = contacts_rx.list() {
                                         let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                     }
                                 }
                            }
                            server::UiCommand::ConfigGet { key } => {
                                if let Ok(cfg) = config::Config::load() {
                                    let value = cfg.get(&key);
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ConfigValue {
                                        key: key.clone(),
                                        value: value.unwrap_or_default(),
                                    }));
                                }
                            }
                            server::UiCommand::ConfigList => {
                                if let Ok(cfg) = config::Config::load() {
                                    let config_data = cfg.list();
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ConfigData {
                                        config: serde_json::to_value(&config_data).unwrap_or_default(),
                                    }));
                                }
                            }
                            server::UiCommand::ConfigSet { key, value } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    if cfg.set(&key, &value).is_ok() {
                                        // Config updated
                                    }
                                }
                            }
                            server::UiCommand::ConfigBootstrapAdd { multiaddr } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    let _ = cfg.add_bootstrap_node(multiaddr.clone());
                                }
                            }
                            server::UiCommand::ConfigBootstrapRemove { multiaddr } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    let _ = cfg.remove_bootstrap_node(&multiaddr);
                                }
                            }
                            server::UiCommand::FactoryReset => {
                                println!("{} Factory Reset initiated from UI...", "⚠".yellow());
                                // Attempt to clean data dir. This is aggressive.
                                if let Ok(data_dir) = config::Config::data_dir() {
                                     // On unix we can delete even if open? Sometimes.
                                     // Best effort: Log and Exit
                                     println!("Process will exit to clear data.");
                                     let _ = std::fs::remove_dir_all(&data_dir);
                                }
                                std::process::exit(0);
                            }
                            server::UiCommand::Restart => {
                                println!("Restart requested from UI - shutting down...");
                                std::process::exit(0);
                            }
                            server::UiCommand::DaemonRpc { id, intent } => {
                                let intent: ClientIntent = serde_json::from_str(&intent)
                                    .unwrap_or(ClientIntent::GetIdentity {});
                                let push = |result: serde_json::Value| {
                                    let resp = rpc_result(Some(serde_json::Value::String(id.clone())), result);
                                    if let Ok(v) = serde_json::to_value(&resp) {
                                        let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                    }
                                };
                                let push_err = |code: i32, msg: String| {
                                    let resp = rpc_error(
                                        Some(serde_json::Value::String(id.clone())),
                                        scmessenger_core::wasm_support::rpc::JsonRpcErrorBody {
                                            code,
                                            message: msg,
                                            data: None,
                                        },
                                    );
                                    if let Ok(v) = serde_json::to_value(&resp) {
                                        let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                    }
                                };
                                match intent {
                                    ClientIntent::GetIdentity {} => {
                                        let i = core_rx.get_identity_info();
                                        push(serde_json::json!({
                                            "peer_id": i.identity_id,
                                            "public_key_hex": i.public_key_hex,
                                            "libp2p_peer_id": i.libp2p_peer_id,
                                            "initialized": i.initialized,
                                            "nickname": i.nickname,
                                        }));
                                    }
                                    ClientIntent::ScanPeers {} => {
                                        let peers: Vec<String> = peers_rx
                                            .lock()
                                            .await
                                            .keys()
                                            .map(|p| p.to_string())
                                            .collect();
                                        push(serde_json::json!({ "peers": peers }));
                                    }
                                    ClientIntent::GetTopology {} => {
                                        let peer_count = peers_rx.lock().await.len();
                                        let (known_peers, bootstrap_nodes) = {
                                            let l = ledger_rx.lock().await;
                                            let known = l
                                                .entries
                                                .values()
                                                .filter(|e| !e.known_topics.is_empty())
                                                .count();
                                            (known, web_ctx.bootstrap_nodes.clone())
                                        };
                                        let topo = MeshTopologyUpdateParams {
                                            peer_count,
                                            known_peers,
                                            bootstrap_nodes,
                                        };
                                        if let Ok(v) = serde_json::to_value(&topo) {
                                            push(v);
                                        }
                                    }
                                    ClientIntent::SendMessage {
                                        recipient,
                                        message,
                                        id: msg_id,
                                    } => {
                                        let peer_id_res = recipient.parse::<libp2p::PeerId>();
                                        let contact_res = contacts_rx.get(recipient.clone());
                                        let target_peer = if let Ok(pid) = peer_id_res {
                                            Some(pid)
                                        } else if let Ok(Some(contact)) = contact_res {
                                            contact.peer_id.parse().ok()
                                        } else {
                                            None
                                        };
                                        let Some(target) = target_peer else {
                                            push_err(-32001, "Recipient not found".into());
                                            continue;
                                        };
                                        let pk_opt = if let Ok(Some(c)) = contacts_rx.get(target.to_string()) {
                                            Some(c.public_key)
                                        } else {
                                            None
                                        };
                                        let Some(pk) = pk_opt else {
                                            push_err(-32002, "No public key for recipient".into());
                                            continue;
                                        };
        match core_rx.prepare_message_with_id(pk.clone(), message.clone(), scmessenger_core::MessageType::Text, None) {
                                            Ok(prep) => {
                                                if swarm_handle
                                                    .send_message(target, prep.envelope_data, None, None)
                                                    .await
                                                    .is_ok()
                                                {
                                                    let mid = msg_id.clone().unwrap_or_default();
                                                    push(serde_json::json!({
                                                        "status": "sent",
                                                        "message_id": mid.clone(),
                                                    }));
                                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(
                                                        server::UiEvent::MessageStatus {
                                                            message_id: mid.clone(),
                                                            status: "sent".to_string(),
                                                        },
                                                    ));
                                                    let dn = notif_delivery_status(DeliveryStatusParams {
                                                        message_id: mid,
                                                        status: "sent".to_string(),
                                                    });
                                                    if let Ok(v) = serde_json::to_value(&dn) {
                                                        let _ =
                                                            ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                                    }
                                                } else {
                                                    push_err(-32003, "Swarm send failed".into());
                                                }
                                            }
                                            Err(e) => {
                                                push_err(-32004, format!("Prepare message: {}", e));
                                            }
                                        }
                                    }
                                    // New intents handled via WebSocket server.
                                    _ => {
                                        push_err(-32601, "Not supported in daemon context".into());
                                    }
                                }
                            }
                        }
                    }

                    // 3. Stdin (User -> App)
                    Ok(Some(line)) = stdin_lines.next_line() => {
                        let line = line.trim();
                        if line.is_empty() {
                             print!("> ");
                             let _ = std::io::Write::flush(&mut std::io::stdout());
                             continue;
                        }
                        if line == "quit" || line == "exit" {
                            println!("Shutting down...");
                            let _ = swarm_handle.shutdown().await;
                            break;
                        }
                        // (Implement simple CLI commands if needed, mirroring old logic)
                        if line == "status" {
                             let c = peers_rx.lock().await.len();
                             println!("Peers: {}", c);
                        }
                        if line == "peers" {
                             let p = peers_rx.lock().await;
                             for k in p.keys() { println!("  {}", k); }
                        }
                        if line == "contacts" {
                            if let Ok(l) = contacts_rx.list() {
                                for c in l { println!("  {}", c.display_name()); }
                            }
                        }

                        print!("> ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    }
                }
    }

    Ok(())
}

/// Headless relay/bootstrap node — runs the full mesh functionality without
/// interactive console. Designed for server, Docker, and GCP deployment.
///
/// Capabilities:
/// - Uses persisted headless network identity (no persisted user profile init)
/// - Starts libp2p swarm listening on configurable multiaddr
/// - Operates as a relay node: forwards all mesh traffic
/// - Subscribes to lobby + mesh gossipsub topics
/// - Serves HTTP landing page and REST API for status
/// - Periodically re-dials bootstrap peers for mesh continuity
/// - Runs forever (no stdin, no quit command)
async fn cmd_relay(listen_addr: String, http_port: u16, node_name: Option<String>) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = Arc::new(IronCore::with_storage(
        storage_path.to_str().unwrap().to_string(),
    ));
    // Load existing identity (if any) so the relay can migrate its network key
    // from the IronCore identity, preserving the PeerId on first upgrade.
    let _ = core.initialize_identity();
    let network_keypair = load_or_create_headless_network_keypair(&storage_path, &core)?;
    let local_peer_id = network_keypair.public().to_peer_id();
    let display_name = node_name
        .clone()
        .unwrap_or_else(|| format!("relay-{}", &local_peer_id.to_string()[..8]));

    // Sync nickname to IronCore identity if provided
    if let Some(ref name) = node_name {
        if let Err(e) = core.set_nickname(name.clone()) {
            tracing::warn!("Failed to sync relay nickname to identity: {}", e);
        }
    }

    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{}",
        "║        SCMessenger Relay/Bootstrap Node (headless)       ║".bright_cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_cyan()
    );
    println!();
    println!("  Node Name:    {}", display_name.bright_green());
    println!(
        "  Peer ID:      {}",
        local_peer_id.to_string().bright_cyan()
    );
    println!("=== OWN_IDENTITY: {} ===", local_peer_id);
    println!(
        "  Public Key:   {}",
        "(headless/identity-agnostic)".bright_yellow()
    );
    println!("  P2P Listen:   {}", listen_addr.green());
    println!("  HTTP Status:  http://0.0.0.0:{}", http_port);
    println!("  WS Bridge:    ws://0.0.0.0:9002 (libp2p-ws)");
    println!(
        "  Discovery:    http://localhost:{}/api/network-info",
        http_port
    );
    println!();

    // Load config for bootstrap nodes
    let config = config::Config::load()?;
    let all_bootstrap = config.bootstrap_nodes.clone();
    println!(
        "  Bootstrap:    {} node(s)",
        all_bootstrap.len().to_string().bright_cyan()
    );
    for (i, node) in all_bootstrap.iter().enumerate() {
        println!("    {}. {}", i + 1, node.dimmed());
    }
    println!();

    // Connection ledger
    let mut connection_ledger = ledger::ConnectionLedger::load(&data_dir)?;
    let known_topics = connection_ledger.all_known_topics();
    for node in &all_bootstrap {
        connection_ledger.add_bootstrap(node, Some(&local_peer_id.to_string()));
    }
    let ledger = Arc::new(tokio::sync::Mutex::new(connection_ledger));

    // Peers map
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // Create transport bridge
    let transport_bridge = Arc::new(tokio::sync::Mutex::new(
        transport_bridge::TransportBridge::new(),
    ));

    // Web context for landing page + API
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: String::new(),
        bootstrap_nodes: all_bootstrap.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
        transport_bridge: transport_bridge.clone(),
        ui_port: http_port,
        core: Some(Arc::clone(&core)),
    });

    // Start HTTP server (landing page + WebSocket)
    let (ui_broadcast, _ui_cmd_rx) = server::start(http_port, web_ctx.clone()).await?;
    println!("{} HTTP server started on port {}", "✓".green(), http_port);

    // Start swarm
    let listen_multiaddr: libp2p::Multiaddr =
        listen_addr.parse().context("Invalid listen multiaddr")?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle = transport::start_swarm(
        network_keypair,
        Some(listen_multiaddr),
        event_tx,
        None,
        true,
    )
    .await?;
    println!("{} P2P swarm started on {}", "✓".green(), listen_addr);

    // Subscribe to topics
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }
    // Subscribe to default topics (hardcoded - matches bootstrap.rs)
    for topic in ["sc-lobby", "sc-mesh"] {
        let _ = swarm_handle.subscribe_topic(topic.to_string()).await;
    }
    println!("{} Subscribed to mesh topics", "✓".green());

    // Contacts + History (for relay message handling)
    let contacts = core.contacts_store_manager();
    let _history = core.history_store_manager();

    // Outbox
    let outbox_path = data_dir.join("outbox");
    let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
    let outbox = match scmessenger_core::store::backend::SledStorage::new(&outbox_path_str) {
        Ok(backend) => Arc::new(tokio::sync::Mutex::new(Outbox::persistent(Arc::new(
            backend,
        )))),
        Err(e) => {
            tracing::warn!(
                "Failed to open persistent outbox, falling back to in-memory: {}",
                e
            );
            Arc::new(tokio::sync::Mutex::new(Outbox::new()))
        }
    };

    // Control API — core is already Arc<IronCore>
    let core_arc = Arc::clone(&core);
    let api_ctx = api::ApiContext {
        core: core_arc.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
    };
    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_ctx).await {
            tracing::error!("API server error: {}", e);
        }
    });
    println!(
        "{} Control API: {}",
        "✓".green(),
        format!("http://127.0.0.1:{}", api::API_PORT).dimmed()
    );

    tokio::spawn(async move {
        ble_daemon::probe_and_log().await;
    });
    let core_ble = Arc::clone(&core_arc);
    let ui_ble = ui_broadcast.clone();
    tokio::spawn(async move {
        ble_mesh::run_ble_central_ingress(core_ble, ui_ble).await;
    });

    let core_ble_adv = Arc::clone(&core_arc);
    tokio::spawn(async move {
        ble_mesh::run_ble_peripheral_advertising(core_ble_adv).await;
    });

    // ── Initial bootstrap dial ──────────────────────────────────────────
    {
        let swarm_clone = swarm_handle.clone();
        let ledger_clone = ledger.clone();
        tokio::spawn(async move {
            let addrs = {
                let l = ledger_clone.lock().await;
                l.dialable_addresses(Some(&local_peer_id.to_string()))
            };
            for (i, (multiaddr_str, _)) in addrs.iter().enumerate() {
                let stripped = ledger::strip_peer_id(multiaddr_str);
                if let Ok(addr) = stripped.parse::<Multiaddr>() {
                    let label = ledger::extract_ip_port(multiaddr_str)
                        .unwrap_or_else(|| multiaddr_str.clone());
                    println!("  {}. 📞 Dialing {} ...", i + 1, label);
                    match swarm_clone.dial(addr).await {
                        Ok(_) => println!("  {} Dial initiated to {}", "✓".green(), label),
                        Err(e) => {
                            tracing::warn!("Dial failed to {}: {}", label, e);
                            ledger_clone.lock().await.record_failure(multiaddr_str);
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
            }
        });
    }

    // ── Periodic bootstrap re-dial (every 120 seconds) ──────────────────
    {
        let swarm_clone = swarm_handle.clone();
        let ledger_clone = ledger.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                let addrs = {
                    let l = ledger_clone.lock().await;
                    l.dialable_addresses(Some(&local_peer_id.to_string()))
                };
                for (multiaddr_str, _) in &addrs {
                    let stripped = ledger::strip_peer_id(multiaddr_str);
                    if let Ok(addr) = stripped.parse::<Multiaddr>() {
                        let _ = swarm_clone.dial(addr).await;
                    }
                }
                tracing::info!("Periodic re-dial: {} addresses attempted", addrs.len());
            }
        });
    }

    // ── Status broadcast (every 10 seconds) ─────────────────────────────
    let ui_broadcast_clone = ui_broadcast.clone();
    let peers_status = peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let count = peers_status.lock().await.len();
            let _ = ui_broadcast_clone.send(server::UiOutbound::Legacy(
                server::UiEvent::NetworkStatus {
                    status: "online".to_string(),
                    peer_count: count,
                },
            ));
        }
    });

    // ── Periodic ledger save (every 60 seconds) ─────────────────────────
    let ledger_save = ledger.clone();
    let data_dir_save = data_dir.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let mut l = ledger_save.lock().await;
            if let Err(e) = l.save(&data_dir_save) {
                tracing::error!("Failed to save ledger: {}", e);
            }
        }
    });

    println!();
    println!("{}", "Relay node is running. Press Ctrl+C to stop.".bold());
    println!();

    // ── Main event loop (headless — no stdin) ───────────────────────────
    let contacts_rx = contacts.clone();
    let ledger_rx = ledger.clone();
    let outbox_rx = outbox.clone();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    SwarmEvent::PeerDiscovered(peer_id) => {
                        let mut p = peers.lock().await;
                        if let std::collections::hash_map::Entry::Vacant(e) = p.entry(peer_id) {
                            e.insert(None);
                            tracing::info!("Peer discovered: {}", peer_id);
                            let _ = contacts_rx.update_last_seen(peer_id.to_string());

                            let (public_key, identity) = contacts_rx.get(peer_id.to_string())
                                .ok().flatten()
                                .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                .unwrap_or((None, None));

                            let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::PeerDiscovered {
                                peer_id: peer_id.to_string(),
                                transport: "tcp".to_string(),
                                public_key: public_key.unwrap_or_default(),
                                identity: identity.unwrap_or_default(),
                            }));
                            let n = notif_peer_discovered(PeerDiscoveredParams {
                                peer_id: peer_id.to_string(),
                                transport: "tcp".to_string(),
                                public_key: contacts_rx
                                    .get(peer_id.to_string())
                                    .ok()
                                    .flatten()
                                    .map(|c| c.public_key),
                            });
                            if let Ok(v) = serde_json::to_value(&n) {
                                let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                            }

                            // Register peer with transport bridge using default capabilities
                            let capabilities = vec![TransportType::Internet, TransportType::Local];
                            let capabilities_clone = capabilities.clone();
                            let mut bridge = transport_bridge.lock().await;
                            bridge.register_peer(peer_id, capabilities);
                            tracing::info!("Registered transport capabilities for {}: {:?}", peer_id, capabilities_clone);

                            // Share ledger with new peer
                            let entries = {
                                let l = ledger_rx.lock().await;
                                l.to_shared_entries()
                            };
                            if let Err(e) = swarm_handle.share_ledger(peer_id, entries).await {
                                tracing::warn!("Failed to share ledger with {}: {}", peer_id, e);
                            }

                            // Flush outbox for this peer
                            let queued = {
                                let mut ob = outbox_rx.lock().await;
                                ob.drain_for_peer(&peer_id.to_string())
                            };
                            if !queued.is_empty() {
                                tracing::info!("Flushing {} queued message(s) to {}", queued.len(), peer_id);
                            }
                            for msg in queued {
                                let msg_id = msg.message_id.clone();
                                if let Err(e) = swarm_handle.send_message(peer_id, msg.envelope_data.clone(), None, None).await {
                                    tracing::warn!("Failed to flush queued message {} to {}: {}", msg_id, peer_id, e);
                                    let mut ob = outbox_rx.lock().await;
                                    let _ = ob.enqueue(msg);
                                }
                            }
                        }
                    }
                    SwarmEvent::PeerDisconnected(peer_id) => {
                        peers.lock().await.remove(&peer_id);
                        let mut l = ledger_rx.lock().await;
                        if let Some(entry) = l.find_by_peer_id(&peer_id.to_string()) {
                            let multiaddr = entry.multiaddr.clone();
                            l.record_failure(&multiaddr);
                        }
                        tracing::info!("Peer disconnected: {}", peer_id);
                    }
                    SwarmEvent::LedgerReceived { from_peer, entries } => {
                        let mut l = ledger_rx.lock().await;
                        let new_count = l.merge_shared_entries(&entries);
                        if new_count > 0 {
                            tracing::info!("Learned {} new peers from {}", new_count, from_peer);
                            if let Err(e) = l.save(&data_dir) {
                                tracing::error!("Failed to save ledger: {}", e);
                            }
                            let new_addrs: Vec<String> = entries.iter()
                                .map(|e| ledger::strip_peer_id(&e.multiaddr))
                                .collect();
                            drop(l);
                            for addr_str in new_addrs {
                                if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                    let _ = swarm_handle.dial(addr).await;
                                }
                            }
                        }
                    }
                    SwarmEvent::PeerIdentified { peer_id, listen_addrs, .. } => {
                        let mut l = ledger_rx.lock().await;
                        for addr in &listen_addrs {
                            l.record_connection(&addr.to_string(), &peer_id.to_string());
                        }
                    }
                    SwarmEvent::TopicDiscovered { peer_id, topic } => {
                        tracing::info!("Topic discovered from {}: {}", peer_id, topic);
                        let mut l = ledger_rx.lock().await;
                        if let Some(entry) = l.find_by_peer_id(&peer_id.to_string()) {
                            let multiaddr = entry.multiaddr.clone();
                            l.record_topic(&multiaddr, &topic);
                        }
                    }
                    SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                        // In relay mode, we automatically peel and forward onion layers
                        if let Ok(msg) = core_arc.receive_message(envelope_data.clone()) {
                            if msg.message_type == scmessenger_core::MessageType::OnionRelay {
                                let next_hop_hex = msg.recipient_id.clone();
                                let payload = msg.payload.clone();

                                if let Ok(next_hop_bytes) = hex::decode(&next_hop_hex) {
                                    if let Ok(libp2p_kp) = libp2p::identity::ed25519::Keypair::try_from_bytes(&mut next_hop_bytes[..32].to_vec()) {
                                        let next_peer_id = libp2p::PeerId::from_public_key(&libp2p::identity::PublicKey::from(libp2p_kp.public()));

                                        tracing::info!("Relay node: forwarding onion packet to {}", next_peer_id);
                                        let swarm_clone = swarm_handle.clone();
                                        tokio::spawn(async move {
                                            let _ = swarm_clone.send_message(next_peer_id, payload, None, None).await;
                                        });
                                    }
                                }
                            }
                        }

                        // Also log standard envelopes for debugging
                        if let Ok(env) = decode_envelope(&envelope_data) {
                            let sender_key = hex::encode(&env.sender_public_key);
                            tracing::debug!(
                                "Relayed envelope from {} sender={} bytes={}",
                                peer_id,
                                &sender_key[..sender_key.len().min(12)],
                                envelope_data.len()
                            );
                        }
                    }
                    SwarmEvent::ListeningOn(addr) => {
                        tracing::info!("Listening on {}", addr);
                    }
                    _ => {}
                }
            }

            // Ctrl+C shutdown
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down relay node...");
                let _ = swarm_handle.shutdown().await;
                let mut l = ledger.lock().await;
                let _ = l.save(&data_dir);
                break;
            }
        }
    }

    println!("{} Relay node stopped.", "✓".green());
    Ok(())
}

async fn cmd_send_offline(recipient: String, message: String) -> Result<()> {
    // Try to use API if a node is running
    if api::is_api_available().await {
        api::send_message_via_api(&recipient, &message)
            .await
            .context("Failed to send message via API")?;
        println!("{} Message sent via running node", "✓".green());
        return Ok(());
    }

    // Fallback to offline mode (encrypt only, no actual send)
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity()
        .context("Failed to load identity")?;

    let contacts = core.contacts_store_manager();

    let contact = find_contact(&contacts, &recipient).context("Contact not found")?;

    let envelope_bytes = core
        .prepare_message(
            contact.public_key.clone(),
            message.clone(),
            scmessenger_core::MessageType::Text,
            None,
        )
        .map(|pm| pm.envelope_data)
        .context("Failed to encrypt message")?;

    println!(
        "{} Message encrypted: {} bytes",
        "✓".green(),
        envelope_bytes.len()
    );

    // Enqueue in the persistent outbox so cmd_start will flush it when the
    // peer comes online.
    let outbox_path = data_dir.join("outbox");
    let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
    match scmessenger_core::store::backend::SledStorage::new(&outbox_path_str) {
        Ok(backend) => {
            let mut outbox = Outbox::persistent(Arc::new(backend));
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let queued_msg = QueuedMessage {
                message_id: uuid::Uuid::new_v4().to_string(),
                recipient_id: contact.peer_id.clone(),
                envelope_data: envelope_bytes,
                queued_at: now,
                attempts: 0,
            };
            match outbox.enqueue(queued_msg) {
                Ok(()) => {
                    println!(
                        "{} Message queued for {} — will be delivered when peer comes online",
                        "✓".green(),
                        contact.display_name().bright_cyan(),
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to enqueue message for {}: {}", contact.peer_id, e);
                    println!("{} Could not queue message: {}", "⚠".yellow(), e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Could not open outbox for queuing: {}", e);
            println!(
                "{} Message encrypted but could not be queued (outbox unavailable: {})",
                "⚠".yellow(),
                e
            );
        }
    }

    Ok(())
}

async fn cmd_status() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());

    let contacts = core.contacts_store_manager();
    let history = core.history_store_manager();
    let stats = history.stats().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    println!("{}", "SCMessenger Status".bold());
    println!();

    println!("Contacts: {}", contacts.list().unwrap_or_default().len());
    println!(
        "Messages: {} (sent: {}, received: {})",
        stats.total_messages, stats.sent_count, stats.received_count
    );

    if api::is_api_available().await {
        println!();
        println!("{}", "Runtime Network Surface".bold());

        match api::get_peers_via_api().await {
            Ok(peers) => {
                println!("Peers: {}", peers.len());
                for peer in peers {
                    let rep_color = if peer.reputation > 80.0 {
                        "green"
                    } else if peer.reputation < 30.0 {
                        "red"
                    } else {
                        "yellow"
                    };
                    println!(
                        "  - {} (reputation: {})",
                        peer.peer_id.dimmed(),
                        format!("{:.1}", peer.reputation).color(rep_color)
                    );
                }
            }
            Err(e) => println!("Peers: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_listeners_via_api().await {
            Ok(listeners) => println!("Listeners: {}", listeners.len()),
            Err(e) => println!("Listeners: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_external_address_via_api().await {
            Ok(addrs) => {
                if addrs.is_empty() {
                    println!("External Addresses: {}", "(none)".dimmed());
                } else {
                    println!("External Addresses:");
                    for addr in addrs {
                        println!("  - {}", addr.dimmed());
                    }
                }
            }
            Err(e) => println!("External Addresses: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_connection_path_state_via_api().await {
            Ok(state) => println!("Connection Path State: {}", state.bright_cyan()),
            Err(e) => println!("Connection Path State: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_drift_state_via_api().await {
            Ok(status) => {
                let state_color = if status.state == "Active" {
                    status.state.bright_green()
                } else {
                    status.state.yellow()
                };
                println!(
                    "Drift Protocol:        {} (store: {} msgs)",
                    state_color, status.store_size
                );
            }
            Err(e) => println!("Drift Protocol:        {} ({})", "unavailable".yellow(), e),
        }

        match api::export_diagnostics_via_api().await {
            Ok(diag) => {
                println!("Diagnostics JSON bytes: {}", diag.len());
            }
            Err(e) => println!("Diagnostics: {} ({})", "unavailable".yellow(), e),
        }
    }

    Ok(())
}

async fn cmd_mark_sent(message_id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let removed = core.mark_message_sent(message_id.clone());
    if removed {
        println!(
            "{} Marked message as sent: {}",
            "✓".green(),
            message_id.bright_cyan()
        );
    } else {
        println!(
            "{} Message ID not found in outbox: {}",
            "⚠".yellow(),
            message_id.dimmed()
        );
    }
    Ok(())
}

async fn cmd_history_clear(yes: bool) -> Result<()> {
    if !yes {
        anyhow::bail!("Refusing destructive clear without --yes");
    }
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    history.clear().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!("{} Cleared all message history", "✓".green());
    Ok(())
}

async fn cmd_history_enforce_retention(max_messages: u32) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    let pruned = history
        .enforce_retention(max_messages)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Retention enforced (max={}): pruned {}",
        "✓".green(),
        max_messages,
        pruned
    );
    Ok(())
}

async fn cmd_history_prune_before(before_timestamp: u64) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    let pruned = history
        .prune_before(before_timestamp)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Pruned {} message(s) older than {}",
        "✓".green(),
        pruned,
        before_timestamp
    );
    Ok(())
}

async fn cmd_block(action: BlockAction) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity()
        .context("Failed to load identity")?;

    match action {
        BlockAction::Add {
            peer_id,
            device_id,
            reason,
        } => {
            core.block_peer(peer_id.clone(), device_id.clone(), reason.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("{} Blocked peer: {}", "✓".green(), peer_id.bright_cyan());
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
            if let Some(r) = reason {
                println!("  Reason: {}", r.dimmed());
            }
        }
        BlockAction::Remove { peer_id, device_id } => {
            core.unblock_peer(peer_id.clone(), device_id.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("{} Unblocked peer: {}", "✓".green(), peer_id.bright_cyan());
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
        }
        BlockAction::Delete {
            peer_id,
            device_id,
            reason,
        } => {
            core.block_and_delete_peer(peer_id.clone(), device_id.clone(), reason.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!(
                "{} Blocked and deleted peer: {} (messages purged)",
                "✓".green(),
                peer_id.bright_cyan()
            );
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
            if let Some(r) = reason {
                println!("  Reason: {}", r.dimmed());
            }
        }
        BlockAction::List => {
            let list = core
                .list_blocked_peers()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if list.is_empty() {
                println!("{}", "No blocked peers.".dimmed());
            } else {
                println!("{} ({} total)", "Blocked Peers".bold(), list.len());
                println!();
                for item in list {
                    let status = if item.is_deleted {
                        "blocked+deleted".bright_red()
                    } else {
                        "blocked".yellow()
                    };
                    println!(
                        "  {} {} ({})",
                        "•".bright_red(),
                        item.peer_id.bright_cyan(),
                        status
                    );
                    println!(
                        "    Blocked at: {}",
                        format_timestamp(item.blocked_at).dimmed()
                    );
                    if let Some(ref reason) = item.reason {
                        println!("    Reason: {}", reason.dimmed());
                    }
                }
            }
        }
        BlockAction::Check { peer_id, device_id } => {
            let blocked = core
                .is_peer_blocked(peer_id.clone(), device_id.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if blocked {
                println!("{} {} is blocked", "✗".red(), peer_id.bright_cyan());
                if let Some(device_id) = device_id {
                    println!("  Device ID: {}", device_id.dimmed());
                }
            } else {
                println!("{} {} is NOT blocked", "✓".green(), peer_id.bright_cyan());
                if let Some(device_id) = device_id {
                    println!("  Device ID: {}", device_id.dimmed());
                }
            }
        }
        BlockAction::Count => {
            let count = core.blocked_count().map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Blocked peers: {}", count);
        }
    }

    Ok(())
}

async fn cmd_history_get(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();

    match history.get(id.clone()) {
        Ok(Some(msg)) => {
            let direction = match msg.direction {
                MessageDirection::Sent => "→ Sent",
                MessageDirection::Received => "← Received",
            };
            println!("{}", "Message Details".bold());
            println!("  ID:        {}", msg.id.bright_cyan());
            println!("  Direction: {}", direction);
            println!("  Peer:      {}", msg.peer_id);
            println!("  Time:      {}", format_timestamp(msg.timestamp));
            println!("  Delivered: {}", msg.delivered);
            println!("  Content:   {}", msg.content);
        }
        Ok(None) => {
            println!("{} Message not found: {}", "⚠".yellow(), id.dimmed());
        }
        Err(e) => {
            anyhow::bail!("Failed to retrieve message: {:?}", e);
        }
    }

    Ok(())
}

async fn cmd_history_stats() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    let stats = history.stats().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    println!("{}", "History Statistics".bold());
    println!("  Total:       {}", stats.total_messages);
    println!("  Sent:        {}", stats.sent_count);
    println!("  Received:    {}", stats.received_count);
    println!("  Undelivered: {}", stats.undelivered_count);

    Ok(())
}

async fn cmd_history_count() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    println!("History count: {}", history.count());
    Ok(())
}

async fn cmd_history_mark_delivered(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    history
        .mark_delivered(id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Marked message as delivered: {}",
        "✓".green(),
        id.bright_cyan()
    );
    Ok(())
}

async fn cmd_history_clear_conversation(peer: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();

    // Try to resolve peer name to peer_id via contacts
    let contacts = core.contacts_store_manager();
    let peer_id = if let Ok(contact) = find_contact(&contacts, &peer) {
        contact.peer_id
    } else {
        peer.clone()
    };

    history
        .remove_conversation(peer_id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Cleared conversation with {}",
        "✓".green(),
        peer_id.bright_cyan()
    );
    Ok(())
}

async fn cmd_history_delete(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let history = core.history_store_manager();
    history
        .delete(id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!("{} Deleted message: {}", "✓".green(), id.bright_cyan());
    Ok(())
}

async fn cmd_test() -> Result<()> {
    println!("{}", "Running self-tests...".bold());
    println!();

    let alice = IronCore::new();
    let bob = IronCore::new();

    alice.initialize_identity()?;
    bob.initialize_identity()?;

    let _alice_info = alice.get_identity_info();
    let bob_info = bob.get_identity_info();

    println!("{} Identity generation", "✓".green());

    let envelope = alice.prepare_message(
        bob_info.public_key_hex.clone().unwrap(),
        "Test message".to_string(),
        scmessenger_core::MessageType::Text,
        None,
    )?;

    println!(
        "{} Message encryption ({} bytes)",
        "✓".green(),
        envelope.envelope_data.len()
    );

    let msg = bob.receive_message(envelope.envelope_data)?;
    assert_eq!(msg.text_content().unwrap(), "Test message");

    println!("{} Message decryption", "✓".green());

    let eve = IronCore::new();
    eve.initialize_identity()?;

    let envelope = alice.prepare_message(
        bob_info.public_key_hex.clone().unwrap(),
        "Secret".to_string(),
        scmessenger_core::MessageType::Text,
        None,
    )?;

    assert!(eve.receive_message(envelope.envelope_data).is_err());
    println!("{} Encryption security", "✓".green());

    println!();
    println!("{}", "All tests passed!".green().bold());

    Ok(())
}

/// Returns true if `s` is exactly 64 lowercase hex characters — the shape of a
/// Blake3 identity_id (32-byte hash → 64 hex chars).  A user who copies their
/// `scm identity` "ID" field will get this format.
fn looks_like_blake3_id(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

/// Returns true if `s` can be parsed as a valid libp2p PeerId
/// (base58-encoded multihash, e.g. "12D3Koo…").
fn looks_like_libp2p_peer_id(s: &str) -> bool {
    s.parse::<libp2p::PeerId>().is_ok()
}

fn find_contact(manager: &ContactManager, query: &str) -> Result<Contact> {
    let list = manager.list().unwrap_or_default();
    let query_lower = query.to_lowercase();

    list.into_iter()
        .find(|c| {
            c.peer_id == query
                || c.nickname.as_ref().map(|n| n.to_lowercase()).as_deref() == Some(&query_lower)
                || c.public_key == query
        })
        .ok_or_else(|| anyhow::anyhow!("Contact not found: {}", query))
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Local, Utc};

    let dt = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = dt.into();

    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn prune_logs(log_dir: &std::path::Path, max_days: u64) -> Result<()> {
    if !log_dir.exists() {
        return Ok(());
    }

    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(max_days * 24 * 60 * 60);

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            tracing::info!("Pruning old log file: {}", path.display());
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn cmd_audit(action: AuditAction) -> Result<()> {
    let _config = config::Config::load()?;
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());

    match action {
        AuditAction::Export { output } => {
            let json = core
                .export_audit_log()
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("{} Audit log exported to {}", "✓".green(), path);
            } else {
                println!("{}", json);
            }
        }
        AuditAction::Verify => match core.validate_audit_chain() {
            Ok(_) => println!("{} Audit chain integrity verified: OK", "✓".green()),
            Err(e) => println!("{} Audit chain validation failed: {:?}", "✗".red(), e),
        },
        AuditAction::Stats => {
            let events = core.get_audit_events_since(0);
            println!("{}", "Audit Log Statistics".bold());
            println!("  Total Events:   {}", events.len());
            if let Some(first) = events.first() {
                println!(
                    "  First Event:    {}",
                    format_timestamp(first.timestamp_unix_secs)
                );
            }
            if let Some(last) = events.last() {
                println!(
                    "  Last Event:     {}",
                    format_timestamp(last.timestamp_unix_secs)
                );
            }
        }
    }
    Ok(())
}

async fn cmd_swarm(action: SwarmAction) -> Result<()> {
    match action {
        SwarmAction::Stats => cmd_swarm_stats().await,
    }
}

async fn cmd_swarm_stats() -> Result<()> {
    println!("{}", "SCMessenger Swarm Connection Stats".bold());
    println!();

    if api::is_api_available().await {
        match api::get_peers_via_api().await {
            Ok(peers) => {
                if peers.is_empty() {
                    println!("{}", "No active peer connections.".yellow());
                    println!();
                    println!("  Start the mesh node with: {}", "scm relay".dimmed());
                    println!(
                        "  Or start the messaging node with: {}",
                        "scm start".dimmed()
                    );
                } else {
                    println!("{:<52} {:<14} {:<10}", "Peer ID", "Reputation", "Status");
                    println!("{:-<52} {:-<14} {:-<10}", "", "", "");

                    for peer in &peers {
                        let status = if peer.reputation > 80.0 {
                            "active".green()
                        } else if peer.reputation > 30.0 {
                            "unstable".yellow()
                        } else {
                            "degraded".red()
                        };

                        println!(
                            "{:<52} {:<14.1} {:<10}",
                            peer.peer_id.dimmed(),
                            peer.reputation,
                            status,
                        );
                    }

                    println!();
                    println!(
                        "{} {} active peer(s) in the swarm.",
                        "ℹ".dimmed(),
                        peers.len()
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to fetch peers: {}", "✗".red(), e);
            }
        }

        if let Ok(listeners) = api::get_listeners_via_api().await {
            println!("  Listeners: {}", listeners.len());
        }

        if let Ok(addrs) = api::get_external_address_via_api().await {
            if !addrs.is_empty() {
                println!("  External addresses:");
                for addr in &addrs {
                    println!("    - {}", addr.dimmed());
                }
            }
        }
    } else {
        let _config = config::Config::load()?;
        let data_dir = config::Config::data_dir()?;
        let storage_path = data_dir.join("storage");
        let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());

        let contacts = core.contacts_store_manager();
        let history = core.history_store_manager();

        let contact_list = contacts.list().unwrap_or_default();
        let stats = history
            .stats()
            .map_err(|e| anyhow::anyhow!("History stats failed: {:?}", e))?;

        println!(
            "  {}",
            "No daemon is running. Local storage overview:".yellow()
        );
        println!();
        println!("  Contacts:        {}", contact_list.len());
        println!("  Total Messages:  {}", stats.total_messages);
        println!("    Sent:          {}", stats.sent_count);
        println!("    Received:      {}", stats.received_count);
        println!();
        println!(
            "  {}",
            "Start the daemon with `scm relay` for live connection stats.".dimmed()
        );
    }

    Ok(())
}
