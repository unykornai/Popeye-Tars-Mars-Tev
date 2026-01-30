//! Node orchestration.
//!
//! Wires together MARS, POPEYE, TEV, and TAR into a running node.

use crate::NodeConfig;
use mars::Runtime;
use popeye::{Network, NetworkConfig, NetworkMessage};
use popeye::message::NetworkEvent;
use tar::Storage;
use tev::{verify_block, verify_transaction};
use tokio::sync::mpsc;

/// The integrated node.
pub struct Node {
    /// Configuration
    config: NodeConfig,

    /// Runtime (MARS)
    runtime: Runtime,

    /// Storage (TAR)
    storage: Storage,

    /// Network (POPEYE)
    network: Network,

    /// Network event receiver
    network_rx: mpsc::Receiver<NetworkEvent>,

    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl Node {
    /// Create a new node from configuration.
    pub fn new(config: NodeConfig) -> Result<Self, NodeError> {
        // Initialize storage (TAR)
        let storage = Storage::new(config.node.data_dir.clone())
            .map_err(|e| NodeError::StorageInit(e.to_string()))?;

        // Initialize runtime (MARS)
        let runtime = if storage.has_state() {
            // Recover from disk
            let state = storage.load_state()
                .map_err(|e| NodeError::StorageInit(e.to_string()))?;
            let last_height = storage.latest_block_height()
                .map_err(|e| NodeError::StorageInit(e.to_string()))?
                .unwrap_or(0);
            
            // Load last block hash
            let last_hash = if last_height > 0 {
                let block: mars::Block = storage.load_block(last_height)
                    .map_err(|e| NodeError::StorageInit(e.to_string()))?;
                block.hash()
            } else {
                mars::Block::genesis().hash()
            };
            
            Runtime::with_state(state, last_hash)
        } else {
            Runtime::new()
        };

        // Initialize network (POPEYE)
        let node_id = Self::derive_node_id(&config);
        let network_config = NetworkConfig::new(config.listen_addr(), node_id)
            .with_max_peers(config.network.max_peers);
        
        let (network, network_rx) = Network::new(network_config);

        Ok(Self {
            config,
            runtime,
            storage,
            network,
            network_rx,
            shutdown_tx: None,
        })
    }

    /// Derive node ID from config (or generate one).
    fn derive_node_id(config: &NodeConfig) -> [u8; 32] {
        if let Some(ref key) = config.runtime.producer_key {
            // Use producer key as node ID (simplified)
            let mut id = [0u8; 32];
            let bytes = key.as_bytes();
            for (i, &b) in bytes.iter().take(32).enumerate() {
                id[i] = b;
            }
            id
        } else {
            // Generate a random ID
            [0u8; 32]
        }
    }

    /// Run the node.
    pub async fn run(&mut self) -> Result<(), NodeError> {
        println!("Starting Unykorn L1 node...");
        println!("  Data dir: {:?}", self.config.node.data_dir);
        println!("  Listen: {}", self.config.listen_addr());
        println!("  Height: {}", self.runtime.height());
        println!("  Producer: {}", self.config.runtime.producer_enabled);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Block production interval (3 seconds for devnet)
        let mut block_interval = tokio::time::interval(tokio::time::Duration::from_secs(3));

        loop {
            tokio::select! {
                // Handle network events
                Some(event) = self.network_rx.recv() => {
                    if let Err(e) = self.handle_network_event(event).await {
                        eprintln!("Error handling network event: {}", e);
                    }
                }

                // Block production (if producer)
                _ = block_interval.tick(), if self.config.runtime.producer_enabled => {
                    match self.produce_block() {
                        Ok(block) => {
                            // Broadcast block to peers
                            let msg = popeye::message::BlockMessage::new(
                                bincode::serialize(&block).unwrap_or_default(),
                                block.height,
                            );
                            let _ = self.network.broadcast(NetworkMessage::Block(msg)).await;
                        }
                        Err(e) => {
                            eprintln!("Block production error: {}", e);
                        }
                    }
                }

                // Handle shutdown
                _ = shutdown_rx.recv() => {
                    println!("Shutting down...");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a network event.
    async fn handle_network_event(&mut self, event: NetworkEvent) -> Result<(), NodeError> {
        match event {
            NetworkEvent::MessageReceived { from: _, message } => {
                self.handle_message(message).await?;
            }
            NetworkEvent::PeerConnected { peer_id } => {
                println!("Peer connected: {:02x}{:02x}...", peer_id[0], peer_id[1]);
            }
            NetworkEvent::PeerDisconnected { peer_id } => {
                println!("Peer disconnected: {:02x}{:02x}...", peer_id[0], peer_id[1]);
            }
        }
        Ok(())
    }

    /// Handle an incoming message.
    async fn handle_message(&mut self, message: NetworkMessage) -> Result<(), NodeError> {
        match message {
            NetworkMessage::Transaction(tx_msg) => {
                self.handle_transaction(tx_msg.payload).await?;
            }
            NetworkMessage::Block(block_msg) => {
                self.handle_block(block_msg.payload).await?;
            }
            NetworkMessage::Ping(n) => {
                // Respond with pong
                let _ = self.network.broadcast(NetworkMessage::Pong(n)).await;
            }
            NetworkMessage::Pong(_) => {
                // Ignore pongs
            }
            NetworkMessage::Handshake(_) => {
                // Handle handshake
            }
        }
        Ok(())
    }

    /// Handle an incoming transaction.
    ///
    /// Flow: POPEYE → TEV → MARS → (broadcast)
    async fn handle_transaction(&mut self, payload: Vec<u8>) -> Result<(), NodeError> {
        // TEV: Verify signature
        let verified = verify_transaction(&payload)
            .map_err(|e| NodeError::ValidationFailed(e.to_string()))?;

        // MARS: Parse and validate
        let tx: mars::Transaction = bincode::deserialize(verified.data())
            .map_err(|_| NodeError::InvalidPayload)?;

        // MARS: Submit to runtime
        self.runtime.submit_transaction(tx)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        // Broadcast to peers
        let msg = popeye::message::TransactionMessage::new(payload);
        let _ = self.network.broadcast(NetworkMessage::Transaction(msg)).await;

        Ok(())
    }

    /// Handle an incoming block.
    ///
    /// Flow: POPEYE → TEV → MARS → TAR
    async fn handle_block(&mut self, payload: Vec<u8>) -> Result<(), NodeError> {
        // TEV: Verify signature
        let verified = verify_block(&payload)
            .map_err(|e| NodeError::ValidationFailed(e.to_string()))?;

        // MARS: Parse and validate
        let block: mars::Block = bincode::deserialize(verified.data())
            .map_err(|_| NodeError::InvalidPayload)?;

        // MARS: Validate block
        self.runtime.validate_block(&block)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        // MARS: Apply block
        self.runtime.apply_block(&block)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        // TAR: Persist
        self.storage.commit(block.height, &block, &self.runtime.state)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("Applied block #{}", block.height);

        // Broadcast to peers
        let msg = popeye::message::BlockMessage::new(payload, block.height);
        let _ = self.network.broadcast(NetworkMessage::Block(msg)).await;

        Ok(())
    }

    /// Produce a block (for block producers).
    pub fn produce_block(&mut self) -> Result<mars::Block, NodeError> {
        let producer_key = self.config.runtime.producer_key
            .as_ref()
            .ok_or(NodeError::NotProducer)?;

        // Parse producer key
        let mut key = [0u8; 32];
        let bytes = producer_key.as_bytes();
        for (i, &b) in bytes.iter().take(32).enumerate() {
            key[i] = b;
        }

        // MARS: Produce block
        let block = self.runtime.produce_block(key);

        // TAR: Persist
        self.storage.commit(block.height, &block, &self.runtime.state)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("Produced block #{}", block.height);

        Ok(block)
    }

    /// Get current block height.
    pub fn height(&self) -> u64 {
        self.runtime.height()
    }

    /// Get mempool size.
    pub fn mempool_size(&self) -> usize {
        self.runtime.mempool_size()
    }

    /// Get peer count.
    pub fn peer_count(&self) -> usize {
        self.network.peer_count()
    }

    /// Shutdown the node.
    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

/// Node errors.
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("storage initialization failed: {0}")]
    StorageInit(String),

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("invalid payload")]
    InvalidPayload,

    #[error("runtime error: {0}")]
    RuntimeError(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("not configured as block producer")]
    NotProducer,

    #[error("network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_node_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = NodeConfig::dev();
        config.node.data_dir = temp_dir.path().to_path_buf();

        let node = Node::new(config).unwrap();
        assert_eq!(node.height(), 0);
    }

    #[test]
    fn test_block_production() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = NodeConfig::dev();
        config.node.data_dir = temp_dir.path().to_path_buf();
        config.runtime.producer_enabled = true;
        config.runtime.producer_key = Some("a".repeat(64));

        let mut node = Node::new(config).unwrap();
        let block = node.produce_block().unwrap();

        assert_eq!(block.height, 1);
        assert_eq!(node.height(), 1);
    }
}
