// Configuration management for SCMessenger CLI
//
// Cross-platform config stored in:
// - macOS: ~/.config/scmessenger/config.toml
// - Linux: ~/.config/scmessenger/config.toml
// - Windows: %APPDATA%\scmessenger\config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Bootstrap nodes for initial network connection
    pub bootstrap_nodes: Vec<String>,

    /// Default port for listening
    pub listen_port: u16,

    /// Enable mDNS for local network discovery
    pub enable_mdns: bool,

    /// Enable DHT for wide area network discovery
    pub enable_dht: bool,

    /// Storage path for messages and identity
    pub storage_path: Option<String>,

    /// Network settings
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Maximum number of peers to maintain
    pub max_peers: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Enable NAT traversal
    pub enable_nat_traversal: bool,

    /// Enable relay fallback
    pub enable_relay: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![
                // Empty by default - users add their own
            ],
            listen_port: 0, // Random port
            enable_mdns: true,
            enable_dht: true,
            storage_path: None,
            network: NetworkConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            connection_timeout: 30,
            enable_nat_traversal: true,
            enable_relay: true,
        }
    }
}

impl Config {
    /// Get the config directory path (cross-platform)
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("scmessenger");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir)
    }

    /// Get the data directory path (cross-platform)
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .context("Failed to determine data directory")?
            .join("scmessenger");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        Ok(data_dir)
    }

    /// Get the config file path
    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if config_file.exists() {
            let contents = std::fs::read_to_string(&config_file)
                .context("Failed to read config file")?;
            let config: Config = serde_json::from_str(&contents)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&config_file, contents)
            .context("Failed to write config file")?;
        Ok(())
    }

    /// Add a bootstrap node
    pub fn add_bootstrap_node(&mut self, node: String) -> Result<()> {
        if !self.bootstrap_nodes.contains(&node) {
            self.bootstrap_nodes.push(node);
            self.save()?;
        }
        Ok(())
    }

    /// Remove a bootstrap node
    pub fn remove_bootstrap_node(&mut self, node: &str) -> Result<()> {
        self.bootstrap_nodes.retain(|n| n != node);
        self.save()?;
        Ok(())
    }

    /// Set a config value
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "listen_port" => {
                self.listen_port = value.parse()
                    .context("Invalid port number")?;
            }
            "enable_mdns" => {
                self.enable_mdns = value.parse()
                    .context("Invalid boolean value")?;
            }
            "enable_dht" => {
                self.enable_dht = value.parse()
                    .context("Invalid boolean value")?;
            }
            "storage_path" => {
                self.storage_path = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "max_peers" => {
                self.network.max_peers = value.parse()
                    .context("Invalid number")?;
            }
            "connection_timeout" => {
                self.network.connection_timeout = value.parse()
                    .context("Invalid number")?;
            }
            "enable_nat_traversal" => {
                self.network.enable_nat_traversal = value.parse()
                    .context("Invalid boolean value")?;
            }
            "enable_relay" => {
                self.network.enable_relay = value.parse()
                    .context("Invalid boolean value")?;
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
        self.save()?;
        Ok(())
    }

    /// Get a config value
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "listen_port" => Some(self.listen_port.to_string()),
            "enable_mdns" => Some(self.enable_mdns.to_string()),
            "enable_dht" => Some(self.enable_dht.to_string()),
            "storage_path" => self.storage_path.clone(),
            "max_peers" => Some(self.network.max_peers.to_string()),
            "connection_timeout" => Some(self.network.connection_timeout.to_string()),
            "enable_nat_traversal" => Some(self.network.enable_nat_traversal.to_string()),
            "enable_relay" => Some(self.network.enable_relay.to_string()),
            _ => None,
        }
    }

    /// List all config values
    pub fn list(&self) -> Vec<(String, String)> {
        vec![
            ("listen_port".to_string(), self.listen_port.to_string()),
            ("enable_mdns".to_string(), self.enable_mdns.to_string()),
            ("enable_dht".to_string(), self.enable_dht.to_string()),
            ("storage_path".to_string(), self.storage_path.clone().unwrap_or_else(|| "(auto)".to_string())),
            ("max_peers".to_string(), self.network.max_peers.to_string()),
            ("connection_timeout".to_string(), format!("{}s", self.network.connection_timeout)),
            ("enable_nat_traversal".to_string(), self.network.enable_nat_traversal.to_string()),
            ("enable_relay".to_string(), self.network.enable_relay.to_string()),
            ("bootstrap_nodes".to_string(), self.bootstrap_nodes.len().to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.listen_port, 0);
        assert!(config.enable_mdns);
        assert!(config.enable_dht);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.listen_port, deserialized.listen_port);
    }
}
