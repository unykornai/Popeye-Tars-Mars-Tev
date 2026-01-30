# NODE — Integration Binary

The node binary wires together all components of the Unykorn L1 blockchain.

## Execution Flow

```
POPEYE receives network message
    ↓
TEV verifies signature
    ↓
MARS validates and applies
    ↓
TAR persists to disk
    ↓
POPEYE broadcasts to peers
```

## Components

- **MARS** - Runtime (state machine)
- **POPEYE** - P2P Networking
- **TEV** - Cryptographic Validation
- **TAR** - Storage/Persistence

## Configuration

The node uses TOML configuration:

```toml
[node]
data_dir = "./data"

[network]
listen_port = 30303
max_peers = 50

[runtime]
producer_key = "0x..."
```

## Running

```bash
# Build
cargo build --release -p node

# Run with default config
./target/release/unykorn

# Run with custom config
./target/release/unykorn --config node.toml
```

## Trust Boundaries

- Nothing crosses from POPEYE → MARS without passing TEV
- TAR only stores data validated by MARS
- POPEYE never mutates state
