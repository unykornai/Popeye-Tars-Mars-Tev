//! The MARS runtime - core execution engine.
//!
//! The Runtime is the heart of the blockchain, responsible for:
//! - Validating transactions
//! - Producing blocks
//! - Applying state transitions
//!
//! # Design Principles
//!
//! - All operations are deterministic
//! - No networking or disk IO
//! - Pure functions for state transitions

use crate::{Block, RuntimeError, State, Transaction};

/// The core runtime execution engine.
///
/// # Usage
///
/// ```rust
/// use mars::Runtime;
///
/// let mut runtime = Runtime::new();
/// // Submit transactions, produce blocks, etc.
/// ```
pub struct Runtime {
    /// Current blockchain state
    pub state: State,

    /// Pending transactions (mempool)
    mempool: Vec<Transaction>,

    /// Last finalized block hash
    last_block_hash: [u8; 32],
}

impl Runtime {
    /// Create a new runtime with genesis state.
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Self {
            state: State::new(),
            mempool: Vec::new(),
            last_block_hash: genesis.hash(),
        }
    }

    /// Create a runtime with existing state (for restart recovery).
    pub fn with_state(state: State, last_block_hash: [u8; 32]) -> Self {
        Self {
            state,
            mempool: Vec::new(),
            last_block_hash,
        }
    }

    /// Submit a transaction to the mempool.
    ///
    /// Returns an error if the transaction is invalid.
    pub fn submit_transaction(&mut self, tx: Transaction) -> Result<(), RuntimeError> {
        self.validate_transaction(&tx)?;
        self.mempool.push(tx);
        Ok(())
    }

    /// Validate a transaction against current state.
    ///
    /// # Checks
    ///
    /// - Sender has sufficient balance
    /// - Nonce matches expected value (accounting for pending mempool txs)
    /// - Amount is non-zero
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), RuntimeError> {
        // Count pending transactions from the same sender in mempool
        let pending_count = self.mempool.iter()
            .filter(|t| t.from == tx.from)
            .count() as u64;

        // Check nonce (account for pending transactions)
        let expected_nonce = self.state.nonce(&tx.from) + pending_count;
        if tx.nonce != expected_nonce {
            return Err(RuntimeError::DuplicateNonce { nonce: tx.nonce });
        }

        // Calculate pending outgoing amount
        let pending_amount: u64 = self.mempool.iter()
            .filter(|t| t.from == tx.from)
            .map(|t| t.amount)
            .sum();

        // Check balance (account for pending transactions)
        let balance = self.state.balance(&tx.from);
        let available = balance.saturating_sub(pending_amount);
        if available < tx.amount {
            return Err(RuntimeError::InvalidTransaction {
                reason: format!(
                    "insufficient balance: have {}, need {}",
                    available, tx.amount
                ),
            });
        }

        Ok(())
    }

    /// Apply a single transaction to state.
    ///
    /// This is a pure function - same inputs always produce same outputs.
    fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), RuntimeError> {
        // Debit sender
        let sender_balance = self.state.balance(&tx.from);
        self.state.set_balance(&tx.from, sender_balance - tx.amount);

        // Credit recipient
        let recipient_balance = self.state.balance(&tx.to);
        self.state.set_balance(&tx.to, recipient_balance + tx.amount);

        // Increment sender nonce
        self.state.increment_nonce(&tx.from);

        Ok(())
    }

    /// Produce a new block from pending transactions.
    ///
    /// This drains the mempool and creates a block at the next height.
    pub fn produce_block(&mut self, producer: [u8; 32]) -> Block {
        // Take all mempool transactions
        let txs: Vec<Transaction> = self.mempool.drain(..).collect();

        // Apply all transactions
        for tx in &txs {
            // Transactions were already validated on submission
            let _ = self.apply_transaction(tx);
        }

        // Update state
        self.state.height += 1;
        self.state.compute_state_root();

        // Create block
        let block = Block::new(
            self.state.height,
            self.last_block_hash,
            self.state.state_root,
            txs,
            producer,
        );

        self.last_block_hash = block.hash();
        block
    }

    /// Validate a block from the network.
    ///
    /// # Checks
    ///
    /// - Height is exactly current + 1
    /// - Parent hash matches
    /// - All transactions are valid
    pub fn validate_block(&self, block: &Block) -> Result<(), RuntimeError> {
        // Check height
        let expected_height = self.state.height + 1;
        if block.height != expected_height {
            return Err(RuntimeError::HeightMismatch {
                expected: expected_height,
                got: block.height,
            });
        }

        // Check parent hash
        if block.parent_hash != self.last_block_hash {
            return Err(RuntimeError::InvalidBlock {
                reason: "parent hash mismatch".to_string(),
            });
        }

        // Validate all transactions
        for tx in &block.txs {
            self.validate_transaction(tx)?;
        }

        Ok(())
    }

    /// Apply a validated block to state.
    ///
    /// Call `validate_block` first!
    pub fn apply_block(&mut self, block: &Block) -> Result<(), RuntimeError> {
        // Apply all transactions
        for tx in &block.txs {
            self.apply_transaction(tx)?;
        }

        // Update state
        self.state.height = block.height;
        self.state.state_root = block.state_root;
        self.last_block_hash = block.hash();

        Ok(())
    }

    /// Get current block height.
    pub fn height(&self) -> u64 {
        self.state.height
    }

    /// Get number of pending transactions.
    pub fn mempool_size(&self) -> usize {
        self.mempool.len()
    }

    /// Get the last block hash.
    pub fn last_block_hash(&self) -> [u8; 32] {
        self.last_block_hash
    }

    /// Clear the mempool.
    pub fn clear_mempool(&mut self) {
        self.mempool.clear();
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn funded_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        // Fund an account for testing
        runtime.state.set_balance(&[1u8; 32], 1000);
        runtime
    }

    #[test]
    fn test_new_runtime() {
        let runtime = Runtime::new();
        assert_eq!(runtime.height(), 0);
        assert_eq!(runtime.mempool_size(), 0);
    }

    #[test]
    fn test_submit_valid_transaction() {
        let mut runtime = funded_runtime();
        let tx = Transaction::new([1u8; 32], [2u8; 32], 100, 0);

        assert!(runtime.submit_transaction(tx).is_ok());
        assert_eq!(runtime.mempool_size(), 1);
    }

    #[test]
    fn test_reject_insufficient_balance() {
        let mut runtime = Runtime::new();
        let tx = Transaction::new([1u8; 32], [2u8; 32], 100, 0);

        let result = runtime.submit_transaction(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_produce_block() {
        let mut runtime = funded_runtime();
        let tx = Transaction::new([1u8; 32], [2u8; 32], 100, 0);
        runtime.submit_transaction(tx).unwrap();

        let block = runtime.produce_block([3u8; 32]);

        assert_eq!(block.height, 1);
        assert_eq!(block.tx_count(), 1);
        assert_eq!(runtime.height(), 1);
        assert_eq!(runtime.mempool_size(), 0);
    }

    #[test]
    fn test_state_transition() {
        let mut runtime = funded_runtime();
        let sender = [1u8; 32];
        let recipient = [2u8; 32];

        let tx = Transaction::new(sender, recipient, 100, 0);
        runtime.submit_transaction(tx).unwrap();
        runtime.produce_block([3u8; 32]);

        assert_eq!(runtime.state.balance(&sender), 900);
        assert_eq!(runtime.state.balance(&recipient), 100);
        assert_eq!(runtime.state.nonce(&sender), 1);
    }

    #[test]
    fn test_nonce_enforcement() {
        let mut runtime = funded_runtime();
        let sender = [1u8; 32];

        // First tx with nonce 0 should succeed
        let tx1 = Transaction::new(sender, [2u8; 32], 100, 0);
        assert!(runtime.submit_transaction(tx1).is_ok());

        // Second tx with nonce 0 should fail (duplicate)
        let tx2 = Transaction::new(sender, [2u8; 32], 100, 0);
        assert!(runtime.submit_transaction(tx2).is_err());
    }
}
