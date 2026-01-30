# TAR — Transaction & Archive Repository

**TAR = Memory with Receipts**

This is how the chain *remembers*.

## Responsibilities

- Persistent block storage
- Persistent state storage
- Transaction indexing
- Restart recovery
- Disk integrity enforcement

## Key Properties

- **Crash-safe writes** - Atomic commits prevent corruption
- **Append-only blocks** - Immutable once written
- **Deterministic reload** - Same state on restart
- **Verifiable continuity** - Chain integrity checks

## Guarantees

- Restart ≠ data loss
- Crash ≠ corruption
- Time ≠ amnesia

## Disk Layout

```
data/
├── blocks/
│   ├── 000000.block
│   ├── 000001.block
│   └── ...
├── state/
│   └── latest.state
└── meta/
    └── chain.meta
```

## Mental Model

TAR is the **ledger vault**.
Once written, it stays written.

## Usage

```rust
use tar::Storage;
use std::path::PathBuf;

let storage = Storage::new(PathBuf::from("./data"));
storage.save_block(&block)?;
let state = storage.load_latest_state()?;
```
