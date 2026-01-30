# run-soak-test.ps1 - Automated soak test with metrics collection
# Usage: .\scripts\run-soak-test.ps1 [-Duration 24] [-MetricsInterval 300]

param(
    [double]$Duration = 24,
    [int]$MetricsInterval = 300,
    [string]$Tag = (Get-Date -Format "yyyyMMdd-HHmmss")
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Binary = Join-Path $ProjectRoot "target\release\unykorn.exe"
$ConfigDir = Join-Path $ProjectRoot "config"
$LogDir = Join-Path $ProjectRoot "logs"
$SoakDir = Join-Path $ProjectRoot "soak_results\soak-$Tag"
$MetricsDir = Join-Path $SoakDir "metrics"
$DataDir = Join-Path $ProjectRoot "dev_data"
$Nodes = @("node-a", "node-b", "node-c")

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  UNYKORN L1 - SOAK TEST RUNNER" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ("  Duration:         {0} hours" -f $Duration) -ForegroundColor Cyan
Write-Host ("  Metrics Interval: {0} seconds" -f $MetricsInterval) -ForegroundColor Cyan
Write-Host ("  Tag:              {0}" -f $Tag) -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $Binary)) {
    Write-Host "ERROR: Release binary not found." -ForegroundColor Red
    exit 1
}

New-Item -ItemType Directory -Path $SoakDir -Force | Out-Null
New-Item -ItemType Directory -Path $MetricsDir -Force | Out-Null
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

Write-Host "Cleaning previous data..." -ForegroundColor Yellow
foreach ($Node in $Nodes) {
    $NodeDataDir = Join-Path $DataDir $Node
    New-Item -ItemType Directory -Path "$NodeDataDir\blocks" -Force | Out-Null
    New-Item -ItemType Directory -Path "$NodeDataDir\state" -Force | Out-Null
    Remove-Item "$NodeDataDir\blocks\*" -Force -ErrorAction SilentlyContinue
    Remove-Item "$NodeDataDir\state\*" -Force -ErrorAction SilentlyContinue
}
Remove-Item "$LogDir\*" -Force -ErrorAction SilentlyContinue

$TestParams = @{
    start_time = (Get-Date -Format "o")
    duration_hours = $Duration
    metrics_interval_seconds = $MetricsInterval
    tag = $Tag
    hostname = $env:COMPUTERNAME
    expected_blocks = [math]::Floor($Duration * 3600 / 3)
}
$TestParams | ConvertTo-Json | Out-File (Join-Path $SoakDir "test_params.json") -Encoding UTF8

Write-Host "Starting 3-node devnet..." -ForegroundColor Cyan
$Processes = @{}

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
    Write-Host ("  Started {0} (PID {1})" -f $Node, $Process.Id) -ForegroundColor Green
}

$Processes | ConvertTo-Json | Out-File (Join-Path $LogDir "pids.json") -Encoding UTF8

Start-Sleep -Seconds 5

$AllRunning = $true
foreach ($Node in $Nodes) {
    $Proc = Get-Process -Id $Processes[$Node] -ErrorAction SilentlyContinue
    if (-not $Proc) {
        Write-Host ("  ERROR: {0} not running!" -f $Node) -ForegroundColor Red
        $AllRunning = $false
    }
}

if (-not $AllRunning) {
    Write-Host "Startup failed." -ForegroundColor Red
    exit 1
}

$EndTime = (Get-Date).AddHours($Duration)
$StartTime = Get-Date
$MetricsCount = 0

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host ("  SOAK TEST RUNNING - End: {0}" -f $EndTime) -ForegroundColor Green
Write-Host "  Press Ctrl+C to stop early" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

