// scmessenger-cli — Complete Desktop CLI
//
// Cross-platform (macOS, Linux, Windows) command-line interface for SCMessenger.

mod config;
mod contacts;
mod history;

use scmessenger_core::IronCore;
use scmessenger_core::transport::{self, SwarmEvent};
use clap::{Parser, Subcommand};
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Context, Result};

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
    Send {
        recipient: String,
        message: String,
    },
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
    Show { contact: String },
    Remove { contact: String },
    Search { query: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    Set { key: String, value: String },
    Get { key: String },
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
        Commands::History { peer, search, limit } => cmd_history(peer, search, limit).await,
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
    core.initialize_identity().context("Failed to initialize identity")?;

    let info = core.get_identity_info();
    println!("  {} Identity created", "✓".green());
    println!();

    println!("{}", "Identity Information:".bold());
    println!("  ID:         {}", info.identity_id.unwrap().bright_cyan());
    println!("  Public Key: {}", info.public_key_hex.unwrap().bright_yellow());
    println!();

    println!("{}", "Next steps:".bold());
    println!("  • Add contacts: {}", "scm contact add <peer-id> <public-key> --name <nickname>".bright_green());
    println!("  • Start node:   {}", "scm start".bright_green());

    Ok(())
}

async fn cmd_identity(action: Option<IdentityAction>) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity().context("Failed to load identity")?;

    let info = core.get_identity_info();

    match action {
        None | Some(IdentityAction::Show) => {
            println!("{}", "Identity Information".bold());
            println!("  ID:         {}", info.identity_id.unwrap().bright_cyan());
            println!("  Public Key: {}", info.public_key_hex.unwrap().bright_yellow());
        }
        Some(IdentityAction::Export) => {
            println!("{}", "Export Identity (Backup)".bold());
            println!();
            println!("{}", "⚠️  WARNING: Keep your keys secure!".bright_red().bold());
            println!();
            println!("Identity ID: {}", info.identity_id.unwrap());
            println!("Public Key:  {}", info.public_key_hex.unwrap());
            println!();
            println!("Keys stored in: {}", storage_path.display().to_string().bright_cyan());
        }
    }

    Ok(())
}

