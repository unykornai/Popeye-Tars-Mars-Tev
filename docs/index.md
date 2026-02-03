---
layout: home
title: Home
nav_order: 1
description: "Unykorn L1 - A Production-Grade Layer-1 Blockchain Runtime Built in Rust"
permalink: /
---

<div align="center">

# 🦄 Unykorn L1 Blockchain
{: .fs-9 }

A Production-Grade Layer-1 Blockchain Runtime Built in Rust
{: .fs-6 .fw-300 }

[Get Started](#quick-start){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/unykornai/Popeye-Tars-Mars-Tev){: .btn .fs-5 .mb-4 .mb-md-0 }

</div>

---

## Why Unykorn L1?

Unykorn L1 is a **closed-loop execution organism** — not just modules, but an integrated blockchain runtime with strict trust boundaries.

### 🔒 Security First

Every message passes through cryptographic verification before reaching the runtime. **Nothing crosses from POPEYE to MARS without passing TEV.**

### ⚡ Performance Optimized

Built in Rust for maximum performance and safety. Deterministic execution with crash-safe persistence.

### 🧩 Modular Architecture

Four distinct layers with clear separation of concerns:
- **POPEYE** - Network Layer (P2P, Gossip)
- **TEV** - Crypto Gate (Verification)
- **MARS** - Runtime (State Machine)
- **TAR** - Storage (Persistence)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        UNYKORN L1 RUNTIME                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    │
│    │  POPEYE  │───▶│   TEV    │───▶│   MARS   │───▶│   TAR    │    │
│    │   🌐     │    │   🔐     │    │   🧠     │    │   💾     │    │
│    │  (P2P)   │    │ (Verify) │    │(Execute) │    │ (Store)  │    │
│    └──────────┘    └──────────┘    └──────────┘    └──────────┘    │
│                                                                     │
│    Network Layer   Crypto Gate     Runtime        Persistence       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- **Rust 1.75+** with `cargo`
- **Git** for version control

### Build

```bash
git clone https://github.com/unykornai/Popeye-Tars-Mars-Tev.git
cd Popeye-Tars-Mars-Tev
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Run

```bash
cargo run --release -p node
```

---

## Design Principles

| Principle | Description |
|:----------|:------------|
| 🔒 **Determinism** | Same inputs → same outputs, always |
| 🧩 **Separation** | Each crate has exactly one responsibility |
| 🛡️ **Isolation** | Failures are contained to one layer |
| 💾 **Recoverability** | Crash-safe persistence with atomic writes |
| 📊 **Observability** | Clear boundaries for logging and metrics |

---

## Test Results

| Crate | Tests | Status |
|:------|:-----:|:------:|
| 🧠 MARS | 14 | ✅ Pass |
| 🌐 POPEYE | 12 | ✅ Pass |
| 🔐 TEV | 9 | ✅ Pass |
| 💾 TAR | 8 | ✅ Pass |
| 🖥️ NODE | 5 | ✅ Pass |
| **Total** | **49** | **All Passing** |

---

## License

Unykorn L1 is open-source software licensed under the [MIT License](https://github.com/unykornai/Popeye-Tars-Mars-Tev/blob/main/LICENSE).
