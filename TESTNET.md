# Unykorn Testnet: Aurora (v0.1.0-testnet)

**Status**: Active  
**Network ID**: `unykorn-testnet-aurora`  
**Consensus**: Unykorn BFT (Tendermint-style)  
**Binary**: `v0.1.0-testnet` (Consensus-enabled)

---

## 🌍 Network Topology

The Aurora testnet consists of 4 known validators with static keys.

| Validator | Role | Public Key (Ed25519) |
|-----------|------|----------------------|
| **Node A** | Bootstrap / Proposer | `a1...` (starts with 3b...) |
| **Node B** | Validator | `b2...` (starts with 4c...) |
| **Node C** | Validator | `c3...` (starts with 5d...) |
| **Node D** | Validator | `d4...` (starts with 6e...) |

## 🔍 How to Verify Consensus

### 1. Check Block Production
Nodes actively produce blocks. Check `data/testnet/node-X/blocks/` for `.block` files.

### 2. Check Finality Certificates
When consensus is reached (≥2/3 quorum), nodes persist finality certificates.
Location: `data/testnet/node-X/state/finality_{height}.json`

*If these files are missing, the network is creating blocks optimistically but not finalizing them (partition or config issue).*

### 3. Verify Determinism
Compare block hashes across validators:
```powershell
Get-FileHash data/testnet/node-a/blocks/000010.block
Get-FileHash data/testnet/node-b/blocks/000010.block
```
Hashes **MUST** match exactly.

---

## 🛠️ Operational Drills

To prove network resilience:

1. **Safety Check**: Kill 1 validator. Network continues (3/4 > 2/3).
2. **Liveness Check**: Kill 2 validators. Network halts (2/4 < 2/3). Restart one to recover.
3. **Recovery Check**: Restart a node. It replays blocks from disk to catch up.

---

## 📜 Operator Logs

Key log messages to watch for:
- `DEBUG consensus::engine: Proposal received from ...`
- `DEBUG consensus::engine: Prevote quorum reached`
- `INFO consensus::engine: Finalized block at height ...`

---

*This document marks the operational reality of the testnet.*
