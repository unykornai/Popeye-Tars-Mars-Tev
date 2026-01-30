//! Peer identification and management.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Unique identifier for a peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    /// Create a new peer ID from bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generate a random peer ID (for testing).
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        for byte in &mut bytes {
            *byte = rand::random();
        }
        Self(bytes)
    }

    /// Get the bytes of this peer ID.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}{:02x}..{:02x}{:02x}",
            self.0[0], self.0[1], self.0[30], self.0[31])
    }
}

mod rand {
    /// Simple random byte generator using system time.
    pub fn random() -> u8 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        (nanos % 256) as u8
    }
}

/// Information about a connected peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Peer's unique identifier
    pub id: PeerId,

    /// Peer's network address
    pub addr: SocketAddr,

    /// Peer's current block height
    pub height: u64,

    /// Protocol version
    pub version: u32,

    /// Connection timestamp
    pub connected_at: u64,
}

impl PeerInfo {
    /// Create new peer info.
    pub fn new(id: PeerId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            height: 0,
            version: 1,
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Update peer's height.
    pub fn update_height(&mut self, height: u64) {
        self.height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_display() {
        let id = PeerId::new([0xab; 32]);
        let display = format!("{}", id);
        assert!(display.contains("abab"));
    }

    #[test]
    fn test_peer_info() {
        let id = PeerId::new([1u8; 32]);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let info = PeerInfo::new(id, addr);

        assert_eq!(info.id, id);
        assert_eq!(info.version, 1);
    }
}
