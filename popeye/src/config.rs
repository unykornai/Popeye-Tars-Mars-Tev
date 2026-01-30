//! Network configuration.

use std::net::SocketAddr;

/// Configuration for the network layer.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// Address to listen on
    pub listen_addr: SocketAddr,

    /// Maximum number of peers
    pub max_peers: usize,

    /// Enable peer discovery
    pub enable_discovery: bool,

    /// Chain ID for network isolation
    pub chain_id: [u8; 32],

    /// Node identity (public key)
    pub node_id: [u8; 32],

    /// Bootstrap peers to connect to
    pub bootstrap_peers: Vec<SocketAddr>,
}

impl NetworkConfig {
    /// Create a new config with default values.
    pub fn new(listen_addr: SocketAddr, node_id: [u8; 32]) -> Self {
        Self {
            listen_addr,
            max_peers: 50,
            enable_discovery: true,
            chain_id: [0u8; 32],
            node_id,
            bootstrap_peers: Vec::new(),
        }
    }

    /// Create a config for local development.
    pub fn local(port: u16, node_id: [u8; 32]) -> Self {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        Self::new(addr, node_id)
    }

    /// Set the chain ID.
    pub fn with_chain_id(mut self, chain_id: [u8; 32]) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Set maximum peers.
    pub fn with_max_peers(mut self, max: usize) -> Self {
        self.max_peers = max;
        self
    }

    /// Add bootstrap peers.
    pub fn with_bootstrap_peers(mut self, peers: Vec<SocketAddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            max_peers: 50,
            enable_discovery: true,
            chain_id: [0u8; 32],
            node_id: [0u8; 32],
            bootstrap_peers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_config() {
        let config = NetworkConfig::local(8080, [1u8; 32]);
        assert_eq!(config.listen_addr.port(), 8080);
    }

    #[test]
    fn test_builder_pattern() {
        let config = NetworkConfig::local(8080, [1u8; 32])
            .with_chain_id([2u8; 32])
            .with_max_peers(100);

        assert_eq!(config.chain_id, [2u8; 32]);
        assert_eq!(config.max_peers, 100);
    }
}
