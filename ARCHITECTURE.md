# Unykorn L1 Architecture

## Overview

Unykorn L1 is a **closed-loop execution organism** - not just modules, but an integrated blockchain runtime with strict trust boundaries.

## Component Responsibilities

### MARS — The Runtime Brain

**MARS = Deterministic State Machine + Execution Engine**

This is the *law* of the chain.

| Responsibility | Description |
|----------------|-------------|
| Canonical State | Owns the single source of truth |
| Transaction Validation | Checks balances, nonces, rules |
| Block Production | Creates deterministic blocks |
| Block Validation | Verifies incoming blocks |
| State Transitions | Pure functions for all mutations |

**What MARS Does NOT Do:**
- No networking code
- No disk IO
- No RPC handling

**Mental Model:** MARS is the **constitutional court**. Every change to reality passes through it.

---

### POPEYE — Eyes & Ears (P2P + Gossip)

**POPEYE = Network Perception + Message Transport**

This is how the chain *sees other nodes*.

| Responsibility | Description |
|----------------|-------------|
| Peer Discovery | mDNS (dev), extensible for production |
| Gossip Propagation | Broadcast transactions and blocks |
| Message Routing | Normalize and forward payloads |
| Duplicate Suppression | Prevent message flooding |
| Backpressure Handling | Rate limiting and flow control |

**What POPEYE Does NOT Do:**
- Never mutates state
- Never validates economics
- Never finalizes blocks

**Mental Model:** POPEYE is the **sensory nervous system**. It hears rumors, not facts.

---

### TEV — Cryptographic Truth Gate

**TEV = Cryptographic Firewall**

This is where *claims become facts*.

| Responsibility | Description |
|----------------|-------------|
| Signature Verification | Ed25519 validation |
| Format Enforcement | 96-byte transport format |
| Replay Protection | Nonce verification |
| Identity Enforcement | Public key ownership checks |

**Transport Format (96 bytes):**
```
[Transaction Data...][Public Key (32 bytes)][Signature (64 bytes)]
```

**What TEV Does NOT Do:**
- No state management
- No networking
- No persistence

**Mental Model:** TEV is the **customs border**. Papers checked. No exceptions.

---

### TAR — Memory with Receipts

**TAR = Persistent Storage Layer**

This is how the chain *remembers*.

| Responsibility | Description |
|----------------|-------------|
| Block Storage | Append-only, immutable |
| State Snapshots | Point-in-time recovery |
| Transaction Indexing | Query by height/hash |
| Crash Recovery | Atomic writes, no corruption |
| Continuity Verification | Chain integrity checks |

**Disk Layout:**
```
data/
├── blocks/
│   ├── 000000.block
│   └── 000001.block
├── state/
│   ├── latest.state
│   └── snapshot_000100.state
└── meta/
    └── chain.meta
```

**Mental Model:** TAR is the **ledger vault**. Once written, it stays written.

---

## Execution Flow

### Transaction Path

```
1. POPEYE receives tx from peer
        ↓
2. TEV verifies signature + format
        ↓
3. MARS validates transaction rules
        ↓
4. TAR persists accepted tx
        ↓
5. MARS includes tx in block
        ↓
6. POPEYE broadcasts block
```

### Block Path

```
1. POPEYE receives block
        ↓
2. TEV verifies block signature
        ↓
3. MARS validates block correctness
        ↓
4. TAR persists block + state
        ↓
5. Runtime advances height
```

**There is no bypass path.**

---

## Trust Boundaries

```
┌──────────────────────────────────────────────────────────────┐
│                       UNTRUSTED ZONE                          │
│    (Network messages, peer data, external input)             │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                         TEV GATE                              │
│              (Cryptographic verification)                     │
│                                                               │
│  ✓ Valid signature    → proceed                              │
│  ✗ Invalid signature  → reject (never reaches MARS)          │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                       TRUSTED ZONE                            │
│             (MARS execution, TAR persistence)                 │
│                                                               │
│  • Deterministic state transitions                           │
│  • Crash-safe writes                                         │
│  • Recoverable on restart                                    │
└──────────────────────────────────────────────────────────────┘
```

---

## Design Properties

| Property | Implementation |
|----------|---------------|
| **Determinism** | Same inputs → same outputs, always |
| **Separation of Concerns** | Each crate has one job |
| **Fault Isolation** | Failures contained to one layer |
| **Recoverability** | TAR provides crash-safe persistence |
| **Observability** | Clear boundaries for logging/metrics |
| **Extensibility** | Consensus plugs in cleanly |

---

## Future Extensions

This architecture supports clean addition of:

### Phase 3: Consensus
- Tendermint / HotStuff
- Validator roles
- Voting messages
- Finality layers

### Governance
- Upgrades as transactions
- Parameter changes
- Validator set changes

### RWA & Compliance
- Attestation anchoring
- Immutable proofs
- Audit trails

### External Interfaces
- RPC servers
- SDKs
- Indexers
- Explorers

All without touching core truth logic.

---

## Summary

| Component | One-Sentence Description |
|-----------|--------------------------|
| **MARS** | Decides what is true |
| **POPEYE** | Hears what others claim |
| **TEV** | Proves who is allowed to speak |
| **TAR** | Remembers what happened |

Together, they form a **real blockchain**, not a demo.
