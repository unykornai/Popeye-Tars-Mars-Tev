//! # TAR — Transaction & Archive Repository
//!
//! TAR is the persistence layer for Unykorn L1.
//! It handles all disk operations with crash-safe guarantees.
//!
//! ## Design Principles
//!
//! - **Crash-safe**: Atomic writes prevent corruption
//! - **Append-only**: Blocks are immutable once written
//! - **Deterministic**: Same data on every restart
//!
//! ## Trust Model
//!
//! TAR only stores data that has passed through TEV and MARS.
//! It never validates - it only remembers.
//!
//! ## Consensus Support
//!
//! TAR also persists consensus artifacts for crash recovery:
//! - Round state (height, round, phase)
//! - Finality certificates
//! - Validator sets

pub mod error;
pub mod storage;
pub mod block_store;
pub mod state_store;
pub mod consensus_store;

pub use error::StorageError;
pub use storage::Storage;
pub use consensus_store::ConsensusStore;
