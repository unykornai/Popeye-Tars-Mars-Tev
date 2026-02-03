//! Consensus crate for Unykorn L1.
//!
//! Implements a deterministic, round-based BFT consensus protocol.
//!
//! # Architecture Position
//!
//! ```text
//! POPEYE → TEV → CONSENSUS → MARS → TAR
//!                    ↑
//!              (this crate)
//! ```
//!
//! # Design Principles
//!
//! 1. **Consensus never mutates state** — MARS is the sole state authority
//! 2. **Consensus never validates signatures** — TEV does all crypto
//! 3. **Consensus never touches the network** — POPEYE handles transport
//! 4. **Consensus never persists data** — TAR handles storage
//!
//! Consensus is a thin coordinator that decides WHICH block becomes canonical.
//!
//! # Protocol Overview
//!
//! Each consensus round has three phases:
//!
//! 1. **Propose** — Deterministic leader broadcasts block proposal
//! 2. **Prevote** — Validators vote on proposal validity
//! 3. **Commit** — Validators commit to finalize
//!
//! Finality is achieved when ≥ 2/3 of validators commit to the same block.
//!
//! # Usage
//!
//! ```ignore
//! use consensus::{ConsensusEngine, ValidatorSet, ConsensusConfig};
//!
//! let (event_tx, event_rx) = mpsc::unbounded_channel();
//! let engine = ConsensusEngine::new(config, validators, keypair, event_tx);
//!
//! // Process incoming messages
//! engine.on_proposal(proposal).await?;
//! engine.on_prevote(prevote).await?;
//! engine.on_commit(commit).await?;
//!
//! // Handle events
//! while let Some(event) = event_rx.recv().await {
//!     match event {
//!         ConsensusEvent::BroadcastProposal(p) => { /* send via POPEYE */ }
//!         ConsensusEvent::BlockFinalized { height, .. } => { /* persist via TAR */ }
//!         // ...
//!     }
//! }
//! ```

pub mod config;
pub mod engine;
pub mod error;
pub mod types;

// Re-exports for convenience
pub use config::ConsensusConfig;
pub use engine::{ConsensusEngine, ConsensusEvent, ProcessResult};
pub use error::{ConsensusError, Result};
pub use types::{
    BlockHash, Commit, CommitSet, ConsensusMessage, FinalityCertificate, Phase, Prevote,
    PrevoteSet, Proposal, RoundState, StateRoot, Validator, ValidatorId, ValidatorSet,
};
