# Unykorn L1 — Consensus Specification

## Version 1.0.0

**Status**: Implemented  
**Authors**: Unykorn Team  
**Last Updated**: February 2026

---

## 1. Overview

This document specifies the **Minimum Viable Consensus (MVC)** protocol for Unykorn L1 — a deterministic, round-based, Byzantine Fault Tolerant consensus mechanism designed to operate within the existing four-chamber architecture without violating trust boundaries.

### 1.1 Design Goals (Non-Negotiable)

Add **multi-node agreement** to Unykorn L1 **without**:

- ❌ Breaking deterministic execution
- ❌ Polluting MARS with network logic
- ❌ Weakening TEV's cryptographic gate
- ❌ Turning POPEYE into a source of truth
- ❌ Introducing economic or governance complexity prematurely

**This is consensus as plumbing, not politics.**

---

## 2. Architecture Position

### 2.1 Where Consensus Lives

```
POPEYE → TEV → CONSENSUS → MARS → TAR
                   ↑
            (new layer)
```

### 2.2 What Consensus Does NOT Own

| Component | Owner | Consensus May NOT... |
|-----------|-------|---------------------|
| State transitions | MARS | Mutate state directly |
| Signature validation | TEV | Verify cryptographic signatures |
| Network transport | POPEYE | Handle peer discovery or gossip |
| Persistence | TAR | Write to disk directly |

### 2.3 What Consensus DOES Own

- ✅ Ordering agreement
- ✅ Block proposal coordination
- ✅ Vote aggregation
- ✅ Finality decisions

**Key Rule**: Consensus never mutates state directly. It only decides *which MARS-produced block becomes canonical*.

---

## 3. Fault Model

### 3.1 Assumptions

| Property | Value |
|----------|-------|
| Maximum Byzantine validators | f < n/3 |
| Network model | Partial synchrony (eventual delivery) |
| Honest behavior | Honest validators follow protocol exactly |
| Identity model | Explicit enumeration with Ed25519 keys |
| Voting weight | 1 per validator (extensible for staking) |

### 3.2 Threat Model

- **Network adversary**: Can delay, reorder, but not drop messages indefinitely
- **Byzantine validators**: May send conflicting votes, invalid blocks, or go offline
- **No trusted setup**: All security derives from cryptographic primitives

---

## 4. Protocol Specification

### 4.1 Round Structure

Consensus proceeds in **numbered rounds**. Each round has **three phases**:

```
┌─────────────────────────────────────────────────────┐
│                    ROUND r                          │
├─────────────┬─────────────┬─────────────────────────┤
│   PROPOSE   │   PREVOTE   │       COMMIT            │
│   Phase 1   │   Phase 2   │       Phase 3           │
├─────────────┼─────────────┼─────────────────────────┤
│ Leader      │ Validators  │ Validators sign         │
│ broadcasts  │ vote for    │ commit if ≥2/3          │
│ proposal    │ or against  │ prevotes received       │
└─────────────┴─────────────┴─────────────────────────┘
```

### 4.2 Phase 1: Propose

1. **Leader Selection**: `leader = validators[round % n]`
2. Leader collects TEV-verified transactions
3. Leader runs MARS to produce:
   ```
   Block {
       height: u64,
       prev_hash: [u8; 32],
       state_root: [u8; 32],
       transactions: Vec<u8>
   }
   ```
4. Leader signs proposal with Ed25519
5. Proposal broadcast via POPEYE

**Critical**: MARS executes **before** consensus agreement. Consensus agrees on *the output*, not the process.

### 4.3 Phase 2: Prevote

1. Validators receive proposal
2. Validators verify:
   - Proposal signature (via TEV)
   - Correct round and height
   - Proposal from correct leader
   - Block validity (re-run MARS if needed)
3. If valid → sign `Prevote(block_hash)`
4. If invalid → sign `Prevote(nil)`
5. Votes flow: POPEYE → TEV → Consensus

### 4.4 Phase 3: Commit

1. If validator observes ≥2/3 prevotes for same `block_hash`:
   - Sign `Commit(block_hash)`
2. When ≥2/3 commits observed:
   - Block is **FINAL**

---

## 5. Finality

### 5.1 Finality Rule

> **A block with ≥2/3 commit signatures is IRREVERSIBLE.**

| Property | Guarantee |
|----------|-----------|
| Reorg after finality | ❌ Never |
| Probabilistic settlement | ❌ Not used |
| Longest-chain rule | ❌ Not used |

### 5.2 Finality Certificate

```rust
pub struct FinalityCertificate {
    height: u64,
    block_hash: [u8; 32],
    commits: Vec<Commit>,
    total_weight: u64,
}
```

A finality certificate is the proof that a block is irreversibly committed.

---

## 6. Fork Choice

### 6.1 Fork Choice Rule (Deterministic)

1. **Prefer finalized block**
2. If none finalized: **prefer block with highest commit quorum**
3. If tie: **lowest block hash** (deterministic tiebreaker)

Because leaders are deterministic, forks are rare and shallow.

### 6.2 Locking

Once a validator sees ≥2/3 prevotes for a block, it **locks** on that block:

```rust
locked_block: Option<BlockHash>,
locked_round: Option<u64>,
```

Locked validators will only prevote for their locked block in subsequent rounds.

---

## 7. Failure Handling

### 7.1 Leader Failure

```
Timeout expires → Move to round r+1 → New leader selected deterministically
```

