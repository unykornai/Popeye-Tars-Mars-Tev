//! Network service.
//!
//! The main network orchestrator that handles peer connections,
//! message routing, and gossip propagation.

use crate::config::NetworkConfig;
use crate::message::{NetworkEvent, NetworkMessage};
use crate::peer::{PeerId, PeerInfo};
use crate::NetworkError;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

/// The main network service.
///
/// Manages peer connections and message routing.
/// Outputs received messages to a channel for the runtime to consume.
pub struct Network {
    /// Configuration
    config: NetworkConfig,

    /// Our peer ID
    local_id: PeerId,

    /// Connected peers
    peers: HashMap<PeerId, PeerInfo>,

    /// Sender for outgoing events
    event_tx: mpsc::Sender<NetworkEvent>,

    /// Recently seen message hashes (for deduplication)
    seen_messages: HashSet<[u8; 32]>,
}

impl Network {
    /// Create a new network service.
    ///
    /// Returns the network and a receiver for events.
    pub fn new(config: NetworkConfig) -> (Self, mpsc::Receiver<NetworkEvent>) {
        let (event_tx, event_rx) = mpsc::channel(1024);
        let local_id = PeerId::new(config.node_id);

        let network = Self {
            config,
            local_id,
            peers: HashMap::new(),
            event_tx,
            seen_messages: HashSet::new(),
        };

        (network, event_rx)
    }

    /// Get our local peer ID.
    pub fn local_id(&self) -> PeerId {
        self.local_id
    }

    /// Get the number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Check if we can accept more peers.
    pub fn can_accept_peer(&self) -> bool {
        self.peers.len() < self.config.max_peers
    }

    /// Add a peer connection.
    pub fn add_peer(&mut self, info: PeerInfo) -> Result<(), NetworkError> {
        if !self.can_accept_peer() {
            return Err(NetworkError::MaxPeersReached);
        }

        self.peers.insert(info.id, info);
        Ok(())
    }

    /// Remove a peer connection.
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Get a peer by ID.
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get all connected peer IDs.
    pub fn peer_ids(&self) -> Vec<PeerId> {
        self.peers.keys().copied().collect()
    }

    /// Check if a message has been seen before (deduplication).
    pub fn is_duplicate(&mut self, hash: &[u8; 32]) -> bool {
        if self.seen_messages.contains(hash) {
            return true;
        }

        // Add to seen set
        self.seen_messages.insert(*hash);

        // Limit size of seen set
        if self.seen_messages.len() > 10000 {
            // Simple eviction - clear half
            let to_keep: Vec<_> = self.seen_messages.iter().take(5000).copied().collect();
            self.seen_messages.clear();
            for h in to_keep {
                self.seen_messages.insert(h);
            }
        }

        false
    }

    /// Broadcast a message to all connected peers.
    ///
    /// POPEYE only broadcasts - it never validates the message content.
    pub async fn broadcast(&self, message: NetworkMessage) -> Result<(), NetworkError> {
        // In a real implementation, this would send to all peers.
        // For now, we just log that we would broadcast.
        let _ = message;
        Ok(())
    }

    /// Handle an incoming message from a peer.
    ///
    /// This forwards the message to the event channel without validation.
    /// TEV will validate before MARS processes.
    pub async fn handle_message(
        &mut self,
        from: PeerId,
        message: NetworkMessage,
    ) -> Result<(), NetworkError> {
        let event = NetworkEvent::MessageReceived {
            from: *from.as_bytes(),
            message,
        };

        self.event_tx
            .send(event)
            .await
            .map_err(|_| NetworkError::ChannelClosed)?;

        Ok(())
    }

    /// Notify of a new peer connection.
    pub async fn notify_peer_connected(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        let event = NetworkEvent::PeerConnected {
            peer_id: *peer_id.as_bytes(),
        };

        self.event_tx
            .send(event)
            .await
            .map_err(|_| NetworkError::ChannelClosed)?;

        Ok(())
    }

    /// Notify of a peer disconnection.
    pub async fn notify_peer_disconnected(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        let event = NetworkEvent::PeerDisconnected {
            peer_id: *peer_id.as_bytes(),
        };

        self.event_tx
            .send(event)
            .await
            .map_err(|_| NetworkError::ChannelClosed)?;

        Ok(())
    }
}

/// Simple network runner for testing.
///
/// In production, this would be replaced with actual libp2p or similar.
pub async fn run_network(network: Network, mut shutdown: mpsc::Receiver<()>) {
    tokio::select! {
        _ = shutdown.recv() => {
            // Graceful shutdown
        }
    }

    // Cleanup
    let _ = network;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let config = NetworkConfig::local(8080, [1u8; 32]);
        let (network, _rx) = Network::new(config);

        assert_eq!(network.peer_count(), 0);
        assert!(network.can_accept_peer());
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = NetworkConfig::local(8080, [1u8; 32])
            .with_max_peers(2);
        let (mut network, _rx) = Network::new(config);

        let peer1 = PeerInfo::new(
            PeerId::new([2u8; 32]),
            "127.0.0.1:8081".parse().unwrap(),
        );
        let peer2 = PeerInfo::new(
            PeerId::new([3u8; 32]),
            "127.0.0.1:8082".parse().unwrap(),
        );
        let peer3 = PeerInfo::new(
            PeerId::new([4u8; 32]),
            "127.0.0.1:8083".parse().unwrap(),
        );

        assert!(network.add_peer(peer1).is_ok());
        assert!(network.add_peer(peer2).is_ok());
        assert!(network.add_peer(peer3).is_err()); // Max reached
    }

    #[tokio::test]
    async fn test_deduplication() {
        let config = NetworkConfig::local(8080, [1u8; 32]);
        let (mut network, _rx) = Network::new(config);

        let hash = [42u8; 32];

        assert!(!network.is_duplicate(&hash));
        assert!(network.is_duplicate(&hash));
    }

    #[tokio::test]
    async fn test_message_forwarding() {
        let config = NetworkConfig::local(8080, [1u8; 32]);
        let (mut network, mut rx) = Network::new(config);

        let from = PeerId::new([2u8; 32]);
        let msg = NetworkMessage::Ping(42);

        network.handle_message(from, msg).await.unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            NetworkEvent::MessageReceived { from: f, message } => {
                assert_eq!(f, [2u8; 32]);
                match message {
                    NetworkMessage::Ping(n) => assert_eq!(n, 42),
                    _ => panic!("wrong message type"),
                }
            }
            _ => panic!("wrong event type"),
        }
    }
}
