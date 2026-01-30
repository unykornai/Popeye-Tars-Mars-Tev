# Unykorn L1 Devnet Operator Runbook

## Quick Reference

| Action | Command |
|--------|---------|
| Start devnet | `.\scripts\start-devnet.ps1` |
| Stop devnet | `.\scripts\stop-devnet.ps1` |
| Health check | `.\scripts\health-check.ps1` |
| Collect logs | `.\scripts\collect-logs.ps1` |
| Metrics JSON | `.\scripts\metrics-snapshot.ps1` |

## Node Configuration

| Node | Port | Role | Config |
|------|------|------|--------|
| node-a | 9001 | Producer | `config/node-a.toml` |
| node-b | 9002 | Validator | `config/node-b.toml` |
| node-c | 9003 | Validator | `config/node-c.toml` |

## Startup Procedure

1. **Build the workspace**
   ```powershell
   cargo build --workspace
   ```

2. **Verify configuration files exist**
   ```powershell
   Get-ChildItem config/*.toml
   ```

3. **Start the devnet**
   ```powershell
   .\scripts\start-devnet.ps1
   ```

4. **Verify nodes are running**
   ```powershell
   .\scripts\health-check.ps1
   ```

## Shutdown Procedure

1. **Graceful shutdown**
   ```powershell
   .\scripts\stop-devnet.ps1
   ```

2. **Force kill (if unresponsive)**
   ```powershell
   Get-Process unykorn | Stop-Process -Force
   ```

3. **Collect logs for analysis**
   ```powershell
   .\scripts\collect-logs.ps1 -Tag "shutdown-$(Get-Date -Format 'yyyyMMdd')"
   ```

## Monitoring

### Log Locations

| Log | Path | Contents |
|-----|------|----------|
| node-a.log | `logs/node-a.log` | Producer activity, block creation |
| node-b.log | `logs/node-b.log` | Validator activity |
| node-c.log | `logs/node-c.log` | Validator activity |
| *.err.log | `logs/*.err.log` | Error output |

### Key Metrics

- **Block height**: Check logs for `Height:` messages
- **Memory usage**: `Get-Process unykorn | Select-Object Id,WorkingSet64`
- **Open connections**: `Get-NetTCPConnection -LocalPort 9001,9002,9003`

### Health Indicators

| Metric | Healthy | Warning | Critical |
|--------|---------|---------|----------|
| Block production | Every 3s | >10s gap | No blocks |
| Memory per node | <100MB | 100-500MB | >500MB |
| Error rate | <1/min | 1-10/min | >10/min |

## Troubleshooting

### Node won't start

1. Check port availability:
   ```powershell
   Get-NetTCPConnection -LocalPort 9001 -ErrorAction SilentlyContinue
   ```

2. Check config file syntax:
   ```powershell
   Get-Content config/node-a.toml
   ```

3. Check for stale lock files:
   ```powershell
   Get-ChildItem dev_data -Recurse -Filter "*.lock"
   ```

### No blocks being produced

1. Verify node-a is the producer:
   ```powershell
   Select-String -Path config/node-a.toml -Pattern "producer_enabled"
   ```

2. Check node-a logs for errors:
   ```powershell
   Get-Content logs/node-a.log -Tail 50
   ```

### Nodes not connecting

1. Check all nodes are running:
   ```powershell
   Get-Process unykorn
   ```

2. Verify peer configuration:
   ```powershell
   Select-String -Path config/*.toml -Pattern "peers"
   ```

### Memory leak suspected

1. Take metrics snapshot:
   ```powershell
   .\scripts\metrics-snapshot.ps1 -OutputFile "metrics-$(Get-Date -Format 'HHmmss').json"
   ```

2. Wait 5 minutes, take another snapshot

3. Compare memory values

## Data Recovery

### Corrupted block store

1. Stop all nodes
2. Backup current data:
   ```powershell
   Copy-Item dev_data dev_data.backup -Recurse
   ```
3. Clear blocks directory:
   ```powershell
   Remove-Item dev_data/blocks/* -Force
   ```
4. Restart nodes (will sync from peers)

### Full data reset

1. Stop all nodes
2. Clear all data:
   ```powershell
   Remove-Item dev_data/blocks/* -Force
   Remove-Item dev_data/state/* -Force
   ```
3. Restart nodes (genesis block will be created)

## Escalation

For issues not covered in this runbook:

1. Collect logs with tag
2. Run metrics snapshot
3. Document reproduction steps
4. Check ARCHITECTURE.md for component boundaries
