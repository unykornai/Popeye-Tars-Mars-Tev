# start-devnet.ps1 - Start the 3-node Unykorn devnet
# Usage: .\scripts\start-devnet.ps1

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Binary = Join-Path $ProjectRoot "target\debug\unykorn.exe"
$ConfigDir = Join-Path $ProjectRoot "config"
$LogDir = Join-Path $ProjectRoot "logs"

# Ensure binary exists
if (-not (Test-Path $Binary)) {
    Write-Host "Building workspace..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build --workspace
    Pop-Location
}

# Create log directory
if (-not (Test-Path $LogDir)) {
    New-Item -ItemType Directory -Path $LogDir | Out-Null
}

# Start nodes
$Nodes = @("node-a", "node-b", "node-c")
$Pids = @()

foreach ($Node in $Nodes) {
    $ConfigFile = Join-Path $ConfigDir "$Node.toml"
    $LogFile = Join-Path $LogDir "$Node.log"
    
    if (-not (Test-Path $ConfigFile)) {
        Write-Host "Config not found: $ConfigFile" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Starting $Node..." -ForegroundColor Cyan
    $Process = Start-Process -FilePath $Binary `
        -ArgumentList "--config", $ConfigFile `
        -RedirectStandardOutput $LogFile `
        -RedirectStandardError (Join-Path $LogDir "$Node.err.log") `
        -PassThru `
        -WindowStyle Hidden
    
    $Pids += $Process.Id
    Write-Host "  PID: $($Process.Id)" -ForegroundColor Green
}

# Save PIDs for stop script
$Pids | Out-File (Join-Path $LogDir "pids.txt")

Write-Host ""
Write-Host "Devnet started successfully!" -ForegroundColor Green
Write-Host "  Nodes: $($Nodes -join ', ')"
Write-Host "  Logs:  $LogDir"
Write-Host ""
Write-Host "To stop: .\scripts\stop-devnet.ps1" -ForegroundColor Yellow
