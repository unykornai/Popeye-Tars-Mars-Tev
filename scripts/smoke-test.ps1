# smoke-test.ps1 - Quick 2-minute smoke test before full soak
# Usage: .\scripts\smoke-test.ps1

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Binary = Join-Path $ProjectRoot "target\release\unykorn.exe"
$ConfigDir = Join-Path $ProjectRoot "config"
$DataDir = Join-Path $ProjectRoot "dev_data"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  UNYKORN L1 - SMOKE TEST (2 minutes)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check binary
Write-Host "[1/6] Checking release binary..." -ForegroundColor Yellow
if (-not (Test-Path $Binary)) {
    Write-Host "  FAIL: Binary not found at $Binary" -ForegroundColor Red
    Write-Host "  Run: cargo build --release --workspace" -ForegroundColor Yellow
    exit 1
}
$BinarySize = [math]::Round((Get-Item $Binary).Length / 1MB, 2)
Write-Host ("  OK: Binary exists ({0} MB)" -f $BinarySize) -ForegroundColor Green

# Check configs
Write-Host "[2/6] Checking configuration files..." -ForegroundColor Yellow
$Configs = @("node-a.toml", "node-b.toml", "node-c.toml")
foreach ($Config in $Configs) {
    $Path = Join-Path $ConfigDir $Config
    if (-not (Test-Path $Path)) {
        Write-Host "  FAIL: Missing $Config" -ForegroundColor Red
        exit 1
    }
}
Write-Host "  OK: All 3 config files present" -ForegroundColor Green

# Clean data
Write-Host "[3/6] Cleaning previous data..." -ForegroundColor Yellow
$DataDir = Join-Path $ProjectRoot "dev_data"
@("node-a", "node-b", "node-c") | ForEach-Object {
    $NodeDataDir = Join-Path $DataDir $_
    New-Item -ItemType Directory -Path "$NodeDataDir\blocks" -Force | Out-Null
    New-Item -ItemType Directory -Path "$NodeDataDir\state" -Force | Out-Null
    Remove-Item "$NodeDataDir\blocks\*" -Force -ErrorAction SilentlyContinue
    Remove-Item "$NodeDataDir\state\*" -Force -ErrorAction SilentlyContinue
}
Write-Host "  OK: Data directories cleaned" -ForegroundColor Green

# Start single node first
Write-Host "[4/6] Testing single node startup..." -ForegroundColor Yellow
$TempLog = [System.IO.Path]::GetTempFileName()
$TempErr = [System.IO.Path]::GetTempFileName()

$NodeA = Start-Process -FilePath $Binary `
    -ArgumentList "--config", (Join-Path $ConfigDir "node-a.toml") `
    -RedirectStandardOutput $TempLog `
    -RedirectStandardError $TempErr `
    -PassThru `
    -WindowStyle Hidden

Start-Sleep -Seconds 3

$StillRunning = Get-Process -Id $NodeA.Id -ErrorAction SilentlyContinue
if (-not $StillRunning) {
    Write-Host "  FAIL: Node crashed immediately" -ForegroundColor Red
    Write-Host "  Stderr:" -ForegroundColor Yellow
    Get-Content $TempErr | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    exit 1
}

# Check for startup message
$LogContent = Get-Content $TempLog -ErrorAction SilentlyContinue
if ($LogContent -match "Starting Unykorn") {
    Write-Host ("  OK: Node started successfully (PID {0})" -f $NodeA.Id) -ForegroundColor Green
} else {
    Write-Host "  WARNING: Node running but no startup message yet" -ForegroundColor Yellow
}

# Stop single node
Stop-Process -Id $NodeA.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Start all 3 nodes
Write-Host "[5/6] Testing 3-node devnet (running 2 minutes)..." -ForegroundColor Yellow
$Nodes = @("node-a", "node-b", "node-c")
$Processes = @{}
$LogDir = Join-Path $ProjectRoot "logs"
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
Remove-Item "$LogDir\*" -Force -ErrorAction SilentlyContinue

foreach ($Node in $Nodes) {
    $ConfigFile = Join-Path $ConfigDir "$Node.toml"
    $StdOut = Join-Path $LogDir "$Node.log"
    $StdErr = Join-Path $LogDir "$Node.err.log"
    
    $Process = Start-Process -FilePath $Binary `
        -ArgumentList "--config", $ConfigFile `
        -RedirectStandardOutput $StdOut `
        -RedirectStandardError $StdErr `
        -PassThru `
        -WindowStyle Hidden
    
    $Processes[$Node] = $Process.Id
    Write-Host ("  Started {0} (PID {1})" -f $Node, $Process.Id) -ForegroundColor Gray
}

