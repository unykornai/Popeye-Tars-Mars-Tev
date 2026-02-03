---
layout: default
title: Quick Start
nav_order: 4
description: "Getting started with Unykorn L1"
---

# 🚀 Quick Start
{: .no_toc }

Get up and running with Unykorn L1 in minutes.
{: .fs-6 .fw-300 }

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Prerequisites

Before you begin, ensure you have the following installed:

| Requirement | Version | Check Command |
|:------------|:--------|:--------------|
| Rust | 1.75+ | `rustc --version` |
| Cargo | Latest | `cargo --version` |
| Git | Latest | `git --version` |

---

## Installation

### Clone the Repository

```bash
git clone https://github.com/unykornai/Popeye-Tars-Mars-Tev.git
cd Popeye-Tars-Mars-Tev
```

### Build the Project

```bash
# Debug build (faster compilation)
cargo build --workspace

# Release build (optimized)
cargo build --release --workspace
```

---

## Running Tests

### Run All Tests

```bash
cargo test --workspace
```

### Run Tests with Output

```bash
cargo test --workspace -- --nocapture
```

### Run Specific Crate Tests

```bash
cargo test -p mars      # Runtime tests
cargo test -p popeye    # Networking tests
cargo test -p tev       # Crypto tests
cargo test -p tar       # Storage tests
cargo test -p node      # Integration tests
```

---

## Running a Single Node

### Default Configuration

```bash
cargo run --release -p node
```

### Development Mode

```bash
cargo run -p node -- --dev
```

### Custom Configuration

```bash
cargo run -p node -- --config config/node-a.toml
```

---

## Running a 3-Node Devnet

### Windows (PowerShell)

```powershell
# Start the devnet
.\scripts\start-devnet.ps1

# Check health
.\scripts\health-check.ps1

# View metrics
.\scripts\metrics-snapshot.ps1

# Stop the devnet
.\scripts\stop-devnet.ps1
```

### Expected Output

When the devnet starts successfully, you should see:

```
Starting Unykorn L1 node...
  Data dir: "dev_data/node-a"
  Listen: 0.0.0.0:30303
  Height: 0
  Producer: true

[INFO] Block produced: height=1, txs=0
[INFO] Block produced: height=2, txs=0
[INFO] Block produced: height=3, txs=0
```

---

## Configuration

### Node Configuration File

Create a `node.toml` file:

```toml
[node]
data_dir = "./data"
log_level = "info"

[network]
listen_addr = "0.0.0.0"
listen_port = 30303
max_peers = 50
bootstrap_peers = []

[runtime]
chain_id = "unykorn-devnet"
producer_enabled = true
producer_key = "your-ed25519-key-hex"
```

### Configuration Options

| Section | Key | Description | Default |
|:--------|:----|:------------|:--------|
| `node` | `data_dir` | Data storage location | `./data` |
| `node` | `log_level` | Logging verbosity | `info` |
| `network` | `listen_addr` | Network interface | `0.0.0.0` |
| `network` | `listen_port` | P2P port | `30303` |
| `network` | `max_peers` | Max connections | `50` |
| `runtime` | `chain_id` | Network identifier | `unykorn-mainnet` |
| `runtime` | `producer_enabled` | Block production | `false` |

---

## Health Checks

### Manual Health Check

```bash
# Check if node is responding
curl http://localhost:8080/health

# Check block height
curl http://localhost:8080/status
```

### Automated Health Check (Windows)

```powershell
.\scripts\health-check.ps1
```

---

## Next Steps

- 📖 Read the [Architecture Guide](architecture.html)
- 📦 Explore the [Components](components.html)
- 🧪 Run [Soak Tests](testing.html)
- ⚙️ Check the [Operator Runbook](operations.html)
