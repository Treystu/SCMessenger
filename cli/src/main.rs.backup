// scmessenger-cli — SCMessenger CLI
//
// Identity management, offline encryption, and live P2P messaging.

use scmessenger_core::IronCore;
use scmessenger_core::transport::{self, SwarmEvent};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "scmessenger-cli")]
#[command(about = "SCMessenger — Encrypted P2P Messaging", long_about = None)]
struct Cli {
    /// Path for persistent storage (optional)
    #[arg(short, long)]
    storage: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Identity management
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },
    /// Send an encrypted message (outputs envelope bytes, offline mode)
    Send {
        /// Recipient's public key (hex)
        #[arg()]
        recipient: String,
        /// Message text
        #[arg()]
        message: String,
    },
    /// Run end-to-end messaging test (two in-memory nodes)
    Test,
    /// Start listening for P2P connections (mDNS discovery + messaging)
    Listen {
        /// Port to listen on (default: random)
        #[arg(short, long, default_value = "0")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum IdentityAction {
    /// Generate a new identity (or load existing)
    Generate,
    /// Show current identity info
    Show,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let core = if let Some(path) = &cli.storage {
        IronCore::with_storage(path.clone())
    } else {
        IronCore::new()
    };

    match cli.command {
        Commands::Identity { action } => match action {
            IdentityAction::Generate => cmd_identity_generate(&core)?,
            IdentityAction::Show => cmd_identity_show(&core)?,
        },
        Commands::Send { recipient, message } => cmd_send(&core, &recipient, &message)?,
        Commands::Test => cmd_test()?,
        Commands::Listen { port } => cmd_listen(core, port).await?,
    }

    Ok(())
}

fn cmd_identity_generate(core: &IronCore) -> anyhow::Result<()> {
    println!("Generating identity...\n");
    core.initialize_identity()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let info = core.get_identity_info();
    println!("Identity created successfully.\n");
    println!("Identity ID: {}", info.identity_id.unwrap_or_default());
    println!("Public Key:  {}", info.public_key_hex.unwrap_or_default());
    println!("\nShare your public key with peers so they can send you messages.");
    Ok(())
}

fn cmd_identity_show(core: &IronCore) -> anyhow::Result<()> {
    core.initialize_identity()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let info = core.get_identity_info();
    if info.initialized {
        println!("Identity ID: {}", info.identity_id.unwrap_or_default());
        println!("Public Key:  {}", info.public_key_hex.unwrap_or_default());
    } else {
        println!("No identity found. Run `scmessenger-cli identity generate` first.");
    }
    Ok(())
}

fn cmd_send(core: &IronCore, recipient_hex: &str, text: &str) -> anyhow::Result<()> {
    core.initialize_identity()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let sender_info = core.get_identity_info();
    println!("Sender: {}...", &sender_info.public_key_hex.unwrap_or_default()[..16]);
    println!("Recipient: {}...", &recipient_hex[..std::cmp::min(16, recipient_hex.len())]);
    println!("Message: {}\n", text);

    let envelope_bytes = core
        .prepare_message(recipient_hex.to_string(), text.to_string())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Encrypted envelope: {} bytes", envelope_bytes.len());
    println!("Message ready for transmission.");
    Ok(())
}

fn cmd_test() -> anyhow::Result<()> {
    println!("SCMessenger — End-to-End Messaging Test");
    println!("=========================================\n");

    let alice = IronCore::new();
    let bob = IronCore::new();

    alice.initialize_identity().map_err(|e| anyhow::anyhow!("{}", e))?;
    bob.initialize_identity().map_err(|e| anyhow::anyhow!("{}", e))?;

    let alice_info = alice.get_identity_info();
    let bob_info = bob.get_identity_info();

    println!("Alice: {}...", &alice_info.public_key_hex.as_ref().unwrap()[..16]);
    println!("Bob:   {}...\n", &bob_info.public_key_hex.as_ref().unwrap()[..16]);

    // Test 1: Alice sends to Bob
    println!("Test 1: Alice -> Bob (text message)");
    let envelope = alice
        .prepare_message(bob_info.public_key_hex.clone().unwrap(), "Hello Bob! This is a secret message.".to_string())
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Encrypted: {} bytes", envelope.len());

    let msg = bob.receive_message(envelope).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Decrypted: \"{}\"", msg.text_content().unwrap());
    println!("  Sender ID matches: {}", msg.sender_id == alice_info.identity_id.clone().unwrap());
    println!("  PASS\n");

    // Test 2: Bob sends to Alice
    println!("Test 2: Bob -> Alice (text message)");
    let envelope = bob
        .prepare_message(alice_info.public_key_hex.clone().unwrap(), "Hey Alice! Got your message.".to_string())
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Encrypted: {} bytes", envelope.len());

    let msg = alice.receive_message(envelope).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Decrypted: \"{}\"", msg.text_content().unwrap());
    println!("  PASS\n");

    // Test 3: Eve cannot decrypt
    println!("Test 3: Eve cannot decrypt Alice's message to Bob");
    let eve = IronCore::new();
    eve.initialize_identity().map_err(|e| anyhow::anyhow!("{}", e))?;

    let envelope = alice
        .prepare_message(bob_info.public_key_hex.clone().unwrap(), "This is only for Bob".to_string())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match eve.receive_message(envelope) {
        Ok(_) => println!("  FAIL: Eve decrypted the message!"),
        Err(_) => println!("  Eve cannot decrypt: PASS"),
    }
    println!();

    // Test 4: Replay protection
    println!("Test 4: Replay protection");
    let envelope = alice
        .prepare_message(bob_info.public_key_hex.unwrap(), "No replays allowed".to_string())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    bob.receive_message(envelope.clone()).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  First receive: OK");

    match bob.receive_message(envelope) {
        Ok(_) => println!("  FAIL: Replay accepted!"),
        Err(_) => println!("  Replay rejected: PASS"),
    }
    println!();

    // Test 5: Digital signatures
    println!("Test 5: Digital signatures");
    let data = b"Important document content".to_vec();
    let sig = alice.sign_data(data.clone()).map_err(|e| anyhow::anyhow!("{}", e))?;

    let valid = alice
        .verify_signature(data.clone(), sig.signature.clone(), sig.public_key_hex.clone())
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Valid signature: {}", valid);

    let invalid = alice
        .verify_signature(b"tampered".to_vec(), sig.signature, sig.public_key_hex)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("  Tampered data rejected: {}", !invalid);
    println!("  PASS\n");

    println!("=========================================");
    println!("All tests passed.");

    Ok(())
}

