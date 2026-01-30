//! Error types for the MARS runtime.
//!
//! All errors are explicit and typed using thiserror.

use thiserror::Error;

/// Errors that can occur during runtime execution.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Transaction validation failed
    #[error("invalid transaction: {reason}")]
    InvalidTransaction { reason: String },

    /// Block validation failed
    #[error("invalid block: {reason}")]
    InvalidBlock { reason: String },

    /// State transition failed
    #[error("state transition failed: {reason}")]
    StateTransitionFailed { reason: String },

    /// Block height mismatch
    #[error("block height mismatch: expected {expected}, got {got}")]
    HeightMismatch { expected: u64, got: u64 },

    /// Duplicate transaction detected
    #[error("duplicate transaction: nonce {nonce} already used")]
    DuplicateNonce { nonce: u64 },
}
