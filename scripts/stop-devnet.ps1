# stop-devnet.ps1 - Stop the Unykorn devnet
# Usage: .\scripts\stop-devnet.ps1

$ErrorActionPreference = "Continue"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$LogDir = Join-Path $ProjectRoot "logs"
$PidFile = Join-Path $LogDir "pids.txt"

# Stop by PID file if exists
if (Test-Path $PidFile) {
    $Pids = Get-Content $PidFile
    foreach ($Pid in $Pids) {
        if ($Pid -match '^\d+$') {
            $Process = Get-Process -Id $Pid -ErrorAction SilentlyContinue
            if ($Process) {
                Write-Host "Stopping PID $Pid ($($Process.ProcessName))..." -ForegroundColor Yellow
                Stop-Process -Id $Pid -Force
            }
        }
    }
    Remove-Item $PidFile -Force
}

# Also stop any stray unykorn processes
$Stray = Get-Process -Name "unykorn" -ErrorAction SilentlyContinue
if ($Stray) {
    Write-Host "Stopping $($Stray.Count) stray unykorn process(es)..." -ForegroundColor Yellow
    $Stray | Stop-Process -Force
}

Write-Host "Devnet stopped." -ForegroundColor Green
