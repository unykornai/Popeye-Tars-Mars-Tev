//! # POPEYE — P2P Networking
//!
//! POPEYE is the networking layer for Unykorn L1.
//! It handles peer discovery, message gossip, and network communication.
//!
//! ## Design Principles
//!
//! - **Never mutates state** - Only delivers messages
//! - **Never validates** - That's TEV and MARS's job
//! - **Event-driven** - Async channels for message passing
//!
//! ## Trust Model
//!
//! POPEYE hears rumors, not facts.
//! All messages must pass through TEV before reaching MARS.

pub mod config;
pub mod error;
pub mod libp2p_network;
pub mod message;
pub mod network;
pub mod peer;

pub use config::NetworkConfig;
pub use error::NetworkError;
pub use libp2p_network::Libp2pNetwork;
pub use message::NetworkMessage;
pub use network::Network;
pub use peer::PeerId;