function Get-Metrics {
    param([int]$Index)
    
    $ElapsedSeconds = ((Get-Date) - $StartTime).TotalSeconds
    $Metrics = @{
        index = $Index
        timestamp = (Get-Date -Format "o")
        elapsed_hours = [math]::Round($ElapsedSeconds / 3600, 2)
        processes = @()
        storage = @{}
        errors = @{}
    }
    
    foreach ($Node in $Nodes) {
        $NodePid = $Processes[$Node]
        $Proc = Get-Process -Id $NodePid -ErrorAction SilentlyContinue
        if ($Proc) {
            $Metrics.processes += @{
                node = $Node
                pid = $NodePid
                memory_mb = [math]::Round($Proc.WorkingSet64 / 1MB, 2)
                cpu_seconds = $Proc.CPU
                threads = $Proc.Threads.Count
                running = $true
            }
        } else {
            $Metrics.processes += @{
                node = $Node
                pid = $NodePid
                running = $false
            }
        }
    }
    
    $BlocksDir = Join-Path $DataDir "node-a\blocks"
    $StateDir = Join-Path $DataDir "node-a\state"
    
    if (Test-Path $BlocksDir) {
        $BlockFiles = Get-ChildItem $BlocksDir -File -ErrorAction SilentlyContinue
        $Metrics.storage["blocks_count"] = $BlockFiles.Count
        $Metrics.storage["blocks_size_kb"] = [math]::Round(($BlockFiles | Measure-Object -Sum Length).Sum / 1KB, 2)
    }
    
    if (Test-Path $StateDir) {
        $StateFiles = Get-ChildItem $StateDir -File -ErrorAction SilentlyContinue
        $Metrics.storage["state_count"] = $StateFiles.Count
        $Metrics.storage["state_size_kb"] = [math]::Round(($StateFiles | Measure-Object -Sum Length).Sum / 1KB, 2)
    }
    
    foreach ($Node in $Nodes) {
        $LogFile = Join-Path $LogDir "$Node.log"
        if (Test-Path $LogFile) {
            $Content = Get-Content $LogFile -ErrorAction SilentlyContinue
            $Metrics.errors[$Node] = ($Content | Select-String -Pattern "error|panic" -SimpleMatch).Count
        }
    }
    
    $MetricsFile = Join-Path $MetricsDir ("metrics_{0:D4}.json" -f $Index)
    $Metrics | ConvertTo-Json -Depth 4 | Out-File $MetricsFile -Encoding UTF8
    
    return $Metrics
}