Timeouts use exponential backoff:
```
timeout(round) = base_timeout + delta * round
```

### 7.2 Validator Offline

Consensus continues as long as ≥2/3 of voting weight is online.

### 7.3 Byzantine Behavior

| Behavior | Response |
|----------|----------|
| Invalid votes | Rejected by TEV |
| Double-voting | Detectable (future slashing hook) |
| Equivocation | Logged, ignored (no slashing yet) |

---

## 8. Persistence

### 8.1 What Gets Persisted (via TAR)

| Artifact | File | Purpose |
|----------|------|---------|
| `RoundState` | `round_state.json` | Resume after crash |
| `FinalityCertificate` | `finality_{height}.json` | Prove finality |
| `ValidatorSet` | `validators.json` | Validator configuration |

### 8.2 Crash Recovery

On restart:
1. Load `round_state.json`
2. Replay to last finalized height
3. Continue from current round

All writes are:
- ✅ Append-only
- ✅ Atomic (temp file + rename)
- ✅ Crash-safe (fsync before rename)

---

## 9. Message Types

### 9.1 Proposal

```rust
pub struct Proposal {
    height: u64,
    round: u64,
    prev_hash: [u8; 32],
    block_hash: [u8; 32],
    state_root: [u8; 32],
    transactions: Vec<u8>,
    proposer: ValidatorId,
    signature: Signature64,
}
```

### 9.2 Prevote

```rust
pub struct Prevote {
    height: u64,
    round: u64,
    block_hash: Option<[u8; 32]>,  // None = nil vote
    validator: ValidatorId,
    signature: Signature64,
}
```

### 9.3 Commit

```rust
pub struct Commit {
    height: u64,
    round: u64,
    block_hash: [u8; 32],
    validator: ValidatorId,
    signature: Signature64,
}
```

---

## 10. Quorum Calculations

### 10.1 Thresholds

| Calculation | Formula | For n=4 |
|-------------|---------|---------|
| Quorum threshold | 2n/3 + 1 | 3 |
| Max faulty | (n-1)/3 | 1 |

### 10.2 Safety Property

If f < n/3 validators are Byzantine:
- **Safety**: No two conflicting blocks can both achieve finality
- **Liveness**: Honest validators will eventually finalize a block

---

## 11. Implementation Status

### 11.1 What Is Built NOW

| Component | Status | Location |
|-----------|--------|----------|
| Round coordinator | ✅ Complete | `consensus/src/engine.rs` |
| Leader selection | ✅ Complete | `consensus/src/types.rs` |
| Proposal/vote messages | ✅ Complete | `consensus/src/types.rs` |
| Quorum detection | ✅ Complete | `consensus/src/types.rs` |
| Finality logic | ✅ Complete | `consensus/src/engine.rs` |
| TAR persistence | ✅ Complete | `tar/src/consensus_store.rs` |
| Unit tests | ✅ 10 passing | `consensus/src/*.rs` |

### 11.2 What Is Explicitly DEFERRED

| Feature | Status | Reason |
|---------|--------|--------|
| Staking | ⏳ Deferred | Economics not needed for correctness |
| Slashing | ⏳ Deferred | Requires staking |
| Tokenomics | ⏳ Deferred | Protocol-layer concern |
| Governance | ⏳ Deferred | Not needed for consensus |
| Smart contracts | ⏳ Deferred | Application-layer concern |
| Parallel execution | ⏳ Deferred | Optimization |

---

## 12. Test Coverage

| Test | Coverage |
|------|----------|
| `validator_set_quorum` | Quorum threshold calculation |
| `leader_rotation` | Deterministic leader selection |
| `prevote_set_aggregation` | Vote collection and weight |
| `duplicate_vote_rejected` | Equivocation prevention |
| `round_state_progression` | State machine transitions |
| `engine_creation` | Engine initialization |
| `start_new_height` | Height advancement |
| `timeout_advances_round` | Round timeout handling |
| `round_state_persistence` | TAR crash recovery |
| `finality_certificate_persistence` | Finality proofs |

---

## 13. Security Considerations

### 13.1 Invariants

1. **No state mutation without MARS**: Consensus only selects blocks
2. **No signature validation without TEV**: All crypto goes through the gate
3. **No network logic in consensus**: POPEYE handles transport
4. **Deterministic execution**: Same inputs → same outputs, always

### 13.2 Attack Surface

| Vector | Mitigation |
|--------|------------|
| Invalid proposals | Rejected by TEV signature check |
| Double voting | Detectable, logged (future slashing) |
| Long-range attacks | Prevented by finality |
| Network partition | Liveness may halt, safety preserved |

---

## 14. Next Steps

**Only after consensus is running on testnet:**

1. Validator lifecycle and slashing
2. Genesis configuration and key ceremony
3. Economic incentives
4. Domain-specific execution (telecom, AI events)
5. Contracts or VMs

**None of these should be touched until consensus is nailed.**

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Round** | A single attempt to reach consensus at a height |
| **Height** | Block number in the chain |
| **Quorum** | ≥2/3 of total voting weight |
| **Finality** | Irreversible commitment to a block |
| **Prevote** | First-round vote for block validity |
| **Commit** | Second-round vote for finalization |
| **Nil vote** | Vote against all proposals in a round |
| **Lock** | Obligation to vote for a specific block |

---

## Appendix B: References

- Tendermint BFT Paper
- HotStuff: BFT Consensus with Linearity
- PBFT: Practical Byzantine Fault Tolerance

---

**End of Specification**
