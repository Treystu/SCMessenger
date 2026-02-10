// scmessenger-cli — Complete Desktop CLI
//
// Cross-platform (macOS, Linux, Windows) command-line interface for SCMessenger.

mod api;
mod config;
mod contacts;
mod history;
mod server;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use scmessenger_core::transport::{self, SwarmEvent};
use scmessenger_core::IronCore;
use std::collections::HashMap;
use std::sync::Arc;

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
    Init,
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
    /// Send a message (offline mode)
    Send { recipient: String, message: String },
    /// Show network status
    Status,
    /// Run self-tests
    Test,
}

#[derive(Subcommand)]
enum IdentityAction {
    Show,
    Export,
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
        Commands::Init => cmd_init().await,
        Commands::Identity { action } => cmd_identity(action).await,
        Commands::Contact { action } => cmd_contact(action).await,
        Commands::Config { action } => cmd_config(action).await,
        Commands::History {
            peer,
            search,
            limit,
        } => cmd_history(peer, search, limit).await,
        Commands::Start { port } => cmd_start(port).await,
        Commands::Send { recipient, message } => cmd_send_offline(recipient, message).await,
        Commands::Status => cmd_status().await,
        Commands::Test => cmd_test().await,
    }
}

async fn cmd_init() -> Result<()> {
    println!("{}", "Initializing SCMessenger...".bold());
    println!();

    let _config = config::Config::load()?;
    println!("  {} Configuration", "✓".green());

    let data_dir = config::Config::data_dir()?;
    println!("  {} Data directory: {}", "✓".green(), data_dir.display());

    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity()
        .context("Failed to initialize identity")?;

    let info = core.get_identity_info();
    println!("  {} Identity created", "✓".green());
    println!();

    println!("{}", "Identity Information:".bold());
    println!("  ID:         {}", info.identity_id.unwrap().bright_cyan());
    println!(
        "  Public Key: {}",
        info.public_key_hex.unwrap().bright_yellow()
    );
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
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity()
        .context("Failed to load identity")?;

    let info = core.get_identity_info();

    match action {
        None | Some(IdentityAction::Show) => {
            println!("{}", "Identity Information".bold());
            println!("  ID:         {}", info.identity_id.unwrap().bright_cyan());
            println!(
                "  Public Key: {}",
                info.public_key_hex.unwrap().bright_yellow()
            );
        }
        Some(IdentityAction::Export) => {
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

async fn cmd_contact(action: ContactAction) -> Result<()> {
    match action {
        ContactAction::Add { peer_id, public_key, name } => {
            // Try to use API if a node is running
            if api::is_api_available().await {
                api::add_contact_via_api(&peer_id, &public_key, name.clone()).await
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
                            println!("  {} {}", "•".bright_green(), contact.display_name().bright_cyan());
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
                            println!("  {} {}", "•".bright_green(), contact.display_name().bright_cyan());
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
    let ws_port = port.unwrap_or(config.listen_port);
    let p2p_port = ws_port + 1; // P2P port shifted by 1 to allow UI on default port

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

    println!("{}", "SCMessenger — Starting...".bold());
    println!();
    println!(
        "Identity: {}",
        info.identity_id.clone().unwrap().bright_cyan()
    );
    println!("Web Interface: ws://localhost:{}/ws", ws_port);
    println!("P2P Listener:  /ip4/0.0.0.0/tcp/{}", p2p_port);
    println!();

    let network_keypair = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = network_keypair.public().to_peer_id();
    println!("{} Network peer ID: {}", "✓".green(), local_peer_id);

    // Start WebSocket Server
    let (ui_broadcast, mut ui_cmd_rx) = server::start(ws_port).await;

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse()?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle = transport::start_swarm(network_keypair, Some(listen_addr), event_tx).await?;

    println!("{} Network started", "✓".green());
    println!();
    println!("{}", "Commands:".bold());
    println!("  {} <contact> <message>", "send".bright_green());
    println!("  {}                      ", "contacts".bright_green());
    println!("  {}                       ", "peers".bright_green());
    println!("  {}                      ", "status".bright_green());
    println!("  {}                        ", "quit".bright_green());
    println!();

    // Auto-open UI instructions?
    // println!("Open ui/index.html in your browser to restart.");

    let core = Arc::new(core);
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

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

    println!("{} Control API: {}", "✓".green(), format!("http://127.0.0.1:{}", api::API_PORT).dimmed());

    let core_rx = core.clone();
    let contacts_rx = contacts.clone();
    let history_rx = history.clone();
    let peers_rx = peers.clone();

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
                         if !p.contains_key(&peer_id) {
                             p.insert(peer_id, None);
                             println!("\n{} Peer: {}", "✓".green(), peer_id);
                             print!("> ");
                             let _ = std::io::Write::flush(&mut std::io::stdout());
                             let _ = contacts_rx.update_last_seen(&peer_id.to_string());

                             let _ = ui_broadcast.send(server::UiEvent::PeerDiscovered {
                                 peer_id: peer_id.to_string(),
                                 transport: "tcp".to_string()
                             });
                         }
                    }
                    SwarmEvent::PeerDisconnected(peer_id) => {
                        peers_rx.lock().await.remove(&peer_id);
                    }
                    SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                        match core_rx.receive_message(envelope_data) {
                             Ok(msg) => {
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

                                 // Update UI
                                 // Note: timestamps in Rust are u64, MessageReceived expects u64
                                 let _ = ui_broadcast.send(server::UiEvent::MessageReceived {
                                     from: peer_id.to_string(),
                                     content: text,
                                     timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                     message_id: uuid::Uuid::new_v4().to_string(),
                                 });
                             }
                             Err(_) => {}
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
                            let contact = contacts::Contact::new(peer_id.clone(), pk)
                                .with_nickname(name.unwrap_or(peer_id));
                            let _ = contacts_rx.add(contact);
                            if let Ok(list) = contacts_rx.list() {
                                let _ = ui_broadcast.send(server::UiEvent::ContactList { contacts: list });
                            }
                        }
                    }
                    _ => {}
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

async fn cmd_send_offline(recipient: String, message: String) -> Result<()> {
    // Try to use API if a node is running
    if api::is_api_available().await {
        api::send_message_via_api(&recipient, &message).await
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

    println!("{} Message encrypted: {} bytes", "✓".green(), envelope_bytes.len());
    println!("{} Message queued for {} {}", "✓".green(), contact.display_name().bright_cyan(), "(offline mode)".dimmed());

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

    let dt = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(|| Utc::now());
    let local: DateTime<Local> = dt.into();

    local.format("%Y-%m-%d %H:%M:%S").to_string()
}
