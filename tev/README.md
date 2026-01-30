# TEV — Trusted Execution & Validation

**TEV = Cryptographic Truth Gate**

This is where *claims become facts*.

## Responsibilities

- Signature verification
- Transaction authenticity checks
- Block authenticity checks
- Replay protection
- Identity enforcement

## Key Rule

**Nothing crosses from POPEYE → MARS without passing TEV.**

## Transport Format

TEV enforces a 96-byte transport signature format:
- 64 bytes: Ed25519 signature
- 32 bytes: Public key

## Design Properties

- **Stateless** - No storage, no persistence
- **No networking** - Pure verification only
- **Type-safe** - Verified vs Unverified types
- **Fail-fast** - Invalid payloads rejected immediately

## Security Guarantees

- Network spam ≠ state corruption
- Invalid blocks ≠ fork risk
- Malformed tx ≠ crash vector

## Mental Model

TEV is the **customs border**.
Papers checked. No exceptions.

## Usage

```rust
use tev::{verify_transaction, verify_block, VerifiedTransaction};

let verified = verify_transaction(raw_bytes)?;
// Only now can it proceed to MARS
```
