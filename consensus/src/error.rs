//! Consensus error types.
//!
//! Explicit error handling for all consensus failures.

use thiserror::Error;

/// Errors that can occur during consensus operations.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Proposal is for wrong round.
    #[error("proposal round {got} does not match expected {expected}")]
    WrongRound { expected: u64, got: u64 },

    /// Proposal is for wrong height.
    #[error("proposal height {got} does not match expected {expected}")]
    WrongHeight { expected: u64, got: u64 },

    /// Proposal is from wrong leader.
    #[error("proposal from {got} but expected leader {expected}")]
    WrongLeader { expected: String, got: String },

    /// Duplicate vote from same validator.
    #[error("duplicate vote from validator {validator} in round {round}")]
    DuplicateVote { validator: String, round: u64 },

    /// Vote is for unknown block.
    #[error("vote references unknown block hash {hash}")]
    UnknownBlock { hash: String },

    /// Validator not in validator set.
    #[error("unknown validator: {validator}")]
    UnknownValidator { validator: String },

    /// Invalid signature on consensus message.
    #[error("invalid signature on {message_type}")]
    InvalidSignature { message_type: String },

    /// Quorum not reached within timeout.
    #[error("quorum timeout in round {round} phase {phase}")]
    QuorumTimeout { round: u64, phase: String },

    /// Block validation failed.
    #[error("block validation failed: {reason}")]
    InvalidBlock { reason: String },

    /// Attempted to finalize already finalized height.
    #[error("height {height} already finalized")]
    AlreadyFinalized { height: u64 },

    /// Fork detected after finality (critical invariant violation).
    #[error("CRITICAL: fork detected after finality at height {height}")]
    ForkAfterFinality { height: u64 },

    /// Internal state corruption.
    #[error("internal consensus state corruption: {details}")]
    StateCorruption { details: String },

    /// Timeout expired.
    #[error("timeout expired for {operation}")]
    Timeout { operation: String },

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Result type alias for consensus operations.
pub type Result<T> = std::result::Result<T, ConsensusError>;
