//! Network error types.

use thiserror::Error;

/// Errors that can occur in the networking layer.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Failed to bind to address
    #[error("failed to bind: {0}")]
    BindFailed(String),

    /// Peer connection failed
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Message send failed
    #[error("send failed")]
    SendFailed,

    /// Channel closed
    #[error("channel closed")]
    ChannelClosed,

    /// Invalid message format
    #[error("invalid message")]
    InvalidMessage,

    /// Peer not found
    #[error("peer not found: {0}")]
    PeerNotFound(String),

    /// Maximum peers reached
    #[error("max peers reached")]
    MaxPeersReached,

    /// Configuration error
    #[error("config error: {0}")]
    ConfigError(String),

    /// Transport error
    #[error("transport error: {0}")]
    TransportError(String),

    /// Behaviour error
    #[error("behaviour error: {0}")]
    BehaviourError(String),

    /// Subscription error
    #[error("subscription error: {0}")]
    SubscriptionError(String),

    /// Invalid address
    #[error("invalid address: {0}")]
    InvalidAddress(String),

    /// Listen error
    #[error("listen error: {0}")]
    ListenError(String),

    /// Dial error
    #[error("dial error: {0}")]
    DialError(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// Publish error
    #[error("publish error: {0}")]
    PublishError(String),
}
