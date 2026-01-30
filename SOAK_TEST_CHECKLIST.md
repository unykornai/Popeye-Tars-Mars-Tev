# Unykorn L1 Soak Test Checklist

## Pre-Test Setup

- [ ] Fresh build: `cargo build --release --workspace`
- [ ] Clean data directories: `Remove-Item dev_data/blocks/*, dev_data/state/* -Force`
- [ ] Verify config files present in `config/`
- [ ] Baseline metrics snapshot saved
- [ ] Log directory cleared: `Remove-Item logs/* -Force`

## Test Configuration

| Parameter | Value |
|-----------|-------|
| Duration | 24 hours |
| Block interval | 3 seconds |
| Expected blocks | ~28,800 |
| Nodes | 3 (1 producer, 2 validators) |
| Synthetic load | None (block production only) |

## Hourly Checks

### Hour 0 (Start)
- [ ] All 3 nodes started
- [ ] Health check passes
- [ ] Block production confirmed in node-a.log
- [ ] Memory baseline recorded

### Hour 1
- [ ] Metrics snapshot: `.\scripts\metrics-snapshot.ps1 -OutputFile metrics-h01.json`
- [ ] Error count in logs: _____
- [ ] Block height: _____
- [ ] Memory (MB) - node-a: _____ node-b: _____ node-c: _____

### Hour 6
- [ ] Metrics snapshot saved
- [ ] No memory growth trend
- [ ] Block height matches expected (~7,200)
- [ ] All nodes still responding

### Hour 12
- [ ] Metrics snapshot saved
- [ ] Memory stable (±10% of baseline)
- [ ] Block height matches expected (~14,400)
- [ ] Log file sizes reasonable (<100MB each)

### Hour 18
- [ ] Metrics snapshot saved
- [ ] No degradation in block timing
- [ ] Error rate stable or declining

### Hour 24 (End)
- [ ] Final metrics snapshot
- [ ] Collect logs with tag: `.\scripts\collect-logs.ps1 -Tag "soak-24h"`
- [ ] Stop devnet gracefully

## Pass/Fail Criteria

### PASS Conditions
- [ ] All 3 nodes running for full 24 hours
- [ ] Block production never stopped
- [ ] Memory stayed under 500MB per node
- [ ] No panic/crash messages in logs
- [ ] Block height within 5% of expected

### FAIL Conditions
- Any node crashed
- Block production gap > 1 minute
- Memory exceeded 500MB
- Unhandled errors in logs
- Data corruption detected

## Stress Variants

### Variant A: Transaction Load
- Inject 100 tx/block using test harness
- Expected: Higher memory, no crashes

### Variant B: Network Partition
- Kill node-b at hour 12
- Restart at hour 13
- Expected: Catches up within 10 minutes

### Variant C: Disk Pressure
- Fill disk to 90% capacity
- Expected: Graceful errors, no corruption

## Post-Test Analysis

1. **Memory trend**: Plot memory over time from metrics files
2. **Block timing**: Analyze log timestamps for gaps
3. **Error categorization**: Group errors by type
4. **Resource usage**: Peak vs average CPU/memory

## Results Summary

| Metric | Expected | Actual | Pass/Fail |
|--------|----------|--------|-----------|
| Duration | 24h | | |
| Final block height | ~28,800 | | |
| Node crashes | 0 | | |
| Memory growth | <10% | | |
| Error count | <100 | | |

## Sign-off

**Tester**: _____________________

**Date**: _____________________

**Result**: PASS / FAIL

**Notes**:



