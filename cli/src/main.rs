// scmessenger-cli — Complete Desktop CLI
//
// Cross-platform (macOS, Linux, Windows) command-line interface for SCMessenger.

mod api;
mod bootstrap;
mod config;
mod contacts;
mod history;
mod ledger;
mod server;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use libp2p::Multiaddr;
use scmessenger_core::message::{decode_envelope, MessageType};
use scmessenger_core::store::{Outbox, QueuedMessage};
use scmessenger_core::transport::{self, SwarmEvent};
use scmessenger_core::IronCore;
use std::collections::HashMap;
use std::sync::Arc;

fn load_or_create_headless_network_keypair(
    storage_path: &std::path::Path,
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
    /// Run headless relay/bootstrap node (no interactive console)
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
    /// Stop the running node
    Stop,
    /// Run self-tests
    Test,
}

#[derive(Subcommand)]
enum IdentityAction {
    Show,
    Export,
    SetName { name: String },
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
    Bootstrap {
        #[command(subcommand)]
        action: BootstrapAction,
    },
}

#[derive(Subcommand)]
enum BootstrapAction {
    Add { multiaddr: String },
    Remove { multiaddr: String },
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

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
        Commands::Test => cmd_test().await,
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
        Some(IdentityAction::Export) => {
            let info = core.get_identity_info();
            println!("{}", "Export Identity (Backup)".bold());
            println!();
            println!(
                "{}",
                "⚠️  WARNING: Keep your keys secure!".bright_red().bold()
            );
            println!();
            println!("Identity ID: {}", info.identity_id.unwrap());
            println!("Public Key:  {}", info.public_key_hex.unwrap());
            println!();
            println!(
                "Keys stored in: {}",
                storage_path.display().to_string().bright_cyan()
            );
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
    if let Some(nick) = info.nickname {
        println!("  Nickname:               {}", nick.bright_cyan());
    }
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
    println!("{}", "Relays / Bootstrap Nodes".bold());
    let nodes = if config.bootstrap_nodes.is_empty() {
        crate::bootstrap::default_bootstrap_nodes()
    } else {
        crate::bootstrap::merge_bootstrap_nodes(config.bootstrap_nodes.clone())
    };

    if nodes.is_empty() {
        println!("  (None configured)");
    } else {
        for (i, node) in nodes.iter().enumerate() {
            println!("  {}. {}", i + 1, node.dimmed());
        }
    }

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

            // Guard: reject Blake3 identity_id where a libp2p Peer ID is required
            if looks_like_blake3_id(&peer_id) {
                eprintln!(
                    "{} That looks like a Blake3 identity ID (64 hex chars), not a libp2p Peer ID.",
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
            let contacts_db = data_dir.join("contacts");
            let contacts = contacts::ContactList::open(contacts_db)?;

            let contact = contacts::Contact::new(peer_id.clone(), public_key)
                .with_nickname(name.clone().unwrap_or_else(|| peer_id.clone()));

            contacts.add(contact)?;

            println!("{} Contact added:", "✓".green());
            if let Some(nickname) = name {
                println!("  Name: {}", nickname.bright_cyan());
            }
            println!("  Peer ID: {}", peer_id);
        }

        _ => {
            // For other contact operations, use direct database access
            let data_dir = config::Config::data_dir()?;
            let contacts_db = data_dir.join("contacts");
            let contacts = contacts::ContactList::open(contacts_db)?;

            match action {
                ContactAction::List => {
                    let list = contacts.list()?;

                    if list.is_empty() {
                        println!("{}", "No contacts yet.".dimmed());
                    } else {
                        println!("{} ({} total)", "Contacts".bold(), list.len());
                        println!();

                        for contact in list {
                            println!(
                                "  {} {}",
                                "•".bright_green(),
                                contact.display_name().bright_cyan()
                            );
                            println!("    Peer ID: {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::Show { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;

                    println!("{}", "Contact Details".bold());
                    println!("  Name:       {}", contact.display_name().bright_cyan());
                    println!("  Peer ID:    {}", contact.peer_id);
                    println!("  Public Key: {}", contact.public_key.bright_yellow());
                    println!("  Added:      {}", format_timestamp(contact.added_at));
                }

                ContactAction::Remove { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;
                    let name = contact.display_name().to_string();

                    contacts.remove(&contact.peer_id)?;
                    println!("{} Removed contact: {}", "✓".green(), name.bright_cyan());
                }

                ContactAction::Search { query } => {
                    let results = contacts.search(&query)?;

                    if results.is_empty() {
                        println!("{}", "No matching contacts.".dimmed());
                    } else {
                        println!("{} ({} matches)", "Search Results".bold(), results.len());
                        println!();

                        for contact in results {
                            println!(
                                "  {} {}",
                                "•".bright_green(),
                                contact.display_name().bright_cyan()
                            );
                            println!("    {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::Add { .. } => unreachable!(),
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

            println!();
            println!("{}", "Bootstrap nodes:".bold());
            if config.bootstrap_nodes.is_empty() {
                println!("  {}", "(none configured)".dimmed());
            } else {
                for (i, node) in config.bootstrap_nodes.iter().enumerate() {
                    println!("  {}. {}", i + 1, node);
                }
            }
        }

        ConfigAction::Bootstrap { action } => match action {
            BootstrapAction::Add { multiaddr } => {
                config.add_bootstrap_node(multiaddr.clone())?;
                println!("{} Added bootstrap node: {}", "✓".green(), multiaddr);
            }

            BootstrapAction::Remove { multiaddr } => {
                config.remove_bootstrap_node(&multiaddr)?;
                println!("{} Removed bootstrap node", "✓".green());
            }

            BootstrapAction::List => {
                println!("{}", "Bootstrap Nodes".bold());
                if config.bootstrap_nodes.is_empty() {
                    println!("  {}", "(none configured)".dimmed());
                } else {
                    for (i, node) in config.bootstrap_nodes.iter().enumerate() {
                        println!("  {}. {}", i + 1, node);
                    }
                }
            }
        },
    }

    Ok(())
}

async fn cmd_history(
    peer_filter: Option<String>,
    search_query: Option<String>,
    limit: usize,
) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let history_db = data_dir.join("history");
    let history = history::MessageHistory::open(history_db)?;

    let messages = if let Some(query) = search_query {
        history.search(&query, limit)?
    } else if let Some(peer) = peer_filter {
        let contacts_db = data_dir.join("contacts");
        let contacts = contacts::ContactList::open(contacts_db)?;

        let peer_id = if let Ok(contact) = find_contact(&contacts, &peer) {
            contact.peer_id
        } else {
            peer
        };

        history.conversation(&peer_id, limit)?
    } else {
        history.recent(None, limit)?
    };

    if messages.is_empty() {
        println!("{}", "No messages found.".dimmed());
        return Ok(());
    }

    println!("{} ({} messages)", "Message History".bold(), messages.len());
    println!();

    for msg in messages {
        let direction = match msg.direction {
            history::Direction::Sent => "→".bright_green(),
            history::Direction::Received => "←".bright_blue(),
        };

        let time = msg.formatted_time().dimmed();
        let peer = msg.peer();

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
    core.initialize_identity()
        .context("Failed to load identity")?;

    let info = core.get_identity_info();

    let contacts_db = data_dir.join("contacts");
    let contacts = Arc::new(contacts::ContactList::open(contacts_db)?);

    let history_db = data_dir.join("history");
    let history = Arc::new(history::MessageHistory::open(history_db)?);

    // ── Outbox — persistent store-and-forward queue ──────────────────────
    // Messages sent to offline peers are queued here and flushed automatically
    // when those peers come online (see PeerDiscovered handler below).
    let outbox_path = data_dir.join("outbox");
    let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
    let outbox = match Outbox::persistent(&outbox_path_str) {
        Ok(ob) => Arc::new(tokio::sync::Mutex::new(ob)),
        Err(e) => {
            tracing::warn!(
                "Failed to open persistent outbox, falling back to in-memory: {}",
                e
            );
            Arc::new(tokio::sync::Mutex::new(Outbox::new()))
        }
    };

    // ── Connection Ledger — persistent peer memory ──────────────────────
    let mut connection_ledger = ledger::ConnectionLedger::load(&data_dir)?;

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
    println!("Landing Page:  http://0.0.0.0:{}", ws_port);
    println!("WebSocket:     ws://localhost:{}/ws", ws_port);
    println!("P2P Listener:  /ip4/0.0.0.0/tcp/{}", p2p_port);
    println!("📒 {}", connection_ledger.summary());
    println!();

    // Use identity keypair for network (unified ID)
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair from identity")?;
    let local_peer_id = network_keypair.public().to_peer_id();

    // NOTE: PeerId is now derived from identity keys. Existing installations that
    // had a separate network_keypair.dat will see their PeerId change. This is
    // intentional to unify identity and network IDs, but may require updating
    // peer expectations/ledgers on migration.

    // ── Seed ledger with bootstrap nodes (after local_peer_id is available) ────
    let all_bootstrap = bootstrap::merge_bootstrap_nodes(config.bootstrap_nodes.clone());
    for node in &all_bootstrap {
        connection_ledger.add_bootstrap(node, Some(&local_peer_id.to_string()));
    }

    println!("{} Peer ID: {}", "✓".green(), local_peer_id);
    println!();

    // Create shared state BEFORE server start so landing page has access
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let ledger = Arc::new(tokio::sync::Mutex::new(connection_ledger));

    // Build web context for landing page + public APIs
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: info.public_key_hex.clone().unwrap_or_default(),
        bootstrap_nodes: all_bootstrap.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
    });

    // Start WebSocket + HTTP Server (serves landing page at /)
    let (ui_broadcast, mut ui_cmd_rx) = server::start(ws_port, web_ctx).await?;

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse()?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle = transport::start_swarm(network_keypair, Some(listen_addr), event_tx).await?;

    println!("{} Network started", "✓".green());

    // Subscribe to any topics from the ledger
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }

    // Subscribe to default topics
    for topic in bootstrap::default_topics() {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }

    println!();
    println!("{}", "Commands:".bold());
    println!("  {} <contact> <message>", "send".bright_green());
    println!("  {}                      ", "contacts".bright_green());
    println!("  {}                       ", "peers".bright_green());
    println!("  {}                      ", "status".bright_green());
    println!("  {}                        ", "quit".bright_green());
    println!();

    let core = Arc::new(core);
    // Note: peers and ledger Arc<Mutex> created above (before server::start)
    // so the landing page and API endpoints have access to them.

    // ── Promiscuous Bootstrap Dialing ────────────────────────────────────
    // Dial bootstrap nodes by IP:Port ONLY (stripped of PeerID).
    // Also dial any peers from the persistent ledger that pass backoff.
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
            let _ = ui_broadcast_clone.send(server::UiEvent::NetworkStatus {
                status: "online".to_string(),
                peer_count: count,
            });
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

    // Start control API server
    let api_ctx = api::ApiContext {
        core: core.clone(),
        contacts: contacts.clone(),
        history: history.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
        peers: peers.clone(),
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
                             let _ = contacts_rx.update_last_seen(&peer_id.to_string());

                             // Try to get public key from existing contact, if available
                             let (public_key, identity) = contacts_rx.get(&peer_id.to_string())
                                 .ok().flatten()
                                 .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                 .unwrap_or((None, None));

                             let _ = ui_broadcast.send(server::UiEvent::PeerDiscovered {
                                 peer_id: peer_id.to_string(),
                                 transport: "tcp".to_string(),
                                 public_key,
                                 identity,
                             });

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
                                 match swarm_handle.send_message(peer_id, msg.envelope_data.clone()).await {
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
                                MessageType::Text => {
                                    let text = msg.text_content().unwrap_or_else(|| "<binary>".into());
                                    let sender_name = contacts_rx.get(&peer_id.to_string())
                                        .ok().flatten()
                                        .map(|c| c.display_name().to_string())
                                        .unwrap_or_else(|| peer_id.to_string());

                                    println!("\n{} {}: {}", "←".bright_blue(), sender_name.bright_cyan(), text);
                                    print!("> ");
                                    let _ = std::io::Write::flush(&mut std::io::stdout());

                                    let record = history::MessageRecord::new_received(peer_id.to_string(), text.clone());
                                    let _ = history_rx.add(record);

                                    let _ = ui_broadcast.send(server::UiEvent::MessageReceived {
                                        from: peer_id.to_string(),
                                        content: text,
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs(),
                                        message_id: msg.id.clone(),
                                    });

                                    // Send delivery receipt back to sender.
                                    if let Some(ref pk_hex) = sender_public_key_hex {
                                        match core_rx.prepare_receipt(pk_hex.clone(), msg.id.clone()) {
                                            Ok(ack_bytes) => {
                                                tracing::debug!("Sending delivery ACK for {} to {}", msg.id, peer_id);
                                                if let Err(e) = swarm_handle.send_message(peer_id, ack_bytes).await {
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
                        let _ = ui_broadcast.send(server::UiEvent::IdentityInfo {
                            peer_id: i.identity_id.unwrap_or_default(),
                            public_key: i.public_key_hex.unwrap_or_default(),
                        });
                    }
                    server::UiCommand::IdentityExport => {
                        let i = core_rx.get_identity_info();
                        let data_dir = config::Config::data_dir().unwrap_or_default();
                        let storage_path = data_dir.join("storage");

                        let _ = ui_broadcast.send(server::UiEvent::IdentityExportData {
                            identity_id: i.identity_id.unwrap_or_default(),
                            public_key: i.public_key_hex.unwrap_or_default(),
                            private_key: "Keys are stored securely in the data directory.".to_string(),
                            storage_path: storage_path.display().to_string(),
                        });
                    }
                    server::UiCommand::ContactList => {
                        if let Ok(list) = contacts_rx.list() {
                            let _ = ui_broadcast.send(server::UiEvent::ContactList { contacts: list });
                        }
                    }
                    server::UiCommand::Status => {
                        let count = peers_rx.lock().await.len();
                        let _ = ui_broadcast.send(server::UiEvent::NetworkStatus {
                            status: "online".to_string(),
                            peer_count: count
                        });
                    }
                    server::UiCommand::Send { recipient, message, id } => {
                        // Resolve recipient to PeerID and PublicKey
                        let peer_id_res = recipient.parse::<libp2p::PeerId>();
                        let contact_res = contacts_rx.get(&recipient);

                        let target_peer = if let Ok(pid) = peer_id_res {
                            Some(pid)
                        } else if let Ok(Some(contact)) = contact_res {
                            contact.peer_id.parse().ok()
                        } else {
                            None
                        };

                        if let Some(target) = target_peer {
                             // Try to find public key
                             let pk_opt = if let Ok(Some(c)) = contacts_rx.get(&target.to_string()) {
                                 Some(c.public_key)
                             } else { None };

                             if let Some(pk) = pk_opt {
                                 if let Ok(env) = core_rx.prepare_message(pk, message.clone()) {
                                     if swarm_handle.send_message(target, env).await.is_ok() {
                                         let _ = ui_broadcast.send(server::UiEvent::MessageStatus {
                                             message_id: id.unwrap_or_default(),
                                             status: "sent".to_string()
                                         });
                                         let record = history::MessageRecord::new_sent(target.to_string(), message);
                                         let _ = history_rx.add(record);
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
                                let _ = ui_broadcast.send(server::UiEvent::Error {
                                    message: format!("Invalid public key: {}", e)
                                });
                                continue;
                            }

                            let contact = contacts::Contact::new(peer_id.clone(), pk)
                                .with_nickname(name.unwrap_or(peer_id));
                            let _ = contacts_rx.add(contact);
                            if let Ok(list) = contacts_rx.list() {
                                let _ = ui_broadcast.send(server::UiEvent::ContactList { contacts: list });
                            }
                        }
                    }
                    server::UiCommand::ContactRemove { contact } => {
                         // remove by peer_id (assuming contact arg is peer_id for now, or resolving nickname)
                         // contacts.remove takes peer_id string
                         if contacts_rx.remove(&contact).is_ok() {
                             if let Ok(list) = contacts_rx.list() {
                                 let _ = ui_broadcast.send(server::UiEvent::ContactList { contacts: list });
                             }
                         }
                    }
                    server::UiCommand::ConfigGet { key } => {
                        if let Ok(cfg) = config::Config::load() {
                            let value = cfg.get(&key);
                            let _ = ui_broadcast.send(server::UiEvent::ConfigValue {
                                key: key.clone(),
                                value,
                            });
                        }
                    }
                    server::UiCommand::ConfigList => {
                        if let Ok(cfg) = config::Config::load() {
                            let config_data = cfg.list();
                            let _ = ui_broadcast.send(server::UiEvent::ConfigData {
                                config: config_data,
                            });
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
                            let _ = cfg.add_bootstrap_node(multiaddr);
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
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    let network_keypair = load_or_create_headless_network_keypair(&storage_path)?;
    let local_peer_id = network_keypair.public().to_peer_id();
    let display_name =
        node_name.unwrap_or_else(|| format!("relay-{}", &local_peer_id.to_string()[..8]));

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
    println!(
        "  Public Key:   {}",
        "(headless/identity-agnostic)".bright_yellow()
    );
    println!("  P2P Listen:   {}", listen_addr.green());
    println!("  HTTP Status:  http://0.0.0.0:{}", http_port);
    println!();

    // Load config for bootstrap nodes
    let config = config::Config::load()?;
    let all_bootstrap = bootstrap::merge_bootstrap_nodes(config.bootstrap_nodes.clone());
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

    // Web context for landing page + API
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: String::new(),
        bootstrap_nodes: all_bootstrap.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
    });

    // Start HTTP server (landing page + WebSocket)
    let (ui_broadcast, _ui_cmd_rx) = server::start(http_port, web_ctx).await?;
    println!("{} HTTP server started on port {}", "✓".green(), http_port);

    // Start swarm
    let listen_multiaddr: libp2p::Multiaddr =
        listen_addr.parse().context("Invalid listen multiaddr")?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle =
        transport::start_swarm(network_keypair, Some(listen_multiaddr), event_tx).await?;
    println!("{} P2P swarm started on {}", "✓".green(), listen_addr);

    // Subscribe to topics
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }
    for topic in bootstrap::default_topics() {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }
    println!("{} Subscribed to mesh topics", "✓".green());

    // Contacts + History (for relay message handling)
    let contacts_db = data_dir.join("contacts");
    let contacts = Arc::new(contacts::ContactList::open(contacts_db)?);
    let history_db = data_dir.join("history");
    let history = Arc::new(history::MessageHistory::open(history_db)?);

    // Outbox
    let outbox_path = data_dir.join("outbox");
    let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
    let outbox = match Outbox::persistent(&outbox_path_str) {
        Ok(ob) => Arc::new(tokio::sync::Mutex::new(ob)),
        Err(e) => {
            tracing::warn!(
                "Failed to open persistent outbox, falling back to in-memory: {}",
                e
            );
            Arc::new(tokio::sync::Mutex::new(Outbox::new()))
        }
    };

    // Control API
    let core = Arc::new(core);
    let api_ctx = api::ApiContext {
        core: core.clone(),
        contacts: contacts.clone(),
        history: history.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
        peers: peers.clone(),
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
            let _ = ui_broadcast_clone.send(server::UiEvent::NetworkStatus {
                status: "online".to_string(),
                peer_count: count,
            });
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
                            let _ = contacts_rx.update_last_seen(&peer_id.to_string());

                            let (public_key, identity) = contacts_rx.get(&peer_id.to_string())
                                .ok().flatten()
                                .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                .unwrap_or((None, None));

                            let _ = ui_broadcast.send(server::UiEvent::PeerDiscovered {
                                peer_id: peer_id.to_string(),
                                transport: "tcp".to_string(),
                                public_key,
                                identity,
                            });

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
                                if let Err(e) = swarm_handle.send_message(peer_id, msg.envelope_data.clone()).await {
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
                        // Headless relay mode intentionally does not decrypt app payloads.
                        // Swarm-level forwarding remains active regardless of local identity state.
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
    // Guard: catch the common mistake of supplying a Blake3 identity_id instead
    // of a contact nickname / libp2p Peer ID.
    if looks_like_blake3_id(&recipient) {
        eprintln!(
            "{} That looks like a Blake3 identity ID (64 hex chars), not a contact nickname or libp2p Peer ID.",
            "⚠ Error:".red()
        );
        eprintln!("  Use a contact nickname, or the 'Peer ID (Network)' shown by: scm identity");
        eprintln!("  The Peer ID starts with '12D3Koo...' and is ~52 characters.");
        return Ok(());
    }

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

    let contacts_db = data_dir.join("contacts");
    let contacts = contacts::ContactList::open(contacts_db)?;

    let contact = find_contact(&contacts, &recipient).context("Contact not found")?;

    let envelope_bytes = core
        .prepare_message(contact.public_key.clone(), message.clone())
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
    match Outbox::persistent(&outbox_path_str) {
        Ok(mut outbox) => {
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
    let contacts_db = data_dir.join("contacts");
    let history_db = data_dir.join("history");

    let contacts = contacts::ContactList::open(contacts_db)?;
    let history = history::MessageHistory::open(history_db)?;
    let stats = history.stats()?;

    println!("{}", "SCMessenger Status".bold());
    println!();

    println!("Contacts: {}", contacts.count());
    println!(
        "Messages: {} (sent: {}, received: {})",
        stats.total_messages, stats.sent_messages, stats.received_messages
    );

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
    )?;

    println!(
        "{} Message encryption ({} bytes)",
        "✓".green(),
        envelope.len()
    );

    let msg = bob.receive_message(envelope)?;
    assert_eq!(msg.text_content().unwrap(), "Test message");

    println!("{} Message decryption", "✓".green());

    let eve = IronCore::new();
    eve.initialize_identity()?;

    let envelope = alice.prepare_message(bob_info.public_key_hex.unwrap(), "Secret".to_string())?;

    assert!(eve.receive_message(envelope).is_err());
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

fn find_contact(contacts: &contacts::ContactList, query: &str) -> Result<contacts::Contact> {
    if let Ok(Some(contact)) = contacts.get(query) {
        return Ok(contact);
    }

    if let Ok(Some(contact)) = contacts.find_by_nickname(query) {
        return Ok(contact);
    }

    if let Ok(Some(contact)) = contacts.find_by_public_key(query) {
        return Ok(contact);
    }

    anyhow::bail!("Contact not found: {}", query)
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Local, Utc};

    let dt = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = dt.into();

    local.format("%Y-%m-%d %H:%M:%S").to_string()
}
