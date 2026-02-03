---
layout: default
title: Architecture
nav_order: 2
description: "Unykorn L1 Architecture and Design"
---

# 🏗️ Architecture
{: .no_toc }

Understanding the Unykorn L1 blockchain architecture and design principles.
{: .fs-6 .fw-300 }

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Overview

Unykorn L1 is a **closed-loop execution organism** — not just modules, but an integrated blockchain runtime with strict trust boundaries.

### The Four Pillars

| Component | Emoji | Role | Philosophy |
|:----------|:-----:|:-----|:-----------|
| **POPEYE** | 🌐 | Network Layer | "Hears rumors, not facts" |
| **TEV** | 🔐 | Crypto Gate | "Papers checked. No exceptions." |
| **MARS** | 🧠 | Runtime | "If MARS says no, the network doesn't matter" |
| **TAR** | 💾 | Storage | "Remembers, but never validates" |

---

## Data Flow

```mermaid
flowchart LR
    subgraph Network["🌐 POPEYE"]
        P1[Peer Discovery]
        P2[Message Gossip]
    end
    
    subgraph Crypto["🔐 TEV"]
        T1[Signature Verification]
        T2[Format Validation]
    end
    
    subgraph Runtime["🧠 MARS"]
        M1[State Machine]
        M2[Block Production]
    end
    
    subgraph Storage["💾 TAR"]
        S1[Block Store]
        S2[State Store]
    end
    
    Network --> Crypto
    Crypto --> Runtime
    Runtime --> Storage
```

---

## Trust Boundaries

### The Security Model

The architecture enforces strict trust boundaries. **Nothing crosses from the network to the runtime without cryptographic verification.**

```
┌──────────────────────────────────────────────────────────────────────┐
│                        UNTRUSTED ZONE                                │
│      (Network messages, peer data, external input)                   │
└──────────────────────────────────────┬───────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          🔐 TEV GATE                                 │
│                   (Cryptographic verification)                       │
│                                                                      │
│   ✅ Valid signature    → Proceed to MARS                            │
│   ❌ Invalid signature  → Reject (never reaches runtime)             │
└──────────────────────────────────────┬───────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────┐
│                        TRUSTED ZONE                                  │
│              (MARS execution, TAR persistence)                       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Transaction Lifecycle

```mermaid
sequenceDiagram
    participant User as 👤 User
    participant Network as 🌐 POPEYE
    participant Crypto as 🔐 TEV
    participant Runtime as 🧠 MARS
    participant Storage as 💾 TAR
    
    User->>Network: Submit Transaction
    Network->>Crypto: Raw TX Bytes
    
    alt Valid Signature
        Crypto->>Runtime: VerifiedTransaction
        Runtime->>Runtime: Validate Rules
        
        alt Valid Transaction
            Runtime->>Storage: Persist to Mempool
            Storage-->>Runtime: Confirmation
            Runtime->>Network: Gossip to Peers
            Network-->>User: TX Accepted
        else Invalid Transaction
            Runtime-->>Network: Reject
            Network-->>User: TX Rejected (rules)
        end
    else Invalid Signature
        Crypto-->>Network: Reject
        Network-->>User: TX Rejected (signature)
    end
```

---

## Block Production

```mermaid
sequenceDiagram
    participant Timer as ⏱️ Block Timer
    participant Runtime as 🧠 MARS
    participant Storage as 💾 TAR
    participant Network as 🌐 POPEYE
    participant Peers as 🌍 Peers
    
    Timer->>Runtime: Tick (3 seconds)
    Runtime->>Runtime: Drain Mempool
    Runtime->>Runtime: Apply Transactions
    Runtime->>Runtime: Compute State Root
    Runtime->>Storage: Persist Block
    Storage-->>Runtime: Block Height N
    Runtime->>Network: Broadcast Block
    Network->>Peers: Gossipsub Publish
```

---

## Component Details

### 🧠 MARS — Runtime Brain

**Responsibilities:**
- ✅ Canonical State - Owns the single source of truth
- ✅ TX Validation - Checks balances, nonces, rules
- ✅ Block Production - Creates deterministic blocks
- ✅ Block Validation - Verifies incoming blocks
- ✅ State Transitions - Pure functions for all mutations

**Restrictions:**
- ❌ No networking code
- ❌ No disk IO
- ❌ No RPC handling

---

### 🌐 POPEYE — Eyes & Ears

**Responsibilities:**
- ✅ Peer Discovery - mDNS for dev, extensible for prod
- ✅ Gossip Propagation - Broadcast transactions and blocks
- ✅ Message Routing - Normalize and forward payloads
- ✅ Duplicate Suppression - Prevent message flooding
- ✅ Backpressure - Rate limiting and flow control

**Restrictions:**
- ❌ Never mutates state
- ❌ Never validates economics
- ❌ Never finalizes blocks

---

### 🔐 TEV — Cryptographic Truth Gate

**Responsibilities:**
- ✅ Signature Verification - Ed25519 validation
- ✅ Format Enforcement - 96-byte transport format
- ✅ Replay Protection - Nonce verification
- ✅ Identity Enforcement - Public key ownership

**Transport Format:**
```
[Transaction Data...][Public Key (32 bytes)][Signature (64 bytes)]
```

**Restrictions:**
- ❌ No state management
- ❌ No networking
- ❌ No persistence

---

### 💾 TAR — Memory with Receipts

**Responsibilities:**
- ✅ Block Storage - Append-only, immutable
- ✅ State Snapshots - Point-in-time recovery
- ✅ TX Indexing - Query by height/hash
- ✅ Crash Recovery - Atomic writes, no corruption
- ✅ Continuity Verification - Chain integrity checks

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

**Restrictions:**
- ❌ Never validates data
- ❌ Never executes logic
- ❌ Never communicates over network

---

## Design Properties

| Property | Implementation |
|:---------|:---------------|
| **Determinism** | Same inputs → same outputs, always |
| **Separation** | Each crate has one job |
| **Fault Isolation** | Failures contained to one layer |
| **Recoverability** | TAR provides crash-safe persistence |
| **Observability** | Clear boundaries for logging/metrics |
| **Extensibility** | Consensus plugs in cleanly |
