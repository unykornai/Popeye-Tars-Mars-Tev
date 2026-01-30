//! Error types for TEV validation.

use thiserror::Error;

/// Errors that can occur during cryptographic validation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Payload format is invalid
    #[error("invalid format: {reason}")]
    InvalidFormat { reason: String },

    /// Signature verification failed
    #[error("invalid signature")]
    InvalidSignature,

    /// Public key is malformed
    #[error("invalid public key")]
    InvalidPublicKey,

    /// Replay attack detected
    #[error("replay attack: nonce {nonce} already used")]
    ReplayDetected { nonce: u64 },
}
