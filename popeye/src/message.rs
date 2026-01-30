//! Network message types.
//!
//! These messages flow between peers and are forwarded to the runtime.

use serde::{Deserialize, Serialize};

/// Messages that can be sent/received over the network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// A transaction to be propagated
    Transaction(TransactionMessage),

    /// A block to be propagated
    Block(BlockMessage),

    /// Peer handshake
    Handshake(HandshakeMessage),

    /// Ping for liveness
    Ping(u64),

    /// Pong response
    Pong(u64),
}

/// Transaction propagation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMessage {
    /// Raw transaction bytes (includes signature)
    pub payload: Vec<u8>,

    /// Timestamp when first seen
    pub timestamp: u64,
}

impl TransactionMessage {
    /// Create a new transaction message.
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Block propagation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMessage {
    /// Raw block bytes (includes signature)
    pub payload: Vec<u8>,

    /// Block height
    pub height: u64,
}

impl BlockMessage {
    /// Create a new block message.
    pub fn new(payload: Vec<u8>, height: u64) -> Self {
        Self { payload, height }
    }
}

/// Peer handshake message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Protocol version
    pub version: u32,

    /// Chain ID
    pub chain_id: [u8; 32],

    /// Current block height
    pub height: u64,

    /// Node's public identity
    pub node_id: [u8; 32],
}

impl HandshakeMessage {
    /// Create a new handshake message.
    pub fn new(chain_id: [u8; 32], height: u64, node_id: [u8; 32]) -> Self {
        Self {
            version: 1,
            chain_id,
            height,
            node_id,
        }
    }
}

/// Internal event for the network service.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// Received a message from a peer
    MessageReceived {
        from: [u8; 32],
        message: NetworkMessage,
    },

    /// New peer connected
    PeerConnected { peer_id: [u8; 32] },

    /// Peer disconnected
    PeerDisconnected { peer_id: [u8; 32] },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_message() {
        let payload = vec![1, 2, 3, 4];
        let msg = TransactionMessage::new(payload.clone());

        assert_eq!(msg.payload, payload);
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_block_message() {
        let msg = BlockMessage::new(vec![1, 2, 3], 10);

        assert_eq!(msg.height, 10);
        assert_eq!(msg.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_handshake_message() {
        let chain_id = [1u8; 32];
        let node_id = [2u8; 32];
        let msg = HandshakeMessage::new(chain_id, 100, node_id);

        assert_eq!(msg.version, 1);
        assert_eq!(msg.chain_id, chain_id);
        assert_eq!(msg.height, 100);
    }
}