try {
    while ((Get-Date) -lt $EndTime) {
        $MetricsCount++
        $Metrics = Get-Metrics -Index $MetricsCount
        
        $ElapsedSeconds = ((Get-Date) - $StartTime).TotalSeconds
        $TotalSeconds = $Duration * 3600
        $ProgressPct = [math]::Round(($ElapsedSeconds / $TotalSeconds) * 100, 1)
        $Remaining = $EndTime - (Get-Date)
        
        $MemTotal = 0
        $Metrics.processes | Where-Object { $_.running -and $_.memory_mb } | ForEach-Object { $MemTotal += $_.memory_mb }
        $BlockCount = $Metrics.storage["blocks_count"]
        $ErrorTotal = 0
        $Metrics.errors.Values | ForEach-Object { $ErrorTotal += $_ }
        $RunningCount = ($Metrics.processes | Where-Object { $_.running }).Count
        
        $StatusColor = if ($RunningCount -eq 3) { "Gray" } else { "Red" }
        Write-Host ("[{0:HH:mm:ss}] {1}% | Nodes: {2}/3 | Blocks: {3} | Mem: {4} MB | Errors: {5} | Left: {6}" -f `
            (Get-Date), $ProgressPct, $RunningCount, $BlockCount, [math]::Round($MemTotal, 0), $ErrorTotal, `
            $Remaining.ToString("hh\:mm\:ss")) -ForegroundColor $StatusColor
        
        if ($RunningCount -lt 3) {
            $Failed = $Metrics.processes | Where-Object { -not $_.running }
            foreach ($F in $Failed) {
                Write-Host ("  WARNING: {0} stopped!" -f $F.node) -ForegroundColor Red
            }
        }
        
        Start-Sleep -Seconds $MetricsInterval
    }
} finally {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "  COLLECTING FINAL METRICS..." -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    
    $FinalMetrics = Get-Metrics -Index 9999
    $FinalMetrics | ConvertTo-Json -Depth 4 | Out-File (Join-Path $SoakDir "final_metrics.json") -Encoding UTF8
    
    Write-Host "Stopping nodes..." -ForegroundColor Yellow
    foreach ($Node in $Nodes) {
        $NodePid = $Processes[$Node]
        $Proc = Get-Process -Id $NodePid -ErrorAction SilentlyContinue
        if ($Proc) {
            Stop-Process -Id $NodePid -Force -ErrorAction SilentlyContinue
            Write-Host ("  Stopped {0}" -f $Node) -ForegroundColor Gray
        }
    }
    
    Copy-Item "$LogDir\*" $SoakDir -Force -ErrorAction SilentlyContinue
    
    $ElapsedSeconds = ((Get-Date) - $StartTime).TotalSeconds
    $FinalMemory = 0
    $FinalMetrics.processes | Where-Object { $_.running -and $_.memory_mb } | ForEach-Object { $FinalMemory += $_.memory_mb }
    $FinalStorage = 0
    if ($FinalMetrics.storage.blocks_size_kb) { $FinalStorage += $FinalMetrics.storage.blocks_size_kb }
    if ($FinalMetrics.storage.state_size_kb) { $FinalStorage += $FinalMetrics.storage.state_size_kb }
    $TotalErrors = 0
    $FinalMetrics.errors.Values | ForEach-Object { $TotalErrors += $_ }
    $NodeFailures = ($FinalMetrics.processes | Where-Object { -not $_.running }).Count
    
    $Summary = @{
        test_completed = (Get-Date -Format "o")
        duration_actual_hours = [math]::Round($ElapsedSeconds / 3600, 2)
        metrics_collected = $MetricsCount
        final_block_count = $FinalMetrics.storage.blocks_count
        expected_block_count = [math]::Floor($ElapsedSeconds / 3)
        block_rate_per_hour = if ($ElapsedSeconds -gt 0) { [math]::Round($FinalMetrics.storage.blocks_count / ($ElapsedSeconds / 3600), 2) } else { 0 }
        final_memory_mb = $FinalMemory
        final_storage_kb = $FinalStorage
        total_errors = $TotalErrors
        node_failures = $NodeFailures
        pass = ($NodeFailures -eq 0) -and ($TotalErrors -lt 100)
    }
    
    $Summary | ConvertTo-Json -Depth 2 | Out-File (Join-Path $SoakDir "summary.json") -Encoding UTF8
    
    Write-Host ""
    $ResultColor = if ($Summary.pass) { "Green" } else { "Red" }
    Write-Host "========================================" -ForegroundColor $ResultColor
    Write-Host "         SOAK TEST RESULTS" -ForegroundColor $ResultColor
    Write-Host "========================================" -ForegroundColor $ResultColor
    Write-Host ("  Duration:       {0} hours" -f $Summary.duration_actual_hours) -ForegroundColor White
    Write-Host ("  Blocks:         {0} (expected {1})" -f $Summary.final_block_count, $Summary.expected_block_count) -ForegroundColor White
    Write-Host ("  Block Rate:     {0}/hour" -f $Summary.block_rate_per_hour) -ForegroundColor White
    Write-Host ("  Memory:         {0} MB" -f [math]::Round($Summary.final_memory_mb, 0)) -ForegroundColor White
    Write-Host ("  Storage:        {0} KB" -f [math]::Round($Summary.final_storage_kb, 0)) -ForegroundColor White
    Write-Host ("  Errors:         {0}" -f $Summary.total_errors) -ForegroundColor $(if ($Summary.total_errors -gt 0) { "Yellow" } else { "White" })
    Write-Host ("  Node Failures:  {0}" -f $Summary.node_failures) -ForegroundColor $(if ($Summary.node_failures -gt 0) { "Red" } else { "White" })
    Write-Host "----------------------------------------" -ForegroundColor $ResultColor
    Write-Host ("  RESULT:         {0}" -f $(if ($Summary.pass) { "PASS" } else { "FAIL" })) -ForegroundColor $ResultColor
    Write-Host "========================================" -ForegroundColor $ResultColor
    Write-Host ""
    Write-Host "Results saved to: $SoakDir" -ForegroundColor Cyan
}
