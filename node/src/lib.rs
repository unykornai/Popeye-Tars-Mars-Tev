//! # Unykorn L1 Node
//!
//! The main binary that integrates all blockchain components.
//!
//! ## Architecture
//!
//! ```text
//! POPEYE (P2P) → TEV (Verify) → MARS (Execute) → TAR (Persist)
//! ```

pub mod config;
pub mod node;

pub use config::NodeConfig;
pub use node::Node;
