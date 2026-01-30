# Unykorn L1 Blockchain - Development Instructions

## Project Overview
Unykorn L1 is a production-grade Layer-1 blockchain runtime built in Rust.

## Architecture Components
- **MARS** - Runtime/State Machine (deterministic execution engine)
- **POPEYE** - P2P Networking (gossip propagation, peer discovery)
- **TEV** - Trusted Execution & Validation (cryptographic verification)
- **TAR** - Transaction & Archive Repository (persistence layer)
- **NODE** - Binary entrypoint (integration of all components)

## Development Rules
- Each crate must compile independently
- No circular dependencies between crates
- MARS never touches networking or disk IO
- POPEYE never mutates blockchain state
- TEV is stateless - no storage, no networking
- TAR handles all persistence with crash-safe writes

## Trust Boundaries
- Network messages (POPEYE) → Verification (TEV) → Validation (MARS) → Persistence (TAR)
- Nothing crosses from POPEYE to MARS without passing TEV

## Build Commands
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

## Coding Standards
- Use explicit error types (thiserror)
- All state transitions must be pure functions
- Document invariants with doc comments
- No TODOs or placeholder logic in committed code
