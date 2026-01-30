//! # MARS — Runtime / State Machine
//!
//! MARS is the deterministic execution engine for Unykorn L1.
//! It owns canonical state and defines how transactions mutate state.
//!
//! ## Design Principles
//!
//! - **Deterministic**: Same inputs always produce same outputs
//! - **Pure**: All state transitions are pure functions
//! - **Isolated**: No networking, no disk IO
//!
//! ## Trust Model
//!
//! If MARS says "no", the network does not matter.
//! Every change to reality passes through this runtime.

pub mod state;
pub mod tx;
pub mod block;
pub mod runtime;
pub mod error;

pub use state::State;
pub use tx::Transaction;
pub use block::Block;
pub use runtime::Runtime;
pub use error::RuntimeError;