async fn cmd_contact(action: ContactAction) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let contacts_db = data_dir.join("contacts");
    let contacts = contacts::ContactList::open(contacts_db)?;

    match action {
        ContactAction::Add { peer_id, public_key, name } => {
            let contact = contacts::Contact::new(peer_id.clone(), public_key)
                .with_nickname(name.clone().unwrap_or_else(|| peer_id.clone()));

            contacts.add(contact)?;

            println!("{} Contact added:", "✓".green());
            if let Some(nickname) = name {
                println!("  Name: {}", nickname.bright_cyan());
            }
            println!("  Peer ID: {}", peer_id);
        }

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

async fn cmd_history(peer_filter: Option<String>, search_query: Option<String>, limit: usize) -> Result<()> {
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
    let port = port.unwrap_or(config.listen_port);

    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity().context("Failed to load identity")?;

    let info = core.get_identity_info();

    let contacts_db = data_dir.join("contacts");
    let contacts = Arc::new(contacts::ContactList::open(contacts_db)?);

    let history_db = data_dir.join("history");
    let history = Arc::new(history::MessageHistory::open(history_db)?);

    println!("{}", "SCMessenger — Starting...".bold());
    println!();
    println!("Identity: {}", info.identity_id.clone().unwrap().bright_cyan());
    println!();

    let network_keypair = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = network_keypair.public().to_peer_id();
    println!("{} Network peer ID: {}", "✓".green(), local_peer_id);

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
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

    let core = Arc::new(core);
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    let core_rx = core.clone();
    let contacts_rx = contacts.clone();
    let history_rx = history.clone();
    let peers_rx = peers.clone();

    let event_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                SwarmEvent::PeerDiscovered(peer_id) => {
                    let mut p = peers_rx.lock().await;
                    if !p.contains_key(&peer_id) {
                        p.insert(peer_id, None);
                        println!("\n{} Peer: {}", "✓".green(), peer_id);
                        print!("> ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                        let _ = contacts_rx.update_last_seen(&peer_id.to_string());
                    }
                }
                SwarmEvent::PeerDisconnected(peer_id) => {
                    peers_rx.lock().await.remove(&peer_id);
                }
                SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                    match core_rx.receive_message(envelope_data) {
                        Ok(msg) => {
                            let text = msg.text_content().unwrap_or_else(|| "<binary>".into());

                            let sender_name = contacts_rx
                                .get(&peer_id.to_string())
                                .ok()
                                .flatten()
                                .map(|c| c.display_name().to_string())
                                .unwrap_or_else(|| peer_id.to_string());

                            println!("\n{} {}: {}", "←".bright_blue(), sender_name.bright_cyan(), text);

                            let record = history::MessageRecord::new_received(peer_id.to_string(), text);
                            let _ = history_rx.add(record);
                        }
                        Err(_) => {}
                    }
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                SwarmEvent::ListeningOn(addr) => {
                    println!("{} Listening on {}", "✓".green(), addr);
                }
                SwarmEvent::AddressReflected { .. } => {}
            }
        }
    });

    let stdin_task = tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;

        let stdin = tokio::io::BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        print!("> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        while let Ok(Some(line)) = lines.next_line().await {
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

            if line == "contacts" {
                match contacts.list() {
                    Ok(list) => {
                        if list.is_empty() {
                            println!("No contacts.");
                        } else {
                            for c in list {
                                println!("  • {}", c.display_name().bright_cyan());
                            }
                        }
                    }
                    Err(_) => {}
                }
                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            if line == "peers" {
                let p = peers.lock().await;
                if p.is_empty() {
                    println!("No peers.");
                } else {
                    for pid in p.keys() {
                        println!("  • {}", pid);
                    }
                }
                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            if line == "status" {
                let peer_count = peers.lock().await.len();
                let contact_count = contacts.count();
                let msg_count = history.count();

                println!("Peers:    {}", peer_count);
                println!("Contacts: {}", contact_count);
                println!("Messages: {}", msg_count);
                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            if line.starts_with("send ") {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() < 3 {
                    println!("Usage: send <contact> <message>");
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    continue;
                }

                let contact_query = parts[1];
                let message_text = parts[2];

                match find_contact(&contacts, contact_query) {
                    Ok(contact) => {
                        match contact.peer_id.parse::<libp2p::PeerId>() {
                            Ok(peer_id) => {
                                match core.prepare_message(contact.public_key.clone(), message_text.to_string()) {
                                    Ok(envelope_bytes) => {
                                        match swarm_handle.send_message(peer_id, envelope_bytes).await {
                                            Ok(_) => {
                                                println!("{} Sent", "✓".green());

                                                let record = history::MessageRecord::new_sent(
                                                    contact.peer_id.clone(),
                                                    message_text.to_string()
                                                );
                                                let _ = history.add(record);
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => println!("{} Contact not found", "✗".red()),
                }

                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            println!("Try: send, contacts, peers, status, quit");
            print!("> ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    });

    tokio::select! {
        _ = event_task => {}
        _ = stdin_task => {}
    }

    Ok(())
}

async fn cmd_send_offline(recipient: String, message: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(storage_path.to_str().unwrap().to_string());
    core.initialize_identity().context("Failed to load identity")?;

    let envelope_bytes = core
        .prepare_message(recipient.clone(), message.clone())
        .context("Failed to encrypt message")?;

    println!("{} Message encrypted: {} bytes", "✓".green(), envelope_bytes.len());

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
    println!("Messages: {} (sent: {}, received: {})",
        stats.total_messages,
        stats.sent_messages,
        stats.received_messages
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
        "Test message".to_string()
    )?;

    println!("{} Message encryption ({} bytes)", "✓".green(), envelope.len());

    let msg = bob.receive_message(envelope)?;
    assert_eq!(msg.text_content().unwrap(), "Test message");

    println!("{} Message decryption", "✓".green());

    let eve = IronCore::new();
    eve.initialize_identity()?;

    let envelope = alice.prepare_message(
        bob_info.public_key_hex.unwrap(),
        "Secret".to_string()
    )?;

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

    let dt = DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now());
    let local: DateTime<Local> = dt.into();

    local.format("%Y-%m-%d %H:%M:%S").to_string()
}
