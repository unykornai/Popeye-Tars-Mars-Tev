//! Storage error types.

use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Error, Debug)]
pub enum StorageError {
    /// I/O operation failed
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failed
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Binary serialization/deserialization failed
    #[error("bincode error: {reason}")]
    Bincode { reason: String },

    /// Data not found
    #[error("not found: {key}")]
    NotFound { key: String },

    /// Data corruption detected
    #[error("data corruption: {reason}")]
    Corruption { reason: String },

    /// Block height mismatch
    #[error("height mismatch: expected {expected}, got {got}")]
    HeightMismatch { expected: u64, got: u64 },
}
