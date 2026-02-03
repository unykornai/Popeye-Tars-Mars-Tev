# Consensus

**Deterministic BFT consensus layer for Unykorn L1.**

## Overview

The Consensus crate implements a **round-based, deterministic leader, Byzantine Fault Tolerant** consensus protocol. It coordinates agreement between validators without violating the core architectural constraints:

- **MARS** remains the sole state authority
- **TEV** remains the cryptographic gate
- **POPEYE** remains the network transport
- **TAR** remains the persistence layer

Consensus is a **thin coordinator** — it decides *which* MARS-produced block becomes canonical, but never mutates state directly.

## Architecture Position

```
POPEYE → TEV → CONSENSUS → MARS → TAR
                   ↑
            (this crate)
```

## Core Concepts

### Round Structure

Each consensus round has three phases:

1. **Propose** — Deterministic leader broadcasts a block proposal
2. **Prevote** — Validators vote on proposal validity
3. **Commit** — Validators commit to finalize the block

### Finality

A block with ≥ 2/3 commit signatures is **irreversible**. No reorgs after finality.

### Fork Choice

1. Prefer finalized block
2. If none finalized: prefer block with highest commit quorum
3. If tie: lowest block hash (deterministic tiebreaker)

## Fault Model

- Up to **f < 1/3 Byzantine validators**
- Partial synchrony (eventual message delivery)
- Honest validators follow protocol exactly

## What This Crate Does NOT Do

- ❌ Mutate state (MARS only)
- ❌ Validate signatures (TEV only)
- ❌ Handle network transport (POPEYE only)
- ❌ Persist data (TAR only)
- ❌ Economic incentives (future work)
- ❌ Slashing (future work)

## Usage

```rust
use consensus::{ConsensusEngine, ValidatorSet, ConsensusConfig};

let config = ConsensusConfig {
    propose_timeout: Duration::from_secs(3),
    prevote_timeout: Duration::from_secs(2),
    commit_timeout: Duration::from_secs(2),
};

let validators = ValidatorSet::new(validator_keys);
let engine = ConsensusEngine::new(config, validators, my_keypair);

// Drive consensus
engine.on_proposal(proposal).await?;
engine.on_prevote(prevote).await?;
engine.on_commit(commit).await?;
```

## Testing

```bash
cargo test -p consensus
```
