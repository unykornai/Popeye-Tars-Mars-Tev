# POPEYE — P2P Networking

**POPEYE = Network Perception + Message Transport**

This is how the chain *sees other nodes*.

## Implementations

POPEYE provides two network implementations:

### Stub Network (`Network`)
- Channel-based local testing
- No real networking
- Perfect for unit tests

### libp2p Network (`Libp2pNetwork`)
- Real P2P networking via libp2p
- Gossipsub for message propagation
- mDNS for local peer discovery
- Noise protocol for encryption
- Yamux for multiplexing

## Responsibilities

- Peer discovery (mDNS for local dev, extensible)
- Gossip propagation (gossipsub)
- Message routing
- Duplicate suppression
- Backpressure handling

## Key Rules

- **NEVER mutates state** - Only delivers messages
- **NEVER validates economics** - That's MARS
- **NEVER finalizes blocks** - Only broadcasts

This prevents an entire class of consensus bugs.

## Message Types

- `TransactionMessage` - Propagate pending transactions
- `BlockMessage` - Propagate new blocks
- `HandshakeMessage` - Initial peer connection

## Gossipsub Topics

- `unykorn/tx/1.0.0` - Transaction propagation
- `unykorn/block/1.0.0` - Block propagation

## Design Properties

- Async Rust (tokio)
- Event-driven architecture
- Channel-based output to Runtime
- Configurable peer limits
- libp2p for production networking

## Mental Model

POPEYE is the **sensory nervous system**.
It hears rumors, not facts.

## Usage

### Stub Network (testing)
```rust
use popeye::{Network, NetworkConfig};

let config = NetworkConfig::local(9001, [1u8; 32]);
let (network, mut receiver) = Network::new(config);

while let Some(event) = receiver.recv().await {
    // Handle network events
}
```

### libp2p Network (production)
```rust
use popeye::{Libp2pNetwork, NetworkConfig};

let config = NetworkConfig::local(9001, [1u8; 32]);
let (mut network, mut receiver) = Libp2pNetwork::new(&config).await?;

// Dial bootstrap peers
network.dial("/ip4/127.0.0.1/tcp/9002".parse()?)?;

// Broadcast a transaction
network.broadcast(NetworkMessage::Transaction(tx_msg))?;

// Run the network event loop
network.run(shutdown_rx).await;
```
        NetworkMessage::Block(bytes) => { /* forward to TEV */ }
    }
}
```
