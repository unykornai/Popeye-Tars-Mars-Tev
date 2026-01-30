//! libp2p-based network implementation.
//!
//! Real P2P networking using gossipsub for message propagation.

use crate::config::NetworkConfig;
use crate::message::{NetworkEvent, NetworkMessage};
use crate::NetworkError;
use futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, MessageId},
    identify, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Gossipsub topic for transactions
const TOPIC_TX: &str = "unykorn/tx/1.0.0";
/// Gossipsub topic for blocks
const TOPIC_BLOCK: &str = "unykorn/block/1.0.0";

/// Combined network behaviour.
#[derive(NetworkBehaviour)]
struct UnykornBehaviour {
    /// Gossipsub for message propagation
    gossipsub: gossipsub::Behaviour,
    /// mDNS for local peer discovery
    mdns: mdns::tokio::Behaviour,
    /// Identify for peer information exchange
    identify: identify::Behaviour,
}

/// libp2p-based network service.
pub struct Libp2pNetwork {
    /// The libp2p swarm
    swarm: Swarm<UnykornBehaviour>,
    /// Channel to send events to the node
    event_tx: mpsc::Sender<NetworkEvent>,
    /// Transaction topic
    topic_tx: IdentTopic,
    /// Block topic
    topic_block: IdentTopic,
}

impl Libp2pNetwork {
    /// Create a new libp2p network.
    pub async fn new(config: &NetworkConfig) -> Result<(Self, mpsc::Receiver<NetworkEvent>), NetworkError> {
        let (event_tx, event_rx) = mpsc::channel(1024);

        // Create topics
        let topic_tx = IdentTopic::new(TOPIC_TX);
        let topic_block = IdentTopic::new(TOPIC_BLOCK);

        // Message ID function (for deduplication)
        let message_id_fn = |message: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_be_bytes().to_vec())
        };

        // Gossipsub config
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| NetworkError::ConfigError(e.to_string()))?;

        // Build swarm
        let swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| NetworkError::TransportError(e.to_string()))?
            .with_behaviour(|key| {
                // Gossipsub
                let gossipsub = gossipsub::Behaviour::new(
                    MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                // mDNS
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                // Identify
                let identify = identify::Behaviour::new(
                    identify::Config::new(
                        "/unykorn/1.0.0".to_string(),
                        key.public(),
                    )
                    .with_agent_version("unykorn/0.1.0".to_string()),
                );

                Ok(UnykornBehaviour {
                    gossipsub,
                    mdns,
                    identify,
                })
            })
            .map_err(|e| NetworkError::BehaviourError(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let mut network = Self {
            swarm,
            event_tx,
            topic_tx: topic_tx.clone(),
            topic_block: topic_block.clone(),
        };

        // Subscribe to topics
        network
            .swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic_tx)
            .map_err(|e| NetworkError::SubscriptionError(e.to_string()))?;
        network
            .swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic_block)
            .map_err(|e| NetworkError::SubscriptionError(e.to_string()))?;

        // Listen on configured address
        let listen_addr: Multiaddr = format!("/ip4/{}/tcp/{}", 
            config.listen_addr.ip(), 
            config.listen_addr.port())
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| NetworkError::InvalidAddress(e.to_string()))?;

        network
            .swarm
            .listen_on(listen_addr)
            .map_err(|e| NetworkError::ListenError(e.to_string()))?;

        info!("Local peer ID: {}", network.swarm.local_peer_id());

        Ok((network, event_rx))
    }

    /// Get our local peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Get the number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.swarm.network_info().num_peers()
    }

    /// Connect to a bootstrap peer.
    pub fn dial(&mut self, addr: Multiaddr) -> Result<(), NetworkError> {
        self.swarm
            .dial(addr)
            .map_err(|e| NetworkError::DialError(e.to_string()))?;
        Ok(())
    }

    /// Broadcast a message to all peers via gossipsub.
    pub fn broadcast(&mut self, message: NetworkMessage) -> Result<(), NetworkError> {
        let data =
            bincode::serialize(&message).map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        let topic = match &message {
            NetworkMessage::Transaction(_) => &self.topic_tx,
            NetworkMessage::Block(_) => &self.topic_block,
            _ => return Ok(()), // Don't broadcast ping/pong/handshake via gossip
        };

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), data)
            .map_err(|e| NetworkError::PublishError(e.to_string()))?;

        Ok(())
    }

    /// Run the network event loop.
    pub async fn run(&mut self, mut shutdown: mpsc::Receiver<()>) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }
                _ = shutdown.recv() => {
                    info!("Network shutdown requested");
                    break;
                }
            }
        }
    }

    /// Handle a swarm event.
    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<UnykornBehaviourEvent>,
    ) -> Result<(), NetworkError> {
        match event {
            SwarmEvent::Behaviour(UnykornBehaviourEvent::Gossipsub(
                gossipsub::Event::Message { message, .. },
            )) => {
                self.handle_gossip_message(message).await?;
            }
            SwarmEvent::Behaviour(UnykornBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    debug!("mDNS discovered peer: {} at {}", peer_id, addr);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(UnykornBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _addr) in peers {
                    debug!("mDNS peer expired: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(UnykornBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
                ..
            })) => {
                debug!(
                    "Identified peer {} running {} with {}",
                    peer_id,
                    info.agent_version,
                    info.protocol_version
                );
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                let peer_bytes = peer_id_to_bytes(&peer_id);
                let _ = self
                    .event_tx
                    .send(NetworkEvent::PeerConnected {
                        peer_id: peer_bytes,
                    })
                    .await;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                let peer_bytes = peer_id_to_bytes(&peer_id);
                let _ = self
                    .event_tx
                    .send(NetworkEvent::PeerDisconnected {
                        peer_id: peer_bytes,
                    })
                    .await;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle an incoming gossip message.
    async fn handle_gossip_message(
        &mut self,
        message: gossipsub::Message,
    ) -> Result<(), NetworkError> {
        let network_message: NetworkMessage = bincode::deserialize(&message.data)
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;

        let from = message
            .source
            .map(|p| peer_id_to_bytes(&p))
            .unwrap_or([0u8; 32]);

        let event = NetworkEvent::MessageReceived {
            from,
            message: network_message,
        };

        self.event_tx
            .send(event)
            .await
            .map_err(|_| NetworkError::ChannelClosed)?;

        Ok(())
    }
}

/// Convert a libp2p PeerId to our 32-byte representation.
fn peer_id_to_bytes(peer_id: &PeerId) -> [u8; 32] {
    let bytes = peer_id.to_bytes();
    let mut result = [0u8; 32];
    let len = bytes.len().min(32);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_libp2p_network_creation() {
        let config = NetworkConfig::local(0, [1u8; 32]); // Port 0 for random
        let result = Libp2pNetwork::new(&config).await;
        assert!(result.is_ok());

        let (network, _rx) = result.unwrap();
        assert!(network.peer_count() == 0);
    }
}
