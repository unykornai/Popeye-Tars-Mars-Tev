# metrics-snapshot.ps1 - Capture metrics from devnet
# Usage: .\scripts\metrics-snapshot.ps1 [-OutputFile <path>]

param(
    [string]$OutputFile = ""
)

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$LogDir = Join-Path $ProjectRoot "logs"
$DataDir = Join-Path $ProjectRoot "dev_data"

$Metrics = @{
    timestamp = (Get-Date -Format "o")
    processes = @()
    storage = @{}
    logs = @{}
}

# Process metrics
$Processes = Get-Process -Name "unykorn" -ErrorAction SilentlyContinue
if ($Processes) {
    foreach ($P in $Processes) {
        $Metrics.processes += @{
            pid = $P.Id
            memory_mb = [math]::Round($P.WorkingSet64 / 1MB, 2)
            cpu_seconds = $P.CPU
            threads = $P.Threads.Count
            handles = $P.HandleCount
        }
    }
}

# Storage metrics
$BlocksDir = Join-Path $DataDir "blocks"
$StateDir = Join-Path $DataDir "state"

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

# Log metrics
$Nodes = @("node-a", "node-b", "node-c")
foreach ($Node in $Nodes) {
    $LogFile = Join-Path $LogDir "$Node.log"
    if (Test-Path $LogFile) {
        $Content = Get-Content $LogFile -ErrorAction SilentlyContinue
        $Metrics.logs[$Node] = @{
            lines = $Content.Count
            size_kb = [math]::Round((Get-Item $LogFile).Length / 1KB, 2)
            errors = ($Content | Select-String -Pattern "error|Error|ERROR" -SimpleMatch).Count
        }
    }
}

# Output
$Json = $Metrics | ConvertTo-Json -Depth 4

if ($OutputFile) {
    $Json | Out-File $OutputFile -Encoding UTF8
    Write-Host "Metrics saved to: $OutputFile" -ForegroundColor Green
} else {
    Write-Output $Json
}
