//! Block types and validation.
//!
//! Blocks are ordered collections of transactions at a specific height.
//! They form the immutable chain of state transitions.

use crate::tx::Transaction;
use serde::{Deserialize, Serialize};

/// A blockchain block.
///
/// # Invariants
///
/// - `height` must be exactly parent height + 1
/// - `parent_hash` must match the hash of the previous block
/// - `transactions` must be ordered and valid
/// - `state_root` must match the state after applying all transactions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// Block height (0 = genesis)
    pub height: u64,

    /// Hash of the parent block
    pub parent_hash: [u8; 32],

    /// State root after applying this block
    pub state_root: [u8; 32],

    /// Block timestamp (Unix epoch seconds)
    pub timestamp: u64,

    /// Transactions in this block
    pub txs: Vec<Transaction>,

    /// Block producer's public key
    pub producer: [u8; 32],

    /// Block signature (64 bytes as Vec for serde compatibility)
    pub signature: Vec<u8>,
}

impl Block {
    /// Create a new block.
    pub fn new(
        height: u64,
        parent_hash: [u8; 32],
        state_root: [u8; 32],
        txs: Vec<Transaction>,
        producer: [u8; 32],
    ) -> Self {
        Self {
            height,
            parent_hash,
            state_root,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            txs,
            producer,
            signature: vec![0u8; 64],
        }
    }

    /// Create the genesis block.
    pub fn genesis() -> Self {
        Self {
            height: 0,
            parent_hash: [0u8; 32],
            state_root: [0u8; 32],
            timestamp: 0,
            txs: Vec::new(),
            producer: [0u8; 32],
            signature: vec![0u8; 64],
        }
    }

    /// Get the bytes to be signed.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.parent_hash);
        bytes.extend_from_slice(&self.state_root);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&(self.txs.len() as u64).to_le_bytes());
        for tx in &self.txs {
            bytes.extend_from_slice(&tx.signing_bytes());
        }
        bytes.extend_from_slice(&self.producer);
        bytes
    }

    /// Compute block hash (simplified - use proper hash in production).
    pub fn hash(&self) -> [u8; 32] {
        let bytes = self.signing_bytes();
        let mut hash = [0u8; 32];
        // Simple hash for now - replace with proper crypto hash
        for (i, byte) in bytes.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        hash
    }

    /// Set the signature for this block.
    pub fn set_signature(&mut self, sig: [u8; 64]) {
        self.signature = sig.to_vec();
    }

    /// Check if this is the genesis block.
    pub fn is_genesis(&self) -> bool {
        self.height == 0
    }

    /// Get the number of transactions in this block.
    pub fn tx_count(&self) -> usize {
        self.txs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.height, 0);
        assert!(genesis.is_genesis());
        assert_eq!(genesis.parent_hash, [0u8; 32]);
    }

    #[test]
    fn test_block_hash_deterministic() {
        let block1 = Block::genesis();
        let block2 = Block::genesis();
        assert_eq!(block1.hash(), block2.hash());
    }

    #[test]
    fn test_block_with_transactions() {
        let tx = Transaction::new([1u8; 32], [2u8; 32], 100, 0);
        let block = Block::new(1, [0u8; 32], [0u8; 32], vec![tx], [3u8; 32]);

        assert_eq!(block.height, 1);
        assert_eq!(block.tx_count(), 1);
        assert!(!block.is_genesis());
    }
}
