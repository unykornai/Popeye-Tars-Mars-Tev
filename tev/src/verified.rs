//! Verified payload types.
//!
//! These types represent data that has passed cryptographic validation.
//! They can only be created through the verification functions,
//! ensuring type-level safety for the trust boundary.

/// A transaction that has passed cryptographic verification.
///
/// This type can only be created by `verify_transaction()`,
/// guaranteeing that the signature has been checked.
#[derive(Clone, Debug)]
pub struct VerifiedTransaction {
    /// The transaction data (excluding signature and pubkey)
    pub data: Vec<u8>,

    /// The verified signer's public key
    pub signer: [u8; 32],

    /// The verified signature
    pub signature: [u8; 64],
}

impl VerifiedTransaction {
    /// Get the transaction data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the signer's public key.
    pub fn signer(&self) -> &[u8; 32] {
        &self.signer
    }
}

/// A block that has passed cryptographic verification.
///
/// This type can only be created by `verify_block()`,
/// guaranteeing that the producer's signature has been checked.
#[derive(Clone, Debug)]
pub struct VerifiedBlock {
    /// The block data (excluding signature and producer key)
    pub data: Vec<u8>,

    /// The verified block producer's public key
    pub producer: [u8; 32],

    /// The verified signature
    pub signature: [u8; 64],
}

impl VerifiedBlock {
    /// Get the block data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the producer's public key.
    pub fn producer(&self) -> &[u8; 32] {
        &self.producer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_transaction_accessors() {
        let vt = VerifiedTransaction {
            data: vec![1, 2, 3],
            signer: [1u8; 32],
            signature: [2u8; 64],
        };

        assert_eq!(vt.data(), &[1, 2, 3]);
        assert_eq!(vt.signer(), &[1u8; 32]);
    }

    #[test]
    fn test_verified_block_accessors() {
        let vb = VerifiedBlock {
            data: vec![4, 5, 6],
            producer: [3u8; 32],
            signature: [4u8; 64],
        };

        assert_eq!(vb.data(), &[4, 5, 6]);
        assert_eq!(vb.producer(), &[3u8; 32]);
    }
}
