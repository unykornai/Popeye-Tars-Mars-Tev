---
layout: default
title: Operations
nav_order: 6
description: "Unykorn L1 Operator Runbook"
---

# 🛠️ Operations
{: .no_toc }

Operator runbook for managing Unykorn L1 nodes.
{: .fs-6 .fw-300 }

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Devnet Operations

### Starting the Devnet

```powershell
.\scripts\start-devnet.ps1
```

This starts a 3-node local network:
- **Node A** (port 30303) - Block Producer
- **Node B** (port 30304) - Validator
- **Node C** (port 30305) - Validator

### Stopping the Devnet

```powershell
.\scripts\stop-devnet.ps1
```

### Health Check

```powershell
.\scripts\health-check.ps1
```

Expected output:
```
✅ Node A: Healthy (height: 1234)
✅ Node B: Healthy (height: 1234)
✅ Node C: Healthy (height: 1234)
```

---

## Monitoring

### Metrics Snapshot

```powershell
.\scripts\metrics-snapshot.ps1 -OutputFile metrics.json
```

### Metrics Fields

| Metric | Description |
|:-------|:------------|
| `block_height` | Current chain height |
| `peer_count` | Connected peers |
| `mempool_size` | Pending transactions |
| `memory_mb` | Memory usage |
| `uptime_seconds` | Node uptime |

### Log Collection

```powershell
.\scripts\collect-logs.ps1 -Tag "incident-2026-02-02"
```

---

## Troubleshooting

### Node Won't Start

**Symptom:** Node fails to start with port binding error.

**Solution:**
```powershell
# Check for existing processes
Get-Process | Where-Object { $_.ProcessName -like "*unykorn*" }

# Kill stale processes
Stop-Process -Name "unykorn" -Force
```

### Memory Growth

**Symptom:** Memory usage growing over time.

**Diagnosis:**
```powershell
# Take memory snapshots
.\scripts\metrics-snapshot.ps1 -OutputFile mem-t0.json
Start-Sleep -Seconds 3600
.\scripts\metrics-snapshot.ps1 -OutputFile mem-t1.json

# Compare
Compare-Object (Get-Content mem-t0.json) (Get-Content mem-t1.json)
```

### Block Production Stopped

**Symptom:** No new blocks being produced.

**Diagnosis:**
1. Check producer node logs
2. Verify producer is enabled in config
3. Check for panic messages

```powershell
Select-String -Path "logs\node-a.log" -Pattern "panic|error" | Select-Object -Last 20
```

### Peers Not Connecting

**Symptom:** Nodes not discovering each other.

**Diagnosis:**
1. Verify mDNS is working
2. Check firewall rules
3. Verify all nodes on same network

```powershell
# Check peer count
.\scripts\health-check.ps1 -Verbose
```

---

## Backup & Recovery

### Backup State

```powershell
# Stop the node first
.\scripts\stop-devnet.ps1

# Backup data directory
Copy-Item -Path "data\node-a" -Destination "backup\node-a-$(Get-Date -Format 'yyyyMMdd')" -Recurse
```

### Restore from Backup

```powershell
# Stop the node
.\scripts\stop-devnet.ps1

# Restore data
Remove-Item -Path "data\node-a" -Recurse -Force
Copy-Item -Path "backup\node-a-20260202" -Destination "data\node-a" -Recurse

# Restart
.\scripts\start-devnet.ps1
```

### Recovery from Crash

TAR provides crash-safe persistence. After a crash:

1. The node will automatically recover on restart
2. State will be restored from the last committed block
3. Pending mempool transactions will be lost

---

## Performance Tuning

### Resource Limits

| Resource | Recommended | Maximum |
|:---------|:------------|:--------|
| Memory | 200MB | 500MB |
| Disk | 50MB/day | - |
| CPU | 5% | 25% |

### Configuration Tuning

```toml
[network]
max_peers = 50          # Reduce for lower memory
connection_timeout = 10  # Increase for slow networks

[runtime]
block_interval = 3      # Seconds between blocks
max_tx_per_block = 1000 # Transaction limit
```

---

## Security

### Key Management

- Producer keys should be stored securely
- Never commit keys to version control
- Use environment variables for sensitive data

### Firewall Rules

```powershell
# Allow P2P port
New-NetFirewallRule -DisplayName "Unykorn P2P" -Direction Inbound -Port 30303 -Protocol TCP -Action Allow
```

### Log Sanitization

Logs may contain sensitive information. Before sharing:

```powershell
# Remove potential secrets
Get-Content logs\node-a.log | 
  ForEach-Object { $_ -replace '[0-9a-f]{64}', '[REDACTED]' } |
  Set-Content logs\node-a-sanitized.log
```

---

## Maintenance Windows

### Planned Maintenance

1. Announce maintenance window
2. Stop accepting new transactions
3. Wait for mempool to drain
4. Stop nodes gracefully
5. Perform maintenance
6. Restart nodes
7. Verify health
8. Resume operations

### Emergency Procedures

1. Stop all nodes immediately
2. Collect logs and metrics
3. Analyze root cause
4. Apply fix
5. Restore from backup if needed
6. Restart with monitoring
