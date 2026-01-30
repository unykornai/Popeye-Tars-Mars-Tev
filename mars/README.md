# MARS — Runtime / State Machine

**MARS = Deterministic State Machine + Execution Engine**

This is the *law* of the chain.

## Responsibilities

- Owns **canonical state**
- Defines **how transactions mutate state**
- Produces **blocks deterministically**
- Validates **everything** before it becomes real

## Core Components

- `State` - The canonical blockchain state
- `Transaction` - A state mutation request
- `Block` - A batch of transactions at a height
- `Runtime` - The execution engine

## Key Properties

- **No RPC dependency** - Does not wait for external coordinator
- **Network-aware** - Can receive data directly from P2P
- **Deterministic by construction** - Same inputs → same outputs, always
- **No networking code** - Delegates to POPEYE
- **No disk IO** - Delegates persistence to TAR

## Mental Model

MARS is the **constitutional court** of the chain.
Every change to reality passes through it.

## Usage

```rust
use mars::Runtime;

let mut runtime = Runtime::new();
runtime.submit_transaction(tx)?;
let block = runtime.produce_block();
```
