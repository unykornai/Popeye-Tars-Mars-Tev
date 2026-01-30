//! Transaction types and validation.
//!
//! Transactions are the atomic units of state mutation.
//! They must be signed and verified by TEV before reaching MARS.

use serde::{Deserialize, Serialize};

/// A blockchain transaction.
///
/// # Fields
///
/// - `from`: Sender's public key (32 bytes)
/// - `to`: Recipient's address (32 bytes)
/// - `amount`: Amount to transfer
/// - `nonce`: Replay protection counter
/// - `payload`: Optional data payload
/// - `signature`: Ed25519 signature (verified by TEV)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Sender's public key
    pub from: [u8; 32],

    /// Recipient's address
    pub to: [u8; 32],

    /// Amount to transfer
    pub amount: u64,

    /// Sender's nonce (for replay protection)
    pub nonce: u64,

    /// Optional payload data
    pub payload: Vec<u8>,

    /// Ed25519 signature (64 bytes as Vec for serde compatibility)
    pub signature: Vec<u8>,
}

impl Transaction {
    /// Create a new unsigned transaction.
    pub fn new(from: [u8; 32], to: [u8; 32], amount: u64, nonce: u64) -> Self {
        Self {
            from,
            to,
            amount,
            nonce,
            payload: Vec::new(),
            signature: vec![0u8; 64],
        }
    }

    /// Create a new transaction with payload.
    pub fn with_payload(
        from: [u8; 32],
        to: [u8; 32],
        amount: u64,
        nonce: u64,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            from,
            to,
            amount,
            nonce,
            payload,
            signature: vec![0u8; 64],
        }
    }

    /// Get the bytes to be signed.
    /// This is the canonical serialization for signature verification.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.from);
        bytes.extend_from_slice(&self.to);
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Set the signature for this transaction.
    pub fn set_signature(&mut self, sig: [u8; 64]) {
        self.signature = sig.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let from = [1u8; 32];
        let to = [2u8; 32];
        let tx = Transaction::new(from, to, 100, 0);

        assert_eq!(tx.from, from);
        assert_eq!(tx.to, to);
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.nonce, 0);
        assert!(tx.payload.is_empty());
    }

    #[test]
    fn test_signing_bytes_deterministic() {
        let tx1 = Transaction::new([1u8; 32], [2u8; 32], 100, 0);
        let tx2 = Transaction::new([1u8; 32], [2u8; 32], 100, 0);

        assert_eq!(tx1.signing_bytes(), tx2.signing_bytes());
    }
}
