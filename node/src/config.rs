//! Node configuration.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Main node configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node configuration section
    #[serde(default)]
    pub node: NodeSection,

    /// Network configuration section
    #[serde(default)]
    pub network: NetworkSection,

    /// Runtime configuration section
    #[serde(default)]
    pub runtime: RuntimeSection,
}

/// Node-specific configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSection {
    /// Data directory for storage
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Network configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkSection {
    /// Port to listen on
    #[serde(default = "default_port")]
    pub listen_port: u16,

    /// Maximum number of peers
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,

    /// Bootstrap peers
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,

    /// Enable peer discovery
    #[serde(default = "default_true")]
    pub enable_discovery: bool,
}

/// Runtime configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeSection {
    /// Chain ID
    #[serde(default)]
    pub chain_id: String,

    /// Block producer mode
    #[serde(default)]
    pub producer_enabled: bool,

    /// Producer's private key (hex encoded, 32 bytes)
    #[serde(default)]
    pub producer_key: Option<String>,
}

// Default value functions
fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_port() -> u16 {
    30303
}

fn default_max_peers() -> usize {
    50
}

fn default_true() -> bool {
    true
}

impl Default for NodeSection {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            log_level: default_log_level(),
        }
    }
}

impl Default for NetworkSection {
    fn default() -> Self {
        Self {
            listen_port: default_port(),
            max_peers: default_max_peers(),
            bootstrap_peers: Vec::new(),
            enable_discovery: true,
        }
    }
}

impl Default for RuntimeSection {
    fn default() -> Self {
        Self {
            chain_id: "unykorn-devnet".to_string(),
            producer_enabled: false,
            producer_key: None,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node: NodeSection::default(),
            network: NetworkSection::default(),
            runtime: RuntimeSection::default(),
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Create a development configuration.
    pub fn dev() -> Self {
        Self {
            node: NodeSection {
                data_dir: PathBuf::from("./dev_data"),
                log_level: "debug".to_string(),
            },
            network: NetworkSection {
                listen_port: 30303,
                max_peers: 10,
                bootstrap_peers: Vec::new(),
                enable_discovery: false,
            },
            runtime: RuntimeSection {
                chain_id: "unykorn-dev".to_string(),
                producer_enabled: true,
                producer_key: Some("0".repeat(64)), // Dev key
            },
        }
    }

    /// Get the listen address.
    pub fn listen_addr(&self) -> SocketAddr {
        format!("0.0.0.0:{}", self.network.listen_port)
            .parse()
            .unwrap()
    }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.network.listen_port, 30303);
        assert_eq!(config.network.max_peers, 50);
    }

    #[test]
    fn test_dev_config() {
        let config = NodeConfig::dev();
        assert!(config.runtime.producer_enabled);
        assert_eq!(config.network.max_peers, 10);
    }

    #[test]
    fn test_serialize_config() {
        let config = NodeConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("listen_port"));
    }
}
