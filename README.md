# Unykorn L1 Blockchain

A production-grade Layer-1 blockchain runtime built in Rust.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      UNYKORN L1                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│   │ POPEYE  │───▶│   TEV   │───▶│  MARS   │───▶│   TAR   │  │
│   │  (P2P)  │    │ (Verify)│    │(Execute)│    │ (Store) │  │
│   └─────────┘    └─────────┘    └─────────┘    └─────────┘  │
│                                                              │
│   Network Layer  │  Crypto Gate │  Runtime   │  Persistence │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Components

| Crate | Purpose | Key Responsibility |
|-------|---------|-------------------|
| **MARS** | Runtime / State Machine | Deterministic execution, block production |
| **POPEYE** | P2P Networking | Peer discovery, gossip propagation |
| **TEV** | Trusted Execution & Validation | Cryptographic verification |
| **TAR** | Transaction & Archive Repository | Crash-safe persistence |
| **NODE** | Binary Entrypoint | Component integration |

## Quick Start

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Run Node

```bash
# Default configuration
cargo run --release -p node

# Development mode
cargo run -p node -- --dev

# Custom configuration
cargo run -p node -- --config node.toml
```

## Trust Boundaries

```
Network (untrusted) → TEV (verify) → MARS (validate) → TAR (persist)
```

- **POPEYE never mutates state** - Only delivers messages
- **TEV is stateless** - Pure cryptographic verification
- **MARS is the law** - If MARS says no, the network doesn't matter
- **TAR is the memory** - Crash-safe, append-only persistence

## Configuration

Create a `node.toml` file:

```toml
[node]
data_dir = "./data"
log_level = "info"

[network]
listen_port = 30303
max_peers = 50
bootstrap_peers = []

[runtime]
chain_id = "unykorn-mainnet"
producer_enabled = false
```

## Project Structure

```
unykorn-l1/
├── Cargo.toml          # Workspace manifest
├── mars/               # Runtime / State Machine
│   └── src/
│       ├── lib.rs
│       ├── state.rs    # Blockchain state
│       ├── tx.rs       # Transaction types
│       ├── block.rs    # Block types
│       ├── runtime.rs  # Execution engine
│       └── error.rs    # Error types
├── popeye/             # P2P Networking
│   └── src/
│       ├── lib.rs
│       ├── network.rs  # Network service
│       ├── message.rs  # Message types
│       ├── peer.rs     # Peer management
│       └── config.rs   # Network config
├── tev/                # Cryptographic Validation
│   └── src/
│       ├── lib.rs
│       ├── signature.rs # Ed25519 operations
│       ├── verified.rs  # Verified types
│       └── error.rs     # Validation errors
├── tar/                # Storage / Persistence
│   └── src/
│       ├── lib.rs
│       ├── storage.rs   # Storage facade
│       ├── block_store.rs
│       └── state_store.rs
└── node/               # Binary Entrypoint
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── node.rs      # Node orchestration
        └── config.rs    # TOML configuration
```

## Development Rules

1. **Each crate must compile independently**
2. **No circular dependencies**
3. **MARS never touches networking or disk IO**
4. **POPEYE never mutates blockchain state**
5. **TEV is stateless - no storage, no networking**
6. **TAR handles all persistence with crash-safe writes**

## Devnet Operations

### Start 3-Node Devnet

```powershell
.\scripts\start-devnet.ps1
```

### Stop Devnet

```powershell
.\scripts\stop-devnet.ps1
```

### Health Check

```powershell
.\scripts\health-check.ps1
```

### Collect Logs

```powershell
.\scripts\collect-logs.ps1 -Tag "my-test-run"
```

See [OPERATOR_RUNBOOK.md](OPERATOR_RUNBOOK.md) for detailed operations.

## P2P Networking

POPEYE supports two networking backends:

- **Stub Network** - For testing (channel-based)
- **libp2p Network** - For production (gossipsub, mDNS, noise, yamux)

The libp2p implementation uses:
- **Gossipsub** for efficient message propagation
- **mDNS** for local peer discovery
- **Noise** for encrypted connections
- **Yamux** for connection multiplexing

## License

MIT