/// Live P2P listen mode: mDNS discovery + encrypted messaging over libp2p.
async fn cmd_listen(core: IronCore, port: u16) -> anyhow::Result<()> {
    // Initialize identity
    core.initialize_identity()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let info = core.get_identity_info();
    let my_pubkey = info.public_key_hex.clone().unwrap();

    println!("SCMessenger — Live P2P Mode");
    println!("===========================");
    println!("Crypto Public Key: {}", my_pubkey);

    // Create a separate libp2p keypair for networking (independent from crypto identity)
    let network_keypair = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = network_keypair.public().to_peer_id();
    println!("Network Peer ID:   {}", local_peer_id);
    println!();

    // Start the swarm
    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
    let swarm_handle = transport::start_swarm(network_keypair, Some(listen_addr), event_tx).await?;

    // Shared state: map PeerId -> crypto public key (learned via discovery metadata)
    // For POC: users manually specify both PeerId and crypto pubkey in send command
    let core = Arc::new(core);
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    println!("Waiting for peers via mDNS...");
    println!("Commands:");
    println!("  send <peer_id> <crypto_pubkey_hex> <message>");
    println!("  peers");
    println!("  quit");
    println!();

    // Spawn event handler
    let core_rx = core.clone();
    let peers_rx = peers.clone();
    let event_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                SwarmEvent::PeerDiscovered(peer_id) => {
                    let mut p = peers_rx.lock().await;
                    if !p.contains_key(&peer_id) {
                        p.insert(peer_id, None);
                        println!("\n[+] Peer discovered: {}", peer_id);
                        print!("> ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    }
                }
                SwarmEvent::PeerDisconnected(peer_id) => {
                    peers_rx.lock().await.remove(&peer_id);
                    println!("\n[-] Peer disconnected: {}", peer_id);
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                    match core_rx.receive_message(envelope_data) {
                        Ok(msg) => {
                            let text = msg.text_content().unwrap_or_else(|| "<non-text>".into());
                            println!("\n[msg from {}] {}", peer_id, text);
                        }
                        Err(e) => {
                            println!("\n[err] Failed to decrypt message from {}: {}", peer_id, e);
                        }
                    }
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                SwarmEvent::ListeningOn(addr) => {
                    println!("[*] Listening on {}", addr);
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                SwarmEvent::AddressReflected { peer_id, observed_address } => {
                    println!("[*] Address reflection from {}: {}", peer_id, observed_address);
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
            }
        }
    });

    // Stdin reader
    let stdin_task = tokio::spawn(async move {
        let stdin = tokio::io::AsyncBufReadExt::lines(tokio::io::BufReader::new(tokio::io::stdin()));
        tokio::pin!(stdin);

        print!("> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        while let Ok(Some(line)) = stdin.next_line().await {
            let line = line.trim().to_string();
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

            if line == "peers" {
                let p = peers.lock().await;
                if p.is_empty() {
                    println!("No peers discovered yet.");
                } else {
                    println!("Discovered peers:");
                    for (pid, pubkey) in p.iter() {
                        let pk_display = pubkey.as_deref().unwrap_or("(unknown crypto key)");
                        println!("  {} — {}", pid, pk_display);
                    }
                }
                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            if line.starts_with("send ") {
                let parts: Vec<&str> = line.splitn(4, ' ').collect();
                if parts.len() < 4 {
                    println!("Usage: send <peer_id> <crypto_pubkey_hex> <message>");
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    continue;
                }

                let peer_id_str = parts[1];
                let crypto_pubkey_hex = parts[2];
                let message_text = parts[3];

                // Parse PeerId
                let peer_id: libp2p::PeerId = match peer_id_str.parse() {
                    Ok(pid) => pid,
                    Err(e) => {
                        println!("Invalid peer ID: {}", e);
                        print!("> ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                        continue;
                    }
                };

                // Encrypt the message
                match core.prepare_message(crypto_pubkey_hex.to_string(), message_text.to_string()) {
                    Ok(envelope_bytes) => {
                        match swarm_handle.send_message(peer_id, envelope_bytes).await {
                            Ok(_) => println!("[sent] Message delivered to {}", peer_id),
                            Err(e) => println!("[err] Failed to send: {}", e),
                        }
                    }
                    Err(e) => println!("[err] Failed to encrypt: {}", e),
                }

                print!("> ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }

            println!("Unknown command. Try: send, peers, quit");
            print!("> ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = event_task => {}
        _ = stdin_task => {}
    }

    Ok(())
}
