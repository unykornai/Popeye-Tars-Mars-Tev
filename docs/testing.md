---
layout: default
title: Testing
nav_order: 5
description: "Unykorn L1 Testing Guide"
---

# 🧪 Testing
{: .no_toc }

Comprehensive testing guide for Unykorn L1.
{: .fs-6 .fw-300 }

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Test Overview

Unykorn L1 has a comprehensive test suite covering all components:

| Crate | Tests | Description |
|:------|:-----:|:------------|
| 🧠 MARS | 14 | State transitions, block production |
| 🌐 POPEYE | 12 | Networking, peer management |
| 🔐 TEV | 9 | Cryptographic verification |
| 💾 TAR | 8 | Storage, persistence |
| 🖥️ NODE | 5 | Integration tests |
| 📝 Docs | 1 | Documentation tests |
| **Total** | **49** | **All Passing** |

---

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### With Output

```bash
cargo test --workspace -- --nocapture
```

### Specific Crate

```bash
cargo test -p mars
cargo test -p popeye
cargo test -p tev
cargo test -p tar
cargo test -p node
```

### Specific Test

```bash
cargo test -p mars test_block_production
```

---

## Test Categories

### Unit Tests

Each module has unit tests in a `tests` submodule:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = State::new();
        assert_eq!(state.height, 0);
    }
}
```

### Integration Tests

The `node` crate includes integration tests that wire components together:

```rust
#[test]
fn test_node_creation() {
    let config = NodeConfig::dev();
    let node = Node::new(config);
    assert!(node.is_ok());
}
```

### Documentation Tests

Code examples in documentation are tested:

```rust
/// Create a new runtime.
///
/// # Example
///
/// ```rust
/// use mars::Runtime;
/// let runtime = Runtime::new();
/// ```
pub fn new() -> Self { ... }
```

---

## Soak Testing

For extended stability testing, see the [Soak Test Checklist](https://github.com/unykornai/Popeye-Tars-Mars-Tev/blob/main/SOAK_TEST_CHECKLIST.md).

### Quick Soak Test

```powershell
.\scripts\run-soak-test.ps1 -Duration 1 -BlockInterval 3
```

### Full 24-Hour Soak Test

```powershell
.\scripts\run-soak-test.ps1 -Duration 24 -BlockInterval 3
```

### Soak Test Parameters

| Parameter | Value | Description |
|:----------|:------|:------------|
| Duration | 24 hours | Test length |
| Block Interval | 3 seconds | Time between blocks |
| Expected Blocks | ~28,800 | Total blocks produced |
| Nodes | 3 | 1 producer, 2 validators |

### Pass/Fail Criteria

**PASS Conditions:**
- ✅ All 3 nodes running for full duration
- ✅ Block production never stopped
- ✅ Memory stayed under 500MB per node
- ✅ No panic/crash messages in logs
- ✅ Block height within 5% of expected

**FAIL Conditions:**
- ❌ Any node crashed
- ❌ Block production gap > 1 minute
- ❌ Memory exceeded 500MB
- ❌ Unhandled errors in logs
- ❌ Data corruption detected

---

## Stress Testing

### Variant A: Transaction Load

Inject synthetic transaction load:

```powershell
.\scripts\run-soak-test.ps1 -Duration 4 -TxPerBlock 100
```

**Expected:** Higher memory usage, no crashes.

### Variant B: Network Partition

Simulate network partitions:

1. Start 3-node devnet
2. Kill node-b at hour 12
3. Restart node-b at hour 13
4. Verify node-b catches up within 10 minutes

### Variant C: Disk Pressure

Test graceful degradation under disk pressure:

1. Fill disk to 90% capacity
2. Continue block production
3. Verify graceful error handling
4. No data corruption

---

## Code Coverage

Generate code coverage reports:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html
```

---

## Benchmarks

Run performance benchmarks:

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo criterion
```

---

## Continuous Integration

The project uses GitHub Actions for CI:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace
```

---

## Test Results Summary

```
running 49 tests

mars::block::tests::test_block_hash_deterministic ... ok
mars::block::tests::test_genesis_block ... ok
mars::runtime::tests::test_new_runtime ... ok
mars::runtime::tests::test_produce_block ... ok
... (all 49 tests pass)

test result: ok. 49 passed; 0 failed; 0 ignored
```
