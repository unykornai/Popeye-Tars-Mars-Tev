//! Blockchain state representation.
//!
//! The `State` struct represents the canonical state of the blockchain.
//! It is designed to be:
//! - Serializable (for persistence via TAR)
//! - Deterministic (same operations always produce same state)
//! - Clone-friendly (for state snapshots)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The canonical blockchain state.
///
/// # Invariants
///
/// - `height` is monotonically increasing
/// - `state_root` is derived deterministically from state data
/// - All accounts have non-negative balances
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    /// Current block height
    pub height: u64,

    /// State root hash (computed after each block)
    pub state_root: [u8; 32],

    /// Account balances (address -> balance)
    pub balances: HashMap<[u8; 32], u64>,

    /// Account nonces for replay protection
    pub nonces: HashMap<[u8; 32], u64>,
}

impl State {
    /// Create a new genesis state.
    pub fn new() -> Self {
        Self {
            height: 0,
            state_root: [0u8; 32],
            balances: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    /// Get the balance for an address.
    pub fn balance(&self, address: &[u8; 32]) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Get the nonce for an address.
    pub fn nonce(&self, address: &[u8; 32]) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    /// Increment the nonce for an address.
    pub fn increment_nonce(&mut self, address: &[u8; 32]) {
        let current = self.nonce(address);
        self.nonces.insert(*address, current + 1);
    }

    /// Set the balance for an address.
    pub fn set_balance(&mut self, address: &[u8; 32], balance: u64) {
        self.balances.insert(*address, balance);
    }

    /// Compute and update the state root.
    /// This is a placeholder - real implementation would use Merkle tree.
    pub fn compute_state_root(&mut self) {
        // Simple hash of height for now - replace with proper Merkle root
        let mut root = [0u8; 32];
        root[0..8].copy_from_slice(&self.height.to_le_bytes());
        self.state_root = root;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = State::new();
        assert_eq!(state.height, 0);
        assert_eq!(state.state_root, [0u8; 32]);
    }

    #[test]
    fn test_balance_operations() {
        let mut state = State::new();
        let addr = [1u8; 32];

        assert_eq!(state.balance(&addr), 0);
        state.set_balance(&addr, 1000);
        assert_eq!(state.balance(&addr), 1000);
    }

    #[test]
    fn test_nonce_operations() {
        let mut state = State::new();
        let addr = [1u8; 32];

        assert_eq!(state.nonce(&addr), 0);
        state.increment_nonce(&addr);
        assert_eq!(state.nonce(&addr), 1);
        state.increment_nonce(&addr);
        assert_eq!(state.nonce(&addr), 2);
    }
}