# Wait and monitor
$TestDuration = 120  # 2 minutes
$CheckInterval = 10
$Elapsed = 0
$Running = 3

while ($Elapsed -lt $TestDuration) {
    Start-Sleep -Seconds $CheckInterval
    $Elapsed += $CheckInterval
    
    # Check processes
    $Running = 0
    foreach ($Node in $Nodes) {
        $Proc = Get-Process -Id $Processes[$Node] -ErrorAction SilentlyContinue
        if ($Proc) { $Running++ }
    }
    
    # Check blocks (from node-a which is the producer)
    $BlockCount = (Get-ChildItem "$DataDir\node-a\blocks" -File -ErrorAction SilentlyContinue).Count
    
    # Memory
    $TotalMem = 0
    foreach ($Node in $Nodes) {
        $Proc = Get-Process -Id $Processes[$Node] -ErrorAction SilentlyContinue
        if ($Proc) { $TotalMem += $Proc.WorkingSet64 }
    }
    $TotalMemMB = [math]::Round($TotalMem / 1MB, 0)
    
    Write-Host ("  [{0:D3}s] Nodes: {1}/3 | Blocks: {2} | Memory: {3} MB" -f $Elapsed, $Running, $BlockCount, $TotalMemMB) -ForegroundColor Gray
    
    if ($Running -lt 3) {
        Write-Host "  FAIL: Node(s) crashed during smoke test" -ForegroundColor Red
        break
    }
}

# Stop all nodes
foreach ($Node in $Nodes) {
    Stop-Process -Id $Processes[$Node] -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 1

# Verify results
Write-Host "[6/6] Analyzing results..." -ForegroundColor Yellow

$FinalBlockCount = (Get-ChildItem "$DataDir\node-a\blocks" -File -ErrorAction SilentlyContinue).Count
$ExpectedBlocks = [math]::Floor($TestDuration / 3)  # 3 second block time
$BlockRatio = if ($ExpectedBlocks -gt 0) { [math]::Round($FinalBlockCount / $ExpectedBlocks * 100, 1) } else { 0 }

# Check for errors in logs
$ErrorCount = 0
foreach ($Node in $Nodes) {
    $LogFile = Join-Path $LogDir "$Node.log"
    if (Test-Path $LogFile) {
        $Errors = (Get-Content $LogFile | Select-String -Pattern "panic|PANIC" -SimpleMatch).Count
        $ErrorCount += $Errors
    }
}

# Results
Write-Host ""
$BlockColor = if ($BlockRatio -ge 80) { "Green" } elseif ($BlockRatio -ge 50) { "Yellow" } else { "Red" }
Write-Host ("  Blocks produced: {0} (expected ~{1}, {2}%)" -f $FinalBlockCount, $ExpectedBlocks, $BlockRatio) -ForegroundColor $BlockColor
$ErrorColor = if ($ErrorCount -eq 0) { "Green" } else { "Red" }
Write-Host ("  Panics in logs:  {0}" -f $ErrorCount) -ForegroundColor $ErrorColor

$Pass = ($Running -eq 3) -and ($BlockRatio -ge 50) -and ($ErrorCount -eq 0)

Write-Host ""
if ($Pass) {
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "         SMOKE TEST: PASS" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Ready for full soak test:" -ForegroundColor Green
    Write-Host "  .\scripts\run-soak-test.ps1 -Duration 24" -ForegroundColor Cyan
} else {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "         SMOKE TEST: FAIL" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Check logs in: $LogDir" -ForegroundColor Yellow
    exit 1
}

# Cleanup temp files
Remove-Item $TempLog -Force -ErrorAction SilentlyContinue
Remove-Item $TempErr -Force -ErrorAction SilentlyContinue
