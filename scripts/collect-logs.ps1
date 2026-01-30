# collect-logs.ps1 - Collect and archive devnet logs
# Usage: .\scripts\collect-logs.ps1 [-Tag <string>]

param(
    [string]$Tag = (Get-Date -Format "yyyyMMdd-HHmmss")
)

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$LogDir = Join-Path $ProjectRoot "logs"
$ArchiveDir = Join-Path $ProjectRoot "log_archives"
$ArchiveName = "logs-$Tag.zip"
$ArchivePath = Join-Path $ArchiveDir $ArchiveName

if (-not (Test-Path $LogDir)) {
    Write-Host "No logs directory found" -ForegroundColor Red
    exit 1
}

# Create archive directory
if (-not (Test-Path $ArchiveDir)) {
    New-Item -ItemType Directory -Path $ArchiveDir | Out-Null
}

# Collect system info
$InfoFile = Join-Path $LogDir "system-info.txt"
@"
Unykorn Devnet Log Collection
=============================
Timestamp: $(Get-Date -Format "o")
Tag: $Tag
Hostname: $env:COMPUTERNAME
User: $env:USERNAME

Processes:
$(Get-Process -Name "unykorn" -ErrorAction SilentlyContinue | Format-Table Id, CPU, WorkingSet64 | Out-String)

Network:
$(Get-NetTCPConnection -LocalPort 9001,9002,9003 -ErrorAction SilentlyContinue | Format-Table LocalPort, State, OwningProcess | Out-String)
"@ | Out-File $InfoFile -Encoding UTF8

# Get metrics snapshot
$MetricsFile = Join-Path $LogDir "metrics.json"
& (Join-Path $ProjectRoot "scripts\metrics-snapshot.ps1") -OutputFile $MetricsFile

# Create archive
Write-Host "Creating archive: $ArchiveName" -ForegroundColor Cyan
Compress-Archive -Path "$LogDir\*" -DestinationPath $ArchivePath -Force

# Clean up temp files
Remove-Item $InfoFile -Force -ErrorAction SilentlyContinue
Remove-Item $MetricsFile -Force -ErrorAction SilentlyContinue

$Size = [math]::Round((Get-Item $ArchivePath).Length / 1KB, 2)
Write-Host "Archive created: $ArchivePath ($Size KB)" -ForegroundColor Green
